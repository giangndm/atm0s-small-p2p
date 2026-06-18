use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use super::create_node;
use crate::{msg::P2pServiceId, router::RouteAction, P2pServiceEvent, PeerId};
use futures::FutureExt;
use test_log::test;

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
    let _opened = stale_requester
        .open_stream(addr2.peer_id(), meta.clone())
        .await
        .expect("stale requester stream open should not panic");

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
            assert_ne!(
                (source, received_meta),
                (forged_source, meta),
                "service must not observe a stream sender id that was forged inside the stream request"
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
