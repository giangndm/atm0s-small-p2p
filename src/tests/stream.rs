use std::time::Duration;

use super::create_node;
use crate::{P2pServiceEvent, PeerId};
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
