use std::{
    collections::{BTreeMap, VecDeque},
    hash::Hash,
};

use super::messages::{Action, BroadcastEvent, BroadcastEventData, Changed, Event, FetchChangedError, KvEvent, NetEvent, RpcEvent, RpcEventData, RpcReq, RpcRes, Slot, SnapshotData, Version};

pub struct LocalStore<N, K, V> {
    pub(crate) session_id: u64,
    pub(crate) slots: BTreeMap<K, Slot<V>>,
    changeds: BTreeMap<Version, Changed<K, V>>,
    max_changeds: usize,
    compose_max_pkts: usize,
    version: Version,
    outs: VecDeque<Event<N, K, V>>,
}

impl<N, K, V> LocalStore<N, K, V>
where
    K: Hash + Ord + Eq + Clone,
    V: Eq + Clone,
{
    pub fn new(session_id: u64, max_changeds: usize, compose_max_pkts: usize) -> Self {
        let compose_max_pkts = compose_max_pkts.max(1);
        LocalStore {
            session_id,
            slots: BTreeMap::new(),
            changeds: BTreeMap::new(),
            max_changeds,
            compose_max_pkts,
            version: Version(0),
            outs: VecDeque::new(),
        }
    }

    pub fn on_tick(&mut self) {
        self.outs.push_back(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
            session_id: self.session_id,
            data: BroadcastEventData::Version(self.version),
        })));
    }

    pub fn set(&mut self, key: K, value: V) {
        let Some(version) = self.version.checked_next() else {
            return;
        };
        self.version = version;
        let changed = Changed {
            key: key.clone(),
            version,
            action: Action::Set(value.clone()),
        };
        self.changeds.insert(version, changed.clone());
        self.outs.push_back(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
            session_id: self.session_id,
            data: BroadcastEventData::Changed(changed),
        })));
        self.outs.push_back(Event::KvEvent(KvEvent::Set(None, key.clone(), value.clone())));
        while self.changeds.len() > self.max_changeds {
            self.changeds.pop_first();
        }
        self.slots.insert(key, Slot::new(value, version));
    }

    pub fn del(&mut self, key: K) {
        if !self.slots.contains_key(&key) {
            return;
        }
        let Some(version) = self.version.checked_next() else {
            return;
        };
        self.version = version;
        self.slots.remove(&key);
        let changed = Changed {
            key: key.clone(),
            version,
            action: Action::Del,
        };
        self.changeds.insert(self.version, changed.clone());
        self.outs.push_back(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
            session_id: self.session_id,
            data: BroadcastEventData::Changed(changed),
        })));
        self.outs.push_back(Event::KvEvent(KvEvent::Del(None, key.clone())));
        while self.changeds.len() > self.max_changeds {
            self.changeds.pop_first();
        }
    }

    pub fn on_rpc_req(&mut self, from_node: N, req: RpcReq<K>) {
        match req {
            RpcReq::FetchChanged { from, count } => {
                let res = RpcRes::FetchChanged(self.changeds_from_to(from, count));
                self.outs.push_back(Event::NetEvent(NetEvent::Unicast(
                    from_node,
                    RpcEvent {
                        session_id: self.session_id,
                        data: RpcEventData::RpcRes(res),
                    },
                )));
            }
            RpcReq::FetchSnapshot { from, max_version, max_items, req_id } => {
                let snapshot_version = max_version.unwrap_or(self.version).min(self.version);
                let res = RpcRes::FetchSnapshot(self.snapshot(from, Some(snapshot_version), max_items), snapshot_version, req_id);
                self.outs.push_back(Event::NetEvent(NetEvent::Unicast(
                    from_node,
                    RpcEvent {
                        session_id: self.session_id,
                        data: RpcEventData::RpcRes(res),
                    },
                )));
            }
        }
    }

    fn changeds_from_to(&self, from: Version, count: u64) -> Result<Vec<Changed<K, V>>, FetchChangedError> {
        let count = count.min(self.compose_max_pkts as u64);
        if count == 0 {
            return Err(FetchChangedError::MissingData);
        }
        let to = Version(from.0.checked_add(count).ok_or(FetchChangedError::MissingData)?);
        let first = self.changeds.first_key_value().ok_or(FetchChangedError::MissingData)?.0;
        let last = self.changeds.last_key_value().ok_or(FetchChangedError::MissingData)?.0;
        let after_last = Version(last.0.checked_add(1).ok_or(FetchChangedError::MissingData)?);
        if to > after_last || from < *first {
            return Err(FetchChangedError::MissingData);
        }
        Ok(self.changeds.range(from..to).map(|(_, v)| v.clone()).collect())
    }

    fn snapshot(&self, from: Option<K>, max_version: Option<Version>, max_items: u64) -> Option<SnapshotData<K, V>> {
        if self.slots.is_empty() {
            return None;
        }

        let from = from.or_else(|| self.slots.first_key_value().map(|(k, _)| k.clone()))?;
        let max_version = max_version.unwrap_or(self.version);
        let max_items = usize::try_from(max_items).unwrap_or(usize::MAX);
        if max_items == 0 {
            return Some(SnapshotData {
                slots: vec![],
                skipped_newer: vec![],
                next_key: None,
            });
        }
        let scan_limit = self.compose_max_pkts.min(max_items);
        let mut slots = Vec::new();
        let mut skipped_newer = Vec::new();
        let mut next_key = None;
        let mut scanned = 0;

        for (key, slot) in self.slots.range(from.clone()..) {
            if scanned >= scan_limit {
                next_key = Some(key.clone());
                break;
            }
            scanned += 1;

            if slot.version <= max_version {
                slots.push((key.clone(), slot.clone()));
            } else {
                skipped_newer.push((key.clone(), slot.version));
            }
        }

        if next_key.is_none() && max_version < self.version {
            for (v, changed) in self.changeds.range(Version(max_version.0 + 1)..) {
                if &changed.key < &from {
                    continue;
                }
                if !slots.iter().any(|(k, _)| k == &changed.key)
                    && !skipped_newer.iter().any(|(k, _)| k == &changed.key)
                {
                    skipped_newer.push((changed.key.clone(), *v));
                }
            }
        }

        Some(SnapshotData { slots, skipped_newer, next_key })
    }

    pub fn pop_out(&mut self) -> Option<Event<N, K, V>> {
        self.outs.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_works() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 3);

        assert_eq!(store.snapshot(None, None, 3), None);

        store.set(1, 101);

        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
                session_id: 1,
                data: BroadcastEventData::Changed(Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(101)
                })
            })))
        );
        assert_eq!(store.pop_out(), Some(Event::KvEvent(KvEvent::Set(None, 1, 101))));
        assert_eq!(store.pop_out(), None);

        assert_eq!(
            store.snapshot(None, None, 3),
            Some(SnapshotData {
                slots: vec![(1, Slot::new(101, Version(1)))],
                skipped_newer: vec![],
                next_key: None,
            })
        );

        store.del(1);

        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
                session_id: 1,
                data: BroadcastEventData::Changed(Changed {
                    key: 1,
                    version: Version(2),
                    action: Action::Del
                })
            })))
        );
        assert_eq!(store.pop_out(), Some(Event::KvEvent(KvEvent::Del(None, 1))));
        assert_eq!(store.pop_out(), None);

        assert_eq!(
            store.changeds_from_to(Version(1), 2),
            Ok(vec![
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(101)
                },
                Changed {
                    key: 1,
                    version: Version(2),
                    action: Action::Del
                }
            ])
        );

        assert_eq!(store.snapshot(None, None, 3), None);
    }

    #[test]
    fn snapshot_multiple_pkts() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 2, 2);
        for i in 1..=10 {
            store.set(i, i);
        }

        assert_eq!(
            store.snapshot(None, None, 2),
            Some(SnapshotData {
                slots: vec![(1, Slot::new(1, Version(1))), (2, Slot::new(2, Version(2)))],
                skipped_newer: vec![],
                next_key: Some(3),
            })
        );

        assert_eq!(
            store.snapshot(Some(3), Some(Version(10)), 2),
            Some(SnapshotData {
                slots: vec![(3, Slot::new(3, Version(3))), (4, Slot::new(4, Version(4)))],
                skipped_newer: vec![],
                next_key: Some(5),
            })
        );

        // last pkt
        assert_eq!(
            store.snapshot(Some(9), Some(Version(10)), 2),
            Some(SnapshotData {
                slots: vec![(9, Slot::new(9, Version(9))), (10, Slot::new(10, Version(10)))],
                skipped_newer: vec![],
                next_key: None,
            })
        );
    }

    #[test]
    fn auto_clear_changeds() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 2, 2);
        for i in 0..3 {
            store.set(i, i);
        }
        assert_eq!(store.changeds.len(), 2);
        assert_eq!(store.changeds_from_to(Version(1), 3), Err(FetchChangedError::MissingData));
        assert_eq!(
            store.changeds_from_to(Version(2), 2),
            Ok(vec![
                Changed {
                    key: 1,
                    version: Version(2),
                    action: Action::Set(1)
                },
                Changed {
                    key: 2,
                    version: Version(3),
                    action: Action::Set(2)
                }
            ])
        );
    }

    #[test]
    fn tick_broadcasts_version() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.on_tick();
        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Broadcast(BroadcastEvent {
                session_id: 1,
                data: BroadcastEventData::Version(Version(0))
            })))
        );
    }

    #[test]
    fn fetch_changed_with_overflowing_from_version_must_not_panic() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.set(1, 1);
        while store.pop_out().is_some() {}

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.on_rpc_req(2, RpcReq::FetchChanged { from: Version(u64::MAX), count: 1 });
        }));

        assert!(result.is_ok(), "untrusted FetchChanged version arithmetic must not panic or wrap");
        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                2,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcRes(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)))
                }
            )))
        );
    }

    #[test]
    fn zero_changed_batch_size_must_not_return_empty_success() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 0);
        store.set(1, 1);
        while store.pop_out().is_some() {}

        store.on_rpc_req(2, RpcReq::FetchChanged { from: Version(1), count: 1 });

        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                2,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcRes(RpcRes::FetchChanged(Ok(vec![Changed {
                        key: 1,
                        version: Version(1),
                        action: Action::Set(1)
                    }])))
                }
            ))),
            "zero compose budget is normalized to a one-change page so FetchChanged can make progress"
        );
    }

    #[test]
    fn fetch_changed_with_zero_count_must_not_return_empty_success() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.set(1, 1);
        while store.pop_out().is_some() {}

        store.on_rpc_req(2, RpcReq::FetchChanged { from: Version(1), count: 0 });

        assert_ne!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                2,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcRes(RpcRes::FetchChanged(Ok(vec![])))
                }
            ))),
            "FetchChanged with zero count must be rejected instead of returning an empty success"
        );
    }

    #[test]
    fn fetch_snapshot_with_lower_cursor_past_end_must_not_panic() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.set(1, 1);
        store.set(2, 2);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.on_rpc_req(
                2,
                RpcReq::FetchSnapshot {
                    from: Some(3),
                    max_version: None,
                    max_items: 2,
                    req_id: 1,
                },
            );
        }));

        assert!(result.is_ok(), "untrusted FetchSnapshot cursor must be handled without panicking");
    }

    #[test]
    fn local_set_at_max_version_must_not_overflow() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.version = Version(u64::MAX);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.set(1, 1);
        }));

        assert!(result.is_ok(), "local version increment must not panic or wrap at u64::MAX");
        assert_eq!(store.version, Version(u64::MAX), "local version must stay at max when no successor exists");
        assert_eq!(store.slots, BTreeMap::new(), "set at max version must not create an unreplicable local slot");
        assert_eq!(store.pop_out(), None, "set at max version must not emit unreplicable events");
    }

    #[test]
    fn local_del_at_max_version_must_not_overflow_or_remove_slot() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.version = Version(u64::MAX);
        store.slots.insert(1, Slot::new(1, Version(u64::MAX)));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.del(1);
        }));

        assert!(result.is_ok(), "local delete version increment must not panic or wrap at u64::MAX");
        assert_eq!(store.version, Version(u64::MAX), "local version must stay at max when no successor exists");
        assert_eq!(
            store.slots,
            BTreeMap::from([(1, Slot::new(1, Version(u64::MAX)))]),
            "delete at max version must not remove an unreplicable slot"
        );
        assert_eq!(store.pop_out(), None, "delete at max version must not emit unreplicable events");
    }

    #[test]
    fn deleting_absent_key_must_not_emit_delete_event() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);

        store.del(99);

        assert_eq!(store.pop_out(), None, "deleting a key that was never present must not broadcast or emit a delete event");
        assert_eq!(store.version, Version(0), "deleting a key that was never present must not advance the replicated version");
    }

    #[test]
    fn snapshot_with_zero_compose_budget_must_make_progress() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 0);
        store.set(1, 1);

        let snapshot = store.snapshot(None, None, 1).expect("snapshot should exist");

        assert_eq!(
            snapshot,
            SnapshotData {
                slots: vec![(1, Slot::new(1, Version(1)))],
                skipped_newer: vec![],
                next_key: None,
            },
            "zero compose budget is normalized to a one-slot page so snapshot sync can make progress"
        );
    }

    #[test]
    fn multi_slot_snapshot_with_zero_compose_budget_must_advance_by_one_slot() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 0);
        store.set(1, 1);
        store.set(2, 2);
        store.set(3, 3);

        let first = store.snapshot(None, None, 1).expect("first snapshot page should exist");
        assert_eq!(
            first,
            SnapshotData {
                slots: vec![(1, Slot::new(1, Version(1)))],
                skipped_newer: vec![],
                next_key: Some(2),
            }
        );

        let second = store.snapshot(first.next_key, Some(Version(3)), 1).expect("second snapshot page should exist");
        assert_eq!(
            second,
            SnapshotData {
                slots: vec![(2, Slot::new(2, Version(2)))],
                skipped_newer: vec![],
                next_key: Some(3),
            }
        );
    }

    #[test]
    fn snapshot_with_skipped_newer_key_must_continue_to_eligible_slots() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 3);
        store.set(1, 10);
        store.set(2, 20);
        store.set(3, 30);
        store.set(2, 21);

        let snapshot = store.snapshot(Some(2), Some(Version(3)), 3);

        assert_eq!(
            snapshot,
            Some(SnapshotData {
                slots: vec![(3, Slot::new(30, Version(3)))],
                skipped_newer: vec![(2, Version(4))],
                next_key: None,
            }),
            "snapshot at max_version should skip newer current slots and keep scanning for eligible keys"
        );
    }

    #[test]
    fn snapshot_empty_page_from_skipped_newer_keys_must_advance_or_complete() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 1);
        store.set(1, 10);
        store.set(2, 20);
        store.set(1, 11);
        store.set(2, 21);

        assert_eq!(
            store.snapshot(Some(1), Some(Version(2)), 1),
            Some(SnapshotData {
                slots: vec![],
                skipped_newer: vec![(1, Version(3))],
                next_key: Some(2),
            }),
            "an empty page caused by a skipped newer key must advance to the next key when more keys remain"
        );
        assert_eq!(
            store.snapshot(Some(2), Some(Version(2)), 1),
            Some(SnapshotData {
                slots: vec![],
                skipped_newer: vec![(2, Version(4))],
                next_key: None,
            }),
            "an empty page caused by skipped newer keys may complete when the scan is exhausted"
        );
    }

    #[test]
    fn continuation_snapshot_response_must_preserve_requested_max_version() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 1);
        store.set(1, 10);
        store.set(2, 20);
        while store.pop_out().is_some() {}

        store.set(3, 30);
        while store.pop_out().is_some() {}

        store.on_rpc_req(
            2,
            RpcReq::FetchSnapshot {
                from: Some(2),
                max_version: Some(Version(2)),
                max_items: 1,
                req_id: 1,
            },
        );

        assert_eq!(
            store.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                2,
                RpcEvent {
                    session_id: 1,
                    data: RpcEventData::RpcRes(RpcRes::FetchSnapshot(
                        Some(SnapshotData {
                            slots: vec![(2, Slot::new(20, Version(2)))],
                            skipped_newer: vec![],
                            next_key: Some(3),
                        }),
                        Version(2),
                        1,
                    ))
                }
            ))),
            "FetchSnapshot responses constrained by max_version must declare that same snapshot version, not the producer's newer live version"
        );
    }

    #[test]
    fn fetch_snapshot_with_zero_max_items_must_not_return_items() {
        let mut store: LocalStore<u16, u16, u16> = LocalStore::new(1, 10, 2);
        store.set(1, 10);
        while store.pop_out().is_some() {}

        store.on_rpc_req(
            2,
            RpcReq::FetchSnapshot {
                from: None,
                max_version: None,
                max_items: 0,
                req_id: 1,
            },
        );

        let Some(Event::NetEvent(NetEvent::Unicast(_, RpcEvent { data: RpcEventData::RpcRes(RpcRes::FetchSnapshot(Some(snapshot), _, _)), .. }))) = store.pop_out() else {
            panic!("local store must answer FetchSnapshot");
        };

        assert!(snapshot.slots.is_empty(), "FetchSnapshot must not return more slots than the caller's max_items=0 limit");
        assert_eq!(snapshot.next_key, None, "FetchSnapshot max_items=0 must not return a non-advancing continuation cursor");
    }
}
