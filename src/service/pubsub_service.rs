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
    time::{sleep_until, Interval},
};

use crate::{ErrorExt, PeerId};

use super::{P2pService, P2pServiceEvent};

mod publisher;
mod subscriber;

pub use publisher::{Publisher, PublisherEvent, PublisherEventOb, PublisherRequester};
pub use subscriber::{Subscriber, SubscriberEvent, SubscriberEventOb, SubscriberRequester};

const HEARTBEAT_INTERVAL_MS: u64 = 5_000;
pub(crate) const PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE: usize = 1024;
const MAX_REMOTE_CREATED_CHANNELS: usize = 1024;
const MAX_REMOTE_ROLE_TOMBSTONES: usize = MAX_REMOTE_CREATED_CHANNELS * 2;
const MAX_REMOTE_MEMBERS_PER_CHANNEL: usize = 1024;
const MAX_HEARTBEAT_CHANNELS_PER_BATCH: usize = 1024;
const MAX_RPC_METHOD_LEN: usize = 1024;
const MAX_PENDING_RPC_REQUESTS: usize = 1024;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[error("TooManyPendingRequests")]
    TooManyPendingRequests,
}

struct PublishRpcReq {
    started_at: Instant,
    timeout: Duration,
    expected_responders: HashSet<PeerSrc>,
    expected_local_subscribers: HashSet<SubscriberHandleId>,
    tx: Option<oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>>,
}

impl PublishRpcReq {
    fn deadline(&self) -> Option<Instant> {
        self.started_at.checked_add(self.timeout)
    }
}

struct FeedbackRpcReq {
    started_at: Instant,
    timeout: Duration,
    expected_responders: HashSet<PeerSrc>,
    expected_local_publishers: HashSet<PublisherHandleId>,
    tx: Option<oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>>,
}

impl FeedbackRpcReq {
    fn deadline(&self) -> Option<Instant> {
        self.started_at.checked_add(self.timeout)
    }
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
    HeartbeatChunk { channels: Vec<ChannelHeartbeat>, is_last: bool },
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
pub(crate) fn encode_feedback_for_test(channel: PubsubChannelId, data: Vec<u8>) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::Feedback(channel, data)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_publish_rpc_for_test(channel: PubsubChannelId, data: Vec<u8>, rpc_id: RpcId, method: String) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::PublishRpc(channel, data, rpc_id, method)).expect("test message should serialize")
}

