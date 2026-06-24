# Coding Agent Guide for `atm0s-small-p2p`

This file is for LLM coding agents that need to use or modify this library.
Prefer the patterns here over inventing new integration styles.

## What This Library Is

`atm0s-small-p2p` is an async Rust P2P library built on QUIC (`quinn`). It
provides:

| Layer | Main Types | Purpose |
| --- | --- | --- |
| Network | `P2pNetwork`, `P2pNetworkConfig`, `P2pNetworkRequester` | Peer lifecycle, QUIC connections, discovery, routing |
| Raw service | `P2pService`, `P2pServiceRequester`, `P2pServiceEvent` | Service-scoped unicast, broadcast, and bidirectional streams |
| Streams | `P2pQuicStream` | `AsyncRead + AsyncWrite` wrapper over QUIC bidirectional streams |
| Pubsub | `PubsubService`, `Publisher`, `Subscriber` | Channel-based publish, feedback, and RPC helpers |
| Alias | `AliasService`, `AliasServiceRequester`, `AliasGuard` | Register/find logical aliases and optionally open streams by alias |
| Replicated KV | `ReplicatedKvService`, `KvEvent` | Local-owned key/value replication across peers |
| Metrics | `MetricsService`, `PeerConnectionMetric` | Connection metrics collection |
| Visualization | `VisualizationService` | Topology collection |

## Dependency Setup

The crate depends on Tokio, Quinn, Rustls, Serde, Bincode, metrics, and
Anyhow. Examples use `clap`, `tracing-subscriber`, `poem`, and `serde_yaml`
as dev dependencies.

Before creating nodes, install the Rustls crypto provider once:

```rust
rustls::crypto::ring::default_provider()
    .install_default()
    .expect("install rustls ring provider");
```

## Create A Node

Use a real certificate/key pair. The examples include development certs in
`certs/`, but production code should provide deployment-specific material.

```rust
use atm0s_small_p2p::{
    InboundPeerBindings, P2pNetwork, P2pNetworkConfig, PeerAddress, PeerId,
    SharedKeyHandshake,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

async fn create_node(
    peer_id: PeerId,
    listen_addr: std::net::SocketAddr,
    advertise_addr: Option<std::net::SocketAddr>,
    seeds: Vec<PeerAddress>,
    cert: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
) -> anyhow::Result<P2pNetwork<SharedKeyHandshake>> {
    P2pNetwork::new(P2pNetworkConfig {
        peer_id,
        listen_addr,
        advertise: advertise_addr.map(Into::into),
        inbound_peer_bindings: InboundPeerBindings::static_bindings(seeds.clone()),
        priv_key: key,
        cert,
        tick_ms: 100,
        seeds,
        secure: SharedKeyHandshake::from("shared-secret"),
    })
    .await
}
```

For local demos/tests, examples use:

```rust
inbound_peer_bindings: InboundPeerBindings::insecure_open_cluster()
```

Do not use `insecure_open_cluster()` for production. Use
`InboundPeerBindings::static_bindings(...)` when peer IDs and remote addresses
are known.

## Runtime Rule: Always Drive `recv()`

`P2pNetwork` is not a background runtime by itself. You must continuously call
`p2p.recv().await`; otherwise ticks, accepts, connects, routing sync,
discovery sync, metrics, and disconnect handling stop.

Typical raw-service layout:

```rust
let mut service = p2p.create_service(0.into());

tokio::spawn(async move {
    while let Ok(_event) = p2p.recv().await {}
});

while let Some(event) = service.recv().await {
    match event {
        P2pServiceEvent::Unicast(from, data) => {}
        P2pServiceEvent::Broadcast(from, data) => {}
        P2pServiceEvent::Stream(from, meta, stream) => {}
        P2pServiceEvent::PeerDisconnected(peer) => {}
    }
}
```

When a helper service has its own loop, drive both loops:

