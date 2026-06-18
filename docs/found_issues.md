# Found Issues

RED-team issue ledger for `atm0s-small-p2p`.

Acceptance rule: an issue is listed here only after reviewer confirmation and
test-case evidence. The tests listed below are expected to fail on the current
audited code.

## Audit Status

- Current consecutive no-new-issue cycles: 0
- Stop condition requested by user: continue until 5 consecutive cycles find no
  new accepted issue.

## Accepted Issues

### ISSUE-001: Forged third-party `PeerStopped` removes a live peer

- Category: security, correctness
- Reviewer: `Leibniz`, confirmed. Also confirmed by `Bernoulli` and `Wegener`.
- Affected code:
  - `src/msg.rs`: `PeerMessage::PeerStopped(PeerId)` carries a free peer id.
  - `src/peer/peer_internal.rs`: accepts and forwards any stopped peer id.
  - `src/lib.rs`: trusts `MainEvent::PeerStopped(conn, peer)` and removes the supplied peer.
  - `src/router.rs`: `del_peer` deletes route state for the supplied peer.
- Impact: any authenticated peer can claim that a third-party peer stopped and
  cause route/discovery removal across the mesh.
- Evidence test:
  - `cargo test forged_peer_stopped_must_not_remove_third_party_route -- --nocapture`
  - Failure summary: route to victim becomes `None`; expected `Some(Next(ConnectionId(20)))`.

### ISSUE-002: Future-dated handshake timestamps are accepted

- Category: security
- Reviewer: `Goodall`, confirmed.
- Affected code:
  - `src/secure.rs`: `validate_handshake` rejects only timestamps older than
    `HANDSHAKE_TIMEOUT`; it has no bounded future-skew check.
- Impact: a valid future-dated handshake token can be accepted before its time
  window and replayed for longer than intended.
- Evidence test:
  - `cargo test rejects_arbitrarily_future_request_timestamp -- --nocapture`
  - Failure summary: verification succeeds for a request timestamped
    `1_000_000_000` while verifier time is `1_000`.

### ISSUE-003: Active route flaps on tiny RTT jitter or equal-cost updates

- Category: stability, bad-network correctness
- Reviewer: `Wegener`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: raw QUIC RTT is fed into routing every tick.
  - `src/router.rs`: route score is recomputed immediately with no hysteresis,
    hold-down, minimum improvement threshold, or equal-cost stability.
  - `src/ctx.rs` and `src/peer/peer_internal.rs`: stream opening follows the
    current route, so flapping can make pipes choose unstable next hops.
- Impact: active paths can jump between alternatives on tiny metric changes,
  making routing noisy and stream/pipe setup unstable.
- Evidence tests:
  - `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`
  - Failure summary: active route switches from `ConnectionId(1)` to
    `ConnectionId(2)` for a 2ms jitter swing.
  - `cargo test should_keep_existing_best_path_on_equal_score -- --nocapture`
  - Failure summary: equal-cost route switches from `ConnectionId(2)` to
    `ConnectionId(1)` due to map ordering.

### ISSUE-004: `PeerStopped(seed)` preserves seed discovery but deletes seed route

- Category: correctness, security
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/discovery.rs`: `remove_remote` ignores tombstones for configured seeds.
  - `src/lib.rs`: still calls `router.del_peer(&peer)` for all stopped peers.
  - `src/router.rs`: has no seed/non-seed distinction.
- Impact: a forged or forwarded stop for a configured seed can temporarily
  blackhole traffic to that seed even though seeds must remain retryable.
- Evidence test:
  - `cargo test peer_stopped_for_seed_must_not_remove_active_seed_route -- --nocapture`
  - Failure summary: route to seed becomes `None`; expected active seed route.

### ISSUE-005: Discovery accepts advertisements for the local peer id

- Category: correctness, security
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/discovery.rs`: `apply_sync` inserts advertised peers without rejecting
    `peer == local`.
  - `src/lib.rs`: tick processing dials every discovered remote candidate.
- Impact: a peer can advertise the receiver's own peer id at an arbitrary
  address, causing self-id remote candidates and confusing connection behavior.
- Evidence test:
  - `cargo test apply_sync_rejects_local_peer_advertisement -- --nocapture`
  - Failure summary: discovery stores a remote candidate for the local peer id.

### ISSUE-006: Router stores and advertises routes to the local peer

- Category: correctness, route-poisoning stability
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/router.rs`: `apply_sync` stores every advertised peer and filters only
    the direct source peer, not `self.peer_id`.
  - `src/router.rs`: `create_sync` can advertise stored local-id routes.
- Impact: a neighbor can inject a route to the receiver's own id; the bogus
  route can be re-advertised to other peers and encourage loops.
- Evidence test:
  - `cargo test should_not_store_or_advertise_route_to_local_peer -- --nocapture`
  - Failure summary: `next_remote(local)` returns a route learned from a peer.

### ISSUE-007: Over-`MAX_HOPS` routes are still usable for forwarding

- Category: correctness, stability under route loops
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/router.rs`: over-hop routes are filtered only in `create_sync`, not
    rejected during `apply_sync` or `action`.
  - `src/peer/peer_internal.rs`: unicast forwarding has no message TTL.
- Impact: paths above the loop-control threshold can still be selected for
  local forwarding, so looped routes can carry traffic until queues or
  connections fail.
- Evidence test:
  - `cargo test should_reject_over_max_hops_for_forwarding -- --nocapture`
  - Failure summary: route with `relay_hops: 7` is still selected.

### ISSUE-008: Routes learned from a peer are advertised back to that peer

- Category: stability, bad-network loop risk
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/router.rs`: `create_sync(dest)` filters only `addr != dest` and hop
    count; it does not apply split-horizon by next-hop connection.
- Impact: peers can feed each other stale relay paths after topology changes.
  Hop count eventually limits advertisements, but temporary loops and noisy
  route churn remain possible.
- Evidence test:
  - `cargo test should_not_advertise_route_back_to_next_hop -- --nocapture`
  - Failure summary: route learned from peer1 is advertised back to peer1.

### ISSUE-009: Untrusted discovery timestamps can overflow or create immortal peers

- Category: security, correctness
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/discovery.rs`: computes `last_updated + TIMEOUT_AFTER` and
    `stopped_at + TIMEOUT_AFTER` on untrusted timestamps.
- Impact: in debug builds, `u64::MAX` timestamps panic. In release builds,
  wrapping can make expiration decisions wrong; far-future timestamps can keep
  non-seed peers alive far longer than intended.
- Evidence test:
  - `cargo test apply_sync_rejects_overflowing_future_timestamp -- --nocapture`
  - Failure summary: test catches an overflow panic at timestamp validation.

### ISSUE-010: Route sync payloads are unbounded at application level

- Category: high-load stability, resource exhaustion
- Reviewer: `Bernoulli`, confirmed.
- Affected code:
  - `src/stream.rs`: main control stream uses `LengthDelimitedCodec::default()`.
  - `src/msg.rs`: sync messages contain unbounded vectors.
  - `src/router.rs`: `apply_sync` allocates route memory for every advertised entry.
  - `src/discovery.rs`: `apply_sync` iterates every discovery entry.
- Impact: a connected peer can send very large sync vectors and force memory,
  CPU, and log growth. The 60 KB object limit does not protect framed control
  messages.
- Evidence test:
  - `cargo test should_reject_excessive_route_sync_entries -- --nocapture`
  - Failure summary: a 1,100-entry route sync is accepted and advertised,
    exceeding the test cap of 1,024.

### ISSUE-011: `open_stream` succeeds after destination service receiver is closed

- Category: correctness, pipe reliability
- Reviewer: `Linnaeus`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: local stream delivery uses
    `service_acceptor.try_send(...)` but ignores its `Result`.
  - `src/peer/peer_internal.rs`: after ignoring the failed local delivery, it
    still sends `StreamConnectRes { result: Ok(...) }` to the opener.
- Impact: the opener receives an apparently valid stream even though no
  destination service can ever accept the pipe.
- Evidence test:
  - `cargo test open_stream_fails_when_destination_service_receiver_is_closed -- --nocapture`
  - Failure summary: `open_stream` returns `Ok(_)` after the destination
    service receiver has been dropped.

### ISSUE-012: `open_stream` succeeds when destination service queue is full

- Category: high-load stability, pipe reliability
- Reviewer: `Pasteur`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: local stream delivery uses a bounded service
    acceptor queue and ignores `try_send` failure.
  - `src/ctx.rs`: service acceptor queues are bounded to 10 pending accepted
    streams.
