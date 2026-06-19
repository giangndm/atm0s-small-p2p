use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};

use crate::{
    discovery::PeerDiscovery,
    msg::{BroadcastMsgId, P2pServiceId, PeerMessage},
    router::RouteAction,
    ConnectionId, MainEvent, P2pNetwork, P2pNetworkConfig, P2pNetworkEvent, P2pServiceEvent, PeerAddress, PeerConnectionMetric, PeerId, PeerMainData,
    SharedKeyHandshake, SharedRouterTable,
};
use futures::FutureExt;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

use super::{create_node, DEFAULT_CLUSTER_CERT, DEFAULT_CLUSTER_KEY, DEFAULT_SECURE_KEY};

fn make_zero_bidi_server_endpoint(bind_addr: SocketAddr) -> anyhow::Result<Endpoint> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
    let mut server_config = ServerConfig::with_single_cert(vec![cert], priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).expect("transport config should be unique");
    transport_config.max_concurrent_uni_streams(10_000_u32.into());
    transport_config.max_concurrent_bidi_streams(0_u32.into());
    transport_config.max_idle_timeout(Some(Duration::from_secs(5).try_into().expect("timeout should configure")));

    Endpoint::server(server_config, bind_addr).map_err(Into::into)
}

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
async fn peer_stopped_must_remove_stopped_neighbour_immediately() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let stopped_conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            return conn;
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("node2 should connect to node1");

    assert!(
        node2.neighbours.has_peer(&addr1.peer_id()),
        "test setup should have node1 marked as a connected neighbour"
    );

    node2
        .process_internal(100, MainEvent::PeerStopped(stopped_conn, addr1.peer_id()))
        .expect("peer stopped event should process");

    assert!(
        !node2.neighbours.has_peer(&addr1.peer_id()),
        "legitimate PeerStopped must immediately remove the stopped non-seed neighbour"
    );
    assert!(
        node2.neighbours.connected_conns().all(|conn| conn.conn_id() != stopped_conn),
        "stopped connection must not remain eligible for tick sync traffic"
    );
}

#[tokio::test]
async fn peer_stopped_route_must_not_be_resurrected_by_connection_ticker() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let stopped_conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            return conn;
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("node2 should connect to node1");

    node2
        .process_internal(100, MainEvent::PeerStopped(stopped_conn, addr1.peer_id()))
        .expect("peer stopped event should process");
    assert_eq!(
        node2.router.action(&addr1.peer_id()),
        None,
        "test setup should remove the stopped peer route first"
    );

    tokio::time::sleep(Duration::from_millis(1200)).await;

    assert_eq!(
        node2.router.action(&addr1.peer_id()),
        None,
        "PeerStopped route cleanup must not be undone by the still-running connection task ticker"
    );
}

#[tokio::test]
async fn stopped_peer_route_must_not_be_resurrected_by_third_party_sync() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let stopped = PeerId::from(2);
    let relay = PeerId::from(3);
    let stopped_conn = ConnectionId::from(20);
    let relay_conn = ConnectionId::from(30);

    node.router.set_direct(stopped_conn, stopped, 10);
    node.router.set_direct(relay_conn, relay, 10);

    node.process_internal(100, MainEvent::PeerStopped(stopped_conn, stopped))
        .expect("stop should process");
    assert_eq!(
        node.router.action(&stopped),
        None,
        "test setup should remove the stopped peer route"
    );

    let remote_router = SharedRouterTable::new(relay);
    remote_router.set_direct(ConnectionId::from(40), stopped, 5);
    let route = remote_router.create_sync(&node.local_id);
    let advertise = node.discovery.create_sync_for(110, &node.local_id);

    node.process_internal(110, MainEvent::PeerData(relay_conn, relay, PeerMainData::Sync { route, advertise }))
        .expect("relay sync should process");

    assert_eq!(
        node.router.action(&stopped),
        None,
        "a graceful-stop tombstone must prevent third-party route sync from making the stopped peer routable again"
    );
}

