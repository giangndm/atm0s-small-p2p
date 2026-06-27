use derive_more::derive::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use super::{discovery::PeerDiscoverySync, router::RouterTableSync, PeerId};

#[derive(Debug, Display, PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Copy)]
pub struct BroadcastMsgId(u64);

#[derive(Debug, Display, PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Copy, From)]
pub struct UnicastAckId(u64);

#[derive(Debug, Display, PartialEq, Deref, Eq, Hash, Serialize, Deserialize, From, Clone, Copy)]
pub struct P2pServiceId(u16);

impl P2pServiceId {
    pub(crate) fn as_service_index(self) -> Option<usize> {
        let index = *self as usize;
        (index < 256).then_some(index)
    }
}

impl BroadcastMsgId {
    pub fn rand() -> Self {
        Self(rand::random())
    }
}

impl UnicastAckId {
    pub fn rand() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PeerMessage {
    Sync { route: RouterTableSync, advertise: PeerDiscoverySync },
    PeerStopped(PeerId),
    Broadcast(PeerId, P2pServiceId, BroadcastMsgId, Vec<u8>),
    Unicast(PeerId, PeerId, P2pServiceId, Vec<u8>),
    UnicastWithAck(UnicastAckId, PeerId, PeerId, P2pServiceId, Vec<u8>),
    UnicastAck(UnicastAckId, Result<(), String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamConnectReq {
    pub source: PeerId,
    pub dest: PeerId,
    pub service: P2pServiceId,
    pub meta: Vec<u8>,
    pub defer_delivery: bool,
}

pub type StreamConnectRes = Result<(), String>;

#[cfg(test)]
mod tests {
    use crate::{
        ctx::{SharedCtx, BROADCAST_DEDUP_WINDOW_SIZE},
        router::SharedRouterTable,
    };

    use super::*;

    #[test]
    fn broadcast_replay_must_not_be_accepted_after_dedup_cache_eviction() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let replayed = BroadcastMsgId(7);
        let source = PeerId::from(2);
        let service_id = P2pServiceId::from(0);

        assert!(ctx.check_broadcast_msg(source, service_id, replayed));
        assert!(!ctx.check_broadcast_msg(source, service_id, replayed));

        for id in 8..(8 + BROADCAST_DEDUP_WINDOW_SIZE as u64) {
            assert!(ctx.check_broadcast_msg(source, service_id, BroadcastMsgId(id)));
        }

        assert!(
            !ctx.check_broadcast_msg(source, service_id, replayed),
            "an already accepted broadcast id must be rejected within the configured freshness window after cache churn"
        );
    }

    #[test]
    fn broadcast_dedup_contains_must_not_insert() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let msg_id = BroadcastMsgId(7);
        let source = PeerId::from(2);
        let service_id = P2pServiceId::from(0);

        assert!(!ctx.has_broadcast_msg(source, service_id, msg_id));
        assert!(ctx.check_broadcast_msg(source, service_id, msg_id));
        assert!(ctx.has_broadcast_msg(source, service_id, msg_id));
        assert!(!ctx.check_broadcast_msg(source, service_id, msg_id));
    }
}
