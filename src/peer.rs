use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::anyhow;
#[cfg(test)]
use metrics::counter;
use metrics::gauge;
use peer_internal::PeerConnectionInternal;
use quinn::{Connecting, Connection, Incoming, RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{channel, error::TrySendError, Sender},
    oneshot,
};

use crate::{
    ctx::SharedCtx,
    msg::P2pServiceId,
    now_ms,
    secure::HandshakeProtocol,
    stream::{wait_object, write_object, P2pQuicStream},
    ConnectionId, InboundPeerBindings, PeerId, P2P_CONNECTION_RTT, P2P_LIVE_CONNECTION_COUNT,
};
#[cfg(test)]
use crate::{P2P_CONNECTION_CONGESTION_EVENTS, P2P_CONNECTION_LOST_BYTES, P2P_CONNECTION_LOST_PKT, P2P_CONNECTION_RECV_BYTES, P2P_CONNECTION_SENT_BYTES, P2P_CONNECTION_UPTIME};

use super::{
    msg::{PeerMessage, UnicastAckId},
    MainEvent,
};

mod peer_alias;
mod peer_internal;

pub use peer_alias::PeerConnectionAlias;
pub use peer_internal::PeerConnectionMetric;

const MAX_CONTROL_PEER_PKT: usize = 60000;
const PEER_SETUP_TIMEOUT: Duration = Duration::from_secs(1);

enum PeerConnectionControl {
    Send(PeerMessage, Option<oneshot::Sender<anyhow::Result<()>>>),
    SendUnicastWithAck(UnicastAckId, PeerId, PeerId, P2pServiceId, Vec<u8>, oneshot::Sender<anyhow::Result<()>>),
    OpenStream(P2pServiceId, PeerId, PeerId, Vec<u8>, bool, oneshot::Sender<anyhow::Result<P2pQuicStream>>),
    Close,
}

#[cfg(test)]
pub(crate) struct TestCongestedPeerAlias {
    alias: PeerConnectionAlias,
    _rx: tokio::sync::mpsc::Receiver<PeerConnectionControl>,
}

#[cfg(test)]
pub(crate) struct TestPeerAlias {
    alias: PeerConnectionAlias,
    rx: tokio::sync::mpsc::Receiver<PeerConnectionControl>,
}

#[cfg(test)]
impl TestCongestedPeerAlias {
    pub(crate) fn alias(&self) -> PeerConnectionAlias {
        self.alias.clone()
    }
}

#[cfg(test)]
impl TestPeerAlias {
    pub(crate) fn alias(&self) -> PeerConnectionAlias {
        self.alias.clone()
    }

    pub(crate) async fn recv_msg(&mut self) -> Option<PeerMessage> {
        match self.rx.recv().await? {
            PeerConnectionControl::Send(msg, _) => Some(msg),
            PeerConnectionControl::SendUnicastWithAck(_, source, dest, service, data, _) => Some(PeerMessage::Unicast(source, dest, service, data)),
            PeerConnectionControl::OpenStream(..) | PeerConnectionControl::Close => None,
        }
    }
}

#[cfg(test)]
pub(crate) fn test_peer_alias(local_id: PeerId, to_id: PeerId, conn_id: ConnectionId) -> TestPeerAlias {
    let (tx, rx) = channel(16);
    TestPeerAlias {
        alias: PeerConnectionAlias::new(local_id, to_id, conn_id, tx),
        rx,
    }
}

#[cfg(test)]
pub(crate) fn test_congested_peer_alias(local_id: PeerId, to_id: PeerId, conn_id: ConnectionId) -> TestCongestedPeerAlias {
    let (tx, rx) = channel(1);
    tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(u64::MAX)), None))
        .expect("test peer-control queue should accept filler message");
    TestCongestedPeerAlias {
        alias: PeerConnectionAlias::new(local_id, to_id, conn_id, tx),
        _rx: rx,
    }
}

pub struct PeerConnection {
    conn_id: ConnectionId,
    peer_id: Option<PeerId>,
    is_connected: bool,
}

impl PeerConnection {
    pub fn new_incoming<SECURE: HandshakeProtocol>(
        secure: Arc<SECURE>,
        local_id: PeerId,
        incoming: Incoming,
        inbound_peer_bindings: Arc<InboundPeerBindings>,
        main_tx: Sender<MainEvent>,
        ctx: SharedCtx,
    ) -> Self {
        let remote = incoming.remote_address();
        let conn_id = ConnectionId::rand();

        tokio::spawn(async move {
            log::info!("[PeerConnection {conn_id}] wait incoming from {remote}");
            let setup = tokio::time::timeout(PEER_SETUP_TIMEOUT, async {
                let connection = incoming.await?;
                log::info!("[PeerConnection {conn_id}] got connection from {remote}");
                let (send, recv) = connection.accept_bi().await?;
                Ok::<_, anyhow::Error>((connection, send, recv))
            })
            .await;

            match setup {
                Ok(Ok((connection, send, recv))) => {
                    if let Err(e) = run_connection(
                        secure,
                        ctx,
                        remote,
                        conn_id,
                        local_id,
                        PeerConnectionDirection::Incoming(inbound_peer_bindings),
                        &connection,
                        send,
                        recv,
                        main_tx.clone(),
                    )
                    .await
                    {
                        log::error!("[PeerConnection {conn_id}] connection from {remote} error {e}");
                        let _ = main_tx.send(MainEvent::PeerConnectError(conn_id, None, e)).await;
                        let _ = tokio::time::timeout(Duration::from_secs(2), connection.closed()).await;
                    }
                }
                Ok(Err(err)) => report_peer_connect_error(&main_tx, conn_id, None, err).await,
                Err(err) => report_peer_connect_error(&main_tx, conn_id, None, err.into()).await,
            }
        });
        Self {
            conn_id,
            peer_id: None,
            is_connected: false,
        }
    }

    pub fn new_connecting<SECURE: HandshakeProtocol>(secure: Arc<SECURE>, local_id: PeerId, to_peer: PeerId, connecting: Connecting, main_tx: Sender<MainEvent>, ctx: SharedCtx) -> Self {
        let remote = connecting.remote_address();
        let conn_id = ConnectionId::rand();

        tokio::spawn(async move {
            let setup = tokio::time::timeout(PEER_SETUP_TIMEOUT, async {
                let connection = connecting.await?;
                log::info!("[PeerConnection {conn_id}] connected to {remote}");
                let (send, recv) = connection.open_bi().await?;
                Ok::<_, anyhow::Error>((connection, send, recv))
            })
            .await;

            match setup {
                Ok(Ok((connection, send, recv))) => {
                    if let Err(e) = run_connection(
                        secure,
                        ctx,
                        remote,
                        conn_id,
                        local_id,
                        PeerConnectionDirection::Outgoing(to_peer),
                        &connection,
                        send,
                        recv,
                        main_tx.clone(),
                    )
                    .await
                    {
                        log::error!("[PeerConnection {conn_id}] connection to {remote} error {e}");
                        let _ = main_tx.send(MainEvent::PeerConnectError(conn_id, Some(to_peer), e)).await;
                    }
                }
                Ok(Err(err)) => report_peer_connect_error(&main_tx, conn_id, Some(to_peer), err).await,
                Err(err) => report_peer_connect_error(&main_tx, conn_id, Some(to_peer), err.into()).await,
            }
        });
        Self {
            conn_id,
            peer_id: Some(to_peer),
            is_connected: false,
        }
    }