#[cfg(test)]
pub(crate) fn encode_feedback_rpc_for_test(channel: PubsubChannelId, data: Vec<u8>, rpc_id: RpcId, method: String) -> Vec<u8> {
    bincode::serialize(&PubsubMessage::FeedbackRpc(channel, data, rpc_id, method)).expect("test message should serialize")
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
    PublishRpcAnswer(SubscriberHandleId, RpcId, PeerSrc, Vec<u8>),
    GuestFeedback(PubsubChannelId, Vec<u8>),
    GuestFeedbackRpc(PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    Feedback(SubscriberHandleId, PubsubChannelId, Vec<u8>),
    FeedbackRpc(SubscriberHandleId, PubsubChannelId, Vec<u8>, String, oneshot::Sender<Result<Vec<u8>, PubsubRpcError>>, Duration),
    FeedbackRpcAnswer(PublisherHandleId, RpcId, PeerSrc, Vec<u8>),
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
struct RemoteRoleTombstone {
    publish_generation: u64,
    subscribe_generation: u64,
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
    fn is_fully_empty(&self) -> bool {
        self.local_publishers.is_empty() && self.local_subscribers.is_empty() && self.remote_publishers.is_empty() && self.remote_subscribers.is_empty()
    }

    fn is_inactive_remote_only(&self) -> bool {
        self.local_publishers.is_empty() && self.local_subscribers.is_empty() && !self.has_active_remote_publishers() && !self.has_active_remote_subscribers()
    }

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

    fn apply_remote_publisher_heartbeat(&mut self, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        Self::apply_remote_role_heartbeat(&mut self.remote_publishers, peer, generation, active)
    }

    fn apply_remote_subscriber_heartbeat(&mut self, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        Self::apply_remote_role_heartbeat(&mut self.remote_subscribers, peer, generation, active)
    }

    fn admit_remote_role(map: &mut HashMap<PeerId, RemoteRoleState>, peer: PeerId) -> bool {
        if map.contains_key(&peer) {
            return true;
        }
        if map.len() >= MAX_REMOTE_MEMBERS_PER_CHANNEL {
            map.retain(|_, state| state.active);
        }
        map.len() < MAX_REMOTE_MEMBERS_PER_CHANNEL
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
                if !Self::admit_remote_role(map, peer) {
                    return None;
                }
                map.insert(peer, RemoteRoleState { generation, active });
                Some((false, active))
            }
        }
    }

    fn apply_remote_role_heartbeat(map: &mut HashMap<PeerId, RemoteRoleState>, peer: PeerId, generation: u64, active: bool) -> Option<(bool, bool)> {
        match map.get_mut(&peer) {
            Some(state) if generation < state.generation => None,
            Some(state) => {
                let was_active = state.active;
                state.generation = generation;
                state.active = active;
                Some((was_active, active))
            }
            None => {
                if !Self::admit_remote_role(map, peer) {
                    return None;
                }
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
    remote_role_tombstones: HashMap<(PubsubChannelId, PeerId), RemoteRoleTombstone>,
    pending_heartbeat_chunks: HashMap<PeerId, HashSet<PubsubChannelId>>,
    publish_rpc_reqs: HashMap<RpcId, PublishRpcReq>,
    feedback_rpc_reqs: HashMap<RpcId, FeedbackRpcReq>,
    local_publish_generation: u64,
    local_subscribe_generation: u64,
    heartbeat_tick: Interval,
}

impl PubsubService {
    pub fn new(service: P2pService) -> Self {
        let (internal_tx, internal_rx) = channel(PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE);
        Self {
            service,
            internal_rx,
            internal_tx,
            channels: HashMap::new(),
            remote_role_tombstones: HashMap::new(),
            pending_heartbeat_chunks: HashMap::new(),
            publish_rpc_reqs: HashMap::new(),
            feedback_rpc_reqs: HashMap::new(),
            local_publish_generation: 0,
            local_subscribe_generation: 0,
            heartbeat_tick: tokio::time::interval(Duration::from_millis(HEARTBEAT_INTERVAL_MS)),
        }
    }

    pub fn requester(&self) -> PubsubServiceRequester {
        PubsubServiceRequester {
            internal_tx: self.internal_tx.clone(),
        }
    }

    fn channel_for_remote_join(&mut self, channel: PubsubChannelId) -> Option<&mut PubsubChannelState> {
        if self.channels.contains_key(&channel) {
            return self.channels.get_mut(&channel);
        }
        if self.channels.len() >= MAX_REMOTE_CREATED_CHANNELS {
            let inactive_channels = self
                .channels
                .iter()
                .filter_map(|(channel, state)| state.is_inactive_remote_only().then_some(*channel))
                .collect::<Vec<_>>();
            for inactive_channel in inactive_channels {
                let Some(state) = self.channels.get(&inactive_channel) else {
                    continue;
                };
                let slots_needed = self.remote_role_tombstone_slots_needed(inactive_channel, state);
                if self.remote_role_tombstones.len() + slots_needed > MAX_REMOTE_ROLE_TOMBSTONES {
                    continue;
                }
                if let Some(state) = self.channels.remove(&inactive_channel) {
                    self.remember_remote_role_tombstones(inactive_channel, state);
                }
            }
            if self.channels.len() >= MAX_REMOTE_CREATED_CHANNELS {
                log::warn!("[PubsubService] drop remote join for {channel}: channel cap {MAX_REMOTE_CREATED_CHANNELS} reached");
                return None;
            }
        }
        Some(self.channels.entry(channel).or_default())
    }

    fn remote_role_tombstone_slots_needed(&self, channel: PubsubChannelId, state: &PubsubChannelState) -> usize {
        let mut peers = HashSet::new();
        for (peer, role) in &state.remote_publishers {
            if !role.active && !self.remote_role_tombstones.contains_key(&(channel, *peer)) {
                peers.insert(*peer);
            }
        }
        for (peer, role) in &state.remote_subscribers {
            if !role.active && !self.remote_role_tombstones.contains_key(&(channel, *peer)) {
                peers.insert(*peer);
            }
        }
        peers.len()
    }

    fn remember_remote_role_tombstones(&mut self, channel: PubsubChannelId, state: PubsubChannelState) {
        for (peer, role) in state.remote_publishers {
            if !role.active {
                self.remember_remote_role_tombstone(channel, peer, Some(role.generation), None);
            }
        }
        for (peer, role) in state.remote_subscribers {
            if !role.active {
                self.remember_remote_role_tombstone(channel, peer, None, Some(role.generation));
            }
        }
    }

    fn remember_remote_role_tombstone(&mut self, channel: PubsubChannelId, peer: PeerId, publish_generation: Option<u64>, subscribe_generation: Option<u64>) {
        if !self.remote_role_tombstones.contains_key(&(channel, peer)) && self.remote_role_tombstones.len() >= MAX_REMOTE_ROLE_TOMBSTONES {
            log::warn!("[PubsubService] drop remote role tombstone for {channel}/{peer}: tombstone cap {MAX_REMOTE_ROLE_TOMBSTONES} reached");
            return;
        }
        let tombstone = self.remote_role_tombstones.entry((channel, peer)).or_default();
        if let Some(generation) = publish_generation {
            tombstone.publish_generation = tombstone.publish_generation.max(generation);
        }
        if let Some(generation) = subscribe_generation {
            tombstone.subscribe_generation = tombstone.subscribe_generation.max(generation);
        }
    }

    fn remote_publisher_join_is_stale(&self, channel: PubsubChannelId, peer: PeerId, generation: u64) -> bool {
        self.remote_role_tombstones.get(&(channel, peer)).is_some_and(|tombstone| tombstone.publish_generation >= generation)
    }

    fn remote_subscriber_join_is_stale(&self, channel: PubsubChannelId, peer: PeerId, generation: u64) -> bool {
        self.remote_role_tombstones.get(&(channel, peer)).is_some_and(|tombstone| tombstone.subscribe_generation >= generation)
    }

    fn clear_remote_publisher_tombstone(&mut self, channel: PubsubChannelId, peer: PeerId) {
        if let Some(tombstone) = self.remote_role_tombstones.get_mut(&(channel, peer)) {
            tombstone.publish_generation = 0;
            if tombstone.subscribe_generation == 0 {
                self.remote_role_tombstones.remove(&(channel, peer));
            }
        }
    }

    fn clear_remote_subscriber_tombstone(&mut self, channel: PubsubChannelId, peer: PeerId) {
        if let Some(tombstone) = self.remote_role_tombstones.get_mut(&(channel, peer)) {
            tombstone.subscribe_generation = 0;
            if tombstone.publish_generation == 0 {
                self.remote_role_tombstones.remove(&(channel, peer));
            }
        }
    }

    fn next_local_publish_generation(&mut self) -> u64 {
        self.local_publish_generation = self.local_publish_generation.saturating_add(1);
        self.local_publish_generation
    }

    fn next_local_subscribe_generation(&mut self) -> u64 {
        self.local_subscribe_generation = self.local_subscribe_generation.saturating_add(1);
        self.local_subscribe_generation
    }

    fn rpc_method_is_allowed(from_peer: PeerId, kind: &str, method: &str) -> bool {
        if method.len() > MAX_RPC_METHOD_LEN {
            log::warn!("[PubsubService] {kind} from {from_peer} has {} byte method, dropping oversized RPC", method.len());
            return false;
        }
        true
    }

    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        loop {
            let rpc_deadline = self.next_rpc_deadline();
            select! {
                _ = self.heartbeat_tick.tick() => {
                    self.on_heartbeat_tick().await?;
                },
                _ = async {
                    match rpc_deadline {
                        Some(deadline) => sleep_until(deadline.into()).await,
                        None => std::future::pending::<()>().await,
                    }
                } => {
                    self.on_rpc_timeout().await?;
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

    fn next_rpc_deadline(&self) -> Option<Instant> {
        self.publish_rpc_reqs
            .values()
            .filter_map(PublishRpcReq::deadline)
            .chain(self.feedback_rpc_reqs.values().filter_map(FeedbackRpcReq::deadline))
            .min()
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
        if heartbeat.len() <= MAX_HEARTBEAT_CHANNELS_PER_BATCH {
            self.broadcast(&PubsubMessage::Heartbeat(heartbeat)).await;
        } else {
            let last_chunk_index = (heartbeat.len() - 1) / MAX_HEARTBEAT_CHANNELS_PER_BATCH;
            for (chunk_index, chunk) in heartbeat.chunks(MAX_HEARTBEAT_CHANNELS_PER_BATCH).enumerate() {
                self.broadcast(&PubsubMessage::HeartbeatChunk {
                    channels: chunk.to_vec(),
                    is_last: chunk_index == last_chunk_index,
                })
                .await;
            }
        }
        Ok(())
    }

    fn apply_heartbeat_rows(&mut self, from_peer: PeerId, channels: Vec<ChannelHeartbeat>) {
        for heartbeat in channels {
            if let Some(state) = self.channels.get_mut(&heartbeat.channel) {
                if let Some((was_active, is_active)) = state.apply_remote_publisher_heartbeat(from_peer, heartbeat.publish_generation, heartbeat.publish) {
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

                if let Some((was_active, is_active)) = state.apply_remote_subscriber_heartbeat(from_peer, heartbeat.subscribe_generation, heartbeat.subscribe) {
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

    fn remove_heartbeat_omissions(&mut self, from_peer: PeerId, seen_channels: &HashSet<PubsubChannelId>) {
        let mut empty_channels = Vec::new();
        for (channel, state) in self.channels.iter_mut() {
            if seen_channels.contains(channel) {
                continue;
            }

            if state.remote_publishers.remove(&from_peer).is_some_and(|role| role.active) {
                log::info!("[PubsubService] remote peer {from_peer} heartbeat omitted {channel} as publisher");
                for sub_tx in state.local_subscribers.values() {
                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PeerLeaved(PeerSrc::Remote(from_peer)));
                }
            }

            if state.remote_subscribers.remove(&from_peer).is_some_and(|role| role.active) {
                log::info!("[PubsubService] remote peer {from_peer} heartbeat omitted {channel} as subscriber");
                for pub_tx in state.local_publishers.values() {
                    Self::try_send_publisher_event(pub_tx, PublisherEvent::PeerLeaved(PeerSrc::Remote(from_peer)));
                }
            }

            if state.is_fully_empty() {
                empty_channels.push(*channel);
            }
        }
        for channel in empty_channels {
            self.channels.remove(&channel);
        }
    }

    async fn on_rpc_timeout(&mut self) -> anyhow::Result<()> {
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
                            let mut admitted = false;
                            if self.remote_publisher_join_is_stale(channel, from_peer, generation) {
                                log::debug!("[PubsubService] ignore stale publisher join from {from_peer} for {channel} generation {generation}");
                            } else {
                                if let Some(state) = self.channel_for_remote_join(channel) {
                                    if matches!(state.apply_remote_publisher(from_peer, generation, true), Some((false, true))) {
                                        admitted = true;
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
                                if admitted {
                                    self.clear_remote_publisher_tombstone(channel, from_peer);
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
                            } else {
                                self.remember_remote_role_tombstone(channel, from_peer, Some(generation), None);
                            }
                        }
                        PubsubMessage::SubscriberJoined(channel, generation) => {
                            let mut reply = None;
                            let mut admitted = false;
                            if self.remote_subscriber_join_is_stale(channel, from_peer, generation) {
                                log::debug!("[PubsubService] ignore stale subscriber join from {from_peer} for {channel} generation {generation}");
                            } else {
                                if let Some(state) = self.channel_for_remote_join(channel) {
                                    if matches!(state.apply_remote_subscriber(from_peer, generation, true), Some((false, true))) {
                                        admitted = true;
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
                                if admitted {
                                    self.clear_remote_subscriber_tombstone(channel, from_peer);
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
                            } else {
                                self.remember_remote_role_tombstone(channel, from_peer, None, Some(generation));
                            }
                        }
                        PubsubMessage::Heartbeat(channels) => {
                            if channels.len() > MAX_HEARTBEAT_CHANNELS_PER_BATCH {
                                log::warn!("[PubsubService] heartbeat from {from_peer} has {} channels, dropping oversized batch", channels.len());
                                return Ok(());
                            }

                            let seen_channels: HashSet<_> = channels.iter().map(|heartbeat| heartbeat.channel).collect();
                            self.pending_heartbeat_chunks.remove(&from_peer);
                            self.apply_heartbeat_rows(from_peer, channels);
                            self.remove_heartbeat_omissions(from_peer, &seen_channels);
                        }
                        PubsubMessage::HeartbeatChunk { channels, is_last } => {
                            if channels.len() > MAX_HEARTBEAT_CHANNELS_PER_BATCH {
                                log::warn!("[PubsubService] heartbeat chunk from {from_peer} has {} channels, dropping oversized batch", channels.len());
                                self.pending_heartbeat_chunks.remove(&from_peer);
                                return Ok(());
                            }

                            {
                                let chunk_seen_channels = channels
                                    .iter()
                                    .filter_map(|heartbeat| self.channels.contains_key(&heartbeat.channel).then_some(heartbeat.channel))
                                    .collect::<Vec<_>>();
                                let seen_channels = self.pending_heartbeat_chunks.entry(from_peer).or_default();
                                seen_channels.extend(chunk_seen_channels);
                            }
                            self.apply_heartbeat_rows(from_peer, channels);

                            if is_last {
                                let seen_channels = self.pending_heartbeat_chunks.remove(&from_peer).unwrap_or_default();
                                self.remove_heartbeat_omissions(from_peer, &seen_channels);
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
                            if !Self::rpc_method_is_allowed(from_peer, "GuestPublishRpc", &method) {
                                return Ok(());
                            }
                            if let Some(state) = self.channels.get(&channel) {
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublishRpc(data.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::Publish(channel, vec) => {
                            if let Some(state) = self.channels.get(&channel) {
                                if !state.has_remote_publisher(from_peer) {
                                    log::warn!("[PubsubService] drop Publish from non-publisher {from_peer} on {channel}");
                                    return Ok(());
                                }
                                for sub_tx in state.local_subscribers.values() {
                                    Self::try_send_subscriber_event(sub_tx, SubscriberEvent::Publish(vec.clone()));
                                }
                            }
                        }
                        PubsubMessage::PublishRpc(channel, vec, rpc_id, method) => {
                            if !Self::rpc_method_is_allowed(from_peer, "PublishRpc", &method) {
                                return Ok(());
                            }
                            if let Some(state) = self.channels.get(&channel) {
                                if !state.has_remote_publisher(from_peer) {
                                    log::warn!("[PubsubService] drop PublishRpc from non-publisher {from_peer} on {channel}");
                                    return Ok(());
                                }
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
                            if !Self::rpc_method_is_allowed(from_peer, "GuestFeedbackRpc", &method) {
                                return Ok(());
                            }
                            if let Some(state) = self.channels.get(&channel) {
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedbackRpc(vec.clone(), rpc_id, method.clone(), PeerSrc::Remote(from_peer)));
                                }
                            }
                        }
                        PubsubMessage::Feedback(channel, vec) => {
                            if let Some(state) = self.channels.get(&channel) {
                                if !state.has_remote_subscriber(from_peer) {
                                    log::warn!("[PubsubService] drop Feedback from non-subscriber {from_peer} on {channel}");
                                    return Ok(());
                                }
                                for (_, pub_tx) in state.local_publishers.iter() {
                                    Self::try_send_publisher_event(pub_tx, PublisherEvent::Feedback(vec.clone()));
                                }
                            }
                        }
                        PubsubMessage::FeedbackRpc(channel, vec, rpc_id, method) => {
                            if !Self::rpc_method_is_allowed(from_peer, "FeedbackRpc", &method) {
                                return Ok(());
                            }
                            if let Some(state) = self.channels.get(&channel) {
                                if !state.has_remote_subscriber(from_peer) {
                                    log::warn!("[PubsubService] drop FeedbackRpc from non-subscriber {from_peer} on {channel}");
                                    return Ok(());
                                }
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
        self.remote_role_tombstones.retain(|(_, tombstone_peer), _| tombstone_peer != &peer);
        self.pending_heartbeat_chunks.remove(&peer);
        let mut empty_channels = Vec::new();
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

            if state.is_fully_empty() {
                empty_channels.push(*channel);
            }
        }
        for channel in empty_channels {
            self.channels.remove(&channel);
        }
    }

    #[allow(clippy::collapsible_else_if)]
    async fn on_internal(&mut self, control: InternalMsg) -> anyhow::Result<()> {
        match control {
            InternalMsg::PublisherCreated(handle_id, channel, tx) => {
                log::info!("[PubsubService] local created pub channel {channel} / {handle_id}");
                let mut should_broadcast = false;
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
                        should_broadcast = true;
                    } else {
                        state.local_publishers.insert(handle_id, tx);
                    }
                }
                if should_broadcast {
                    let generation = self.next_local_publish_generation();
                    if let Some(state) = self.channels.get_mut(&channel) {
                        state.local_publish_generation = generation;
                    }
                    self.broadcast(&PubsubMessage::PublisherJoined(channel, generation)).await;
                }
            }
            InternalMsg::PublisherDestroyed(handle_id, channel) => {
                log::info!("[PubsubService] local destroyed pub channel {channel} / {handle_id}");

                let mut should_broadcast = false;
                let should_prune;
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
                        should_broadcast = true;
                    }
                    should_prune = state.is_fully_empty();
                }
                if should_broadcast {
                    let generation = self.next_local_publish_generation();
                    if let Some(state) = self.channels.get_mut(&channel) {
                        state.local_publish_generation = generation;
                    }
                    self.broadcast(&PubsubMessage::PublisherLeaved(channel, generation)).await;
                }
                if should_prune {
                    self.channels.remove(&channel);
                }
            }
            InternalMsg::SubscriberCreated(handle_id, channel, tx) => {
                log::info!("[PubsubService] local created sub channel {channel} / {handle_id}");
                let mut should_broadcast = false;
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
                        should_broadcast = true;
                    } else {
                        state.local_subscribers.insert(handle_id, tx);
                    }
                }
                if should_broadcast {
                    let generation = self.next_local_subscribe_generation();
                    if let Some(state) = self.channels.get_mut(&channel) {
                        state.local_subscribe_generation = generation;
                    }
                    self.broadcast(&PubsubMessage::SubscriberJoined(channel, generation)).await;
                }
            }
            InternalMsg::SubscriberDestroyed(handle_id, channel) => {
                log::info!("[PubsubService] local destroyed sub channel {channel} / {handle_id}");
                let mut should_broadcast = false;
                let should_prune;
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
                        should_broadcast = true;
                    }
                    should_prune = state.is_fully_empty();
                }
                if should_broadcast {
                    let generation = self.next_local_subscribe_generation();
                    if let Some(state) = self.channels.get_mut(&channel) {
                        state.local_subscribe_generation = generation;
                    }
                    self.broadcast(&PubsubMessage::SubscriberLeaved(channel, generation)).await;
                }
                if should_prune {
                    self.channels.remove(&channel);
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
                    } else if self.publish_rpc_reqs.len() >= MAX_PENDING_RPC_REQUESTS {
                        let _ = tx.send(Err(PubsubRpcError::TooManyPendingRequests));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        let mut expected_local_subscribers = HashSet::new();
                        for (subscriber_handle_id, sub_tx) in state.local_subscribers.iter() {
                            if Self::try_send_subscriber_event(sub_tx, SubscriberEvent::GuestPublishRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                                expected_local_subscribers.insert(*subscriber_handle_id);
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
                                    expected_local_subscribers,
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
                    } else if self.publish_rpc_reqs.len() >= MAX_PENDING_RPC_REQUESTS {
                        let _ = tx.send(Err(PubsubRpcError::TooManyPendingRequests));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        let mut expected_local_subscribers = HashSet::new();
                        for (subscriber_handle_id, sub_tx) in state.local_subscribers.iter() {
                            if Self::try_send_subscriber_event(sub_tx, SubscriberEvent::PublishRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                                expected_local_subscribers.insert(*subscriber_handle_id);
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
                                    expected_local_subscribers,
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
                    } else if self.feedback_rpc_reqs.len() >= MAX_PENDING_RPC_REQUESTS {
                        let _ = tx.send(Err(PubsubRpcError::TooManyPendingRequests));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        let mut expected_local_publishers = HashSet::new();
                        for (publisher_handle_id, pub_tx) in state.local_publishers.iter() {
                            if Self::try_send_publisher_event(pub_tx, PublisherEvent::GuestFeedbackRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                                expected_local_publishers.insert(*publisher_handle_id);
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
                                    expected_local_publishers,
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
                    } else if self.feedback_rpc_reqs.len() >= MAX_PENDING_RPC_REQUESTS {
                        let _ = tx.send(Err(PubsubRpcError::TooManyPendingRequests));
                    } else {
                        let mut delivered = 0;
                        let mut expected_responders = HashSet::new();
                        let mut expected_local_publishers = HashSet::new();
                        for (publisher_handle_id, pub_tx) in state.local_publishers.iter() {
                            if Self::try_send_publisher_event(pub_tx, PublisherEvent::FeedbackRpc(data.clone(), req_id, method.clone(), PeerSrc::Local)) {
                                delivered += 1;
                                expected_responders.insert(PeerSrc::Local);
                                expected_local_publishers.insert(*publisher_handle_id);
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
                                    expected_local_publishers,
                                    tx: Some(tx),
                                },
                            );
                        }
                    }
                } else {
                    let _ = tx.send(Err(PubsubRpcError::NoDestination));
                }
            }
            InternalMsg::PublishRpcAnswer(handle_id, rpc_id, peer_src, data) => {
                if let PeerSrc::Remote(peer) = peer_src {
                    self.try_send_to(peer, &PubsubMessage::PublishRpcAnswer(data, rpc_id)).await;
                } else {
                    if self
                        .publish_rpc_reqs
                        .get(&rpc_id)
                        .is_some_and(|req| req.expected_responders.contains(&PeerSrc::Local) && req.expected_local_subscribers.contains(&handle_id))
                    {
                        let mut req = self.publish_rpc_reqs.remove(&rpc_id).expect("checked pending publish RPC");
                        let _ = req.tx.take().expect("should have req_tx").send(Ok(data));
                    } else if self.publish_rpc_reqs.contains_key(&rpc_id) {
                        log::warn!("[PubsubService] got local PublishRpcAnswer from unexpected responder for req_id {rpc_id}");
                    } else {
                        log::warn!("[PubsubService] got local PublishRpcAnswer with invalid req_id {rpc_id}");
                    }
                }
            }
            InternalMsg::FeedbackRpcAnswer(handle_id, rpc_id, peer_src, data) => {
                if let PeerSrc::Remote(peer) = peer_src {
                    self.try_send_to(peer, &PubsubMessage::FeedbackRpcAnswer(data, rpc_id)).await;
                } else {
                    if self
                        .feedback_rpc_reqs
                        .get(&rpc_id)
                        .is_some_and(|req| req.expected_responders.contains(&PeerSrc::Local) && req.expected_local_publishers.contains(&handle_id))
                    {
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
        let data = bincode::serialize(&ob)?;
        self.publish_as_guest(channel, data).await
    }

    pub async fn publish_rpc_as_guest(&self, channel: PubsubChannelId, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestPublishRpc(channel, data, method.to_owned(), tx, timeout), "publish_rpc_as_guest")?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn publish_rpc_as_guest_ob<REQ: Serialize, RES: DeserializeOwned>(&self, channel: PubsubChannelId, method: &str, req: REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(&req)?;
        let res = self.publish_rpc_as_guest(channel, method, data, timeout).await?;
        Ok(bincode::deserialize(&res)?)
    }

    pub async fn feedback_as_guest(&self, channel: PubsubChannelId, data: Vec<u8>) -> anyhow::Result<()> {
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestFeedback(channel, data), "feedback_as_guest")?;
        Ok(())
    }

    pub async fn feedback_as_guest_ob<Ob: Serialize>(&self, channel: PubsubChannelId, ob: Ob) -> anyhow::Result<()> {
        let data = bincode::serialize(&ob)?;
        self.feedback_as_guest(channel, data).await
    }

    pub async fn feedback_rpc_as_guest(&self, channel: PubsubChannelId, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(&self.internal_tx, InternalMsg::GuestFeedbackRpc(channel, data, method.to_owned(), tx, timeout), "feedback_rpc_as_guest")?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn feedback_rpc_as_guest_ob<REQ: Serialize, RES: DeserializeOwned>(&self, channel: PubsubChannelId, method: &str, req: REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(&req)?;
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
    use crate::{
        ctx::SharedCtx,
        msg::{P2pServiceId, PeerMessage},
        peer::{test_congested_peer_alias, test_peer_alias},
        router::SharedRouterTable,
        ConnectionId,
    };

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
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for _ in 0..=MAX_PENDING_RPC_REQUESTS {
            let (tx, _rx) = oneshot::channel();
            service
                .on_internal(InternalMsg::GuestPublishRpc(channel, vec![1], "hold".to_string(), tx, Duration::from_secs(3600)))
                .await
                .expect("publish RPC should be accepted");
        }

        let pending_rpcs = service.publish_rpc_reqs.len();
        assert!(pending_rpcs <= MAX_PENDING_RPC_REQUESTS, "pending publish RPC requests must be bounded, got {pending_rpcs}");
    }

    #[tokio::test]
    async fn pubsub_rpc_huge_timeout_must_not_panic_deadline_calculation() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();
        let (tx, _rx) = oneshot::channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        service
            .on_internal(InternalMsg::GuestPublishRpc(channel, vec![1], "hold".to_string(), tx, Duration::MAX))
            .await
            .expect("publish RPC should process");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service.next_rpc_deadline()));

        assert!(result.is_ok(), "caller-supplied huge pubsub RPC timeouts must not panic service deadline calculation");
    }

    #[tokio::test]
    async fn pending_publish_rpc_requests_must_be_bounded_with_draining_local_subscriber() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        let mut rxs = Vec::new();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for idx in 0..=MAX_PENDING_RPC_REQUESTS {
            let (tx, rx) = oneshot::channel();
            rxs.push(rx);

            service
                .on_internal(InternalMsg::GuestPublishRpc(channel, vec![1], "hold".to_string(), tx, Duration::from_secs(3600)))
                .await
                .expect("publish RPC should process");

            if idx < MAX_PENDING_RPC_REQUESTS {
                sub_rx.try_recv().expect("subscriber queue should be drained");
            }
        }

        let pending_rpcs = service.publish_rpc_reqs.len();
        assert_eq!(rxs.pop().expect("overflow request should have a response").try_recv(), Ok(Err(PubsubRpcError::TooManyPendingRequests)));
        assert_eq!(
            sub_rx.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty),
            "rejected RPC must not enqueue an orphan subscriber event"
        );
        assert!(pending_rpcs <= MAX_PENDING_RPC_REQUESTS, "pending publish RPC requests must be bounded, got {pending_rpcs}");
    }

    #[tokio::test]
    async fn pending_feedback_rpc_requests_must_be_bounded_with_draining_local_publisher() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let subscriber = subscriber_handle(SubscriberLocalId::rand());
        let (pub_tx, mut pub_rx) = publisher_event_channel();
        let (sub_tx, _sub_rx) = subscriber_event_channel();
        let mut rxs = Vec::new();

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber, channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        pub_rx.try_recv().expect("publisher should observe initial local subscriber");

        for idx in 0..=MAX_PENDING_RPC_REQUESTS {
            let (tx, rx) = oneshot::channel();
            rxs.push(rx);

            service
                .on_internal(InternalMsg::FeedbackRpc(subscriber, channel, vec![1], "hold".to_string(), tx, Duration::from_secs(3600)))
                .await
                .expect("feedback RPC should process");

            if idx < MAX_PENDING_RPC_REQUESTS {
                pub_rx.try_recv().expect("publisher queue should be drained");
            }
        }

        let pending_rpcs = service.feedback_rpc_reqs.len();
        assert_eq!(rxs.pop().expect("overflow request should have a response").try_recv(), Ok(Err(PubsubRpcError::TooManyPendingRequests)));
        assert_eq!(
            pub_rx.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty),
            "rejected RPC must not enqueue an orphan publisher event"
        );
        assert!(pending_rpcs <= MAX_PENDING_RPC_REQUESTS, "pending feedback RPC requests must be bounded, got {pending_rpcs}");
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
            service.on_internal(InternalMsg::PublishRpcAnswer(
                subscriber_handle(SubscriberLocalId::rand()),
                RpcId(7),
                PeerSrc::Remote(remote),
                b"answer".to_vec(),
            )),
        )
        .await;

        assert!(result.is_ok(), "remote pubsub RPC answers must not block the service loop behind a full peer-control queue");
    }

    #[tokio::test]
    async fn remote_publisher_memberships_must_be_bounded() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();
        let joined = bincode::serialize(&PubsubMessage::PublisherJoined(channel, 1)).expect("test message should serialize");

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for peer in 0..=MAX_REMOTE_MEMBERS_PER_CHANNEL {
            service
                .on_service(P2pServiceEvent::Unicast(PeerId::from(peer as u64 + 10), joined.clone()))
                .await
                .expect("remote publisher join should be processed");
        }

        let remote_publishers = service.channels.get(&channel).expect("channel should exist").active_remote_publishers_count();
        assert!(
            remote_publishers <= MAX_REMOTE_MEMBERS_PER_CHANNEL,
            "remote publisher memberships must be bounded, got {remote_publishers}"
        );
    }

    #[tokio::test]
    async fn remote_subscriber_memberships_must_be_bounded() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (pub_tx, _pub_rx) = publisher_event_channel();
        let joined = bincode::serialize(&PubsubMessage::SubscriberJoined(channel, 1)).expect("test message should serialize");

        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");

        for peer in 0..=MAX_REMOTE_MEMBERS_PER_CHANNEL {
            service
                .on_service(P2pServiceEvent::Unicast(PeerId::from(peer as u64 + 10), joined.clone()))
                .await
                .expect("remote subscriber join should be processed");
        }

        let remote_subscribers = service.channels.get(&channel).expect("channel should exist").active_remote_subscribers().count();
        assert!(
            remote_subscribers <= MAX_REMOTE_MEMBERS_PER_CHANNEL,
            "remote subscriber memberships must be bounded, got {remote_subscribers}"
        );
    }

    #[test]
    fn remote_membership_cap_must_allow_existing_peer_updates() {
        let mut state = PubsubChannelState::default();

        for peer in 0..MAX_REMOTE_MEMBERS_PER_CHANNEL {
            assert_eq!(state.apply_remote_publisher(PeerId::from(peer as u64 + 10), 1, true), Some((false, true)));
        }

        let existing = PeerId::from(10);
        assert_eq!(state.apply_remote_publisher(existing, 2, false), Some((true, false)));
        assert!(!state.has_remote_publisher(existing), "existing peer leave must still apply when membership map is full");
    }

    #[test]
    fn remote_membership_cap_must_evict_inactive_entries_before_rejecting_new_peer() {
        let mut state = PubsubChannelState::default();

        for peer in 0..MAX_REMOTE_MEMBERS_PER_CHANNEL {
            let active = peer != 0;
            assert_eq!(state.apply_remote_publisher(PeerId::from(peer as u64 + 10), 1, active), Some((false, active)));
        }

        let new_peer = PeerId::from(20_000);
        assert_eq!(state.apply_remote_publisher(new_peer, 1, true), Some((false, true)));
        assert!(state.has_remote_publisher(new_peer), "new peer should be admitted after pruning inactive entries");
        assert!(state.remote_publishers.len() <= MAX_REMOTE_MEMBERS_PER_CHANNEL);
    }

    #[tokio::test]
    async fn remote_heartbeat_memberships_must_be_bounded() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let (sub_tx, _sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        for peer in 0..=MAX_REMOTE_MEMBERS_PER_CHANNEL {
            service
                .on_service(P2pServiceEvent::Unicast(
                    PeerId::from(peer as u64 + 10),
                    encode_heartbeat_for_test_with_generation(channel, true, 1, false, 0),
                ))
                .await
                .expect("remote heartbeat should be processed");
        }

        let remote_publishers = service.channels.get(&channel).expect("channel should exist").active_remote_publishers_count();
        assert!(
            remote_publishers <= MAX_REMOTE_MEMBERS_PER_CHANNEL,
            "heartbeat publisher memberships must be bounded, got {remote_publishers}"
        );
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
    async fn early_remote_subscriber_join_must_survive_late_local_publisher_creation() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_joined_for_test(channel)))
            .await
            .expect("early remote subscriber join should be processed");

        let (pub_tx, mut pub_rx) = publisher_event_channel();
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("late publisher should be registered");

        assert!(
            service.channels.get(&channel).expect("channel should exist after publisher creation").has_remote_subscriber(remote),
            "remote subscriber join received before local channel creation must be retained"
        );
        assert_eq!(
            pub_rx.try_recv(),
            Ok(PublisherEvent::PeerJoined(PeerSrc::Remote(remote))),
            "late local publisher must observe the already-joined remote subscriber"
        );
    }

    #[tokio::test]
    async fn remote_created_channel_cap_must_recover_after_disconnect_prunes_empty_channels() {
        let mut service = test_service();
        let remote = PeerId::from(2);

        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(PubsubChannelId(channel as u64 + 10))))
                .await
                .expect("early remote publisher join should be processed");
        }
        assert_eq!(service.channels.len(), MAX_REMOTE_CREATED_CHANNELS);

        service
            .on_service(P2pServiceEvent::PeerDisconnected(remote))
            .await
            .expect("disconnect should prune remote-only channels");
        assert_eq!(service.channels.len(), 0, "remote-only channel state must be reclaimed after disconnect");

        let channel = PubsubChannelId(999_999);
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("new early remote publisher join should be processed after pruning");

        assert!(
            service.channels.get(&channel).expect("new channel should be retained").has_remote_publisher(remote),
            "remote-created channel cap must not stay exhausted after disconnect cleanup"
        );
    }

    #[tokio::test]
    async fn remote_created_channel_cap_must_recover_after_remote_leaves_make_channels_inactive() {
        let mut service = test_service();
        let remote = PeerId::from(2);

        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
                .await
                .expect("early remote publisher join should be processed");
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_leaved_for_test(channel, 2)))
                .await
                .expect("remote publisher leave should be processed");
        }
        assert_eq!(service.channels.len(), MAX_REMOTE_CREATED_CHANNELS);

        let channel = PubsubChannelId(999_999);
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("new early remote publisher join should be processed after inactive-channel reclamation");

        assert!(
            service.channels.get(&channel).expect("new channel should be retained").has_remote_publisher(remote),
            "remote-created channel cap must reclaim inactive remote-only channels under pressure"
        );
        assert!(service.channels.len() <= MAX_REMOTE_CREATED_CHANNELS);
    }

    #[tokio::test]
    async fn remote_created_channel_cap_must_recover_after_inactive_heartbeats() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let mut inactive_heartbeats = Vec::new();

        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
                .await
                .expect("early remote publisher join should be processed");
            inactive_heartbeats.push(ChannelHeartbeat {
                channel,
                publish: false,
                publish_generation: 2,
                subscribe: false,
                subscribe_generation: 1,
            });
        }
        let inactive = bincode::serialize(&PubsubMessage::Heartbeat(inactive_heartbeats)).expect("heartbeat should serialize");
        service.on_service(P2pServiceEvent::Unicast(remote, inactive)).await.expect("inactive heartbeat should be processed");
        assert_eq!(service.channels.len(), MAX_REMOTE_CREATED_CHANNELS);

        let channel = PubsubChannelId(999_999);
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("new early remote publisher join should be processed after inactive-channel reclamation");

        assert!(
            service.channels.get(&channel).expect("new channel should be retained").has_remote_publisher(remote),
            "remote-created channel cap must reclaim heartbeat-inactivated remote-only channels under pressure"
        );
        assert!(service.channels.len() <= MAX_REMOTE_CREATED_CHANNELS);
    }

    #[tokio::test]
    async fn reclaimed_remote_publisher_tombstone_must_reject_stale_join() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let stale_channel = PubsubChannelId(10);

        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
                .await
                .expect("early remote publisher join should be processed");
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_leaved_for_test(channel, 2)))
                .await
                .expect("remote publisher leave should be processed");
        }

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(PubsubChannelId(999_999))))
            .await
            .expect("cap pressure should reclaim inactive remote-only channels");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(stale_channel)))
            .await
            .expect("stale publisher join should be ignored");

        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), stale_channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        assert!(
            !service.channels.get(&stale_channel).expect("subscriber should create channel").has_remote_publisher(remote),
            "stale publisher join must not resurrect after inactive tombstone reclamation"
        );
        assert!(sub_rx.try_recv().is_err(), "stale publisher join must not notify a later local subscriber");
    }

    #[tokio::test]
    async fn reclaimed_remote_subscriber_tombstone_must_reject_stale_join() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let stale_channel = PubsubChannelId(10);

        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_joined_for_test(channel)))
                .await
                .expect("early remote subscriber join should be processed");
            service
                .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_leaved_for_test(channel, 2)))
                .await
                .expect("remote subscriber leave should be processed");
        }

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_joined_for_test(PubsubChannelId(999_999))))
            .await
            .expect("cap pressure should reclaim inactive remote-only channels");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_joined_for_test(stale_channel)))
            .await
            .expect("stale subscriber join should be ignored");

        let (pub_tx, mut pub_rx) = publisher_event_channel();
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), stale_channel, pub_tx))
            .await
            .expect("publisher should be registered");

        assert!(
            !service.channels.get(&stale_channel).expect("publisher should create channel").has_remote_subscriber(remote),
            "stale subscriber join must not resurrect after inactive tombstone reclamation"
        );
        assert!(pub_rx.try_recv().is_err(), "stale subscriber join must not notify a later local publisher");
    }

    #[tokio::test]
    async fn unknown_publisher_leave_must_tombstone_stale_join() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let channel = PubsubChannelId(10);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_leaved_for_test(channel, 2)))
            .await
            .expect("out-of-order remote publisher leave should be processed");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("stale publisher join should be ignored");

        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        assert!(
            !service.channels.get(&channel).expect("subscriber should create channel").has_remote_publisher(remote),
            "a newer leave for an unknown channel must tombstone and reject delayed older publisher joins"
        );
        assert!(sub_rx.try_recv().is_err(), "stale publisher join must not notify a later local subscriber");
    }

    #[tokio::test]
    async fn unknown_subscriber_leave_must_tombstone_stale_join() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let channel = PubsubChannelId(10);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_leaved_for_test(channel, 2)))
            .await
            .expect("out-of-order remote subscriber leave should be processed");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_subscriber_joined_for_test(channel)))
            .await
            .expect("stale subscriber join should be ignored");

        let (pub_tx, mut pub_rx) = publisher_event_channel();
        service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), channel, pub_tx))
            .await
            .expect("publisher should be registered");

        assert!(
            !service.channels.get(&channel).expect("publisher should create channel").has_remote_subscriber(remote),
            "a newer leave for an unknown channel must tombstone and reject delayed older subscriber joins"
        );
        assert!(pub_rx.try_recv().is_err(), "stale subscriber join must not notify a later local publisher");
    }

    #[tokio::test]
    async fn tombstone_must_survive_newer_join_dropped_by_channel_cap() {
        let mut service = test_service();
        let remote = PeerId::from(2);
        let stale_channel = PubsubChannelId(10);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(stale_channel)))
            .await
            .expect("early remote publisher join should be processed");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_leaved_for_test(stale_channel, 2)))
            .await
            .expect("remote publisher leave should be processed");
        for channel in 0..MAX_REMOTE_CREATED_CHANNELS {
            let handle = subscriber_handle(SubscriberLocalId::rand());
            let (sub_tx, _sub_rx) = subscriber_event_channel();
            service
                .on_internal(InternalMsg::SubscriberCreated(handle, PubsubChannelId(channel as u64 + 10_000), sub_tx))
                .await
                .expect("local subscriber should make channel non-reclaimable");
        }
        assert!(service.channels.len() > MAX_REMOTE_CREATED_CHANNELS);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(PubsubChannelId(888_888))))
            .await
            .expect("cap pressure should reclaim stale inactive channel");
        assert!(
            !service.channels.contains_key(&stale_channel),
            "cap pressure should reclaim the inactive remote-only channel into a tombstone"
        );

        let newer_join = bincode::serialize(&PubsubMessage::PublisherJoined(stale_channel, 3)).expect("publisher join should serialize");
        service
            .on_service(P2pServiceEvent::Unicast(remote, newer_join))
            .await
            .expect("newer join should be dropped but not clear tombstone");

        let removable = service.channels.keys().copied().find(|channel| *channel != stale_channel).expect("local channels should exist");
        service.channels.remove(&removable);

        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(stale_channel)))
            .await
            .expect("stale publisher join should be ignored after dropped newer join");
        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), stale_channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        assert!(
            !service.channels.get(&stale_channel).expect("subscriber should create channel").has_remote_publisher(remote),
            "dropped newer join must not clear tombstone and allow stale publisher resurrection"
        );
        assert!(sub_rx.try_recv().is_err(), "stale publisher join must not notify a later local subscriber");
    }

    #[tokio::test]
    async fn inactive_channel_must_not_be_reclaimed_when_tombstone_cap_would_drop_generations() {
        let mut service = test_service();
        let inactive_channels = [PubsubChannelId(10), PubsubChannelId(11), PubsubChannelId(12)];

        for channel in inactive_channels {
            for peer in 0..MAX_REMOTE_MEMBERS_PER_CHANNEL {
                let peer = PeerId::from(peer as u64 + channel.0 * 10_000 + 1);
                service
                    .on_service(P2pServiceEvent::Unicast(peer, encode_publisher_joined_for_test(channel)))
                    .await
                    .expect("early remote publisher join should be processed");
                service
                    .on_service(P2pServiceEvent::Unicast(peer, encode_publisher_leaved_for_test(channel, 2)))
                    .await
                    .expect("remote publisher leave should be processed");
            }
        }

        for channel in 0..(MAX_REMOTE_CREATED_CHANNELS - 3) {
            let (sub_tx, _sub_rx) = subscriber_event_channel();
            service
                .on_internal(InternalMsg::SubscriberCreated(
                    subscriber_handle(SubscriberLocalId::rand()),
                    PubsubChannelId(channel as u64 + 100_000),
                    sub_tx,
                ))
                .await
                .expect("local subscriber should make channel non-reclaimable");
        }
        assert_eq!(service.channels.len(), MAX_REMOTE_CREATED_CHANNELS);

        service
            .on_service(P2pServiceEvent::Unicast(PeerId::from(99_999), encode_publisher_joined_for_test(PubsubChannelId(999_999))))
            .await
            .expect("cap pressure should reclaim only channels whose tombstones fit");

        let protected_channel = inactive_channels
            .into_iter()
            .find(|channel| service.channels.contains_key(channel))
            .expect("at least one inactive channel should remain when tombstone cap would overflow");
        let protected_peer = PeerId::from(protected_channel.0 * 10_000 + 1);
        assert!(
            service.channels.contains_key(&protected_channel),
            "inactive channel must stay resident when reclaiming it would drop stale-generation tombstones"
        );
        service
            .on_service(P2pServiceEvent::Unicast(protected_peer, encode_publisher_joined_for_test(protected_channel)))
            .await
            .expect("stale publisher join should be rejected by retained channel state");
        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), protected_channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        assert!(
            !service.channels.get(&protected_channel).expect("protected channel should remain").has_remote_publisher(protected_peer),
            "retained inactive channel must continue rejecting delayed stale joins after tombstone cap pressure"
        );
        assert!(sub_rx.try_recv().is_err(), "stale publisher join must not notify a later local subscriber");
    }

    #[tokio::test]
    async fn pubsub_heartbeat_channel_batches_must_be_bounded() {
        let mut service = test_service();
        let from_peer = PeerId::from(2);
        let mut heartbeats = Vec::new();

        for channel in 0..=MAX_HEARTBEAT_CHANNELS_PER_BATCH {
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

        assert_eq!(updated_channels, 0, "oversized pubsub heartbeat batches must be dropped, updated {updated_channels} channels");
    }

    #[tokio::test]
    async fn pubsub_heartbeat_channel_batch_at_cap_must_be_accepted() {
        let mut service = test_service();
        let from_peer = PeerId::from(2);
        let mut heartbeats = Vec::new();

        for channel in 0..MAX_HEARTBEAT_CHANNELS_PER_BATCH {
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

        assert_eq!(updated_channels, MAX_HEARTBEAT_CHANNELS_PER_BATCH, "pubsub heartbeat batches at the cap must be accepted");
    }

    #[tokio::test]
    async fn pubsub_outbound_heartbeat_batches_must_respect_inbound_cap() {
        let local = PeerId::from(1);
        let remote = PeerId::from(2);
        let conn = ConnectionId::from(7);
        let mut ctx = SharedCtx::new(local, SharedRouterTable::new(local));
        let service_id = P2pServiceId::from(0);
        let (mut base_service, service_tx) = P2pService::build(service_id, ctx.clone());
        base_service.set_registered(ctx.set_service(service_id, service_tx));
        let mut service = PubsubService::new(base_service);

        for idx in 0..=MAX_HEARTBEAT_CHANNELS_PER_BATCH {
            let channel = PubsubChannelId(idx as u64 + 10);
            let (sub_tx, _sub_rx) = subscriber_event_channel();
            service
                .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
                .await
                .expect("subscriber should be registered");
        }

        let mut peer = test_peer_alias(local, remote, conn);
        ctx.register_conn(conn, peer.alias());

        service.on_heartbeat_tick().await.expect("heartbeat tick should process");

        let mut total_rows = 0;
        let mut batches = 0;
        while let Ok(Some(PeerMessage::Broadcast(_, _, _, data))) = tokio::time::timeout(Duration::from_millis(20), peer.recv_msg()).await {
            let (rows, _is_last) = match bincode::deserialize(&data).expect("heartbeat should deserialize") {
                PubsubMessage::Heartbeat(rows) => (rows, true),
                PubsubMessage::HeartbeatChunk { channels, is_last } => (channels, is_last),
                _ => panic!("heartbeat broadcast must contain a pubsub heartbeat"),
            };
            assert!(
                rows.len() <= MAX_HEARTBEAT_CHANNELS_PER_BATCH,
                "outbound pubsub heartbeats must not exceed the inbound batch cap, got {} rows",
                rows.len()
            );
            total_rows += rows.len();
            batches += 1;
        }

        assert!(batches >= 2, "outbound heartbeat should be split into multiple capped batches");
        assert_eq!(total_rows, MAX_HEARTBEAT_CHANNELS_PER_BATCH + 1, "chunked heartbeat broadcasts must preserve every local channel row");
    }

    #[tokio::test]
    async fn pubsub_chunked_heartbeat_must_not_remove_roles_from_previous_chunk() {
        let mut service = test_service();
        let from_peer = PeerId::from(2);
        let mut heartbeats = Vec::new();

        for idx in 0..=MAX_HEARTBEAT_CHANNELS_PER_BATCH {
            let channel = PubsubChannelId(idx as u64 + 10);
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

        let last_chunk_index = (heartbeats.len() - 1) / MAX_HEARTBEAT_CHANNELS_PER_BATCH;
        for (chunk_index, chunk) in heartbeats.chunks(MAX_HEARTBEAT_CHANNELS_PER_BATCH).enumerate() {
            let payload = bincode::serialize(&PubsubMessage::HeartbeatChunk {
                channels: chunk.to_vec(),
                is_last: chunk_index == last_chunk_index,
            })
            .expect("test heartbeat should serialize");
            service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("heartbeat chunk should be processed");
        }

        let updated_channels = service.channels.values().filter(|state| state.has_remote_publisher(from_peer)).count();

        assert_eq!(
            updated_channels,
            MAX_HEARTBEAT_CHANNELS_PER_BATCH + 1,
            "valid chunked heartbeat batches from one peer must preserve roles learned from earlier chunks"
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

        let empty_channels = service.channels.values().filter(|state| state.is_fully_empty()).count();

        assert_eq!(empty_channels, 0, "empty pubsub channel state must be removed after last local handle drops");
    }

    #[tokio::test]
    async fn empty_pubsub_publisher_channels_must_be_removed_after_last_local_handle_drops() {
        const MAX_EMPTY_CHANNELS: usize = 1024;
        let mut service = test_service();

        for channel in 0..=MAX_EMPTY_CHANNELS {
            let channel = PubsubChannelId(channel as u64 + 10);
            let handle_id = publisher_handle(PublisherLocalId::rand());
            let (pub_tx, _pub_rx) = publisher_event_channel();

            service
                .on_internal(InternalMsg::PublisherCreated(handle_id, channel, pub_tx))
                .await
                .expect("publisher should be registered");
            service.on_internal(InternalMsg::PublisherDestroyed(handle_id, channel)).await.expect("publisher should be destroyed");
        }

        let empty_channels = service.channels.values().filter(|state| state.is_fully_empty()).count();

        assert_eq!(empty_channels, 0, "empty publisher-only pubsub channel state must be removed after last local handle drops");
    }

    #[tokio::test]
    async fn pubsub_prune_must_preserve_channels_with_remote_state() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let remote = PeerId::from(2);
        let handle_id = subscriber_handle(SubscriberLocalId::rand());
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(handle_id, channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        service
            .on_service(P2pServiceEvent::Unicast(remote, encode_publisher_joined_for_test(channel)))
            .await
            .expect("remote publisher join should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(remote))));

        service.on_internal(InternalMsg::SubscriberDestroyed(handle_id, channel)).await.expect("subscriber should be destroyed");

        assert!(
            service.channels.get(&channel).is_some_and(|state| state.has_remote_publisher(remote)),
            "destroying the last local handle must not prune retained remote publisher state"
        );
    }

    #[tokio::test]
    async fn pubsub_recreate_after_prune_must_use_newer_publisher_generation() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let first_handle = publisher_handle(PublisherLocalId::rand());
        let (first_pub_tx, _first_pub_rx) = publisher_event_channel();

        service
            .on_internal(InternalMsg::PublisherCreated(first_handle, channel, first_pub_tx))
            .await
            .expect("publisher should be registered");
        let first_generation = service.channels.get(&channel).expect("channel should exist").local_publish_generation;

        service
            .on_internal(InternalMsg::PublisherDestroyed(first_handle, channel))
            .await
            .expect("publisher should be destroyed");
        let leave_generation = service.local_publish_generation;
        assert!(!service.channels.contains_key(&channel), "empty channel should be pruned after the last local publisher is destroyed");

        let second_handle = publisher_handle(PublisherLocalId::rand());
        let (second_pub_tx, _second_pub_rx) = publisher_event_channel();
        service
            .on_internal(InternalMsg::PublisherCreated(second_handle, channel, second_pub_tx))
            .await
            .expect("publisher should be recreated");
        let recreated_generation = service.channels.get(&channel).expect("channel should exist").local_publish_generation;

        assert!(leave_generation > first_generation);
        assert!(
            recreated_generation > leave_generation,
            "recreated publisher join generation must be newer than the pruned channel's leave generation"
        );
    }

    #[tokio::test]
    async fn pubsub_recreate_after_prune_must_use_newer_subscriber_generation() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let first_handle = subscriber_handle(SubscriberLocalId::rand());
        let (first_sub_tx, _first_sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(first_handle, channel, first_sub_tx))
            .await
            .expect("subscriber should be registered");
        let first_generation = service.channels.get(&channel).expect("channel should exist").local_subscribe_generation;

        service
            .on_internal(InternalMsg::SubscriberDestroyed(first_handle, channel))
            .await
            .expect("subscriber should be destroyed");
        let leave_generation = service.local_subscribe_generation;
        assert!(!service.channels.contains_key(&channel), "empty channel should be pruned after the last local subscriber is destroyed");

        let second_handle = subscriber_handle(SubscriberLocalId::rand());
        let (second_sub_tx, _second_sub_rx) = subscriber_event_channel();
        service
            .on_internal(InternalMsg::SubscriberCreated(second_handle, channel, second_sub_tx))
            .await
            .expect("subscriber should be recreated");
        let recreated_generation = service.channels.get(&channel).expect("channel should exist").local_subscribe_generation;

        assert!(leave_generation > first_generation);
        assert!(
            recreated_generation > leave_generation,
            "recreated subscriber join generation must be newer than the pruned channel's leave generation"
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
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let from_peer = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        service
            .on_service(P2pServiceEvent::Unicast(from_peer, encode_publisher_joined_for_test(channel)))
            .await
            .expect("publisher join should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(from_peer))));

        let oversized_method = "m".repeat(MAX_RPC_METHOD_LEN + 1);
        let payload = bincode::serialize(&PubsubMessage::PublishRpc(channel, vec![1], RpcId::rand(), oversized_method)).expect("test RPC should serialize");

        service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("publish RPC should be processed");

        assert!(sub_rx.try_recv().is_err(), "oversized pubsub RPC methods must be dropped");
    }

    #[tokio::test]
    async fn pubsub_rpc_method_at_cap_must_be_accepted() {
        let mut service = test_service();
        let channel = PubsubChannelId(1);
        let from_peer = PeerId::from(2);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();

        service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), channel, sub_tx))
            .await
            .expect("subscriber should be registered");

        service
            .on_service(P2pServiceEvent::Unicast(from_peer, encode_publisher_joined_for_test(channel)))
            .await
            .expect("publisher join should be processed");
        assert_eq!(sub_rx.try_recv(), Ok(SubscriberEvent::PeerJoined(PeerSrc::Remote(from_peer))));

        let method = "m".repeat(MAX_RPC_METHOD_LEN);
        let payload = bincode::serialize(&PubsubMessage::PublishRpc(channel, vec![1], RpcId::rand(), method)).expect("test RPC should serialize");

        service.on_service(P2pServiceEvent::Unicast(from_peer, payload)).await.expect("publish RPC should be processed");

        let event = sub_rx.try_recv().expect("subscriber should receive publish RPC at the cap");
        match event {
            SubscriberEvent::PublishRpc(_, _, method, PeerSrc::Remote(peer)) => {
                assert_eq!(peer, from_peer);
                assert_eq!(method.len(), MAX_RPC_METHOD_LEN, "pubsub RPC methods at the cap must be accepted");
            }
            other => panic!("expected PublishRpc event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn pubsub_other_inbound_rpc_methods_must_be_bounded() {
        let oversized_method = "m".repeat(MAX_RPC_METHOD_LEN + 1);
        let from_peer = PeerId::from(2);

        let mut guest_publish_service = test_service();
        let guest_publish_channel = PubsubChannelId(1);
        let (sub_tx, mut sub_rx) = subscriber_event_channel();
        guest_publish_service
            .on_internal(InternalMsg::SubscriberCreated(subscriber_handle(SubscriberLocalId::rand()), guest_publish_channel, sub_tx))
            .await
            .expect("subscriber should be registered");
        let guest_publish_payload =
            bincode::serialize(&PubsubMessage::GuestPublishRpc(guest_publish_channel, vec![1], RpcId::rand(), oversized_method.clone())).expect("test guest publish RPC should serialize");
        guest_publish_service
            .on_service(P2pServiceEvent::Unicast(from_peer, guest_publish_payload))
            .await
            .expect("guest publish RPC should be processed");
        assert!(sub_rx.try_recv().is_err(), "oversized GuestPublishRpc methods must be dropped");

        let mut guest_feedback_service = test_service();
        let guest_feedback_channel = PubsubChannelId(2);
        let (pub_tx, mut pub_rx) = publisher_event_channel();
        guest_feedback_service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), guest_feedback_channel, pub_tx))
            .await
            .expect("publisher should be registered");
        let guest_feedback_payload =
            bincode::serialize(&PubsubMessage::GuestFeedbackRpc(guest_feedback_channel, vec![1], RpcId::rand(), oversized_method.clone())).expect("test guest feedback RPC should serialize");
        guest_feedback_service
            .on_service(P2pServiceEvent::Unicast(from_peer, guest_feedback_payload))
            .await
            .expect("guest feedback RPC should be processed");
        assert!(pub_rx.try_recv().is_err(), "oversized GuestFeedbackRpc methods must be dropped");

        let mut feedback_service = test_service();
        let feedback_channel = PubsubChannelId(3);
        let (pub_tx, mut pub_rx) = publisher_event_channel();
        feedback_service
            .on_internal(InternalMsg::PublisherCreated(publisher_handle(PublisherLocalId::rand()), feedback_channel, pub_tx))
            .await
            .expect("publisher should be registered");
        feedback_service
            .on_service(P2pServiceEvent::Unicast(from_peer, encode_subscriber_joined_for_test(feedback_channel)))
            .await
            .expect("subscriber join should be processed");
        assert_eq!(pub_rx.try_recv(), Ok(PublisherEvent::PeerJoined(PeerSrc::Remote(from_peer))));
        let feedback_payload = bincode::serialize(&PubsubMessage::FeedbackRpc(feedback_channel, vec![1], RpcId::rand(), oversized_method)).expect("test feedback RPC should serialize");
        feedback_service
            .on_service(P2pServiceEvent::Unicast(from_peer, feedback_payload))
            .await
            .expect("feedback RPC should be processed");
        assert!(pub_rx.try_recv().is_err(), "oversized FeedbackRpc methods must be dropped");
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
