use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::anyhow;
use metrics::{counter, gauge};
use peer_internal::PeerConnectionInternal;
use quinn::{Connecting, Connection, Incoming, RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{channel, Sender},
    oneshot,
};

use crate::{
    ctx::SharedCtx,
    msg::P2pServiceId,
    now_ms,
    secure::HandshakeProtocol,
    stream::{wait_object, write_object, P2pQuicStream},
    ConnectionId, PeerId, P2P_CONNECTION_CONGESTION_EVENTS, P2P_CONNECTION_LOST_BYTES, P2P_CONNECTION_LOST_PKT, P2P_CONNECTION_RECV_BYTES, P2P_CONNECTION_RTT, P2P_CONNECTION_SENT_BYTES,
    P2P_CONNECTION_UPTIME, P2P_LIVE_CONNECTION_COUNT,
};

use super::{msg::PeerMessage, MainEvent};

mod peer_alias;
mod peer_internal;

pub use peer_alias::PeerConnectionAlias;
pub use peer_internal::PeerConnectionMetric;

const MAX_CONTROL_PEER_PKT: usize = 60000;

enum PeerConnectionControl {
    Send(PeerMessage, Option<oneshot::Sender<anyhow::Result<()>>>),
    OpenStream(P2pServiceId, PeerId, PeerId, Vec<u8>, oneshot::Sender<anyhow::Result<P2pQuicStream>>),
}

pub struct PeerConnection {
    conn_id: ConnectionId,
    peer_id: Option<PeerId>,
    is_connected: bool,
}

