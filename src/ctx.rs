use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::anyhow;
use futures::{stream::FuturesUnordered, StreamExt};
use lru::LruCache;
use parking_lot::RwLock;
use tokio::sync::mpsc::{error::TrySendError, Sender};

use crate::{
    msg::{BroadcastMsgId, P2pServiceId, PeerMessage},
    peer::{PeerConnectionAlias, PeerConnectionMetric},
    router::{RouteAction, SharedRouterTable},
    service::P2pServiceEvent,
    stream::P2pQuicStream,
    ConnectionId, PeerId,
};

const BROADCAST_ADMISSION_TIMEOUT: Duration = Duration::from_millis(25);
pub(crate) const BROADCAST_DEDUP_WINDOW_SIZE: usize = 8192;
const PEER_STOPPED_DEDUP_CACHE_SIZE: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BroadcastDedupKey {
    source: PeerId,
    service_id: P2pServiceId,
    msg_id: BroadcastMsgId,
}

#[derive(Debug)]
struct BroadcastDedupCache {
    current: LruCache<BroadcastDedupKey, ()>,
    previous: LruCache<BroadcastDedupKey, ()>,
}

impl BroadcastDedupCache {
    fn new() -> Self {
        Self {
            current: LruCache::new(BROADCAST_DEDUP_WINDOW_SIZE.try_into().expect("should ok")),
            previous: LruCache::new(BROADCAST_DEDUP_WINDOW_SIZE.try_into().expect("should ok")),
        }
    }

    fn check(&mut self, key: BroadcastDedupKey) -> bool {
        if self.current.contains(&key) || self.previous.contains(&key) {
            return false;
        }

        if self.current.len() == BROADCAST_DEDUP_WINDOW_SIZE {
            self.previous = std::mem::replace(&mut self.current, LruCache::new(BROADCAST_DEDUP_WINDOW_SIZE.try_into().expect("should ok")));
        }

        self.current.put(key, ());
        true
    }
}

#[derive(Debug)]
struct SharedCtxInternal {
    conns: HashMap<ConnectionId, PeerConnectionAlias>,
    conn_metrics: HashMap<ConnectionId, (PeerId, PeerConnectionMetric)>,
    received_broadcast_msg: BroadcastDedupCache,
    received_peer_stopped_msg: LruCache<PeerId, ()>,
    services: [Option<Sender<P2pServiceEvent>>; 256],
}

impl SharedCtxInternal {
    fn set_service(&mut self, service_id: P2pServiceId, tx: Sender<P2pServiceEvent>) {
        let Some(index) = service_id.as_service_index() else {
            log::warn!("[SharedCtx] reject out-of-range service id {service_id}");
            return;
        };
        if self.services[index].as_ref().is_some_and(|existing| !existing.is_closed()) {
            log::warn!("[SharedCtx] reject duplicate live service id {service_id}");
            return;
        }
        self.services[index] = Some(tx);
    }

    fn get_service(&self, service_id: &P2pServiceId) -> Option<Sender<P2pServiceEvent>> {
        let Some(index) = service_id.as_service_index() else {
            log::warn!("[SharedCtx] reject out-of-range service id {service_id}");
            return None;
        };
        self.services[index].clone()
    }

    fn service_senders(&self) -> Vec<Sender<P2pServiceEvent>> {
        self.services.iter().filter_map(Clone::clone).collect()
    }

    fn register_conn(&mut self, conn: ConnectionId, alias: PeerConnectionAlias) {
        self.clear_peer_stopped_msg(alias.to_id());
        self.conns.insert(conn, alias);
    }

    fn unregister_conn(&mut self, conn: &ConnectionId) {
        self.conns.remove(conn);
        self.conn_metrics.remove(conn);
    }

    fn conn(&self, conn: &ConnectionId) -> Option<PeerConnectionAlias> {
        self.conns.get(conn).cloned()
    }

    fn conns(&self) -> Vec<PeerConnectionAlias> {
        self.conns.values().cloned().collect::<Vec<_>>()
    }

