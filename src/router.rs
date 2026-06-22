//! Simple p2p router table
//! The idea behind it is a spread routing, we allow some route loop then it is resolve by 2 method:
//!
//! - Direct rtt always has lower rtt
//! - MAX_HOPS will reject some loop after direct connection disconnected

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::ConnectionId;

use super::PeerId;

const MAX_HOPS: u8 = 6;
const PATH_SWITCH_SCORE_MARGIN: u32 = 2;
const REMOVED_DIRECT_CACHE_SIZE: usize = 8192;
const MAX_SYNC_ENTRIES: usize = 1024;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PathMetric {
    relay_hops: u8,
    rtt_ms: u16,
}

impl From<(u8, u16)> for PathMetric {
    fn from(value: (u8, u16)) -> Self {
        Self { relay_hops: value.0, rtt_ms: value.1 }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouterTableSync(Vec<(PeerId, PathMetric)>);

#[derive(Debug, Default)]
struct PeerMemory {
    best: Option<ConnectionId>,
    paths: BTreeMap<ConnectionId, PathMetric>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RouteAction {
    Local,
    Next(ConnectionId),
}

#[derive(Debug)]
struct RouterTable {
    peer_id: PeerId,
    peers: BTreeMap<PeerId, PeerMemory>,
    directs: BTreeMap<ConnectionId, (PeerId, PathMetric)>,
    removed_directs: LruCache<ConnectionId, ()>,
}

impl RouterTable {
    fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            peers: Default::default(),
            directs: Default::default(),
            removed_directs: LruCache::new(REMOVED_DIRECT_CACHE_SIZE.try_into().expect("should create nonzero cache size")),
        }
    }

    fn local_id(&self) -> PeerId {
        self.peer_id
    }

    fn create_sync(&self, dest: &PeerId) -> RouterTableSync {
        RouterTableSync(
            self.peers
                .iter()
                .map(|(addr, history)| (*addr, history.best_metric().expect("should have best")))
                .filter(|(addr, metric)| !dest.eq(addr) && !self.peer_id.eq(addr) && metric.relay_hops <= MAX_HOPS && !self.route_uses_peer_as_next_hop(addr, dest))
                .take(MAX_SYNC_ENTRIES)
                .collect::<Vec<_>>(),
        )
    }

    fn route_uses_peer_as_next_hop(&self, route: &PeerId, peer: &PeerId) -> bool {
        self.peers
            .get(route)
            .and_then(PeerMemory::best)
            .and_then(|conn| self.directs.get(&conn))
            .is_some_and(|(direct_peer, _)| direct_peer == peer)
    }

    fn apply_sync(&mut self, conn: ConnectionId, sync: RouterTableSync) {
        self.apply_sync_filtered(conn, sync, |_| false);
    }

    fn apply_sync_filtered<F>(&mut self, conn: ConnectionId, sync: RouterTableSync, mut reject_peer: F)
    where
        F: FnMut(&PeerId) -> bool,
    {
        let Some((from_peer, direct_metric)) = self.directs.get(&conn).copied() else {
            log::debug!("[RouterTable] ignore stale sync from removed connection {conn}");
            return;
        };

        if sync.0.len() > MAX_SYNC_ENTRIES {
            log::debug!("[RouterTable] ignore oversized sync from {from_peer}: {} routes exceeds cap {MAX_SYNC_ENTRIES}", sync.0.len());
            return;
        }

        let mut seen_peers = BTreeSet::new();
        let mut new_paths = BTreeMap::<PeerId, PathMetric>::new();
        for (peer, metric) in sync.0 {
            if self.peer_id.eq(&peer) {
                continue;
            }

            if !seen_peers.insert(peer) {
                log::debug!("[RouterTable] ignore malformed sync from {from_peer}: duplicate route to {peer:?}");
                return;
            }

            if reject_peer(&peer) {
                continue;
            }

            let Some(metric) = metric.checked_add(direct_metric) else {
                continue;
            };

            if metric.relay_hops <= MAX_HOPS {
                new_paths.insert(peer, metric);
            }
        }

        // ensure we have memory for each sync paths
        for peer in new_paths.keys() {
            self.peers.entry(*peer).or_default();
        }

        // only loop over peer which don't equal source, because it is direct connection
        for (peer, memory) in self.peers.iter_mut().filter(|(p, _)| !from_peer.eq(p)) {
            let previous = memory.paths.contains_key(&conn);
            let current = new_paths.remove(peer);
            match (previous, current) {
                (true, Some(new_metric)) => {
                    // has update
                    memory.paths.insert(conn, new_metric);
                    Self::select_best_for(peer, memory);
                }
                (true, None) => {
                    // delete
                    log::info!("[RouterTable] remove path for {peer}");
                    memory.paths.remove(&conn);
                    Self::select_best_for(peer, memory);
                }
                (false, Some(new_metric)) => {
                    // new create
                    log::info!("[RouterTable] create path for {peer}");
                    memory.paths.insert(conn, new_metric);
                    Self::select_best_for(peer, memory);
                }
                _ => { //dont changed
                }
            }
        }
        self.peers.retain(|_k, v| v.best().is_some());
    }

    fn set_direct(&mut self, conn: ConnectionId, to: PeerId, ttl_ms: u16) {
        if self.removed_directs.contains(&conn) {
            log::debug!("[RouterTable] ignore direct update from removed connection {conn}");
            return;
        }

        self.directs.insert(conn, (to, (1, ttl_ms).into()));
        let memory = self.peers.entry(to).or_default();
        memory.paths.insert(conn, PathMetric { relay_hops: 0, rtt_ms: ttl_ms });
        Self::select_best_for(&to, memory);
    }

    fn del_direct(&mut self, conn: &ConnectionId) {
        self.removed_directs.get_or_insert(*conn, || ());
        if let Some((to, _)) = self.directs.remove(conn) {
            if let Some(memory) = self.peers.get_mut(&to) {
                memory.paths.remove(conn);
                Self::select_best_for(&to, memory);
                if memory.best().is_none() {
                    self.peers.remove(&to);
                }
            }
        }

        // we also need to remove relayed path which go over this connection
        for (peer, memory) in self.peers.iter_mut() {
            if memory.paths.remove(conn).is_some() {
                Self::select_best_for(peer, memory);
            }
        }
        self.peers.retain(|_k, v| v.best().is_some());
    }

    fn del_peer(&mut self, peer: &PeerId) {
        self.peers.remove(peer);

        let direct_conns = self.directs.iter().filter_map(|(conn, (direct_peer, _))| direct_peer.eq(peer).then_some(*conn)).collect::<Vec<_>>();

        for conn in direct_conns {
            self.removed_directs.get_or_insert(conn, || ());
            self.directs.remove(&conn);
            for (route_peer, memory) in self.peers.iter_mut() {
                if memory.paths.remove(&conn).is_some() {
                    Self::select_best_for(route_peer, memory);
                }
            }
        }
        self.peers.retain(|_k, v| v.best().is_some());
    }

    fn del_learned_peer(&mut self, peer: &PeerId) {
        let direct_conns = self
            .directs
            .iter()
            .filter_map(|(conn, (direct_peer, _))| direct_peer.eq(peer).then_some(*conn))
            .collect::<BTreeSet<_>>();
        if let Some(memory) = self.peers.get_mut(peer) {
            memory.paths.retain(|conn, _metric| direct_conns.contains(conn));
            Self::select_best_for(peer, memory);
        }
        self.peers.retain(|_k, v| v.best().is_some());
    }

    fn is_direct_peer(&self, conn: &ConnectionId, peer: &PeerId) -> bool {
        self.directs.get(conn).is_some_and(|(direct_peer, _)| direct_peer == peer)
    }

    fn action(&self, dest: &PeerId) -> Option<RouteAction> {
        if self.peer_id.eq(dest) {
            Some(RouteAction::Local)
        } else {
            self.peers.get(dest)?.best().map(RouteAction::Next)
        }
    }

    /// Get next remote
    fn next_remote(&self, next: &PeerId) -> Option<(ConnectionId, PathMetric)> {
        let memory = self.peers.get(next)?;
        let best = memory.best()?;
        let metric = memory.best_metric().expect("should have metric");
        Some((best, metric))
    }

    fn select_best_for(dest: &PeerId, memory: &mut PeerMemory) {
        if let Some((new_best, metric)) = memory.select_best() {
            log::info!(
                "[RouterTable] to {dest} select new path over {new_best} with rtt {} ms over {} hop(s)",
                metric.rtt_ms,
                metric.relay_hops
            );
        }
    }

    fn neighbours(&self) -> Vec<(ConnectionId, PeerId, u16)> {
        self.directs.iter().map(|(k, (peer, v))| (*k, *peer, v.rtt_ms)).collect()
    }
}

impl PathMetric {
    fn score(&self) -> u32 {
        self.rtt_ms as u32 + self.relay_hops as u32 * 10
    }

    fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            relay_hops: self.relay_hops.checked_add(rhs.relay_hops)?,
            rtt_ms: self.rtt_ms.checked_add(rhs.rtt_ms)?,
        })
    }
}

impl PeerMemory {
    fn best(&self) -> Option<ConnectionId> {
        self.best
    }

    fn best_metric(&self) -> Option<PathMetric> {
        self.best.map(|p| *self.paths.get(&p).expect("should have metric with best path"))
    }

    fn select_best(&mut self) -> Option<(ConnectionId, PathMetric)> {
        let previous = self.best;
        let direct_available = self.paths.values().any(|metric| metric.relay_hops == 0);
        let previous_candidate = previous.and_then(|peer| {
            self.paths.get(&peer).and_then(|metric| {
                if !direct_available || metric.relay_hops == 0 {
                    Some((peer, metric.score()))
                } else {
                    None
                }
            })
        });

        let Some((mut best_peer, mut best_score)) = previous_candidate.or_else(|| {
            self.paths
                .iter()
                .filter(|(_, metric)| !direct_available || metric.relay_hops == 0)
                .map(|(peer, metric)| (*peer, metric.score()))
                .min_by_key(|(_, score)| *score)
        }) else {
            self.best = None;
            return None;
        };

        for (peer, metric) in &self.paths {
            if direct_available && metric.relay_hops != 0 {
                continue;
            }
            let score = metric.score();
            if score + PATH_SWITCH_SCORE_MARGIN < best_score {
                best_peer = *peer;
                best_score = score;
            }
        }

        self.best = Some(best_peer);
        if self.best != previous {
            let metric = self.best_metric().expect("should have best metric after select success");
            Some((best_peer, metric))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedRouterTable {
    table: Arc<RwLock<RouterTable>>,
}

impl SharedRouterTable {
    pub fn new(address: PeerId) -> Self {
        Self {
            table: Arc::new(RwLock::new(RouterTable::new(address))),
        }
    }

    pub fn local_id(&self) -> PeerId {
        self.table.read().local_id()
    }

    pub fn create_sync(&self, dest: &PeerId) -> RouterTableSync {
        self.table.read().create_sync(dest)
    }

    pub fn apply_sync(&self, conn: ConnectionId, sync: RouterTableSync) {
        self.table.write().apply_sync(conn, sync);
    }

    pub fn apply_sync_filtered<F>(&self, conn: ConnectionId, sync: RouterTableSync, reject_peer: F)
    where
        F: FnMut(&PeerId) -> bool,
    {
        self.table.write().apply_sync_filtered(conn, sync, reject_peer);
    }

    pub fn set_direct(&self, conn: ConnectionId, to: PeerId, ttl_ms: u16) {
        self.table.write().set_direct(conn, to, ttl_ms);
    }

    pub fn del_direct(&self, conn: &ConnectionId) {
        self.table.write().del_direct(conn);
    }

    pub fn del_peer(&self, peer: &PeerId) {
        self.table.write().del_peer(peer);
    }

    pub fn del_learned_peer(&self, peer: &PeerId) {
        self.table.write().del_learned_peer(peer);
    }

    pub fn is_direct_peer(&self, conn: &ConnectionId, peer: &PeerId) -> bool {
        self.table.read().is_direct_peer(conn, peer)
    }

    pub fn action(&self, dest: &PeerId) -> Option<RouteAction> {
        self.table.read().action(dest)
    }

    pub fn next_remote(&self, dest: &PeerId) -> Option<(ConnectionId, PathMetric)> {
        self.table.read().next_remote(dest)
    }

    pub fn neighbours(&self) -> Vec<(ConnectionId, PeerId, u16)> {
        self.table.read().neighbours()
    }
}

#[cfg(test)]
mod tests {
    use crate::{router::RouterTableSync, ConnectionId, PeerId};

    use super::{RouteAction, RouterTable, MAX_HOPS, MAX_SYNC_ENTRIES};

    #[test_log::test]
    fn route_local() {
        let table = RouterTable::new(PeerId(0));
        assert_eq!(table.action(&PeerId(0)), Some(RouteAction::Local));
    }

    #[test_log::test]
    fn create_correct_direct_sync() {
        let mut table = RouterTable::new(PeerId(0));

        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);
        let peer2 = PeerId(2);
        let conn2 = ConnectionId(2);
        let peer3 = PeerId(3);

        table.set_direct(conn1, peer1, 100);
        table.set_direct(conn2, peer2, 200);

        assert_eq!(table.next_remote(&peer1), Some((conn1, (0, 100).into())));
        assert_eq!(table.next_remote(&peer2), Some((conn2, (0, 200).into())));
        assert_eq!(table.next_remote(&peer3), None);

        assert_eq!(table.create_sync(&peer1), RouterTableSync(vec![(peer2, (0, 200).into())]));
        assert_eq!(table.create_sync(&peer2), RouterTableSync(vec![(peer1, (0, 100).into())]));
    }

    #[test_log::test]
    fn apply_correct_direct_sync() {
        let mut table = RouterTable::new(PeerId(0));

        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);
        let peer2 = PeerId(2);
        let peer3 = PeerId(3);
        let peer4 = PeerId(4);
        let conn4 = ConnectionId(4);

        table.set_direct(conn1, peer1, 100);
        table.set_direct(conn4, peer4, 400);

        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (0, 200).into())]));

        // now we have NODO => peer1 => peer2
        assert_eq!(table.next_remote(&peer1), Some((conn1, (0, 100).into())));
        assert_eq!(table.next_remote(&peer2), Some((conn1, (1, 300).into())));
        assert_eq!(table.next_remote(&peer3), None);

        assert_eq!(table.create_sync(&peer1), RouterTableSync(vec![(peer4, (0, 400).into())]));
        assert_eq!(table.create_sync(&peer4), RouterTableSync(vec![(peer1, (0, 100).into()), (peer2, (1, 300).into())]));
    }

    #[test_log::test]
    fn dont_create_sync_over_max_hops() {
        let mut table = RouterTable::new(PeerId(0));

        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);
        let peer2 = PeerId(2);
        let peer3 = PeerId(3);
        let conn3 = ConnectionId(3);

        table.set_direct(conn1, peer1, 100);
        table.set_direct(conn3, peer3, 300);

        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (MAX_HOPS, 200).into())]));
        assert_eq!(table.next_remote(&peer2), None);

        // we shouldn't create sync with peer2 because it over MAX_HOPS
        assert_eq!(table.create_sync(&peer3), RouterTableSync(vec![(peer1, (0, 100).into())]));
    }

    #[test_log::test]
    fn should_remove_relay_path_after_disconnect() {
        let mut table = RouterTable::new(PeerId(0));

        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);

        let peer2 = PeerId(2);

        table.set_direct(conn1, peer1, 100);

        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (MAX_HOPS, 200).into())]));
        assert_eq!(table.next_remote(&peer2), None);

        // after disconnect peer1
        table.del_direct(&conn1);

        // we should not have peer2 anymore
        assert_eq!(table.next_remote(&peer2), None);
    }

    #[test_log::test]
    fn should_ignore_stale_sync_after_direct_disconnect() {
        let mut table = RouterTable::new(PeerId(0));

        let relay = PeerId(1);
        let dest = PeerId(2);
        let conn = ConnectionId(1);

        table.set_direct(conn, relay, 100);
        table.apply_sync(conn, RouterTableSync(vec![(dest, (0, 200).into())]));

        assert_eq!(table.next_remote(&relay), Some((conn, (0, 100).into())));
        assert_eq!(table.next_remote(&dest), Some((conn, (1, 300).into())));

        table.del_direct(&conn);

        assert_eq!(table.next_remote(&relay), None);
        assert_eq!(table.next_remote(&dest), None);

        table.apply_sync(conn, RouterTableSync(vec![(dest, (0, 200).into())]));

        assert_eq!(table.next_remote(&relay), None);
        assert_eq!(table.next_remote(&dest), None);
    }

    #[test_log::test]
    fn should_remove_stopped_peer_path() {
        let mut table = RouterTable::new(PeerId(0));

        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);
        let peer2 = PeerId(2);

        table.set_direct(conn1, peer1, 100);
        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (1, 200).into())]));

        assert!(table.next_remote(&peer2).is_some());

        table.del_peer(&peer2);

        assert_eq!(table.next_remote(&peer2), None);
        assert_eq!(table.next_remote(&peer1), Some((conn1, (0, 100).into())));
    }

    #[test_log::test]
    fn should_remove_learned_peer_path_without_removing_direct_peer() {
        let local = PeerId(0);
        let expired = PeerId(2);
        let relay = PeerId(3);
        let direct_conn = ConnectionId(20);
        let relay_conn = ConnectionId(30);
        let mut table = RouterTable::new(local);

        table.set_direct(relay_conn, relay, 10);
        table.apply_sync(relay_conn, RouterTableSync(vec![(expired, (0, 5).into())]));
        assert_eq!(table.action(&expired), Some(RouteAction::Next(relay_conn)));

        table.del_learned_peer(&expired);
        assert_eq!(table.action(&expired), None);

        table.set_direct(direct_conn, expired, 20);
        table.apply_sync(relay_conn, RouterTableSync(vec![(expired, (0, 5).into())]));
        assert_eq!(table.action(&expired), Some(RouteAction::Next(direct_conn)));

        table.del_learned_peer(&expired);
        assert_eq!(table.action(&expired), Some(RouteAction::Next(direct_conn)));
        assert!(table.is_direct_peer(&direct_conn, &expired));
    }

    #[test_log::test]
    fn should_keep_existing_best_path_on_equal_score() {
        let mut table = RouterTable::new(PeerId(0));

        let relay1 = PeerId(1);
        let relay2 = PeerId(2);
        let dest = PeerId(3);
        let conn1 = ConnectionId(1);
        let conn2 = ConnectionId(2);

        table.set_direct(conn2, relay2, 10);
        table.apply_sync(conn2, RouterTableSync(vec![(dest, (1, 100).into())]));
        assert_eq!(table.next_remote(&dest), Some((conn2, (2, 110).into())));

        table.set_direct(conn1, relay1, 10);
        table.apply_sync(conn1, RouterTableSync(vec![(dest, (1, 100).into())]));

        assert_eq!(
            table.next_remote(&dest),
            Some((conn2, (2, 110).into())),
            "equal-cost route updates should not make the active path jump between connections"
        );
    }

    #[test_log::test]
    fn active_path_should_not_jump_for_tiny_rtt_jitter() {
        let mut table = RouterTable::new(PeerId(0));
        let peer1 = PeerId(1);
        let conn1 = ConnectionId(1);
        let peer2 = PeerId(2);
        let conn2 = ConnectionId(2);
        let dest = PeerId(9);

        table.set_direct(conn1, peer1, 10);
        table.set_direct(conn2, peer2, 10);

        table.apply_sync(conn1, RouterTableSync(vec![(dest, (0, 100).into())]));
        table.apply_sync(conn2, RouterTableSync(vec![(dest, (0, 101).into())]));
        assert_eq!(table.action(&dest), Some(RouteAction::Next(conn1)));

        table.apply_sync(conn2, RouterTableSync(vec![(dest, (0, 99).into())]));

        assert_eq!(
            table.action(&dest),
            Some(RouteAction::Next(conn1)),
            "tiny RTT jitter should not make the active path jump between connections"
        );
    }

    #[test_log::test]
    fn route_sync_must_reject_duplicate_peer_entries() {
        let mut table = RouterTable::new(PeerId(0));
        let relay = PeerId(1);
        let dest = PeerId(9);
        let conn = ConnectionId(1);

        table.set_direct(conn, relay, 10);
        table.apply_sync(conn, RouterTableSync(vec![(dest, (0, 500).into()), (dest, (0, 1).into())]));

        assert_eq!(
            table.next_remote(&dest),
            None,
            "malformed route syncs with duplicate destination peer entries must be rejected instead of accepting the last attacker-controlled metric"
        );
    }

    #[test_log::test]
    fn direct_peer_route_must_not_be_replaced_by_relayed_path() {
        let mut table = RouterTable::new(PeerId(0));
        let relay = PeerId(1);
        let direct_peer = PeerId(2);
        let relay_conn = ConnectionId(1);
        let direct_conn = ConnectionId(2);

        table.set_direct(relay_conn, relay, 1);
        table.set_direct(direct_conn, direct_peer, 1000);
        assert_eq!(table.action(&direct_peer), Some(RouteAction::Next(direct_conn)));

        table.apply_sync(relay_conn, RouterTableSync(vec![(direct_peer, (0, 1).into())]));

        assert_eq!(
            table.action(&direct_peer),
            Some(RouteAction::Next(direct_conn)),
            "traffic to a directly connected peer must stay on that authenticated direct connection"
        );
    }

    #[test_log::test]
    fn should_not_store_or_advertise_route_to_local_peer() {
        let local = PeerId(0);
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);
        let conn1 = ConnectionId(1);
        let conn2 = ConnectionId(2);
        let mut table = RouterTable::new(local);

        table.set_direct(conn1, peer1, 10);
        table.set_direct(conn2, peer2, 10);
        table.apply_sync(conn1, RouterTableSync(vec![(local, (0, 1).into())]));

        assert_eq!(table.next_remote(&local), None, "router must not store relay routes to the local peer");
        assert!(
            !table.create_sync(&peer2).0.iter().any(|(peer, _)| *peer == local),
            "router must not advertise poisoned routes to the local peer"
        );
    }

    #[test_log::test]
    fn should_reject_over_max_hops_for_forwarding() {
        let mut table = RouterTable::new(PeerId(0));
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);
        let conn1 = ConnectionId(1);

        table.set_direct(conn1, peer1, 10);
        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (MAX_HOPS, 10).into())]));

        assert_eq!(table.next_remote(&peer2), None, "over-MAX_HOPS paths should not be usable for local forwarding");
    }

    #[test_log::test]
    fn should_not_advertise_route_back_to_next_hop() {
        let mut table = RouterTable::new(PeerId(0));
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);
        let conn1 = ConnectionId(1);

        table.set_direct(conn1, peer1, 10);
        table.apply_sync(conn1, RouterTableSync(vec![(peer2, (0, 10).into())]));

        assert_eq!(
            table.create_sync(&peer1),
            RouterTableSync(vec![]),
            "route learned from a peer should not be advertised back to that peer"
        );
    }

    #[test_log::test]
    fn should_reject_excessive_route_sync_entries() {
        let mut table = RouterTable::new(PeerId(0));
        let conn1 = ConnectionId(1);

        table.set_direct(conn1, PeerId(1), 10);
        let huge_sync = (10_000..11_100).map(|id| (PeerId(id), (0, 1).into())).collect::<Vec<_>>();
        table.apply_sync(conn1, RouterTableSync(huge_sync));

        assert_eq!(table.next_remote(&PeerId(10_000)), None, "oversized route syncs must be rejected without creating route state");
        assert_eq!(table.create_sync(&PeerId(1)), RouterTableSync(vec![]), "oversized route syncs must not be re-advertised");
    }

    #[test_log::test]
    fn create_sync_must_cap_outbound_route_entries() {
        let mut table = RouterTable::new(PeerId(0));

        for id in 1..=1_100 {
            table.set_direct(ConnectionId(id), PeerId(id), 10);
        }

        let sync = table.create_sync(&PeerId(2_000));

        assert_eq!(sync.0.len(), MAX_SYNC_ENTRIES, "outbound route syncs must cap the number of advertised entries");
    }

    #[test_log::test]
    fn should_reject_overflowing_route_sync_metric_without_panic() {
        let mut table = RouterTable::new(PeerId(0));
        let conn1 = ConnectionId(1);
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);

        table.set_direct(conn1, peer1, 10);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            table.apply_sync(conn1, RouterTableSync(vec![(peer2, (u8::MAX, u16::MAX).into())]));
        }));

        assert!(result.is_ok(), "untrusted route metrics must not overflow or panic during path composition");
        assert_eq!(table.next_remote(&peer2), None, "overflowing route metrics must be rejected, not wrapped into a usable path");
    }

    #[test_log::test]
    fn should_not_overflow_score_during_best_path_selection() {
        let mut table = RouterTable::new(PeerId(0));
        let peer1 = PeerId(1);
        let peer2 = PeerId(2);
        let conn1 = ConnectionId(1);
        let conn2 = ConnectionId(2);

        table.set_direct(conn1, peer1, 0);
        table.set_direct(conn2, peer2, 50);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            table.apply_sync(conn1, RouterTableSync(vec![(peer2, (1, 65525).into())]));
        }));

        assert!(result.is_ok(), "route score calculation must not overflow or panic");
        assert_eq!(
            table.next_remote(&peer2),
            Some((conn2, (0, 50).into())),
            "overflowing route score must not wrap and beat the direct path"
        );
    }
}
