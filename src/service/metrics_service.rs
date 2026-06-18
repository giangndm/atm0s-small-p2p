use std::{collections::VecDeque, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::{select, time::Interval};

use crate::{peer::PeerConnectionMetric, ConnectionId, ErrorExt, P2pServiceEvent, PeerId};

use super::P2pService;

#[derive(Debug, PartialEq, Eq)]
pub enum MetricsServiceEvent {
    OnPeerConnectionMetric(PeerId, Vec<(ConnectionId, PeerId, PeerConnectionMetric)>),
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::FutureExt;
    use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable};

    #[tokio::test]
    async fn metrics_recv_after_base_service_close_must_not_panic() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = MetricsService::new(None, base_service, false);
        drop(service_tx);

        let result = std::panic::AssertUnwindSafe(service.recv()).catch_unwind().await;

        assert!(
            matches!(result, Ok(Err(_))),
            "metrics recv must return an error when the base service channel closes instead of panicking"
        );
    }

    #[tokio::test]
    async fn metrics_info_batches_must_be_bounded() {
        const MAX_METRICS_PER_INFO: usize = 1024;
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = MetricsService::new(None, base_service, false);
        let metric = PeerConnectionMetric {
            uptime: 1,
            rtt: 2,
            sent_pkt: 3,
            lost_pkt: 4,
            lost_bytes: 5,
            send_bytes: 6,
            recv_bytes: 7,
            current_mtu: 1200,
        };
        let metrics = (0..=MAX_METRICS_PER_INFO)
            .map(|idx| (ConnectionId::from(idx as u64 + 10), PeerId::from(idx as u64 + 100), metric.clone()))
            .collect::<Vec<_>>();

        service_tx
            .send(P2pServiceEvent::Unicast(PeerId::from(2), encode_info_for_test(metrics)))
            .await
            .expect("metrics service channel should accept test message");

        let MetricsServiceEvent::OnPeerConnectionMetric(_, delivered) = service.recv().await.expect("metrics event should be emitted");

        assert!(delivered.len() <= MAX_METRICS_PER_INFO, "metrics Info batches must be bounded, got {} rows", delivered.len());
    }
}

#[derive(Deserialize, Serialize)]
enum Message {
    Scan,
    Info(Vec<(ConnectionId, PeerId, PeerConnectionMetric)>),
}

#[cfg(test)]
pub(crate) fn encode_info_for_test(metrics: Vec<(ConnectionId, PeerId, PeerConnectionMetric)>) -> Vec<u8> {
    bincode::serialize(&Message::Info(metrics)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_scan_for_test() -> Vec<u8> {
    bincode::serialize(&Message::Scan).expect("test message should serialize")
}

const DEFAULT_COLLECTOR_INTERVAL: u64 = 1;

pub struct MetricsService {
    is_collector: bool,
    service: P2pService,
    ticker: Interval,
    outs: VecDeque<MetricsServiceEvent>,
}

impl MetricsService {
    pub fn new(collect_interval: Option<Duration>, service: P2pService, is_collector: bool) -> Self {
        let ticker = tokio::time::interval(collect_interval.unwrap_or(Duration::from_secs(DEFAULT_COLLECTOR_INTERVAL)));

        Self {
            is_collector,
            ticker,
            service,
            outs: VecDeque::new(),
        }
    }

    pub async fn recv(&mut self) -> anyhow::Result<MetricsServiceEvent> {
        loop {
            if let Some(out) = self.outs.pop_front() {
                return Ok(out);
            }

            select! {
                _ = self.ticker.tick() => {
                    if self.is_collector {
                        let metrics = self.service.ctx.metrics();
                        self.outs.push_back(MetricsServiceEvent::OnPeerConnectionMetric(self.service.router().local_id(), metrics));

                        let requester = self.service.requester();
                        tokio::spawn(async move {
                            requester.send_broadcast(bincode::serialize(&Message::Scan).expect("should convert to buf")).await;
                        });
                    }
                }
                event = self.service.recv() => match event.expect("should work") {
                    P2pServiceEvent::Unicast(from, data) | P2pServiceEvent::Broadcast(from, data) => {
                        if let Ok(msg) = bincode::deserialize::<Message>(&data) {
                            match msg {
                                Message::Scan => {
                                    let metrics = self.service.ctx.metrics();
                                    let requester = self.service.requester();
                                    tokio::spawn(async move {
                                        requester.try_send_unicast(from, bincode::serialize(&Message::Info(metrics)).expect("should convert to buf"))
                                            .await
                                            .print_on_err("send metrics info to collector error");
                                    });
                                }
                                Message::Info(peer_metrics) => {
                                    self.outs.push_back(MetricsServiceEvent::OnPeerConnectionMetric(from, peer_metrics));
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