    fn update_conn_metrics(&mut self, conn: &ConnectionId, peer: PeerId, metrics: PeerConnectionMetric) {
        self.conn_metrics.insert(*conn, (peer, metrics));
    }

    fn metrics(&self) -> Vec<(ConnectionId, PeerId, PeerConnectionMetric)> {
        let mut ret = vec![];
        for (conn, (peer, metrics)) in self.conn_metrics.clone() {
            ret.push((conn, peer, metrics));
        }

        ret
    }

    /// check if we already got the message
    /// if is not, it return true and save to cache list
    /// if already it return false and do nothing
    fn check_broadcast_msg(&mut self, source: PeerId, service_id: P2pServiceId, msg_id: BroadcastMsgId) -> bool {
        self.received_broadcast_msg.check(BroadcastDedupKey { source, service_id, msg_id })
    }

    fn try_mark_peer_stopped_msg_after<F>(&mut self, peer_id: PeerId, admit: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        if self.received_peer_stopped_msg.contains(&peer_id) {
            false
        } else if admit() {
            self.received_peer_stopped_msg.put(peer_id, ());
            true
        } else {
            false
        }
    }

    fn clear_peer_stopped_msg(&mut self, peer_id: PeerId) {
        self.received_peer_stopped_msg.pop(&peer_id);
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::channel;

    use super::*;

    #[test]
    fn get_service_must_reject_out_of_range_id_without_panicking() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));

        assert!(ctx.get_service(&P2pServiceId::from(256u16)).is_none());
    }

    #[test]
    fn set_service_must_keep_valid_highest_id() {
        let mut ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (tx, _rx) = channel(1);

        ctx.set_service(P2pServiceId::from(255u16), tx);

        assert!(ctx.get_service(&P2pServiceId::from(255u16)).is_some());
    }

    #[test]
    fn peer_stopped_admission_must_deduplicate_per_stopped_peer() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let stopped = PeerId::from(2);

        assert!(ctx.try_mark_peer_stopped_msg_after(stopped, || true));
        assert!(!ctx.try_mark_peer_stopped_msg_after(stopped, || true));
        assert!(ctx.try_mark_peer_stopped_msg_after(PeerId::from(3), || true));
    }

    #[test]
    fn peer_stopped_admission_must_not_mark_failed_admit() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let stopped = PeerId::from(2);
        let mut duplicate_admit_calls = 0;

        assert!(!ctx.try_mark_peer_stopped_msg_after(stopped, || false));
        assert!(ctx.try_mark_peer_stopped_msg_after(stopped, || true));
        assert!(!ctx.try_mark_peer_stopped_msg_after(stopped, || {
            duplicate_admit_calls += 1;
            true
        }));
        assert_eq!(duplicate_admit_calls, 0, "duplicate PeerStopped messages must not run admission again");
    }

    #[tokio::test]
    async fn peer_disconnected_notification_must_retry_when_service_queue_full() {
        let mut ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (tx, mut rx) = channel(1);
        let disconnected = PeerId::from(2);

        tx.try_send(P2pServiceEvent::Unicast(PeerId::from(9), b"filler".to_vec()))
            .expect("test service queue should accept filler");
        ctx.set_service(P2pServiceId::from(7), tx);

        ctx.try_send_peer_disconnected_to_services(disconnected);

        assert!(matches!(rx.recv().await, Some(P2pServiceEvent::Unicast(_, _))));
        assert_eq!(
            tokio::time::timeout(Duration::from_millis(100), rx.recv()).await,
            Ok(Some(P2pServiceEvent::PeerDisconnected(disconnected))),
            "peer disconnect notifications must survive transient full service queues"
        );
    }
}

#[derive(Debug, Clone)]
pub struct SharedCtx {
    local_id: PeerId,
    ctx: Arc<RwLock<SharedCtxInternal>>,
    router: SharedRouterTable,
}