#[tokio::test]
async fn discovery_timeout_must_remove_route_to_expired_non_seed() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let relay = PeerId::from(2);
    let expired = PeerId::from(3);
    let relay_conn = ConnectionId::from(20);
    let expired_addr: PeerAddress = "3@127.0.0.1:9003".parse().expect("expired address should parse");

    let mut remote_discovery = PeerDiscovery::default();
    remote_discovery.enable_local(expired, expired_addr.network_address().clone());
    node.discovery.apply_sync(100, remote_discovery.create_sync_for(100, &node.local_id));

    node.router.set_direct(relay_conn, relay, 10);
    let remote_router = SharedRouterTable::new(relay);
    remote_router.set_direct(ConnectionId::from(30), expired, 5);
    node.router.apply_sync(relay_conn, remote_router.create_sync(&node.local_id));
    assert_eq!(
        node.router.action(&expired),
        Some(RouteAction::Next(relay_conn)),
        "test setup should make the expired peer routable through the relay"
    );

    node.process_tick(30_100).expect("tick should process discovery timeout");

    assert_eq!(
        node.router.action(&expired),
        None,
        "when a non-seed expires from discovery, stale router paths to it must not remain usable"
    );
}

#[tokio::test]
async fn forged_peer_stopped_must_not_be_forwarded_to_other_neighbours() {
    let (mut relay, relay_addr) = create_node(true, 2, vec![]).await;
    tokio::spawn(async move { while relay.recv().await.is_ok() {} });

    let (mut attacker, _attacker_addr) = create_node(false, 1, vec![relay_addr.clone()]).await;
    let attacker_ctx = attacker.ctx.clone();
    tokio::spawn(async move { while attacker.recv().await.is_ok() {} });

    let (mut victim, victim_addr) = create_node(false, 4, vec![relay_addr.clone()]).await;
    tokio::spawn(async move { while victim.recv().await.is_ok() {} });

    let (mut observer, _observer_addr) = create_node(false, 3, vec![relay_addr]).await;
    let observer_router = observer.router.clone();
    tokio::spawn(async move { while observer.recv().await.is_ok() {} });
    let victim_peer = victim_addr.peer_id();

    let attacker_conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let (Some(conn), Some(RouteAction::Next(_))) = (attacker_ctx.conns().into_iter().next(), observer_router.action(&victim_peer)) {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("observer should learn a route to the victim and attacker should connect to relay");

    attacker_conn
        .try_send(PeerMessage::PeerStopped(victim_peer))
        .expect("attacker should be able to send forged stop over authenticated relay connection");

    let route_was_removed = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if observer_router.action(&victim_peer).is_none() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or(false);

    assert!(
        !route_was_removed,
        "a relay must not forward forged PeerStopped for an unrelated victim to other neighbours"
    );
}

#[tokio::test]
async fn peer_stopped_forwarding_must_be_deduplicated_in_mesh() {
    let (mut stopped, stopped_addr) = create_node(false, 1, vec![]).await;
    let stopped_requester = stopped.requester();

    let (mut node_b, addr_b) = create_node(false, 2, vec![]).await;
    let requester_b = node_b.requester();
    let (mut node_c, addr_c) = create_node(false, 3, vec![]).await;
    let requester_c = node_c.requester();
    let (mut node_d, addr_d) = create_node(false, 4, vec![]).await;
    let requester_d = node_d.requester();

    stopped_requester.try_connect(addr_b.clone());
    requester_b.try_connect(addr_c.clone());
    requester_c.try_connect(addr_d.clone());
    requester_d.try_connect(addr_b.clone());

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if matches!(stopped.router.action(&addr_b.peer_id()), Some(RouteAction::Next(_)))
                && matches!(node_b.router.action(&addr_c.peer_id()), Some(RouteAction::Next(_)))
                && matches!(node_c.router.action(&addr_d.peer_id()), Some(RouteAction::Next(_)))
                && matches!(node_d.router.action(&addr_b.peer_id()), Some(RouteAction::Next(_)))
            {
                break;
            }

            tokio::select! {
                _ = stopped.recv() => {}
                _ = node_b.recv() => {}
                _ = node_c.recv() => {}
                _ = node_d.recv() => {}
            }
        }
    })
    .await
    .expect("test mesh should become connected");

    while stopped.main_rx.try_recv().is_ok() {}
    while node_b.main_rx.try_recv().is_ok() {}
    while node_c.main_rx.try_recv().is_ok() {}
    while node_d.main_rx.try_recv().is_ok() {}

    stopped.shutdown_gracefully().await;

    let mut stopped_events = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(deadline) => break,
            event = node_b.main_rx.recv() => {
                if matches!(event, Some(MainEvent::PeerStopped(_, peer)) if peer == stopped_addr.peer_id()) {
                    stopped_events += 1;
                }
            }
            event = node_c.main_rx.recv() => {
                if matches!(event, Some(MainEvent::PeerStopped(_, peer)) if peer == stopped_addr.peer_id()) {
                    stopped_events += 1;
                }
            }
            event = node_d.main_rx.recv() => {
                if matches!(event, Some(MainEvent::PeerStopped(_, peer)) if peer == stopped_addr.peer_id()) {
                    stopped_events += 1;
                }
            }
        }
    }

    assert!(
        stopped_events <= 3,
        "one graceful stop should be forwarded at most once per live node, got {stopped_events} PeerStopped events"
    );
}

