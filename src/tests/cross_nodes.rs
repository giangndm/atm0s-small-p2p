use std::time::Duration;

use test_log::test;

use crate::{P2pServiceEvent, router::RouteAction};

use super::create_node;

#[test(tokio::test)]
async fn send_direct() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let data = "from_node1".as_bytes().to_vec();
    service1.send_unicast(addr2.peer_id(), data.clone()).await.expect("should send ok");
    assert_eq!(service2.recv().await, Some(P2pServiceEvent::Unicast(addr1.peer_id(), data)));

    let data = "from_node2".as_bytes().to_vec();
    service2.send_unicast(addr1.peer_id(), data.clone()).await.expect("should send ok");
    assert_eq!(service1.recv().await, Some(P2pServiceEvent::Unicast(addr2.peer_id(), data)));
}

#[test(tokio::test)]
async fn send_error() {
    // without connect 2 peers, it should error to send data
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let data = "from_node1".as_bytes().to_vec();
    assert!(service1.send_unicast(addr2.peer_id(), data.clone()).await.is_err());

    let data = "from_node2".as_bytes().to_vec();
    assert!(service2.send_unicast(addr1.peer_id(), data.clone()).await.is_err());
}

#[test(tokio::test)]
async fn send_relay() {
    let (mut node1, addr1) = create_node(false, 1, vec![]).await;
    let mut service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let relay_peer = addr2.peer_id();
    let node2_requester = node2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let (mut node3, addr3) = create_node(false, 3, vec![]).await;
    let node3_requester = node3.requester();
    let mut service3 = node3.create_service(0.into());
    tokio::spawn(async move { while node3.recv().await.is_ok() {} });

    node2_requester.connect(addr1.clone()).await.expect("should connect success");
    node3_requester.connect(addr2).await.expect("should connect success");

    tokio::time::sleep(Duration::from_secs(1)).await;

    let data = "from_node1".as_bytes().to_vec();
    service1.send_unicast(addr3.peer_id(), data.clone()).await.expect("should send ok");
    assert_eq!(service3.recv().await, Some(P2pServiceEvent::Unicast(relay_peer, data)));

    let data = "from_node3".as_bytes().to_vec();
    service3.send_unicast(addr1.peer_id(), data.clone()).await.expect("should send ok");
    assert_eq!(service1.recv().await, Some(P2pServiceEvent::Unicast(relay_peer, data)));
}

#[test(tokio::test)]
async fn broadcast_direct() {
    let (mut node1, addr1) = create_node(false, 1, vec![]).await;
    let mut service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    log::info!("sending broadcast message");

    let data = "from_node1".as_bytes().to_vec();
    service1.send_broadcast(data.clone()).await.expect("node1 should send broadcast");
    assert_eq!(service2.recv().await, Some(P2pServiceEvent::Broadcast(addr1.peer_id(), data)));

    let data = "from_node2".as_bytes().to_vec();
    service2.send_broadcast(data.clone()).await.expect("node2 should send broadcast");
    assert_eq!(service1.recv().await, Some(P2pServiceEvent::Broadcast(addr2.peer_id(), data)));
}

