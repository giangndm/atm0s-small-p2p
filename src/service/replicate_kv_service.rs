//!
//! This module implement replicated local_kv which data is replicated to all nodes.
//! Each key-value is store in local-node and broadcast to all other nodes, which allow other node can access data with local-speed.
//! For simplicity, data is only belong to which node it created from. If a node disconnected, it's data will be deleted all other nodes.
//! Some useful usecase: session map
//!

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    future::poll_fn,
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
};

use local_storage::LocalStore;
use remote_storage::RemoteStore;
use serde::{de::DeserializeOwned, Serialize};
use tokio::time::Interval;

use crate::PeerId;

use super::{P2pService, P2pServiceEvent};

mod local_storage;
mod messages;
mod remote_storage;

use messages::{BroadcastEvent, Event, NetEvent, RpcEvent};

pub use messages::KvEvent;

const REMOTE_TIMEOUT_MS: u128 = 10_000;
const MAX_PENDING_OUT_EVENTS: usize = 1024;
const MAX_REMOTE_STORES: usize = 1024;

pub struct ReplicatedKvStore<N, K, V> {
    remotes: HashMap<N, RemoteStore<N, K, V>>,
    local: LocalStore<N, K, V>,
    outs: VecDeque<Event<N, K, V>>,
}

impl<N, K, V> ReplicatedKvStore<N, K, V>
where
    N: Debug + Eq + Hash + Clone,
    K: Debug + Hash + Ord + Eq + Clone,
    V: Debug + Eq + Clone,
{
    pub fn new(max_changeds: usize, max_compose_pkts: usize) -> Self {
        ReplicatedKvStore {
            remotes: HashMap::new(),
            local: LocalStore::new(max_changeds, max_compose_pkts),
            outs: VecDeque::new(),
        }
    }

    pub fn on_tick(&mut self) {
        self.local.on_tick();
        while let Some(event) = self.local.pop_out() {
            Self::push_out(&mut self.outs, event);
        }
        self.cleanup_timed_out_remotes();
        for remote in self.remotes.values_mut() {
            remote.on_tick();
            while let Some(event) = remote.pop_out() {
                Self::push_out(&mut self.outs, event);
            }
        }
    }

    fn cleanup_timed_out_remotes(&mut self) {
        let outs = &mut self.outs;
        self.remotes.retain(|node, remote| {
            let keep = remote.last_active().elapsed().as_millis() < REMOTE_TIMEOUT_MS;
            if !keep {
                log::info!("[ReplicatedKvService] remove remote {node:?} after timeout");
                remote.destroy();
                while let Some(event) = remote.pop_out() {
                    Self::push_out(outs, event);
                }
            }
            keep
        });
    }

    pub fn set(&mut self, key: K, value: V) {
        self.local.set(key.clone(), value.clone());
        while let Some(event) = self.local.pop_out() {
            Self::push_out(&mut self.outs, event);
        }
    }

    pub fn del(&mut self, key: K) {
        self.local.del(key.clone());
        while let Some(event) = self.local.pop_out() {
            Self::push_out(&mut self.outs, event);
        }
    }

    pub fn on_remote_event(&mut self, from: N, event: NetEvent<N, K, V>) {
        if !self.remotes.contains_key(&from) && matches!(event, NetEvent::Unicast(_, RpcEvent::RpcRes(_))) {
            log::warn!("[ReplicatedKvService] reject unsolicited RPC response from unknown remote {from:?}");
            return;
        }

        if !self.remotes.contains_key(&from) {
            if self.remotes.len() >= MAX_REMOTE_STORES {
                self.cleanup_timed_out_remotes();
                if self.remotes.len() >= MAX_REMOTE_STORES {
                    log::warn!("[ReplicatedKvService] reject new remote {from:?}: remote store cap {MAX_REMOTE_STORES} reached");
                    return;
                }
            }

            log::info!("[ReplicatedKvService] add remote {from:?}");
            let mut remote = RemoteStore::new(from.clone());
            while let Some(event) = remote.pop_out() {
                Self::push_out(&mut self.outs, event);
            }
            self.remotes.insert(from.clone(), remote);
        }

        match event {
            NetEvent::Broadcast(event) => {
                if let Some(remote) = self.remotes.get_mut(&from) {
                    remote.on_broadcast(event);
                    while let Some(event) = remote.pop_out() {
                        Self::push_out(&mut self.outs, event);
                    }
                }
            }
            NetEvent::Unicast(_from, event) => match event {
                RpcEvent::RpcReq(rpc_req) => {
                    self.local.on_rpc_req(from, rpc_req);
                    while let Some(event) = self.local.pop_out() {
                        Self::push_out(&mut self.outs, event);
                    }
                }
                RpcEvent::RpcRes(rpc_res) => {
                    if let Some(remote) = self.remotes.get_mut(&from) {
                        remote.on_rpc_res(rpc_res);
                        while let Some(event) = remote.pop_out() {
                            Self::push_out(&mut self.outs, event);
                        }
                    }
                }
            },
        }
    }

    pub fn on_peer_disconnected(&mut self, peer: N) {
        if let Some(mut remote) = self.remotes.remove(&peer) {
            log::info!("[ReplicatedKvService] remove remote {peer:?} after peer disconnected");
            remote.destroy();
            while let Some(event) = remote.pop_out() {
                Self::push_out(&mut self.outs, event);
            }
        }
    }

    fn push_out(outs: &mut VecDeque<Event<N, K, V>>, event: Event<N, K, V>) {
        if outs.len() >= MAX_PENDING_OUT_EVENTS {
            outs.pop_front();
        }
        outs.push_back(event);
    }

    fn pop(&mut self) -> Option<Event<N, K, V>> {
        self.outs.pop_front()
    }
}

