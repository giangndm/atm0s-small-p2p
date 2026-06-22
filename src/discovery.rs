use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{NetworkAddress, PeerAddress};

use super::PeerId;

const TIMEOUT_AFTER: u64 = 30_000;
const MAX_SYNC_ENTRIES: usize = 1024;
const MAX_STOPPED_TOMBSTONES: usize = 1024;

fn timestamp_is_live(timestamp: u64, now_ms: u64) -> bool {
    timestamp <= now_ms && timestamp.checked_add(TIMEOUT_AFTER).is_some_and(|expires_at| expires_at > now_ms)
}

fn is_dialable_advertise_address(address: &NetworkAddress) -> bool {
    !address.ip().is_unspecified() && address.port() != 0
}

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
        if !is_dialable_advertise_address(&address) {
            log::warn!("[PeerDiscovery] ignore non-dialable local advertise address {address}");
            self.local = None;
            return;
        }

        log::info!("[PeerDiscovery] enable local as {address}");
        self.local = Some((peer_id, address));
    }

    pub fn clear_timeout(&mut self, now_ms: u64) -> Vec<PeerId> {
        let mut expired = Vec::new();
        self.remotes.retain(|peer, (last_updated, _addr)| {
            if timestamp_is_live(*last_updated, now_ms) {
                true
            } else {
                log::info!("[PeerDiscovery] remove timeout {peer}");
                expired.push(*peer);
                false
            }
        });
        self.stopped.retain(|peer, stopped_at| {
            if timestamp_is_live(*stopped_at, now_ms) {
                true
            } else {
                log::info!("[PeerDiscovery] clear stopped tombstone {peer}");
                false
            }
        });
        expired
    }

    pub fn remove_remote(&mut self, now_ms: u64, peer: &PeerId) {
        if self.seeds.iter().any(|seed| seed.peer_id().eq(peer)) {
            return;
        }
        self.stopped.insert(*peer, now_ms);
        if self.stopped.len() > MAX_STOPPED_TOMBSTONES {
            if let Some(evicted) = self.stopped.iter().map(|(peer, stopped_at)| (*stopped_at, *peer)).min().map(|(_, peer)| peer) {
                self.stopped.remove(&evicted);
                log::debug!("[PeerDiscovery] evict stopped tombstone {evicted}");
            }
        }
        if self.remotes.remove(peer).is_some() {
            log::info!("[PeerDiscovery] remove stopped peer {peer}");
        }
    }

    pub fn is_stopped(&self, now_ms: u64, peer: &PeerId) -> bool {
        self.stopped.get(peer).copied().is_some_and(|stopped_at| timestamp_is_live(stopped_at, now_ms))
    }

    pub fn create_sync_for(&self, now_ms: u64, dest: &PeerId) -> PeerDiscoverySync {
        let iter = self.local.iter().map(|(p, addr)| (*p, now_ms, addr.clone()));
        PeerDiscoverySync(
            self.remotes
                .iter()
                .filter(|(k, _)| !dest.eq(k))
                .map(|(k, (v1, v2))| (*k, *v1, v2.clone()))
                .chain(iter)
                .take(MAX_SYNC_ENTRIES)
                .collect::<Vec<_>>(),
        )
    }

    pub fn apply_sync(&mut self, now_ms: u64, sync: PeerDiscoverySync) {
        log::debug!("[PeerDiscovery] apply sync with {} addrs", sync.0.len());
        if sync.0.len() > MAX_SYNC_ENTRIES {
            log::debug!("[PeerDiscovery] ignore oversized sync: {} addrs exceeds cap {MAX_SYNC_ENTRIES}", sync.0.len());
            return;
        }

        for (peer, last_updated, address) in sync.0.into_iter() {
            if !timestamp_is_live(last_updated, now_ms) {
                continue;
            }

            if self.local.as_ref().is_some_and(|(local_peer, _)| *local_peer == peer) {
                continue;
            }

            if self.seeds.iter().any(|seed| seed.peer_id() == peer) {
                continue;
            }

            if !is_dialable_advertise_address(&address) {
                log::warn!("[PeerDiscovery] ignore non-dialable remote advertise address {address}");
                continue;
            }

            if let Some(stopped_at) = self.stopped.get(&peer).copied() {
                if timestamp_is_live(stopped_at, now_ms) && last_updated <= stopped_at {
                    continue;
                }
                self.stopped.remove(&peer);
            }

            match self.remotes.get(&peer) {
                Some((existing_updated, _)) if *existing_updated >= last_updated => {}
                _ => {
                    if self.remotes.insert(peer, (last_updated, address)).is_none() {
                        log::info!("[PeerDiscovery] added new peer {peer}");
                    }
                }
            }
        }
    }
    pub fn remotes(&self) -> impl Iterator<Item = PeerAddress> + '_ {
        let local_peer = self.local.as_ref().map(|(peer, _)| *peer);
        self.remotes
            .iter()
            .map(|(p, (_, a))| PeerAddress(*p, a.clone()))
            .chain(self.seeds.iter().filter(move |seed| Some(seed.peer_id()) != local_peer).cloned())
    }
}