impl SharedCtx {
    pub fn new(local_id: PeerId, router: SharedRouterTable) -> Self {
        Self {
            local_id,
            ctx: Arc::new(RwLock::new(SharedCtxInternal {
                conns: Default::default(),
                conn_metrics: Default::default(),
                received_broadcast_msg: BroadcastDedupCache::new(),
                received_peer_stopped_msg: LruCache::new(PEER_STOPPED_DEDUP_CACHE_SIZE.try_into().expect("should ok")),
                services: std::array::from_fn(|_| None),
            })),
            router,
        }
    }

    pub fn local_id(&self) -> PeerId {
        self.local_id
    }

    pub(super) fn set_service(&mut self, service_id: P2pServiceId, tx: Sender<P2pServiceEvent>) {
        self.ctx.write().set_service(service_id, tx);
    }

    pub fn register_conn(&self, conn: ConnectionId, alias: PeerConnectionAlias) {
        self.ctx.write().register_conn(conn, alias);
    }

    pub fn unregister_conn(&self, conn: &ConnectionId) {
        self.ctx.write().unregister_conn(conn);
    }

    pub fn conn(&self, conn: &ConnectionId) -> Option<PeerConnectionAlias> {
        self.ctx.read().conn(conn)
    }

    pub fn conns(&self) -> Vec<PeerConnectionAlias> {
        self.ctx.read().conns()
    }

    pub fn update_metrics(&self, conn: &ConnectionId, peer: PeerId, metrics: PeerConnectionMetric) {
        self.ctx.write().update_conn_metrics(conn, peer, metrics);
    }

    pub fn metrics(&self) -> Vec<(ConnectionId, PeerId, PeerConnectionMetric)> {
        self.ctx.read().metrics()
    }

    pub fn router(&self) -> &SharedRouterTable {
        &self.router
    }

    pub fn get_service(&self, service_id: &P2pServiceId) -> Option<Sender<P2pServiceEvent>> {
        self.ctx.read().get_service(service_id)
    }

    pub fn try_send_peer_disconnected_to_services(&self, peer: PeerId) {
        let services = self.ctx.read().service_senders();
        for service in services {
            match service.try_send(P2pServiceEvent::PeerDisconnected(peer)) {
                Ok(()) => {}
                Err(TrySendError::Full(event)) => {
                    tokio::spawn(async move {
                        match tokio::time::timeout(BROADCAST_ADMISSION_TIMEOUT, service.send(event)).await {
                            Ok(Ok(())) => {}
                            Ok(Err(err)) => log::warn!("[SharedCtx] send peer disconnected for {peer} to service failed: {err}"),
                            Err(_) => log::warn!("[SharedCtx] send peer disconnected for {peer} to service timed out"),
                        }
                    });
                }
                Err(TrySendError::Closed(_)) => {
                    log::warn!("[SharedCtx] send peer disconnected for {peer} to service failed: receiving half closed");
                }
            }
        }
    }

    /// check if we already got the message
    /// if is not, it return true and save to cache list
    /// if already it return false and do nothing
    pub fn check_broadcast_msg(&self, source: PeerId, service_id: P2pServiceId, msg_id: BroadcastMsgId) -> bool {
        self.ctx.write().check_broadcast_msg(source, service_id, msg_id)
    }

    pub fn try_mark_peer_stopped_msg_after<F>(&self, peer_id: PeerId, admit: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        self.ctx.write().try_mark_peer_stopped_msg_after(peer_id, admit)
    }

    pub fn try_send_unicast(&self, service_id: P2pServiceId, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        let next = self.router.action(&dest).ok_or(anyhow!("route not found"))?;
        match next {
            RouteAction::Local => {
                anyhow::bail!("unsupported send to local node")
            }
            RouteAction::Next(next) => {
                let source = self.router.local_id();
                self.conn(&next)
                    .ok_or(anyhow!("peer not found {next}"))?
                    .try_send(PeerMessage::Unicast(source, dest, service_id, data))?;
                Ok(())
            }
        }
    }

