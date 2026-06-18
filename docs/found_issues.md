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
