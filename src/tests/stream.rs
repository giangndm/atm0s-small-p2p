use std::{
    net::UdpSocket,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use super::create_node;
use crate::{
    msg::{P2pServiceId, StreamConnectReq},
    quic::make_server_endpoint,
    router::RouteAction,
    secure::HandshakeProtocol,
    stream::{wait_object, write_object, P2pQuicStream},
    ConnectionId, P2pNetworkEvent, P2pServiceEvent, PeerId, SharedKeyHandshake, SharedRouterTable, CERT_DOMAIN_NAME,
};
use futures::FutureExt;
use quinn::{Endpoint, ServerConfig, TransportConfig, VarInt};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use test_log::test;

#[derive(Deserialize, Serialize)]
struct RawConnectReq {
    from: PeerId,
    to: PeerId,
    auth: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
struct RawConnectRes {
    result: Result<Vec<u8>, String>,
}

fn make_small_stream_receive_endpoint(
    bind_addr: std::net::SocketAddr,
    stream_window: u32,
) -> anyhow::Result<Endpoint> {
    let priv_key = PrivatePkcs8KeyDer::from(super::DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(super::DEFAULT_CLUSTER_CERT.to_vec());
    let mut server_config = ServerConfig::with_single_cert(vec![cert], priv_key.into())?;
    let mut transport = TransportConfig::default();
    let window = VarInt::from_u32(stream_window);
    transport.stream_receive_window(window);
    transport.receive_window(window);
    transport.max_concurrent_uni_streams(10_000_u32.into());
    transport.max_concurrent_bidi_streams(10_000_u32.into());
    transport.max_idle_timeout(Some(
        Duration::from_secs(5)
            .try_into()
            .expect("timeout should configure"),
    ));
    server_config.transport_config(Arc::new(transport));
    Endpoint::server(server_config, bind_addr).map_err(Into::into)
}

#[tokio::test]
async fn open_stream_fails_when_destination_service_receiver_is_closed() {
    let (mut node1, _addr1) = create_node(false, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let requester1 = node1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![]).await;
    let service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    requester1.connect(addr2.clone()).await.expect("connect attempt should be queued");
    drop(service2);

    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(
        service1.open_stream(addr2.peer_id(), vec![]).await.is_err(),
        "open_stream must fail if the destination service cannot receive the accepted stream"
    );
}

#[test(tokio::test)]
async fn open_stream_does_not_succeed_when_destination_service_queue_is_full() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1]).await;
    let _service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut held_streams = Vec::new();
    for _ in 0..10 {
        let stream = tokio::time::timeout(Duration::from_secs(2), service1.open_stream(addr2.peer_id(), vec![]))
            .await
            .expect("stream setup should not hang before the service queue is full")
            .expect("stream setup should succeed while destination service queue has capacity");
        held_streams.push(stream);
    }

    let result = tokio::time::timeout(Duration::from_secs(2), service1.open_stream(addr2.peer_id(), vec![])).await;

    assert!(
        !matches!(result, Ok(Ok(_))),
        "open_stream must not report success when the destination service queue is full and no task can consume the accepted pipe"
    );
}

#[tokio::test]
async fn open_stream_to_local_returns_error_not_panic() {
    let (mut node, addr) = create_node(false, 1, vec![]).await;
    let service = node.create_service(0.into());

    let result = std::panic::AssertUnwindSafe(service.open_stream(addr.peer_id(), vec![])).catch_unwind().await;

    assert!(matches!(result, Ok(Err(_))), "open_stream to local node must return Err, not panic");
}

