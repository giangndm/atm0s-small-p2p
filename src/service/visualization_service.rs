use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use serde::{Deserialize, Serialize};
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

pub struct VisualizationService {
    service: P2pService,
    neighbours: HashMap<PeerId, u64>,
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
                                    let requester = self.service.requester();
                                    let neighbours = requester.router().neighbours();
                                    tokio::spawn(async move {
                                        requester
                                            .send_unicast(from, bincode::serialize(&Message::Info(neighbours)).expect("should convert to buf"))
                                            .await
                                            .print_on_err("send neighbour info to visualization collector");
                                    });
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
}
