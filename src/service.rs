use std::task::{Context, Poll};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{ctx::SharedCtx, msg::P2pServiceId, router::SharedRouterTable, stream::P2pQuicStream, PeerId};

pub mod alias_service;
pub mod metrics_service;
pub mod pubsub_service;
pub mod replicate_kv_service;
pub mod visualization_service;

const SERVICE_CHANNEL_SIZE: usize = 10;

#[derive(Debug, PartialEq, Eq)]
pub enum P2pServiceEvent {
    Unicast(PeerId, Vec<u8>),
    Broadcast(PeerId, Vec<u8>),
    Stream(PeerId, Vec<u8>, P2pQuicStream),
    PeerDisconnected(PeerId),
}

#[derive(Debug, Clone)]
pub struct P2pServiceRequester {
    service: P2pServiceId,
    ctx: SharedCtx,
    service_tx: Sender<P2pServiceEvent>,
}

pub struct P2pService {
    service: P2pServiceId,
    ctx: SharedCtx,
    rx: Receiver<P2pServiceEvent>,
    service_tx: Sender<P2pServiceEvent>,
}

impl P2pService {
    pub(super) fn build(service: P2pServiceId, ctx: SharedCtx) -> (Self, Sender<P2pServiceEvent>) {
        let (tx, rx) = channel(SERVICE_CHANNEL_SIZE);
        (
            Self {
                service,
                ctx,
                rx,
                service_tx: tx.clone(),
            },
            tx,
        )
    }

    pub fn requester(&self) -> P2pServiceRequester {
        P2pServiceRequester {
            service: self.service,
            ctx: self.ctx.clone(),
            service_tx: self.service_tx.clone(),
        }
    }

    pub async fn send_unicast(&self, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        self.ctx.send_unicast(self.service, dest, data).await
    }

    pub async fn send_broadcast(&self, data: Vec<u8>) -> anyhow::Result<usize> {
        self.ctx.send_broadcast(self.service, data).await
    }

    pub async fn try_send_unicast(&self, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        self.ctx.try_send_unicast(self.service, dest, data)
    }

    pub async fn try_send_broadcast(&self, data: Vec<u8>) -> anyhow::Result<usize> {
        self.ctx.try_send_broadcast(self.service, data)
    }

    pub async fn open_stream(&self, dest: PeerId, meta: Vec<u8>) -> anyhow::Result<P2pQuicStream> {
        self.ctx.open_stream(self.service, dest, meta).await
    }

    pub fn router(&self) -> &SharedRouterTable {
        self.ctx.router()
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<P2pServiceEvent>> {
        self.rx.poll_recv(cx)
    }

    pub async fn recv(&mut self) -> Option<P2pServiceEvent> {
        self.rx.recv().await
    }
}

impl P2pServiceRequester {
    fn ensure_live(&self) -> anyhow::Result<()> {
        if self.service_tx.is_closed() {
            anyhow::bail!("service requester is stale");
        }
        Ok(())
    }

    pub async fn send_unicast(&self, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        self.ensure_live()?;
        self.ctx.send_unicast(self.service, dest, data).await
    }

    pub(crate) async fn send_unicast_unacked(&self, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        self.ensure_live()?;
        self.ctx.send_unicast_unacked(self.service, dest, data).await
    }

    pub async fn send_broadcast(&self, data: Vec<u8>) -> anyhow::Result<usize> {
        self.ctx.send_broadcast(self.service, data).await
    }

    pub async fn try_send_unicast(&self, dest: PeerId, data: Vec<u8>) -> anyhow::Result<()> {
        self.ensure_live()?;
        self.ctx.try_send_unicast(self.service, dest, data)
    }

    pub async fn try_send_broadcast(&self, data: Vec<u8>) -> anyhow::Result<usize> {
        self.ctx.try_send_broadcast(self.service, data)
    }

    pub async fn open_stream(&self, dest: PeerId, meta: Vec<u8>) -> anyhow::Result<P2pQuicStream> {
        self.ensure_live()?;
        self.ctx.open_stream(self.service, dest, meta).await
    }

    pub fn router(&self) -> &SharedRouterTable {
        self.ctx.router()
    }
}
