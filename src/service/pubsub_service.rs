//! Simple PubsubService with multi-publishers, multi-subscribers style
//!
//! We trying to implement a pubsub service with only Unicast and Broadcast, without any database.
//! Each time new producer is created or destroyed, it will broadcast to all other nodes, same with new subscriber.
//!
//! For avoiding channel state out-of-sync, we add simple heartbeat, each some seconds each node will broadcast a list of active channel with flag publish and subscribe.

use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use anyhow::anyhow;
use derive_more::derive::{Display, From};
use publisher::{PublisherHandleId, PublisherLocalId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use subscriber::{SubscriberHandleId, SubscriberLocalId};
use thiserror::Error;
use tokio::{
    select,
    sync::{
        mpsc::{channel, error::TrySendError, Receiver, Sender},
        oneshot,
    },
    time::Interval,
};

use crate::{ErrorExt, PeerId};

use super::{P2pService, P2pServiceEvent};

mod publisher;
mod subscriber;

pub use publisher::{Publisher, PublisherEvent, PublisherEventOb, PublisherRequester};
pub use subscriber::{Subscriber, SubscriberEvent, SubscriberEventOb, SubscriberRequester};

const HEARTBEAT_INTERVAL_MS: u64 = 5_000;
const RPC_TICK_INTERVAL_MS: u64 = 1_000;
pub(crate) const PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE: usize = 1024;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PeerSrc {
    Local,
    Remote(PeerId),
}

#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct RpcId(u64);

impl RpcId {
    pub fn rand() -> Self {
        RpcId(rand::random())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelHeartbeat {
    channel: PubsubChannelId,
    publish: bool,
    publish_generation: u64,
    subscribe: bool,
    subscribe_generation: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PubsubRpcError {
    #[error("Timeout")]
    Timeout,
    #[error("NoDestination")]
    NoDestination,
}

struct PublishRpcReq {
    started_at: Instant,
    timeout: Duration,
    expected_responders: HashSet<PeerSrc>,
    tx: Option<oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>>,
}

struct FeedbackRpcReq {
    started_at: Instant,
    timeout: Duration,
    expected_responders: HashSet<PeerSrc>,
    tx: Option<oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum PubsubMessage {
    PublisherJoined(PubsubChannelId, u64),
    PublisherLeaved(PubsubChannelId, u64),
    SubscriberJoined(PubsubChannelId, u64),
    SubscriberLeaved(PubsubChannelId, u64),
    Heartbeat(Vec<ChannelHeartbeat>),
    GuestPublish(PubsubChannelId, Vec<u8>),
    GuestPublishRpc(PubsubChannelId, Vec<u8>, RpcId, String),
    Publish(PubsubChannelId, Vec<u8>),
    PublishRpc(PubsubChannelId, Vec<u8>, RpcId, String),
    PublishRpcAnswer(Vec<u8>, RpcId),
    GuestFeedback(PubsubChannelId, Vec<u8>),
    GuestFeedbackRpc(PubsubChannelId, Vec<u8>, RpcId, String),
    Feedback(PubsubChannelId, Vec<u8>),
    FeedbackRpc(PubsubChannelId, Vec<u8>, RpcId, String),
    FeedbackRpcAnswer(Vec<u8>, RpcId),
}

#[cfg(test)]
pub(crate) fn encode_publish_rpc_answer_for_test(data: Vec<u8>, rpc_id: RpcId) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::PublishRpcAnswer(data, rpc_id)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_feedback_rpc_answer_for_test(data: Vec<u8>, rpc_id: RpcId) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::FeedbackRpcAnswer(data, rpc_id)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_subscriber_joined_for_test(channel: PubsubChannelId) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::SubscriberJoined(channel, 1)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_publisher_joined_for_test(channel: PubsubChannelId) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::PublisherJoined(channel, 1)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_publish_for_test(channel: PubsubChannelId, data: Vec<u8>) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::Publish(channel, data)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_publish_rpc_for_test(channel: PubsubChannelId, data: Vec<u8>, rpc_id: RpcId, method: String) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::PublishRpc(channel, data, rpc_id, method)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_heartbeat_for_test(channel: PubsubChannelId, publish: bool, subscribe: bool) -> Vec<u8> {
    encode_heartbeat_for_test_with_generation(channel, publish, 1, subscribe, 1)
}

#[cfg(test)]
fn encode_heartbeat_for_test_with_generation(channel: PubsubChannelId, publish: bool, publish_generation: u64, subscribe: bool, subscribe_generation: u64) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::Heartbeat(vec![ChannelHeartbeat {
        channel,
        publish,
        publish_generation,
        subscribe,
        subscribe_generation,
    }]))
    .expect("test message should serialize")
}

#[cfg(test)]
fn encode_publisher_leaved_for_test(channel: PubsubChannelId, generation: u64) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::PublisherLeaved(channel, generation)).expect("test message should serialize")
}

#[cfg(test)]
fn encode_subscriber_leaved_for_test(channel: PubsubChannelId, generation: u64) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::SubscriberLeaved(channel, generation)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_empty_heartbeat_for_test() -> Vec<u8> {
    bincode::serialize(&PubsubMessage::Heartbeat(vec![])).expect("test message should serialize")
}

enum InternalMsg {
    PublisherCreated(PublisherHandleId, PubsubChannelId, Sender<PublisherEvent>),
    PublisherDestroyed(PublisherHandleId, PubsubChannelId),
    SubscriberCreated(SubscriberHandleId, PubsubChannelId, Sender<SubscriberEvent>),
    SubscriberDestroyed(SubscriberHandleId, PubsubChannelId),
    GuestPublish(PubsubChannelId, Vec<u8>),
    GuestPublishRpc(PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    Publish(PublisherHandleId, PubsubChannelId, Vec<u8>),
    PublishRpc(PublisherHandleId, PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    PublishRpcAnswer(RpcId, PeerSrc, Vec<u8>),
    GuestFeedback(PubsubChannelId, Vec<u8>),
    GuestFeedbackRpc(PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    Feedback(SubscriberHandleId, PubsubChannelId, Vec<u8>),
    FeedbackRpc(SubscriberHandleId, PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    FeedbackRpcAnswer(RpcId, PeerSrc, Vec<u8>),
}

#[derive(Debug, From, Display, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PubsubChannelId(u64);

#[derive(Debug, Clone)]
pub struct PubsubServiceRequester {
    internal_tx: Sender<InternalMsg>,
}

#[derive(Debug, Default)]
struct RemoteRoleState {
    generation: u64,
    active: bool,
}

#[derive(Debug, Default)]
struct PubsubChannelState {
    remote_publishers: HashMap<PeerId, RemoteRoleState>,
    remote_subscribers: HashMap<PeerId, RemoteRoleState>,
    local_publishers: HashMap<PublisherHandleId, Sender<PublisherEvent>>,
    local_subscribers: HashMap<SubscriberHandleId, Sender<SubscriberEvent>>,
    local_publish_generation: u64,
    local_subscribe_generation: u64,
}

impl PubsubChannelState {
    fn active_remote_publishers(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.remote_publishers.iter().filter_map(|(peer, state)| state.active.then_some(*peer))
    }

    fn active_remote_subscribers(&self) -> impl Iterator<Item = PeerId> + '_ {
        self.remote_subscribers.iter().filter_map(|(peer, state)| state.active.then_some(*peer))
    }

    fn has_remote_publisher(&self, peer: PeerId) -> bool {
        self.remote_publishers.get(&peer).is_some_and(|state| state.active)
    }

    fn has_remote_subscriber(&self, peer: PeerId) -> bool {
        self.remote_subscribers.get(&peer).is_some_and(|state| state.active)
    }

    fn active_remote_publishers_count(&self) -> usize {
        self.active_remote_publishers().count()
    }

    fn has_active_remote_publishers(&self) -> bool {
        self.remote_publishers.values().any(|state| state.active)
    }

    fn has_active_remote_subscribers(&self) -> bool {
        self.remote_subscribers.values().any(|state| state.active)
    }

    fn apply_remote_publisher(&mut self, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        Self::apply_remote_role(&mut self.remote_publishers, peer, generation, active)
    }

    fn apply_remote_subscriber(&mut self, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        Self::apply_remote_role(&mut self.remote_subscribers, peer, generation, active)
    }

    fn apply_remote_role(map: &mut HashMap<PeerId, RemoteRoleState>, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        match map.get_mut(&peer) {
            Some(state) if generation <= state.generation => None,
            Some(state) => {
                let was_active = state.active;
                state.generation = generation;
                state.active = active;
                Some((was_active, active))
            }
            None => {
                map.insert(peer, RemoteRoleState { generation, active });
                Some((false, active))
            }
        }
    }
}

pub struct PubsubService {
    service: P2pService,
    internal_tx: Sender<InternalMsg>,
    internal_rx: Receiver<InternalMsg>,
    channels: HashMap<PubsubChannelId, PubsubChannelState>,
    publish_rpc_reqs: HashMap<RpcId, PublishRpcReq>,
    feedback_rpc_reqs: HashMap<RpcId, FeedbackRpcReq>,
    heartbeat_tick: Interval,
    rpc_tick: Interval,
}

impl PubsubService {
    pub fn new(service: P2pService) -> Self {
        let (internal_tx, internal_rx) = channel(PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        Self {
            service,
            internal_rx,
            internal_tx,
            channels: HashMap::new(),
            publish_rpc_reqs: HashMap::new(),
            feedback_rpc_reqs: HashMap::new(),
            heartbeat_tick: tokio::time::interval(Duration::from_millis(HEARTBEAT_INTERVAL_MS)),
            rpc_tick: tokio::time::interval(Duration::from_millis(RPC_TICK_INTERVAL_MS)),
        }
    }

    pub fn requester(&self) -> PubsubServiceRequester {
        PubsubServiceRequester {
            internal_tx: self.internal_tx.clone(),
        }
    }

    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                _ = self.heartbeat_tick.tick() => {
                    self.on_heartbeat_tick().await?;
                },
                _ = self.rpc_tick.tick() => {
                    self.on_rpc_tick().await?;
                },
                e = self.service.recv() => {
                    self.on_service(e.ok_or_else(|| anyhow!("service channel failed"))?).await?;
                },
                e = self.internal_rx.recv() => {
                    self.on_internal(e.ok_or_else(|| anyhow!("internal channel crash"))?).await?;
                },
            }
        }
    }

    async fn on_heartbeat_tick(&mut self) -> anyhow::Result<()> {
        let mut heartbeat = vec![];
        for (channel, state) in self.channels.iter() {
            heartbeat.push(ChannelHeartbeat {
                channel: *channel,
                publish: !state.local_publishers.is_empty(),
                publish_generation: state.local_publish_generation,
                subscribe: !state.local_subscribers.is_empty(),
                subscribe_generation: state.local_subscribe_generation,
            });
        }
        self.broadcast(&PubsubMessage::Heartbeat(heartbeat)).await;
        Ok(())
    }

    async fn on_rpc_tick(&mut self) -> anyhow::Result<()> {
        for (_req_id, req) in self.publish_rpc_reqs.iter_mut() {
            if req.started_at.elapsed() >= req.timeout {
                let _ = req.tx.take().expect("should have tx").send(Err(PubsubRpcError::Timeout));
            }
        }

        for (_req_id, req) in self.feedback_rpc_reqs.iter_mut() {
            if req.started_at.elapsed() >= req.timeout {
                let _ = req.tx.take().expect("should have tx").send(Err(PubsubRpcError::Timeout));
            }
        }

        self.publish_rpc_reqs.retain(|_req_id, req| req.tx.is_some());
        self.feedback_rpc_reqs.retain(|_req_id, req| req.tx.is_some());

        Ok(())
    }

    async fn on_service(&mut self, event: P2pServiceEvent) -> anyhow::Result<()> {
        match event {
            P2pServiceEvent::Unicast(from_peer, vec) | P2pServiceEvent::Broadcast(from_peer, vec) => {
                if let Ok(msg) = bincode::deserialize::<PubsubMessage>(&vec) {
                    match msg {
                        PubsubMessage::PublisherJoined(channel, generation) => {
                            let mut reply = None;
                            if let Some(state) = self.channels.get_mut(&channel) {
                                if matches!(state.apply_remote_publisher(from_peer, generation, true), Some((false, true))) {
                                    log::info!("[PubsubService] remote peer {from_peer} joined to {channel} as publisher");
                                    // we have new remote publisher then we fire event to local
                                    for sub_tx in state.local_subscribers.values() {
                                        Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerJoined(PeerSrc::Remote(from_peer)));
                                    }
                                    // we also send subscribe state it remote, as publisher it only care about wherever this node is a subscriber
                                    if !state.local_subscribers.is_empty() {
                                        reply = Some(PubsubMessage::SubscriberJoined(channel, state.local_subscribe_generation));
                                    }
                                }
                            }
                            if let Some(reply) = reply {
                                self.send_to(from_peer, &reply).await;
                            }
                        }
                        PubsubMessage::PublisherLeaved(channel, generation) => {
                            if let Some(state) = self.channels.get_mut(&channel) {
                                if matches!(state.apply_remote_publisher(from_peer, generation, false), Some((true, false))) {
                                    log::info!("[PubsubService] remote peer {from_peer} leaved from {channel} as publisher");
                                    // we have remove remote publisher then we fire event to local
                                    for sub_tx in state.local_subscribers.values() {
                                        Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerLeaved(PeerSrc::Remote(from_peer)));
                                    }
                                }
                            }
                        }
                        PubsubMessage::SubscriberJoined(channel, generation) => {
                            let mut reply = None;
                            if let Some(state) = self.channels.get_mut(&channel) {
                                if matches!(state.apply_remote_subscriber(from_peer, generation, true), Some((false, true))) {
                                    log::info!("[PubsubService] remote peer {from_peer} joined to {channel} as subscriber");
                                    // we have new remote publisher then we fire event to local
                                    for (_, pub_tx) in state.local_publishers.iter() {
                                        Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerJoined(PeerSrc::Remote(from_peer)));
                                    }
                                    // we also send publisher state it remote, as subscriber it only care about wherever this node is a publisher
                                    if !state.local_publishers.is_empty() {
                                        reply = Some(PubsubMessage::PublisherJoined(channel, state.local_publish_generation));
                                    }
                                }
                            }
                            if let Some(reply) = reply {
                                self.send_to(from_peer, &reply).await;
                            }
                        }
                        PubsubMessage::SubscriberLeaved(channel, generation) => {
                            if let Some(state) = self.channels.get_mut(&channel) {
                                if matches!(state.apply_remote_subscriber(from_peer, generation, false), Some((true, false))) {
                                    log::info!("[PubsubService] remote peer {from_peer} leaved from {channel} as subscriber");
                                    // we have remove remote publisher then we fire event to local
                                    for (_, pub_tx) in state.local_publishers.iter() {
                                        Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerLeaved(PeerSrc::Remote(from_peer)));
                                    }
                                }
                            }
                        }
                        PubsubMessage::Heartbeat(channels) => {
                            for heartbeat in channels {
                                if let Some(state) = self.channels.get_mut(&heartbeat.channel) {
                                    if let Some((was_active, is_active)) = state.apply_remote_publisher(from_peer, heartbeat.publish_generation, heartbeat.publish) {
                                        if was_active != is_active {
                                            for sub_tx in state.local_subscribers.values() {
                                                let event = if is_active {
                                                    SubscriberEvent::PeerJoined(PeerSrc::Remote(from_peer))
                                                } else {
                                                    SubscriberEvent::PeerLeaved(PeerSrc::Remote(from_peer))
                                                };
                                                Self::try_send_subscriber_event(sub_tx, event);
                                            }
                                        }
                                    }

                                    if let Some((was_active, is_active)) = state.apply_remote_subscriber(from_peer, heartbeat.subscribe_generation, heartbeat.subscribe) {
                                        if was_active != is_active {
                                            for (_, pub_tx) in state.local_publishers.iter() {
                                                let event = if is_active {
                                                    PublisherEvent::PeerJoined(PeerSrc::Remote(from_peer))
                                                } else {
                                                    PublisherEvent::PeerLeaved(PeerSrc::Remote(from_peer))
                                                };
                                                Self::try_send_publisher_event(pub_tx, event);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        PubsubMessage::GuestPublish(channel, data) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublish(data.clone()));
                                }
                            }
                        }
                        PubsubMessage::GuestPublishRpc(channel, data, rpc_id, method) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublishRpc(data.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::Publish(channel, vec) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::Publish(vec.clone()));
                                }
                            }
                        }
                        PubsubMessage::PublishRpc(channel, vec, rpc_id, method) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PublishRpc(vec.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::PublishRpcAnswer(data, rpc_id) => {
                            let responder = PeerSrc::Remote(from_peer);
                            if self.publish_rpc_reqs.get(&rpc_id).is_some_and(|req| req.expected_responders.contains(&responder)) {
                                let mut req = self.publish_rpc_reqs.remove(&rpc_id).expect("checked pending publish RPC");
                                let _ = req.tx.take().expect("should have req_tx").send(Ok(data));
                            } else if self.publish_rpc_reqs.contains_key(&rpc_id) {
                                log::warn!("[PubsubService] got PublishRpcAnswer from unexpected responder {from_peer} for req_id {rpc_id}");
                            } else {
                                log::warn!("[PubsubService] got PublishRpcAnswer with invalid req_id {rpc_id}");
                            }
                        }
                        PubsubMessage::GuestFeedback(channel, vec) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedback(vec.clone()));
                                }
                            }
                        }
                        PubsubMessage::GuestFeedbackRpc(channel, vec, rpc_id, method) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedbackRpc(vec.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::Feedback(channel, vec) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::Feedback(vec.clone()));
                                }
                            }
                        }
                        PubsubMessage::FeedbackRpc(channel, vec, rpc_id, method) => {
                            if let Some(state) = self.channels.get(&channel) {
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::FeedbackRpc(vec.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::FeedbackRpcAnswer(data, rpc_id) => {
                            let responder = PeerSrc::Remote(from_peer);
                            if self.feedback_rpc_reqs.get(&rpc_id).is_some_and(|req| req.expected_responders.contains(&responder)) {
                                let mut req = self.feedback_rpc_reqs.remove(&rpc_id).expect("checked pending feedback RPC");
                                let _ = req.tx.take().expect("should have req_tx").send(Ok(data));
                            } else if self.feedback_rpc_reqs.contains_key(&rpc_id) {
                                log::warn!("[PubsubService] got FeedbackRpcAnswer from unexpected responder {from_peer} for req_id {rpc_id}");
                            } else {
                                log::warn!("[PubsubService] got FeedbackRpcAnswer with invalid req_id {rpc_id}");
                            }
                        }
                    }
                }
                Ok(())
            }
            P2pServiceEvent::Stream(..) => Ok(()),
            P2pServiceEvent::PeerDisconnected(peer) => {
                self.on_peer_disconnected(peer);
                Ok(())
            }
        }
    }

    fn on_peer_disconnected(&mut self, peer: PeerId) {
        for (channel, state) in self.channels.iter_mut() {
            if state.remote_publishers.remove(&peer).is_some_and(|role| role.active) {
                log::info!("[PubsubService] remote peer {peer} disconnected from {channel} as publisher");
                for sub_tx in state.local_subscribers.values() {
                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerLeaved(PeerSrc::Remote(peer)));
                }
            }

            if state.remote_subscribers.remove(&peer).is_some_and(|role| role.active) {
                log::info!("[PubsubService] remote peer {peer} disconnected from {channel} as subscriber");
                for pub_tx in state.local_publishers.values() {
                    Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerLeaved(PeerSrc::Remote(peer)));
                }
            }
        }
    }

    #[allow(clippy::collapsible_else_if)]
    async fn on_internal(&mut self, control: InternalMsg) -> anyhow::Result<()> {
        match control {
            InternalMsg::PublisherCreated(handle_id, channel, tx) => {
                log::info!("[PubsubService] local created pub channel {channel} / {handle_id}");
                let mut broadcast = None;
                {
                    let state = self.channels.entry(channel).or_default();
                    if !state.local_subscribers.is_empty() {
                        // notify that we already have local subscribers
                        Self::try_send_publisher_event(&tx, PublisherEvent::PeerJoined(PeerSrc::Local));
                    }
                    for peer in state.active_remote_subscribers() {
                        Self::try_send_publisher_event(&tx, PublisherEvent::PeerJoined(PeerSrc::Remote(peer)));
                    }
                    if state.local_publishers.is_empty() {
                        // if this is first local_publisher => notify to all local_subscribers
                        for sub_tx in state.local_subscribers.values() {
                            Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerJoined(PeerSrc::Local));
                        }
                        state.local_publishers.insert(handle_id, tx);
                        state.local_publish_generation = state.local_publish_generation.saturating_add(1);
                        broadcast = Some(PubsubMessage::PublisherJoined(channel, state.local_publish_generation));
                    } else {
                        state.local_publishers.insert(handle_id, tx);
                    }
                }
                if let Some(msg) = broadcast {
                    self.broadcast(&msg).await;
                }
            }
            InternalMsg::PublisherDestroyed(handle_id, channel) => {
                log::info!("[PubsubService] local destroyed pub channel {channel} / {handle_id}");

                let mut broadcast = None;
                {
                    let Some(state) = self.channels.get_mut(&channel) else {
                        return Ok(());
                    };
                    if state.local_publishers.remove(&handle_id).is_none() {
                        return Ok(());
                    }
                    if state.local_publishers.is_empty() {
                        // if this is last local_publisher => notify all subscribers
                        for sub_tx in state.local_subscribers.values() {
                            Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerLeaved(PeerSrc::Local));
                        }
                        state.local_publish_generation = state.local_publish_generation.saturating_add(1);
                        broadcast = Some(PubsubMessage::PublisherLeaved(channel, state.local_publish_generation));
                    }
                }
                if let Some(msg) = broadcast {
                    self.broadcast(&msg).await;
                }
            }
            InternalMsg::SubscriberCreated(handle_id, channel, tx) => {
                log::info!("[PubsubService] local created sub channel {channel} / {handle_id}");
                let mut broadcast = None;
                {
                    let state = self.channels.entry(channel).or_default();
                    if !state.local_publishers.is_empty() {
                        // notify that we already have local publishers
                        Self::try_send_subscriber_event(&tx, SubscriberEvent::PeerJoined(PeerSrc::Local));
                    }
                    for peer in state.active_remote_publishers() {
                        Self::try_send_subscriber_event(&tx, SubscriberEvent::PeerJoined(PeerSrc::Remote(peer)));
                    }
                    if state.local_subscribers.is_empty() {
                        // if this is first local_subsrciber => notify to all local_publishers
                        for (_, pub_tx) in state.local_publishers.iter() {
                            Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerJoined(PeerSrc::Local));
                        }
                        state.local_subscribers.insert(handle_id, tx);
                        state.local_subscribe_generation = state.local_subscribe_generation.saturating_add(1);
                        broadcast = Some(PubsubMessage::SubscriberJoined(channel, state.local_subscribe_generation));
                    } else {
                        state.local_subscribers.insert(handle_id, tx);
                    }
                }
                if let Some(msg) = broadcast {
                    self.broadcast(&msg).await;
                }
            }
            InternalMsg::SubscriberDestroyed(handle_id, channel) => {
                log::info!("[PubsubService] local destroyed sub channel {channel} / {handle_id}");
                let mut broadcast = None;
                {
                    let Some(state) = self.channels.get_mut(&channel) else {
                        return Ok(());
                    };
                    if state.local_subscribers.remove(&handle_id).is_none() {
                        return Ok(());
                    }
                    if state.local_subscribers.is_empty() {
                        // if this is last local_subscriber => notify all publishers
                        for (_, pub_tx) in state.local_publishers.iter() {
                            Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerLeaved(PeerSrc::Local));
                        }
                        state.local_subscribe_generation = state.local_subscribe_generation.saturating_add(1);
                        broadcast = Some(PubsubMessage::SubscriberLeaved(channel, state.local_subscribe_generation));
                    }
                }
                if let Some(msg) = broadcast {
                    self.broadcast(&msg).await;
                }
            }
            InternalMsg::GuestPublish(channel, vec) => {
                if let Some(state) = self.channels.get(&channel) {
                    for sub_tx in state.local_subscribers.values() {
                        Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublish(vec.clone()));
                    }
                    for sub_peer in state.active_remote_subscribers() {
                        let _ = self.send_to(sub_peer, &PubsubMessage::GuestPublish(channel, vec.clone())).await;
                    }
                }
            }
            InternalMsg::GuestPublishRpc(channel, data, method, tx, timeout) => {
                if let Some(state) = self.channels.get(&channel) {
                    let req_id = RpcId::rand();
                    if state.local_subscribers.is_empty() && !state.has_active_remote_subscribers() {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        for sub_tx in state.local_subscribers.values() {
                            if Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublishRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                            }
                        }
                        for pub_peer in state.active_remote_subscribers() {
                            if self.send_to(pub_peer, &PubsubMessage::GuestPublishRpc(channel, data.clone(), req_id, method.clone())).await {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Remote(pub_peer));
                            }
                        }
                        if delivered == 0 {
                            let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        } else {
                            self.publish_rpc_reqs.insert(
                                req_id,
                                PublishRpcReq {
                                    started_at: Instant::now(),
                                    timeout,
                                    expected_responders,
                                    tx: Some(tx),
                                },
                            );
                        }
                    }
                } else {
                    let _ = tx.send(Err(PubsubRpcError::NoDestination));
                }
            }
            InternalMsg::Publish(handle_id, channel, vec) => {
                if let Some(state) = self.channels.get(&channel) {
                    if !state.local_publishers.contains_key(&handle_id) {
                        return Ok(());
                    }
                    for sub_tx in state.local_subscribers.values() {
                        Self::try_send_subscriber_event(sub_tx, SubscriberEvent::Publish(vec.clone()));
                    }
                    for sub_peer in state.active_remote_subscribers() {
                        let _ = self.send_to(sub_peer, &PubsubMessage::Publish(channel, vec.clone())).await;
                    }
                }
            }
            InternalMsg::PublishRpc(handle_id, channel, data, method, tx, timeout) => {
                if let Some(state) = self.channels.get(&channel) {
                    if !state.local_publishers.contains_key(&handle_id) {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        return Ok(());
                    }
                    let req_id = RpcId::rand();
                    if state.local_subscribers.is_empty() && !state.has_active_remote_subscribers() {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        for sub_tx in state.local_subscribers.values() {
                            if Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PublishRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                            }
                        }
                        for pub_peer in state.active_remote_subscribers() {
                            if self.send_to(pub_peer, &PubsubMessage::PublishRpc(channel, data.clone(), req_id, method.clone())).await {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Remote(pub_peer));
                            }
                        }
                        if delivered == 0 {
                            let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        } else {
                            self.publish_rpc_reqs.insert(
                                req_id,
                                PublishRpcReq {
                                    started_at: Instant::now(),
                                    timeout,
                                    expected_responders,
                                    tx: Some(tx),
                                },
                            );
                        }
                    }
                } else {
                    let _ = tx.send(Err(PubsubRpcError::NoDestination));
                }
            }
            InternalMsg::GuestFeedback(channel, vec) => {
                if let Some(state) = self.channels.get(&channel) {
                    for (_, pub_tx) in state.local_publishers.iter() {
                        Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedback(vec.clone()));
                    }
                    for pub_peer in state.active_remote_publishers() {
                        let _ = self.send_to(pub_peer, &PubsubMessage::GuestFeedback(channel, vec.clone())).await;
                    }
                }
            }
            InternalMsg::GuestFeedbackRpc(channel, data, method, tx, timeout) => {
                if let Some(state) = self.channels.get(&channel) {
                    let req_id = RpcId::rand();
                    if state.local_publishers.is_empty() && !state.has_active_remote_publishers() {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        for (_, pub_tx) in state.local_publishers.iter() {
                            if Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedbackRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                            }
                        }
                        for pub_peer in state.active_remote_publishers() {
                            if self.send_to(pub_peer, &PubsubMessage::GuestFeedbackRpc(channel, data.clone(), req_id, method.clone())).await {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Remote(pub_peer));
                            }
                        }
                        if delivered == 0 {
                            let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        } else {
                            self.feedback_rpc_reqs.insert(
                                req_id,
                                FeedbackRpcReq {
                                    started_at: Instant::now(),
                                    timeout,
                                    expected_responders,
                                    tx: Some(tx),
                                },
                            );
                        }
                    }
                } else {
                    let _ = tx.send(Err(PubsubRpcError::NoDestination));
                }
            }
            InternalMsg::Feedback(handle_id, channel, vec) => {
                if let Some(state) = self.channels.get(&channel) {
                    if !state.local_subscribers.contains_key(&handle_id) {
                        return Ok(());
                    }
                    for (_, pub_tx) in state.local_publishers.iter() {
                        Self::try_send_publisher_event(pub_tx, PublisherEvent::Feedback(vec.clone()));
                    }
                    for pub_peer in state.active_remote_publishers() {
                        let _ = self.send_to(pub_peer, &PubsubMessage::Feedback(channel, vec.clone())).await;
                    }
                }
            }
            InternalMsg::FeedbackRpc(handle_id, channel, data, method, tx, timeout) => {
                if let Some(state) = self.channels.get(&channel) {
                    if !state.local_subscribers.contains_key(&handle_id) {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        return Ok(());
                    }
                    let req_id = RpcId::rand();
                    if state.local_publishers.is_empty() && !state.has_active_remote_publishers() {
                        let _ = tx.send(Err(PubsubRpcError::NoDestination));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        for (_, pub_tx) in state.local_publishers.iter() {
                            if Self::try_send_publisher_event(pub_tx, PublisherEvent::FeedbackRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                            }
                        }
                        for pub_peer in state.active_remote_publishers() {
                            if self.send_to(pub_peer, &PubsubMessage::FeedbackRpc(channel, data.clone(), req_id, method.clone())).await {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Remote(pub_peer));
                            }
                        }
                        if delivered == 0 {
                            let _ = tx.send(Err(PubsubRpcError::NoDestination));
                        } else {
                            self.feedback_rpc_reqs.insert(
                                req_id,
                                FeedbackRpcReq {
                                    started_at: Instant::now(),
                                    timeout,
                                    expected_responders,
                                    tx: Some(tx),
                                },
                            );
                        }
                    }
                } else {
                    let _ = tx.send(Err(PubsubRpcError::NoDestination));
                }
            }
            InternalMsg::PublishRpcAnswer(rpc_id, peer_src, data) => {
                if let PeerSrc::Remote(peer) = peer_src {
                    self.try_send_to(peer, &PubsubMessage::PublishRpcAnswer(data, rpc_id)).await;
                } else {
                    if self.publish_rpc_reqs.get(&rpc_id).is_some_and(|req| req.expected_responders.contains(&PeerSrc::Local)) {
                        let mut req = self.publish_rpc_reqs.remove(&rpc_id).expect("checked pending publish RPC");
                        let _ = req.tx.take().expect("should have req_tx").send(Ok(data));
                    } else if self.publish_rpc_reqs.contains_key(&rpc_id) {
                        log::warn!("[PubsubService] got local PublishRpcAnswer from unexpected responder for req_id {rpc_id}");
                    } else {
                        log::warn!("[PubsubService] got local PublishRpcAnswer with invalid req_id {rpc_id}");
                    }
                }
            }
            InternalMsg::FeedbackRpcAnswer(rpc_id, peer_src, data) => {
                if let PeerSrc::Remote(peer) = peer_src {
                    self.try_send_to(peer, &PubsubMessage::FeedbackRpcAnswer(data, rpc_id)).await;
                } else {
                    if self.feedback_rpc_reqs.get(&rpc_id).is_some_and(|req| req.expected_responders.contains(&PeerSrc::Local)) {
                        let mut req = self.feedback_rpc_reqs.remove(&rpc_id).expect("checked pending feedback RPC");
                        let _ = req.tx.take().expect("should have req_tx").send(Ok(data));
                    } else if self.feedback_rpc_reqs.contains_key(&rpc_id) {
                        log::warn!("[PubsubService] got local FeedbackRpcAnswer from unexpected responder for req_id {rpc_id}");
                    } else {
                        log::warn!("[PubsubService] got local FeedbackRpcAnswer with invalid req_id {rpc_id}");
                    }
                }
            }
        }
        Ok(())
    }

    fn try_send_subscriber_event(tx: &Sender<SubscriberEvent>, event: SubscriberEvent) -> bool {
        match tx.try_send(event) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                log::debug!("[PubsubService] local subscriber event queue full");
                false
            }
            Err(TrySendError::Closed(_)) => {
                log::debug!("[PubsubService] local subscriber event queue closed");
                false
            }
        }
    }

    fn try_send_publisher_event(tx: &Sender<PublisherEvent>, event: PublisherEvent) -> bool {
        match tx.try_send(event) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                log::debug!("[PubsubService] local publisher event queue full");
                false
            }
            Err(TrySendError::Closed(_)) => {
                log::debug!("[PubsubService] local publisher event queue closed");
                false
            }
        }
    }

    async fn send_to(&self, dest: PeerId, msg: &PubsubMessage) -> bool {
        let result = self.service.send_unicast(dest, bincode::serialize(msg).expect("should convert to binary")).await;
        result.print_on_err("[PubsubService] send data");
        result.is_ok()
    }

    async fn try_send_to(&self, dest: PeerId, msg: &PubsubMessage) {
        self.service
            .try_send_unicast(dest, bincode::serialize(msg).expect("should convert to binary"))
            .await
            .print_on_err("[PubsubService] try send data");
    }

    async fn broadcast(&self, msg: &PubsubMessage) {
        self.service
            .send_broadcast(bincode::serialize(msg).expect("should convert to binary"))
            .await
            .print_on_err("[PubsubService] broadcast data");
    }
}