#[tokio::test]
async fn relay_stream_must_not_forward_back_to_ingress_peer() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let service1 = node1.create_service(0.into());
    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;

    let (mut node1_to_node2, mut node2_to_node1) = (None, None);
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            tokio::select! {
                event = node1.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr2.peer_id() {
                            node1_to_node2 = Some(conn);
                        }
                    }
                }
                event = node2.recv() => {
                    if let Ok(P2pNetworkEvent::PeerConnected(conn, peer)) = event {
                        if peer == addr1.peer_id() {
                            node2_to_node1 = Some(conn);
                        }
                    }
                }
            }

            if node1_to_node2.is_some() && node2_to_node1.is_some() {
                break;
            }
        }
    })
    .await
    .expect("nodes should connect");

    let ghost = PeerId::from(99);
    let node1_to_node2 = node1_to_node2.expect("node1 should have conn to node2");
    let node2_to_node1 = node2_to_node1.expect("node2 should have conn to node1");

    let node2_advertised_routes = SharedRouterTable::new(addr2.peer_id());
    node2_advertised_routes.set_direct(ConnectionId::from(200), ghost, 1);
    node1
        .router
        .apply_sync(node1_to_node2, node2_advertised_routes.create_sync(&addr1.peer_id()));

    let node1_advertised_routes = SharedRouterTable::new(addr1.peer_id());
    node1_advertised_routes.set_direct(ConnectionId::from(100), ghost, 1);
    node2
        .router
        .apply_sync(node2_to_node1, node1_advertised_routes.create_sync(&addr2.peer_id()));

    let result = tokio::time::timeout(Duration::from_millis(500), service1.open_stream(ghost, b"loop".to_vec()))
        .await
        .expect("relay stream loop should be rejected promptly instead of recursively opening relayed streams");

    assert!(result.is_err(), "relay must reject forwarding a stream back to its ingress peer");
}

#[tokio::test]
async fn dropped_service_requester_must_not_continue_opening_streams() {
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

    let meta = b"stale-service-stream".to_vec();
    assert!(
        stale_requester.open_stream(addr2.peer_id(), meta.clone()).await.is_err(),
        "a requester cloned from a dropped P2pService must fail instead of opening streams"
    );

    let delivered = tokio::time::timeout(Duration::from_millis(500), service2.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Stream(_, received_meta, _))) if received_meta == meta),
        "a requester cloned from a dropped P2pService must not continue opening streams"
    );
}

#[tokio::test]
async fn stream_source_must_be_bound_to_authenticated_connection_peer() {
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

    let forged_source = PeerId::from(99);
    let meta = b"forged-stream-source".to_vec();
    let _opened_stream = conn
        .open_stream(0.into(), forged_source, addr2.peer_id(), meta.clone())
        .await
        .expect("forged stream setup should complete on the authenticated connection");

    let event = tokio::time::timeout(Duration::from_secs(1), service2.recv())
        .await
        .expect("destination service should receive or reject the injected stream")
        .expect("destination service channel should stay open");

    match event {
        P2pServiceEvent::Stream(source, received_meta, _stream) => {
            assert_eq!(
                (source, received_meta),
                (addr1.peer_id(), meta),
                "service must observe the authenticated connection peer, not the stream request's forged source"
            );
        }
        other => panic!("expected a stream event, got {other:?}"),
    }
}

#[tokio::test]
async fn relayed_stream_source_must_be_bound_to_previous_hop_peer() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let node1_ctx = node1.ctx.clone();
    let service1 = node1.create_service(0.into());
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1]).await;
    let _service2 = node2.create_service(0.into());
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let (mut node3, addr3) = create_node(false, 3, vec![addr2.clone()]).await;
    let mut service3 = node3.create_service(0.into());
    tokio::spawn(async move { while node3.recv().await.is_ok() {} });

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if matches!(service1.router().action(&addr3.peer_id()), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node1 should learn a relay route to node3 through node2");

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node1_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node1 should have a connection to relay node2");

    let forged_source = PeerId::from(99);
    let meta = b"forged-relay-stream-source".to_vec();
    let _opened_stream = conn
        .open_stream(0.into(), forged_source, addr3.peer_id(), meta.clone())
        .await
        .expect("forged relayed stream setup should complete");

    let event = tokio::time::timeout(Duration::from_secs(1), service3.recv())
        .await
        .expect("destination service should receive the relayed stream")
        .expect("destination service channel should stay open");

    match event {
        P2pServiceEvent::Stream(source, received_meta, _stream) => {
            assert_eq!(
                (source, received_meta),
                (addr2.peer_id(), meta),
                "final destination must observe the authenticated previous-hop relay peer, not a forged stream source"
            );
        }
        other => panic!("expected a stream event, got {other:?}"),
    }
}