#[tokio::test]
async fn peer_stopped_must_not_block_connection_task_on_full_main_queue() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let service1_requester = service1.requester();
    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = node2.create_service(0.into());

    let conn_to_node1 = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            return conn;
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("node2 should connect to node1");

    for idx in 0..10 {
        node2
            .main_tx
            .try_send(MainEvent::PeerStats(
                ConnectionId::from(1000 + idx),
                PeerId::from(1000 + idx),
                PeerConnectionMetric {
                    uptime: 1,
                    rtt: 1,
                    sent_pkt: 1,
                    lost_pkt: 0,
                    lost_bytes: 0,
                    send_bytes: 1,
                    recv_bytes: 1,
                    current_mtu: 1200,
                },
            ))
            .expect("test should fill node2 main queue");
    }

    let conn = node1.ctx.conns().into_iter().next().expect("node1 should have a connection to node2");
    conn.try_send(PeerMessage::PeerStopped(addr1.peer_id()))
        .expect("stop notification should enqueue to peer connection");

    tokio::time::sleep(Duration::from_millis(100)).await;

    service1_requester
        .send_unicast(addr2.peer_id(), b"after-stop".to_vec())
        .await
        .expect("unicast send should enqueue to connection");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert_eq!(
        delivered,
        Ok(Some(P2pServiceEvent::Unicast(addr1.peer_id(), b"after-stop".to_vec()))),
        "PeerStopped handling must not block the connection task when the main event queue is full"
    );

    drop(service1);
    drop(service2);
    let _ = conn_to_node1;
}

#[tokio::test]
async fn peer_connected_must_not_block_authenticated_connection_run_loop_on_full_main_queue() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = node1.create_service(0.into());

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let _service2 = node2.create_service(0.into());
    let requester2 = node2.requester();

    assert!(
        matches!(
            tokio::time::timeout(Duration::from_secs(1), node1.recv()).await,
            Ok(Ok(P2pNetworkEvent::Continue))
        ),
        "node1 should process initial tick"
    );
    assert!(
        matches!(
            tokio::time::timeout(Duration::from_secs(1), node2.recv()).await,
            Ok(Ok(P2pNetworkEvent::Continue))
        ),
        "node2 should process initial tick"
    );

    requester2.try_connect(addr1.clone());

    assert!(
        matches!(
            tokio::time::timeout(Duration::from_secs(2), node2.recv()).await,
            Ok(Ok(P2pNetworkEvent::Continue))
        ),
        "node2 should process the queued connect command"
    );

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if matches!(node1.recv().await, Ok(P2pNetworkEvent::Continue)) {
                break;
            }
        }
    })
    .await
    .expect("node1 should accept the incoming connection");

    for idx in 0..10 {
        node1
            .main_tx
            .try_send(MainEvent::PeerStats(
                ConnectionId::from(10_000 + idx),
                PeerId::from(10_000 + idx),
                PeerConnectionMetric {
                    uptime: 1,
                    rtt: 1,
                    sent_pkt: 0,
                    lost_pkt: 0,
                    lost_bytes: 0,
                    send_bytes: 0,
                    recv_bytes: 0,
                    current_mtu: 1200,
                },
            ))
            .expect("test should fill node1 main queue");
    }

    let conn_to_node1 = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(conn) = node2.ctx.conns().into_iter().next() {
                return conn;
            }
            let _ = node2.recv().await;
        }
    })
    .await
    .expect("node2 should register a connection alias to node1");

    conn_to_node1
        .try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), b"after-auth".to_vec()))
        .expect("test unicast should enqueue to node2 peer task");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service1.recv()).await;

    assert_eq!(
        delivered,
        Ok(Some(P2pServiceEvent::Unicast(addr2.peer_id(), b"after-auth".to_vec()))),
        "authenticated connection must process traffic even if PeerConnected event delivery is backpressured"
    );
}

