use std::net::SocketAddr;

use atm0s_small_p2p::{InboundPeerBindings, P2pNetwork, P2pNetworkConfig, PeerId, SharedKeyHandshake};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

pub const DEFAULT_CLUSTER_CERT: &[u8] = include_bytes!("../certs/dev.cluster.cert");
pub const DEFAULT_CLUSTER_KEY: &[u8] = include_bytes!("../certs/dev.cluster.key");

async fn readme_snippet(addr: SocketAddr, advertise: bool) -> anyhow::Result<()> {
    let priv_key: PrivatePkcs8KeyDer<'_> = PrivatePkcs8KeyDer::from(DEFAULT_CLUSTER_KEY.to_vec());
    let cert = CertificateDer::from(DEFAULT_CLUSTER_CERT.to_vec());
    let seeds = vec![];

    let peer_id = PeerId::from(1);
    let mut network = P2pNetwork::new(P2pNetworkConfig {
        peer_id,
        listen_addr: addr,
        advertise: advertise.then(|| addr.into()),
        // Open-cluster discovery demo; production deployments should configure static inbound bindings.
        inbound_peer_bindings: InboundPeerBindings::insecure_open_cluster(),
        priv_key,
        cert,
        tick_ms: 100,
        seeds,
        secure: SharedKeyHandshake::from("DEFAULT_SECURE_KEY"),
    })
    .await?;

    let service = network.create_service(1.into());
    let _ = service;

    Ok(())
}

fn main() {}