```rust
let mut pubsub = PubsubService::new(p2p.create_service(0.into()));
let pubsub_req = pubsub.requester();

tokio::spawn(async move {
    while let Ok(_) = p2p.recv().await {}
});
tokio::spawn(async move {
    let _ = pubsub.run_loop().await;
});
```

## Peer Addresses And Discovery

`PeerAddress` serializes as:

```text
<peer_id>@<socket_addr>
```

Example:

```rust
let seed: PeerAddress = "1@127.0.0.1:9000".parse()?;
```

Discovery behavior:

| Concept | Behavior |
| --- | --- |
| Seeds | Stable dial candidates; not removed by stopped-peer tombstones |
| Advertise address | Gossiped only when dialable; `0.0.0.0` or port `0` is rejected |
| Non-seed stopped peers | Removed and tombstoned for a timeout so stale routes do not immediately return |
| Sync size | Discovery and route syncs are capped internally |

## Connecting Peers

Use `P2pNetworkRequester`:

```rust
let requester = p2p.requester();
requester.connect(peer_addr).await?;
```

`connect` reports errors such as duplicate attempts, closed queues, or failed
authentication. `try_connect` is best-effort and drops the request if the
control queue is full or closed.

## Raw Service API

Create one service per service ID:

```rust
let mut service = p2p.create_service(0.into());
```

Service IDs are backed by a 256-slot table. Use IDs `0..=255`. Duplicate live
registration or out-of-range IDs create an unregistered service; calls on it
return errors.

Send APIs:

| Method | Behavior |
| --- | --- |
| `send_unicast(dest, data).await` | Route to peer and wait for delivery acknowledgement |
| `try_send_unicast(dest, data).await` | Queue without waiting; errors if route/queue unavailable |
| `send_broadcast(data).await` | Send to connected peers; returns accepted peer count |
| `try_send_broadcast(data).await` | Best-effort broadcast; returns accepted peer count |
| `open_stream(dest, meta).await` | Open a bidirectional `P2pQuicStream` to the destination service |

Important constraints:

- Sending to the local node is unsupported and returns an error.
- Unicast/open-stream require a known route; wait until routing has converged
  or use explicit `connect`.
- Backpressure is real. `send_unicast` may wait if destination service queues
  are full. Use timeouts at the application boundary if needed.
- `open_stream` fails rather than reporting success when the destination
  service is closed or cannot accept the stream.
- `P2pServiceRequester` becomes stale after the owning `P2pService` is dropped.

## Streams

`P2pQuicStream` implements Tokio `AsyncRead` and `AsyncWrite`.

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

let mut stream = service.open_stream(dest_peer, b"meta".to_vec()).await?;
stream.write_all(b"hello").await?;
stream.shutdown().await?;
```

Inbound streams arrive as:

```rust
P2pServiceEvent::Stream(from_peer, meta, stream)
```

QUIC configuration currently allows one main control stream plus 16 app
bidirectional streams per connection. Unidirectional streams are disabled.

## Shutdown

Use graceful shutdown when possible:

```rust
p2p.shutdown_gracefully().await;
```

This attempts to notify connected peers with `PeerStopped` before closing the
endpoint. Plain `shutdown()` closes immediately. For alias service shutdown,
also call `AliasServiceRequester::shutdown()` so alias waiters get `None` and
remote peers receive alias shutdown cleanup.

## Security Guidance

The default provided handshake is `SharedKeyHandshake`.

It checks:

- shared secret hash
- expected source/destination peer IDs
- initiator/responder role
- timestamp freshness with limited future skew
- replay protection

Production guidance:

- Use a strong shared secret, not `"insecure"` or example constants.
- Use static inbound peer bindings when possible.
- Keep peer clocks reasonably synchronized; handshakes are time-bound.
- Do not expose metrics/visualization scan responses to untrusted peers.

## Routing Model

Routing is automatic. Each connection exchanges route and discovery syncs on
network ticks.

Key behavior:

| Topic | Detail |
| --- | --- |
| Direct peers | Preferred over relayed paths |
| Relayed paths | Supported up to a bounded hop count |
| Path switching | Uses score margin to avoid noisy route flapping |
| Removed direct conns | Remembered briefly to ignore stale updates |
| Broadcast | Deduplicated by source/service/message ID |

For app code, use `service.router().action(&peer_id)` only as a readiness hint.
Do not mutate routing directly.

## Pubsub Service

Create and run:

```rust
use atm0s_small_p2p::pubsub_service::{PubsubChannelId, PubsubService};