#[test(tokio::test)]
async fn broadcast_relay() {
    let (mut node1, addr1) = create_node(false, 1, vec![]).await;
    let mut service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let (mut node3, addr3) = create_node(false, 3, vec![addr2.clone()]).await;
    let mut service3 = node3.create_service(0.into());
    tokio::spawn(async move { while node3.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let data = "from_node1".as_bytes().to_vec();
    service1.send_broadcast(data.clone()).await.expect("node1 should send broadcast");
    assert_eq!(service2.recv().await, Some(P2pServiceEvent::Broadcast(addr1.peer_id(), data.clone())));
    assert_eq!(service3.recv().await, Some(P2pServiceEvent::Broadcast(addr1.peer_id(), data)));

    let data = "from_node2".as_bytes().to_vec();
    service2.send_broadcast(data.clone()).await.expect("node2 should send broadcast");
    assert_eq!(service1.recv().await, Some(P2pServiceEvent::Broadcast(addr2.peer_id(), data.clone())));
    assert_eq!(service3.recv().await, Some(P2pServiceEvent::Broadcast(addr2.peer_id(), data)));

    let data = "from_node3".as_bytes().to_vec();
    service3.send_broadcast(data.clone()).await.expect("node3 should send broadcast");
    assert_eq!(service1.recv().await, Some(P2pServiceEvent::Broadcast(addr3.peer_id(), data.clone())));
    assert_eq!(service2.recv().await, Some(P2pServiceEvent::Broadcast(addr3.peer_id(), data)));
}

#[test(tokio::test)]
async fn inbound_unicast_must_not_drop_when_service_queue_is_full() {
    let (mut node1, _addr1) = create_node(false, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let requester1 = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    requester1.connect(addr2.clone()).await.expect("connect should be queued");
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(service1.router().action(&addr2.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("route to node2 should become available");

    for idx in 0..10 {
        service1.send_unicast(addr2.peer_id(), vec![idx as u8]).await.expect("sender should report queued unicast");
    }

    let eleventh = tokio::spawn(async move {
        service1.send_unicast(addr2.peer_id(), vec![10]).await.expect("sender should complete after destination service drains");
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!eleventh.is_finished(), "direct send_unicast should backpressure while the destination service queue is full");

    let mut received = Vec::new();
    match tokio::time::timeout(Duration::from_millis(100), service2.recv()).await {
        Ok(Some(P2pServiceEvent::Unicast(_, data))) => received.push(data),
        other => panic!("expected first queued unicast before unblocking sender, got {other:?}"),
    }

    tokio::time::timeout(Duration::from_secs(2), eleventh)
        .await
        .expect("11th direct unicast should complete after queue capacity returns")
        .expect("11th direct unicast task should not panic");

    for _ in received.len()..11 {
        match tokio::time::timeout(Duration::from_millis(100), service2.recv()).await {
            Ok(Some(P2pServiceEvent::Unicast(_, data))) => received.push(data),
            other => panic!("expected preserved unicast after backpressure, got {other:?}"),
        }
    }

    assert_eq!(
        received.len(),
        11,
        "inbound unicast delivery must apply backpressure or otherwise preserve messages instead of silently dropping when the local service queue is full"
    );
}

#[test(tokio::test)]
async fn unicast_must_not_report_success_when_destination_service_receiver_is_closed() {
    let (mut node1, _addr1) = create_node(false, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let requester1 = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let service2 = node2.create_service(0.into());
    drop(service2);
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    requester1.connect(addr2.clone()).await.expect("connect should be queued");
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(service1.router().action(&addr2.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("route to node2 should become available");

    let result = service1.send_unicast(addr2.peer_id(), b"closed-destination".to_vec()).await;

    assert!(result.is_err(), "send_unicast must not report success when the destination service receiver is already closed");
}

#[test(tokio::test)]
async fn inbound_broadcast_must_not_drop_when_service_queue_is_full() {
    let (mut node1, addr1) = create_node(false, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let requester1 = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    requester1.connect(addr2.clone()).await.expect("connect should be queued");
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(service1.router().action(&addr2.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("route to node2 should become available");

    let expected = 11usize;
    for idx in 0..expected {
        service1.send_broadcast(vec![idx as u8]).await.expect("node1 should send broadcast");
    }

    tokio::time::sleep(Duration::from_millis(300)).await;

    let mut received = Vec::new();
    for _ in 0..expected {
        match tokio::time::timeout(Duration::from_millis(100), service2.recv()).await {
            Ok(Some(P2pServiceEvent::Broadcast(peer, data))) if peer == addr1.peer_id() => received.push(data),
            _ => break,
        }
    }

    assert_eq!(
        received.len(),
        expected,
        "inbound broadcast delivery must apply backpressure or otherwise preserve messages instead of silently dropping when the local service queue is full"
    );
}