#[tokio::test]
async fn outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open() {
    let raw_server = make_zero_bidi_server_endpoint(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)).expect("raw server should bind");
    let raw_addr = raw_server.local_addr().expect("raw server should have a local addr");
    let raw_server_task = tokio::spawn(async move {
        if let Some(incoming) = raw_server.accept().await {
            let _connection = incoming.await.expect("raw server should accept transport connection");
            std::future::pending::<()>().await;
        }
    });

    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let requester = node.requester();
    let raw_peer = PeerAddress::new(PeerId::from(200), raw_addr.into());

    requester.try_connect(raw_peer);

    tokio::time::timeout(Duration::from_secs(1), async {
        while node.neighbours.len() == 0 {
            let _ = node.recv().await;
        }
    })
    .await
    .expect("connect command should create a pending neighbour");

    let cleaned = tokio::time::timeout(Duration::from_secs(2), async {
        while node.neighbours.len() != 0 {
            let _ = node.recv().await;
        }
    })
    .await;

    raw_server_task.abort();

    assert!(
        cleaned.is_ok(),
        "outbound setup must time out and remove the pending neighbour when the QUIC peer never permits opening the P2P control stream; pending neighbours: {}",
        node.neighbours.len()
    );
}

#[tokio::test]
async fn peer_disconnected_must_not_block_alias_cleanup_on_full_main_queue() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let live_conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            return conn;
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("node2 should connect to node1");

    assert!(
        node2.ctx.conn(&live_conn).is_some(),
        "test setup should have a registered live alias"
    );

    for idx in 0..10 {
        node2
            .main_tx
            .try_send(MainEvent::PeerStats(
                ConnectionId::from(3000 + idx),
                PeerId::from(3000 + idx),
                PeerConnectionMetric {
                    uptime: 1,
                    rtt: 1,
                    sent_pkt: 0,
                    lost_pkt: 0,
                    lost_bytes: 0,
                    send_bytes: 0,
                    recv_bytes: 0,
                    current_mtu: 1200,
                },
            ))
            .expect("main queue should accept filler event");
    }

    node1.shutdown();

    tokio::time::sleep(Duration::from_millis(200)).await;

    assert!(
        node2.ctx.conn(&live_conn).is_none(),
        "disconnect cleanup must unregister the peer alias even when the bounded main event queue is full"
    );
}

#[tokio::test]
async fn stale_peer_connected_event_must_not_install_unusable_route() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let stale_conn = ConnectionId::from(404);
    let peer = PeerId::from(2);

    node.process_internal(100, MainEvent::PeerConnected(stale_conn, peer, 10))
        .expect("stale peer connected event should process");

    assert_eq!(
        node.router.action(&peer),
        None,
        "PeerConnected for an unknown connection id must not create a route without a live peer alias"
    );
}

#[tokio::test]
async fn stale_peer_connect_error_must_not_remove_live_neighbour() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let live_conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                _ = node1.recv() => {}
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            return conn;
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("node2 should connect to node1");

    assert!(
        node2.neighbours.has_peer(&addr1.peer_id()),
        "test setup should have a live connected neighbour"
    );

    node2
        .process_internal(
            100,
            MainEvent::PeerConnectError(live_conn, Some(addr1.peer_id()), anyhow::anyhow!("stale connect error")),
        )
        .expect("stale connect error should process");

    assert!(
        node2.neighbours.has_peer(&addr1.peer_id()),
        "PeerConnectError for an already-connected neighbour must be ignored instead of removing the live connection"
    );
}

