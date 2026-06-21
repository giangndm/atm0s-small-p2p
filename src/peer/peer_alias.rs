//! PeerAlias allow control a peer-connection from othert task
//! This is done by using control_tx to send control to running task over channel

use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    msg::{P2pServiceId, PeerMessage, UnicastAckId},
    stream::P2pQuicStream,
    ConnectionId, PeerId,
};

use super::PeerConnectionControl;

const UNICAST_ACK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct PeerConnectionAlias {
    local_id: PeerId,
    to_id: PeerId,
    conn_id: ConnectionId,
    control_tx: Sender<PeerConnectionControl>,
}

impl PeerConnectionAlias {
    pub(super) fn new(local_id: PeerId, to_id: PeerId, conn_id: ConnectionId, control_tx: Sender<PeerConnectionControl>) -> Self {
        Self { local_id, to_id, conn_id, control_tx }
    }

    #[allow(unused)]
    pub(super) fn conn_id(&self) -> ConnectionId {
        self.conn_id
    }

    #[allow(unused)]
    pub(super) fn local_id(&self) -> PeerId {
        self.local_id
    }

    pub(crate) fn to_id(&self) -> PeerId {
        self.to_id
    }

    pub(crate) fn try_send(&self, msg: PeerMessage) -> anyhow::Result<()> {
        Ok(self.control_tx.try_send(PeerConnectionControl::Send(msg, None))?)
    }

    pub(crate) fn try_close(&self) -> anyhow::Result<()> {
        Ok(self.control_tx.try_send(PeerConnectionControl::Close)?)
    }

    pub(crate) async fn send(&self, msg: PeerMessage) -> anyhow::Result<()> {
        Ok(self.control_tx.send(PeerConnectionControl::Send(msg, None)).await?)
    }

    pub(crate) async fn send_wait(&self, msg: PeerMessage) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.control_tx.send(PeerConnectionControl::Send(msg, Some(tx))).await?;
        rx.await?
    }

    pub(crate) async fn send_unicast_with_ack(&self, source: PeerId, dest: PeerId, service: P2pServiceId, data: Vec<u8>) -> anyhow::Result<()> {
        let ack_id = UnicastAckId::rand();
        let (tx, rx) = oneshot::channel();
        self.control_tx
            .try_send(PeerConnectionControl::SendUnicastWithAck(ack_id, source, dest, service, data, tx))?;
        tokio::time::timeout(UNICAST_ACK_TIMEOUT, rx).await.map_err(|_| anyhow::anyhow!("unicast ack timed out"))??
    }

    pub(crate) async fn open_stream(&self, service: P2pServiceId, source: PeerId, dest: PeerId, meta: Vec<u8>) -> anyhow::Result<P2pQuicStream> {
        let (tx, rx) = oneshot::channel();
        self.control_tx.send(PeerConnectionControl::OpenStream(service, source, dest, meta, tx)).await?;
        rx.await?
    }
}