- Impact: under load, a peer can report stream setup success for a pipe that
  was never handed to the destination service.
- Evidence test:
  - `cargo test open_stream_does_not_succeed_when_destination_service_queue_is_full -- --nocapture`
  - Failure summary: the 11th stream reports success while the destination
    service queue is already full and unconsumed.

### ISSUE-013: `open_stream` to the local peer panics instead of returning an error

- Category: correctness, API stability
- Reviewer: `Kuhn`, confirmed.
- Affected code:
  - `src/service.rs`: service `open_stream` APIs delegate directly to
    `P2pCtx::open_stream`.
  - `src/router.rs`: routing to the local peer returns `RouteAction::Local`.
  - `src/ctx.rs`: the local-route branch panics with
    `unsupported open_stream to local node`.
- Impact: an invalid but plausible destination id can crash the caller task
  instead of returning a recoverable `Err`.
- Evidence test:
  - `cargo test open_stream_to_local_returns_error_not_panic -- --nocapture`
  - Failure summary: the call unwinds at `src/ctx.rs:211`; the test expected
    `Ok(Err(_))`.

### ISSUE-014: Unicast sender identity is not bound to the authenticated connection

- Category: security, correctness
- Reviewer: `Carson`, confirmed.
- Affected code:
  - `src/msg.rs`: `PeerMessage::Unicast(source, dest, ...)` carries a
    peer-controlled `source`.
  - `src/peer/peer_internal.rs`: inbound unicast delivery and forwarding trust
    the message-body `source` instead of the authenticated connection peer.
  - `src/service.rs`: `P2pServiceEvent::Unicast` exposes the trusted-looking
    sender id directly to service consumers.
- Impact: any connected peer can impersonate another peer to application
  services, including multi-hop services that make authorization or replication
  decisions based on the reported sender id.
- Evidence test:
  - `cargo test unicast_source_must_be_bound_to_authenticated_connection_peer -- --nocapture`
  - Failure summary: node2 receives
    `P2pServiceEvent::Unicast(PeerId(99), ...)` from a message sent over
    node1's authenticated connection.

### ISSUE-015: Broadcast sender identity is not bound to the authenticated connection

- Category: security, correctness
- Reviewer: `Carson`, confirmed. Also confirmed by external review subagent
  `019eda94-71c2-73c1-b06f-0b40ff01a1a7`.
- Affected code:
  - `src/msg.rs`: `PeerMessage::Broadcast(source, ...)` carries a
    peer-controlled `source`.
  - `src/peer/peer_internal.rs`: inbound broadcast forwarding and local
    delivery trust the message-body `source`.
  - `src/service.rs`: `P2pServiceEvent::Broadcast` exposes that sender id to
    service consumers.
- Impact: any connected peer can impersonate another broadcaster to services
  such as pubsub, alias, metrics, visualization, and replicated KV.
- Evidence test:
  - `cargo test broadcast_source_must_be_bound_to_authenticated_connection_peer -- --nocapture`
  - Failure summary: node2 receives
    `P2pServiceEvent::Broadcast(PeerId(99), ...)` from a message sent over
    node1's authenticated connection.

### ISSUE-016: `connect()` can report success before peer identity is authenticated

- Category: correctness, API stability
- Reviewer: `Maxwell`, confirmed.
- Affected code:
  - `src/requester.rs`: `P2pNetworkRequester::connect` waits only for the
    control-loop oneshot.
  - `src/lib.rs`: `process_control` replies `Ok(())` after QUIC dialing is
    spawned.
  - `src/peer.rs`: remote peer id/auth verification happens later in
    `run_connection` and failures are reported asynchronously as
    `PeerConnectError`.
- Impact: callers can treat a wrong-peer or failed-auth connection as
  successful, then send traffic into a route that never becomes authenticated.
- Evidence test:
  - `cargo test connect_must_fail_when_remote_peer_id_does_not_match_address -- --nocapture`
  - Failure summary: `connect(99@node1_addr)` returns `Ok(())` before the
    remote endpoint proves it is peer 99.

### ISSUE-017: Broadcast duplicate suppression is keyed only by message id

- Category: security, correctness, bad-network stability
- Reviewer: `Pascal`, confirmed.
- Affected code:
  - `src/ctx.rs`: `received_broadcast_msg` is an `LruCache<BroadcastMsgId, ()>`.
  - `src/ctx.rs`: `check_broadcast_msg` marks only the message id as delivered,
    without source or service in the cache key.
  - `src/peer/peer_internal.rs`: inbound broadcasts call
    `check_broadcast_msg(msg_id)` before forwarding or local delivery.
- Impact: a malicious or buggy peer can preempt a broadcast id and suppress a
  later broadcast using the same id from another source or service. Combined
  with the forged-source issue, this lets an attacker impersonate and poison
  duplicate state before a legitimate broadcast arrives.
- Evidence test:
  - `cargo test broadcast_dedup_must_include_source_not_only_message_id -- --nocapture`
  - Failure summary: after a forged first broadcast with `PeerId(99)`, the
    later broadcast from `PeerId(1)` with the same `BroadcastMsgId` times out
    instead of being delivered.

### ISSUE-018: Stream sender identity is not bound to the authenticated connection

- Category: security, correctness, pipe reliability
- Reviewer: `Rawls`, confirmed.
- Affected code:
  - `src/msg.rs`: `StreamConnectReq` carries a peer-controlled `source`.
  - `src/peer/peer_internal.rs`: `open_bi` writes the caller-supplied `source`,
    and `accept_bi` delivers that `source` to the destination service unchanged.
  - `src/service.rs`: `P2pServiceEvent::Stream` exposes the trusted-looking
    stream source id directly to service consumers.
- Impact: any connected peer can open a pipe while impersonating another peer.
  Services that authorize streams or bind stream state by sender id can be
  tricked into trusting the wrong remote identity.
- Evidence test:
  - `cargo test stream_source_must_be_bound_to_authenticated_connection_peer -- --nocapture`
  - Failure summary: node2 receives
    `P2pServiceEvent::Stream(PeerId(99), ...)` from a stream opened over
    node1's authenticated connection.

### ISSUE-019: Alias local registration refcount overflows after 255 guards

- Category: high-load stability, correctness
- Reviewer: `Ohm`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: local alias registrations are counted in
    `HashMap<AliasId, u8>`.
  - `src/service/alias_service.rs`: `AliasControl::Register` increments the
    `u8` refcount with `*ref_count += 1`.
  - `src/service/alias_service.rs`: `AliasControl::Unregister` relies on that
    refcount to decide when to remove and broadcast `NotifyDel`.
- Impact: 256 live guards for the same alias panic in debug builds and wrap in
  release builds, corrupting alias lifetime accounting and causing later drops
  to remove or advertise the alias incorrectly.
- Evidence test:
  - `cargo test registering_same_alias_many_times_must_not_overflow_refcount -- --nocapture`
  - Failure summary: the 256th registration panics at the `u8` increment with
    `attempt to add with overflow`.

### ISSUE-020: Pubsub RPC answers are accepted by `RpcId` only

- Category: security, correctness
- Reviewer: `Popper`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PublishRpcAnswer(data, rpc_id)`
    removes `publish_rpc_reqs[rpc_id]` and completes the caller without
    checking the answering peer.
  - `src/service/pubsub_service.rs`: inbound `FeedbackRpcAnswer(data, rpc_id)`
    has the same behavior for feedback RPCs.
  - `src/service/pubsub_service.rs`: pending request structs store only
    timeout and response channel, not expected peer/channel/method metadata.
- Impact: any connected peer that learns or races a valid `RpcId` can complete
  another peer's pending pubsub RPC with attacker-controlled data.
- Evidence test:
  - `cargo test pubsub_publish_rpc_answer_must_be_bound_to_expected_responder -- --nocapture`
  - Failure summary: node3 injects `PublishRpcAnswer(..., rpc_id)` and node1's
    `publish_rpc` completes with `forged-rpc-answer` even though node3 was not
    the subscriber handling the RPC.

### ISSUE-021: Handshake timeout check overflows on maximum timestamp

- Category: security, correctness
- Reviewer: `Dewey`, confirmed.
- Affected code:
  - `src/secure.rs`: `validate_handshake` computes
    `handshake_data.timestamp + HANDSHAKE_TIMEOUT` on a signed but
    peer-controlled timestamp.
- Impact: a peer with the shared key can send a valid handshake timestamped at
  `u64::MAX`; debug builds panic on the addition, while release builds wrap and
  make timeout validation incorrect.
- Evidence test:
  - `cargo test rejects_overflowing_request_timestamp_without_panic -- --nocapture`
  - Failure summary: verification panics at the timeout addition with
    `attempt to add with overflow` instead of returning `Err`.

### ISSUE-022: Alias shutdown from one peer clears all cached aliases

- Category: security, correctness
- Reviewer: `Archimedes`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasMessage::Shutdown` iterates over and
    removes every alias cache entry.
  - `src/service/alias_service.rs`: the shutdown branch ignores the sender
    peer id, unlike `NotifyDel`, which removes only the sender from an alias's
    peer set.
