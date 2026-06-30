use std::collections::HashMap;

use crate::{peer::PeerConnection, ConnectionId, PeerId};

#[derive(Default)]
pub struct NetworkNeighbours {
    conns: HashMap<ConnectionId, PeerConnection>,
}

impl NetworkNeighbours {
    pub fn insert(&mut self, conn_id: ConnectionId, conn: PeerConnection) {
        self.conns.insert(conn_id, conn);
    }

    pub fn get(&self, conn_id: &ConnectionId) -> Option<&PeerConnection> {
        self.conns.get(conn_id)
    }

    pub fn has_peer(&self, peer: &PeerId) -> bool {
        self.conns.values().any(|c| c.is_connected() && c.peer_id().eq(&Some(*peer)))
    }

    pub fn has_peer_connection_attempt(&self, peer: &PeerId) -> bool {
        self.conns.values().any(|c| c.peer_id().eq(&Some(*peer)))
    }

    pub fn mark_connected(&mut self, conn_id: &ConnectionId, peer: PeerId) -> Option<()> {
        self.conns.get_mut(conn_id)?.set_connected(peer);
        Some(())
    }

    pub fn remove(&mut self, conn_id: &ConnectionId) -> Option<()> {
        self.conns.remove(conn_id)?;
        Some(())
    }

    pub fn connected_conns(&self) -> impl Iterator<Item = &PeerConnection> {
        self.conns.values().filter(|c| c.is_connected())
    }

    #[allow(dead_code)]
    pub fn pending_unauthenticated_inbound_count<F>(&self, mut is_authenticated: F) -> usize
    where
        F: FnMut(&ConnectionId) -> bool,
    {
        self.conns
            .iter()
            .filter(|(conn_id, c)| !c.is_connected() && c.peer_id().is_none() && !is_authenticated(conn_id))
            .count()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.conns.len()
    }
}
