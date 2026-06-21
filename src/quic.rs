use std::{net::SocketAddr, sync::Arc, time::Duration};

use quinn::{ClientConfig, Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

const MAX_CONCURRENT_MAIN_STREAMS: u32 = 1;
const MAX_CONCURRENT_APP_STREAMS: u32 = 16;
const MAX_CONCURRENT_BIDI_STREAMS: u32 = MAX_CONCURRENT_MAIN_STREAMS + MAX_CONCURRENT_APP_STREAMS;

pub fn make_server_endpoint(bind_addr: SocketAddr, priv_key: PrivatePkcs8KeyDer<'static>, cert: CertificateDer<'static>) -> anyhow::Result<Endpoint> {
    let server_config = configure_server(priv_key, cert.clone())?;
    let client_config = configure_client(&[cert])?;
    let mut endpoint = Endpoint::server(server_config, bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// Returns default server configuration along with its certificate.
fn configure_server(priv_key: PrivatePkcs8KeyDer<'static>, cert: CertificateDer<'static>) -> anyhow::Result<ServerConfig> {
    let cert_chain = vec![cert];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).expect("should get mut");
    transport_config.max_concurrent_uni_streams(10_000_u32.into());
    transport_config.max_concurrent_bidi_streams(MAX_CONCURRENT_BIDI_STREAMS.into());
    transport_config.max_idle_timeout(Some(Duration::from_secs(5).try_into().expect("Should config timeout")));

    Ok(server_config)
}

fn configure_client(server_certs: &[CertificateDer]) -> anyhow::Result<ClientConfig> {
    let mut certs = rustls::RootCertStore::empty();
    for cert in server_certs {
        certs.add(cert.clone())?;
    }
    let mut config = ClientConfig::with_root_certificates(Arc::new(certs))?;

    let mut transport = TransportConfig::default();
    transport.keep_alive_interval(Some(Duration::from_secs(1)));
    transport.max_idle_timeout(Some(Duration::from_secs(5).try_into().expect("Should config timeout")));
    transport.max_concurrent_bidi_streams(MAX_CONCURRENT_BIDI_STREAMS.into());
    transport.max_concurrent_uni_streams(10_000_u32.into());
    config.transport_config(Arc::new(transport));
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, UdpSocket};

    use futures::AsyncWriteExt;

    use crate::CERT_DOMAIN_NAME;

    use super::*;

    pub const DEFAULT_CLUSTER_CERT: &[u8] = include_bytes!("../certs/dev.cluster.cert");
    pub const DEFAULT_CLUSTER_KEY: &[u8] = include_bytes!("../certs/dev.cluster.key");

    #[tokio::test]
    async fn test_open_stream() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());

        let server = make_server_endpoint(SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 10001), priv_key.clone_key(), cert.clone()).unwrap();

        tokio::spawn(async move {
            while let Some(conn) = server.accept().await {
                tokio::spawn(async move {
                    let conn = conn.await.unwrap();
                    let mut queue = vec![];
                    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
                        let mut buf = vec![0; 1024];
                        let len = recv.read(&mut buf).await.unwrap().unwrap();
                        assert_eq!(&buf[..len], b"Hello, world!");
                        send.write_all(b"Hello, world!").await.unwrap();
                        send.finish().unwrap();
                        send.flush().await.unwrap();
                        queue.push((send, recv));
                    }
                });
            }
        });

        let client = make_server_endpoint(SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 10002), priv_key, cert).unwrap();
        let conn = client.connect(SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 10001), CERT_DOMAIN_NAME).unwrap().await.unwrap();
        let mut queue = vec![];
        for i in 0..MAX_CONCURRENT_APP_STREAMS {
            println!("test_open_stream {i}");
            let (mut send, mut recv) = conn.open_bi().await.unwrap();
            send.write_all(b"Hello, world!").await.unwrap();
            let mut buf = vec![0; 1024];
            let len = recv.read(&mut buf).await.unwrap().unwrap();
            assert_eq!(&buf[..len], b"Hello, world!");
            queue.push((send, recv));
        }
    }

    #[tokio::test]
    async fn unused_unidirectional_streams_must_not_be_admitted() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server_addr = UdpSocket::bind("127.0.0.1:0").expect("server udp should bind").local_addr().expect("server addr should exist");
        let client_addr = UdpSocket::bind("127.0.0.1:0").expect("client udp should bind").local_addr().expect("client addr should exist");
        let priv_key = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
        let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());

        let server = make_server_endpoint(server_addr, priv_key.clone_key(), cert.clone()).expect("server endpoint should build");
        let client = make_server_endpoint(client_addr, priv_key, cert).expect("client endpoint should build");

        let server_task = tokio::spawn(async move {
            let connecting = server.accept().await.expect("server should accept connection");
            let _conn = connecting.await.expect("server should complete connection");
            std::future::pending::<()>().await;
        });

        let conn = client
            .connect(server_addr, CERT_DOMAIN_NAME)
            .expect("client connect should start")
            .await
            .expect("client should connect");
        let mut opened = 0;

        for _ in 0..17 {
            if tokio::time::timeout(Duration::from_millis(100), conn.open_uni()).await.is_ok() {
                opened += 1;
            }
        }

        server_task.abort();

        assert_eq!(opened, 0, "the P2P protocol does not accept unidirectional QUIC streams, so transport config should not admit them");
    }
}
