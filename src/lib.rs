use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    net::SocketAddr,
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use ctx::SharedCtx;
use derive_more::derive::{Deref, Display, From};
use discovery::{PeerDiscovery, PeerDiscoverySync};
use msg::{P2pServiceId, PeerMessage};
use neighbours::NetworkNeighbours;
use peer::PeerConnection;
use quinn::{Endpoint, Incoming, VarInt};
use router::RouterTableSync;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    task::JoinHandle,
    time::Interval,
};

use crate::quic::make_server_endpoint;

mod ctx;
mod discovery;
mod msg;
mod neighbours;
mod peer;
mod quic;
mod requester;
mod router;
mod secure;
mod service;
mod stats;
mod stream;
#[cfg(test)]
mod tests;
mod utils;

pub use peer::PeerConnectionMetric;
pub use requester::P2pNetworkRequester;
pub use router::SharedRouterTable;
pub use secure::*;
pub use service::*;
pub use stats::*;
pub use stream::P2pQuicStream;
pub use utils::*;

#[derive(Debug, Display, Clone, Copy, From, Deref, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PeerId(u64);

#[derive(Debug, Display, From, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConnectionId(u64);

impl ConnectionId {
    pub fn rand() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug, Clone, From, Display, Deref, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkAddress(SocketAddr);

#[derive(Debug, Clone, From, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerAddress(PeerId, NetworkAddress);

impl PeerAddress {
    pub fn new(p: PeerId, a: NetworkAddress) -> Self {
        Self(p, a)
    }

    pub fn peer_id(&self) -> PeerId {
        self.0
    }

    pub fn network_address(&self) -> &NetworkAddress {
        &self.1
    }
}

impl Display for PeerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.peer_id(), self.network_address())
    }
}

impl FromStr for PeerAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid format, expected 'peer_id@network_address'".to_string());
        }
        let peer_id = parts[0].parse::<u64>().map(PeerId).map_err(|e| e.to_string())?;
        let network_address = parts[1].parse::<SocketAddr>().map(NetworkAddress).map_err(|e| e.to_string())?;
        Ok(Self(peer_id, network_address))
    }
}

pub const CERT_DOMAIN_NAME: &str = "cluster";
pub(crate) const NETWORK_CONTROL_QUEUE_SIZE: usize = 1024;
#[allow(dead_code)]
pub(crate) const MAX_PENDING_UNAUTHENTICATED_INBOUND_CONNECTIONS: usize = 16;

#[derive(Debug)]
enum PeerMainData {
    Sync { route: RouterTableSync, advertise: PeerDiscoverySync },
}

#[allow(clippy::enum_variant_names)]
enum MainEvent {
    PeerConnected(ConnectionId, PeerId, u16),
    PeerConnectError(ConnectionId, Option<PeerId>, anyhow::Error),
    PeerData(ConnectionId, PeerId, PeerMainData),
    PeerStopped(ConnectionId, PeerId),
    PeerStats(ConnectionId, PeerId, PeerConnectionMetric),
    PeerDisconnected(ConnectionId, PeerId),
}

enum ControlCmd {
    Connect(PeerAddress, Option<oneshot::Sender<anyhow::Result<()>>>),
}

pub struct P2pNetworkConfig<SECURE> {
    pub peer_id: PeerId,
    pub listen_addr: SocketAddr,
    pub advertise: Option<NetworkAddress>,
    pub priv_key: PrivatePkcs8KeyDer<'static>,
    pub cert: CertificateDer<'static>,
    pub tick_ms: u64,
    pub seeds: Vec<PeerAddress>,
    pub secure: SECURE,
}

#[derive(Debug, PartialEq, Eq)]
pub enum P2pNetworkEvent {
    PeerConnected(ConnectionId, PeerId),
    PeerDisconnected(ConnectionId, PeerId),
    Continue,
}

pub struct P2pNetwork<SECURE> {
    local_id: PeerId,
    endpoint: Endpoint,
    control_tx: Sender<ControlCmd>,
    control_rx: Receiver<ControlCmd>,
    main_tx: Sender<MainEvent>,
    main_rx: Receiver<MainEvent>,
    neighbours: NetworkNeighbours,
    ticker: Interval,
    router: SharedRouterTable,
    discovery: PeerDiscovery,
    pending_sync_tasks: HashMap<ConnectionId, JoinHandle<()>>,
    pending_connects: HashMap<ConnectionId, oneshot::Sender<anyhow::Result<()>>>,
    ctx: SharedCtx,
    secure: Arc<SECURE>,
}