#[cfg(test)]
mod test {
    use crate::{discovery::PeerDiscoverySync, PeerAddress, PeerId};

    use super::{is_dialable_advertise_address, PeerDiscovery, MAX_STOPPED_TOMBSTONES, MAX_SYNC_ENTRIES, TIMEOUT_AFTER};

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
    fn local_sync_must_not_advertise_unroutable_wildcard_address() {
        let mut discovery = PeerDiscovery::default();
        let local = PeerId(1);
        let remote = PeerId(2);
        let wildcard: PeerAddress = "1@0.0.0.0:0".parse().expect("wildcard address should parse");

        discovery.enable_local(local, wildcard.network_address().clone());

        assert_eq!(
            discovery.create_sync_for(100, &remote),
            PeerDiscoverySync(vec![]),
            "unroutable wildcard or port-zero local advertise addresses must not be gossiped as dial candidates"
        );
    }

    #[test_log::test]
    fn local_sync_must_not_advertise_unspecified_ip_address() {
        let mut discovery = PeerDiscovery::default();
        let local = PeerId(1);
        let remote = PeerId(2);
        let unspecified_ip = peer_addr("1@0.0.0.0:9000");

        discovery.enable_local(local, unspecified_ip.network_address().clone());

        assert_eq!(
            discovery.create_sync_for(100, &remote),
            PeerDiscoverySync(vec![]),
            "unspecified local advertise IPs must not be gossiped as dial candidates"
        );
    }

    #[test_log::test]
    fn local_sync_must_not_advertise_port_zero_address() {
        let mut discovery = PeerDiscovery::default();
        let local = PeerId(1);
        let remote = PeerId(2);
        let port_zero = peer_addr("1@127.0.0.1:0");

        discovery.enable_local(local, port_zero.network_address().clone());

        assert_eq!(
            discovery.create_sync_for(100, &remote),
            PeerDiscoverySync(vec![]),
            "port-zero local advertise addresses must not be gossiped as dial candidates"
        );
    }

    #[test_log::test]
    fn apply_sync_must_reject_non_dialable_remote_addresses() {
        let wildcard = peer_addr("2@0.0.0.0:0");
        let port_zero = peer_addr("3@127.0.0.1:0");
        let mut discovery = PeerDiscovery::default();

        discovery.apply_sync(
            100,
            PeerDiscoverySync(vec![
                (wildcard.peer_id(), 100, wildcard.network_address().clone()),
                (port_zero.peer_id(), 100, port_zero.network_address().clone()),
            ]),
        );

        assert_eq!(
            discovery.remotes().next(),
            None,
            "remote discovery syncs must reject non-dialable wildcard or port-zero addresses before they become dial candidates"
        );
    }

    #[test_log::test]
    fn invalid_local_advertise_clears_previous_valid_address() {
        let mut discovery = PeerDiscovery::default();
        let local = PeerId(1);
        let remote = PeerId(2);
        let valid = peer_addr("1@127.0.0.1:9000");
        let invalid = peer_addr("1@127.0.0.1:0");

        discovery.enable_local(local, valid.network_address().clone());
        discovery.enable_local(local, invalid.network_address().clone());

        assert_eq!(
            discovery.create_sync_for(100, &remote),
            PeerDiscoverySync(vec![]),
            "invalid local advertise reconfiguration must clear a previously gossiped valid address"
        );
    }