#[tokio::test]
async fn peer_connected_must_not_rebind_existing_connection_to_different_peer() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let conn = ConnectionId::from(88);
    let real_peer = PeerId::from(2);
    let forged_peer = PeerId::from(99);

    node.router.set_direct(conn, real_peer, 10);

    let event = node
        .process_internal(100, MainEvent::PeerConnected(conn, forged_peer, 10))
        .expect("duplicate connected event should process");

    assert_eq!(
        event,
        P2pNetworkEvent::Continue,
        "PeerConnected for an already-bound connection must be ignored when the peer id changes"
    );
    assert_eq!(
        node.router.action(&real_peer),
        Some(RouteAction::Next(conn)),
        "the original route owner must remain attached to the connection"
    );
    assert_eq!(
        node.router.action(&forged_peer),
        None,
        "a duplicate PeerConnected event must not create a second direct route for the same connection"
    );
}

#[tokio::test]
async fn stale_peer_data_event_must_not_panic_without_direct_route() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let stale_conn = ConnectionId::from(404);
    let peer = PeerId::from(2);
    let route = node.router.create_sync(&peer);
    let advertise = node.discovery.create_sync_for(100, &peer);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        node.process_internal(100, MainEvent::PeerData(stale_conn, peer, PeerMainData::Sync { route, advertise }))
    }));

    assert!(
        matches!(result, Ok(Ok(P2pNetworkEvent::Continue))),
        "PeerData for a stale connection id must be ignored or return an error instead of panicking"
    );
}

#[tokio::test]
async fn peer_data_must_validate_peer_matches_connection() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let conn = ConnectionId::from(10);
    let real_peer = PeerId::from(2);
    let forged_peer = PeerId::from(99);
    let advertised_peer = PeerId::from(4);
    let remote_router = SharedRouterTable::new(real_peer);

    node.router.set_direct(conn, real_peer, 10);
    remote_router.set_direct(ConnectionId::from(20), advertised_peer, 5);

    let route = remote_router.create_sync(&node.local_id);
    let advertise = node.discovery.create_sync_for(100, &node.local_id);

    node.process_internal(100, MainEvent::PeerData(conn, forged_peer, PeerMainData::Sync { route, advertise }))
        .expect("mismatched peer data event should process");

    assert_eq!(
        node.router.action(&advertised_peer),
        None,
        "PeerData must be ignored when the reported peer id does not match the live connection owner"
    );
}

#[tokio::test]
async fn stale_peer_stats_event_must_not_publish_metrics_for_unknown_connection() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let stale_conn = ConnectionId::from(404);
    let peer = PeerId::from(2);
    let metrics = PeerConnectionMetric {
        uptime: 1,
        rtt: 2,
        sent_pkt: 3,
        lost_pkt: 4,
        lost_bytes: 5,
        send_bytes: 6,
        recv_bytes: 7,
        current_mtu: 1200,
    };

    node.process_internal(100, MainEvent::PeerStats(stale_conn, peer, metrics))
        .expect("stale peer stats event should process");

    assert!(
        node.ctx.metrics().is_empty(),
        "PeerStats for an unknown connection id must not be exported as live connection metrics"
    );
}

#[tokio::test]
async fn peer_stats_must_validate_peer_matches_connection() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let conn = ConnectionId::from(66);
    let real_peer = PeerId::from(2);
    let forged_peer = PeerId::from(99);
    let metrics = PeerConnectionMetric {
        uptime: 1,
        rtt: 2,
        sent_pkt: 3,
        lost_pkt: 4,
        lost_bytes: 5,
        send_bytes: 6,
        recv_bytes: 7,
        current_mtu: 1200,
    };

    node.router.set_direct(conn, real_peer, 10);
    node.process_internal(100, MainEvent::PeerStats(conn, forged_peer, metrics))
        .expect("stats event should process");

    assert!(
        node.ctx.metrics().is_empty(),
        "PeerStats must be ignored when the reported peer id does not match the connection owner"
    );
}

#[tokio::test]
async fn stale_peer_disconnected_event_must_not_emit_user_disconnect() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let stale_conn = ConnectionId::from(404);
    let peer = PeerId::from(2);

    let event = node
        .process_internal(100, MainEvent::PeerDisconnected(stale_conn, peer))
        .expect("stale disconnect event should process");

    assert_eq!(
        event,
        P2pNetworkEvent::Continue,
        "PeerDisconnected for an unknown connection id must be ignored instead of emitted to users"
    );
}

