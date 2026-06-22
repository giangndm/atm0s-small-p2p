use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{select, task::JoinHandle, time::Interval};

use crate::{peer::PeerConnectionMetric, ConnectionId, P2pServiceEvent, PeerId};

use super::P2pService;

#[derive(Debug, PartialEq, Eq)]
pub enum MetricsServiceEvent {
    OnPeerConnectionMetric(PeerId, Vec<(ConnectionId, PeerId, PeerConnectionMetric)>),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable};
    use futures::FutureExt;

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
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = MetricsService::new(Some(Duration::from_secs(3600)), base_service, true);
        let peer = PeerId::from(2);
        let _ = service.recv().await.expect("collector should emit initial local metrics");
        service.pending_info_responders.insert(peer);
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
            .send(P2pServiceEvent::Unicast(peer, encode_info_for_test(metrics)))
            .await
            .expect("metrics service channel should accept test message");

        let delivered = tokio::time::timeout(Duration::from_millis(200), service.recv()).await;

        assert!(
            !matches!(delivered, Ok(Ok(MetricsServiceEvent::OnPeerConnectionMetric(delivered_peer, _))) if delivered_peer == peer),
            "oversized correlated metrics Info batches must be rejected"
        );
    }

    #[tokio::test]
    async fn metrics_info_batch_at_cap_must_be_accepted() {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (base_service, service_tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = MetricsService::new(Some(Duration::from_secs(3600)), base_service, true);
        let peer = PeerId::from(2);
        let _ = service.recv().await.expect("collector should emit initial local metrics");
        service.pending_info_responders.insert(peer);
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
        let metrics = (0..MAX_METRICS_PER_INFO)
            .map(|idx| (ConnectionId::from(idx as u64 + 10), PeerId::from(idx as u64 + 100), metric.clone()))
            .collect::<Vec<_>>();

        service_tx
            .send(P2pServiceEvent::Unicast(peer, encode_info_for_test(metrics)))
            .await
            .expect("metrics service channel should accept test message");

        let MetricsServiceEvent::OnPeerConnectionMetric(delivered_peer, delivered) = service.recv().await.expect("metrics event should be emitted");
        assert_eq!(delivered_peer, peer);
        assert_eq!(delivered.len(), MAX_METRICS_PER_INFO);
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
const SCAN_RESPONSE_SEND_TIMEOUT: Duration = Duration::from_secs(1);
const SCAN_RESPONSE_RETRY_DELAY: Duration = Duration::from_millis(5);
const SCAN_BROADCAST_SEND_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_METRICS_PER_INFO: usize = 1024;

pub struct MetricsService {
    is_collector: bool,
    service: P2pService,
    ticker: Interval,
    outs: VecDeque<MetricsServiceEvent>,
    trusted_scan_collectors: HashSet<PeerId>,
    pending_scan_responses: HashSet<PeerId>,
    pending_info_responders: HashSet<PeerId>,
    scan_response_tasks: FuturesUnordered<JoinHandle<PeerId>>,
    pending_scan_broadcast: Option<JoinHandle<()>>,
}

impl MetricsService {
    pub fn new(collect_interval: Option<Duration>, service: P2pService, is_collector: bool) -> Self {
        let collect_interval = match collect_interval {
            Some(interval) if interval.is_zero() => Duration::from_secs(DEFAULT_COLLECTOR_INTERVAL),
            Some(interval) => interval,
            None => Duration::from_secs(DEFAULT_COLLECTOR_INTERVAL),
        };
        let ticker = tokio::time::interval(collect_interval);

        Self {
            is_collector,
            ticker,
            service,
            outs: VecDeque::new(),
            trusted_scan_collectors: HashSet::new(),
            pending_scan_responses: HashSet::new(),
            pending_info_responders: HashSet::new(),
            scan_response_tasks: FuturesUnordered::new(),
            pending_scan_broadcast: None,
        }
    }

    pub fn with_trusted_scan_collectors<I>(mut self, collectors: I) -> Self
    where
        I: IntoIterator<Item = PeerId>,
    {
        self.trusted_scan_collectors = collectors.into_iter().collect();
        self
    }

    fn on_scan(&mut self, from: PeerId) {
        if self.is_collector || !self.trusted_scan_collectors.contains(&from) || !self.pending_scan_responses.insert(from) {
            return;
        }

        let metrics = self.service.ctx.metrics();
        let requester = self.service.requester();
        let data = bincode::serialize(&Message::Info(metrics)).expect("should convert to buf");
        self.scan_response_tasks.push(tokio::spawn(async move {
            let deadline = tokio::time::Instant::now() + SCAN_RESPONSE_SEND_TIMEOUT;
            loop {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    log::warn!("send metrics info to collector timed out");
                    break;
                }

                let remaining = deadline.saturating_duration_since(now);
                match tokio::time::timeout(remaining, requester.send_unicast_unacked(from, data.clone())).await {
                    Ok(Ok(())) => break,
                    Ok(Err(err)) if tokio::time::Instant::now() >= deadline => {
                        log::warn!("send metrics info to collector timed out: {err}");
                        break;
                    }
                    Ok(Err(_)) => tokio::time::sleep(SCAN_RESPONSE_RETRY_DELAY).await,
                    Err(_) => {
                        log::warn!("send metrics info to collector timed out");
                        break;
                    }
                }
            }
            from
        }));
    }

    pub async fn recv(&mut self) -> anyhow::Result<MetricsServiceEvent> {
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
                        log::warn!("metrics scan response task failed: {err}");
                    }
                }
            }
            if self.pending_scan_broadcast.as_ref().is_some_and(|task| task.is_finished()) {
                if let Some(task) = self.pending_scan_broadcast.take() {
                    if let Err(err) = task.await {
                        log::warn!("metrics scan broadcast task failed: {err}");
                    }
                }
            }

            select! {
                _ = self.ticker.tick() => {
                    if self.is_collector {
                        let metrics = self.service.ctx.metrics();
                        self.pending_info_responders = self.service.ctx.conns().into_iter().map(|conn| conn.to_id()).collect();
                        self.outs.push_back(MetricsServiceEvent::OnPeerConnectionMetric(self.service.router().local_id(), metrics));

                        if self.pending_scan_broadcast.is_none() {
                            let requester = self.service.requester();
                            let data = bincode::serialize(&Message::Scan).expect("should convert to buf");
                            self.pending_scan_broadcast = Some(tokio::spawn(async move {
                                match tokio::time::timeout(SCAN_BROADCAST_SEND_TIMEOUT, requester.send_broadcast(data)).await {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(err)) => log::warn!("metrics scan broadcast failed: {err}"),
                                    Err(_) => log::warn!("metrics scan broadcast timed out"),
                                }
                            }));
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
                                Message::Info(peer_metrics) => {
                                    if self.is_collector && self.pending_info_responders.remove(&from) {
                                        if peer_metrics.len() > MAX_METRICS_PER_INFO {
                                            log::warn!("metrics info from {from} has {} rows, dropping oversized batch", peer_metrics.len());
                                        } else {
                                            self.outs.push_back(MetricsServiceEvent::OnPeerConnectionMetric(from, peer_metrics));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(P2pServiceEvent::Stream(..) | P2pServiceEvent::PeerDisconnected(..)) => {}
                    None => anyhow::bail!("metrics base service channel closed"),
                }
            }
        }
    }
}
