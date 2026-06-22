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

pub(crate) const LOCAL_SUBSCRIBER_EVENT_QUEUE_SIZE: usize = 1024;

#[derive(Debug, Display, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct SubscriberLocalId(u64);
impl SubscriberLocalId {
    pub fn rand() -> Self {
        Self(rand::random())
    }

    #[cfg(test)]
    pub(crate) fn from_raw_for_test(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct SubscriberHandleId {
    local_id: SubscriberLocalId,
    owner: u64,
}

impl SubscriberHandleId {
    pub(crate) fn new(local_id: SubscriberLocalId) -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        Self {
            local_id,
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl fmt::Display for SubscriberHandleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.local_id, self.owner)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriberEvent {
    PeerJoined(PeerSrc),
    PeerLeaved(PeerSrc),
    GuestPublish(Vec<u8>),
    GuestPublishRpc(Vec<u8>, RpcId, String, PeerSrc),
    Publish(Vec<u8>),
    PublishRpc(Vec<u8>, RpcId, String, PeerSrc),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubscriberEventOb<Fb> {
    PeerJoined(PeerSrc),
    PeerLeaved(PeerSrc),
    Publish(Fb),
    PublishDeseializeErr(Vec<u8>),
    PublishRpc(Fb, RpcId, String, PeerSrc),
    PublishRpcDeseializeErr(Vec<u8>, RpcId, String, PeerSrc),
    GuestPublish(Fb),
    GuestPublishDeseializeErr(Vec<u8>),
    GuestPublishRpc(Fb, RpcId, String, PeerSrc),
    GuestPublishRpcDeseializeErr(Vec<u8>, RpcId, String, PeerSrc),
}

pub struct Subscriber {
    local_id: SubscriberLocalId,
    handle_id: SubscriberHandleId,
    channel_id: PubsubChannelId,
    control_tx: Sender<InternalMsg>,
    requester: SubscriberRequester,
    sub_rx: Receiver<SubscriberEvent>,
    _sub_tx_guard: Sender<SubscriberEvent>,
    registered: bool,
}

impl Subscriber {
    pub(super) fn build(channel_id: PubsubChannelId, control_tx: Sender<InternalMsg>) -> Self {
        let (sub_tx, sub_rx) = channel(LOCAL_SUBSCRIBER_EVENT_QUEUE_SIZE);
        let local_id = SubscriberLocalId::rand();
        let handle_id = SubscriberHandleId::new(local_id);
        log::info!("[Subscriber {channel_id}/{local_id}] created");
        let registered = match try_send_internal_control(&control_tx, InternalMsg::SubscriberCreated(handle_id, channel_id, sub_tx.clone()), "subscriber registration") {
            Ok(()) => true,
            Err(err) => {
                log::debug!("[Subscriber {channel_id}/{local_id}] registration dropped: {err}");
                false
            }
        };

        Self {
            local_id,
            handle_id,
            channel_id,
            control_tx: control_tx.clone(),
            requester: SubscriberRequester {
                handle_id,
                channel_id,
                control_tx,
                registered,
            },
            sub_rx,
            _sub_tx_guard: sub_tx,
            registered,
        }
    }

    pub fn requester(&self) -> &SubscriberRequester {
        &self.requester
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<SubscriberEvent>> {
        self.sub_rx.poll_recv(cx)
    }

    pub async fn recv(&mut self) -> anyhow::Result<SubscriberEvent> {
        self.sub_rx.recv().await.ok_or_else(|| anyhow!("internal channel error"))
    }

    pub async fn recv_ob<Fb: DeserializeOwned>(&mut self) -> anyhow::Result<SubscriberEventOb<Fb>> {
        let event = match self.recv().await? {
            SubscriberEvent::PeerJoined(peer_src) => SubscriberEventOb::PeerJoined(peer_src),
            SubscriberEvent::PeerLeaved(peer_src) => SubscriberEventOb::PeerLeaved(peer_src),
            SubscriberEvent::Publish(data) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    SubscriberEventOb::Publish(ob)
                } else {
                    SubscriberEventOb::PublishDeseializeErr(data)
                }
            }
            SubscriberEvent::PublishRpc(data, rpc_id, method, peer_src) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    SubscriberEventOb::PublishRpc(ob, rpc_id, method, peer_src)
                } else {
                    SubscriberEventOb::PublishRpcDeseializeErr(data, rpc_id, method, peer_src)
                }
            }
            SubscriberEvent::GuestPublish(data) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    SubscriberEventOb::GuestPublish(ob)
                } else {
                    SubscriberEventOb::GuestPublishDeseializeErr(data)
                }
            }
            SubscriberEvent::GuestPublishRpc(data, rpc_id, method, peer_src) => {
                if let Ok(ob) = bincode::deserialize(&data) {
                    SubscriberEventOb::GuestPublishRpc(ob, rpc_id, method, peer_src)
                } else {
                    SubscriberEventOb::GuestPublishRpcDeseializeErr(data, rpc_id, method, peer_src)
                }
            }
        };
        Ok(event)
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        log::info!("[Subscriber {}/{}] destroy", self.channel_id, self.local_id);
        if !self.registered {
            return;
        }
        if let Err(err) = try_send_internal_control(&self.control_tx, InternalMsg::SubscriberDestroyed(self.handle_id, self.channel_id), "subscriber destruction") {
            log::debug!("[Subscriber {}/{}] destruction dropped: {err}", self.channel_id, self.local_id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscriberRequester {
    handle_id: SubscriberHandleId,
    channel_id: PubsubChannelId,
    control_tx: Sender<InternalMsg>,
    registered: bool,
}

impl SubscriberRequester {
    fn ensure_registered(&self, context: &str) -> anyhow::Result<()> {
        if self.registered {
            Ok(())
        } else {
            Err(anyhow!("{context}: subscriber registration was not admitted"))
        }
    }

    pub async fn feedback(&self, data: Vec<u8>) -> anyhow::Result<()> {
        self.ensure_registered("subscriber feedback")?;
        try_send_internal_control(&self.control_tx, InternalMsg::Feedback(self.handle_id, self.channel_id, data), "subscriber feedback")?;
        Ok(())
    }

    pub async fn feedback_ob<Ob: Serialize>(&self, ob: &Ob) -> anyhow::Result<()> {
        let data = bincode::serialize(ob)?;
        self.feedback(data).await
    }

    pub async fn feedback_rpc(&self, method: &str, data: Vec<u8>, timeout: Duration) -> anyhow::Result<Vec<u8>> {
        self.ensure_registered("subscriber feedback_rpc")?;
        let (tx, rx) = oneshot::channel::<Result<Vec<u8>, PubsubRpcError>>();
        try_send_internal_control(
            &self.control_tx,
            InternalMsg::FeedbackRpc(self.handle_id, self.channel_id, data, method.to_owned(), tx, timeout),
            "subscriber feedback_rpc",
        )?;
        let data = rx.await??;
        Ok(data)
    }

    pub async fn feedback_rpc_ob<REQ: Serialize, RES: DeserializeOwned>(&self, method: &str, req: &REQ, timeout: Duration) -> anyhow::Result<RES> {
        let data = bincode::serialize(req)?;
        let res = self.feedback_rpc(method, data, timeout).await?;
        Ok(bincode::deserialize(&res)?)
    }

    pub async fn answer_publish_rpc(&self, rpc: RpcId, source: PeerSrc, data: Vec<u8>) -> anyhow::Result<()> {
        self.ensure_registered("subscriber answer_publish_rpc")?;
        try_send_internal_control(&self.control_tx, InternalMsg::PublishRpcAnswer(self.handle_id, rpc, source, data), "subscriber answer_publish_rpc")?;
        Ok(())
    }

    pub async fn answer_publish_rpc_ob<RES: Serialize>(&self, rpc: RpcId, source: PeerSrc, res: &RES) -> anyhow::Result<()> {
        self.answer_publish_rpc(rpc, source, bincode::serialize(res)?).await
    }
}