    pub fn conn_id(&self) -> ConnectionId {
        self.conn_id
    }

    pub fn peer_id(&self) -> Option<PeerId> {
        self.peer_id
    }

    pub fn set_connected(&mut self, peer_id: PeerId) {
        if self.peer_id.is_none() {
            self.peer_id = Some(peer_id);
        }
        self.is_connected = true
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
}

async fn report_peer_connect_error(main_tx: &Sender<MainEvent>, conn_id: ConnectionId, peer: Option<PeerId>, err: anyhow::Error) {
    if main_tx.send(MainEvent::PeerConnectError(conn_id, peer, err)).await.is_err() {
        log::warn!("[PeerConnection {conn_id}] main loop closed before connect error event");
    }
}

enum PeerConnectionDirection {
    Incoming(Arc<InboundPeerBindings>),
    Outgoing(PeerId),
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectReq {
    from: PeerId,
    to: PeerId,
    auth: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectRes {
    result: Result<Vec<u8>, String>,
}

#[allow(clippy::too_many_arguments)]
async fn run_connection<SECURE: HandshakeProtocol>(
    secure: Arc<SECURE>,
    ctx: SharedCtx,
    remote: SocketAddr,
    conn_id: ConnectionId,
    local_id: PeerId,
    direction: PeerConnectionDirection,
    connection: &Connection,
    mut send: SendStream,
    mut recv: RecvStream,
    main_tx: Sender<MainEvent>,
) -> anyhow::Result<()> {
    let to_id = tokio::time::timeout(PEER_SETUP_TIMEOUT, authenticate_peer(secure, remote, local_id, direction, &mut send, &mut recv)).await??;

    let rtt_ms = connection.rtt().as_millis().min(u16::MAX as u128) as u16;
    let (control_tx, control_rx) = channel(10);
    let alias = PeerConnectionAlias::new(local_id, to_id, conn_id, control_tx);
    let mut internal = PeerConnectionInternal::new(ctx.clone(), conn_id, to_id, connection.clone(), send, recv, main_tx.clone(), control_rx);
    log::info!("[PeerConnection {conn_id}] started {remote}, rtt: {rtt_ms}");
    ctx.register_conn(conn_id, alias);
    gauge!(P2P_LIVE_CONNECTION_COUNT).increment(1);
    let mut connected_retry = match main_tx.try_send(MainEvent::PeerConnected(conn_id, to_id, rtt_ms)) {
        Ok(()) => None,
        Err(TrySendError::Closed(_)) => {
            log::warn!("[PeerConnection {conn_id}] main loop closed before connected event");
            ctx.unregister_conn(&conn_id);
            emit_connection_teardown_metrics(local_id, to_id);
            return Ok(());
        }
        Err(TrySendError::Full(event)) => {
            let retry_tx = main_tx.clone();
            Some(tokio::spawn(async move {
                if retry_tx.send(event).await.is_err() {
                    log::warn!("[PeerConnection {conn_id}] main loop closed before delayed connected event");
                }
            }))
        }
    };
    log::info!("[PeerConnection {conn_id}] run loop for {remote}");
    if let Err(e) = internal.run_loop().await {
        log::error!("[PeerConnection {conn_id}] {remote} error {e}");
    }
    log::info!("[PeerConnection {conn_id}] end loop for {remote}");
    if let Some(retry) = connected_retry.take() {
        if !retry.is_finished() {
            retry.abort();
        }
    }
    ctx.unregister_conn(&conn_id);
    emit_connection_teardown_metrics(local_id, to_id);
    let _ = main_tx.send(MainEvent::PeerDisconnected(conn_id, to_id)).await;
    Ok(())
}

async fn authenticate_peer<SECURE: HandshakeProtocol>(
    secure: Arc<SECURE>,
    remote: SocketAddr,
    local_id: PeerId,
    direction: PeerConnectionDirection,
    send: &mut SendStream,
    recv: &mut RecvStream,
) -> anyhow::Result<PeerId> {
    match direction {
        PeerConnectionDirection::Outgoing(dest) => {
            let auth = secure.create_request(local_id, dest, now_ms());
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(send, &ConnectReq { from: local_id, to: dest, auth }).await?;
            let res: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(recv).await?;
            log::info!("{res:?}");
            match res.result {
                Ok(auth) => {
                    if let Err(e) = secure.verify_response(auth, dest, local_id, now_ms()) {
                        return Err(anyhow!("destination auth failure: {e}"));
                    }
                    Ok(dest)
                }
                Err(err) => Err(anyhow!("destination rejected: {err}")),
            }
        }
        PeerConnectionDirection::Incoming(inbound_peer_bindings) => {
            let req: ConnectReq = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(recv).await?;
            if req.to != local_id {
                write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                    send,
                    &ConnectRes {
                        result: Err("destination not match".to_owned()),
                    },
                )
                .await?;
                Err(anyhow!("destination wrong"))
            } else if req.from == local_id {
                write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                    send,
                    &ConnectRes {
                        result: Err("source must not match destination".to_owned()),
                    },
                )
                .await?;
                Err(anyhow!("source wrong"))
            } else if !inbound_peer_bindings.is_authorized(req.from, remote) {
                write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                    send,
                    &ConnectRes {
                        result: Err("source not authorized for remote address".to_owned()),
                    },
                )
                .await?;
                Err(anyhow!("source not authorized for remote address"))
            } else if let Err(e) = secure.verify_request(req.auth, req.from, req.to, now_ms()) {
                write_object::<_, _, MAX_CONTROL_PEER_PKT>(send, &ConnectRes { result: Err(e.clone()) }).await?;
                Err(anyhow!("destination auth failure: {e}"))
            } else {
                let auth = secure.create_response(req.to, req.from, now_ms());
                write_object::<_, _, MAX_CONTROL_PEER_PKT>(send, &ConnectRes { result: Ok(auth) }).await?;
                Ok(req.from)
            }
        }
    }
}