- Impact: any peer that can send alias service messages can evict alias hints
  learned from unrelated peers, forcing needless scans and disrupting alias
  lookups across the cluster.
- Evidence test:
  - `cargo test shutdown_from_one_peer_must_not_clear_aliases_from_other_peers -- --nocapture`
  - Failure summary: an alias learned from `peer2` is removed after `peer1`
    sends `Shutdown`.

### ISSUE-023: Replicated KV `FetchChanged` version arithmetic overflows

- Category: security, correctness
- Reviewer: `Dirac`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`: remote
    `RpcReq::FetchChanged { from, count }` is passed to `changeds_from_to`.
  - `src/service/replicate_kv_service/local_storage.rs`: `changeds_from_to`
    computes `from + count.min(...)`.
  - `src/service/replicate_kv_service/messages.rs`: `Version + u64` performs
    raw `u64` addition.
- Impact: a malicious peer can request changes from `Version(u64::MAX)` and
  crash debug builds or trigger wrapped range logic in release builds.
- Evidence test:
  - `cargo test fetch_changed_with_overflowing_from_version_must_not_panic -- --nocapture`
  - Failure summary: the remote fetch path panics at `Version::add` with
    `attempt to add with overflow`.

### ISSUE-024: Peer message codec lacks the 60 KB application payload cap

- Category: high-load stability, resource exhaustion
- Reviewer: `Descartes`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: the main peer stream uses
    `Framed<P2pQuicStream, BincodeCodec<PeerMessage>>`.
  - `src/stream.rs`: `BincodeCodec` uses `LengthDelimitedCodec::default()`
    without configuring the project-level application max.
  - `src/msg.rs`: `PeerMessage::Unicast` and `PeerMessage::Broadcast` carry
    arbitrary `Vec<u8>` service payloads.
- Impact: although tokio-util has an 8 MiB default frame cap, service messages
  bypass the 60 KB cap used for stream connect objects. A connected peer can
  force multi-MiB frame allocation/deserialization and broadcast forwarding
  clones.
- Evidence test:
  - `cargo test peer_message_codec_must_reject_oversized_service_payloads -- --nocapture`
  - Failure summary: a 70 KB unicast service payload is encoded successfully
    instead of being rejected before framing.

### ISSUE-025: Replicated KV `FetchSnapshot` reversed bounds panic

- Category: security, correctness
- Reviewer: `Hilbert`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: unicast RPC requests from peers are
    passed into `ReplicatedKvStore::on_remote_event`.
  - `src/service/replicate_kv_service/local_storage.rs`: remote
    `RpcReq::FetchSnapshot { from, to, max_version }` is passed directly to
    `snapshot`.
  - `src/service/replicate_kv_service/local_storage.rs`: `snapshot` calls
    `self.slots.range(from..=to)` without checking `from <= to`.
- Impact: a malicious peer can send reversed snapshot bounds and panic the
  local service path inside `BTreeMap::range`.
- Evidence test:
  - `cargo test fetch_snapshot_with_reversed_bounds_must_not_panic -- --nocapture`
  - Failure summary: `BTreeMap::range` panics with
    `range start is greater than range end`.

### ISSUE-026: Pubsub heartbeat does not remove stale remote subscribers

- Category: bad-network correctness, stability
- Reviewer: `Euclid`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: the module comment says heartbeat repairs
    out-of-sync channel state.
  - `src/service/pubsub_service.rs`: inbound `Heartbeat` adds remote publishers
    and subscribers when flags are true, but never removes existing remote state
    when flags are false.
  - `src/service/pubsub_service.rs`: explicit `SubscriberLeaved` removes and
    emits `PeerLeaved`, so a lost leave message cannot be repaired by heartbeat.
- Impact: after a lost leave message or bad network period, publishers can keep
  stale remote subscriber destinations forever and never notify local users that
  the peer left the channel.
- Evidence test:
  - `cargo test pubsub_heartbeat_must_remove_stale_remote_subscriber -- --nocapture`
  - Failure summary: after node2 heartbeats `subscribe=false`, node1's
    publisher times out waiting for `PeerLeaved(Remote(node2))`.

### ISSUE-027: Replicated KV stores unbounded future changed broadcasts

- Category: high-load stability, resource exhaustion
- Reviewer: `Socrates`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`: `WorkingState`
    stores future changes in `pendings: BTreeMap<Version, Changed<K, V>>`.
  - `src/service/replicate_kv_service/remote_storage.rs`: inbound
    `BroadcastEvent::Changed` inserts every version greater than the current
    version before attempting to apply contiguous changes.
  - `src/service/replicate_kv_service/remote_storage.rs`: discontinuities send
    a `FetchChanged` request, but do not cap, compact, or resync pending
    future entries.
- Impact: a malicious or buggy peer can send many far-future change broadcasts
  and force unbounded pending memory growth on receivers.
- Evidence test:
  - `cargo test working_state_must_cap_pending_future_changes -- --nocapture`
  - Failure summary: 2,049 far-future changes remain pending, exceeding the
    test cap of 1,024.

### ISSUE-028: Stale network requester panics after network drop

- Category: correctness, API stability
- Reviewer: `Heisenberg`, confirmed.
- Affected code:
  - `src/requester.rs`: `P2pNetworkRequester::connect` calls
    `control_tx.send(...).expect("should send to main loop")`.
  - `src/requester.rs`: `P2pNetworkRequester::try_connect` has the same panic
    path.
- Impact: a cloned requester handle can outlive `P2pNetwork`; using it after
  the control receiver is closed panics through the public API instead of
  returning a recoverable error or no-op.
- Evidence test:
  - `cargo test requester_connect_after_network_drop_returns_error_not_panic -- --nocapture`
  - Failure summary: after dropping `P2pNetwork`, `requester.connect(...)`
    panics at `src/requester.rs:12` with `SendError`.
  - Additional reviewed evidence from `Newton`:
    `cargo test requester_try_connect_after_network_drop_must_not_panic -- --nocapture`
  - Failure summary: after dropping `P2pNetwork`, `requester.try_connect(...)`
    panics at `src/requester.rs:17` with `SendError`.

### ISSUE-029: Stale alias requester panics after service drop

- Category: correctness, API stability
- Reviewer: `Singer`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasServiceRequester::register`,
    `find`, and `shutdown` call `tx.send(...).expect(...)`.
  - `src/service/alias_service.rs`: `AliasGuard::drop` also expects the
    service control channel to remain open.
- Impact: alias requester or guard handles can outlive `AliasService`; using or
  dropping them after the service is gone panics through the public service API.
- Evidence test:
  - `cargo test alias_find_after_service_drop_returns_none_not_panic -- --nocapture`
  - Failure summary: after dropping `AliasService`, `requester.find(...)`
    panics at `src/service/alias_service.rs:99` with `SendError`.

### ISSUE-030: Duplicate service creation panics instead of returning an error

- Category: correctness, API stability
- Reviewer: `Fermat`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::create_service` exposes a public
    `P2pService` return type with no recoverable duplicate-id error path.
  - `src/ctx.rs`: `SharedCtxInternal::set_service` enforces uniqueness with
    `assert!(..., "Service ID already used")`.
- Impact: creating a duplicate service id is a plausible caller error, but it
  unwinds the caller instead of returning `Err` or `None`.
- Evidence test:
  - `cargo test duplicate_service_creation_must_not_panic -- --nocapture`
  - Failure summary: the second `create_service(0.into())` panics at
    `src/ctx.rs:28` with `Service ID already used`.

### ISSUE-031: Replicated KV local version increment overflows