#[tokio::test]
async fn peer_disconnected_must_validate_peer_matches_connection() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let conn = ConnectionId::from(77);
    let real_peer = PeerId::from(2);
    let forged_peer = PeerId::from(99);

    node.router.set_direct(conn, real_peer, 10);

    let event = node
        .process_internal(100, MainEvent::PeerDisconnected(conn, forged_peer))
        .expect("disconnect event should process");

    assert_eq!(
        event,
        P2pNetworkEvent::Continue,
        "PeerDisconnected must be ignored when the reported peer id does not match the connection owner"
    );
    assert_eq!(
        node.router.action(&real_peer),
        Some(RouteAction::Next(conn)),
        "a forged disconnect peer id must not remove the route for the real connection peer"
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
async fn connect_to_own_peer_address_must_fail() {
    let (mut node, addr) = create_node(true, 1, vec![]).await;
    let requester = node.requester();
    let connect = requester.connect(addr.clone());
    tokio::pin!(connect);

    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            tokio::select! {
                res = &mut connect => return res,
                event = node.recv() => {
                    event.expect("node should keep running while self-connect is tested");
                }
            }
        }
    })
    .await
    .expect("self-connect attempt should finish");

    assert!(
        result.is_err(),
        "connect() to the node's own advertised peer address must fail instead of creating a self connection"
    );
}

#[tokio::test]
async fn concurrent_connects_to_same_peer_must_be_coalesced() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, _addr2) = create_node(false, 2, vec![]).await;
    let requester2 = node2.requester();

    for _ in 0..4 {
        requester2.try_connect(addr1.clone());
    }

    let mut connected_to_node1 = 0;
    let deadline = tokio::time::sleep(Duration::from_secs(2));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            event = node2.recv() => {
                if let P2pNetworkEvent::PeerConnected(_conn, peer) = event.expect("node2 should keep running") {
                    if peer == addr1.peer_id() {
                        connected_to_node1 += 1;
                    }
                }
            }
        }
    }

    assert!(
        connected_to_node1 <= 1,
        "concurrent connect attempts to the same peer must be coalesced, got {connected_to_node1} connected events"
    );
}

#[tokio::test]
async fn requester_connect_backlog_must_be_bounded() {
    const MAX_PENDING_CONNECTS: usize = 1024;
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let requester = node.requester();

    for peer in 0..=MAX_PENDING_CONNECTS {
        let target: PeerAddress = format!("{}@127.0.0.1:10000", peer + 10).parse().expect("target address should parse");
        requester.try_connect(target);
    }

    assert!(
        node.control_rx.len() <= MAX_PENDING_CONNECTS,
        "pending requester connect commands must be bounded, got {}",
        node.control_rx.len()
    );
}

#[tokio::test]
async fn inbound_duplicate_connections_from_same_peer_must_be_coalesced() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;

    let (mut node2, _addr2) = create_node(false, 2, vec![]).await;
    let requester2 = node2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    for _ in 0..4 {
        requester2.try_connect(addr1.clone());
    }

    let mut connected_from_node2 = 0;
    let deadline = tokio::time::sleep(Duration::from_secs(2));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            event = node1.recv() => {
                if let P2pNetworkEvent::PeerConnected(_conn, peer) = event.expect("node1 should keep running") {
                    if peer == PeerId::from(2) {
                        connected_from_node2 += 1;
                    }
                }
            }
        }
    }

    assert!(
        connected_from_node2 <= 1,
        "inbound duplicate connections from one peer must be coalesced, got {connected_from_node2} connected events"
    );
}

#[tokio::test]
async fn requester_connect_after_network_drop_returns_error_not_panic() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let requester = node.requester();
    drop(node);

    let target: PeerAddress = "2@127.0.0.1:10000".parse().expect("target address should parse");
    let result = std::panic::AssertUnwindSafe(requester.connect(target)).catch_unwind().await;

    assert!(matches!(result, Ok(Err(_))), "connect on a stale requester must return Err instead of panicking");
}