    #[test]
    fn dialable_advertise_address_validation_accepts_valid_addresses() {
        let valid = peer_addr("1@127.0.0.1:9000");

        assert!(is_dialable_advertise_address(valid.network_address()));
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
    fn discovery_sync_must_reject_duplicate_peer_entries() {
        let mut discovery = PeerDiscovery::default();
        let peer = PeerId(7);
        let fresh_addr = peer_addr("7@127.0.0.1:9001");
        let stale_addr = peer_addr("7@127.0.0.1:9000");

        discovery.apply_sync(
            210,
            PeerDiscoverySync(vec![(peer, 200, fresh_addr.network_address().clone()), (peer, 100, stale_addr.network_address().clone())]),
        );

        assert_eq!(
            discovery.remotes().collect::<Vec<_>>(),
            vec![fresh_addr],
            "duplicate discovery rows for one peer must be rejected or resolved by newest timestamp, not by attacker-controlled row order"
        );
    }

    #[test_log::test]
    fn discovery_sync_must_reject_excessive_entries() {
        let mut discovery = PeerDiscovery::default();
        let huge_sync = (10_000..11_100)
            .map(|id| {
                let peer = peer_addr(&format!("{id}@127.0.0.1:{}", 9000 + id - 10_000));
                (peer.peer_id(), 100, peer.network_address().clone())
            })
            .collect::<Vec<_>>();

        discovery.apply_sync(100, PeerDiscoverySync(huge_sync));

        assert_eq!(discovery.remotes().next(), None, "oversized discovery syncs must be rejected without creating remote peers");
    }

    #[test_log::test]
    fn create_sync_for_must_cap_outbound_discovery_entries() {
        let mut discovery = PeerDiscovery::default();

        for id in 1..=1_100 {
            let peer = peer_addr(&format!("{id}@127.0.0.1:{}", 9000 + id));
            discovery.apply_sync(100, PeerDiscoverySync(vec![(peer.peer_id(), 100, peer.network_address().clone())]));
        }

        let sync = discovery.create_sync_for(100, &PeerId(2_000));

        assert_eq!(sync.0.len(), MAX_SYNC_ENTRIES, "outbound discovery syncs must cap the number of advertised entries");
    }

    #[test_log::test]
    fn clear_timeout() {
        let mut discovery = PeerDiscovery::default();

        let peer1 = PeerId(1);
        let peer1_addr = peer_addr("1@127.0.0.1:9000");

        discovery.apply_sync(100, PeerDiscoverySync(vec![(peer1, 90, peer1_addr.network_address().clone())]));

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![peer1_addr]);

        assert_eq!(discovery.clear_timeout(TIMEOUT_AFTER + 90), vec![peer1]);

        assert_eq!(discovery.remotes().next(), None);
    }

    #[test_log::test]
    fn non_seed_discovered_peer_ages_out_but_seed_remains_retryable() {
        let seed = peer_addr("1@127.0.0.1:9000");
        let discovered = peer_addr("2@127.0.0.1:9001");
        let mut discovery = PeerDiscovery::new(vec![seed.clone()]);

        discovery.apply_sync(100, PeerDiscoverySync(vec![(discovered.peer_id(), 100, discovered.network_address().clone())]));

        assert_eq!(discovery.remotes().collect::<Vec<_>>(), vec![discovered.clone(), seed.clone()]);

        assert_eq!(discovery.clear_timeout(100 + TIMEOUT_AFTER), vec![discovered.peer_id()]);

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
    fn graceful_stop_tombstones_evict_oldest_deterministically() {
        let mut discovery = PeerDiscovery::default();

        for peer in 0..MAX_STOPPED_TOMBSTONES {
            discovery.remove_remote(200, &PeerId(peer as u64 + 10));
        }

        let tied_oldest = PeerId(9);
        discovery.remove_remote(200, &tied_oldest);

        assert_eq!(discovery.stopped.len(), MAX_STOPPED_TOMBSTONES);
        assert!(
            !discovery.stopped.contains_key(&tied_oldest),
            "when tombstones tie on timestamp, the lowest peer id should be evicted deterministically"
        );

        let oldest = PeerId(2_000);
        discovery.remove_remote(100, &oldest);

        assert_eq!(discovery.stopped.len(), MAX_STOPPED_TOMBSTONES);
        assert!(!discovery.stopped.contains_key(&oldest), "the oldest stopped tombstone should be evicted after cap overflow");
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