fn emit_connection_teardown_metrics(local_id: PeerId, to_id: PeerId) {
    gauge!(P2P_LIVE_CONNECTION_COUNT).decrement(1);
    gauge!(P2P_CONNECTION_RTT, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).set(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        net::UdpSocket,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
    };

    use futures::{SinkExt, StreamExt};
    use metrics::{Counter, CounterFn, Gauge, Histogram, Key, KeyName, Metadata, Recorder, SharedString, Unit};
    use quinn::{ClientConfig, Endpoint, ServerConfig, TransportConfig, VarInt};
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
    use tokio_util::codec::Framed;

    use crate::{
        discovery::PeerDiscovery,
        msg::P2pServiceId,
        neighbours::NetworkNeighbours,
        quic::make_server_endpoint,
        router::{RouteAction, SharedRouterTable},
        service::{
            metrics_service::{encode_scan_for_test as encode_metrics_scan_for_test, MetricsService},
            visualization_service::{encode_scan_for_test as encode_visualization_scan_for_test, VisualizationService},
            P2pService, P2pServiceEvent,
        },
        stream::BincodeCodec,
        InboundPeerBindings, NetworkAddress, P2pNetwork, P2pNetworkConfig, P2pNetworkEvent, PeerAddress, PeerMainData, SharedKeyHandshake, CERT_DOMAIN_NAME,
    };

    const DEFAULT_CLUSTER_CERT: &[u8] = include_bytes!("../certs/dev.cluster.cert");
    const DEFAULT_CLUSTER_KEY: &[u8] = include_bytes!("../certs/dev.cluster.key");

    fn registered_test_service(service_id: P2pServiceId, mut ctx: SharedCtx) -> (P2pService, tokio::sync::mpsc::Sender<P2pServiceEvent>) {
        let (mut service, tx) = P2pService::build(service_id, ctx.clone());
        service.set_registered(ctx.set_service(service_id, tx.clone()));
        (service, tx)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum MetricKind {
        Counter,
        Gauge,
        Histogram,
    }

    #[derive(Default)]
    struct KindCollisionRecorder {
        kinds: Mutex<HashMap<String, MetricKind>>,
        collision: AtomicBool,
    }

    impl KindCollisionRecorder {
        fn record_kind(&self, key: &Key, kind: MetricKind) {
            let mut kinds = self.kinds.lock().expect("test recorder mutex should not be poisoned");
            match kinds.insert(key.name().to_owned(), kind) {
                Some(existing) if existing != kind => self.collision.store(true, Ordering::SeqCst),
                _ => {}
            }
        }

        fn has_collision(&self) -> bool {
            self.collision.load(Ordering::SeqCst)
        }
    }

    impl Recorder for KindCollisionRecorder {
        fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
            self.record_kind(key, MetricKind::Counter);
            Counter::noop()
        }

        fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
            self.record_kind(key, MetricKind::Gauge);
            Gauge::noop()
        }

        fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
            self.record_kind(key, MetricKind::Histogram);
            Histogram::noop()
        }
    }

    #[derive(Default)]
    struct MonotonicCounterRecorder {
        values: Arc<Mutex<HashMap<String, u64>>>,
        decreased: Arc<AtomicBool>,
    }

    impl MonotonicCounterRecorder {
        fn has_decrease(&self) -> bool {
            self.decreased.load(Ordering::SeqCst)
        }
    }

    struct MonotonicCounterHandle {
        name: String,
        values: Arc<Mutex<HashMap<String, u64>>>,
        decreased: Arc<AtomicBool>,
    }

    impl CounterFn for MonotonicCounterHandle {
        fn increment(&self, value: u64) {
            let mut values = self.values.lock().expect("test recorder mutex should not be poisoned");
            let next = values.get(&self.name).copied().unwrap_or_default().saturating_add(value);
            values.insert(self.name.clone(), next);
        }

        fn absolute(&self, value: u64) {
            let mut values = self.values.lock().expect("test recorder mutex should not be poisoned");
            if let Some(previous) = values.insert(self.name.clone(), value) {
                if value < previous {
                    self.decreased.store(true, Ordering::SeqCst);
                }
            }
        }
    }

    impl Recorder for MonotonicCounterRecorder {
        fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
            Counter::from_arc(Arc::new(MonotonicCounterHandle {
                name: key.name().to_string(),
                values: self.values.clone(),
                decreased: self.decreased.clone(),
            }))
        }

        fn register_gauge(&self, _key: &Key, _metadata: &Metadata<'_>) -> Gauge {
            Gauge::noop()
        }

        fn register_histogram(&self, _key: &Key, _metadata: &Metadata<'_>) -> Histogram {
            Histogram::noop()
        }
    }

    #[test]
    fn connection_teardown_must_not_emit_rtt_as_counter() {
        let recorder = KindCollisionRecorder::default();

        metrics::with_local_recorder(&recorder, || {
            gauge!(P2P_CONNECTION_RTT, "peer_id" => "1", "connect_to" => "2").set(10);
            emit_connection_teardown_metrics(PeerId::from(1), PeerId::from(2));
        });

        assert!(
            !recorder.has_collision(),
            "connection teardown must not emit p2p_connection_rtt as a counter after it was emitted and described as a gauge"
        );
    }

    #[test]
    fn connection_teardown_must_not_reset_monotonic_counters() {
        let recorder = MonotonicCounterRecorder::default();

        metrics::with_local_recorder(&recorder, || {
            counter!(P2P_CONNECTION_UPTIME, "peer_id" => "1", "connect_to" => "2").absolute(10);
            counter!(P2P_CONNECTION_SENT_BYTES, "peer_id" => "1", "connect_to" => "2").absolute(1024);
            counter!(P2P_CONNECTION_RECV_BYTES, "peer_id" => "1", "connect_to" => "2").absolute(2048);
            counter!(P2P_CONNECTION_LOST_BYTES, "peer_id" => "1", "connect_to" => "2").absolute(64);
            counter!(P2P_CONNECTION_LOST_PKT, "peer_id" => "1", "connect_to" => "2").absolute(4);
            counter!(P2P_CONNECTION_CONGESTION_EVENTS, "peer_id" => "1", "connect_to" => "2").absolute(2);
            emit_connection_teardown_metrics(PeerId::from(1), PeerId::from(2));
        });

        assert!(!recorder.has_decrease(), "connection teardown must not reset monotonic connection counters to zero");
    }

    struct LargeAuthHandshake;

    impl HandshakeProtocol for LargeAuthHandshake {
        fn create_request(&self, _from: PeerId, _to: PeerId, _now: u64) -> Vec<u8> {
            vec![7; 58_000]
        }

        fn verify_request(&self, _data: Vec<u8>, _expected_from: PeerId, _expected_to: PeerId, _now: u64) -> Result<(), String> {
            Ok(())
        }

        fn create_response(&self, _from: PeerId, _to: PeerId, _now: u64) -> Vec<u8> {
            vec![8; 64]
        }

        fn verify_response(&self, _data: Vec<u8>, _expected_from: PeerId, _expected_to: PeerId, _now: u64) -> Result<(), String> {
            Ok(())
        }
    }

    struct LargeResponseHandshake;

    impl HandshakeProtocol for LargeResponseHandshake {
        fn create_request(&self, _from: PeerId, _to: PeerId, _now: u64) -> Vec<u8> {
            vec![7; 64]
        }

        fn verify_request(&self, _data: Vec<u8>, _expected_from: PeerId, _expected_to: PeerId, _now: u64) -> Result<(), String> {
            Ok(())
        }

        fn create_response(&self, _from: PeerId, _to: PeerId, _now: u64) -> Vec<u8> {
            vec![8; 58_000]
        }

        fn verify_response(&self, _data: Vec<u8>, _expected_from: PeerId, _expected_to: PeerId, _now: u64) -> Result<(), String> {
            Ok(())
        }
    }

    fn make_small_stream_receive_endpoint(bind_addr: SocketAddr, stream_window: u32) -> anyhow::Result<Endpoint> {
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut server_config = ServerConfig::with_single_cert(vec![cert.clone()], priv_key.into())?;
        let mut transport = TransportConfig::default();
        let window = VarInt::from_u32(stream_window);
        transport.stream_receive_window(window);
        transport.receive_window(window);
        transport.max_concurrent_uni_streams(10_000_u32.into());
        transport.max_concurrent_bidi_streams(10_000_u32.into());
        transport.max_idle_timeout(Some(Duration::from_secs(5).try_into().expect("timeout should configure")));
        server_config.transport_config(Arc::new(transport));

        let mut certs = rustls::RootCertStore::empty();
        certs.add(cert)?;
        let mut client_config = ClientConfig::with_root_certificates(Arc::new(certs))?;
        let mut client_transport = TransportConfig::default();
        client_transport.stream_receive_window(window);
        client_transport.receive_window(window);
        client_transport.max_concurrent_uni_streams(10_000_u32.into());
        client_transport.max_concurrent_bidi_streams(10_000_u32.into());
        client_transport.max_idle_timeout(Some(Duration::from_secs(5).try_into().expect("timeout should configure")));
        client_config.transport_config(Arc::new(client_transport));
        let mut endpoint = Endpoint::server(server_config, bind_addr)?;
        endpoint.set_default_client_config(client_config);
        Ok(endpoint)
    }

    #[test]
    fn stale_pending_outgoing_peer_does_not_suppress_reconnect() {
        let peer = PeerId::from(42);
        let conn_id = ConnectionId::from(7);
        let mut neighbours = NetworkNeighbours::default();

        neighbours.insert(
            conn_id,
            PeerConnection {
                conn_id,
                peer_id: Some(peer),
                is_connected: false,
            },
        );

        assert!(
            !neighbours.has_peer(&peer),
            "an unconnected outgoing attempt must not make reconnect logic believe peer {peer} is already connected"
        );
    }

    #[tokio::test]
    async fn incoming_connect_error_after_main_drop_must_not_panic_task() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");

        let panicked = Arc::new(AtomicBool::new(false));
        let previous_hook = std::panic::take_hook();
        let hook_flag = panicked.clone();
        std::panic::set_hook(Box::new(move |info| {
            hook_flag.store(true, Ordering::SeqCst);
            eprintln!("{info}");
        }));

        let ctx = SharedCtx::new(PeerId::from(1), SharedRouterTable::new(PeerId::from(1)));
        let (main_tx, main_rx) = channel(1);
        drop(main_rx);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let _conn = PeerConnection::new_incoming(
            Arc::new(SharedKeyHandshake::from("atm0s")),
            PeerId::from(1),
            incoming,
            Arc::new(InboundPeerBindings::default()),
            main_tx,
            ctx,
        );
        let connected = connecting.await.expect("client should connect");
        connected.close(VarInt::from_u32(0), b"close before p2p control stream");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        while tokio::time::Instant::now() < deadline && !panicked.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        std::panic::set_hook(previous_hook);

        assert!(
            !panicked.load(Ordering::SeqCst),
            "incoming connection error reporting must not panic when the main event receiver has already closed"
        );
    }

    #[tokio::test]
    async fn connect_error_report_after_main_drop_must_not_panic() {
        let (main_tx, main_rx) = channel(1);
        drop(main_rx);

        report_peer_connect_error(&main_tx, ConnectionId::from(1), Some(PeerId::from(2)), anyhow!("connect failed")).await;
    }

    #[tokio::test]
    async fn authenticated_peer_alias_must_be_cleaned_if_main_loop_closed_before_connected_event() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");
        let secure = Arc::new(SharedKeyHandshake::from("atm0s"));
        let local_id = PeerId::from(1);
        let remote_id = PeerId::from(2);
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, main_rx) = channel(1);
        drop(main_rx);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, Arc::new(InboundPeerBindings::insecure_open_cluster()), main_tx, ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: remote_id, to: local_id, auth })
            .await
            .expect("client should send connect request");
        let _ = tokio::time::timeout(Duration::from_millis(200), wait_object::<_, ConnectRes, MAX_CONTROL_PEER_PKT>(&mut recv)).await;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
        while tokio::time::Instant::now() < deadline && ctx.conn(&conn_id).is_none() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        assert!(
            ctx.conn(&conn_id).is_none(),
            "authenticated peer alias must be unregistered when PeerConnected cannot be delivered to the closed main loop"
        );
    }

    #[tokio::test]
    async fn inbound_handshake_must_reject_peer_claiming_local_id() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");
        let secure = Arc::new(SharedKeyHandshake::from("atm0s"));
        let local_id = PeerId::from(1);
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, mut main_rx) = channel(1);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, Arc::new(InboundPeerBindings::default()), main_tx, ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(local_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: local_id, to: local_id, auth })
            .await
            .expect("client should send self-identity connect request");

        let response: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await.expect("server should answer the rejected connect request");
        assert!(response.result.is_err(), "inbound handshake must reject a remote peer claiming the receiver's own PeerId");

        let event = tokio::time::timeout(Duration::from_millis(200), main_rx.recv()).await;
        assert!(
            !matches!(event, Ok(Some(MainEvent::PeerConnected(_, peer, _))) if peer == local_id),
            "self-identity handshake must not emit PeerConnected for the local PeerId"
        );
        assert!(ctx.conn(&conn_id).is_none(), "self-identity handshake must not register a peer alias");
    }

    #[tokio::test]
    async fn inbound_handshake_must_reject_peer_claiming_third_party_id() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");
        let secure = Arc::new(SharedKeyHandshake::from("atm0s"));
        let local_id = PeerId::from(1);
        let claimed_peer = PeerId::from(99);
        let inbound_peer_bindings = Arc::new(InboundPeerBindings::default());
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, mut main_rx) = channel(1);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, inbound_peer_bindings, main_tx, ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(claimed_peer, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(
            &mut send,
            &ConnectReq {
                from: claimed_peer,
                to: local_id,
                auth,
            },
        )
        .await
        .expect("client should send third-party identity connect request");

        let response: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await.expect("server should answer the rejected connect request");
        assert!(response.result.is_err(), "inbound handshake must reject a remote peer claiming an arbitrary third-party PeerId");

        let event = tokio::time::timeout(Duration::from_millis(200), main_rx.recv()).await;
        assert!(
            !matches!(event, Ok(Some(MainEvent::PeerConnected(_, peer, _))) if peer == claimed_peer),
            "third-party identity handshake must not emit PeerConnected for the claimed PeerId"
        );
        assert!(ctx.conn(&conn_id).is_none(), "third-party identity handshake must not register a peer alias");
    }

    #[tokio::test]
    async fn inbound_handshake_must_accept_bound_peer_claim() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");
        let secure = Arc::new(SharedKeyHandshake::from("atm0s"));
        let local_id = PeerId::from(1);
        let remote_id = PeerId::from(2);
        let inbound_peer_bindings = Arc::new(InboundPeerBindings::static_bindings([PeerAddress::new(remote_id, NetworkAddress::from(client_addr))]));
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, mut main_rx) = channel(1);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, inbound_peer_bindings, main_tx, ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: remote_id, to: local_id, auth })
            .await
            .expect("client should send bound identity connect request");

        let response: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await.expect("server should answer the accepted connect request");
        assert!(response.result.is_ok(), "inbound handshake must accept a peer id bound to the observed remote address");

        let event = tokio::time::timeout(Duration::from_millis(200), main_rx.recv()).await;
        assert!(
            matches!(event, Ok(Some(MainEvent::PeerConnected(event_conn, peer, _))) if event_conn == conn_id && peer == remote_id),
            "bound inbound peer must emit PeerConnected for its configured PeerId"
        );
        assert!(ctx.conn(&conn_id).is_some(), "bound inbound peer must register a peer alias");
    }

    #[tokio::test]
    async fn valid_sync_must_survive_full_main_event_queue() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind client udp").local_addr().expect("should read client addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");
        let secure = Arc::new(SharedKeyHandshake::from("atm0s"));
        let local_id = PeerId::from(1);
        let remote_id = PeerId::from(2);
        let advertised_peer = PeerId::from(4);
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, mut main_rx) = channel(1);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, Arc::new(InboundPeerBindings::insecure_open_cluster()), main_tx.clone(), ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: remote_id, to: local_id, auth })
            .await
            .expect("client should send connect request");
        let _: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await.expect("client should receive connect response");

        match main_rx.recv().await.expect("connected event should be delivered") {
            MainEvent::PeerConnected(event_conn, event_peer, rtt) => ctx.router().set_direct(event_conn, event_peer, rtt),
            _ => panic!("expected connected event"),
        }

        main_tx
            .try_send(MainEvent::PeerStopped(ConnectionId::from(999), PeerId::from(999)))
            .expect("test should fill the one-slot main queue");

        let remote_router = SharedRouterTable::new(remote_id);
        remote_router.set_direct(ConnectionId::from(20), advertised_peer, 5);
        let route = remote_router.create_sync(&local_id);
        let advertise = PeerDiscovery::new(vec![]).create_sync_for(now_ms(), &local_id);
        let mut framed = Framed::new(P2pQuicStream::new(recv, send), BincodeCodec::<PeerMessage>::default());
        framed.send(PeerMessage::Sync { route, advertise }).await.expect("remote should send a valid sync frame");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let _dummy = main_rx.recv().await.expect("dummy event should drain from the full queue");
        let delivered = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let MainEvent::PeerData(event_conn, remote_peer, PeerMainData::Sync { route, advertise }) = main_rx.recv().await.expect("peer task should keep main event channel open") {
                    assert_eq!(event_conn, conn_id, "sync must come from the authenticated connection");
                    assert_eq!(remote_peer, remote_id, "sync must be attributed to the authenticated peer");
                    ctx.router().apply_sync(event_conn, route);
                    let _ = advertise;
                    break;
                }
            }
        })
        .await;

        assert!(
            delivered.is_ok(),
            "valid route/discovery sync must be queued or retried instead of dropped when the main event queue is briefly full"
        );
        assert_eq!(
            ctx.router().action(&advertised_peer),
            Some(RouteAction::Next(conn_id)),
            "dropping the sync leaves a valid destination unreachable for later unicast or stream setup"
        );
    }

    #[tokio::test]
    async fn outbound_peer_setup_must_timeout_when_connect_request_write_stalls() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind node udp").local_addr().expect("should read node addr");
        let raw_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind raw peer udp").local_addr().expect("should read raw peer addr");
        let raw_peer = make_small_stream_receive_endpoint(raw_addr, 37).expect("raw peer endpoint should build");
        let raw_peer_id = PeerId::from(2);
        let (stream_accepted_tx, mut stream_accepted_rx) = tokio::sync::oneshot::channel();

        let raw_task = tokio::spawn(async move {
            let connecting = raw_peer.accept().await.expect("raw peer should accept transport");
            let connection = connecting.await.expect("raw peer should complete transport");
            let stream = connection.accept_bi().await.expect("raw peer should accept p2p control stream");
            let _ = stream_accepted_tx.send(());
            let _held_stream = stream;
            std::future::pending::<()>().await;
        });

        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: PeerId::from(1),
            listen_addr,
            advertise: None,
            inbound_peer_bindings: Default::default(),
            priv_key,
            cert,
            tick_ms: 100,
            seeds: vec![],
            secure: LargeAuthHandshake,
        })
        .await
        .expect("node should build");
        let requester = node.requester();
        requester.try_connect((raw_peer_id, raw_addr.into()).into());

        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                tokio::select! {
                    _ = &mut stream_accepted_rx => break,
                    event = node.recv() => {
                        let _ = event.expect("node recv should keep running");
                    }
                }
            }
        })
        .await
        .expect("raw peer should accept the p2p control stream");

        let deadline = tokio::time::Instant::now() + Duration::from_millis(2500);
        while tokio::time::Instant::now() < deadline {
            let _ = tokio::time::timeout(Duration::from_millis(100), node.recv()).await;
        }
        raw_task.abort();

        assert_eq!(
            node.neighbours.len(),
            0,
            "outbound peer setup must time out and remove the pending neighbour when writing ConnectReq stalls behind peer flow control"
        );
    }

    #[tokio::test]
    async fn inbound_peer_setup_must_timeout_when_connect_response_write_stalls() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind server udp").local_addr().expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind raw client udp").local_addr().expect("should read raw client addr");
        let server_priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let server_cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let server = make_server_endpoint(server_addr, server_priv_key, server_cert.clone()).expect("server endpoint should build");
        let client = make_small_stream_receive_endpoint(client_addr, 37).expect("small-window client endpoint should build");
        let local_id = PeerId::from(1);
        let remote_id = PeerId::from(2);
        let ctx = SharedCtx::new(local_id, SharedRouterTable::new(local_id));
        let (main_tx, mut main_rx) = channel(1);

        let connecting = client.connect(server_addr, CERT_DOMAIN_NAME).expect("raw client should start connecting");
        let incoming = server.accept().await.expect("server should accept incoming connection");
        let _conn = PeerConnection::new_incoming(
            Arc::new(LargeResponseHandshake),
            local_id,
            incoming,
            Arc::new(InboundPeerBindings::insecure_open_cluster()),
            main_tx,
            ctx,
        );
        let connection = connecting.await.expect("raw client should complete transport");
        let (mut send, recv) = connection.open_bi().await.expect("raw client should open p2p control stream");
        let auth = LargeResponseHandshake.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: remote_id, to: local_id, auth })
            .await
            .expect("raw client should send connect request");
        let _held_recv = recv;

        let event = tokio::time::timeout(Duration::from_millis(2500), main_rx.recv()).await;

        assert!(
            matches!(event, Ok(Some(MainEvent::PeerConnectError(_, None, _)))),
            "inbound peer setup must time out and report PeerConnectError when writing ConnectRes stalls behind peer flow control"
        );
    }

    #[tokio::test]
    async fn outbound_control_send_must_not_block_peer_read_loop() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind node udp").local_addr().expect("should read node addr");
        let raw_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind raw peer udp").local_addr().expect("should read raw peer addr");
        let raw_peer = make_small_stream_receive_endpoint(raw_addr, 37).expect("raw peer endpoint should build");
        let local_id = PeerId::from(1);
        let raw_peer_id = PeerId::from(2);
        let secure = SharedKeyHandshake::from("atm0s");
        let (raw_framed_tx, mut raw_framed_rx) = tokio::sync::oneshot::channel();

        let raw_task = tokio::spawn(async move {
            let connecting = raw_peer.accept().await.expect("raw peer should accept transport");
            let connection = connecting.await.expect("raw peer should complete transport");
            let (send, recv) = connection.accept_bi().await.expect("raw peer should accept p2p control stream");
            let mut stream = P2pQuicStream::new(recv, send);
            let req: ConnectReq = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut stream).await.expect("raw peer should receive connect request");
            secure.verify_request(req.auth, local_id, raw_peer_id, now_ms()).expect("raw peer should verify connect request");
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                &mut stream,
                &ConnectRes {
                    result: Ok(secure.create_response(raw_peer_id, local_id, now_ms())),
                },
            )
            .await
            .expect("raw peer should write connect response");
            let _ = raw_framed_tx.send(Framed::new(stream, BincodeCodec::<PeerMessage>::default()));
            std::future::pending::<()>().await;
        });

        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: local_id,
            listen_addr,
            advertise: None,
            inbound_peer_bindings: Default::default(),
            priv_key,
            cert,
            tick_ms: 100,
            seeds: vec![],
            secure: SharedKeyHandshake::from("atm0s"),
        })
        .await
        .expect("node should build");
        let requester = node.requester();
        requester.try_connect((raw_peer_id, raw_addr.into()).into());

        let mut raw_framed = tokio::time::timeout(Duration::from_secs(3), async {
            let mut raw_framed = None;
            loop {
                tokio::select! {
                    framed = &mut raw_framed_rx, if raw_framed.is_none() => {
                        raw_framed = Some(framed.expect("raw framed should be sent"));
                    }
                    event = node.recv() => {
                        if let Ok(P2pNetworkEvent::PeerConnected(_, peer)) = event {
                            assert_eq!(peer, raw_peer_id);
                        }
                    }
                }
                if node.ctx.conns().into_iter().next().is_some() {
                    if let Some(framed) = raw_framed {
                        return framed;
                    }
                }
            }
        })
        .await
        .expect("node should connect to raw peer");

        let conn = node.ctx.conns().into_iter().next().expect("node should have raw peer alias");
        for _ in 0..10 {
            conn.try_send(PeerMessage::Broadcast(local_id, P2pServiceId::from(1), crate::msg::BroadcastMsgId::rand(), vec![7; 59_000]))
                .expect("large control message should enqueue");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;

        raw_framed.send(PeerMessage::PeerStopped(raw_peer_id)).await.expect("raw peer should send stop frame");

        let progressed = tokio::time::timeout(Duration::from_millis(750), async {
            loop {
                match node.main_rx.recv().await.expect("main channel should remain open") {
                    MainEvent::PeerStopped(_, peer) if peer == raw_peer_id => break,
                    MainEvent::PeerDisconnected(_, peer) if peer == raw_peer_id => break,
                    _ => {}
                }
            }
        })
        .await;
        raw_task.abort();

        assert!(
            progressed.is_ok(),
            "outbound control writes must not park peer progress; the connection should process later inbound frames or close promptly"
        );
    }

    #[tokio::test]
    async fn pending_unicast_acks_must_be_count_bounded() {
        const MAX_EXPECTED_PENDING_UNICAST_ACKS: usize = 16;

        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind node udp").local_addr().expect("should read node addr");
        let raw_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind raw peer udp").local_addr().expect("should read raw peer addr");
        let raw_peer =
            make_server_endpoint(raw_addr, PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec()), CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec())).expect("raw peer endpoint should build");
        let local_id = PeerId::from(1);
        let raw_peer_id = PeerId::from(2);
        let secure = SharedKeyHandshake::from("atm0s");
        let (raw_framed_tx, mut raw_framed_rx) = tokio::sync::oneshot::channel();

        let raw_task = tokio::spawn(async move {
            let connecting = raw_peer.accept().await.expect("raw peer should accept transport");
            let connection = connecting.await.expect("raw peer should complete transport");
            let (send, recv) = connection.accept_bi().await.expect("raw peer should accept p2p control stream");
            let mut stream = P2pQuicStream::new(recv, send);
            let req: ConnectReq = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut stream).await.expect("raw peer should receive connect request");
            secure.verify_request(req.auth, local_id, raw_peer_id, now_ms()).expect("raw peer should verify connect request");
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                &mut stream,
                &ConnectRes {
                    result: Ok(secure.create_response(raw_peer_id, local_id, now_ms())),
                },
            )
            .await
            .expect("raw peer should write connect response");
            let _ = raw_framed_tx.send(Framed::new(stream, BincodeCodec::<PeerMessage>::default()));
            std::future::pending::<()>().await;
        });

        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: local_id,
            listen_addr,
            advertise: None,
            inbound_peer_bindings: Default::default(),
            priv_key,
            cert,
            tick_ms: 100,
            seeds: vec![],
            secure: SharedKeyHandshake::from("atm0s"),
        })
        .await
        .expect("node should build");
        let requester = node.requester();
        requester.try_connect((raw_peer_id, raw_addr.into()).into());

        let mut raw_framed = tokio::time::timeout(Duration::from_secs(3), async {
            let mut raw_framed = None;
            loop {
                tokio::select! {
                    framed = &mut raw_framed_rx, if raw_framed.is_none() => {
                        raw_framed = Some(framed.expect("raw framed should be sent"));
                    }
                    event = node.recv() => {
                        if let Ok(P2pNetworkEvent::PeerConnected(_, peer)) = event {
                            assert_eq!(peer, raw_peer_id);
                        }
                    }
                }
                if node.ctx.conns().into_iter().next().is_some() {
                    if let Some(framed) = raw_framed {
                        return framed;
                    }
                }
            }
        })
        .await
        .expect("node should connect to raw peer");

        let conn = node.ctx.conns().into_iter().next().expect("node should have raw peer alias");
        let mut sends = Vec::new();
        for idx in 0..(MAX_EXPECTED_PENDING_UNICAST_ACKS + 8) {
            let conn = conn.clone();
            sends.push(tokio::spawn(
                async move { conn.send_unicast_with_ack(local_id, raw_peer_id, P2pServiceId::from(1), vec![idx as u8]).await },
            ));
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let mut admitted = 0usize;
        while let Ok(Some(Ok(PeerMessage::UnicastWithAck(..)))) = tokio::time::timeout(Duration::from_millis(25), raw_framed.next()).await {
            admitted += 1;
        }

        for send in sends {
            send.abort();
        }
        raw_task.abort();

        assert!(
            admitted <= MAX_EXPECTED_PENDING_UNICAST_ACKS,
            "pending unicast ack tracking must be count-bounded before timeout expiry, admitted {admitted} unacked sends"
        );
    }

    #[tokio::test]
    async fn send_broadcast_must_not_block_on_full_peer_control_queue() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx, _rx) = channel(1);

        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept first message");

        let alias = PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx);
        ctx.register_conn(ConnectionId::from(1), alias);

        let result = tokio::time::timeout(Duration::from_millis(50), ctx.send_broadcast(P2pServiceId::from(1), vec![1, 2, 3])).await;

        assert!(result.is_ok(), "send_broadcast must not wait indefinitely on a congested peer control queue");
    }

    #[tokio::test]
    async fn send_broadcast_must_not_wait_one_timeout_per_congested_peer() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let mut held_receivers = Vec::new();

        for idx in 0..8 {
            let (tx, rx) = channel(1);
            tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
                .expect("test control queue should accept first message");
            held_receivers.push(rx);

            let peer = PeerId::from(10 + idx);
            let conn = ConnectionId::from(100 + idx);
            ctx.register_conn(conn, PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx));
        }

        let result = tokio::time::timeout(Duration::from_millis(75), ctx.send_broadcast(P2pServiceId::from(1), vec![1, 2, 3])).await;

        assert!(
            result.is_ok(),
            "send_broadcast must use a global deadline or parallel admission instead of waiting one timeout per congested peer"
        );
    }

    #[tokio::test]
    async fn send_broadcast_must_report_when_all_peer_channels_are_closed() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx1, rx1) = channel(1);
        let (tx2, rx2) = channel(1);
        drop(rx1);
        drop(rx2);

        ctx.register_conn(ConnectionId::from(1), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx1));
        ctx.register_conn(ConnectionId::from(2), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(2), ConnectionId::from(2), tx2));

        let result = ctx.send_broadcast(P2pServiceId::from(1), b"lost-broadcast".to_vec()).await;

        assert!(result.is_err(), "send_broadcast must report an error when every peer control channel is closed, got {result:?}");
    }

    #[test]
    fn peer_stopped_admission_must_not_suppress_new_peer_lifecycle() {
        let local = PeerId::from(0);
        let peer = PeerId::from(1);
        let conn = ConnectionId::from(1);
        let ctx = SharedCtx::new(local, SharedRouterTable::new(local));
        let (tx, _rx) = channel(1);

        assert!(ctx.try_mark_peer_stopped_msg_after(peer, || true));

        ctx.register_conn(conn, PeerConnectionAlias::new(local, peer, conn, tx));

        assert!(
            ctx.try_mark_peer_stopped_msg_after(peer, || true),
            "a PeerStopped dedup mark from an old lifecycle must not suppress a later stop after the peer reconnects"
        );
    }

    #[tokio::test]
    async fn metrics_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx, mut rx) = channel(1);

        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept filler message");

        ctx.register_conn(ConnectionId::from(1), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx));
        let (base_service, _service_tx) = registered_test_service(P2pServiceId::from(7), ctx);
        let mut metrics = MetricsService::new(Some(Duration::from_millis(1)), base_service, true);

        for _ in 0..8 {
            let _ = metrics.recv().await.expect("collector tick should emit local metrics");
        }

        let _filler = rx.recv().await.expect("filler should drain");
        let mut duplicate_scans = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        while tokio::time::Instant::now() < deadline {
            if let Ok(Some(PeerConnectionControl::Send(PeerMessage::Broadcast(_, _, _, _), None))) = tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
                duplicate_scans += 1;
                if duplicate_scans > 1 {
                    break;
                }
            }
        }

        assert!(
            duplicate_scans <= 1,
            "metrics collector must coalesce scan broadcasts while the previous broadcast is still backpressured; got {duplicate_scans}"
        );
    }

    #[tokio::test]
    async fn visualization_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx, mut rx) = channel(1);

        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept filler message");

        ctx.register_conn(ConnectionId::from(1), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx));
        let (base_service, _service_tx) = registered_test_service(P2pServiceId::from(8), ctx);
        let mut visualization = VisualizationService::new(Some(Duration::from_millis(1)), true, base_service);
        let _ = visualization.recv().await.expect("initial local peer event should be emitted");

        for _ in 0..8 {
            let _ = visualization.recv().await.expect("collector tick should emit local topology");
        }

        let _filler = rx.recv().await.expect("filler should drain");
        let mut duplicate_scans = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        while tokio::time::Instant::now() < deadline {
            if let Ok(Some(PeerConnectionControl::Send(PeerMessage::Broadcast(_, _, _, _), None))) = tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
                duplicate_scans += 1;
                if duplicate_scans > 1 {
                    break;
                }
            }
        }

        assert!(
            duplicate_scans <= 1,
            "visualization collector must coalesce scan broadcasts while the previous broadcast is still backpressured; got {duplicate_scans}"
        );
    }

    #[tokio::test]
    async fn metrics_scan_response_must_not_be_dropped_when_peer_control_queue_is_full() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let peer = PeerId::from(1);
        let (tx, mut rx) = channel(1);

        router.set_direct(conn, peer, 0);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept filler message");
        ctx.register_conn(conn, PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx));

        let (base_service, service_tx) = registered_test_service(P2pServiceId::from(9), ctx);
        let mut metrics = MetricsService::new(None, base_service, false).with_trusted_scan_collectors([peer]);
        service_tx
            .send(P2pServiceEvent::Broadcast(peer, encode_metrics_scan_for_test()))
            .await
            .expect("test service queue should accept scan");

        let _ = tokio::time::timeout(Duration::from_millis(20), metrics.recv()).await;
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }

        let _filler = rx.recv().await.expect("filler should drain");
        let found = tokio::time::timeout(Duration::from_millis(100), async {
            while let Some(control) = rx.recv().await {
                if matches!(control, PeerConnectionControl::Send(PeerMessage::Unicast(_, _, _, _), None)) {
                    return true;
                }
            }
            false
        })
        .await
        .unwrap_or(false);

        assert!(
            found,
            "metrics Scan response must be queued, retried, or backpressured instead of dropped when the peer control queue is briefly full"
        );
    }

    #[tokio::test]
    async fn visualization_scan_responses_must_not_accumulate_behind_full_peer_control_queue() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let peer = PeerId::from(1);
        let (tx, mut rx) = channel(1);

        router.set_direct(conn, peer, 0);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept filler message");
        ctx.register_conn(conn, PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx));

        let (base_service, service_tx) = registered_test_service(P2pServiceId::from(10), ctx);
        let mut visualization = VisualizationService::new(None, false, base_service).with_trusted_scan_collectors([peer]);
        for _ in 0..8 {
            service_tx
                .send(P2pServiceEvent::Broadcast(peer, encode_visualization_scan_for_test()))
                .await
                .expect("test service queue should accept scan");
            let _ = tokio::time::timeout(Duration::from_millis(20), visualization.recv()).await;
        }
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }

        let _filler = rx.recv().await.expect("filler should drain");
        let mut responses = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        while tokio::time::Instant::now() < deadline {
            if let Ok(Some(PeerConnectionControl::Send(PeerMessage::Unicast(_, _, _, _), None))) = tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
                responses += 1;
                if responses > 1 {
                    break;
                }
            }
        }

        assert_eq!(
            responses, 1,
            "visualization Scan responses must be retried and coalesced while the previous response is still backpressured; got {responses}"
        );
    }

    #[tokio::test]
    async fn metrics_scan_responses_must_not_accumulate_behind_full_peer_control_queue() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let peer = PeerId::from(1);
        let (tx, mut rx) = channel(1);

        router.set_direct(conn, peer, 0);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept filler message");
        ctx.register_conn(conn, PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx));

        let (base_service, service_tx) = registered_test_service(P2pServiceId::from(11), ctx);
        let mut metrics = MetricsService::new(None, base_service, false).with_trusted_scan_collectors([peer]);
        for _ in 0..8 {
            service_tx
                .send(P2pServiceEvent::Broadcast(peer, encode_metrics_scan_for_test()))
                .await
                .expect("test service queue should accept scan");
            let _ = tokio::time::timeout(Duration::from_millis(20), metrics.recv()).await;
        }
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }

        let _filler = rx.recv().await.expect("filler should drain");
        let mut responses = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        while tokio::time::Instant::now() < deadline {
            if let Ok(Some(PeerConnectionControl::Send(PeerMessage::Unicast(_, _, _, _), None))) = tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
                responses += 1;
                if responses > 1 {
                    break;
                }
            }
        }

        assert!(
            responses <= 1,
            "metrics Scan responses must be coalesced while the previous response is still backpressured; got {responses}"
        );
    }

    #[tokio::test]
    async fn try_send_broadcast_must_report_when_all_peer_queues_reject() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx1, mut rx1) = channel(1);
        let (tx2, mut rx2) = channel(1);

        tx1.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(91)), None))
            .expect("test control queue should accept first filler message");
        tx2.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(92)), None))
            .expect("test control queue should accept second filler message");

        ctx.register_conn(ConnectionId::from(1), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx1));
        ctx.register_conn(ConnectionId::from(2), PeerConnectionAlias::new(PeerId::from(0), PeerId::from(2), ConnectionId::from(2), tx2));

        let result = ctx.try_send_broadcast(P2pServiceId::from(1), b"lost-broadcast".to_vec());

        let mut delivered = 0;
        for rx in [&mut rx1, &mut rx2] {
            while let Ok(control) = rx.try_recv() {
                if matches!(control, PeerConnectionControl::Send(PeerMessage::Broadcast(..), _)) {
                    delivered += 1;
                }
            }
        }

        assert!(
            result.is_err(),
            "try_send_broadcast must report an error when every peer control queue rejects the broadcast, got {result:?}"
        );
        assert!(delivered == 0, "try_send_broadcast should not report all-failed delivery while a broadcast was actually queued");
    }

    #[tokio::test]
    async fn send_unicast_must_not_block_on_full_peer_control_queue() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let peer = PeerId::from(1);
        let (tx, _rx) = channel(1);

        router.set_direct(conn, peer, 0);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept first message");

        let alias = PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx);
        ctx.register_conn(conn, alias);

        let result = tokio::time::timeout(Duration::from_millis(50), ctx.send_unicast(P2pServiceId::from(1), peer, vec![1, 2, 3])).await;

        assert!(result.is_ok(), "send_unicast must not wait indefinitely on a congested peer control queue");
    }

    #[tokio::test]
    async fn send_unicast_to_relay_must_not_block_on_full_peer_control_queue() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let relay = PeerId::from(1);
        let dest = PeerId::from(2);
        let (tx, _rx) = channel(1);

        router.set_direct(conn, relay, 0);
        let remote_router = SharedRouterTable::new(relay);
        remote_router.set_direct(ConnectionId::from(99), dest, 1);
        router.apply_sync(conn, remote_router.create_sync(&PeerId::from(0)));
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept first message");

        let alias = PeerConnectionAlias::new(PeerId::from(0), relay, conn, tx);
        ctx.register_conn(conn, alias);

        let result = tokio::time::timeout(Duration::from_millis(50), ctx.send_unicast(P2pServiceId::from(1), dest, vec![1, 2, 3])).await;

        assert!(result.is_ok(), "send_unicast to a relayed destination must not wait indefinitely on a congested peer control queue");
    }

    #[tokio::test]
    async fn open_stream_must_not_block_on_full_peer_control_queue() {
        let router = SharedRouterTable::new(PeerId::from(0));
        let ctx = SharedCtx::new(PeerId::from(0), router.clone());
        let conn = ConnectionId::from(1);
        let peer = PeerId::from(1);
        let (tx, _rx) = channel(1);

        router.set_direct(conn, peer, 0);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept first message");

        let alias = PeerConnectionAlias::new(PeerId::from(0), peer, conn, tx);
        ctx.register_conn(conn, alias);

        let result = tokio::time::timeout(Duration::from_millis(50), ctx.open_stream(P2pServiceId::from(1), peer, b"meta".to_vec())).await;

        assert!(result.is_ok(), "open_stream must not wait indefinitely on a congested peer control queue");
    }

    #[tokio::test]
    async fn unicast_relay_must_not_forward_back_to_ingress_peer() {
        let local = PeerId::from(2);
        let ingress_peer = PeerId::from(1);
        let destination = PeerId::from(99);
        let ingress_conn = ConnectionId::from(10);
        let ctx = SharedCtx::new(local, SharedRouterTable::new(local));
        let (tx, _rx) = channel(1);

        ctx.register_conn(ingress_conn, PeerConnectionAlias::new(local, ingress_peer, ingress_conn, tx));
        ctx.router().set_direct(ingress_conn, ingress_peer, 10);

        let remote_router = SharedRouterTable::new(ingress_peer);
        remote_router.set_direct(ConnectionId::from(99), destination, 10);
        ctx.router().apply_sync(ingress_conn, remote_router.create_sync(&local));

        assert_eq!(
            ctx.router().action(&destination),
            Some(RouteAction::Next(ingress_conn)),
            "test setup must expose the bad route state: destination routes back to ingress"
        );
        assert_eq!(
            peer_internal::unicast_route_decision(ctx.router().action(&destination), ingress_conn),
            peer_internal::UnicastRouteDecision::DropIngressLoop(ingress_conn),
            "inbound unicast forwarding must drop instead of sending back over ingress"
        );
    }

    #[tokio::test]
    async fn tick_sync_must_not_be_dropped_when_peer_control_queue_is_full() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind test udp").local_addr().expect("should read test udp addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: PeerId::from(1),
            listen_addr,
            advertise: None,
            inbound_peer_bindings: Default::default(),
            priv_key,
            cert,
            tick_ms: 100,
            seeds: vec![],
            secure: SharedKeyHandshake::from("atm0s"),
        })
        .await
        .expect("should create test node");
        let conn = ConnectionId::from(10);
        let peer = PeerId::from(2);

        let (tx, mut rx) = channel(1);
        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test should fill peer control queue");

        node.neighbours.insert(
            conn,
            PeerConnection {
                conn_id: conn,
                peer_id: Some(peer),
                is_connected: true,
            },
        );
        node.router.set_direct(conn, peer, 10);
        node.ctx.register_conn(conn, PeerConnectionAlias::new(node.local_id, peer, conn, tx));

        node.process_tick(100).expect("tick should process");

        let _dummy = rx.recv().await.expect("dummy should drain");
        let delivered = tokio::time::timeout(Duration::from_millis(100), async {
            while let Some(control) = rx.recv().await {
                if matches!(control, PeerConnectionControl::Send(PeerMessage::Sync { .. }, None)) {
                    break;
                }
            }
        })
        .await;

        assert!(
            delivered.is_ok(),
            "route/discovery tick sync must be queued, coalesced, or retried instead of dropped when the peer control queue is briefly full"
        );
    }

    #[tokio::test]
    async fn shutdown_gracefully_must_not_wait_one_second_per_congested_peer() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0").expect("should bind test udp").local_addr().expect("should read test udp addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: PeerId::from(1),
            listen_addr,
            advertise: Some(NetworkAddress::from(listen_addr)),
            inbound_peer_bindings: Default::default(),
            priv_key,
            cert,
            tick_ms: 100,
            seeds: vec![],
            secure: SharedKeyHandshake::from("atm0s"),
        })
        .await
        .expect("should create test node");

        let mut held_receivers = Vec::new();
        for idx in 0..2 {
            let (tx, rx) = channel(1);
            tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
                .expect("test peer control queue should be filled");
            held_receivers.push(rx);

            let conn = ConnectionId::from(100 + idx);
            node.ctx.register_conn(conn, PeerConnectionAlias::new(PeerId::from(1), PeerId::from(10 + idx), conn, tx));
        }

        let result = tokio::time::timeout(Duration::from_millis(1500), node.shutdown_gracefully()).await;

        assert!(
            result.is_ok(),
            "graceful shutdown must use a global deadline or parallel notification, not wait one second per congested peer"
        );
    }
}