fn try_send_internal_control(tx: &Sender<InternalMsg>, msg: InternalMsg, context: &str) -> anyhow::Result<()> {
    match tx.try_send(msg) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => Err(anyhow!("{context}: pubsub internal control queue full")),
        Err(TrySendError::Closed(_)) => Err(anyhow!("{context}: pubsub internal control queue closed")),
    }
}

impl PubsubServiceRequester {
    pub async fn publish_as_guest(&self, channel: PubsubChannelId, data: Vec<u8>) -> anyhow::Result<()> {
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestPublish(channel, data), "publish_as_guest")?;
        Ok(())
    }

    pub async fn publish_as_guest_ob<Ob: Serialize>(&self, channel: PubsubChannelId, ob: Ob) -> anyhow::Result<()> {
        let data = bincode::serialize(&ob).expect("should serialize");
        self.publish_as_guest(channel, data).await
    }

    pub async fn publish_rpc_as_guest(&self, channel: PubsubChannelId, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestPublishRpc(channel, data, method.to_owned(), tx, timeout), "publish_rpc_as_guest")?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn publish_rpc_as_guest_ob<REQ: Serialize, RES: DeserializeOwned>(&self, channel: PubsubChannelId, method: &str, req: REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(&req).expect("should serialize");
        let res = self.publish_rpc_as_guest(channel, method, data, timeout).await?;
        Ok(bincode::deserialize(&res)?)
    }

    pub async fn feedback_as_guest(&self, channel: PubsubChannelId, data: Vec<u8>) -> anyhow::Result<()> {
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestFeedback(channel, data), "feedback_as_guest")?;
        Ok(())
    }

    pub async fn feedback_as_guest_ob<Ob: Serialize>(&self, channel: PubsubChannelId, ob: Ob) -> anyhow::Result<()> {
        let data = bincode::serialize(&ob).expect("should serialize");
        self.feedback_as_guest(channel, data).await
    }

    pub async fn feedback_rpc_as_guest(&self, channel: PubsubChannelId, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestFeedbackRpc(channel, data, method.to_owned(), tx, timeout), "feedback_rpc_as_guest")?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn feedback_rpc_as_guest_ob<REQ: Serialize, RES: DeserializeOwned>(&self, channel: PubsubChannelId, method: &str, req: REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(&req).expect("should serialize");
        let res = self.feedback_rpc_as_guest(channel, method, data, timeout).await?;
        Ok(bincode::deserialize(&res)?)
    }

    pub async fn publisher(&self, channel: PubsubChannelId) -> Publisher {
        Publisher::build(channel, self.internal_tx.clone())
    }

    pub async fn subscriber(&self, channel: PubsubChannelId) -> Subscriber {
        Subscriber::build(channel, self.internal_tx.clone())
    }
}