let mut pubsub = PubsubService::new(p2p.create_service(0.into()));
let req = pubsub.requester();
tokio::spawn(async move { let _ = pubsub.run_loop().await; });

let channel: PubsubChannelId = 1000.into();
let mut publisher = req.publisher(channel).await;
let mut subscriber = req.subscriber(channel).await;
```

Publisher:

```rust
publisher.requester().publish(vec![1, 2, 3]).await?;
publisher.requester().publish_ob(&my_value).await?;
let res = publisher
    .requester()
    .publish_rpc("method", request_bytes, std::time::Duration::from_secs(1))
    .await?;
```

Subscriber:

```rust
subscriber.requester().feedback(vec![4, 5, 6]).await?;
subscriber.requester().feedback_ob(&my_value).await?;
```

Event handling:

| Publisher sees | Subscriber sees |
| --- | --- |
| `PeerJoined`, `PeerLeaved` | `PeerJoined`, `PeerLeaved` |
| `Feedback`, `GuestFeedback` | `Publish`, `GuestPublish` |
| `FeedbackRpc`, `GuestFeedbackRpc` | `PublishRpc`, `GuestPublishRpc` |

Use `recv_ob<T>()` for bincode-typed event decoding. It returns deserialize
error variants instead of panicking.

Pubsub notes:

- Local publisher/subscriber pairs work without remote peers.
- Membership is maintained by join/leave messages plus heartbeats.
- RPC methods are bounded in size.
- Pending RPC requests and internal control queues are bounded; handle errors.

## Alias Service

Alias service maps `AliasId` to a node.

```rust
use atm0s_small_p2p::alias_service::{AliasId, AliasService};

let mut alias_service = AliasService::new(p2p.create_service(10.into()));
let alias_req = alias_service.requester();
tokio::spawn(async move { let _ = alias_service.run_loop().await; });

let guard = alias_req.register(AliasId::from(42))?;
let found = alias_req.find(AliasId::from(42)).await;
```

Keep the returned `AliasGuard` alive while the alias should remain registered.
Dropping it unregisters the alias. `find` returns:

- `Some(AliasFoundLocation::Local)`
- `Some(AliasFoundLocation::Hint(peer))`
- `Some(AliasFoundLocation::Scan(peer))`
- `None`

To open a stream by alias:

```rust
let app_service_req = app_service.requester();
let stream_location = alias_req
    .open_stream(AliasId::from(42), app_service_req, b"meta".to_vec())
    .await?;
```

## Replicated KV Service

Use when data is owned by the node that created it and should be readable
locally on every peer. Remote data is removed when the owning peer disconnects
or times out.

```rust
use atm0s_small_p2p::replicate_kv_service::{KvEvent, ReplicatedKvService};

let mut kv = ReplicatedKvService::<u64, u64>::new(
    p2p.create_service(20.into()),
    65_000, // max changed entries retained
    16_000, // max composed packet/page budget
);

kv.set(1, 100);
kv.del(1);

while let Some(event) = kv.recv().await {
    match event {
        KvEvent::Set(owner, key, value) => {}
        KvEvent::Del(owner, key) => {}
    }
}
```

`owner == None` means the change originated locally. `Some(peer)` means remote
data owned by that peer.

## Metrics Service

`MetricsService` emits connection metrics:

```rust
use atm0s_small_p2p::metrics_service::{MetricsService, MetricsServiceEvent};

