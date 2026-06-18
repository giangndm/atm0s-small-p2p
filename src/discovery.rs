use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{NetworkAddress, PeerAddress};

use super::PeerId;

const TIMEOUT_AFTER: u64 = 30_000;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDiscoverySync(Vec<(PeerId, u64, NetworkAddress)>);

#[derive(Debug, Default)]
pub struct PeerDiscovery {
    seeds: Vec<PeerAddress>,
    local: Option<(PeerId, NetworkAddress)>,
    remotes: BTreeMap<PeerId, (u64, NetworkAddress)>,
    stopped: BTreeMap<PeerId, u64>,
}

impl PeerDiscovery {
    pub fn new(seeds: Vec<PeerAddress>) -> Self {
        Self {
            seeds,
            local: None,
            remotes: Default::default(),
            stopped: Default::default(),
        }
    }

    pub fn enable_local(&mut self, peer_id: PeerId, address: NetworkAddress) {
        log::info!("[PeerDiscovery] enable local as {address}");
        self.local = Some((peer_id, address));
    }

    pub fn clear_timeout(&mut self, now_ms: u64) {
        self.remotes.retain(|peer, (last_updated, _addr)| {
            if *last_updated + TIMEOUT_AFTER > now_ms {
                true
            } else {
                log::info!("[PeerDiscovery] remove timeout {peer}");
                false
            }
        });
        self.stopped.retain(|peer, stopped_at| {
            if *stopped_at + TIMEOUT_AFTER > now_ms {
                true
            } else {
                log::info!("[PeerDiscovery] clear stopped tombstone {peer}");
                false
            }
        });
    }

    pub fn remove_remote(&mut self, now_ms: u64, peer: &PeerId) {
        if self.seeds.iter().any(|seed| seed.peer_id().eq(peer)) {
            return;
        }
        self.stopped.insert(*peer, now_ms);
        if self.remotes.remove(peer).is_some() {
            log::info!("[PeerDiscovery] remove stopped peer {peer}");
        }
    }

    pub fn create_sync_for(&self, now_ms: u64, dest: &PeerId) -> PeerDiscoverySync {
        let iter = self.local.iter().map(|(p, addr)| (*p, now_ms, addr.clone()));
        PeerDiscoverySync(
            self.remotes
                .iter()
                .filter(|(k, _)| !dest.eq(k))
                .map(|(k, (v1, v2))| (*k, *v1, v2.clone()))
                .chain(iter)
                .collect::<Vec<_>>(),
        )
    }

    pub fn apply_sync(&mut self, now_ms: u64, sync: PeerDiscoverySync) {
        log::debug!("[PeerDiscovery] apply sync with addrs: {:?}", sync.0);
        for (peer, last_updated, address) in sync.0.into_iter() {
            if self.stopped.get(&peer).is_some_and(|stopped_at| *stopped_at + TIMEOUT_AFTER > now_ms) {
                continue;
            }
            if last_updated + TIMEOUT_AFTER > now_ms {
                #[allow(clippy::collapsible_else_if)]
                if self.remotes.insert(peer, (last_updated, address)).is_none() {
                    log::info!("[PeerDiscovery] added new peer {peer}");
                }
            }
        }
    }
    pub fn remotes(&self) -> impl Iterator<Item = PeerAddress> + '_ {
        self.remotes.iter().map(|(p, (_, a))| PeerAddress(*p, a.clone())).chain(self.seeds.iter().cloned())
    }
}

#[cfg(test)]
mod test {
    use crate::{discovery::PeerDiscoverySync, PeerAddress, PeerId};

    use super::{PeerDiscovery, TIMEOUT_AFTER};

    fn peer_addr(addr: &str) -> PeerAddress {
        addr.parse().expect("should parse peer address")
    }

    #[test_log::test]
    fn create_local_sync() {
        let mut discovery = PeerDiscovery::default();

        let peer1 = PeerId(1);
        let peer1_addr: PeerAddress = "1@127.0.0.1:9000".parse().expect("should parse peer address");

        let peer2 = PeerId(2);

        assert_eq!(discovery.create_sync_for(0, &peer2), PeerDiscoverySync(vec![]));

        discovery.enable_local(peer1, peer1_addr.network_address().clone());

        assert_eq!(discovery.create_sync_for(100, &peer2), PeerDiscoverySync(vec![(peer1, 100, peer1_addr.network_address().clone())]));
        assert_eq!(discovery.remotes().next(), None);
    }

