use std::time::Duration;

use test_log::test;
use tokio::time::timeout;

use crate::{
    msg::PeerMessage,
    pubsub_service::{
        encode_heartbeat_for_test, encode_publish_for_test, encode_publish_rpc_answer_for_test, encode_publish_rpc_for_test, encode_subscriber_joined_for_test, PeerSrc, PublisherEvent,
        PubsubChannelId, PubsubService, RpcId, SubscriberEvent,
    },
};

use super::create_node;

#[test(tokio::test)]
async fn pubsub_local_single_pair_pub_first() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service1_requester.subscriber(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_local_single_pair_sub_first() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber = service1_requester.subscriber(channel_id).await;
    let mut publisher = service1_requester.publisher(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_local_multi_subs() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber1 = service1_requester.subscriber(channel_id).await;
    let mut subscriber2 = service1_requester.subscriber(channel_id).await;
    let mut publisher = service1_requester.publisher(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber1.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, subscriber2.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );
    assert!(timeout(ttl, publisher.recv()).await.is_err()); // it should timeout because we don't fire join 2 times

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber1.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );
    assert_eq!(
        timeout(ttl, subscriber2.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber1.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );

    subscriber2.requester().feedback(vec![3, 4, 5]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![3, 4, 5])
    );
}

#[test(tokio::test)]
async fn pubsub_local_multi_pubs() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher1 = service1_requester.publisher(channel_id).await;
    let mut publisher2 = service1_requester.publisher(channel_id).await;
    let mut subscriber = service1_requester.subscriber(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert!(timeout(ttl, subscriber.recv()).await.is_err()); // it should timeout because we don't fire join 2 times
    assert_eq!(
        timeout(ttl, publisher1.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher2.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );

    publisher1.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    publisher2.requester().publish(vec![1, 2, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 4])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher1.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
    assert_eq!(
        timeout(ttl, publisher2.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_remote_single_pair_pub_first() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let ttl = Duration::from_secs(1);

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service2_requester.subscriber(channel_id).await;

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_remote_single_pair_sub_first() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    let ttl = Duration::from_secs(1);
    tokio::time::sleep(Duration::from_secs(1)).await;

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber = service1_requester.subscriber(channel_id).await;
    let mut publisher = service2_requester.publisher(channel_id).await;

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_remote_multi_subs() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    let ttl = Duration::from_secs(1);
    tokio::time::sleep(Duration::from_secs(1)).await;

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber1 = service1_requester.subscriber(channel_id).await;
    let mut subscriber2 = service2_requester.subscriber(channel_id).await;
    let mut publisher = service1_requester.publisher(channel_id).await;

    assert_eq!(
        timeout(ttl, subscriber1.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, subscriber2.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber1.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );
    assert_eq!(
        timeout(ttl, subscriber2.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber1.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );

    subscriber2.requester().feedback(vec![3, 4, 5]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![3, 4, 5])
    );
}

#[test(tokio::test)]
async fn pubsub_remote_multi_pubs() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    let ttl = Duration::from_secs(1);
    tokio::time::sleep(Duration::from_secs(1)).await;

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher1 = service1_requester.publisher(channel_id).await;
    let mut publisher2 = service2_requester.publisher(channel_id).await;
    let mut subscriber = service1_requester.subscriber(channel_id).await;

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher1.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher2.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );

    publisher1.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    publisher2.requester().publish(vec![1, 2, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 4])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher1.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
    assert_eq!(
        timeout(ttl, publisher2.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_remote_heartbeat_restore() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service2_requester.subscriber(channel_id).await;

    let ttl = Duration::from_secs(1);
    // now it will error because it created before nodes join to network
    assert!(timeout(ttl, subscriber.recv()).await.is_err());
    assert!(timeout(ttl, publisher.recv()).await.is_err());

    // now we wait 5 seconds
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    publisher.requester().publish(vec![1, 2, 3]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::Publish(vec![1, 2, 3])
    );

    subscriber.requester().feedback(vec![2, 3, 4]).await.expect("should ok");
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::Feedback(vec![2, 3, 4])
    );
}

#[test(tokio::test)]
async fn pubsub_publish_rpc_local() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service1_requester.subscriber(channel_id).await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );

    tokio::spawn(async move {
        let rpc_event = timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv");
        if let SubscriberEvent::PublishRpc(data, rpc_id, method, source) = rpc_event {
            assert_eq!(data, vec![1, 2, 3]);
            assert_eq!(method, "ping");
            assert_eq!(source, PeerSrc::Local);
            subscriber.requester().answer_publish_rpc(rpc_id, source, vec![2, 3, 4]).await.expect("should answer");
        } else {
            panic!("must received SubscriberEvent::PublishRpc");
        }
    });

    let res = publisher.requester().publish_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.expect("should ok");
    assert_eq!(res, vec![2, 3, 4]);
}

#[test(tokio::test)]
async fn pubsub_feedback_rpc_local() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service1_requester.subscriber(channel_id).await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Local)
    );

    tokio::spawn(async move {
        let rpc_event = timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv");
        if let PublisherEvent::FeedbackRpc(data, rpc_id, method, source) = rpc_event {
            assert_eq!(data, vec![1, 2, 3]);
            assert_eq!(method, "ping");
            assert_eq!(source, PeerSrc::Local);
            publisher.requester().answer_feedback_rpc(rpc_id, source, vec![2, 3, 4]).await.expect("should answer");
        } else {
            panic!("must received SubscriberEvent::PublishRpc");
        }
    });

    let res = subscriber.requester().feedback_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.expect("should ok");
    assert_eq!(res, vec![2, 3, 4]);
}