pub struct ReplicatedKvService<K, V> {
    service: P2pService,
    tick: Interval,
    store: ReplicatedKvStore<PeerId, K, V>,
}

impl<K, V> ReplicatedKvService<K, V>
where
    K: Debug + Hash + Ord + Eq + Clone + DeserializeOwned + Serialize,
    V: Debug + Eq + Clone + DeserializeOwned + Serialize,
{
    pub fn new(service: P2pService, max_changeds: usize, max_compose_pkts: usize) -> Self {
        Self {
            service,
            tick: tokio::time::interval(std::time::Duration::from_millis(1000)),
            store: ReplicatedKvStore::new(max_changeds, max_compose_pkts),
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        self.store.set(key, value);
    }

    pub fn del(&mut self, key: K) {
        self.store.del(key);
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<KvEvent<PeerId, K, V>>> {
        loop {
            if let Some(event) = self.store.pop() {
                match event {
                    Event::NetEvent(net_event) => match net_event {
                        NetEvent::Broadcast(broadcast_event) => {
                            match bincode::serialize(&broadcast_event) {
                                Ok(data) => {
                                    let _ = self.service.try_send_broadcast(data);
                                }
                                Err(err) => log::error!("[ReplicatedKvService] serialize broadcast error {err}"),
                            }
                            continue;
                        }
                        NetEvent::Unicast(to_node, rpc_event) => {
                            match bincode::serialize(&rpc_event) {
                                Ok(data) => {
                                    let _ = self.service.try_send_unicast(to_node, data);
                                }
                                Err(err) => log::error!("[ReplicatedKvService] serialize unicast error {err}"),
                            }
                            continue;
                        }
                    },
                    Event::KvEvent(kv_event) => {
                        return Poll::Ready(Some(kv_event));
                    }
                }
            }

            match Pin::new(&mut self.tick).poll_tick(cx) {
                Poll::Ready(_) => {
                    self.store.on_tick();
                    continue;
                }
                Poll::Pending => {}
            }

            match self.service.poll_recv(cx) {
                Poll::Ready(Some(P2pServiceEvent::Unicast(peer_id, vec))) => {
                    match bincode::deserialize::<RpcEvent<K, V>>(&vec) {
                        Ok(event) => self.store.on_remote_event(peer_id, NetEvent::Unicast(peer_id, event)),
                        Err(err) => log::error!("[ReplicatedKvService] deserialize error {err}"),
                    }
                    continue;
                }
                Poll::Ready(Some(P2pServiceEvent::Broadcast(peer_id, vec))) => {
                    match bincode::deserialize::<BroadcastEvent<K, V>>(&vec) {
                        Ok(event) => self.store.on_remote_event(peer_id, NetEvent::Broadcast(event)),
                        Err(err) => log::error!("[ReplicatedKvService] deserialize error {err}"),
                    }
                    continue;
                }
                Poll::Ready(Some(P2pServiceEvent::Stream(..))) => continue,
                Poll::Ready(Some(P2pServiceEvent::PeerDisconnected(peer_id))) => {
                    self.store.on_peer_disconnected(peer_id);
                    continue;
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    pub async fn recv(&mut self) -> Option<KvEvent<PeerId, K, V>> {
        poll_fn(|cx| self.poll_recv(cx)).await
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;
    use serde::{Deserialize, Serializer};

    use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable};

    use super::messages::Version;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    struct FailingSerializeValue(u16);

    impl Serialize for FailingSerializeValue {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional replicated kv serialization failure"))
        }
    }

    fn test_service() -> P2pService {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (service, _tx) = P2pService::build(P2pServiceId::from(0), ctx);
        service
    }

    #[test]
    fn remote_store_creation_must_be_bounded() {
        let mut store: ReplicatedKvStore<u64, u64, u64> = ReplicatedKvStore::new(10, 10);

        for from in 0..=MAX_REMOTE_STORES as u64 {
            store.on_remote_event(from, NetEvent::Broadcast(BroadcastEvent::Version(Version(0))));
        }

        let remote_count = store.remotes.len();
        let queued_sync_requests = store.outs.len();

        assert!(remote_count <= MAX_REMOTE_STORES, "remote stores must be bounded, got {remote_count}");
        assert!(queued_sync_requests <= MAX_REMOTE_STORES, "queued full-sync requests must be bounded, got {queued_sync_requests}");
    }

    #[test]
    fn unsolicited_rpc_response_from_unknown_peer_must_not_create_remote_store() {
        let mut store: ReplicatedKvStore<u64, u64, u64> = ReplicatedKvStore::new(10, 10);
        let unknown_peer = 42;

        store.on_remote_event(
            unknown_peer,
            NetEvent::Unicast(unknown_peer, RpcEvent::RpcRes(messages::RpcRes::FetchChanged(Err(messages::FetchChangedError::MissingData)))),
        );

        assert!(store.remotes.is_empty(), "an unsolicited RPC response from an unknown peer must not allocate remote state");
        assert!(store.outs.is_empty(), "an unsolicited RPC response from an unknown peer must not queue a full-sync request");
    }

    #[test]
    fn replicated_kv_local_outbound_event_queue_must_be_bounded() {
        const MAX_PENDING_EVENTS: usize = 1024;
        let mut store: ReplicatedKvStore<u64, u64, u64> = ReplicatedKvStore::new(10, 10);

        for key in 0..=MAX_PENDING_EVENTS {
            store.set(key as u64, key as u64);
        }

        assert!(store.outs.len() <= MAX_PENDING_EVENTS, "replicated KV outbound event queue must be bounded, got {}", store.outs.len());
    }

    #[test]
    fn paginated_full_sync_must_recover_when_snapshot_version_becomes_unavailable() {
        let mut local: LocalStore<u16, u16, u16> = LocalStore::new(10, 1);
        local.set(1, 10);
        local.set(2, 20);
        while local.pop_out().is_some() {}

        let mut remote: RemoteStore<u16, u16, u16> = RemoteStore::new(1);
        assert_eq!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(messages::RpcReq::FetchSnapshot {
                    from: None,
                    max_version: None,
                    max_items: 1024,
                })
            )))
        );

        local.on_rpc_req(
            1,
            messages::RpcReq::FetchSnapshot {
                from: None,
                max_version: None,
                max_items: 1024,
            },
        );
        let Some(Event::NetEvent(NetEvent::Unicast(_, RpcEvent::RpcRes(messages::RpcRes::FetchSnapshot(first_page, first_version))))) = local.pop_out() else {
            panic!("local store must answer the initial snapshot request");
        };
        assert_eq!(first_version, Version(2));

        remote.on_rpc_res(messages::RpcRes::FetchSnapshot(first_page, first_version));
        assert_eq!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(
                1,
                RpcEvent::RpcReq(messages::RpcReq::FetchSnapshot {
                    from: Some(2),
                    max_version: Some(Version(2)),
                    max_items: 1024,
                })
            )))
        );

        local.set(2, 21);
        while local.pop_out().is_some() {}
        local.on_rpc_req(
            1,
            messages::RpcReq::FetchSnapshot {
                from: Some(2),
                max_version: Some(Version(2)),
                max_items: 1024,
            },
        );
        let Some(Event::NetEvent(NetEvent::Unicast(_, RpcEvent::RpcRes(messages::RpcRes::FetchSnapshot(continuation, continuation_version))))) = local.pop_out() else {
            panic!("local store must answer the continuation snapshot request");
        };
        assert_eq!(
            continuation,
            Some(messages::SnapshotData {
                slots: vec![],
                skipped_newer: vec![(2, Version(3))],
                next_key: None,
            }),
            "current local storage should skip the newer key and complete this pivoted snapshot page"
        );
        assert_eq!(continuation_version, Version(2));

        remote.on_rpc_res(messages::RpcRes::FetchSnapshot(continuation, continuation_version));

        assert_eq!(remote.pop_out(), Some(Event::KvEvent(messages::KvEvent::Set(Some(1), 1, 10))));
        assert_eq!(
            remote.pop_out(),
            Some(Event::NetEvent(NetEvent::Unicast(1, RpcEvent::RpcReq(messages::RpcReq::FetchChanged { from: Version(3), count: 1 })))),
            "after the pivoted full sync reports a skipped newer key, the remote should catch up without waiting for another broadcast"
        );
        assert_eq!(remote.pop_out(), None);

        local.on_rpc_req(1, messages::RpcReq::FetchChanged { from: Version(3), count: 1 });
        let Some(Event::NetEvent(NetEvent::Unicast(_, RpcEvent::RpcRes(changed)))) = local.pop_out() else {
            panic!("local store must answer fetch-changed catch-up");
        };
        remote.on_rpc_res(changed);

        assert_eq!(remote.pop_out(), Some(Event::KvEvent(messages::KvEvent::Set(Some(1), 2, 21))));
        assert_eq!(remote.pop_out(), None);
    }

    #[tokio::test]
    async fn replicated_kv_recv_must_not_panic_on_value_serialize_failure() {
        let mut service = ReplicatedKvService::<u16, FailingSerializeValue>::new(test_service(), 10, 10);
        service.set(1, FailingSerializeValue(7));

        let result = std::panic::AssertUnwindSafe(service.recv()).catch_unwind().await;

        assert!(result.is_ok(), "replicated KV service must not panic while serializing caller-provided values for outbound events");
    }
}
