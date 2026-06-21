use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
    time::Duration,
};

use anyhow::anyhow;
use derive_more::derive::Display;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot,
};

use super::{try_send_internal_control, InternalMsg, PeerSrc, PubsubChannelId, PubsubRpcError, RpcId};

pub(crate) const LOCAL_PUBLISHER_EVENT_QUEUE_SIZE: usize = 1024;

#[derive(Debug, Display, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct PublisherLocalId(u64);
impl PublisherLocalId {
    pub fn rand() -> Self {
        Self(rand::random())
    }

    #[cfg(test)]
    pub(crate) fn from_raw_for_test(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct PublisherHandleId {
    local_id: PublisherLocalId,
    owner: u64,
}

impl PublisherHandleId {
    pub(crate) fn new(local_id: PublisherLocalId) -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        Self {
            local_id,
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl fmt::Display for PublisherHandleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.local_id, self.owner)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublisherEvent {
    PeerJoined(PeerSrc),
    PeerLeaved(PeerSrc),
    Feedback(Vec<u8>),
    FeedbackRpc(Vec<u8>, RpcId, String, PeerSrc),
    GuestFeedback(Vec<u8>),
    GuestFeedbackRpc(Vec<u8>, RpcId, String, PeerSrc),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublisherEventOb<Fb> {
    PeerJoined(PeerSrc),
    PeerLeaved(PeerSrc),
    Feedback(Fb),
    FeedbackDeseializeErr(Vec<u8>),
    FeedbackRpc(Fb, RpcId, String, PeerSrc),
    FeedbackRpcDeseializeErr(Vec<u8>, RpcId, String, PeerSrc),
    GuestFeedback(Fb),
    GuestFeedbackDeseializeErr(Vec<u8>),
    GuestFeedbackRpc(Fb, RpcId, String, PeerSrc),
    GuestFeedbackRpcDeseializeErr(Vec<u8>, RpcId, String, PeerSrc),
}

pub struct Publisher {
    local_id: PublisherLocalId,
    handle_id: PublisherHandleId,
    channel_id: PubsubChannelId,
    control_tx: Sender<InternalMsg>,
    requester: PublisherRequester,
    pub_rx: Receiver<PublisherEvent>,
}

impl Publisher {
    pub(super) fn build(channel_id: PubsubChannelId, control_tx: Sender<InternalMsg>) -> Self {
        let (pub_tx, pub_rx) = channel(LOCAL_PUBLISHER_EVENT_QUEUE_SIZE);
        let local_id = PublisherLocalId::rand();
        let handle_id = PublisherHandleId::new(local_id);
        log::info!("[Publisher {channel_id}/{local_id}] created");
        if let Err(err) = try_send_internal_control(&control_tx, InternalMsg::PublisherCreated(handle_id, channel_id, pub_tx), "publisher registration") {
            log::debug!("[Publisher {channel_id}/{local_id}] registration dropped: {err}");
        }

        Self {
            local_id,
            handle_id,
            channel_id,
            control_tx: control_tx.clone(),
            requester: PublisherRequester { handle_id, channel_id, control_tx },
            pub_rx,
        }
    }

    pub fn requester(&self) -> &PublisherRequester {
        &self.requester
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<PublisherEvent>> {
        self.pub_rx.poll_recv(cx)
    }

    pub async fn recv(&mut self) -> anyhow::Result<PublisherEvent> {
        self.pub_rx.recv().await.ok_or_else(|| anyhow!("internal channel error"))
    }

    pub async fn recv_ob<Fb: DeserializeOwned>(&mut self) -> anyhow::Result<PublisherEventOb<Fb>> {
        let event = match self.recv().await? {
            PublisherEvent::PeerJoined(peer_src) => PublisherEventOb::PeerJoined(peer_src),
            PublisherEvent::PeerLeaved(peer_src) => PublisherEventOb::PeerLeaved(peer_src),
            PublisherEvent::Feedback(data) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    PublisherEventOb::Feedback(ob)
                } else {
                    PublisherEventOb::FeedbackDeseializeErr(data)
                }
            }
            PublisherEvent::FeedbackRpc(data, rpc_id, method, peer_src) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    PublisherEventOb::FeedbackRpc(ob, rpc_id, method, peer_src)
                } else {
                    PublisherEventOb::FeedbackRpcDeseializeErr(data, rpc_id, method, peer_src)
                }
            }
            PublisherEvent::GuestFeedback(data) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    PublisherEventOb::GuestFeedback(ob)
                } else {
                    PublisherEventOb::GuestFeedbackDeseializeErr(data)
                }
            }
            PublisherEvent::GuestFeedbackRpc(data, rpc_id, method, peer_src) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    PublisherEventOb::GuestFeedbackRpc(ob, rpc_id, method, peer_src)
                } else {
                    PublisherEventOb::GuestFeedbackRpcDeseializeErr(data, rpc_id, method, peer_src)
                }
            }
        };
        Ok(event)
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        log::info!("[Publisher {}/{}] destroy", self.channel_id, self.local_id);
        if let Err(err) = try_send_internal_control(&self.control_tx, InternalMsg::PublisherDestroyed(self.handle_id, self.channel_id), "publisher destruction") {
            log::debug!("[Publisher {}/{}] destruction dropped: {err}", self.channel_id, self.local_id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublisherRequester {
    handle_id: PublisherHandleId,
    channel_id: PubsubChannelId,
    control_tx: Sender<InternalMsg>,
}

impl PublisherRequester {
    pub async fn publish(&self, data: Vec<u8>) -> anyhow::Result<()> {
        try_send_internal_control(&self.control_tx, InternalMsg::Publish(self.handle_id, self.channel_id, data), "publisher publish")?;
        Ok(())
    }

    pub async fn publish_ob<Ob: Serialize>(&self, ob: &Ob) -> anyhow::Result<()> {
        let data = bincode::serialize(ob).expect("should serialize");
        self.publish(data).await
    }

    pub async fn publish_rpc(&self, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(
            &self.control_tx,
            InternalMsg::PublishRpc(self.handle_id, self.channel_id, data, method.to_owned(), tx, timeout),
            "publisher publish_rpc",
        )?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn publish_rpc_ob<REQ: Serialize, RES: DeserializeOwned>(&self, method: &str, req: &REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(req).expect("should convert to buffer");
        let res = self.publish_rpc(method, data, timeout).await?;
        Ok(bincode::deserialize(&res)?)
    }

    pub async fn answer_feedback_rpc(&self, rpc: RpcId, source: PeerSrc, data: Vec<u8>) -> anyhow::Result<()> {
        try_send_internal_control(&self.control_tx, InternalMsg::FeedbackRpcAnswer(self.handle_id, rpc, source, data), "publisher answer_feedback_rpc")?;
        Ok(())
    }

    pub async fn answer_feedback_rpc_ob<RES: Serialize>(&self, rpc: RpcId, source: PeerSrc, res: &RES) -> anyhow::Result<()> {
        self.answer_feedback_rpc(rpc, source, bincode::serialize(res).expect("should serialize")).await
    }
}