#[test(tokio::test)]
async fn pubsub_publish_rpc_remote() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    tokio::time::sleep(Duration::from_secs(1)).await;

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service2_requester.subscriber(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    tokio::spawn(async move {
        let rpc_event = timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv");
        if let SubscriberEvent::PublishRpc(data, rpc_id, method, source) = rpc_event {
            assert_eq!(data, vec![1, 2, 3]);
            assert_eq!(method, "ping");
            assert_eq!(source, PeerSrc::Remote(addr1.peer_id()));
            subscriber.requester().answer_publish_rpc(rpc_id, source, vec![2, 3, 4]).await.expect("should answer");
        } else {
            panic!("must received SubscriberEvent::PublishRpc");
        }
    });

    let res = publisher.requester().publish_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.expect("should ok");
    assert_eq!(res, vec![2, 3, 4]);
}

#[test(tokio::test)]
async fn pubsub_feedback_rpc_remote() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    tokio::time::sleep(Duration::from_secs(1)).await;

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service2_requester.subscriber(channel_id).await;

    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("should not timeout").expect("should recv"),
        SubscriberEvent::PeerJoined(PeerSrc::Remote(addr1.peer_id()))
    );
    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    tokio::spawn(async move {
        let rpc_event = timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv");
        if let PublisherEvent::FeedbackRpc(data, rpc_id, method, source) = rpc_event {
            assert_eq!(data, vec![1, 2, 3]);
            assert_eq!(method, "ping");
            assert_eq!(source, PeerSrc::Remote(addr2.peer_id()));
            publisher.requester().answer_feedback_rpc(rpc_id, source, vec![2, 3, 4]).await.expect("should answer");
        } else {
            panic!("must received SubscriberEvent::PublishRpc");
        }
    });

    let res = subscriber.requester().feedback_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.expect("should ok");
    assert_eq!(res, vec![2, 3, 4]);
}

#[test(tokio::test)]
async fn pubsub_publish_rpc_no_destination() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let publisher = service1_requester.publisher(channel_id).await;
    assert!(publisher.requester().publish_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.is_err());
}