#[cfg(test)]
mod test {
    use futures::FutureExt;
    use serde::Serializer;
    use tokio::sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    };

    use super::*;
    use crate::{ctx::SharedCtx, msg::P2pServiceId, peer::test_congested_peer_alias, router::SharedRouterTable, ConnectionId};

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom("intentional serialization failure"))
        }
    }

    fn test_service() -> PubsubService {
        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (service, _tx) = P2pService::build(P2pServiceId::from(0), ctx);
        PubsubService::new(service)
    }

    fn publisher_handle(local_id: PublisherLocalId) -> PublisherHandleId {
        PublisherHandleId::new(local_id)
    }

    fn subscriber_handle(local_id: SubscriberLocalId) -> SubscriberHandleId {
        SubscriberHandleId::new(local_id)
    }

    fn subscriber_event_channel() -> (Sender<SubscriberEvent>, Receiver<SubscriberEvent>) {
        channel(subscriber::LOCAL_SUBSCRIBER_EVENT_QUEUE_SIZE)
    }

    fn publisher_event_channel() -> (Sender<PublisherEvent>, Receiver<PublisherEvent>) {
        channel(publisher::LOCAL_PUBLISHER_EVENT_QUEUE_SIZE)
    }

    async fn fill_internal_control_queue(requester: &PubsubServiceRequester) -> Vec<Publisher> {
        let mut publishers = Vec::with_capacity(PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        for channel in 0..PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE {
            publishers.push(requester.publisher(PubsubChannelId(channel as u64 + 10)).await);
        }
        publishers
    }

    #[tokio::test]
    async fn pubsub_internal_control_backlog_must_be_bounded() {
        let service = test_service();
        let requester = service.requester();
        let mut publishers = Vec::new();

        for channel in 0..=PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE {
            publishers.push(requester.publisher(PubsubChannelId(channel as u64 + 10)).await);
        }

        assert!(
            service.internal_rx.len() <= PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE,
            "pending pubsub internal control messages must be bounded, got {}",
            service.internal_rx.len()
        );
    }

    #[tokio::test]
    async fn pubsub_guest_publish_returns_error_when_internal_queue_full() {
        let service = test_service();
        let requester = service.requester();
        let _publishers = fill_internal_control_queue(&requester).await;

        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        assert!(requester.publish_as_guest(PubsubChannelId(1), b"payload".to_vec()).await.is_err());
        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
    }

    #[tokio::test]
    async fn pubsub_guest_publish_rpc_returns_error_when_internal_queue_full() {
        let service = test_service();
        let requester = service.requester();
        let _publishers = fill_internal_control_queue(&requester).await;

        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        assert!(requester.publish_rpc_as_guest(PubsubChannelId(1), "method", b"payload".to_vec(), Duration::from_secs(1)).await.is_err());
        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
    }

    #[tokio::test]
    async fn pubsub_guest_feedback_returns_error_when_internal_queue_full() {
        let service = test_service();
        let requester = service.requester();
        let _publishers = fill_internal_control_queue(&requester).await;

        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        assert!(requester.feedback_as_guest(PubsubChannelId(1), b"payload".to_vec()).await.is_err());
        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
    }

    #[tokio::test]
    async fn pubsub_guest_feedback_rpc_returns_error_when_internal_queue_full() {
        let service = test_service();
        let requester = service.requester();
        let _publishers = fill_internal_control_queue(&requester).await;

        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        assert!(requester
            .feedback_rpc_as_guest(PubsubChannelId(1), "method", b"payload".to_vec(), Duration::from_secs(1))
            .await
            .is_err());
        assert_eq!(service.internal_rx.len(), PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
    }

    #[tokio::test]
    async fn pending_publish_rpc_requests_must_be_bounded() {
        const MAX_PENDING_RPCS: usize = 1024;
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for _ in 0..=MAX_PENDING_RPCS {
            let (tx, _rx) = oneshot::channel();
            service
                .on_internal(InternalMsg::GuestPublishRpc(channel, vec![1], "hold".to_string(), tx, Duration::from_secs(3600)))
                .await
                .expect("publish RPC should be accepted");
        }

        let pending_rpcs = service.publish_rpc_reqs.len();
        assert!(pending_rpcs <= MAX_PENDING_RPCS, "pending publish RPC requests must be bounded, got {pending_rpcs}");
    }

    #[tokio::test]
    async fn pubsub_rpc_must_return_no_destination_when_all_remote_sends_fail() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let stale_remote = PeerId::from(99);

        service.channels.entry(channel).or_default().apply_remote_subscriber(stale_remote, 1, true);

        let (tx, mut rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::GuestPublishRpc(channel, b"payload".to_vec(), "method".to_string(), tx, Duration::from_secs(60)))
            .await
            .expect("RPC should process");

        assert_eq!(
            rx.try_recv(),
            Ok(Err(PubsubRpcError::NoDestination)),
            "if every remote send fails immediately, RPC must fail as NoDestination instead of waiting for timeout"
        );
        assert!(service.publish_rpc_reqs.is_empty(), "failed fanout must not leave a pending RPC request");
    }

    #[tokio::test]
    async fn pubsub_rpc_must_return_no_destination_when_all_local_sends_fail() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, sub_rx) = subscriber_event_channel();
        drop(sub_rx);

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("closed subscriber sender should still reach current broken state");

        let (tx, mut rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::GuestPublishRpc(channel, b"payload".to_vec(), "method".to_string(), tx, Duration::from_secs(60)))
            .await
            .expect("RPC should process");

        assert_eq!(
            rx.try_recv(),
            Ok(Err(PubsubRpcError::NoDestination)),
            "if every local RPC event send fails immediately, RPC must fail as NoDestination instead of waiting for timeout"
        );
        assert!(service.publish_rpc_reqs.is_empty(), "failed local fanout must not leave a pending RPC request");
    }

    #[tokio::test]
    async fn pubsub_rpc_must_return_no_destination_when_all_local_subscriber_queues_are_full() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        for _ in 0..subscriber::LOCAL_SUBSCRIBER_EVENT_QUEUE_SIZE {
            sub_tx.try_send(SubscriberEvent::Publish(Vec::new())).expect("test subscriber queue should accept events until full");
        }

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("full subscriber sender should still be registered");

        let (tx, mut rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::GuestPublishRpc(channel, b"payload".to_vec(), "method".to_string(), tx, Duration::from_secs(60)))
            .await
            .expect("RPC should process");

        assert_eq!(
            rx.try_recv(),
            Ok(Err(PubsubRpcError::NoDestination)),
            "if every local RPC event queue is full, RPC must fail as NoDestination instead of waiting for timeout"
        );
        assert!(service.publish_rpc_reqs.is_empty(), "full local fanout must not leave a pending RPC request");
    }

    #[tokio::test]
    async fn feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let subscriber_handle = subscriber_handle(SubscriberLocalId::rand());
        let (pub_tx, _pub_rx) = publisher_event_channel();
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        for _ in 0..publisher::LOCAL_PUBLISHER_EVENT_QUEUE_SIZE {
            pub_tx.try_send(PublisherEvent::Feedback(Vec::new())).expect("test publisher queue should accept events until full");
        }

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("full publisher sender should still be registered");
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle, channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        let (tx, mut rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::FeedbackRpc(
                subscriber_handle,
                channel,
                b"payload".to_vec(),
                "method".to_string(),
                tx,
                Duration::from_secs(60),
            ))
            .await
            .expect("feedback RPC should process");

        assert_eq!(
            rx.try_recv(),
            Ok(Err(PubsubRpcError::NoDestination)),
            "if every local feedback RPC event queue is full, RPC must fail as NoDestination instead of waiting for timeout"
        );
        assert!(service.feedback_rpc_reqs.is_empty(), "full local feedback fanout must not leave a pending RPC request");
    }

    #[tokio::test]
    async fn guest_feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (pub_tx, _pub_rx) = publisher_event_channel();

        for _ in 0..publisher::LOCAL_PUBLISHER_EVENT_QUEUE_SIZE {
            pub_tx.try_send(PublisherEvent::Feedback(Vec::new())).expect("test publisher queue should accept events until full");
        }

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("full publisher sender should still be registered");

        let (tx, mut rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::GuestFeedbackRpc(channel, b"payload".to_vec(), "method".to_string(), tx, Duration::from_secs(60)))
            .await
            .expect("guest feedback RPC should process");

        assert_eq!(
            rx.try_recv(),
            Ok(Err(PubsubRpcError::NoDestination)),
            "if every local guest feedback RPC event queue is full, RPC must fail as NoDestination instead of waiting for timeout"
        );
        assert!(service.feedback_rpc_reqs.is_empty(), "full local guest feedback fanout must not leave a pending RPC request");
    }

    #[tokio::test]
    async fn pubsub_remote_rpc_answer_must_not_block_service_loop_on_full_peer_control_queue() {
        let local = PeerId::from(1);
        let remote = PeerId::from(2);
        let conn = ConnectionId::from(7);
        let router = SharedRouterTable::new(local);
        let ctx = SharedCtx::new(local, router.clone());
        let congested = test_congested_peer_alias(local, remote, conn);
        router.set_direct(conn, remote, 0);
        ctx.register_conn(conn, congested.alias());
        let (base_service, _tx) = P2pService::build(P2pServiceId::from(0), ctx);
        let mut service = PubsubService::new(base_service);

        let result = tokio::time::timeout(
            Duration::from_millis(50),
            service.on_internal(InternalMsg::PublishRpcAnswer(RpcId(7), PeerSrc::Remote(remote), b"answer".to_vec())),
        )
        .await;

        assert!(result.is_ok(), "remote pubsub RPC answers must not block the service loop behind a full peer-control queue");
    }

    #[tokio::test]
    async fn remote_publisher_memberships_must_be_bounded() {
        const MAX_REMOTE_MEMBERS: usize = 1024;
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();
        let joined = bincode::serialize(&PubsubMessage::PublisherJoined(channel, 1)).expect("test message should serialize");

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for peer in 0..=MAX_REMOTE_MEMBERS {
            service
                .on_service(P2pServiceEvent::Unicast(PeerId::from(peer as u64 + 10), joined.clone()))
                .await
                .expect("remote publisher join should be processed");
        }

        let remote_publishers = service.channels.get(&channel).expect("channel should exist").active_remote_publishers_count();
        assert!(remote_publishers <= MAX_REMOTE_MEMBERS, "remote publisher memberships must be bounded, got {remote_publishers}");
    }

    #[tokio::test]
    async fn local_subscriber_event_backlog_must_be_bounded() {
        const MAX_LOCAL_EVENT_BACKLOG: usize = 1024;
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let from_peer = PeerId::from(2);
        let (sub_tx, sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for i in 0..=MAX_LOCAL_EVENT_BACKLOG {
            let payload = bincode::serialize(&PubsubMessage::Publish(channel, vec![i as u8])).expect("test publish should serialize");
            service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("publish should be processed");
        }

        assert!(
            sub_rx.len() <= MAX_LOCAL_EVENT_BACKLOG,
            "undrained local subscriber event backlog must be bounded, got {}",
            sub_rx.len()
        );
    }

    #[tokio::test]
    async fn local_publisher_event_backlog_must_be_bounded() {
        const MAX_LOCAL_EVENT_BACKLOG: usize = 1024;
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let from_peer = PeerId::from(2);
        let (pub_tx, pub_rx) = publisher_event_channel();

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");

        for i in 0..=MAX_LOCAL_EVENT_BACKLOG {
            let payload = bincode::serialize(&PubsubMessage::Feedback(channel, vec![i as u8])).expect("test feedback should serialize");
            service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("feedback should be processed");
        }

        assert!(pub_rx.len() <= MAX_LOCAL_EVENT_BACKLOG, "undrained local publisher event backlog must be bounded, got {}", pub_rx.len());
    }

    #[tokio::test]
    async fn new_local_pubsub_handles_must_observe_existing_remote_members() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote_publisher = PeerId::from(2);
        let remote_subscriber = PeerId::from(3);
        let (pub_tx, mut pub_rx) = publisher_event_channel();
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        let state = service.channels.entry(channel).or_default();
        state.apply_remote_publisher(remote_publisher, 1, true);
        state.apply_remote_subscriber(remote_subscriber, 1, true);

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        let mut publisher_events = Vec::new();
        while let Ok(event) = pub_rx.try_recv() {
            publisher_events.push(event);
        }
        let mut subscriber_events = Vec::new();
        while let Ok(event) = sub_rx.try_recv() {
            subscriber_events.push(event);
        }

        assert!(
            publisher_events.contains(&PublisherEvent::PeerJoined(PeerSrc::Remote(remote_subscriber))),
            "a new local publisher must observe already-known remote subscribers, got {publisher_events:?}"
        );
        assert!(
            subscriber_events.contains(&SubscriberEvent::PeerJoined(PeerSrc::Remote(remote_publisher))),
            "a new local subscriber must observe already-known remote publishers, got {subscriber_events:?}"
        );
    }

    #[tokio::test]
    async fn early_remote_publisher_join_must_survive_late_local_subscriber_creation() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("early remote publisher join should be processed");

        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("late subscriber should be registered");

        assert!(
            service.channels.get(&channel).expect("channel should exist after subscriber creation").has_remote_publisher(remote),
            "remote publisher join received before local channel creation must be retained"
        );
        assert_eq!(
            sub_rx.try_recv(),
            Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))),
            "late local subscriber must observe the already-joined remote publisher"
        );
    }

    #[tokio::test]
    async fn pubsub_heartbeat_channel_batches_must_be_bounded() {
        const MAX_HEARTBEAT_CHANNELS: usize = 1024;
        let mut service = test_service();
        let from_peer = PeerId::from(2);
        let mut heartbeats = Vec::new();

        for channel in 0..=MAX_HEARTBEAT_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            let (sub_tx, _sub_rx) = subscriber_event_channel();
            service
                .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
                .await
                .expect("subscriber should be registered");
            heartbeats.push(ChannelHeartbeat {
                channel,
                publish: true,
                publish_generation: 1,
                subscribe: false,
                subscribe_generation: 1,
            });
        }

        let payload = bincode::serialize(&PubsubMessage::Heartbeat(heartbeats)).expect("test heartbeat should serialize");
        service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("heartbeat should be processed");

        let updated_channels = service.channels.values().filter(|state| state.has_remote_publisher(from_peer)).count();

        assert!(
            updated_channels <= MAX_HEARTBEAT_CHANNELS,
            "pubsub heartbeat channel batches must be bounded, updated {updated_channels} channels"
        );
    }

    #[tokio::test]
    async fn empty_pubsub_channels_must_be_removed_after_last_local_handle_drops() {
        const MAX_EMPTY_CHANNELS: usize = 1024;
        let mut service = test_service();

        for channel in 0..=MAX_EMPTY_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            let local_id = SubscriberLocalId::rand();
            let handle_id = subscriber_handle(local_id);
            let (sub_tx, _sub_rx) = subscriber_event_channel();

            service
                .on_internal(InternalMsg::SubscriberCreated(handle_id, channel, sub_tx))
                .await
                .expect("subscriber should be registered");
            service.on_internal(InternalMsg::SubscriberDestroyed(handle_id, channel)).await.expect("subscriber should be destroyed");
        }

        let empty_channels = service
            .channels
            .values()
            .filter(|state| state.local_publishers.is_empty() && state.local_subscribers.is_empty() && !state.has_active_remote_publishers() && !state.has_active_remote_subscribers())
            .count();

        assert!(
            empty_channels <= MAX_EMPTY_CHANNELS,
            "empty pubsub channel state must be removed after last local handle drops, got {empty_channels}"
        );
    }

    #[tokio::test]
    async fn stale_pubsub_destroy_must_not_create_phantom_channel() {
        let mut service = test_service();
        let publisher_channel = PubsubChannelId(77);
        let subscriber_channel = PubsubChannelId(78);

        service
            .on_internal(InternalMsg::PublisherDestroyed(publisher_handle(PublisherLocalId::rand()), publisher_channel))
            .await
            .expect("stale publisher destroy should be processed");
        service
            .on_internal(InternalMsg::SubscriberDestroyed(subscriber_handle(SubscriberLocalId::rand()), subscriber_channel))
            .await
            .expect("stale subscriber destroy should be processed");

        assert!(
            !service.channels.contains_key(&publisher_channel),
            "destroy for an unknown publisher handle must not create phantom channel state"
        );
        assert!(
            !service.channels.contains_key(&subscriber_channel),
            "destroy for an unknown subscriber handle must not create phantom channel state"
        );
    }

    #[tokio::test]
    async fn duplicate_publisher_local_id_must_not_detach_live_handle() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let local_id = PublisherLocalId::from_raw_for_test(7);
        let (first_pub_tx, mut first_pub_rx) = publisher_event_channel();
        let (second_pub_tx, _second_pub_rx) = publisher_event_channel();
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(local_id), channel, first_pub_tx))
            .await
            .expect("first publisher should be registered");
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(local_id), channel, second_pub_tx))
            .await
            .expect("duplicate publisher id should be handled without detaching the first handle");
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        assert_eq!(
            first_pub_rx.try_recv(),
            Ok(PublisherEvent::PeerJoined(PeerSrc::Local)),
            "a duplicate local publisher id must not silently replace an existing live publisher handle"
        );
    }

    #[tokio::test]
    async fn duplicate_subscriber_local_id_must_not_detach_live_handle() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let local_id = SubscriberLocalId::from_raw_for_test(7);
        let (first_sub_tx, mut first_sub_rx) = subscriber_event_channel();
        let (second_sub_tx, _second_sub_rx) = subscriber_event_channel();
        let (pub_tx, _pub_rx) = publisher_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(local_id), channel, first_sub_tx))
            .await
            .expect("first subscriber should be registered");
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(local_id), channel, second_sub_tx))
            .await
            .expect("duplicate subscriber id should be handled without detaching the first handle");
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");

        assert_eq!(
            first_sub_rx.try_recv(),
            Ok(SubscriberEvent::PeerJoined(PeerSrc::Local)),
            "a duplicate local subscriber id must not silently replace an existing live subscriber handle"
        );
    }

    #[tokio::test]
    async fn stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, true, 2, false, 2)))
            .await
            .expect("heartbeat should be processed");

        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))));

        let stale_leave = encode_publisher_leaved_for_test(channel, 1);
        service.on_service(P2pServiceEvent::Unicast(remote, stale_leave)).await.expect("stale leave should be processed");

        assert!(
            service.channels.get(&channel).expect("channel should exist").has_remote_publisher(remote),
            "stale PublisherLeaved must not remove a publisher confirmed by a newer heartbeat"
        );
        assert!(sub_rx.try_recv().is_err(), "stale PublisherLeaved must not emit PeerLeaved for a still-live remote publisher");
    }

    #[tokio::test]
    async fn pubsub_restart_with_reset_generation_must_restore_remote_membership() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, true, 2, false, 0)))
            .await
            .expect("old live heartbeat should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))));

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_leaved_for_test(channel, 3)))
            .await
            .expect("old leave should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerLeaved(PeerSrc::Remote(remote))));

        service
            .on_service(P2pServiceEvent::PeerDisconnected(remote))
            .await
            .expect("disconnect should clear remote pubsub tombstone");

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, true, 1, false, 0)))
            .await
            .expect("fresh post-restart heartbeat should be processed");

        assert!(
            service.channels.get(&channel).expect("channel should exist").has_remote_publisher(remote),
            "a restarted peer with reset pubsub generation must be able to restore membership"
        );
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))));
    }

    #[tokio::test]
    async fn pubsub_restart_with_reset_subscriber_generation_must_restore_remote_membership() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);
        let (pub_tx, mut pub_rx) = publisher_event_channel();

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, false, 0, true, 2)))
            .await
            .expect("old live heartbeat should be processed");
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerJoined(PeerSrc::Remote(remote))));

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_leaved_for_test(channel, 3)))
            .await
            .expect("old leave should be processed");
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerLeaved(PeerSrc::Remote(remote))));

        service
            .on_service(P2pServiceEvent::PeerDisconnected(remote))
            .await
            .expect("disconnect should clear remote pubsub tombstone");

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, false, 0, true, 1)))
            .await
            .expect("fresh post-restart heartbeat should be processed");

        assert!(
            service.channels.get(&channel).expect("channel should exist").has_remote_subscriber(remote),
            "a restarted peer with reset pubsub subscriber generation must be able to restore membership"
        );
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerJoined(PeerSrc::Remote(remote))));
    }

    #[tokio::test]
    async fn pubsub_peer_disconnect_must_remove_active_remote_membership() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        let (pub_tx, mut pub_rx) = publisher_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Local)));
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerJoined(PeerSrc::Local)));

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_heartbeat_for_test_with_generation(channel, true, 2, true, 2)))
            .await
            .expect("remote live heartbeat should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))));
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerJoined(PeerSrc::Remote(remote))));

        service.on_service(P2pServiceEvent::PeerDisconnected(remote)).await.expect("disconnect should be processed");

        assert!(
            !service.channels.get(&channel).expect("channel should exist").has_remote_publisher(remote),
            "disconnect must remove remote publisher membership"
        );
        assert!(
            !service.channels.get(&channel).expect("channel should exist").has_remote_subscriber(remote),
            "disconnect must remove remote subscriber membership"
        );
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerLeaved(PeerSrc::Remote(remote))));
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerLeaved(PeerSrc::Remote(remote))));
    }

    #[tokio::test]
    async fn pubsub_rpc_methods_must_be_bounded() {
        const MAX_METHOD_LEN: usize = 1024;
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let from_peer = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        let oversized_method = "m".repeat(MAX_METHOD_LEN + 1);
        let payload = bincode::serialize(&PubsubMessage::PublishRpc(channel, vec![1], RpcId::rand(), oversized_method)).expect("test RPC should serialize");

        service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("publish RPC should be processed");

        let event = sub_rx.try_recv().expect("subscriber should receive publish RPC");
        match event {
            SubscriberEvent::PublishRpc(_, _, method, PeerSrc::Remote(_)) => assert!(method.len() <= MAX_METHOD_LEN, "pubsub RPC method names must be bounded, got {} bytes", method.len()),
            other => panic!("expected PublishRpc event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn pubsub_guest_object_publish_must_return_error_on_serialize_failure() {
        let requester = test_service().requester();

        let result = std::panic::AssertUnwindSafe(requester.publish_as_guest_ob(PubsubChannelId(1), FailingSerialize)).catch_unwind().await;

        assert!(
            matches!(result, Ok(Err(_))),
            "object publish helpers must return serialization errors instead of panicking through the public API"
        );
    }
}