impl<SECURE: HandshakeProtocol> P2pNetwork<SECURE> {
    pub async fn new(cfg: P2pNetworkConfig<SECURE>) -> anyhow::Result<Self> {
        log::info!("[P2pNetwork] starting node {}@{}", cfg.peer_id, cfg.listen_addr);
        if cfg.tick_ms == 0 {
            anyhow::bail!("P2pNetworkConfig.tick_ms must be greater than 0");
        }

        let endpoint = make_server_endpoint(cfg.listen_addr, cfg.priv_key, cfg.cert)?;
        let (main_tx, main_rx) = channel(10);
        let (control_tx, control_rx) = channel(NETWORK_CONTROL_QUEUE_SIZE);
        let mut discovery = PeerDiscovery::new(cfg.seeds);
        let router = SharedRouterTable::new(cfg.peer_id);

        if let Some(addr) = cfg.advertise {
            discovery.enable_local(cfg.peer_id, addr);
        }

        Ok(Self {
            local_id: cfg.peer_id,
            endpoint,
            neighbours: NetworkNeighbours::default(),
            main_tx,
            main_rx,
            control_tx,
            control_rx,
            ticker: tokio::time::interval(Duration::from_millis(cfg.tick_ms)),
            ctx: SharedCtx::new(cfg.peer_id, router.clone()),
            router,
            discovery,
            pending_sync_tasks: HashMap::new(),
            pending_connects: HashMap::new(),
            secure: Arc::new(cfg.secure),
        })
    }

    pub fn create_service(&mut self, service_id: P2pServiceId) -> P2pService {
        let (mut service, tx) = P2pService::build(service_id, self.ctx.clone());
        let registered = self.ctx.set_service(service_id, tx);
        service.set_registered(registered);
        service
    }

    pub fn requester(&mut self) -> P2pNetworkRequester {
        P2pNetworkRequester { control_tx: self.control_tx.clone() }
    }

    pub async fn recv(&mut self) -> anyhow::Result<P2pNetworkEvent> {
        select! {
            _ = self.ticker.tick() => {
                self.process_tick(now_ms())
            }
            connecting = self.endpoint.accept() => {
                self.process_incoming(connecting.ok_or(anyhow!("quic crash"))?)
            },
            event = self.main_rx.recv() => {
                self.process_internal(now_ms(), event.ok_or(anyhow!("internal channel crash"))?)
            },
            event = self.control_rx.recv() => {
                self.process_control(event.ok_or(anyhow!("internal channel crash"))?)
            },

        }
    }

    pub fn shutdown(&mut self) {
        self.endpoint.close(VarInt::from_u32(0), "Shutdown".as_bytes());
    }

    pub async fn shutdown_gracefully(&mut self) {
        let conns = self.ctx.conns();
        let local_id = self.local_id;
        let notifications = conns.into_iter().map(|conn| async move { conn.send_wait(PeerMessage::PeerStopped(local_id)).await });
        let notify_all = async {
            for result in futures::future::join_all(notifications).await {
                if let Err(err) = result {
                    log::warn!("[P2pNetwork] graceful shutdown notify failed: {err}");
                }
            }
        };
        if let Err(err) = tokio::time::timeout(Duration::from_secs(1), notify_all).await {
            log::warn!("[P2pNetwork] graceful shutdown notify timeout: {err}");
        }
        self.shutdown();
    }

    fn process_tick(&mut self, now_ms: u64) -> anyhow::Result<P2pNetworkEvent> {
        for peer in self.discovery.clear_timeout(now_ms) {
            self.router.del_learned_peer(&peer);
        }
        self.pending_sync_tasks.retain(|_, task| !task.is_finished());
        for conn in self.neighbours.connected_conns() {
            let peer_id = conn.peer_id().expect("connected neighbours should have peer_id");
            let conn_id = conn.conn_id();
            let route: router::RouterTableSync = self.router.create_sync(&peer_id);
            let advertise = self.discovery.create_sync_for(now_ms, &peer_id);
            if let Some(alias) = self.ctx.conn(&conn_id) {
                if let Err(e) = alias.try_send(PeerMessage::Sync { route, advertise }) {
                    log::error!("[P2pNetwork] try send message to peer {peer_id} over conn {conn_id} error {e}");
                    if let Some(task) = self.pending_sync_tasks.remove(&conn_id) {
                        task.abort();
                    }
                    let route = self.router.create_sync(&peer_id);
                    let advertise = self.discovery.create_sync_for(now_ms, &peer_id);
                    self.pending_sync_tasks.insert(
                        conn_id,
                        tokio::spawn(async move {
                            if let Err(err) = alias.send(PeerMessage::Sync { route, advertise }).await {
                                log::debug!("[P2pNetwork] retry send sync to peer {peer_id} over conn {conn_id} failed: {err}");
                            }
                        }),
                    );
                } else if let Some(task) = self.pending_sync_tasks.remove(&conn_id) {
                    task.abort();
                }
            } else if let Some(task) = self.pending_sync_tasks.remove(&conn_id) {
                task.abort();
            }
        }
        let remotes: Vec<_> = self.discovery.remotes().collect();
        for addr in remotes {
            self.process_connect(addr, None)?;
        }

        Ok(P2pNetworkEvent::Continue)
    }

