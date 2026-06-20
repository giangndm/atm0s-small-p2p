use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::anyhow;
use lru::LruCache;
use parking_lot::RwLock;
use tokio::sync::mpsc::Sender;

use crate::{
    msg::{BroadcastMsgId, P2pServiceId, PeerMessage},
    peer::{PeerConnectionAlias, PeerConnectionMetric},
    router::{RouteAction, SharedRouterTable},
    service::P2pServiceEvent,
    stream::P2pQuicStream,
    ConnectionId, PeerId,
};

const BROADCAST_ADMISSION_TIMEOUT: Duration = Duration::from_millis(25);

#[derive(Debug)]
struct SharedCtxInternal {
    conns: HashMap<ConnectionId, PeerConnectionAlias>,
    conn_metrics: HashMap<ConnectionId, (PeerId, PeerConnectionMetric)>,
    received_broadcast_msg: LruCache<BroadcastMsgId, ()>,
    received_peer_stopped_msg: LruCache<PeerId, ()>,
    services: [Option<Sender<P2pServiceEvent>>; 256],
}

impl SharedCtxInternal {
    fn set_service(&mut self, service_id: P2pServiceId, tx: Sender<P2pServiceEvent>) {
        let index = service_id.as_service_index().expect("Service ID out of range");
        assert!(self.services[index].is_none(), "Service ID already used");
        self.services[index] = Some(tx);
    }

    fn get_service(&self, service_id: &P2pServiceId) -> Option<Sender<P2pServiceEvent>> {
        let Some(index) = service_id.as_service_index() else {
            log::warn!("[SharedCtx] reject out-of-range service id {service_id}");
            return None;
        };
        self.services[index].clone()
    }

    fn register_conn(&mut self, conn: ConnectionId, alias: PeerConnectionAlias) {
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
    fn check_broadcast_msg(&mut self, id: BroadcastMsgId) -> bool {
        if !self.received_broadcast_msg.contains(&id) {
            self.received_broadcast_msg.get_or_insert(id, || ());
            true
        } else {
            false
        }
    }

    fn check_peer_stopped_msg(&mut self, peer_id: PeerId) -> bool {
        if !self.received_peer_stopped_msg.contains(&peer_id) {
            self.received_peer_stopped_msg.get_or_insert(peer_id, || ());
            true
        } else {
            false
        }
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

        assert!(ctx.check_peer_stopped_msg(stopped));
        assert!(!ctx.check_peer_stopped_msg(stopped));
        assert!(ctx.check_peer_stopped_msg(PeerId::from(3)));
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
                received_broadcast_msg: LruCache::new(8192.try_into().expect("should ok")),
                received_peer_stopped_msg: LruCache::new(8192.try_into().expect("should ok")),
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

    /// check if we already got the message
    /// if is not, it return true and save to cache list
    /// if already it return false and do nothing
    pub fn check_broadcast_msg(&self, id: BroadcastMsgId) -> bool {
        self.ctx.write().check_broadcast_msg(id)
    }

    pub fn check_peer_stopped_msg(&self, peer_id: PeerId) -> bool {
        self.ctx.write().check_peer_stopped_msg(peer_id)
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
                self.conn(&next)
                    .ok_or(anyhow!("peer not found {next}"))?
                    .send(PeerMessage::Unicast(source, dest, service_id, data))
                    .await?;
                Ok(())
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

        self.check_broadcast_msg(msg_id);
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

        let mut accepted = 0;
        for conn_alias in conns {
            let msg = PeerMessage::Broadcast(source, service_id, msg_id, data.clone());
            match tokio::time::timeout(BROADCAST_ADMISSION_TIMEOUT, conn_alias.send(msg)).await {
                Ok(Ok(())) => accepted += 1,
                Ok(Err(err)) => log::warn!("[ShareCtx] broadcast data over peer alias failed: {err}"),
                Err(_) => log::warn!("[ShareCtx] broadcast data over peer alias timed out"),
            }
        }
        if accepted == 0 {
            anyhow::bail!("[ShareCtx] broadcast rejected by all peer aliases");
        }

        self.check_broadcast_msg(msg_id);
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
