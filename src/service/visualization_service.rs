use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio::{select, time::Interval};

use crate::{now_ms, ConnectionId, ErrorExt, PeerId};

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

pub struct VisualizationService {
    service: P2pService,
    neighbours: HashMap<PeerId, u64>,
    pending_scan_responses: HashSet<PeerId>,
    scan_response_tasks: FuturesUnordered<JoinHandle<PeerId>>,
    ticker: Interval,
    collect_interval: Option<Duration>,
    collect_me: bool,
    outs: VecDeque<VisualizationServiceEvent>,
}

fn is_peer_timed_out(now: u64, last_updated: u64, interval: Duration) -> bool {
    now >= last_updated + interval.as_millis() as u64 * 2
}

impl VisualizationService {
    pub fn new(collect_interval: Option<Duration>, collect_me: bool, service: P2pService) -> Self {
        let ticker = tokio::time::interval(collect_interval.unwrap_or(Duration::from_secs(100)));

        Self {
            ticker,
            collect_interval,
            collect_me,
            neighbours: HashMap::new(),
            pending_scan_responses: HashSet::new(),
            scan_response_tasks: FuturesUnordered::new(),
            outs: if collect_me {
                VecDeque::from([VisualizationServiceEvent::PeerJoined(service.router().local_id(), vec![])])
            } else {
                VecDeque::new()
            },
            service,
        }
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

            select! {
                _ = self.ticker.tick() => {
                    if let Some(interval) = self.collect_interval {
                        if self.collect_me {
                            // for update local node
                            self.outs.push_back(VisualizationServiceEvent::PeerUpdated(self.service.router().local_id(), self.service.router().neighbours()));
                        }

                        let requester = self.service.requester();
                        tokio::spawn(async move {
                            requester.send_broadcast(bincode::serialize(&Message::Scan).expect("should convert to buf")).await;
                        });

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
                event = self.service.recv() => match event.expect("should work") {
                    P2pServiceEvent::Unicast(from, data) | P2pServiceEvent::Broadcast(from, data) => {
                        if let Ok(msg) = bincode::deserialize::<Message>(&data) {
                            match msg {
                                Message::Scan => {
                                    if self.pending_scan_responses.insert(from) {
                                        let requester = self.service.requester();
                                        let neighbours = requester.router().neighbours();
                                        let data = bincode::serialize(&Message::Info(neighbours)).expect("should convert to buf");
                                        self.scan_response_tasks.push(tokio::spawn(async move {
                                            // Coalesce repeated scans while one response is backpressured.
                                            match tokio::time::timeout(SCAN_RESPONSE_SEND_TIMEOUT, requester.send_unicast(from, data)).await {
                                                Ok(result) => result.print_on_err("send neighbour info to visualization collector"),
                                                Err(_) => log::warn!("send neighbour info to visualization collector timed out"),
                                            }
                                            from
                                        }));
                                    }
                                }
                                Message::Info(neighbours) => {
                                    if self.neighbours.insert(from, now_ms()).is_none() {
                                        self.outs.push_back(VisualizationServiceEvent::PeerJoined(from, neighbours));
                                    } else {
                                        self.outs.push_back(VisualizationServiceEvent::PeerUpdated(from, neighbours));
                                    }
                                }
                            }
                        }
                    }
                    P2pServiceEvent::Stream(..) => {}
                }
            }
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
        const MAX_REMOTE_PEERS: usize = 1024;
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let info = encode_info_for_test(vec![]);

        for peer in 0..=MAX_REMOTE_PEERS {
            service_tx
                .send(P2pServiceEvent::Unicast(PeerId::from(peer as u64 + 10), info.clone()))
                .await
                .expect("visualization service channel should accept test message");
            let _ = service.recv().await.expect("visualization event should be emitted");
        }

        let remote_peers = service.neighbours.len();
        assert!(remote_peers <= MAX_REMOTE_PEERS, "visualization remote peer state must be bounded, got {remote_peers}");
    }

    #[tokio::test]
    async fn visualization_info_batches_must_be_bounded() {
        const MAX_TOPOLOGY_ROWS_PER_INFO: usize = 1024;
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = VisualizationService::new(None, false, base_service);
        let neighbours = (0..=MAX_TOPOLOGY_ROWS_PER_INFO)
            .map(|idx| (ConnectionId::from(idx as u64 + 10), PeerId::from(idx as u64 + 100), idx as u16))
            .collect::<Vec<_>>();

        service_tx
            .send(P2pServiceEvent::Unicast(PeerId::from(2), encode_info_for_test(neighbours)))
            .await
            .expect("visualization service channel should accept test message");

        let event = service.recv().await.expect("visualization event should be emitted");
        let VisualizationServiceEvent::PeerJoined(_, delivered) = event else {
            panic!("expected PeerJoined event, got {event:?}");
        };

        assert!(
            delivered.len() <= MAX_TOPOLOGY_ROWS_PER_INFO,
            "visualization Info batches must be bounded, got {} rows",
            delivered.len()
        );
    }
}