- Category: long-running correctness, stability
- Reviewer: `Sagan`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`: local `set` and `del`
    both advance the store with `self.version = self.version + 1`.
  - `src/service/replicate_kv_service/messages.rs`: `Version::add` uses raw
    `u64` addition.
- Impact: after enough local writes for the store version to reach
  `u64::MAX`, the next write panics in debug builds. In release builds it can
  wrap to zero, breaking monotonic version ordering and corrupting replication
  sync assumptions.
- Evidence test:
  - `cargo test local_set_at_max_version_must_not_overflow -- --nocapture`
  - Failure summary: `LocalStore::set` at `Version(u64::MAX)` panics at
    `src/service/replicate_kv_service/messages.rs:37` with
    `attempt to add with overflow`.

### ISSUE-032: Replicated KV zero snapshot page size stalls full sync

- Category: correctness, bad-network stability
- Reviewer: `Sartre`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvStore::new` accepts
    `max_compose_pkts == 0`.
  - `src/service/replicate_kv_service/local_storage.rs`: `LocalStore::new`
    stores `compose_max_pkts` unchecked.
  - `src/service/replicate_kv_service/local_storage.rs`: `snapshot` checks
    `slots.len() >= self.compose_max_pkts` before pushing any slot, so a zero
    page size returns an empty page with `next_key`.
  - `src/service/replicate_kv_service/remote_storage.rs`: `SyncFullState`
    treats an empty snapshot with `next_key` as progress and requests the same
    page range again.
- Impact: a valid public configuration can make full snapshot sync never
  converge. Nodes can repeatedly exchange empty `FetchSnapshot` pages and retry
  traffic without applying data.
- Evidence test:
  - `cargo test snapshot_with_zero_compose_budget_must_make_progress -- --nocapture`
  - Failure summary: `LocalStore::snapshot` with `compose_max_pkts = 0`
    returns `slots: []` and `next_key: Some(...)`, proving snapshot paging can
    request continuation without advancing.

### ISSUE-033: Router route-sync metric arithmetic overflows

- Category: security, correctness, bad-network stability
- Reviewer: `Schrodinger`, confirmed.
- Affected code:
  - `src/router.rs`: `RouterTableSync` accepts peer-supplied `PathMetric`
    values.
  - `src/router.rs`: `RouterTable::apply_sync` composes advertised metrics with
    the direct-link metric using `new_metric += *direct_metric`.
  - `src/router.rs`: `PathMetric::add_assign` uses raw `u8` and `u16`
    addition.
- Impact: a connected peer can advertise maximum route metrics and panic a
  debug build during route composition. In release builds the arithmetic can
  wrap, turning an invalid route into a potentially low-cost usable path before
  later filtering.
- Evidence test:
  - `cargo test should_reject_overflowing_route_sync_metric_without_panic -- --nocapture`
  - Failure summary: applying a route sync with `(u8::MAX, u16::MAX)` panics at
    `src/router.rs:196` with `attempt to add with overflow`.

### ISSUE-034: Replicated KV full sync accepts future-version snapshot slots

- Category: security, correctness
- Reviewer: `Zeno`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts untrusted
    `RpcRes::FetchSnapshot(Some(snapshot), version)`.
  - `src/service/replicate_kv_service/remote_storage.rs`: snapshot slots are
    emitted as `KvEvent::Set` and inserted without validating
    `slot.version <= version`.
  - `src/service/replicate_kv_service/remote_storage.rs`: the receiver then
    transitions to `WorkingState::new(version)`.
- Impact: a malicious peer can declare a low snapshot version while including
  slots from a higher future version. The receiver stores and emits the data
  while believing the remote is still at the lower version, corrupting
  replication state and later incremental-sync assumptions.
- Evidence test:
  - `cargo test full_sync_must_reject_snapshot_slot_newer_than_declared_version -- --nocapture`
  - Failure summary: a snapshot response declaring `Version(1)` but containing
    `Slot::new(..., Version(99))` is accepted, stored, and emitted instead of
    being rejected.

### ISSUE-035: Alias lookup stores unbounded duplicate waiters

- Category: high-load stability, resource exhaustion
- Reviewer: `Mencius`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `FindRequest.waits` is an unbounded `Vec`.
  - `src/service/alias_service.rs`: duplicate `AliasControl::Find` calls for
    the same alias push another oneshot sender into the existing request and
    return.
  - `src/service/alias_service.rs`: waiters are drained only when the alias is
    found or the scan/hint timeout fires.
- Impact: local callers can issue many concurrent `find` operations for one
  missing or stale alias. The service suppresses duplicate network scans but
  still accumulates arbitrary local waiters and memory under one `find_reqs`
  entry until timeout.
- Evidence test:
  - `cargo test duplicate_find_waiters_for_same_alias_must_be_bounded -- --nocapture`
  - Failure summary: 1,025 duplicate find waiters are stored for one alias,
    exceeding the test cap of 1,024.

### ISSUE-036: Alias find timeout arithmetic overflows near maximum timestamp

- Category: long-running stability, correctness
- Reviewer: `Aristotle`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasServiceInternal::on_tick` checks
    hint timeouts with `requested_at + HINT_TIMEOUT_MS <= now`.
  - `src/service/alias_service.rs`: scan timeouts use
    `requested_at + SCAN_TIMEOUT_MS <= now`.
  - `src/service/alias_service.rs`: alias find state stores raw `now_ms()`
    values in `FindRequestState`.
- Impact: in a very long-running process or with internal time near
  `u64::MAX`, alias find timeout checks panic in debug builds. In release
  builds the deadline arithmetic can wrap and make timeout behavior incorrect.
- Evidence test:
  - `cargo test find_timeout_at_max_timestamp_must_not_overflow -- --nocapture`
  - Failure summary: ticking a pending alias find created at `u64::MAX - 10`
    panics at `src/service/alias_service.rs:244` with
    `attempt to add with overflow`.

### ISSUE-037: Replicated KV full-sync consumer emits reversed snapshot bounds

- Category: security, correctness
- Reviewer: `McClintock`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts untrusted snapshot pagination metadata.
  - `src/service/replicate_kv_service/remote_storage.rs`: when
    `snapshot.next_key` is present, it sends a follow-up `FetchSnapshot` using
    `from = next_key` and `to = self.biggest_key` without validating
    `next_key <= biggest_key`.
  - `src/service/replicate_kv_service/local_storage.rs`: recipients of that
    invalid request hit the reversed-bounds snapshot producer path.
- Impact: a malicious peer can reply to full sync with `next_key > biggest_key`
  and make the receiver emit protocol-invalid reversed snapshot bounds. If sent
  to an unfixed peer, that request can trigger the producer-side panic from
  ISSUE-025; even with that panic fixed, the consumer is still trusting invalid
  pagination metadata.
- Evidence test:
  - `cargo test full_sync_must_reject_snapshot_next_key_past_biggest_key -- --nocapture`
  - Failure summary: a response with `next_key = 2` and `biggest_key = 1`
    makes the receiver emit `FetchSnapshot { from: Some(2), to: Some(1), ... }`.

### ISSUE-038: Replicated KV full-sync consumer accepts empty continuation pages

- Category: bad-network stability, correctness
- Reviewer: `Beauvoir`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts untrusted
    `RpcRes::FetchSnapshot(Some(snapshot), version)`.
  - `src/service/replicate_kv_service/remote_storage.rs`: the code has a
    `TODO check snapshot is not empty` but still accepts `slots: []` with
    `next_key: Some(...)`.
  - `src/service/replicate_kv_service/remote_storage.rs`: if `next_key` is
    present, it emits another `FetchSnapshot` even though no slot was applied.
- Impact: a malicious or buggy peer can keep a receiver in full sync forever
  with empty continuation pages. The receiver repeatedly requests more snapshot
  data without applying data or reaching `WorkingState`.
- Evidence test:
  - `cargo test full_sync_must_reject_empty_snapshot_page_with_next_key -- --nocapture`
  - Failure summary: an empty snapshot page with `next_key: Some(1)` causes the
    receiver to emit another `FetchSnapshot` instead of rejecting the
    non-progressing page.

### ISSUE-039: Pubsub accepts member messages from peers without channel membership

- Category: security, correctness
- Reviewer: `Faraday`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PubsubMessage::Publish` delivers
    to local subscribers if the channel exists, without checking that
    `from_peer` is in `remote_publishers`.
  - `src/service/pubsub_service.rs`: inbound `PubsubMessage::Feedback`
    similarly delivers to local publishers without checking `remote_subscribers`.
  - `src/service/pubsub_service.rs`: `remote_publishers` and
    `remote_subscribers` are maintained but not used to authorize normal member
    traffic.
- Impact: any connected authenticated peer can inject normal `Publish` data
  into subscribers for an existing channel without joining as a publisher. The
  feedback direction has the same membership bypass for peers that never joined
  as subscribers.
- Evidence test:
  - `cargo test pubsub_publish_must_require_remote_publisher_membership -- --nocapture`
  - Failure summary: node2 injects `PubsubMessage::Publish` into node1's
    subscriber without ever joining the channel as a publisher.