    pub async fn send_unicast(&self, service_id: P2pServiceId, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        let next = self.router.action(&dest).ok_or(anyhow!("route not found"))?;
        match next {
            RouteAction::Local => {
                anyhow::bail!("unsupported send to local node")
            }
            RouteAction::Next(next) => {
                let source = self.router.local_id();
                let conn = self.conn(&next).ok_or(anyhow!("peer not found {next}"))?;
                conn.send_unicast_with_ack(source, dest, service_id, data).await?;
                Ok(())
            }
        }
    }

    pub(crate) async fn send_unicast_unacked(&self, service_id: P2pServiceId, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        let next = self.router.action(&dest).ok_or(anyhow!("route not found"))?;
        match next {
            RouteAction::Local => {
                anyhow::bail!("unsupported send to local node")
            }
            RouteAction::Next(next) => {
                let source = self.router.local_id();
                self.conn(&next)
                    .ok_or(anyhow!("peer not found {next}"))?
                    .send(PeerMessage::Unicast(source, dest, service_id, data))
                    .await
            }
        }
    }

    pub fn try_send_broadcast(&self, service_id: P2pServiceId, data: Vec<u8>) -> anyhow::Result<usize> {
        let msg_id = BroadcastMsgId::rand();
        let source = self.router.local_id();
        let conns = self.conns();
        log::debug!("[ShareCtx] broadcast to {conns:?} connections");
        if conns.is_empty() {
            anyhow::bail!("[ShareCtx] broadcast has no connected peers");
        }

        let mut accepted = 0;
        for conn_alias in conns {
            match conn_alias.try_send(PeerMessage::Broadcast(source, service_id, msg_id, data.clone())) {
                Ok(()) => accepted += 1,
                Err(err) => log::warn!("[ShareCtx] broadcast data over peer alias failed: {err}"),
            }
        }
        if accepted == 0 {
            anyhow::bail!("[ShareCtx] broadcast rejected by all peer aliases");
        }

        self.check_broadcast_msg(source, service_id, msg_id);
        Ok(accepted)
    }

    pub async fn send_broadcast(&self, service_id: P2pServiceId, data: Vec<u8>) -> anyhow::Result<usize> {
        let msg_id = BroadcastMsgId::rand();
        let source = self.router.local_id();
        let conns = self.conns();
        log::debug!("[ShareCtx] broadcast to {conns:?} connections");
        if conns.is_empty() {
            anyhow::bail!("[ShareCtx] broadcast has no connected peers");
        }

        let mut pending_sends = FuturesUnordered::new();
        for conn_alias in conns {
            let msg = PeerMessage::Broadcast(source, service_id, msg_id, data.clone());
            pending_sends.push(async move { tokio::time::timeout(BROADCAST_ADMISSION_TIMEOUT, conn_alias.send(msg)).await });
        }

        let mut accepted = 0;
        while let Some(result) = pending_sends.next().await {
            match result {
                Ok(Ok(())) => accepted += 1,
                Ok(Err(err)) => log::warn!("[ShareCtx] broadcast data over peer alias failed: {err}"),
                Err(_) => log::warn!("[ShareCtx] broadcast data over peer alias timed out"),
            }
        }
        if accepted == 0 {
            anyhow::bail!("[ShareCtx] broadcast rejected by all peer aliases");
        }

        self.check_broadcast_msg(source, service_id, msg_id);
        Ok(accepted)
    }

    pub async fn open_stream(&self, service: P2pServiceId, dest: PeerId, meta: Vec<u8>) -> anyhow::Result<P2pQuicStream> {
        let next = self.router.action(&dest).ok_or(anyhow!("route not found"))?;
        match next {
            RouteAction::Local => {
                anyhow::bail!("unsupported open_stream to local node")
            }
            RouteAction::Next(next) => {
                let source = self.router.local_id();
                Ok(self.conn(&next).ok_or(anyhow!("peer not found {next}"))?.open_stream(service, source, dest, meta).await?)
            }
        }
    }
}