    #[test_log::test]
    fn apply_sync() {
        let mut discovery = PeerDiscovery::default();

        let peer1 = PeerId(1);
        let peer1_addr: PeerAddress = "1@127.0.0.1:9000".parse().expect("should parse peer address");

        let peer2 = PeerId(2);

        discovery.apply_sync(100, PeerDiscoverySync(vec![(peer1, 90, peer1_addr.network_address().clone())]));

        assert_eq!(discovery.create_sync_for(100, &peer2), PeerDiscoverySync(vec![(peer1, 90, peer1_addr.network_address().clone())]));
        assert_eq!(discovery.create_sync_for(100, &peer1), PeerDiscoverySync(vec![]));
        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![peer1_addr]);
    }

    #[test_log::test]
    fn apply_sync_timeout() {
        let mut discovery = PeerDiscovery::default();

        let peer1 = PeerId(1);
        let peer1_addr: PeerAddress = "1@127.0.0.1:9000".parse().expect("should parse peer address");

        let peer2 = PeerId(2);

        discovery.apply_sync(TIMEOUT_AFTER + 100, PeerDiscoverySync(vec![(peer1, 100, peer1_addr.network_address().clone())]));

        assert_eq!(discovery.create_sync_for(100, &peer2), PeerDiscoverySync(vec![]));
        assert_eq!(discovery.create_sync_for(100, &peer1), PeerDiscoverySync(vec![]));
        assert_eq!(discovery.remotes().next(), None);
    }

    #[test_log::test]
    fn apply_sync_must_not_overwrite_newer_discovery_with_stale_advertisement() {
        let mut discovery = PeerDiscovery::default();
        let peer = PeerId(1);
        let fresh_addr = peer_addr("1@127.0.0.1:9001");
        let stale_addr = peer_addr("1@127.0.0.1:9000");

        discovery.apply_sync(200, PeerDiscoverySync(vec![(peer, 200, fresh_addr.network_address().clone())]));
        discovery.apply_sync(210, PeerDiscoverySync(vec![(peer, 100, stale_addr.network_address().clone())]));

        assert_eq!(
            discovery.remotes().collect::<Vec<_>>(),
            vec![fresh_addr],
            "stale discovery advertisements must not overwrite newer peer addresses"
        );
    }

    #[test_log::test]
    fn clear_timeout() {
        let mut discovery = PeerDiscovery::default();

        let peer1 = PeerId(1);
        let peer1_addr = peer_addr("1@127.0.0.1:9000");

        discovery.apply_sync(100, PeerDiscoverySync(vec![(peer1, 90, peer1_addr.network_address().clone())]));

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![peer1_addr]);

        discovery.clear_timeout(TIMEOUT_AFTER + 90);

        assert_eq!(discovery.remotes().next(), None);
    }

    #[test_log::test]
    fn non_seed_discovered_peer_ages_out_but_seed_remains_retryable() {
        let seed = peer_addr("1@127.0.0.1:9000");
        let discovered = peer_addr("2@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::new(vec![seed.clone()]);

        discovery.apply_sync(100, PeerDiscoverySync(vec![(discovered.peer_id(), 100, discovered.network_address().clone())]));

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![discovered.clone(), seed.clone()]);

        discovery.clear_timeout(100 + TIMEOUT_AFTER);

        assert_eq!(
            discovery.remotes().collect::<Vec<_>>(),
            vec![seed],
            "discovered non-seed peers should expire after a long outage, while seed peers remain available for retry"
        );
    }

    #[test_log::test]
    fn graceful_stop_tombstone_removes_discovered_non_seed_immediately() {
        let stopped = peer_addr("2@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::default();

        discovery.apply_sync(100, PeerDiscoverySync(vec![(stopped.peer_id(), 100, stopped.network_address().clone())]));

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![stopped.clone()]);

        discovery.remove_remote(110, &stopped.peer_id());

        assert_eq!(
            discovery.remotes().next(),
            None,
            "a graceful stop notification should evict a previously discovered non-seed peer without waiting for timeout"
        );
    }

    #[test_log::test]
    fn graceful_stop_tombstone_keeps_seed_retry_address() {
        let seed = peer_addr("1@127.0.0.1:9000");
        let mut discovery = PeerDiscovery::new(vec![seed.clone()]);

        discovery.remove_remote(110, &seed.peer_id());

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![seed], "seed peers should remain retryable even after a stop signal");
    }

    #[test_log::test]
    fn graceful_stop_tombstones_must_be_bounded_for_unknown_peers() {
        const MAX_STOPPED_TOMBSTONES: usize = 1024;
        let mut discovery = PeerDiscovery::default();

        for peer in 0..=MAX_STOPPED_TOMBSTONES {
            discovery.remove_remote(100, &PeerId(peer as u64 + 10));
        }

        assert!(
            discovery.stopped.len() <= MAX_STOPPED_TOMBSTONES,
            "stopped-peer tombstones must be bounded even for unknown non-seed peers, got {}",
            discovery.stopped.len()
        );
    }

    #[test_log::test]
    fn apply_sync_must_not_duplicate_or_override_configured_seed() {
        let seed = peer_addr("1@127.0.0.1:9000");
        let advertised_seed = peer_addr("1@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::new(vec![seed.clone()]);

        discovery.apply_sync(100, PeerDiscoverySync(vec![(advertised_seed.peer_id(), 100, advertised_seed.network_address().clone())]));

        assert_eq!(
            discovery.remotes().collect::<Vec<_>>(),
            vec![seed],
            "remote discovery advertisements must not duplicate or override configured seed addresses"
        );
    }

    #[test_log::test]
    fn graceful_stop_tombstone_ignores_stale_non_seed_advertise() {
        let stopped = peer_addr("2@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::default();

        discovery.apply_sync(100, PeerDiscoverySync(vec![(stopped.peer_id(), 100, stopped.network_address().clone())]));
        discovery.remove_remote(110, &stopped.peer_id());
        discovery.apply_sync(120, PeerDiscoverySync(vec![(stopped.peer_id(), 100, stopped.network_address().clone())]));

        assert_eq!(
            discovery.remotes().next(),
            None,
            "stale advertisements should not immediately re-add a gracefully stopped non-seed peer"
        );
    }

    #[test_log::test]
    fn graceful_stop_tombstone_must_allow_fresh_restart_advertise() {
        let stopped = peer_addr("2@127.0.0.1:9001");
        let restarted = peer_addr("2@127.0.0.1:9002");
        let mut discovery = PeerDiscovery::default();

        discovery.apply_sync(100, PeerDiscoverySync(vec![(stopped.peer_id(), 100, stopped.network_address().clone())]));
        discovery.remove_remote(110, &stopped.peer_id());
        discovery.apply_sync(120, PeerDiscoverySync(vec![(restarted.peer_id(), 120, restarted.network_address().clone())]));

        assert_eq!(
            discovery.remotes().collect::<Vec<_>>(),
            vec![restarted],
            "fresh restart advertisements newer than the stop event must not be suppressed by the stale-stop tombstone"
        );
    }

    #[test_log::test]
    fn apply_sync_rejects_local_peer_advertisement() {
        let local = PeerId(1);
        let local_addr = peer_addr("1@127.0.0.1:9000");
        let attacker_local_addr = peer_addr("1@127.0.0.1:9999");
        let mut discovery = PeerDiscovery::default();
        discovery.enable_local(local, local_addr.network_address().clone());

        discovery.apply_sync(100, PeerDiscoverySync(vec![(local, 100, attacker_local_addr.network_address().clone())]));

        assert!(discovery.remotes().all(|addr| addr.peer_id() != local), "sync must not create a remote candidate for the local peer id");
    }

    #[test_log::test]
    fn configured_seed_with_local_peer_id_must_not_be_dial_candidate() {
        let local = peer_addr("1@127.0.0.1:9000");
        let mut discovery = PeerDiscovery::new(vec![local.clone()]);
        discovery.enable_local(local.peer_id(), local.network_address().clone());

        assert!(
            discovery.remotes().all(|addr| addr.peer_id() != local.peer_id()),
            "configured seeds with the local peer id must not be returned as remote dial candidates"
        );
    }

    #[test_log::test]
    fn apply_sync_rejects_overflowing_future_timestamp() {
        let peer = peer_addr("2@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::default();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            discovery.apply_sync(100, PeerDiscoverySync(vec![(peer.peer_id(), u64::MAX, peer.network_address().clone())]));
        }));

        assert!(result.is_ok(), "untrusted discovery timestamps must not panic on overflow");
        assert_eq!(discovery.remotes().next(), None, "untrusted future/overflow timestamp should not create an immortal peer");
    }
}