### ISSUE-040: Metrics and visualization services panic on zero collection interval

- Category: correctness, API stability
- Reviewer: `Volta`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: `MetricsService::new` passes
    `collect_interval.unwrap_or(...)` directly to `tokio::time::interval`.
  - `src/service/visualization_service.rs`: `VisualizationService::new` does
    the same with its public `collect_interval`.
- Impact: public service constructors accept `Some(Duration::ZERO)` but do not
  validate it. Tokio panics synchronously for a zero interval, so a caller
  configuration error unwinds construction instead of returning an error,
  normalizing to a minimum, or using the default.
- Evidence test:
  - `cargo test metrics_service_zero_collect_interval_must_not_panic -- --nocapture`
  - Failure summary: `MetricsService::new(Some(Duration::ZERO), ...)` panics at
    `src/service/metrics_service.rs:32` with `` `period` must be non-zero. ``.

### ISSUE-041: Alias lookup tracks unbounded distinct pending misses

- Category: high-load stability, bad-network stability
- Reviewer: `Hubble`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasServiceInternal::find_reqs` stores
    pending lookups in a `HashMap<AliasId, FindRequest>` with no admission
    limit or backpressure.
  - `src/service/alias_service.rs`: every unique missing
    `AliasControl::Find` queues a broadcast `AliasMessage::Scan` and inserts a
    separate pending request until timeout.
- Impact: under high local load or a bad network where aliases remain missing,
  callers can create unbounded pending lookup state across distinct aliases.
  The same burst also queues one broadcast scan per unique miss, amplifying
  memory use and network work until the timeout loop catches up. This is
  distinct from ISSUE-035, which covers many duplicate waiters for one alias.
- Evidence test:
  - `cargo test distinct_pending_find_requests_must_be_bounded -- --nocapture`
  - Failure summary: 1025 unique missing alias lookups leave 1025 pending
    `find_reqs`, failing the bounded-pending-request assertion.

### ISSUE-042: Visualization peer timeout arithmetic overflows

- Category: correctness, long-running stability
- Reviewer: `Franklin`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: peer expiry compares
    `now >= last_updated + interval.as_millis() as u64 * 2`.
  - `src/service/visualization_service.rs`: the timeout deadline uses unchecked
    `u64` addition and unchecked conversion from `Duration::as_millis()`.
- Impact: in a very long-running process, or with a very large configured
  collection interval, visualization peer expiry can panic in debug builds. In
  release builds the deadline can wrap, causing peers to be reported as left
  before the mathematical timeout.
- Evidence test:
  - `cargo test visualization_peer_timeout_deadline_must_not_overflow -- --nocapture`
  - Failure summary: with `last_updated = u64::MAX - 10` and a 6 ms interval,
    the timeout helper panics at `src/service/visualization_service.rs:36` with
    `attempt to add with overflow`.

### ISSUE-043: Pubsub retains unbounded unanswered RPC requests

- Category: high-load stability
- Reviewer: `Darwin`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `publish_rpc_reqs` and
    `feedback_rpc_reqs` are unbounded `HashMap`s keyed by random `RpcId`.
  - `src/service/pubsub_service.rs`: guest and member publish/feedback RPC
    paths insert pending requests whenever there is at least one destination,
    using caller-supplied timeouts and no admission cap or backpressure.
  - `src/service/pubsub_service.rs`: cleanup only happens after an answer or
    the periodic timeout sweep.
- Impact: a local high-load caller can retain many unanswered pubsub RPC
  waiters for long timeout windows, growing memory linearly and producing local
  fanout work for each pending RPC. This is distinct from ISSUE-039's remote
  membership bypass and ISSUE-041's alias lookup backlog.
- Evidence test:
  - `cargo test pending_publish_rpc_requests_must_be_bounded -- --nocapture`
  - Failure summary: 1025 unanswered guest publish RPCs with a local subscriber
    destination leave 1025 entries in `publish_rpc_reqs`, failing the bounded
    pending-RPC assertion.

### ISSUE-044: Router best-path score arithmetic overflows

- Category: correctness, bad-network stability
- Reviewer: `Mendel`, confirmed.
- Affected code:
  - `src/router.rs`: `PathMetric::score` computes
    `rtt_ms + relay_hops as u16 * 10` with unchecked `u16` arithmetic.
  - `src/router.rs`: `PeerMemory::select_best` calls `score()` while choosing
    the active path.
- Impact: an advertised route can compose successfully, then panic later during
  best-path selection if its RTT is near `u16::MAX` and hop count is nonzero.
  In release builds the score can wrap and make an awful relayed path look
  cheaper than a direct path. This is distinct from ISSUE-033, which overflows
  during route metric composition in `AddAssign`.
- Evidence test:
  - `cargo test should_not_overflow_score_during_best_path_selection -- --nocapture`
  - Failure summary: a composed metric `(relay_hops: 2, rtt_ms: 65525)` panics
    at `src/router.rs:190` with `attempt to add with overflow` while selecting
    the best path.

### ISSUE-045: Replicated KV creates unbounded remote stores

- Category: high-load stability, bad-network stability
- Reviewer: `Gauss`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvStore::on_remote_event`
    creates a `RemoteStore` for every previously unseen `from` node before
    applying any admission limit or flow control.
  - `src/service/replicate_kv_service.rs`: `remotes` is an unbounded
    `HashMap<N, RemoteStore<N, K, V>>`.
  - `src/service/replicate_kv_service/remote_storage.rs`: each new
    `RemoteStore` enters full sync and immediately queues a `FetchSnapshot`
    request.
- Impact: under high load or bad-network sender churn, many distinct remote
  identities can grow replicated-KV remote state linearly during the idle
  timeout window. Each new remote also queues outgoing full-sync work, so the
  memory growth is coupled with extra network fanout.
- Evidence test:
  - `cargo test remote_store_creation_must_be_bounded -- --nocapture`
  - Failure summary: 1025 distinct remote `Version(0)` broadcasts leave 1025
    `RemoteStore` entries, failing the bounded-remote assertion.

### ISSUE-046: Replicated KV accepts unbounded FetchChanged response batches

- Category: high-load stability, resource exhaustion
- Reviewer: `Feynman`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` accepts
    `RpcRes::FetchChanged(Ok(changeds))` from a remote peer.
  - `src/service/replicate_kv_service/remote_storage.rs`: every response item
    with `changed.version > self.version` is inserted into the unbounded
    `pendings` map before `apply_pendings`.
- Impact: a malicious or buggy peer can answer a fetch request, or send an
  unsolicited response, with a large batch of far-future changes and force
  immediate memory growth in one frame. This is distinct from ISSUE-027, which
  covers the broadcast ingress path into the same pending map.
- Evidence test:
  - `cargo test working_state_must_cap_pending_fetch_changed_response -- --nocapture`
  - Failure summary: a single `FetchChanged(Ok(...))` response containing
    versions `2..=2050` leaves 2049 pending changes, exceeding the test cap of
    1024.

### ISSUE-047: Replicated KV full sync accepts mismatched continuation versions

- Category: security, correctness
- Reviewer: `Tesla`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` locks `self.version` from the first snapshot
    page.
  - `src/service/replicate_kv_service/remote_storage.rs`: continuation
    `FetchSnapshot` responses ignore their declared `version` instead of
    requiring it to match the locked snapshot version.
  - `src/service/replicate_kv_service/remote_storage.rs`: continuation slots
    are emitted and stored without checking they are no newer than the locked
    snapshot version.
- Impact: a malicious peer can send a first snapshot page at one version, then
  send a continuation page from a later version. The receiver can mix newer data
  into a snapshot while transitioning to `WorkingState` at the older locked
  version, corrupting replication consistency. This is distinct from ISSUE-034,
  which covers a single response whose slots exceed that same response's
  declared version.
- Evidence test:
  - `cargo test full_sync_must_reject_continuation_snapshot_version_mismatch -- --nocapture`
  - Failure summary: after locking the snapshot at `Version(1)`, a continuation
    response declaring `Version(2)` stores key `2` instead of rejecting the
    mismatched page.

### ISSUE-048: Pubsub RPC member messages bypass channel membership

