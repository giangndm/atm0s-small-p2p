use std::time::Duration;

use crate::{
    metrics_service::{encode_info_for_test, encode_scan_for_test, MetricsService, MetricsServiceEvent},
    msg::PeerMessage,
    ConnectionId, P2pServiceEvent, PeerConnectionMetric, PeerId,
};
use test_log::test;

use super::create_node;

#[test(tokio::test)]
async fn metric_collect() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = MetricsService::new(None, node1.create_service(0.into()), true);
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, _) = create_node(true, 2, vec![addr1.clone()]).await;
    let mut service2 = MetricsService::new(None, node2.create_service(0.into()), false);
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { while service2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut event_from_peers = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let event = tokio::time::timeout(Duration::from_secs(1), service1.recv()).await.expect("").expect("");
        let MetricsServiceEvent::OnPeerConnectionMetric(peer, metrics) = event;
        event_from_peers.push((peer, metrics.len()));

        let local_events = event_from_peers.iter().filter(|(peer, len)| *peer == PeerId(1) && *len == 1).count();
        let remote_events = event_from_peers.iter().filter(|(peer, len)| *peer == PeerId(2) && *len == 1).count();
        if local_events >= 2 && remote_events >= 2 {
            return;
        }
    }

    panic!("collector should receive local and scanned remote metrics, got {event_from_peers:?}");
}

#[test(tokio::test)]
async fn metrics_service_zero_collect_interval_must_not_panic() {
    let (mut node, _addr) = create_node(true, 1, vec![]).await;
    let service = node.create_service(0.into());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _metrics = MetricsService::new(Some(Duration::ZERO), service, true);
    }));

    assert!(result.is_ok(), "zero metrics collection interval must be rejected or normalized without panicking");
}

#[test(tokio::test)]
async fn metrics_info_must_not_be_accepted_without_scan_request() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = MetricsService::new(Some(Duration::from_secs(60)), node1.create_service(0.into()), true);
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    let _ = service1.recv().await.expect("collector should emit initial local metrics");

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node2_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node2 should connect to node1");

    let forged_metrics = vec![(
        ConnectionId::from(999),
        PeerId::from(123),
        PeerConnectionMetric {
            uptime: 999,
            rtt: 7,
            sent_pkt: 8,
            lost_pkt: 9,
            lost_bytes: 10,
            send_bytes: 11,
            recv_bytes: 12,
            current_mtu: 1200,
        },
    )];
    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_info_for_test(forged_metrics.clone())))
        .expect("attacker should be able to inject a metrics info frame");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service1.recv()).await;

    assert!(
        !matches!(
            delivered,
            Ok(Ok(MetricsServiceEvent::OnPeerConnectionMetric(peer, metrics)))
                if peer == addr2.peer_id() && metrics == forged_metrics
        ),
        "metrics service must not accept unsolicited metric Info frames without a prior Scan request"
    );
}

#[test(tokio::test)]
async fn metrics_scan_must_not_disclose_metrics_to_non_collector() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut metrics1 = MetricsService::new(None, node1.create_service(0.into()), false);
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while metrics1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node2_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node2 should connect to node1");

    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_scan_for_test()))
        .expect("attacker should be able to inject a metrics scan frame");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Unicast(peer, _))) if peer == addr1.peer_id()),
        "metrics service must not disclose metric Info frames to arbitrary peers that send Scan"
    );
}