#[test(tokio::test)]
async fn pubsub_publisher_after_service_drop_must_not_be_dead_on_arrival() {
    let (mut node, _addr) = create_node(true, 1, vec![]).await;
    let service = PubsubService::new(node.create_service(0.into()));
    let requester = service.requester();
    drop(service);
    tokio::spawn(async move { while node.recv().await.is_ok() {} });

    let mut publisher = requester.publisher(PubsubChannelId::from(1000)).await;
    let result = timeout(Duration::from_millis(50), publisher.recv()).await;

    assert!(
        result.is_err(),
        "creating a publisher after PubsubService drop must fail instead of returning an immediately closed handle"
    );
}

#[test(tokio::test)]
async fn dropped_publisher_requester_must_not_continue_publishing() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let channel_id: PubsubChannelId = 1000.into();
    let publisher = service1_requester.publisher(channel_id).await;
    let stale_requester = publisher.requester().clone();
    let mut subscriber = service1_requester.subscriber(channel_id).await;
    let ttl = Duration::from_secs(1);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("subscriber should observe initial local publisher").expect("subscriber channel should stay open"),
        SubscriberEvent::PeerJoined(PeerSrc::Local)
    );

    drop(publisher);

    assert_eq!(
        timeout(ttl, subscriber.recv()).await.expect("subscriber should observe dropped local publisher").expect("subscriber channel should stay open"),
        SubscriberEvent::PeerLeaved(PeerSrc::Local)
    );

    stale_requester.publish(b"stale-publish".to_vec()).await.expect("stale requester send should not panic");

    let delivered = timeout(Duration::from_millis(500), subscriber.recv()).await;
    assert!(
        !matches!(delivered, Ok(Ok(SubscriberEvent::Publish(data))) if data == b"stale-publish".to_vec()),
        "a requester cloned from a dropped Publisher must not continue publishing on that channel"
    );
}

#[test(tokio::test)]
async fn pubsub_publish_rpc_answer_must_be_bound_to_expected_responder() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, _addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let mut service2 = PubsubService::new(node2.create_service(0.into()));
    let service2_requester = service2.requester();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });
    tokio::spawn(async move { service2.run_loop().await });

    let (mut node3, addr3) = create_node(false, 3, vec![addr1.clone()]).await;
    let node3_ctx = node3.ctx.clone();
    tokio::spawn(async move { while node3.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let ttl = Duration::from_secs(1);
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;
    let mut subscriber = service2_requester.subscriber(channel_id).await;

    assert_eq!(
        timeout(ttl, publisher.recv()).await.expect("should not timeout").expect("should recv"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(2.into()))
    );

    let publisher_requester = publisher.requester().clone();
    let publish_task = tokio::spawn(async move { publisher_requester.publish_rpc("ping", vec![1, 2, 3], Duration::from_secs(2)).await });

    let rpc_id = tokio::time::timeout(ttl, async {
        loop {
            if let SubscriberEvent::PublishRpc(_, rpc_id, _, PeerSrc::Remote(from)) = subscriber.recv().await.expect("should recv") {
                assert_eq!(from, addr1.peer_id());
                return rpc_id;
            }
        }
    })
    .await
    .expect("subscriber should receive publish RPC request");

    let conn = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(conn) = node3_ctx.conns().into_iter().next() {
                return conn;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("node3 should connect to node1");

    let fake_answer = b"forged-rpc-answer".to_vec();
    let payload = encode_publish_rpc_answer_for_test(fake_answer.clone(), rpc_id);
    conn.try_send(PeerMessage::Unicast(addr3.peer_id(), addr1.peer_id(), 0.into(), payload))
        .expect("attacker should be able to inject a pubsub RPC answer");

    if let Ok(joined) = timeout(Duration::from_millis(500), publish_task).await {
        let result = joined.expect("publish task should not panic");
        assert!(
            !matches!(result, Ok(data) if data == fake_answer),
            "publish_rpc must not complete from an answer sent by an unrelated peer"
        );
    }
}

#[test(tokio::test)]
async fn pubsub_heartbeat_must_remove_stale_remote_subscriber() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let channel_id: PubsubChannelId = 1000.into();
    let mut publisher = service1_requester.publisher(channel_id).await;

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

    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_subscriber_joined_for_test(channel_id)))
        .expect("subscriber joined should be injected");

    assert_eq!(
        timeout(Duration::from_secs(1), publisher.recv())
            .await
            .expect("publisher should receive join")
            .expect("publisher channel should stay open"),
        PublisherEvent::PeerJoined(PeerSrc::Remote(addr2.peer_id()))
    );

    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_heartbeat_for_test(channel_id, false, false)))
        .expect("heartbeat should be injected");

    assert_eq!(
        timeout(Duration::from_millis(500), publisher.recv())
            .await
            .expect("publisher should receive leave from heartbeat")
            .expect("publisher channel should stay open"),
        PublisherEvent::PeerLeaved(PeerSrc::Remote(addr2.peer_id()))
    );
}