- Category: security, correctness
- Reviewer: `Kierkegaard`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PubsubMessage::PublishRpc`
    delivers method, payload, `RpcId`, and remote source to local subscribers
    without checking that `from_peer` is in `remote_publishers`.
  - `src/service/pubsub_service.rs`: inbound `PubsubMessage::FeedbackRpc` has
    the symmetric missing check against `remote_subscribers`.
- Impact: any connected authenticated peer can invoke subscriber or publisher
  RPC handlers for an existing channel without joining that channel. This is
  related to ISSUE-039's ordinary `Publish`/`Feedback` data injection, but
  distinct because it reaches RPC method handlers and can trigger side effects
  and responses.
- Evidence test:
  - `cargo test pubsub_publish_rpc_must_require_remote_publisher_membership -- --nocapture`
  - Failure summary: node2 injects `PubsubMessage::PublishRpc` into node1's
    subscriber without ever joining the channel as a publisher.

### ISSUE-049: Broadcast fanout can block on one congested peer control queue

- Category: high-load stability, bad-network stability
- Reviewer: `Cicero`, confirmed.
- Affected code:
  - `src/ctx.rs`: `SharedCtx::send_broadcast` awaits
    `conn_alias.send(...)` sequentially for every connection.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits a bounded peer
    control channel.
  - `src/service/alias_service.rs`, `src/service/metrics_service.rs`,
    `src/service/visualization_service.rs`, and `src/service/pubsub_service.rs`
    use the awaited broadcast path.
- Impact: one stalled or congested peer control queue can block service
  broadcast fanout and stall the caller's service loop before later peers are
  attempted. This is distinct from prior service-queue/open-stream issues
  because it affects ordinary broadcast fanout.
- Evidence test:
  - `cargo test send_broadcast_must_not_block_on_full_peer_control_queue -- --nocapture`
  - Failure summary: a full synthetic peer control channel makes
    `SharedCtx::send_broadcast` exceed the 50 ms timeout instead of completing
    without blocking on that peer.

### ISSUE-050: Unicast send can block on a congested peer control queue

- Category: high-load stability, bad-network stability
- Reviewer: `Ramanujan`, confirmed.
- Affected code:
  - `src/ctx.rs`: `SharedCtx::send_unicast` awaits
    `conn_alias.send(...)` for the selected next hop.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits a bounded peer
    control channel.
  - `src/service/alias_service.rs`, `src/service/visualization_service.rs`,
    and `src/service/pubsub_service.rs` use awaited unicast paths for replies
    or directed member traffic.
- Impact: a stalled or congested selected peer queue can block a service loop
  on a single unicast. This is distinct from ISSUE-049, which covers broadcast
  fanout and starvation of later peers.
- Evidence test:
  - `cargo test send_unicast_must_not_block_on_full_peer_control_queue -- --nocapture`
  - Failure summary: a full synthetic peer control channel makes
    `SharedCtx::send_unicast` exceed the 50 ms timeout instead of failing fast
    or using the nonblocking unicast path.

### ISSUE-051: Legitimate PeerStopped leaves stopped neighbour connected

- Category: correctness, graceful-shutdown stability
- Reviewer: `Anscombe`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerStopped(conn, peer)` by removing discovery and router
    state only.
  - `src/lib.rs`: the same branch does not remove the `conn` from
    `NetworkNeighbours` or unregister the shared connection context.
  - `src/lib.rs`: tick processing still iterates `connected_conns()`, and
    connect logic still uses `has_peer()` to suppress reconnects.
- Impact: after a legitimate graceful stop notification, a non-seed peer can
  remain marked as a connected neighbour until a later QUIC disconnect event.
  During that window reconnects can be suppressed and tick sync work can still
  target a stopped peer.
- Evidence test:
  - `cargo test peer_stopped_must_remove_stopped_neighbour_immediately -- --nocapture`
  - Failure summary: after processing `MainEvent::PeerStopped(conn, peer)`,
    `node.neighbours.has_peer(&peer)` remains true instead of immediately
    removing the stopped non-seed neighbour.

### ISSUE-052: Out-of-range service ids panic service registration

- Category: correctness, API stability
- Reviewer: `James`, confirmed.
- Affected code:
  - `src/msg.rs`: `P2pServiceId` is a `u16`, allowing values outside the
    256-slot service table.
  - `src/ctx.rs`: `SharedCtxInternal::set_service` indexes
    `services[*service_id as usize]` without range validation.
  - `src/lib.rs`: `P2pNetwork::create_service` exposes this unchecked path as
    a public API.
- Impact: callers can request `P2pServiceId(256)` or larger and panic service
  registration with an out-of-bounds index. This is distinct from ISSUE-030,
  which covers duplicate ids inside the valid service table range.
- Evidence test:
  - `cargo test out_of_range_service_id_must_not_panic -- --nocapture`
  - Failure summary: `node.create_service(P2pServiceId::from(256u16))` panics
    at `src/ctx.rs:28` with `index out of bounds: the len is 256 but the index
    is 256`.

### ISSUE-053: Inbound out-of-range service ids kill peer connection tasks

- Category: security, bad-network stability
- Reviewer: `Hooke`, confirmed.
- Affected code:
  - `src/msg.rs`: `P2pServiceId` is deserialized from the wire as a `u16`,
    including values outside the 256-slot service table.
  - `src/peer/peer_internal.rs`: inbound `PeerMessage::Unicast` and
    `PeerMessage::Broadcast` call `ctx.get_service(&service_id)` when the
    message targets a local service.
  - `src/ctx.rs`: `SharedCtxInternal::get_service` indexes
    `services[service_id as usize]` without range validation.
- Impact: an authenticated remote peer can send a unicast or broadcast with
  `P2pServiceId(256)` or larger and panic the receiver's peer connection task.
  This is distinct from ISSUE-052 because it is remote-triggerable through the
  inbound lookup path, not local service registration.
- Evidence test:
  - `cargo test inbound_out_of_range_unicast_service_id_must_not_kill_connection -- --nocapture`
  - Failure summary: an inbound unicast with `P2pServiceId(256)` panics at
    `src/ctx.rs:33` with `index out of bounds: the len is 256 but the index is
    256`, then a valid follow-up unicast on the same connection fails because
    the peer connection channel is closed.

### ISSUE-054: Zero network tick interval panics node construction

- Category: correctness, configuration stability
- Reviewer: `Hooke`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::new` passes `cfg.tick_ms` directly to
    `tokio::time::interval(Duration::from_millis(cfg.tick_ms))`.
  - `src/lib.rs`: `P2pNetworkConfig::tick_ms` is public and has no validation
    that it is non-zero.
- Impact: a caller can construct `P2pNetworkConfig` with `tick_ms = 0` and
  panic node creation instead of receiving a recoverable configuration error or
  a normalized minimum tick. This is distinct from ISSUE-040, which covers
  metrics and visualization service collection intervals.
- Evidence test:
  - `cargo test zero_network_tick_interval_must_not_panic -- --nocapture`
  - Failure summary: `P2pNetwork::new` panics at `src/lib.rs:184` with
    `` `period` must be non-zero. `` when `tick_ms` is zero.

### ISSUE-055: Discovery advertisements can duplicate configured seed ids

- Category: correctness, seed stability
- Reviewer: `Hooke`, confirmed.
- Affected code:
  - `src/discovery.rs`: `PeerDiscovery::apply_sync` accepts remote
    advertisements for peers that are already configured as seeds.
  - `src/discovery.rs`: `PeerDiscovery::remotes` returns learned remotes first
    and then configured seeds, so a remote-advertised seed id can appear before
    the trusted configured seed address.
- Impact: a peer can advertise the id of a configured seed at a different
  address. The node then keeps both candidates and may attempt the untrusted
  address before the configured seed, adding connection churn and weakening the
  invariant that seed nodes are preserved from static configuration.
- Evidence test:
  - `cargo test apply_sync_must_not_duplicate_or_override_configured_seed -- --nocapture`
  - Failure summary: after applying an advertisement for seed peer `1` at
    `127.0.0.1:9001`, `remotes()` returns both `1@127.0.0.1:9001` and the
    configured `1@127.0.0.1:9000`; expected only the configured seed address.

### ISSUE-056: Stream open can block on congested peer control queue

- Category: stability, high-load backpressure
- Reviewer: `Confucius`, confirmed.
- Affected code:
  - `src/ctx.rs`: `SharedCtx::open_stream` awaits the target peer alias
    directly after route lookup.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::open_stream` awaits
    `control_tx.send(PeerConnectionControl::OpenStream(...))` on a bounded
    peer control queue before any stream setup timeout can apply.
  - `src/peer/peer_internal.rs`: `OPEN_BI_TIMEOUT` only wraps
    `connection.open_bi()` after the peer task has already received the
    `OpenStream` command.
- Impact: under high load, a full peer control queue can make stream-opening
  callers wait indefinitely before the operation reaches the existing
  `OPEN_BI_TIMEOUT`. This is distinct from ISSUE-049 and ISSUE-050 because it
  affects the stream-opening API and bypasses its stream setup timeout before a
  stream attempt starts.
