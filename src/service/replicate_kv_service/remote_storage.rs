use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    time::Instant,
};

use super::messages::{Action, BroadcastEvent, Changed, Event, KvEvent, NetEvent, RpcEvent, RpcReq, RpcRes, Slot, Version};

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Debug, PartialEq, Eq)]
enum RemoteStoreState<N, K, V> {
    SyncFull(SyncFullState<N, K, V>),
    Working(WorkingState<N, K, V>),
    Destroy(DestroyState<N, K, V>),
}

impl<N, K, V> State<N, K, V> for RemoteStoreState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        match self {
            RemoteStoreState::SyncFull(state) => state.init(ctx, now),
            RemoteStoreState::Working(state) => state.init(ctx, now),
            RemoteStoreState::Destroy(state) => state.init(ctx, now),
        }
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_tick(ctx, now),
            RemoteStoreState::Working(state) => state.on_tick(ctx, now),
            RemoteStoreState::Destroy(state) => state.on_tick(ctx, now),
        }
    }

    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>) {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_broadcast(ctx, now, event),
            RemoteStoreState::Working(state) => state.on_broadcast(ctx, now, event),
            RemoteStoreState::Destroy(state) => state.on_broadcast(ctx, now, event),
        }
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) {
        match self {
            RemoteStoreState::SyncFull(state) => state.on_rpc_res(ctx, now, event),
            RemoteStoreState::Working(state) => state.on_rpc_res(ctx, now, event),
            RemoteStoreState::Destroy(state) => state.on_rpc_res(ctx, now, event),
        }
    }
}

struct StateCtx<N, K, V> {
    remote: N,
    slots: BTreeMap<K, Slot<V>>,
    outs: VecDeque<Event<N, K, V>>,
    next_state: Option<RemoteStoreState<N, K, V>>,
}

trait State<N, K, V> {
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant);
    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant);
    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>);
    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>);
}

pub struct RemoteStore<N, K, V> {
    ctx: StateCtx<N, K, V>,
    state: RemoteStoreState<N, K, V>,
    last_active: Instant,
}

