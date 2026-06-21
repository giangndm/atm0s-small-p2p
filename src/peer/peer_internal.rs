//! Task runner for a single connection
//! This must ensure not blocking by other actor.
//! We have some strict rules
//!
//! - Only use async with current connection stream
//! - For other communication should use try_send for avoiding blocking

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use futures::{SinkExt, StreamExt};
use metrics::{counter, gauge};
use quinn::{Connection, RecvStream, SendStream, VarInt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::copy_bidirectional,
    select,
    sync::{
        mpsc::{Receiver, Sender},
        OwnedSemaphorePermit, Semaphore,
    },
    time::Interval,
};
use tokio_util::codec::Framed;

use crate::{
    ctx::SharedCtx,
    msg::{P2pServiceId, PeerMessage, StreamConnectReq, StreamConnectRes, UnicastAckId},
    router::RouteAction,
    stream::{wait_object, write_object, BincodeCodec, P2pQuicStream},
    utils::ErrorExt,
    ConnectionId, MainEvent, P2pServiceEvent, PeerId, PeerMainData, P2P_CONNECTION_CONGESTION_EVENTS, P2P_CONNECTION_LOST_BYTES, P2P_CONNECTION_LOST_PKT, P2P_CONNECTION_RECV_BYTES,
    P2P_CONNECTION_RTT, P2P_CONNECTION_SENT_BYTES, P2P_CONNECTION_UPTIME,
};

use super::PeerConnectionControl;

const OPEN_BI_TIMEOUT: Duration = Duration::from_secs(2);
const ACCEPT_BI_INITIAL_REQ_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_PENDING_ACCEPT_BI: usize = 16;
const LOCAL_SERVICE_DELIVERY_TIMEOUT: Duration = Duration::from_secs(1);
const UNICAST_ACK_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_CONTROL_STREAM_PKT: usize = 60000;
const MAX_PEER_MESSAGE_FRAME: usize = 60000;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PeerConnectionMetric {
    pub uptime: u64,
    pub rtt: u16,
    pub sent_pkt: u64,
    pub lost_pkt: u64,
    pub lost_bytes: u64,
    pub send_bytes: u64,
    pub recv_bytes: u64,
    pub current_mtu: u16,
}

pub struct PeerConnectionInternal {
    conn_id: ConnectionId,
    to_id: PeerId,
    ctx: SharedCtx,
    remote: SocketAddr,
    connection: Connection,
    framed: Framed<P2pQuicStream, BincodeCodec<PeerMessage>>,
    main_tx: Sender<MainEvent>,
    control_rx: Receiver<PeerConnectionControl>,
    pending_unicast_acks: HashMap<UnicastAckId, (tokio::sync::oneshot::Sender<anyhow::Result<()>>, Instant)>,
    pending_accept_bi: Arc<Semaphore>,
    ticker: Interval,
    started: Instant,
}