- Evidence test:
  - `cargo test open_stream_must_not_block_on_full_peer_control_queue -- --nocapture`
  - Failure summary: the test fills a synthetic bounded peer control queue,
    then `SharedCtx::open_stream` fails the 50 ms timeout assertion because it
    is blocked while awaiting admission of `PeerConnectionControl::OpenStream`.

### ISSUE-057: Stale peer-connected events install unusable routes

- Category: correctness, async race stability
- Reviewer: `Russell`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerConnected` by calling `router.set_direct(conn, peer,
    ttl_ms)` before checking whether the connection id exists in
    `neighbours`.
  - `src/neighbours.rs`: `NetworkNeighbours::mark_connected` returns `None`
    for an unknown connection id, but the caller ignores that result.
- Impact: a stale or otherwise unknown internal `PeerConnected` event can
  install a direct router path to a connection id that has no live neighbour or
  peer alias. Later traffic can observe a route but fail to find the underlying
  connection, producing noisy path state and failed sends.
- Evidence test:
  - `cargo test stale_peer_connected_event_must_not_install_unusable_route -- --nocapture`
  - Failure summary: after processing
    `MainEvent::PeerConnected(ConnectionId(404), PeerId(2), 10)`, the router
    returns `Some(Next(ConnectionId(404)))`; expected no route because that
    connection id was never registered as a neighbour.

### ISSUE-058: Pubsub requester can create dead-on-arrival handles

- Category: correctness, API stability
- Reviewer: `Kant`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `PubsubServiceRequester::publisher`
    always returns `Publisher::build(...)` and has no error path if the
    `PubsubService` task/control receiver has gone away.
  - `src/service/pubsub_service/publisher.rs`: `Publisher::build` ignores the
    result of `control_tx.send(InternalMsg::PublisherCreated(...))`; when the
    send fails, the registration sender is dropped and the returned
    publisher's event receiver is already closed.
  - `src/service/pubsub_service.rs` and
    `src/service/pubsub_service/subscriber.rs`: `subscriber` uses the same
    fire-and-forget registration shape.
- Impact: a cloned `PubsubServiceRequester` can outlive `PubsubService` and
  still manufacture publisher/subscriber handles that were never registered
  with the service. Callers receive a normal-looking handle, but its event
  channel is closed immediately and later operations fail through unrelated
  internal-channel errors instead of creation returning a clear error.
- Evidence test:
  - `cargo test pubsub_publisher_after_service_drop_must_not_be_dead_on_arrival -- --nocapture`
  - Failure summary: after dropping `PubsubService`, `requester.publisher(...)`
    returns a `Publisher`, but `publisher.recv()` returns immediately instead
    of waiting for events; expected handle creation to fail or avoid returning
    an already-closed publisher.

### ISSUE-059: Replicated KV full sync accepts `None` as a fake continuation

- Category: correctness, data consistency
- Reviewer: `Boole`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts `RpcRes::FetchSnapshot(None, version)`
    as a completed empty snapshot regardless of whether a prior snapshot page
    advertised `next_key`.
  - `src/service/replicate_kv_service/remote_storage.rs`: after a partial
    `Some(snapshot)` page, the receiver stores slots and requests the next
    range, but a later `None` response transitions directly to `WorkingState`.
- Impact: a malicious or buggy peer can truncate a paginated full sync. The
  receiver applies the first page, then accepts `None` as completion for the
  continuation and starts working with incomplete remote data. This is distinct
  from ISSUE-038's empty `Some(snapshot)` continuation loop and ISSUE-047's
  mismatched continuation version because this path silently completes sync
  with missing data.
- Evidence test:
  - `cargo test full_sync_must_reject_none_continuation_after_partial_snapshot -- --nocapture`
  - Failure summary: after a first page with `next_key: Some(2)`, a
    continuation response of `RpcRes::FetchSnapshot(None, Version(1))` sets
    `ctx.next_state` to `Some(Working(...))`; expected the partial sync to stay
    incomplete and reject the fake continuation.

### ISSUE-060: Dropped services leave their service id permanently reserved

- Category: correctness, API lifecycle stability
- Reviewer: `Ampere`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::create_service` registers a service sender in
    shared context but returns only `P2pService`.
  - `src/ctx.rs`: `SharedCtxInternal::set_service` stores
    `Some(Sender<P2pServiceEvent>)` in the 256-slot service table and never
    clears it.
  - `src/service.rs`: `P2pService` owns the receiver but has no `Drop` path to
    unregister the service id.
- Impact: after a service receiver is dropped, the service id remains reserved
  for the lifetime of the node. A live node cannot restart or replace that
  service id, and inbound messages for the dropped service still resolve to a
  stale sender before being discarded. This is distinct from ISSUE-030, which
  covers duplicate creation while the first service is still live.
- Evidence test:
  - `cargo test dropped_service_id_must_be_reusable -- --nocapture`
  - Failure summary: after dropping the first `P2pService`, creating a
    replacement with the same id panics at `src/ctx.rs:28` with
    `Service ID already used`.

### ISSUE-061: Visualization accepts unsolicited forged topology info

- Category: security, topology correctness
- Reviewer: `Halley`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: `VisualizationService::recv`
    accepts `Message::Info(neighbours)` from any unicast or broadcast sender
    and immediately emits `PeerJoined` or `PeerUpdated`.
  - `src/service/visualization_service.rs`: the service does not track pending
    scan requests, expected responders, nonces, or validate supplied neighbour
    data against known router state.
- Impact: any connected peer can poison visualization topology by sending an
  unsolicited `Info` frame with arbitrary connection ids, peer ids, and RTTs.
  Downstream consumers can observe fake peer joins/updates without this node
  requesting or validating that data. This is distinct from the visualization
  timeout arithmetic issue and from pubsub channel-membership issues.
- Evidence test:
  - `cargo test visualization_info_must_not_be_accepted_without_scan_request -- --nocapture`
  - Failure summary: node2 injects `Message::Info([(ConnectionId(999),
    PeerId(123), 7)])` to node1, and node1 emits a matching `PeerJoined`
    event even though it never requested a scan response.

### ISSUE-062: Metrics accepts unsolicited forged connection metrics

- Category: security, monitoring correctness
- Reviewer: `Locke`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: `MetricsService::recv` accepts any
    unicast or broadcast frame that deserializes as `Message`.
  - `src/service/metrics_service.rs`: `Message::Info(peer_metrics)` is emitted
    directly as `MetricsServiceEvent::OnPeerConnectionMetric(from,
    peer_metrics)`.
  - `src/service/metrics_service.rs`: the service does not track pending scan
    requests, expected responders, nonces, collector-only state, or validate
    that metric entries correspond to real peer connections.
- Impact: any connected peer can inject arbitrary connection metrics into a
  collector, including fake connection ids, peers, RTT, loss, and byte counts.
  Monitoring, alerting, or health decisions built on this service can observe
  forged data. This shares an unsolicited-response pattern with ISSUE-061, but
  affects the metrics service and peer-connection metric events rather than
  visualization topology.
- Evidence test:
  - `cargo test metrics_info_must_not_be_accepted_without_scan_request -- --nocapture`
  - Failure summary: node2 injects a forged `Message::Info` containing
    `ConnectionId(999)` and fake metric counters; node1 emits the matching
    `OnPeerConnectionMetric` event even though it never requested a scan
    response.

### ISSUE-063: Stale peer data events panic without a direct route

- Category: correctness, async race stability
- Reviewer: `Chandrasekhar`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerData(conn, ..., PeerMainData::Sync { ... })` by calling
    `router.apply_sync(conn, route)` without checking whether `conn` is still
    known and directly connected.
  - `src/router.rs`: `RouterTable::apply_sync` calls
    `self.directs.get(&conn).expect("should have direct metric with apply_sync")`.
- Impact: a stale peer task can deliver sync data after its direct connection
  state has already been removed, crashing main network event processing
  instead of ignoring the stale event or returning an error. This is distinct
  from ISSUE-057, which installs an unusable route on stale `PeerConnected`;
  this issue is a stale `PeerData::Sync` panic.
- Evidence test:
  - `cargo test stale_peer_data_event_must_not_panic_without_direct_route -- --nocapture`
  - Failure summary: processing `MainEvent::PeerData(ConnectionId(404), ...)`
    for an unknown connection panics at `src/router.rs:76` with
    `should have direct metric with apply_sync`.

### ISSUE-064: Stale peer stats events publish metrics for unknown connections

- Category: correctness, monitoring stability
- Reviewer: `Avicenna`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerStats(conn, to_peer, metrics)` by directly calling
    `ctx.update_metrics(&conn, to_peer, metrics)`.
  - `src/ctx.rs`: `SharedCtxInternal::update_conn_metrics` inserts into
    `conn_metrics` without checking whether `conn` exists in the live
    connection map.