#[test(tokio::test)]
async fn pubsub_feedback_rpc_no_destination() {
    let (mut node1, _addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    // we create publisher first
    let channel_id: PubsubChannelId = 1000.into();
    let subscriber = service1_requester.subscriber(channel_id).await;
    assert!(subscriber.requester().feedback_rpc("ping", vec![1, 2, 3], Duration::from_secs(1)).await.is_err());
}

#[test(tokio::test)]
async fn pubsub_publish_must_require_remote_publisher_membership() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber = service1_requester.subscriber(channel_id).await;

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

    let injected = b"publish-without-membership".to_vec();
    conn.try_send(PeerMessage::Unicast(addr2.peer_id(), addr1.peer_id(), 0.into(), encode_publish_for_test(channel_id, injected.clone())))
        .expect("attacker should be able to inject a pubsub publish frame");

    let delivered = timeout(Duration::from_millis(500), subscriber.recv()).await;

    assert!(
        !matches!(delivered, Ok(Ok(SubscriberEvent::Publish(data))) if data == injected),
        "pubsub Publish must not be delivered from a peer that has not joined the channel as a publisher"
    );
}

#[test(tokio::test)]
async fn pubsub_publish_rpc_must_require_remote_publisher_membership() {
    let (mut node1, addr1) = create_node(true, 1, vec![]).await;
    let mut service1 = PubsubService::new(node1.create_service(0.into()));
    let service1_requester = service1.requester();
    tokio::spawn(async move { while node1.recv().await.is_ok() {} });
    tokio::spawn(async move { service1.run_loop().await });

    let (mut node2, addr2) = create_node(false, 2, vec![addr1.clone()]).await;
    let node2_ctx = node2.ctx.clone();
    tokio::spawn(async move { while node2.recv().await.is_ok() {} });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let channel_id: PubsubChannelId = 1000.into();
    let mut subscriber = service1_requester.subscriber(channel_id).await;

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

    let injected = b"publish-rpc-without-membership".to_vec();
    let method = "side-effect".to_string();
    conn.try_send(PeerMessage::Unicast(
        addr2.peer_id(),
        addr1.peer_id(),
        0.into(),
        encode_publish_rpc_for_test(channel_id, injected.clone(), RpcId::rand(), method.clone()),
    ))
    .expect("attacker should be able to inject a pubsub publish RPC frame");

    let delivered = timeout(Duration::from_millis(500), subscriber.recv()).await;

    assert!(
        !matches!(delivered, Ok(Ok(SubscriberEvent::PublishRpc(data, _rpc_id, event_method, PeerSrc::Remote(from)))) if data == injected && event_method == method && from == addr2.peer_id()),
        "pubsub PublishRpc must not invoke subscriber RPC handlers from a peer that has not joined the channel as a publisher"
    );
}