let mut metrics = MetricsService::new(
    Some(std::time::Duration::from_secs(1)),
    p2p.create_service(30.into()),
    true, // collector
);

while let Ok(event) = metrics.recv().await {
    match event {
        MetricsServiceEvent::OnPeerConnectionMetric(peer, rows) => {}
    }
}
```

Non-collector nodes only answer scans from trusted collectors:

```rust
let metrics = MetricsService::new(None, service, false)
    .with_trusted_scan_collectors([collector_peer]);
```

The crate also registers metrics names via `init_metrics()`, including live
connections, RTT, bytes sent/received, loss, congestion events, and alias cache
metrics.

## Visualization Service

Use for topology collection:

```rust
use atm0s_small_p2p::visualization_service::{
    VisualizationService, VisualizationServiceEvent,
};

let mut viz = VisualizationService::new(
    Some(std::time::Duration::from_secs(5)),
    true, // include local node in output
    p2p.create_service(31.into()),
);
```

Like metrics, scan responders should use `with_trusted_scan_collectors`.

## Testing Patterns

Use ephemeral ports in tests:

```rust
let addr = std::net::UdpSocket::bind("127.0.0.1:0")?.local_addr()?;
```

Start each node loop:

```rust
tokio::spawn(async move {
    while node.recv().await.is_ok() {}
});
```

Wait for routes before sending:

```rust
tokio::time::timeout(std::time::Duration::from_secs(3), async {
    loop {
        if service.router().action(&dest).is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
})
.await?;
```

Useful commands:

```bash
cargo fmt --check
CARGO_BUILD_JOBS=8 cargo clippy --all-targets -- -D warnings
CARGO_BUILD_JOBS=8 cargo test
CARGO_BUILD_JOBS=8 cargo test --lib service
CARGO_BUILD_JOBS=8 cargo test --lib requester
CARGO_BUILD_JOBS=8 cargo test --lib cross_nodes
CARGO_BUILD_JOBS=8 cargo test --lib security
```

Benchmark profiles live in `benchmarks/stream_limit_profiles.yaml`, and reports
are in `docs/stream_limit_benchmark_report.md` plus
`docs/throughput_benchmark_report.md`.

## Common Agent Mistakes To Avoid

| Mistake | Correct Approach |
| --- | --- |
| Creating a network and never calling `recv()` | Spawn or select a loop that continuously calls `p2p.recv().await` |
| Using `insecure_open_cluster()` in production docs/code | Use `InboundPeerBindings::static_bindings(...)` |
| Assuming route availability immediately after node creation | Explicitly connect or wait for `router().action(...)` |
| Ignoring service registration failure | Treat send/open errors as real; service IDs must be `0..=255` and unique while live |
| Dropping `P2pService` while keeping its requester | Requester calls will fail as stale |
| Treating `try_*` sends as reliable | Use `send_*` when the caller needs delivery/backpressure semantics |
| Blocking service loops | Drain service events promptly or spawn handlers |
| Forgetting graceful shutdown | Use `shutdown_gracefully().await` and service-specific shutdown when available |
| Exposing metrics/visualization scans to anyone | Configure trusted collectors |

## Public Re-Exports

The crate root re-exports the most important items:

```rust
use atm0s_small_p2p::{
    alias_service, metrics_service, pubsub_service, replicate_kv_service,
    visualization_service, ConnectionId, InboundPeerBindings, NetworkAddress,
    P2pNetwork, P2pNetworkConfig, P2pNetworkEvent, P2pNetworkRequester,
    P2pQuicStream, P2pService, P2pServiceEvent, P2pServiceRequester,
    PeerAddress, PeerConnectionMetric, PeerId, SharedKeyHandshake,
    SharedRouterTable,
};
```

When in doubt, copy the runtime structure from `examples/kv.rs` or the
cross-node tests instead of building a new architecture.