    fn process_incoming(&mut self, incoming: Incoming) -> anyhow::Result<P2pNetworkEvent> {
        let remote = incoming.remote_address();
        log::info!("[P2pNetwork] incoming connect from {remote} => accept");
        let conn = PeerConnection::new_incoming(self.secure.clone(), self.local_id, incoming, self.main_tx.clone(), self.ctx.clone());
        self.neighbours.insert(conn.conn_id(), conn);
        Ok(P2pNetworkEvent::Continue)
    }

    fn process_internal(&mut self, now_ms: u64, event: MainEvent) -> anyhow::Result<P2pNetworkEvent> {
        match event {
            MainEvent::PeerConnected(conn, peer, ttl_ms) => {
                log::info!("[P2pNetwork] connection {conn} connected to {peer}");
                let Some(alias) = self.ctx.conn(&conn) else {
                    log::warn!("[P2pNetwork] ignore peer connected for {peer} from unknown connection {conn}");
                    if let Some(tx) = self.pending_connects.remove(&conn) {
                        tx.send(Err(anyhow!("connected event for unknown connection {conn}"))).print_on_err2("[P2pNetwork] send connect answer");
                    }
                    return Ok(P2pNetworkEvent::Continue);
                };
                if alias.to_id() != peer {
                    log::warn!("[P2pNetwork] ignore peer connected for {peer} from connection {conn} bound to {}", alias.to_id());
                    if let Some(tx) = self.pending_connects.remove(&conn) {
                        tx.send(Err(anyhow!("connected peer mismatch for connection {conn}: expected {}, got {peer}", alias.to_id())))
                            .print_on_err2("[P2pNetwork] send connect answer");
                    }
                    return Ok(P2pNetworkEvent::Continue);
                }
                if self.neighbours.has_peer(&peer) {
                    if self.router.is_direct_peer(&conn, &peer) {
                        log::warn!("[P2pNetwork] ignore duplicate peer connected for {peer} from already-direct connection {conn}");
                    } else {
                        log::warn!("[P2pNetwork] reject duplicate peer connected for {peer} from connection {conn}");
                        alias.try_close().print_on_err2("[P2pNetwork] close duplicate peer connection");
                        self.router.del_direct(&conn);
                        self.neighbours.remove(&conn);
                        self.ctx.unregister_conn(&conn);
                        if let Some(tx) = self.pending_connects.remove(&conn) {
                            tx.send(Err(anyhow!("duplicate connection to peer {peer}"))).print_on_err2("[P2pNetwork] send connect answer");
                        }
                    }
                    return Ok(P2pNetworkEvent::Continue);
                }
                self.router.set_direct(conn, peer, ttl_ms);
                self.neighbours.mark_connected(&conn, peer);
                if let Some(tx) = self.pending_connects.remove(&conn) {
                    tx.send(Ok(())).print_on_err2("[P2pNetwork] send connect answer");
                }
                Ok(P2pNetworkEvent::PeerConnected(conn, peer))
            }
            MainEvent::PeerData(conn, peer, data) => {
                log::debug!("[P2pNetwork] connection {conn} on data {data:?} from {peer}");
                if !self.router.is_direct_peer(&conn, &peer) {
                    log::warn!("[P2pNetwork] ignore peer data for {peer} from non-direct connection {conn}");
                    return Ok(P2pNetworkEvent::Continue);
                }

                match data {
                    PeerMainData::Sync { route, advertise } => {
                        self.discovery.apply_sync(now_ms, advertise);
                        self.router.apply_sync_filtered(conn, route, |peer| self.discovery.is_stopped(now_ms, peer));
                    }
                }
                Ok(P2pNetworkEvent::Continue)
            }
            MainEvent::PeerStopped(conn, peer) => {
                log::info!("[P2pNetwork] connection {conn} reported peer {peer} stopped");
                if !self.router.is_direct_peer(&conn, &peer) {
                    log::warn!("[P2pNetwork] ignore peer stopped for {peer} from non-direct connection {conn}");
                    return Ok(P2pNetworkEvent::Continue);
                }

                if let Some(task) = self.pending_sync_tasks.remove(&conn) {
                    task.abort();
                }
                self.discovery.remove_remote(now_ms, &peer);
                self.router.del_peer(&peer);
                self.neighbours.remove(&conn);
                self.ctx.unregister_conn(&conn);
                self.ctx.try_send_peer_disconnected_to_services(peer);
                Ok(P2pNetworkEvent::PeerDisconnected(conn, peer))
            }
            MainEvent::PeerConnectError(conn, peer, err) => {
                log::error!("[P2pNetwork] connection {conn} outgoing: {peer:?} error {err}");
                if self.neighbours.get(&conn).is_some_and(|conn| conn.is_connected()) {
                    log::warn!("[P2pNetwork] ignore stale connect error for already connected {conn}");
                    return Ok(P2pNetworkEvent::Continue);
                }

                if let Some(task) = self.pending_sync_tasks.remove(&conn) {
                    task.abort();
                }
                if let Some(tx) = self.pending_connects.remove(&conn) {
                    tx.send(Err(err)).print_on_err2("[P2pNetwork] send connect answer");
                }
                self.neighbours.remove(&conn);
                Ok(P2pNetworkEvent::Continue)
            }
            MainEvent::PeerDisconnected(conn, peer) => {
                log::info!("[P2pNetwork] connection {conn} disconnected from {peer}");
                if !self.router.is_direct_peer(&conn, &peer) {
                    log::warn!("[P2pNetwork] ignore peer disconnected for {peer} from non-direct connection {conn}");
                    return Ok(P2pNetworkEvent::Continue);
                }

                if let Some(task) = self.pending_sync_tasks.remove(&conn) {
                    task.abort();
                }
                self.router.del_direct(&conn);
                self.neighbours.remove(&conn);
                self.ctx.try_send_peer_disconnected_to_services(peer);
                Ok(P2pNetworkEvent::PeerDisconnected(conn, peer))
            }
            MainEvent::PeerStats(conn, to_peer, metrics) => {
                log::debug!("[P2pNetwork] conn {conn} to peer {to_peer} metrics {:?}", metrics);
                if !self.router.is_direct_peer(&conn, &to_peer) {
                    log::warn!("[P2pNetwork] ignore peer stats for {to_peer} from non-direct connection {conn}");
                    return Ok(P2pNetworkEvent::Continue);
                }

                self.ctx.update_metrics(&conn, to_peer, metrics);
                Ok(P2pNetworkEvent::Continue)
            }
        }
    }

