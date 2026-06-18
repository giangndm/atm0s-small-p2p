use std::time::Duration;

use crate::{
    msg::{BroadcastMsgId, PeerMessage},
    router::RouteAction,
    ConnectionId, MainEvent, P2pServiceEvent, PeerAddress, PeerId,
};

use super::create_node;

#[tokio::test]
async fn forged_peer_stopped_must_not_remove_third_party_route() {
    let (mut node, _addr) = create_node(false, 3, vec![]).await;
    let attacker = PeerId::from(1);
    let victim = PeerId::from(2);
    let attacker_conn = ConnectionId::from(10);
    let victim_conn = ConnectionId::from(20);

    node.router.set_direct(attacker_conn, attacker, 10);
    node.router.set_direct(victim_conn, victim, 10);

    assert_eq!(node.router.action(&victim), Some(RouteAction::Next(victim_conn)));

    node.process_internal(100, MainEvent::PeerStopped(attacker_conn, victim)).expect("peer stopped event should process");

    assert_eq!(
        node.router.action(&victim),
        Some(RouteAction::Next(victim_conn)),
        "a peer must not be able to remove a third-party route with forged PeerStopped"
    );
}

#[tokio::test]
async fn peer_stopped_for_seed_must_not_remove_active_seed_route() {
    let seed_addr: PeerAddress = "1@127.0.0.1:10000".parse().expect("seed address should parse");
    let (mut node, _addr) = create_node(false, 3, vec![seed_addr.clone()]).await;
    let attacker = PeerId::from(2);
    let seed = seed_addr.peer_id();
    let attacker_conn = ConnectionId::from(10);
    let seed_conn = ConnectionId::from(20);

    node.router.set_direct(attacker_conn, attacker, 10);
    node.router.set_direct(seed_conn, seed, 10);

    node.process_internal(100, MainEvent::PeerStopped(attacker_conn, seed)).expect("peer stopped event should process");

    assert_eq!(
        node.router.action(&seed),
        Some(RouteAction::Next(seed_conn)),
        "a stopped notification must not remove the active route to a configured seed"
    );
}

#[tokio::test]
async fn unicast_source_must_be_bound_to_authenticated_connection_peer() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let node1_ctx = node1.ctx.clone();
    let _service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node1_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node1 should connect to node2");

    let forged_source = PeerId::from(99);
    let data = b"forged-source".to_vec();
    conn.try_send(PeerMessage::Unicast(forged_source, addr2.peer_id(), 0.into(), data.clone()))
        .expect("forged message should be sent over the authenticated connection");

    let event = tokio::time::timeout(Duration::from_secs(1), service2.recv())
        .await
        .expect("destination service should receive or reject the injected unicast")
        .expect("destination service channel should stay open");

    assert_ne!(
        event,
        P2pServiceEvent::Unicast(forged_source, data),
        "service must not observe a sender id that was forged inside the message body"
    );
}

#[tokio::test]
async fn broadcast_source_must_be_bound_to_authenticated_connection_peer() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let node1_ctx = node1.ctx.clone();
    let _service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, _addr2) = create_node(false, 2, vec![addr1]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node1_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node1 should connect to node2");

    let forged_source = PeerId::from(99);
    let data = b"forged-broadcast-source".to_vec();
    conn.try_send(PeerMessage::Broadcast(forged_source, 0.into(), BroadcastMsgId::rand(), data.clone()))
        .expect("forged broadcast should be sent over the authenticated connection");

    let event = tokio::time::timeout(Duration::from_secs(1), service2.recv())
        .await
        .expect("destination service should receive or reject the injected broadcast")
        .expect("destination service channel should stay open");

    assert_ne!(
        event,
        P2pServiceEvent::Broadcast(forged_source, data),
        "service must not observe a broadcast sender id that was forged inside the message body"
    );
}

#[tokio::test]
async fn connect_must_fail_when_remote_peer_id_does_not_match_address() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, _addr2) = create_node(false, 2, vec![]).await;
    let requester2 = node2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let wrong_addr = PeerAddress::new(PeerId::from(99), addr1.network_address().clone());

    assert!(requester2.connect(wrong_addr).await.is_err(), "connect must not return Ok before the remote peer id is authenticated");
}

#[tokio::test]
async fn broadcast_dedup_must_include_source_not_only_message_id() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let node1_ctx = node1.ctx.clone();
    let _service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node1_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node1 should connect to node2");

    let msg_id = BroadcastMsgId::rand();
    let attack_data = b"poisoned-cache".to_vec();
    let legitimate_data = b"legitimate-broadcast".to_vec();

    conn.try_send(PeerMessage::Broadcast(PeerId::from(99), 0.into(), msg_id, attack_data.clone()))
        .expect("attacker should be able to send first broadcast");
    assert_eq!(
        service2.recv().await,
        Some(P2pServiceEvent::Broadcast(PeerId::from(99), attack_data)),
        "the first broadcast establishes the duplicate-cache entry"
    );

    conn.try_send(PeerMessage::Broadcast(addr1.peer_id(), 0.into(), msg_id, legitimate_data.clone()))
        .expect("legitimate broadcast should be sent with the same message id");

    let second = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;
    assert_eq!(
        second,
        Ok(Some(P2pServiceEvent::Broadcast(addr1.peer_id(), legitimate_data))),
        "broadcast duplicate suppression must not let one source poison the same id for another source"
    );
}