impl<N, K, V> RemoteStore<N, K, V>
where
    N: Debug + Clone,
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
{
    pub fn new(remote: N) -> Self {
        let mut ctx = StateCtx {
            remote,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let mut state = SyncFullState::default();
        state.init(&mut ctx, Instant::now());

        Self {
            ctx,
            state: RemoteStoreState::SyncFull(state),
            last_active: Instant::now(),
        }
    }

    pub fn on_tick(&mut self) {
        self.state.on_tick(&mut self.ctx, Instant::now());
    }

    pub fn destroy(&mut self) {
        let mut state = DestroyState { _tmp: PhantomData };
        state.init(&mut self.ctx, Instant::now());
        self.state = RemoteStoreState::Destroy(state);
    }

    pub fn last_active(&self) -> Instant {
        self.last_active
    }

    pub fn on_broadcast(&mut self, event: BroadcastEvent<K, V>) {
        self.last_active = Instant::now();
        self.state.on_broadcast(&mut self.ctx, Instant::now(), event);
        if let Some(mut next_state) = self.ctx.next_state.take() {
            next_state.init(&mut self.ctx, Instant::now());
            self.state = next_state;
        }
    }

    pub fn on_rpc_res(&mut self, event: RpcRes<K, V>) {
        self.last_active = Instant::now();
        self.state.on_rpc_res(&mut self.ctx, Instant::now(), event);
        if let Some(mut next_state) = self.ctx.next_state.take() {
            next_state.init(&mut self.ctx, Instant::now());
            self.state = next_state;
        }
    }

    pub fn pop_out(&mut self) -> Option<Event<N, K, V>> {
        self.ctx.outs.pop_front()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SyncFullState<N, K, V> {
    version: Option<Version>,
    biggest_key: Option<K>,
    sending_req: Option<(Instant, NetEvent<N, K, V>)>,
    _tmp: PhantomData<(N, K, V)>,
}

impl<N, K, V> Default for SyncFullState<N, K, V> {
    fn default() -> Self {
        Self {
            version: None,
            biggest_key: None,
            sending_req: None,
            _tmp: PhantomData,
        }
    }
}

impl<N, K, V> State<N, K, V> for SyncFullState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        log::info!("[RemoteStore {:?}] switch to syncFull", ctx.remote);
        while let Some((k, _v)) = ctx.slots.pop_first() {
            ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), k)));
        }
        // first time we don't have information about data then request snapshot without from and to
        // after it response, we will request snapshot with from and to if needed
        let req = NetEvent::Unicast(
            ctx.remote.clone(),
            RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                from: None,
                to: None,
                max_version: None,
            }),
        );
        self.sending_req = Some((now, req.clone()));
        ctx.outs.push_back(Event::NetEvent(req));
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        if let Some((sent_at, req)) = self.sending_req.as_mut() {
            let now = now;
            if now - *sent_at >= REQUEST_TIMEOUT {
                log::warn!("[RemoteStore {:?}] syncFull timeout => resend", ctx.remote);
                *sent_at = now;
                ctx.outs.push_back(Event::NetEvent(req.clone()));
            }
        }
    }

    fn on_broadcast(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, _event: BroadcastEvent<K, V>) {
        // dont process here
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: RpcRes<K, V>) {
        match event {
            RpcRes::FetchChanged { .. } => {
                // dont process here
            }
            RpcRes::FetchSnapshot(Some(snapshot), version) => {
                // TODO check snapshot is not empty
                log::info!(
                    "[RemoteStore {:?}] got snapshot {} slots and biggest_key {:?}, current version {version:?}, next {:?}",
                    ctx.remote,
                    snapshot.slots.len(),
                    snapshot.biggest_key,
                    snapshot.next_key,
                );
                for (k, slot) in snapshot.slots.into_iter() {
                    ctx.outs.push_back(Event::KvEvent(KvEvent::Set(Some(ctx.remote.clone()), k.clone(), slot.value.clone())));
                    ctx.slots.insert(k, slot);
                }
                if self.version.is_none() {
                    self.version = Some(version);
                    self.biggest_key = Some(snapshot.biggest_key);
                }
                if let Some(next_key) = snapshot.next_key {
                    let to = self.biggest_key.clone().expect("should have biggest key");
                    let max_version = self.version.expect("should have version");

                    log::info!(
                        "[RemoteStore {:?}] request more snapshot data with from {next_key:?} and to {to:?}, max_version {max_version:?}",
                        ctx.remote
                    );
                    let req = NetEvent::Unicast(
                        ctx.remote.clone(),
                        RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                            from: Some(next_key),
                            to: Some(to),
                            max_version: Some(max_version),
                        }),
                    );
                    self.sending_req = Some((now, req.clone()));
                    ctx.outs.push_back(Event::NetEvent(req));
                } else {
                    let version = self.version.expect("should have version");
                    log::info!("[RemoteStore {:?}] switch to working with {} slots and version {version:?}", ctx.remote, ctx.slots.len());
                    self.sending_req = None;
                    ctx.next_state = Some(RemoteStoreState::Working(WorkingState::new(version)));
                }
            }
            RpcRes::FetchSnapshot(None, version) => {
                log::info!("[RemoteStore {:?}] switch to working with 0 slots and version {version:?}", ctx.remote);
                self.sending_req = None;
                ctx.next_state = Some(RemoteStoreState::Working(WorkingState::new(version)));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WorkingState<N, K, V> {
    version: Version,
    // this is a list of changeds in order, we use it to detect discontinuity to send fetchChanged
    pendings: BTreeMap<Version, Changed<K, V>>,
    sending_req: Option<(Instant, NetEvent<N, K, V>)>,
    _tmp: PhantomData<(N, V)>,
}

impl<N, K, V> WorkingState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn new(version: Version) -> Self {
        Self {
            version,
            pendings: BTreeMap::new(),
            sending_req: None,
            _tmp: PhantomData,
        }
    }

    fn apply_pendings(&mut self, ctx: &mut StateCtx<N, K, V>) {
        while let Some(entry) = self.pendings.first_entry() {
            if *entry.key() == self.version + 1 {
                self.version = self.version + 1;
                let (_, data) = self.pendings.pop_first().expect("should have data");
                match data.action {
                    Action::Set(value) => {
                        log::debug!("[RemoteStore {:?}] apply set k {:?} => version {:?}", ctx.remote, data.key, data.version);
                        ctx.outs.push_back(Event::KvEvent(KvEvent::Set(Some(ctx.remote.clone()), data.key.clone(), value.clone())));
                        ctx.slots.insert(data.key, Slot::new(value, data.version));
                    }
                    Action::Del => {
                        log::debug!("[RemoteStore {:?}] apply del k {:?} => version {:?}", ctx.remote, data.key, data.version);
                        ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), data.key.clone())));
                        ctx.slots.remove(&data.key);
                    }
                }
            } else {
                let from = self.version + 1;
                let count = *entry.key() - self.version - 1;
                log::warn!("[RemoteStore {:?}] apply pendings discontinuity => request fetch changed from {from:?} count {count}", ctx.remote);
                let req = NetEvent::Unicast(ctx.remote.clone(), RpcEvent::RpcReq(RpcReq::FetchChanged { from, count }));
                self.sending_req = Some((Instant::now(), req.clone()));
                ctx.outs.push_back(Event::NetEvent(req));
                break;
            }
        }
    }
}