- Impact: stale peer-task stats can create exported metrics for a connection
  that was never registered or is no longer live. This pollutes monitoring
  output even though the underlying connection alias/neighbour is absent. This
  is distinct from ISSUE-057's stale route installation and ISSUE-063's stale
  sync-data panic.
- Evidence test:
  - `cargo test stale_peer_stats_event_must_not_publish_metrics_for_unknown_connection -- --nocapture`
  - Failure summary: processing `MainEvent::PeerStats(ConnectionId(404), ...)`
    on a fresh node makes `node.ctx.metrics()` non-empty; expected stale stats
    for an unknown connection to be ignored or rejected.

### ISSUE-065: Stale disconnect events are emitted to users

- Category: correctness, API event stability
- Reviewer: `Harvey`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerDisconnected(conn, peer)` by attempting router/neighbour
    cleanup and then unconditionally returning
    `P2pNetworkEvent::PeerDisconnected(conn, peer)`.
  - `src/neighbours.rs`: `NetworkNeighbours::remove` returns `None` for an
    unknown connection id, but the caller ignores that result.
- Impact: stale peer-task disconnect messages can surface as public disconnect
  events even when the connection was never registered or already removed.
  Applications can receive false peer-disconnected notifications and corrupt
  their own connection state. This is distinct from the stale-event route,
  sync, and metrics issues because it affects the public event stream.
- Evidence test:
  - `cargo test stale_peer_disconnected_event_must_not_emit_user_disconnect -- --nocapture`
  - Failure summary: processing
    `MainEvent::PeerDisconnected(ConnectionId(404), PeerId(2))` on a fresh
    node returns `P2pNetworkEvent::PeerDisconnected(...)`; expected the stale
    event to be ignored as `Continue`.

### ISSUE-066: Disconnect events do not validate peer id against connection owner

- Category: correctness, routing stability
- Reviewer: `Raman`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerDisconnected(conn, peer)` by deleting the direct route for
    `conn` and emitting `P2pNetworkEvent::PeerDisconnected(conn, peer)` without
    checking that `peer` matches the peer currently attached to `conn`.
- Impact: a mismatched disconnect event for a live connection can remove the
  route for the real connected peer while notifying applications that a
  different peer disconnected. This can corrupt routing state and public
  connection state under stale, reordered, or malformed internal events. This
  is distinct from ISSUE-065 because it covers a known connection id whose
  reported peer id is wrong, rather than an entirely unknown connection id.
- Evidence test:
  - `cargo test peer_disconnected_must_validate_peer_matches_connection -- --nocapture`
  - Failure summary: a direct route for `ConnectionId(77) -> PeerId(2)`
    receives `MainEvent::PeerDisconnected(ConnectionId(77), PeerId(99))`;
    current code emits `PeerDisconnected(ConnectionId(77), PeerId(99))`
    instead of ignoring the peer mismatch.

### ISSUE-067: PeerConnected can rebind an existing connection to a different peer

- Category: correctness, routing stability
- Reviewer: `Ptolemy`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerConnected(conn, peer, ttl_ms)` by unconditionally calling
    `router.set_direct(conn, peer, ttl_ms)` and emitting `PeerConnected`.
  - `src/router.rs`: `RouterTable::set_direct` unconditionally inserts into
    `directs` by `ConnectionId`, allowing an existing connection owner to be
    replaced.
- Impact: a duplicate, stale, or malformed connected event for an already-bound
  connection can reassign that connection from the real peer to a different
  peer. This corrupts direct routing state and emits a misleading public
  connection event. This is distinct from ISSUE-057, which covers an unknown
  connection id installing an unusable route.
- Evidence test:
  - `cargo test peer_connected_must_not_rebind_existing_connection_to_different_peer -- --nocapture`
  - Failure summary: after prebinding `ConnectionId(88) -> PeerId(2)`,
    processing `MainEvent::PeerConnected(ConnectionId(88), PeerId(99), 10)`
    emits `PeerConnected(ConnectionId(88), PeerId(99))`; expected the peer
    mismatch to be ignored as `Continue`.

### ISSUE-068: PeerStats can relabel a known connection to the wrong peer

- Category: correctness, observability integrity
- Reviewer: `Curie`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerStats(conn, to_peer, metrics)` by directly calling
    `ctx.update_metrics(&conn, to_peer, metrics)` without checking that
    `to_peer` matches the peer currently attached to `conn`.
  - `src/ctx.rs`: `SharedCtxInternal::update_conn_metrics` stores the supplied
    `(conn, peer, metrics)` tuple without validating the connection owner.
- Impact: a stale, reordered, or malformed stats event for a live connection can
  publish metrics under the wrong peer id. Applications and monitoring systems
  can observe forged or corrupted peer-connection metrics even when the
  connection id is otherwise valid. This is distinct from ISSUE-064, which
  covers accepting stats for an unknown connection id; this issue covers a
  known connection id with a mismatched peer id.
- Evidence test:
  - `cargo test peer_stats_must_validate_peer_matches_connection -- --nocapture`
  - Failure summary: after prebinding `ConnectionId(66) -> PeerId(2)`,
    processing `MainEvent::PeerStats(ConnectionId(66), PeerId(99), metrics)`
    inserts exported metrics for `PeerId(99)`; expected the mismatched stats
    event to be ignored.

### ISSUE-069: Dropped publisher requesters can still publish

- Category: correctness, pubsub lifecycle stability
- Reviewer: `Epicurus`, confirmed.
- Affected code:
  - `src/service/pubsub_service/publisher.rs`: `PublisherRequester` is cloneable
    and keeps `local_id`, `channel_id`, and the service control sender after the
    owning `Publisher` is dropped.
  - `src/service/pubsub_service/publisher.rs`: `Publisher::drop` sends
    `InternalMsg::PublisherDestroyed(local_id, channel_id)`.
  - `src/service/pubsub_service.rs`:
    `InternalMsg::Publish(_local_id, channel, vec)` ignores `_local_id` and
    delivers to local/remote subscribers whenever the channel exists.
- Impact: after the last local publisher is dropped and subscribers observe
  `PeerLeaved(PeerSrc::Local)`, any cloned requester from that dropped publisher
  can still publish on the channel. This violates publisher lifetime semantics
  and can deliver messages from a publisher that the pubsub state already
  considers gone. This is distinct from ISSUE-058, which covers creating
  dead-on-arrival handles after the whole `PubsubService` is gone.
- Evidence test:
  - `cargo test dropped_publisher_requester_must_not_continue_publishing -- --nocapture`
  - Failure summary: after dropping the publisher and receiving
    `SubscriberEvent::PeerLeaved(PeerSrc::Local)`, a cloned stale requester
    publishes `stale-publish`, and the subscriber still receives
    `SubscriberEvent::Publish`.

### ISSUE-070: Dropped subscriber requesters can still send feedback

- Category: correctness, pubsub lifecycle stability
- Reviewer: `Einstein`, confirmed.
- Affected code:
  - `src/service/pubsub_service/subscriber.rs`: `SubscriberRequester` is
    cloneable and retains `local_id`, `channel_id`, and the service control
    sender after the owning `Subscriber` is dropped.
  - `src/service/pubsub_service/subscriber.rs`: `Subscriber::drop` sends
    `InternalMsg::SubscriberDestroyed(local_id, channel_id)`.
  - `src/service/pubsub_service.rs`:
    `InternalMsg::Feedback(_local_id, channel, vec)` ignores `_local_id` and
    delivers to local/remote publishers whenever the channel exists.
- Impact: after the last local subscriber is dropped and publishers observe
  `PeerLeaved(PeerSrc::Local)`, any cloned requester from that dropped
  subscriber can still send feedback on the channel. This violates subscriber
  lifetime semantics and can deliver messages from a subscriber the pubsub
  state already considers gone. This is distinct from ISSUE-069 because it
  covers stale feedback from a dropped subscriber rather than stale publishes
  from a dropped publisher.
- Evidence test:
  - `cargo test dropped_subscriber_requester_must_not_continue_feedback -- --nocapture`
  - Failure summary: after dropping the subscriber and receiving
    `PublisherEvent::PeerLeaved(PeerSrc::Local)`, a cloned stale requester sends
    `stale-feedback`, and the publisher still receives
    `PublisherEvent::Feedback`.