    fn process_control(&mut self, cmd: ControlCmd) -> anyhow::Result<P2pNetworkEvent> {
        match cmd {
            ControlCmd::Connect(addr, tx) => self.process_connect(addr, tx),
        }
    }

    fn process_connect(&mut self, addr: PeerAddress, tx: Option<oneshot::Sender<anyhow::Result<()>>>) -> anyhow::Result<P2pNetworkEvent> {
        if addr.peer_id() == self.local_id {
            let res = Err(anyhow!("refuse to connect to local peer {}", self.local_id));
            if let Some(tx) = tx {
                tx.send(res).print_on_err2("[P2pNetwork] send connect answer");
            }
            return Ok(P2pNetworkEvent::Continue);
        }

        let res = if self.neighbours.has_peer_connection_attempt(&addr.peer_id()) {
            if tx.is_some() {
                Err(anyhow!("connection attempt to peer {} already exists", addr.peer_id()))
            } else {
                Ok(())
            }
        } else {
            log::info!("[P2pNetwork] connecting to {addr}");
            match self.endpoint.connect(*addr.network_address().deref(), CERT_DOMAIN_NAME) {
                Ok(connecting) => {
                    let conn = PeerConnection::new_connecting(self.secure.clone(), self.local_id, addr.peer_id(), connecting, self.main_tx.clone(), self.ctx.clone());
                    let conn_id = conn.conn_id();
                    self.neighbours.insert(conn_id, conn);
                    if let Some(tx) = tx {
                        self.pending_connects.insert(conn_id, tx);
                    }
                    return Ok(P2pNetworkEvent::Continue);
                }
                Err(err) => Err(err.into()),
            }
        };

        if let Some(tx) = tx {
            tx.send(res).print_on_err2("[P2pNetwork] send connect answer");
        }

        Ok(P2pNetworkEvent::Continue)
    }
}