impl<N, K, V> State<N, K, V> for WorkingState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Clone,
    N: Debug + Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        // dont need init in working state
        log::info!("[RemoteStore {:?}] switch to working", ctx.remote);
    }

    fn on_tick(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant) {
        if let Some((sent_at, req)) = self.sending_req.as_mut() {
            if now - *sent_at >= REQUEST_TIMEOUT {
                log::warn!("[RemoteStore {:?}] fetch changed timeout", ctx.remote);
                *sent_at = now;
                ctx.outs.push_back(Event::NetEvent(req.clone()));
            }
        }
    }

    fn on_broadcast(&mut self, ctx: &mut StateCtx<N, K, V>, now: Instant, event: BroadcastEvent<K, V>) {
        match event {
            BroadcastEvent::Changed(changed) => {
                if self.version < changed.version {
                    self.pendings.insert(changed.version, changed);
                    self.apply_pendings(ctx);
                }
            }
            BroadcastEvent::Version(version) => {
                if version > self.version && self.pendings.is_empty() {
                    // resync part
                    let from = self.version + 1;
                    let count = version - self.version;
                    log::warn!("[RemoteStore {:?}] received discontinuity version => request fetch changed from {from:?} count {count}", ctx.remote);
                    let req = NetEvent::Unicast(ctx.remote.clone(), RpcEvent::RpcReq(RpcReq::FetchChanged { from, count }));
                    self.sending_req = Some((now, req.clone()));
                    ctx.outs.push_back(Event::NetEvent(req));
                }
            }
        }
    }

    fn on_rpc_res(&mut self, ctx: &mut StateCtx<N, K, V>, _now: Instant, event: RpcRes<K, V>) {
        match event {
            RpcRes::FetchChanged(Ok(changeds)) => {
                log::info!("[RemoteStore {:?}] fetch changed success with {} changeds => apply", ctx.remote, changeds.len());
                for changed in changeds {
                    if changed.version > self.version {
                        self.pendings.insert(changed.version, changed);
                    }
                }
                self.sending_req = None;
                self.apply_pendings(ctx);
            }
            RpcRes::FetchChanged(Err(err)) => {
                log::info!("[RemoteStore] fetch changed error: {err:?} => switch to resyncFull");
                self.sending_req = None;
                ctx.next_state = Some(RemoteStoreState::SyncFull(SyncFullState::default()));
            }
            RpcRes::FetchSnapshot { .. } => {
                // not process here
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DestroyState<N, K, V> {
    _tmp: PhantomData<(N, K, V)>,
}

impl<N, K, V> State<N, K, V> for DestroyState<N, K, V>
where
    K: Debug + Hash + Ord + Eq + Clone,
    V: Clone,
    N: Clone,
{
    fn init(&mut self, ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        while let Some((k, _v)) = ctx.slots.pop_first() {
            ctx.outs.push_back(Event::KvEvent(KvEvent::Del(Some(ctx.remote.clone()), k)));
        }
    }

    fn on_tick(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant) {
        // dont process here
    }

    fn on_broadcast(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, _event: BroadcastEvent<K, V>) {
        // dont process here
    }

    fn on_rpc_res(&mut self, _ctx: &mut StateCtx<N, K, V>, _now: Instant, _event: RpcRes<K, V>) {
        // dont process here
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::messages::{FetchChangedError, SnapshotData};

    use super::*;

    /// restore with some data
    #[test]
    fn test_restore_full_single_pkt() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    to: None,
                    max_version: None
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // we don't need to resend again because it too fast
        state.on_tick(&mut ctx, now);
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1)))]));
        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(1)))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);

        // we don't need to resend again because it already received answer
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn full_sync_must_reject_snapshot_slot_newer_than_declared_version() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(99)))],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert!(
            ctx.slots.is_empty(),
            "snapshot slots newer than the declared snapshot version must be rejected"
        );
        assert_eq!(ctx.outs.pop_front(), None, "invalid snapshot data must not be emitted as KvEvent::Set");
    }

    #[test]
    fn full_sync_must_reject_snapshot_next_key_past_biggest_key() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: Some(2),
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert_ne!(
            ctx.outs.pop_back(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    to: Some(1),
                    max_version: Some(Version(1)),
                })
            ))),
            "snapshot next_key greater than biggest_key must not make the receiver emit reversed FetchSnapshot bounds"
        );
    }

    #[test]
    fn full_sync_must_reject_empty_snapshot_page_with_next_key() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![],
                    next_key: Some(1),
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert!(
            ctx.outs.is_empty(),
            "empty snapshot pages with next_key must be rejected because full sync cannot make progress"
        );
    }

    #[test]
    fn full_sync_must_reject_initial_empty_snapshot_with_nonzero_version() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.next_state, None,
            "initial empty snapshot pages with nonzero version must not complete full sync"
        );
    }

    #[test]
    fn full_sync_must_reject_snapshot_slot_past_biggest_key() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(1)))],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert!(
            !ctx.slots.contains_key(&2),
            "snapshot slots beyond biggest_key must be rejected"
        );
        assert_eq!(
            ctx.next_state, None,
            "full sync must not complete after accepting a slot beyond biggest_key"
        );
    }

    #[test]
    fn full_sync_snapshot_pages_must_be_bounded() {
        const MAX_SNAPSHOT_SLOTS_PER_PAGE: u16 = 1024;
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        let slots = (0..=MAX_SNAPSHOT_SLOTS_PER_PAGE)
            .map(|key| (key, Slot::new(key, Version(key as u64 + 1))))
            .collect::<Vec<_>>();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots,
                    next_key: None,
                    biggest_key: MAX_SNAPSHOT_SLOTS_PER_PAGE,
                }),
                Version(MAX_SNAPSHOT_SLOTS_PER_PAGE as u64 + 1),
            ),
        );

        assert!(
            ctx.slots.len() <= MAX_SNAPSHOT_SLOTS_PER_PAGE as usize,
            "full-sync snapshot pages must be capped, got {} slots",
            ctx.slots.len()
        );
    }

    #[test]
    fn full_sync_must_reject_unsorted_snapshot_slots() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(1))), (1, Slot::new(1, Version(1)))],
                    next_key: None,
                    biggest_key: 2,
                }),
                Version(1),
            ),
        );

        assert!(
            ctx.slots.is_empty(),
            "snapshot slots must be rejected unless they are ordered by key"
        );
        assert_eq!(
            ctx.next_state, None,
            "full sync must not complete after accepting unsorted snapshot slots"
        );
    }

    #[test]
    fn full_sync_must_reject_duplicate_snapshot_keys() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1))), (1, Slot::new(2, Version(1)))],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.slots,
            BTreeMap::new(),
            "snapshot pages with duplicate keys must be rejected"
        );
        assert_eq!(
            ctx.next_state, None,
            "full sync must not complete after accepting duplicate snapshot keys"
        );
    }

    #[test]
    fn full_sync_must_reject_continuation_slot_before_requested_key() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: Some(5),
                    biggest_key: 10,
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(4, Slot::new(4, Version(1)))],
                    next_key: None,
                    biggest_key: 10,
                }),
                Version(1),
            ),
        );

        assert!(
            !ctx.slots.contains_key(&4),
            "continuation snapshot slots before the requested next_key must be rejected"
        );
        assert_eq!(
            ctx.next_state, None,
            "full sync must not complete after accepting a continuation slot before the requested next_key"
        );
    }

    #[test]
    fn full_sync_must_reject_continuation_snapshot_version_mismatch() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: Some(2),
                    biggest_key: 2,
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(2)))],
                    next_key: None,
                    biggest_key: 2,
                }),
                Version(2),
            ),
        );

        assert!(
            !ctx.slots.contains_key(&2),
            "continuation snapshot page with a different declared version must be rejected"
        );
        assert_eq!(
            ctx.next_state, None,
            "full sync must not transition to WorkingState after accepting data newer than the locked snapshot version"
        );
    }

    #[test]
    fn full_sync_must_reject_none_continuation_after_partial_snapshot() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: Some(2),
                    biggest_key: 2,
                }),
                Version(1),
            ),
        );
        ctx.outs.clear();

        state.on_rpc_res(&mut ctx, now, RpcRes::FetchSnapshot(None, Version(1)));

        assert_eq!(
            ctx.next_state, None,
            "full sync must not treat None as completion after a partial snapshot requested a continuation"
        );
    }

    #[test]
    fn full_sync_must_reject_stale_terminal_snapshot_after_continuation_request() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    next_key: Some(2),
                    biggest_key: 3,
                }),
                Version(3),
            ),
        );

        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 10))));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    to: Some(3),
                    max_version: Some(Version(3)),
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(10, Version(1)))],
                    next_key: None,
                    biggest_key: 1,
                }),
                Version(1),
            ),
        );

        assert_eq!(
            ctx.next_state, None,
            "stale terminal snapshot responses must not complete full sync while a continuation range is outstanding"
        );
        assert_eq!(
            ctx.slots,
            BTreeMap::from([(1, Slot::new(10, Version(1)))]),
            "stale terminal snapshot responses must not hide keys still pending in the requested continuation range"
        );
    }

    #[test]
    fn test_restore_full_resend() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    to: None,
                    max_version: None
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // we need to resend again because it timeout
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    to: None,
                    max_version: None
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// restore with some data
    #[test]
    fn test_restore_multi_single_pkt() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = SyncFullState::default();
        state.init(&mut ctx, now);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: None,
                    to: None,
                    max_version: None
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // got first sync
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(1, Slot::new(1, Version(1)))],
                    next_key: Some(2),
                    biggest_key: 2,
                }),
                Version(2),
            ),
        );

        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchSnapshot {
                    from: Some(2),
                    to: Some(2),
                    max_version: Some(Version(2))
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // got last sync
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchSnapshot(
                Some(SnapshotData {
                    slots: vec![(2, Slot::new(2, Version(2)))],
                    next_key: None,
                    biggest_key: 2,
                }),
                Version(2),
            ),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1))), (2, Slot::new(2, Version(2)))]));
        assert_eq!(ctx.next_state, Some(RemoteStoreState::Working(WorkingState::new(Version(2)))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 2, 2))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// start with zero
    #[test]
    fn test_working_state_zero() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(1),
            }),
        );

        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(1)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// start with zero but got out of sync
    #[test]
    fn test_working_state_zero_out_of_sync() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(1)));

        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    /// After missing changed we got Changed event
    #[test]
    fn test_working_state_missing_changed() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }),
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }),
        );

        assert_eq!(state.pendings.len(), 0);
        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(2)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 2))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);

        // after received FetchChanged, it should be rejected
        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }])),
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // after received FetchChanged it should not be resend
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_cancel_fetch_changed_when_broadcast_fills_gap() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(2),
            }),
        );
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(1),
            }),
        );
        assert_eq!(state.version, Version(2), "broadcast gap fill should advance the working version");
        while ctx.outs.pop_front().is_some() {}

        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "FetchChanged retry must be cancelled after broadcasts fill the missing gap"
        );
    }

    /// After missing changed we got FetchChanged response
    #[test]
    fn test_working_state_missing_changed2() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }),
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(2),
            }])),
        );

        assert_eq!(state.pendings.len(), 0);
        assert_eq!(ctx.slots, BTreeMap::from([(1, Slot::new(1, Version(2)))]));
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 2))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_reject_unsolicited_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(9),
            }])),
        );

        assert_eq!(
            state.version,
            Version(0),
            "unsolicited FetchChanged success must not advance the working version"
        );
        assert_eq!(
            ctx.slots,
            BTreeMap::new(),
            "unsolicited FetchChanged success must not mutate replicated slots"
        );
        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "unsolicited FetchChanged success must not emit local KvEvent changes"
        );
    }

    #[test]
    fn working_state_must_reject_unsolicited_fetch_changed_error() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore {
            ctx: StateCtx {
                remote: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(1)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(1))),
            last_active: now,
        };

        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));

        assert_eq!(
            remote.ctx.slots,
            BTreeMap::from([(7, Slot::new(70, Version(1)))]),
            "unsolicited FetchChanged errors must not clear existing remote slots"
        );
        assert_eq!(
            remote.pop_out(),
            None,
            "unsolicited FetchChanged errors must not emit deletes or start a full resync"
        );
        assert_eq!(
            remote.state,
            RemoteStoreState::Working(WorkingState::new(Version(1))),
            "unsolicited FetchChanged errors must not leave WorkingState"
        );
    }

    #[test]
    fn solicited_full_resync_must_not_delete_existing_slots_before_snapshot_completes() {
        let now = Instant::now();
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore {
            ctx: StateCtx {
                remote: 1,
                slots: BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(2))),
            last_active: now,
        };

        remote.on_broadcast(BroadcastEvent::Version(Version(5)));
        assert!(matches!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 }))))
        ));
        assert_eq!(remote.pop_out(), None);

        remote.on_rpc_res(RpcRes::FetchChanged(Err(FetchChangedError::MissingData)));

        assert_eq!(
            remote.ctx.slots,
            BTreeMap::from([(1, Slot::new(10, Version(1))), (2, Slot::new(20, Version(2)))]),
            "existing remote data should remain visible until the replacement full snapshot is complete"
        );
        assert_ne!(
            remote.pop_out(),
            Some(Event::KvEvent(KvEvent::Del(Some(1), 1))),
            "a legitimate full-resync fallback must not emit false deletes before it has replacement data"
        );
    }

    #[test]
    fn ignored_rpc_response_must_not_refresh_remote_activity() {
        let stale = Instant::now() - Duration::from_secs(11);
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore {
            ctx: StateCtx {
                remote: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(1)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(1))),
            last_active: stale,
        };

        remote.on_rpc_res(RpcRes::FetchSnapshot(None, Version(99)));

        assert_eq!(
            remote.last_active(),
            stale,
            "ignored or unsolicited RPC responses must not refresh remote activity and prevent timeout cleanup"
        );
        assert_eq!(remote.pop_out(), None, "ignored RPC responses must not emit local events");
    }

    #[test]
    fn ignored_broadcast_must_not_refresh_remote_activity() {
        let stale = Instant::now() - Duration::from_secs(11);
        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore {
            ctx: StateCtx {
                remote: 1,
                slots: BTreeMap::from([(7, Slot::new(70, Version(5)))]),
                outs: VecDeque::new(),
                next_state: None,
            },
            state: RemoteStoreState::Working(WorkingState::new(Version(5))),
            last_active: stale,
        };

        remote.on_broadcast(BroadcastEvent::Version(Version(5)));

        assert_eq!(
            remote.last_active(),
            stale,
            "ignored stale broadcasts must not refresh remote activity and prevent timeout cleanup"
        );
        assert_eq!(remote.pop_out(), None, "ignored stale broadcasts must not emit local events");
    }

    #[test]
    fn working_state_must_reject_duplicate_fetch_changed_versions() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(1)));
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(1),
                },
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(9),
                },
            ])),
        );

        assert_eq!(
            state.version,
            Version(0),
            "FetchChanged responses with duplicate versions must not advance the working version"
        );
        assert_eq!(
            ctx.slots,
            BTreeMap::new(),
            "FetchChanged responses with duplicate versions must not overwrite and apply one entry"
        );
        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "FetchChanged responses with duplicate versions must not emit local KvEvent changes"
        );
    }

    #[test]
    fn working_state_must_reject_fetch_changed_versions_beyond_requested_count() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(1)));
        ctx.outs.clear();

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(1),
                },
                Changed {
                    key: 2,
                    version: Version(2),
                    action: Action::Set(2),
                },
            ])),
        );

        assert_eq!(
            state.version,
            Version(0),
            "FetchChanged responses must not apply versions beyond the requested count"
        );
        assert_eq!(
            ctx.slots,
            BTreeMap::new(),
            "FetchChanged responses must reject extra versions outside the requested range"
        );
        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "FetchChanged responses with extra versions must not emit local KvEvent changes"
        );
    }

    #[test]
    fn working_state_must_not_cancel_repair_after_empty_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(1)));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(&mut ctx, now, RpcRes::FetchChanged(Ok(vec![])));
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert!(
            ctx.next_state.is_some() || ctx.outs.pop_front().is_some(),
            "empty FetchChanged success must not cancel the in-flight repair without retrying or starting full resync"
        );
    }

    #[test]
    fn working_state_must_continue_repair_after_partial_fetch_changed_success() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(5)));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 5 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![
                Changed {
                    key: 1,
                    version: Version(1),
                    action: Action::Set(10),
                },
                Changed {
                    key: 2,
                    version: Version(2),
                    action: Action::Set(20),
                },
            ])),
        );

        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 1, 10))));
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Set(Some(1), 2, 20))));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(3), count: 3 })))),
            "partial FetchChanged success must continue repairing the remaining requested versions"
        );
    }

    #[test]
    fn working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(1)));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(&mut ctx, now, BroadcastEvent::Version(Version(5)));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 5 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_rpc_res(
            &mut ctx,
            now,
            RpcRes::FetchChanged(Ok(vec![Changed {
                key: 1,
                version: Version(1),
                action: Action::Set(10),
            }])),
        );

        while matches!(ctx.outs.pop_front(), Some(Event::KvEvent(_))) {}
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(2), count: 4 })))),
            "stale response for the old narrow request must not cancel the newer repair for versions 2..=5"
        );
    }

    #[test]
    fn test_working_state_resend_timeout_fetch_changed() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 1,
                version: Version(2),
                action: Action::Set(1),
            }),
        );

        assert_eq!(state.pendings.len(), 1);
        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        // now after timeout we should resend
        state.on_tick(&mut ctx, now + REQUEST_TIMEOUT + Duration::from_millis(1));
        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(RpcReq::FetchChanged { from: Version(1), count: 1 }))))
        );
        assert_eq!(ctx.outs.pop_front(), None);
    }

    #[test]
    fn working_state_must_not_duplicate_inflight_fetch_changed_for_same_gap() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));
        state.init(&mut ctx, now);

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 10,
                version: Version(10),
                action: Action::Set(10),
            }),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(RpcReq::FetchChanged {
                    from: Version(1),
                    count: 9
                })
            )))
        );
        assert_eq!(ctx.outs.pop_front(), None);

        state.on_broadcast(
            &mut ctx,
            now + Duration::from_millis(10),
            BroadcastEvent::Changed(Changed {
                key: 11,
                version: Version(11),
                action: Action::Set(11),
            }),
        );

        assert_eq!(
            ctx.outs.pop_front(),
            None,
            "same-gap FetchChanged repair is already in flight and must not be duplicated before timeout or response"
        );
    }

    #[test]
    fn working_state_must_cap_pending_future_changes() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        for version in 2..=2_050 {
            state.on_broadcast(
                &mut ctx,
                now,
                BroadcastEvent::Changed(Changed {
                    key: version as u16,
                    version: Version(version),
                    action: Action::Set(version as u16),
                }),
            );
        }

        assert!(
            state.pendings.len() <= 1024,
            "future changed broadcasts from one remote must be capped to avoid unbounded memory growth"
        );
    }

    #[test]
    fn working_state_must_reject_duplicate_pending_changed_broadcast_versions() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));

        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 7,
                version: Version(2),
                action: Action::Set(20),
            }),
        );
        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 7,
                version: Version(2),
                action: Action::Set(99),
            }),
        );
        state.on_broadcast(
            &mut ctx,
            now,
            BroadcastEvent::Changed(Changed {
                key: 7,
                version: Version(1),
                action: Action::Set(10),
            }),
        );

        assert_eq!(
            ctx.slots.get(&7),
            Some(&Slot::new(20, Version(2))),
            "duplicate pending Changed broadcasts must not overwrite the first accepted value for the same version"
        );
    }

    #[test]
    fn working_state_must_cap_pending_fetch_changed_response() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::new(),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = WorkingState::new(Version(0));
        let changeds = (2..=2_050)
            .map(|version| Changed {
                key: version as u16,
                version: Version(version),
                action: Action::Set(version as u16),
            })
            .collect();

        state.on_rpc_res(&mut ctx, now, RpcRes::FetchChanged(Ok(changeds)));

        assert!(
            state.pendings.len() <= 1024,
            "future changed RPC responses from one remote must be capped to avoid unbounded memory growth, got {}",
            state.pendings.len()
        );
    }

    #[test]
    fn destroy_remote_should_clear_slots() {
        let mut ctx: StateCtx<u16, u16, u16> = StateCtx {
            remote: 1,
            slots: BTreeMap::from([(1, Slot::new(1, Version(1)))]),
            outs: VecDeque::new(),
            next_state: None,
        };

        let now = Instant::now();
        let mut state = DestroyState { _tmp: PhantomData };
        state.init(&mut ctx, now);

        assert_eq!(ctx.slots, BTreeMap::new());
        assert_eq!(ctx.next_state, None);
        assert_eq!(ctx.outs.pop_front(), Some(Event::KvEvent(KvEvent::Del(Some(1), 1))));
        assert_eq!(ctx.outs.pop_front(), None);
    }
}