#[tokio::test]
async fn requester_try_connect_after_network_drop_must_not_panic() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let requester = node.requester();
    drop(node);

    let target: PeerAddress = "2@127.0.0.1:10000".parse().expect("test address should parse");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        requester.try_connect(target);
    }));

    assert!(result.is_ok(), "try_connect on a stale requester must return or no-op instead of panicking");
}

#[tokio::test]
async fn dropped_service_requester_must_not_continue_sending_unicast() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let stale_requester = service1.requester();
    let node1_requester = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    node1_requester.connect(addr2.clone()).await.expect("connect should succeed");
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(stale_requester.router().action(&addr2.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("route to node2 should become available");
    drop(service1);

    let data = b"stale-service-unicast".to_vec();
    stale_requester
        .send_unicast(addr2.peer_id(), data.clone())
        .await
        .expect("stale requester send should not panic");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Unicast(_, received))) if received == data),
        "a requester cloned from a dropped P2pService must not continue sending unicast messages"
    );
}

#[tokio::test]
async fn dropped_service_requester_must_not_continue_sending_broadcast() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let stale_requester = service1.requester();
    let node1_requester = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let mut service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    node1_requester.connect(addr2.clone()).await.expect("connect should succeed");
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(stale_requester.router().action(&addr2.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("route to node2 should become available");
    drop(service1);

    let data = b"stale-service-broadcast".to_vec();
    stale_requester.send_broadcast(data.clone()).await;

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Broadcast(_, received))) if received == data),
        "a requester cloned from a dropped P2pService must not continue sending broadcast messages"
    );
}

#[tokio::test]
async fn duplicate_service_creation_must_not_panic() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let _first = node.create_service(0.into());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _second = node.create_service(0.into());
    }));

    assert!(result.is_ok(), "creating a duplicate service id must return a recoverable error instead of panicking");
}

#[tokio::test]
async fn dropped_service_id_must_be_reusable() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;
    let service = node.create_service(0.into());
    drop(service);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _replacement = node.create_service(0.into());
    }));

    assert!(
        result.is_ok(),
        "dropping a service receiver must unregister the service id or allow a replacement without panicking"
    );
}

#[tokio::test]
async fn out_of_range_service_id_must_not_panic() {
    let (mut node, _addr) = create_node(false, 1, vec![]).await;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _service = node.create_service(P2pServiceId::from(256u16));
    }));

    assert!(
        result.is_ok(),
        "creating an out-of-range service id must return a recoverable error instead of panicking"
    );
}

#[tokio::test]
async fn inbound_out_of_range_unicast_service_id_must_not_kill_connection() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let node1_ctx = node1.ctx.clone();
    let _service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
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

    conn.try_send(PeerMessage::Unicast(
        addr1.peer_id(),
        addr2.peer_id(),
        P2pServiceId::from(256u16),
        b"bad-service-id".to_vec(),
    ))
    .expect("out-of-range unicast should be sent over the authenticated connection");

    let data = b"valid-after-bad-service-id".to_vec();
    tokio::time::sleep(Duration::from_millis(50)).await;
    conn.try_send(PeerMessage::Unicast(addr1.peer_id(), addr2.peer_id(), 0.into(), data.clone()))
        .expect("connection task must survive an inbound unknown out-of-range service id");

    let event = tokio::time::timeout(Duration::from_secs(1), service2.recv())
        .await
        .expect("valid follow-up unicast should still be delivered")
        .expect("destination service channel should stay open");

    assert_eq!(
        event,
        P2pServiceEvent::Unicast(addr1.peer_id(), data),
        "inbound unknown out-of-range service id must be rejected without killing the connection"
    );
}

#[tokio::test]
async fn zero_network_tick_interval_must_not_panic() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let priv_key: PrivatePkcs8KeyDer<'_> = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
    let addr = {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("test socket should bind");
        socket.local_addr().expect("test socket should have local address")
    };

    let result = std::panic::AssertUnwindSafe(P2pNetwork::new(P2pNetworkConfig {
        peer_id: PeerId::from(1),
        listen_addr: addr,
        advertise: None,
        priv_key,
        cert,
        tick_ms: 0,
        seeds: vec![],
        secure: SharedKeyHandshake::from(DEFAULT_SECURE_KEY),
    }))
    .catch_unwind()
    .await;

    assert!(result.is_ok(), "zero network tick interval must be rejected or normalized without panicking");
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
