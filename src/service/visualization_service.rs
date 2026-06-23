use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio::{select, time::Interval};

use crate::{now_ms, ConnectionId, PeerId};

use super::{P2pService, P2pServiceEvent};

#[derive(Debug, PartialEq, Eq)]
pub enum VisualizationServiceEvent {
    PeerJoined(PeerId, Vec<(ConnectionId, PeerId, u16)>),
    PeerUpdated(PeerId, Vec<(ConnectionId, PeerId, u16)>),
    PeerLeaved(PeerId),
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    Scan,
    Info(Vec<(ConnectionId, PeerId, u16)>),
}

#[cfg(test)]
pub(crate) fn encode_info_for_test(neighbours: Vec<(ConnectionId, PeerId, u16)>) -> Vec<u8> {
    bincode::serialize(&Message::Info(neighbours)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_scan_for_test() -> Vec<u8> {
    bincode::serialize(&Message::Scan).expect("test message should serialize")
}

const SCAN_RESPONSE_SEND_TIMEOUT: Duration = Duration::from_secs(1);
const SCAN_RESPONSE_RETRY_DELAY: Duration = Duration::from_millis(5);
const SCAN_BROADCAST_SEND_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_COLLECT_INTERVAL: Duration = Duration::from_secs(100);
const MAX_VISUALIZATION_REMOTE_PEERS: usize = 1024;
const MAX_TOPOLOGY_ROWS_PER_INFO: usize = 1024;

pub struct VisualizationService {
    service: P2pService,
    neighbours: HashMap<PeerId, u64>,
    trusted_scan_collectors: HashSet<PeerId>,
    pending_scan_responses: HashSet<PeerId>,
    pending_info_responders: HashSet<PeerId>,
    scan_response_tasks: FuturesUnordered<JoinHandle<PeerId>>,
    pending_scan_broadcast: Option<JoinHandle<()>>,
    ticker: Interval,
    collect_interval: Option<Duration>,
    collect_me: bool,
    outs: VecDeque<VisualizationServiceEvent>,
}

fn is_peer_timed_out(now: u64, last_updated: u64, interval: Duration) -> bool {
    let Ok(timeout_ms) = u64::try_from(interval.as_millis().saturating_mul(2)) else {
        return false;
    };
    let Some(deadline) = last_updated.checked_add(timeout_ms) else {
        return false;
    };
    now >= deadline
}

impl VisualizationService {
    pub fn new(collect_interval: Option<Duration>, collect_me: bool, service: P2pService) -> Self {
        let collect_interval = collect_interval.map(|interval| {
            if interval.is_zero() {
                DEFAULT_COLLECT_INTERVAL
            } else {
                interval
            }
        });
        let ticker = tokio::time::interval(collect_interval.unwrap_or(DEFAULT_COLLECT_INTERVAL));

        Self {
            ticker,
            collect_interval,
            collect_me,
            neighbours: HashMap::new(),
            trusted_scan_collectors: HashSet::new(),
            pending_scan_responses: HashSet::new(),
            pending_info_responders: HashSet::new(),
            scan_response_tasks: FuturesUnordered::new(),
            pending_scan_broadcast: None,
            outs: if collect_me {
                VecDeque::from([VisualizationServiceEvent::PeerJoined(service.router().local_id(), vec![])])
            } else {
                VecDeque::new()
            },
            service,
        }
    }

    pub fn with_trusted_scan_collectors<I>(mut self, collectors: I) -> Self
    where
        I: IntoIterator<Item = PeerId>,
    {
        self.trusted_scan_collectors = collectors.into_iter().collect();
        self
    }

    pub async fn recv(&mut self) -> anyhow::Result<VisualizationServiceEvent> {
        loop {
            if let Some(out) = self.outs.pop_front() {
                return Ok(out);
            }
            while let Some(task) = self.scan_response_tasks.next().now_or_never().flatten() {
                match task {
                    Ok(peer) => {
                        self.pending_scan_responses.remove(&peer);
                    }
                    Err(err) => {
                        log::warn!("visualization scan response task failed: {err}");
                    }
                }
            }
            if self.pending_scan_broadcast.as_ref().is_some_and(|task| task.is_finished()) {
                if let Some(task) = self.pending_scan_broadcast.take() {
                    if let Err(err) = task.await {
                        log::warn!("visualization scan broadcast task failed: {err}");
                    }
                }
            }

            select! {
                Some(task) = self.scan_response_tasks.next(), if !self.scan_response_tasks.is_empty() => {
                    match task {
                        Ok(peer) => {
                            self.pending_scan_responses.remove(&peer);
                        }
                        Err(err) => {
                            log::warn!("visualization scan response task failed: {err}");
                        }
                    }
                }
                _ = self.ticker.tick() => {
                    if let Some(interval) = self.collect_interval {
                        if self.collect_me {
                            // for update local node
                            self.outs.push_back(VisualizationServiceEvent::PeerUpdated(self.service.router().local_id(), self.service.router().neighbours()));
                        }

                        self.pending_info_responders = self.service.ctx.conns().into_iter().map(|conn| conn.to_id()).collect();

                        if self.pending_scan_broadcast.is_none() {
                            let requester = self.service.requester();
                            let data = bincode::serialize(&Message::Scan).expect("should convert to buf");
                            self.pending_scan_broadcast = Some(tokio::spawn(async move {
                                match tokio::time::timeout(SCAN_BROADCAST_SEND_TIMEOUT, requester.send_broadcast(data)).await {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(err)) => log::warn!("visualization scan broadcast failed: {err}"),
                                    Err(_) => log::warn!("visualization scan broadcast timed out"),
                                }
                            }));
                        }

                        let now = now_ms();
                        let mut timeout_peers = vec![];
                        for (peer, last_updated) in self.neighbours.iter() {
                            if is_peer_timed_out(now, *last_updated, interval) {
                                timeout_peers.push(*peer);
                                self.outs.push_back(VisualizationServiceEvent::PeerLeaved(*peer));
                            }
                        }

                        for peer in timeout_peers {
                            self.neighbours.remove(&peer);
                        }
                    }
                }
                event = self.service.recv() => match event {
                    Some(P2pServiceEvent::Broadcast(from, data)) => {
                        if let Ok(msg) = bincode::deserialize::<Message>(&data) {
                            match msg {
                                Message::Scan => {
                                    self.on_scan(from);
                                }
                                Message::Info(_) => {}
                            }
                        }
                    }
                    Some(P2pServiceEvent::Unicast(from, data)) => {
                        if let Ok(msg) = bincode::deserialize::<Message>(&data) {
                            match msg {
                                Message::Scan => {}
                                Message::Info(neighbours) => {
                                    if self.pending_info_responders.remove(&from) {
                                        self.on_info(from, neighbours);
                                    }
                                }
                            }
                        }
                    }
                    Some(P2pServiceEvent::Stream(..)) => {}
                    Some(P2pServiceEvent::PeerDisconnected(peer)) => {
                        self.pending_info_responders.remove(&peer);
                        self.pending_scan_responses.remove(&peer);
                        if self.neighbours.remove(&peer).is_some() {
                            self.outs.push_back(VisualizationServiceEvent::PeerLeaved(peer));
                        }
                    }
                    None => anyhow::bail!("visualization base service channel closed"),
                }
            }
        }
    }

    fn on_info(&mut self, from: PeerId, neighbours: Vec<(ConnectionId, PeerId, u16)>) {
        if neighbours.len() > MAX_TOPOLOGY_ROWS_PER_INFO {
            log::warn!("visualization info from {from} has {} rows, dropping oversized batch", neighbours.len());
            return;
        }

        let now = now_ms();
        if let std::collections::hash_map::Entry::Occupied(mut entry) = self.neighbours.entry(from) {
            entry.insert(now);
            self.outs.push_back(VisualizationServiceEvent::PeerUpdated(from, neighbours));
            return;
        }
        if self.neighbours.len() >= MAX_VISUALIZATION_REMOTE_PEERS {
            return;
        }
        self.neighbours.insert(from, now);
        self.outs.push_back(VisualizationServiceEvent::PeerJoined(from, neighbours));
    }

    fn on_scan(&mut self, from: PeerId) {
        if self.trusted_scan_collectors.contains(&from) && self.pending_scan_responses.insert(from) {
            let requester = self.service.requester();
            let neighbours = requester.router().neighbours();
            let data = bincode::serialize(&Message::Info(neighbours)).expect("should convert to buf");
            self.scan_response_tasks.push(tokio::spawn(async move {
                let deadline = tokio::time::Instant::now() + SCAN_RESPONSE_SEND_TIMEOUT;
                loop {
                    let now = tokio::time::Instant::now();
                    if now >= deadline {
                        log::warn!("send neighbour info to visualization collector timed out");
                        break;
                    }

                    let remaining = deadline.saturating_duration_since(now);
                    match tokio::time::timeout(remaining, requester.send_unicast_unacked(from, data.clone())).await {
                        Ok(Ok(())) => break,
                        Ok(Err(err)) if tokio::time::Instant::now() >= deadline => {
                            log::warn!("send neighbour info to visualization collector timed out: {err}");
                            break;
                        }
                        Ok(Err(_)) => tokio::time::sleep(SCAN_RESPONSE_RETRY_DELAY).await,
                        Err(_) => {
                            log::warn!("send neighbour info to visualization collector timed out");
                            break;
                        }
                    }
                }
                from
            }));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable};
    use futures::FutureExt;

    #[tokio::test]
    async fn visualization_recv_after_base_service_close_must_not_panic() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        drop(service_tx);

        let result = std::panic::AssertUnwindSafe(service.recv()).catch_unwind().await;

        assert!(
            matches!(result, Ok(Err(_))),
            "visualization recv must return an error when the base service channel closes instead of panicking"
        );
    }

    #[test]
    fn visualization_peer_timeout_deadline_must_not_overflow() {
        let last_updated = u64::MAX - 10;
        let interval = Duration::from_millis(6);
        let now = u64::MAX;

        let result = std::panic::catch_unwind(|| is_peer_timed_out(now, last_updated, interval));

        assert!(result.is_ok(), "visualization timeout arithmetic must not panic near u64::MAX");
        assert!(
            !result.expect("timeout calculation should not panic"),
            "visualization timeout deadline must not wrap and expire a peer before the mathematical deadline"
        );
    }

    #[tokio::test]
    async fn visualization_remote_peers_must_be_bounded() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let info = encode_info_for_test(vec![]);

        for peer in 0..MAX_VISUALIZATION_REMOTE_PEERS {
            let peer = PeerId::from(peer as u64 + 10);
            service.pending_info_responders.insert(peer);
            service_tx
                .send(P2pServiceEvent::Unicast(peer, info.clone()))
                .await
                .expect("visualization service channel should accept test message");
            let _ = service.recv().await.expect("visualization event should be emitted");
        }

        service.pending_info_responders.insert(PeerId::from(20_000));
        service_tx
            .send(P2pServiceEvent::Unicast(PeerId::from(20_000), info.clone()))
            .await
            .expect("visualization service channel should accept rejected test message");

        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.recv()).await.is_err(),
            "rejected new remote peers must not emit visualization events"
        );
        let remote_peers = service.neighbours.len();
        assert!(remote_peers <= MAX_VISUALIZATION_REMOTE_PEERS, "visualization remote peer state must be bounded, got {remote_peers}");
    }

    #[tokio::test]
    async fn visualization_existing_peer_update_must_work_when_remote_peer_cap_full() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let info = encode_info_for_test(vec![]);

        for peer in 0..MAX_VISUALIZATION_REMOTE_PEERS {
            let peer = PeerId::from(peer as u64 + 10);
            service.pending_info_responders.insert(peer);
            service_tx
                .send(P2pServiceEvent::Unicast(peer, info.clone()))
                .await
                .expect("visualization service channel should accept test message");
            let _ = service.recv().await.expect("visualization event should be emitted");
        }

        let existing = PeerId::from(10);
        service.pending_info_responders.insert(existing);
        service_tx
            .send(P2pServiceEvent::Unicast(existing, encode_info_for_test(vec![(ConnectionId::from(7), PeerId::from(8), 9)])))
            .await
            .expect("visualization service channel should accept update message");

        let event = service.recv().await.expect("existing remote update should be emitted");
        assert_eq!(event, VisualizationServiceEvent::PeerUpdated(existing, vec![(ConnectionId::from(7), PeerId::from(8), 9)]));
        assert_eq!(service.neighbours.len(), MAX_VISUALIZATION_REMOTE_PEERS);
    }

    #[tokio::test]
    async fn visualization_peer_disconnected_known_peer_must_emit_leave_and_clear_pending() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let peer = PeerId::from(2);

        service.neighbours.insert(peer, now_ms());
        service.pending_info_responders.insert(peer);
        service.pending_scan_responses.insert(peer);
        service_tx
            .send(P2pServiceEvent::PeerDisconnected(peer))
            .await
            .expect("visualization service channel should accept disconnect");

        assert_eq!(service.recv().await.expect("disconnect should emit leave"), VisualizationServiceEvent::PeerLeaved(peer));
        assert!(!service.neighbours.contains_key(&peer));
        assert!(!service.pending_info_responders.contains(&peer));
        assert!(!service.pending_scan_responses.contains(&peer));
    }

    #[tokio::test]
    async fn visualization_peer_disconnected_unknown_peer_must_not_emit_leave() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);

        service_tx
            .send(P2pServiceEvent::PeerDisconnected(PeerId::from(2)))
            .await
            .expect("visualization service channel should accept disconnect");

        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.recv()).await.is_err(),
            "unknown peer disconnects must not emit spurious leave events"
        );
    }

    #[tokio::test]
    async fn visualization_stale_info_after_peer_disconnected_must_be_ignored() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let peer = PeerId::from(2);

        service.neighbours.insert(peer, now_ms());
        service.pending_info_responders.insert(peer);
        service_tx
            .send(P2pServiceEvent::PeerDisconnected(peer))
            .await
            .expect("visualization service channel should accept disconnect");
        assert_eq!(service.recv().await.expect("disconnect should emit leave"), VisualizationServiceEvent::PeerLeaved(peer));

        service_tx
            .send(P2pServiceEvent::Unicast(peer, encode_info_for_test(vec![(ConnectionId::from(7), PeerId::from(8), 9)])))
            .await
            .expect("visualization service channel should accept stale info");

        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.recv()).await.is_err(),
            "stale Info after disconnect must not emit a new visualization event"
        );
        assert!(!service.neighbours.contains_key(&peer), "stale Info after disconnect must not recreate peer state");
    }

    #[tokio::test]
    async fn visualization_broadcast_remote_peers_must_be_bounded() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let info = encode_info_for_test(vec![]);

        service_tx
            .send(P2pServiceEvent::Broadcast(PeerId::from(10), info))
            .await
            .expect("visualization service channel should accept ignored test message");

        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.recv()).await.is_err(),
            "broadcast Info frames must not emit visualization events"
        );
        assert_eq!(service.neighbours.len(), 0);
    }

    #[tokio::test]
    async fn visualization_info_batches_must_be_bounded() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let neighbours = (0..=MAX_TOPOLOGY_ROWS_PER_INFO)
            .map(|idx| (ConnectionId::from(idx as u64 + 10), PeerId::from(idx as u64 + 100), idx as u16))
            .collect::<Vec<_>>();
        service.pending_info_responders.insert(PeerId::from(2));

        service_tx
            .send(P2pServiceEvent::Unicast(PeerId::from(2), encode_info_for_test(neighbours)))
            .await
            .expect("visualization service channel should accept test message");

        assert!(
            tokio::time::timeout(Duration::from_millis(20), service.recv()).await.is_err(),
            "oversized visualization Info batches must be dropped"
        );
    }

    #[tokio::test]
    async fn visualization_info_batch_at_cap_must_be_accepted() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let neighbours = (0..MAX_TOPOLOGY_ROWS_PER_INFO)
            .map(|idx| (ConnectionId::from(idx as u64 + 10), PeerId::from(idx as u64 + 100), idx as u16))
            .collect::<Vec<_>>();
        service.pending_info_responders.insert(PeerId::from(2));

        service_tx
            .send(P2pServiceEvent::Unicast(PeerId::from(2), encode_info_for_test(neighbours)))
            .await
            .expect("visualization service channel should accept test message");

        let event = service.recv().await.expect("visualization event should be emitted");
        let VisualizationServiceEvent::PeerJoined(_, delivered) = event else {
            panic!("expected PeerJoined event, got {event:?}");
        };

        assert_eq!(delivered.len(), MAX_TOPOLOGY_ROWS_PER_INFO);
    }
}
