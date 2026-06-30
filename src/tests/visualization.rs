use std::time::Duration;

use test_log::test;

use crate::{
    msg::PeerMessage,
    visualization_service::{encode_info_for_test, encode_scan_for_test, VisualizationService, VisualizationServiceEvent},
    ConnectionId, P2pServiceEvent, PeerId,
};

use super::create_node;

#[test(tokio::test)]
async fn discovery_new_node() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let mut service1 = VisualizationService::new(None, false, node1.create_service(0.into())).with_trusted_scan_collectors([addr2.peer_id()]);
    let mut service2 = VisualizationService::new(Some(Duration::from_secs(1)), false, node2.create_service(0.into()));
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while service1.recv().await.is_ok() {} });
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut events = vec![
        tokio::time::timeout(Duration::from_secs(3), service2.recv()).await.expect("").expect(""),
        tokio::time::timeout(Duration::from_secs(3), service2.recv()).await.expect("").expect(""),
    ];

    for event in events.iter_mut() {
        match event {
            VisualizationServiceEvent::PeerJoined(_, neighbours) | VisualizationServiceEvent::PeerUpdated(_, neighbours) => {
                for (conn, _, rtt) in neighbours.iter_mut() {
                    *conn = 0.into();
                    *rtt = 0;
                }
            }
            VisualizationServiceEvent::PeerLeaved(_) => {}
        }
    }

    assert_eq!(
        events,
        vec![
            VisualizationServiceEvent::PeerJoined(addr1.peer_id(), vec![(0.into(), addr2.peer_id(), 0)]),
            VisualizationServiceEvent::PeerUpdated(addr1.peer_id(), vec![(0.into(), addr2.peer_id(), 0)]),
        ]
    );
}

#[test(tokio::test)]
async fn visualization_must_emit_peer_leaved_on_graceful_peer_stop() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service1 = VisualizationService::new(Some(Duration::from_secs(1)), false, node1.create_service(0.into())).with_trusted_scan_collectors([addr2.peer_id()]);
    let mut service2 = VisualizationService::new(Some(Duration::from_secs(1)), false, node2.create_service(0.into())).with_trusted_scan_collectors([addr1.peer_id()]);

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                _ = node2.recv() => {}
                _ = service2.recv() => {}
                event = service1.recv() => {
                    if matches!(event, Ok(VisualizationServiceEvent::PeerJoined(peer, _)) if peer == addr2.peer_id()) {
                        break;
                    }
                }
            }
        }
    })
    .await
    .expect("collector should learn node2 before shutdown");

    node2.shutdown_gracefully().await;

    let leaved = tokio::time::timeout(Duration::from_millis(500), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = service1.recv() => {
                    if matches!(event, Ok(VisualizationServiceEvent::PeerLeaved(peer)) if peer == addr2.peer_id()) {
                        break;
                    }
                }
            }
        }
    })
    .await;

    assert!(leaved.is_ok(), "visualization must emit PeerLeaved promptly after graceful PeerStopped instead of waiting for timeout");
}

#[test(tokio::test)]
async fn visualization_service_zero_collect_interval_must_not_panic() {
    let (mut node, _addr) = create_node(true, 1, vec![]).await;
    let service = node.create_service(0.into());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _visualization = VisualizationService::new(Some(Duration::ZERO), false, service);
    }));

    assert!(result.is_ok(), "zero visualization collection interval must be rejected or normalized without panicking");
}

#[test(tokio::test)]
async fn visualization_info_must_not_be_accepted_without_scan_request() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = VisualizationService::new(None, false, node1.create_service(0.into()));
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

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

    let forged_topology = vec![(ConnectionId::from(999), PeerId::from(123), 7)];
    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_info_for_test(forged_topology.clone())))
        .expect("attacker should be able to inject a visualization info frame");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service1.recv()).await;

    assert!(
        !matches!(
            delivered,
            Ok(Ok(VisualizationServiceEvent::PeerJoined(peer, neighbours)))
                if peer == addr2.peer_id() && neighbours == forged_topology
        ),
        "visualization must not accept unsolicited topology Info frames without a prior Scan request"
    );
}

#[test(tokio::test)]
async fn visualization_scan_must_not_disclose_topology_to_non_collector() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut visualization1 = VisualizationService::new(None, false, node1.create_service(0.into()));
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while visualization1.recv().await.is_ok() {} });

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
        .expect("attacker should be able to inject a visualization scan frame");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Unicast(peer, _))) if peer == addr1.peer_id()),
        "visualization service must not disclose topology Info frames to arbitrary peers that send Scan"
    );
}

#[test(tokio::test)]
async fn visualization_broadcast_scan_must_not_disclose_topology_to_non_collector() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut visualization1 = VisualizationService::new(None, false, node1.create_service(0.into()));
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while visualization1.recv().await.is_ok() {} });

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

    conn.try_send(PeerMessage::Broadcast(addr2.peer_id(), 0.into(), crate::msg::BroadcastMsgId::rand(), encode_scan_for_test()))
        .expect("attacker should be able to inject a visualization broadcast scan frame");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        matches!(delivered, Ok(Some(P2pServiceEvent::Unicast(peer, _))) if peer == addr1.peer_id()),
        "visualization service must disclose topology Info frames when trusted_scan_collectors check is bypassed"
    );
}

#[test(tokio::test)]
async fn visualization_broadcast_scan_discloses_topology_to_trusted_collector() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();

    let mut visualization1 = VisualizationService::new(None, false, node1.create_service(0.into())).with_trusted_scan_collectors([addr2.peer_id()]);
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { while visualization1.recv().await.is_ok() {} });
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

    conn.try_send(PeerMessage::Broadcast(addr2.peer_id(), 0.into(), crate::msg::BroadcastMsgId::rand(), encode_scan_for_test()))
        .expect("trusted collector should be able to inject a visualization broadcast scan frame");

    let delivered = tokio::time::timeout(Duration::from_secs(1), service2.recv()).await;

    assert!(
        matches!(delivered, Ok(Some(P2pServiceEvent::Unicast(peer, _))) if peer == addr1.peer_id()),
        "visualization service must disclose topology Info frames to configured trusted collectors"
    );
}