impl PeerConnection {
    pub fn new_incoming<SECURE: HandshakeProtocol>(secure: Arc<SECURE>, local_id: PeerId, incoming: Incoming, main_tx: Sender<MainEvent>, ctx: SharedCtx) -> Self {
        let remote = incoming.remote_address();
        let conn_id = ConnectionId::rand();

        tokio::spawn(async move {
            log::info!("[PeerConnection {conn_id}] wait incoming from {remote}");
            match incoming.await {
                Ok(connection) => {
                    log::info!("[PeerConnection {conn_id}] got connection from {remote}");
                    match connection.accept_bi().await {
                        Ok((send, recv)) => {
                            if let Err(e) = run_connection(secure, ctx, remote, conn_id, local_id, PeerConnectionDirection::Incoming, &connection, send, recv, main_tx.clone()).await {
                                log::error!("[PeerConnection {conn_id}] connection from {remote} error {e}");
                                let _ = main_tx.send(MainEvent::PeerConnectError(conn_id, None, e)).await;
                                let _ = tokio::time::timeout(Duration::from_secs(2), connection.closed()).await;
                            }
                        }
                        Err(err) => main_tx.send(MainEvent::PeerConnectError(conn_id, None, err.into())).await.expect("should send to main"),
                    }
                }
                Err(err) => main_tx.send(MainEvent::PeerConnectError(conn_id, None, err.into())).await.expect("should send to main"),
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
            match connecting.await {
                Ok(connection) => {
                    log::info!("[PeerConnection {conn_id}] connected to {remote}");
                    match connection.open_bi().await {
                        Ok((send, recv)) => {
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
                        Err(err) => main_tx.send(MainEvent::PeerConnectError(conn_id, Some(to_peer), err.into())).await.expect("should send to main"),
                    }
                }
                Err(err) => main_tx.send(MainEvent::PeerConnectError(conn_id, Some(to_peer), err.into())).await.expect("should send to main"),
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

enum PeerConnectionDirection {
    Incoming,
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
    let to_id = if let PeerConnectionDirection::Outgoing(dest) = direction {
        let auth = secure.create_request(local_id, dest, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectReq { from: local_id, to: dest, auth }).await?;
        let res: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await?;
        log::info!("{res:?}");
        match res.result {
            Ok(auth) => {
                if let Err(e) = secure.verify_response(auth, dest, local_id, now_ms()) {
                    return Err(anyhow!("destination auth failure: {e}"));
                }
                dest
            }
            Err(err) => {
                return Err(anyhow!("destination rejected: {err}"));
            }
        }
    } else {
        let req: ConnectReq = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv).await?;
        if let Err(e) = secure.verify_request(req.auth, req.from, req.to, now_ms()) {
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectRes { result: Err(e.clone()) }).await?;
            return Err(anyhow!("destination auth failure: {e}"));
        } else if req.to != local_id {
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(
                &mut send,
                &ConnectRes {
                    result: Err("destination not match".to_owned()),
                },
            )
            .await?;
            return Err(anyhow!("destination wrong"));
        } else {
            let auth = secure.create_response(req.to, req.from, now_ms());
            write_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut send, &ConnectRes { result: Ok(auth) }).await?;
            req.from
        }
    };

    let rtt_ms = connection.rtt().as_millis().min(u16::MAX as u128) as u16;
    let (control_tx, control_rx) = channel(10);
    let alias = PeerConnectionAlias::new(local_id, to_id, conn_id, control_tx);
    let mut internal = PeerConnectionInternal::new(ctx.clone(), conn_id, to_id, connection.clone(), send, recv, main_tx.clone(), control_rx);
    log::info!("[PeerConnection {conn_id}] started {remote}, rtt: {rtt_ms}");
    ctx.register_conn(conn_id, alias);
    gauge!(P2P_LIVE_CONNECTION_COUNT).increment(1);
    if main_tx.send(MainEvent::PeerConnected(conn_id, to_id, rtt_ms)).await.is_err() {
        log::warn!("[PeerConnection {conn_id}] main loop closed before connected event");
        return Ok(());
    }
    log::info!("[PeerConnection {conn_id}] run loop for {remote}");
    if let Err(e) = internal.run_loop().await {
        log::error!("[PeerConnection {conn_id}] {remote} error {e}");
    }
    let _ = main_tx.send(MainEvent::PeerDisconnected(conn_id, to_id)).await;
    log::info!("[PeerConnection {conn_id}] end loop for {remote}");
    ctx.unregister_conn(&conn_id);
    gauge!(P2P_LIVE_CONNECTION_COUNT).decrement(1);
    gauge!(P2P_CONNECTION_RTT, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).set(0);
    counter!(P2P_CONNECTION_UPTIME, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_RTT, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_SENT_BYTES, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_RECV_BYTES, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_LOST_BYTES, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_LOST_PKT, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    counter!(P2P_CONNECTION_CONGESTION_EVENTS, "peer_id" => local_id.to_string(), "connect_to" => format!("{to_id}")).absolute(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::UdpSocket,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    use futures::SinkExt;
    use quinn::{ClientConfig, Endpoint, ServerConfig, TransportConfig, VarInt};
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
    use tokio_util::codec::Framed;

    use crate::{
        discovery::PeerDiscovery,
        neighbours::NetworkNeighbours,
        quic::make_server_endpoint,
        router::{RouteAction, SharedRouterTable},
        stream::BincodeCodec,
        NetworkAddress, P2pNetwork, P2pNetworkConfig, PeerMainData, SharedKeyHandshake, CERT_DOMAIN_NAME,
    };

    const DEFAULT_CLUSTER_CERT: &[u8] = include_bytes!("../certs/dev.cluster.cert");
    const DEFAULT_CLUSTER_KEY: &[u8] = include_bytes!("../certs/dev.cluster.key");

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
        let _conn = PeerConnection::new_incoming(Arc::new(SharedKeyHandshake::from("atm0s")), PeerId::from(1), incoming, main_tx, ctx);
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
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, main_tx, ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(
            &mut send,
            &ConnectReq {
                from: remote_id,
                to: local_id,
                auth,
            },
        )
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
        let conn = PeerConnection::new_incoming(secure.clone(), local_id, incoming, main_tx.clone(), ctx.clone());
        let conn_id = conn.conn_id();
        let connection = connecting.await.expect("client should connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("client should open control stream");

        let auth = secure.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(
            &mut send,
            &ConnectReq {
                from: remote_id,
                to: local_id,
                auth,
            },
        )
        .await
        .expect("client should send connect request");
        let _: ConnectRes = wait_object::<_, _, MAX_CONTROL_PEER_PKT>(&mut recv)
            .await
            .expect("client should receive connect response");

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
        framed
            .send(PeerMessage::Sync { route, advertise })
            .await
            .expect("remote should send a valid sync frame");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let _dummy = main_rx.recv().await.expect("dummy event should drain from the full queue");
        let delivered = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                match main_rx.recv().await.expect("peer task should keep main event channel open") {
                    MainEvent::PeerData(event_conn, remote_peer, PeerMainData::Sync { route, advertise }) => {
                        assert_eq!(event_conn, conn_id, "sync must come from the authenticated connection");
                        assert_eq!(remote_peer, remote_id, "sync must be attributed to the authenticated peer");
                        ctx.router().apply_sync(event_conn, route);
                        let _ = advertise;
                        break;
                    }
                    _ => {}
                }
            }
        })
        .await;

        assert!(delivered.is_ok(), "valid route/discovery sync must be queued or retried instead of dropped when the main event queue is briefly full");
        assert_eq!(
            ctx.router().action(&advertised_peer),
            Some(RouteAction::Next(conn_id)),
            "dropping the sync leaves a valid destination unreachable for later unicast or stream setup"
        );
    }

    #[tokio::test]
    async fn outbound_peer_setup_must_timeout_when_connect_request_write_stalls() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind node udp")
            .local_addr()
            .expect("should read node addr");
        let raw_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind raw peer udp")
            .local_addr()
            .expect("should read raw peer addr");
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
        let server_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind server udp")
            .local_addr()
            .expect("should read server addr");
        let client_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind raw client udp")
            .local_addr()
            .expect("should read raw client addr");
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
        let _conn = PeerConnection::new_incoming(Arc::new(LargeResponseHandshake), local_id, incoming, main_tx, ctx);
        let connection = connecting.await.expect("raw client should complete transport");
        let (mut send, recv) = connection.open_bi().await.expect("raw client should open p2p control stream");
        let auth = LargeResponseHandshake.create_request(remote_id, local_id, now_ms());
        write_object::<_, _, MAX_CONTROL_PEER_PKT>(
            &mut send,
            &ConnectReq {
                from: remote_id,
                to: local_id,
                auth,
            },
        )
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
    async fn send_broadcast_must_not_block_on_full_peer_control_queue() {
        let ctx = SharedCtx::new(PeerId::from(0), SharedRouterTable::new(PeerId::from(0)));
        let (tx, _rx) = channel(1);

        tx.try_send(PeerConnectionControl::Send(PeerMessage::PeerStopped(PeerId::from(99)), None))
            .expect("test control queue should accept first message");

        let alias = PeerConnectionAlias::new(PeerId::from(0), PeerId::from(1), ConnectionId::from(1), tx);
        ctx.register_conn(ConnectionId::from(1), alias);

        let result = tokio::time::timeout(
            Duration::from_millis(50),
            ctx.send_broadcast(P2pServiceId::from(1), vec![1, 2, 3]),
        )
        .await;

        assert!(
            result.is_ok(),
            "send_broadcast must not wait indefinitely on a congested peer control queue"
        );
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

        let result = tokio::time::timeout(
            Duration::from_millis(50),
            ctx.send_unicast(P2pServiceId::from(1), peer, vec![1, 2, 3]),
        )
        .await;

        assert!(
            result.is_ok(),
            "send_unicast must not wait indefinitely on a congested peer control queue"
        );
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

        let result = tokio::time::timeout(
            Duration::from_millis(50),
            ctx.open_stream(P2pServiceId::from(1), peer, b"meta".to_vec()),
        )
        .await;

        assert!(
            result.is_ok(),
            "open_stream must not wait indefinitely on a congested peer control queue"
        );
    }

    #[tokio::test]
    async fn tick_sync_must_not_be_dropped_when_peer_control_queue_is_full() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listen_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind test udp")
            .local_addr()
            .expect("should read test udp addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: PeerId::from(1),
            listen_addr,
            advertise: None,
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
        let listen_addr = UdpSocket::bind("127.0.0.1:0")
            .expect("should bind test udp")
            .local_addr()
            .expect("should read test udp addr");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
        let mut node = P2pNetwork::new(P2pNetworkConfig {
            peer_id: PeerId::from(1),
            listen_addr,
            advertise: Some(NetworkAddress::from(listen_addr)),
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
            node.ctx.register_conn(
                conn,
                PeerConnectionAlias::new(PeerId::from(1), PeerId::from(10 + idx), conn, tx),
            );
        }

        let result = tokio::time::timeout(Duration::from_millis(1500), node.shutdown_gracefully()).await;

        assert!(
            result.is_ok(),
            "graceful shutdown must use a global deadline or parallel notification, not wait one second per congested peer"
        );
    }
}