impl PeerConnectionInternal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: SharedCtx,
        conn_id: ConnectionId,
        to_id: PeerId,
        connection: Connection,
        main_send: SendStream,
        main_recv: RecvStream,
        main_tx: Sender<MainEvent>,
        control_rx: Receiver<PeerConnectionControl>,
    ) -> Self {
        let stream = P2pQuicStream::new(main_recv, main_send);

        Self {
            conn_id,
            to_id,
            ctx,
            remote: connection.remote_address(),
            connection,
            framed: Framed::new(stream, peer_message_codec()),
            main_tx,
            control_rx,
            pending_unicast_acks: HashMap::new(),
            pending_accept_bi: Arc::new(Semaphore::new(MAX_PENDING_ACCEPT_BI)),
            ticker: tokio::time::interval(Duration::from_secs(1)),
            started: Instant::now(),
        }
    }

    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                _ = self.ticker.tick() => {
                    self.expire_pending_unicast_acks();
                    let rtt_ms = self.connection.rtt().as_millis().min(u16::MAX as u128) as u16;
                    let connection_stats = self.connection.stats();
                    self.ctx.router().set_direct(self.conn_id, self.to_id, rtt_ms);
                    let metrics = PeerConnectionMetric {
                        uptime: self.started.elapsed().as_secs(),
                        sent_pkt: connection_stats.path.sent_packets,
                        lost_pkt: connection_stats.path.lost_packets,
                        lost_bytes: connection_stats.path.lost_bytes,
                        rtt: rtt_ms,
                        send_bytes: connection_stats.udp_tx.bytes,
                        recv_bytes: connection_stats.udp_rx.bytes,
                        current_mtu: connection_stats.path.current_mtu,
                    };

                    gauge!(P2P_CONNECTION_RTT, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).set(metrics.rtt as f64);
                    counter!(P2P_CONNECTION_UPTIME, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(metrics.uptime);
                    counter!(P2P_CONNECTION_SENT_BYTES, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(metrics.send_bytes);
                    counter!(P2P_CONNECTION_RECV_BYTES, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(metrics.recv_bytes);
                    counter!(P2P_CONNECTION_LOST_BYTES, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(metrics.lost_bytes);
                    counter!(P2P_CONNECTION_LOST_PKT, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(metrics.lost_pkt);
                    counter!(P2P_CONNECTION_CONGESTION_EVENTS, "peer_id" => self.ctx.local_id().to_string(), "connect_to" => format!("{}", self.to_id)).absolute(connection_stats.path.congestion_events);
                    let _ = self.main_tx.try_send(MainEvent::PeerStats(self.conn_id, self.to_id, metrics));
                },
                open = self.connection.accept_bi() => {
                    let (send, recv) = open?;
                    self.on_accept_bi(send, recv).await?;
                },
                frame = self.framed.next() => {
                    let msg = frame.ok_or(anyhow!("peer main stream ended"))??;
                    self.on_msg(msg).await?;
                },
                control = self.control_rx.recv() => {
                    let control = control.ok_or(anyhow!("peer control channel ended"))?;
                    self.on_control(control).await?;
                }
            }
        }
    }

    async fn on_accept_bi(&mut self, send: SendStream, recv: RecvStream) -> anyhow::Result<()> {
        log::info!("[PeerConnectionInternal {}] on new bi", self.remote);
        let Ok(permit) = self.pending_accept_bi.clone().try_acquire_owned() else {
            log::warn!("[PeerConnectionInternal {}] too many pending inbound stream-connect handshakes", self.remote);
            return Ok(());
        };
        let stream = P2pQuicStream::new(recv, send);
        tokio::spawn(accept_bi(self.to_id, stream, self.ctx.clone(), permit));
        Ok(())
    }

    async fn on_control(&mut self, control: PeerConnectionControl) -> anyhow::Result<()> {
        match control {
            PeerConnectionControl::Send(item, tx) => {
                let res = self.framed.send(item).await.map_err(Into::into);
                if let Some(tx) = tx {
                    let _ = tx.send(res);
                    Ok(())
                } else {
                    res
                }
            }
            PeerConnectionControl::SendUnicastWithAck(ack_id, source, dest, service, data, tx) => {
                let res = self.framed.send(PeerMessage::UnicastWithAck(ack_id, source, dest, service, data)).await.map_err(Into::into);
                match res {
                    Ok(()) => {
                        self.pending_unicast_acks.insert(ack_id, (tx, Instant::now() + UNICAST_ACK_TIMEOUT));
                        Ok(())
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err));
                        Ok(())
                    }
                }
            }
            PeerConnectionControl::OpenStream(service, source, dest, meta, tx) => {
                let remote = self.remote;
                let connection = self.connection.clone();
                tokio::spawn(async move {
                    log::info!("[PeerConnectionInternal {remote}] open_bi for service {service}");
                    let res = open_bi(connection, source, dest, service, meta).await;
                    if let Err(e) = &res {
                        log::error!("[PeerConnectionInternal {remote}] open_bi for service {service} error {e}");
                    } else {
                        log::info!("[PeerConnectionInternal {remote}] open_bi for service {service} success");
                    }
                    tx.send(res).map_err(|_| "internal channel error").print_on_err("[PeerConnectionInternal] answer open_bi");
                });
                Ok(())
            }
            PeerConnectionControl::Close => {
                log::info!("[PeerConnectionInternal {}] close requested", self.remote);
                self.connection.close(VarInt::from_u32(0), b"duplicate peer connection");
                Err(anyhow!("peer connection closed by control"))
            }
        }
    }

    async fn on_msg(&mut self, msg: PeerMessage) -> anyhow::Result<()> {
        match msg {
            PeerMessage::Sync { route, advertise } => {
                if let Err(_e) = self
                    .main_tx
                    .send(MainEvent::PeerData(self.conn_id, self.to_id, PeerMainData::Sync { route, advertise }))
                    .await
                {
                    log::warn!("[PeerConnectionInternal {}] main loop closed", self.remote);
                }
            }
            PeerMessage::PeerStopped(peer_id) => {
                if peer_id != self.to_id {
                    log::warn!("[PeerConnectionInternal {}] ignore peer stopped for {peer_id} from direct peer {}", self.remote, self.to_id);
                    return Ok(());
                }

                if !self.ctx.check_peer_stopped_msg(peer_id) {
                    log::debug!("[PeerConnectionInternal {}] peer stopped {peer_id} already delivered", self.remote);
                    return Ok(());
                }

                for conn in self.ctx.conns().into_iter().filter(|p| !self.to_id.eq(&p.to_id())) {
                    conn.try_send(PeerMessage::PeerStopped(peer_id))
                        .print_on_err("[PeerConnectionInternal] forward peer stopped over peer alias");
                }

                if let Err(_e) = self.main_tx.try_send(MainEvent::PeerStopped(self.conn_id, peer_id)) {
                    log::warn!("[PeerConnectionInternal {}] queue main loop full", self.remote);
                }
            }
            PeerMessage::Broadcast(source, service_id, msg_id, data) => {
                let effective_source = self.to_id;
                if source != effective_source {
                    log::warn!("[PeerConnectionInternal {}] normalize broadcast source {source} to authenticated peer {}", self.remote, self.to_id);
                }

                if self.ctx.check_broadcast_msg(effective_source, service_id, msg_id) {
                    for conn in self.ctx.conns().into_iter().filter(|p| !self.to_id.eq(&p.to_id())) {
                        conn.try_send(PeerMessage::Broadcast(effective_source, service_id, msg_id, data.clone()))
                            .print_on_err("[PeerConnectionInternal] broadcast data over peer alias");
                    }

                    if let Some(service) = self.ctx.get_service(&service_id) {
                        log::debug!("[PeerConnectionInternal {}] broadcast msg {msg_id} to service {service_id}", self.remote);
                        let _ = send_local_service_event(self.remote, service_id, &service, P2pServiceEvent::Broadcast(effective_source, data)).await;
                    } else {
                        log::warn!("[PeerConnectionInternal {}] broadcast msg to unknown service {service_id}", self.remote);
                    }
                } else {
                    log::debug!("[PeerConnectionInternal {}] broadcast msg {msg_id} already deliveried", self.remote);
                }
            }
            PeerMessage::Unicast(source, dest, service_id, data) => {
                let effective_source = self.to_id;
                if source != effective_source {
                    log::warn!(
                        "[PeerConnectionInternal {}] normalize forged unicast source {source} from authenticated peer {}",
                        self.remote,
                        self.to_id
                    );
                }

                match unicast_route_decision(self.ctx.router().action(&dest), self.conn_id) {
                    UnicastRouteDecision::Local => {
                        if let Some(service) = self.ctx.get_service(&service_id) {
                            let _ = send_local_service_event(self.remote, service_id, &service, P2pServiceEvent::Unicast(effective_source, data)).await;
                        } else {
                            log::warn!("[PeerConnectionInternal {}] service {service_id} not found", self.remote);
                        }
                    }
                    UnicastRouteDecision::Forward(next) => {
                        if let Some(conn) = self.ctx.conn(&next) {
                            conn.try_send(PeerMessage::Unicast(effective_source, dest, service_id, data))
                                .print_on_err("[PeerConnectionInternal] send data over peer alias");
                        } else {
                            log::warn!("[PeerConnectionInternal {}] peer {next} not found", self.remote);
                        }
                    }
                    UnicastRouteDecision::DropIngressLoop(next) => {
                        log::warn!("[PeerConnectionInternal {}] drop unicast relay to {dest}: next hop {next} is ingress connection", self.remote);
                    }
                    UnicastRouteDecision::NoRoute => {
                        log::warn!("[PeerConnectionInternal {}] path to {dest} not found", self.remote);
                    }
                }
            }
            PeerMessage::UnicastWithAck(ack_id, source, dest, service_id, data) => {
                let effective_source = self.to_id;
                if source != effective_source {
                    log::warn!(
                        "[PeerConnectionInternal {}] normalize forged acked unicast source {source} from authenticated peer {}",
                        self.remote,
                        self.to_id
                    );
                }

                let res = match unicast_route_decision(self.ctx.router().action(&dest), self.conn_id) {
                    UnicastRouteDecision::Local => {
                        if let Some(service) = self.ctx.get_service(&service_id) {
                            send_local_service_event(self.remote, service_id, &service, P2pServiceEvent::Unicast(effective_source, data)).await
                        } else {
                            log::warn!("[PeerConnectionInternal {}] service {service_id} not found", self.remote);
                            Err(anyhow!("service not found"))
                        }
                    }
                    UnicastRouteDecision::Forward(next) => {
                        log::warn!("[PeerConnectionInternal {}] reject acked unicast relay to {dest}: next hop {next} is not local", self.remote);
                        Err(anyhow!("acked unicast relay is unsupported"))
                    }
                    UnicastRouteDecision::DropIngressLoop(next) => {
                        log::warn!("[PeerConnectionInternal {}] drop acked unicast relay to {dest}: next hop {next} is ingress connection", self.remote);
                        Err(anyhow!("acked unicast ingress loop"))
                    }
                    UnicastRouteDecision::NoRoute => {
                        log::warn!("[PeerConnectionInternal {}] path to {dest} not found", self.remote);
                        Err(anyhow!("route not found"))
                    }
                };
                let ack = res.map_err(|err| err.to_string());
                self.framed.send(PeerMessage::UnicastAck(ack_id, ack)).await?;
            }
            PeerMessage::UnicastAck(ack_id, result) => {
                if let Some((tx, _)) = self.pending_unicast_acks.remove(&ack_id) {
                    let _ = tx.send(result.map_err(|err| anyhow!(err)));
                } else {
                    log::debug!("[PeerConnectionInternal {}] ignore unknown unicast ack {ack_id}", self.remote);
                }
            }
        }
        Ok(())
    }

    fn expire_pending_unicast_acks(&mut self) {
        let now = Instant::now();
        let expired: Vec<_> = self.pending_unicast_acks.iter().filter_map(|(ack_id, (_, deadline))| (*deadline <= now).then_some(*ack_id)).collect();
        for ack_id in expired {
            if let Some((tx, _)) = self.pending_unicast_acks.remove(&ack_id) {
                let _ = tx.send(Err(anyhow!("unicast ack timed out")));
            }
        }
    }
}