#[tokio::test]
async fn inbound_out_of_range_stream_service_id_must_not_panic_accept_task() {
    let accept_task_panicked = Arc::new(AtomicBool::new(false));
    let previous_hook = std::panic::take_hook();
    let hook_flag = accept_task_panicked.clone();
    std::panic::set_hook(Box::new(move |info| {
        let message = info.to_string();
        if message.contains("src/ctx.rs") && message.contains("index out of bounds") {
            hook_flag.store(true, Ordering::SeqCst);
        }
    }));

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

    let invalid = conn.open_stream(P2pServiceId::from(256u16), addr1.peer_id(), addr2.peer_id(), b"bad-stream-service-id".to_vec()).await;

    assert!(
        invalid.is_err(),
        "out-of-range stream service ids must be rejected with an error instead of panicking in the accept task"
    );
    std::panic::set_hook(previous_hook);
    assert!(
        !accept_task_panicked.load(Ordering::SeqCst),
        "out-of-range stream service ids must not panic in the accept task"
    );

    let meta = b"valid-after-bad-stream-service-id".to_vec();
    let _valid_stream = conn
        .open_stream(0.into(), addr1.peer_id(), addr2.peer_id(), meta.clone())
        .await
        .expect("stream accept path must survive an inbound unknown out-of-range service id");

    let event = tokio::time::timeout(Duration::from_secs(1), service2.recv())
        .await
        .expect("valid follow-up stream should still be delivered")
        .expect("destination service channel should stay open");

    assert!(
        matches!(event, P2pServiceEvent::Stream(source, received_meta, _stream) if source == addr1.peer_id() && received_meta == meta),
        "inbound unknown out-of-range stream service id must be rejected without killing later valid stream delivery"
    );
}

