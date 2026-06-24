use anyhow::anyhow;
use tokio::sync::{
    mpsc::{error::TrySendError, Sender},
    oneshot,
};

use crate::{ControlCmd, PeerAddress};

pub struct P2pNetworkRequester {
    pub(crate) control_tx: Sender<ControlCmd>,
}

impl P2pNetworkRequester {
    pub async fn connect(&self, addr: PeerAddress) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.control_tx.try_send(ControlCmd::Connect(addr, Some(tx))).map_err(|err| match err {
            TrySendError::Full(_) => anyhow!("network control queue is full"),
            TrySendError::Closed(_) => anyhow!("network control queue is closed"),
        })?;
        rx.await?
    }

    pub fn try_connect(&self, addr: PeerAddress) {
        match self.control_tx.try_send(ControlCmd::Connect(addr, None)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => log::debug!("[P2pNetworkRequester] network control queue full, dropping best-effort connect"),
            Err(TrySendError::Closed(_)) => log::debug!("[P2pNetworkRequester] network control queue closed, dropping best-effort connect"),
        }
    }
}