fn peer_message_codec() -> BincodeCodec<PeerMessage> {
    BincodeCodec::with_max_frame_length(MAX_PEER_MESSAGE_FRAME)
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum UnicastRouteDecision {
    Local,
    Forward(ConnectionId),
    DropIngressLoop(ConnectionId),
    NoRoute,
}

pub(super) fn unicast_route_decision(action: Option<RouteAction>, ingress: ConnectionId) -> UnicastRouteDecision {
    match action {
        Some(RouteAction::Local) => UnicastRouteDecision::Local,
        Some(RouteAction::Next(next)) if next == ingress => UnicastRouteDecision::DropIngressLoop(next),
        Some(RouteAction::Next(next)) => UnicastRouteDecision::Forward(next),
        None => UnicastRouteDecision::NoRoute,
    }
}

async fn send_local_service_event(remote: SocketAddr, service_id: P2pServiceId, service: &Sender<P2pServiceEvent>, event: P2pServiceEvent) -> anyhow::Result<()> {
    match tokio::time::timeout(LOCAL_SERVICE_DELIVERY_TIMEOUT, service.send(event)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => {
            log::warn!("[PeerConnectionInternal {remote}] send local service {service_id} msg failed: {err}");
            Err(anyhow!("service closed"))
        }
        Err(_) => {
            log::warn!("[PeerConnectionInternal {remote}] send local service {service_id} msg timed out");
            Err(anyhow!("service queue full"))
        }
    }
}

async fn open_bi(connection: Connection, source: PeerId, dest: PeerId, service: P2pServiceId, meta: Vec<u8>) -> anyhow::Result<P2pQuicStream> {
    tokio::time::timeout(OPEN_BI_TIMEOUT, async {
        let (send, recv) = connection.open_bi().await?;
        let mut stream = P2pQuicStream::new(recv, send);
        write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &StreamConnectReq { source, dest, service, meta }).await?;
        let res = wait_object::<_, StreamConnectRes, MAX_CONTROL_STREAM_PKT>(&mut stream).await?;
        res.map(|_| stream).map_err(|e| anyhow!("{e}"))
    })
    .await
    .map_err(|_| anyhow!("open_bi stream setup timed out"))?
}