#[tokio::test]
async fn open_stream_must_timeout_when_peer_withholds_connect_response() {
    let (mut node, addr) = create_node(false, 1, vec![]).await;
    let service = node.create_service(0.into());
    let requester = node.requester();
    tokio::spawn(async move { while node.recv().await.is_ok() {} });

    let raw_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind raw peer udp").local_addr().expect("should get raw peer addr");
    let priv_key = PrivatePkcs8KeyDer::from(super::DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(super::DEFAULT_CLUSTER_CERT.to_vec());
    let raw_peer = make_server_endpoint(raw_addr, priv_key, cert).expect("should create raw peer endpoint");
    let raw_peer_id = PeerId::from(2);
    let local_peer_id = addr.peer_id();
    let secure: SharedKeyHandshake = super::DEFAULT_SECURE_KEY.into();
    let (stream_req_tx, stream_req_rx) = tokio::sync::oneshot::channel();

    let raw_task = tokio::spawn({
        async move {
            let connecting = raw_peer.accept().await.expect("raw peer should accept incoming connect");
            let connection = connecting.await.expect("raw peer should complete QUIC connection");
            let (send, recv) = connection.accept_bi().await.expect("raw peer should accept main control stream");
            let mut main_stream = P2pQuicStream::new(recv, send);
            let request: RawConnectReq = wait_object::<_, _, 60000>(&mut main_stream)
                .await
                .expect("raw peer should receive connect request");
            secure
                .verify_request(request.auth, request.from, request.to, crate::now_ms())
                .expect("raw peer should verify connect request");
            write_object::<_, _, 60000>(
                &mut main_stream,
                &RawConnectRes {
                    result: Ok(secure.create_response(raw_peer_id, local_peer_id, crate::now_ms())),
                },
            )
            .await
            .expect("raw peer should write connect response");

            let (send, recv) = connection.accept_bi().await.expect("raw peer should accept stream connect");
            let mut stream = P2pQuicStream::new(recv, send);
            let request: StreamConnectReq = wait_object::<_, _, 60000>(&mut stream)
                .await
                .expect("raw peer should receive stream connect request");
            let _ = stream_req_tx.send(request);
            std::future::pending::<()>().await;
        }
    });

    requester
        .connect((raw_peer_id, raw_addr.into()).into())
        .await
        .expect("connect to raw peer should be queued");

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if matches!(service.router().action(&raw_peer_id), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("node should learn direct route to raw peer");

    let mut open = tokio::spawn({
        let service_requester = service.requester();
        async move { service_requester.open_stream(raw_peer_id, b"withheld-response".to_vec()).await }
    });

    let request = tokio::time::timeout(Duration::from_secs(1), stream_req_rx)
        .await
        .expect("raw peer should observe stream open")
        .expect("raw peer should send observed stream request");
    assert_eq!(request.dest, raw_peer_id);
    assert_eq!(request.source, local_peer_id);

    let result = tokio::time::timeout(Duration::from_millis(2500), &mut open).await;
    if result.is_err() {
        open.abort();
    }
    raw_task.abort();

    assert!(
        matches!(result, Ok(Ok(Err(_)))),
        "open_stream must return an error when the peer withholds StreamConnectRes instead of hanging past the setup deadline"
    );
}

#[tokio::test]
async fn open_stream_must_timeout_when_connect_request_write_stalls() {
    let (mut node, addr) = create_node(false, 1, vec![]).await;
    let service = node.create_service(0.into());
    let requester = node.requester();
    tokio::spawn(async move { while node.recv().await.is_ok() {} });

    let raw_addr = UdpSocket::bind("127.0.0.1:0")
        .expect("should bind raw peer udp")
        .local_addr()
        .expect("should get raw peer addr");
    let raw_peer = make_small_stream_receive_endpoint(raw_addr, 37)
        .expect("should create small-window raw peer endpoint");
    let raw_peer_id = PeerId::from(2);
    let local_peer_id = addr.peer_id();
    let secure: SharedKeyHandshake = super::DEFAULT_SECURE_KEY.into();
    let (stream_accepted_tx, stream_accepted_rx) = tokio::sync::oneshot::channel();

    let raw_task = tokio::spawn({
        async move {
            let connecting = raw_peer
                .accept()
                .await
                .expect("raw peer should accept incoming connect");
            let connection = connecting.await.expect("raw peer should complete QUIC connection");
            let (send, recv) = connection
                .accept_bi()
                .await
                .expect("raw peer should accept main control stream");
            let mut main_stream = P2pQuicStream::new(recv, send);
            let request: RawConnectReq = wait_object::<_, _, 60000>(&mut main_stream)
                .await
                .expect("raw peer should receive connect request");
            secure
                .verify_request(request.auth, request.from, request.to, crate::now_ms())
                .expect("raw peer should verify connect request");
            write_object::<_, _, 60000>(
                &mut main_stream,
                &RawConnectRes {
                    result: Ok(secure.create_response(raw_peer_id, local_peer_id, crate::now_ms())),
                },
            )
            .await
            .expect("raw peer should write connect response");

            let stream = connection
                .accept_bi()
                .await
                .expect("raw peer should accept stream connect");
            let _ = stream_accepted_tx.send(());
            let _held_stream = stream;
            std::future::pending::<()>().await;
        }
    });

    requester
        .connect((raw_peer_id, raw_addr.into()).into())
        .await
        .expect("connect to raw peer should be queued");

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if matches!(service.router().action(&raw_peer_id), Some(RouteAction::Next(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("node should learn direct route to raw peer");

    let mut open = tokio::spawn({
        let service_requester = service.requester();
        async move { service_requester.open_stream(raw_peer_id, vec![7; 59_000]).await }
    });

    tokio::time::timeout(Duration::from_secs(1), stream_accepted_rx)
        .await
        .expect("raw peer should accept stream open")
        .expect("raw peer should signal accepted stream");

    let result = tokio::time::timeout(Duration::from_millis(2500), &mut open).await;
    if result.is_err() {
        open.abort();
    }
    raw_task.abort();

    assert!(
        matches!(result, Ok(Ok(Err(_)))),
        "open_stream must return an error when writing StreamConnectReq stalls behind peer flow control"
    );
}

#[tokio::test]
async fn relay_must_not_deliver_downstream_stream_after_upstream_setup_closes() {
    let raw_addr = UdpSocket::bind("127.0.0.1:0")
        .expect("should bind raw node1 udp")
        .local_addr()
        .expect("should get raw node1 addr");
    let priv_key = PrivatePkcs8KeyDer::from(super::DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(super::DEFAULT_CLUSTER_CERT.to_vec());
    let raw_node1 = make_server_endpoint(raw_addr, priv_key, cert).expect("should create raw node1 endpoint");
    let raw_node1_id = PeerId::from(1);

    let (mut node2, addr2) = create_node(true, 2, vec![]).await;
    let requester2 = node2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    let (mut node3, addr3) = create_node(false, 3, vec![]).await;
    let mut service3 = node3.create_service(0.into());
    tokio::spawn(async move { while node3.recv().await.is_ok() {} });

    requester2.connect(addr3.clone()).await.expect("node2 should connect to node3");
    tokio::time::sleep(Duration::from_millis(500)).await;

    let connection = raw_node1
        .connect(**addr2.network_address(), CERT_DOMAIN_NAME)
        .expect("raw node1 should start QUIC connect")
        .await
        .expect("raw node1 should connect to node2");
    let (send, recv) = connection.open_bi().await.expect("raw node1 should open main control stream");
    let mut main_stream = P2pQuicStream::new(recv, send);
    let secure: SharedKeyHandshake = super::DEFAULT_SECURE_KEY.into();
    write_object::<_, _, 60000>(
        &mut main_stream,
        &RawConnectReq {
            from: raw_node1_id,
            to: addr2.peer_id(),
            auth: secure.create_request(raw_node1_id, addr2.peer_id(), crate::now_ms()),
        },
    )
    .await
    .expect("raw node1 should send authenticated connect request");
    let response: RawConnectRes = wait_object::<_, _, 60000>(&mut main_stream)
        .await
        .expect("raw node1 should receive connect response");
    secure
        .verify_response(response.result.expect("node2 should accept raw node1"), addr2.peer_id(), raw_node1_id, crate::now_ms())
        .expect("raw node1 should verify connect response");

    let meta = b"orphan-relay-stream".to_vec();
    let (mut send, mut recv) = connection.open_bi().await.expect("raw node1 should open stream setup");
    write_object::<_, _, 60000>(
        &mut send,
        &StreamConnectReq {
            source: raw_node1_id,
            dest: addr3.peer_id(),
            service: 0.into(),
            meta: meta.clone(),
        },
    )
    .await
    .expect("raw node1 should send stream connect request");
    send.finish().expect("raw node1 should finish the upstream send side after setup request");
    recv.stop(VarInt::from_u32(0)).expect("raw node1 should stop the upstream receive side before setup ack");

    let delivered = tokio::time::timeout(Duration::from_millis(500), service3.recv()).await;

    assert!(
        !matches!(delivered, Ok(Some(P2pServiceEvent::Stream(_, received_meta, _))) if received_meta == meta),
        "relay must not deliver a downstream stream after the upstream stream was closed before setup ack"
    );
}

#[tokio::test]
async fn idle_inbound_stream_connects_must_be_admission_bounded() {
    const ACCEPTABLE_IDLE_STREAMS: usize = 16;
    const ATTEMPTED_IDLE_STREAMS: usize = ACCEPTABLE_IDLE_STREAMS + 1;

    let (mut node, addr) = create_node(false, 2, vec![]).await;
    let _service = node.create_service(0.into());
    tokio::spawn(async move { while node.recv().await.is_ok() {} });

    let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should get client addr");
    let priv_key = PrivatePkcs8KeyDer::from(super::DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(super::DEFAULT_CLUSTER_CERT.to_vec());
    let client = make_server_endpoint(client_addr, priv_key, cert).expect("should create raw client endpoint");

    let connection = client
        .connect(**addr.network_address(), CERT_DOMAIN_NAME)
        .expect("raw client should start QUIC connect")
        .await
        .expect("raw client should connect");

    let (send, recv) = connection.open_bi().await.expect("raw client should open main control stream");
    let mut main_stream = P2pQuicStream::new(recv, send);
    let attacker = PeerId::from(99);
    let secure: SharedKeyHandshake = super::DEFAULT_SECURE_KEY.into();
    write_object::<_, _, 60000>(
        &mut main_stream,
        &RawConnectReq {
            from: attacker,
            to: addr.peer_id(),
            auth: secure.create_request(attacker, addr.peer_id(), crate::now_ms()),
        },
    )
    .await
    .expect("raw client should send authenticated connect request");
    let response: RawConnectRes = wait_object::<_, _, 60000>(&mut main_stream)
        .await
        .expect("raw client should receive connect response");
    secure
        .verify_response(response.result.expect("connect response should be accepted"), addr.peer_id(), attacker, crate::now_ms())
        .expect("raw client should verify connect response");

    let mut idle_streams = Vec::new();
    for _ in 0..ACCEPTABLE_IDLE_STREAMS {
        let stream = tokio::time::timeout(Duration::from_millis(500), connection.open_bi())
            .await
            .expect("idle stream open within the admission cap should not hang")
            .expect("idle stream open within the admission cap should be transport-accepted");
        idle_streams.push(stream);
    }

    let rejected_or_timed_out = matches!(
        tokio::time::timeout(Duration::from_millis(500), connection.open_bi()).await,
        Ok(Err(_)) | Err(_)
    );

    assert_eq!(idle_streams.len(), ACCEPTABLE_IDLE_STREAMS, "the stream cap should allow exactly {ACCEPTABLE_IDLE_STREAMS} idle stream-connect attempts after the main control stream");
    assert!(rejected_or_timed_out, "the {ATTEMPTED_IDLE_STREAMS}th idle inbound stream-connect attempt must be rejected or time out once the stream cap is reached");
}

#[tokio::test]
async fn unauthenticated_inbound_connections_must_be_admission_bounded() {
    const ACCEPTABLE_PENDING_CONNECTIONS: usize = 16;
    const ATTEMPTED_PENDING_CONNECTIONS: usize = ACCEPTABLE_PENDING_CONNECTIONS + 1;

    let (mut node, addr) = create_node(false, 2, vec![]).await;
    let _service = node.create_service(0.into());
    tokio::spawn(async move { while node.recv().await.is_ok() {} });

    let mut accepted_connections = Vec::new();
    let mut rejected_or_timed_out = 0;
    for _ in 0..ATTEMPTED_PENDING_CONNECTIONS {
        let client_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind client udp")
            .local_addr()
            .expect("should get client addr");
        let priv_key = PrivatePkcs8KeyDer::from(super::DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(super::DEFAULT_CLUSTER_CERT.to_vec());
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("should create raw client endpoint");
        match tokio::time::timeout(
            Duration::from_secs(1),
            client.connect(**addr.network_address(), CERT_DOMAIN_NAME)
                .expect("raw client should start QUIC connect"),
        )
        .await
        {
            Ok(Ok(connection)) => accepted_connections.push((client, connection)),
            Ok(Err(_)) | Err(_) => rejected_or_timed_out += 1,
        }
    }

    assert!(
        accepted_connections.len() <= ACCEPTABLE_PENDING_CONNECTIONS,
        "unauthenticated inbound connections must be capped or timed out before more than {ACCEPTABLE_PENDING_CONNECTIONS} can accumulate"
    );
    assert!(
        rejected_or_timed_out > 0,
        "at least one unauthenticated inbound connection attempt must be rejected or time out once the pending cap is reached"
    );
}