async fn accept_bi(authenticated_ingress_peer: PeerId, mut stream: P2pQuicStream, ctx: SharedCtx, _permit: OwnedSemaphorePermit) -> anyhow::Result<()> {
    let req = tokio::time::timeout(ACCEPT_BI_INITIAL_REQ_TIMEOUT, wait_object::<_, StreamConnectReq, MAX_CONTROL_STREAM_PKT>(&mut stream))
        .await
        .map_err(|_| anyhow!("stream connect request timed out"))??;
    let StreamConnectReq { dest, source, service, meta } = req;
    let effective_source = authenticated_ingress_peer;
    if source != effective_source {
        log::warn!("[PeerConnectionInternal {authenticated_ingress_peer}] normalize forged stream source {source} from authenticated peer {effective_source}");
    }

    match ctx.router().action(&dest) {
        Some(RouteAction::Local) => {
            if let Some(service_tx) = ctx.get_service(&service) {
                log::info!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => process local");
                let permit = match tokio::time::timeout(LOCAL_SERVICE_DELIVERY_TIMEOUT, service_tx.reserve()).await {
                    Ok(Ok(permit)) => permit,
                    Ok(Err(_)) => {
                        log::warn!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => service closed");
                        write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Err::<(), _>("service closed".to_string())).await?;
                        return Err(anyhow!("service closed"));
                    }
                    Err(_) => {
                        log::warn!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => service queue full");
                        write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Err::<(), _>("service queue full".to_string())).await?;
                        return Err(anyhow!("service queue full"));
                    }
                };
                write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Ok::<_, String>(())).await?;
                permit.send(P2pServiceEvent::Stream(effective_source, meta, stream));
                Ok(())
            } else {
                log::warn!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => service not found");
                write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Err::<(), _>("service not found".to_string())).await?;
                Err(anyhow!("service not found"))
            }
        }
        Some(RouteAction::Next(next)) => {
            if let Some(alias) = ctx.conn(&next) {
                log::info!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => forward to {next}");
                write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Ok::<_, String>(())).await?;
                match alias.open_stream(service, effective_source, dest, meta).await {
                    Ok(mut next_stream) => {
                        log::info!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => start copy_bidirectional");
                        match copy_bidirectional(&mut next_stream, &mut stream).await {
                            Ok(stats) => {
                                log::info!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} done {stats:?}");
                            }
                            Err(err) => {
                                log::error!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} err {err}");
                            }
                        }
                        Ok(())
                    }
                    Err(err) => {
                        log::error!("[PeerConnectionInternal {authenticated_ingress_peer}] stream service {service} source {effective_source} to dest {dest} => open bi error {err}");
                        Err(err)
                    }
                }
            } else {
                log::warn!(
                    "[PeerConnectionInternal {authenticated_ingress_peer}] new stream with service {service} source {effective_source} to dest {dest} => but connection for next {next} not found"
                );
                write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Err::<(), _>("route not found".to_string())).await?;
                Err(anyhow!("route not found"))
            }
        }
        None => {
            log::warn!("[PeerConnectionInternal {authenticated_ingress_peer}] new stream with service {service} source {effective_source} to dest {dest} => but route path not found");
            write_object::<_, _, MAX_CONTROL_STREAM_PKT>(&mut stream, &Err::<(), _>("route not found".to_string())).await?;
            Err(anyhow!("route not found"))
        }
    }
}
