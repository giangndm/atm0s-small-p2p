# Found Issues

RED-team issue ledger for `atm0s-small-p2p`.

Acceptance rule: an issue is listed here only after reviewer confirmation and
test-case evidence. The tests listed below are expected to fail on the current
audited code.

Issue score: 0 means low priority and not needed now; 100 means critical and
must resolve.

## Audit Status

- Current consecutive no-new-issue cycles: 323
- Stop condition requested by user: continue until 5 consecutive cycles find no
  new accepted issue.

## Root Cause Summary

This is the short version of the issue ledger. The detailed entries below remain
the source of truth for evidence and reviewer decisions.

### RC-1: Authenticated connection identity is not the authority for peer claims

- Representative issues: ISSUE-001, ISSUE-004, ISSUE-014, ISSUE-015,
  ISSUE-018, ISSUE-020, ISSUE-039, ISSUE-048, ISSUE-066, ISSUE-067,
  ISSUE-068, ISSUE-090, ISSUE-115, ISSUE-116, ISSUE-145, ISSUE-189,
  ISSUE-194.
- Pattern: message payloads and internal events carry peer ids, RPC ids, or
  source identities that are trusted without binding them back to the live
  authenticated connection, local handle, expected responder, channel role, or
  the invariant that a shared-key holder may not authenticate as the local node
  or an arbitrary third-party peer.
- Minimal fix proposal: add one validation layer at each ingress boundary:
  derive `source` from the authenticated connection, validate
  `(ConnectionId, PeerId)` against neighbour state before processing main
  events, reject self-identity and unauthorized third-party peer admission
  before aliases are registered, and store expected responder/handle metadata
  in pending RPC/find records before accepting answers.

### RC-2: Protocol state machines lack request correlation and monotonicity checks

- Representative issues: ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-047,
  ISSUE-059, ISSUE-071, ISSUE-081 through ISSUE-089, ISSUE-095, ISSUE-099,
  ISSUE-110, ISSUE-111, ISSUE-138, ISSUE-141, ISSUE-143, ISSUE-152,
  ISSUE-154, ISSUE-155, ISSUE-158, ISSUE-166, ISSUE-171, ISSUE-175,
  ISSUE-186.
- Pattern: replicated-KV full sync, changed repair, alias lookup, metrics,
  visualization, and pubsub flows accept stale, unsolicited, reordered, or
  mismatched responses or broadcasts because handlers do not verify
  outstanding request shape, bounds, version, continuation key, expected phase,
  or whether an event actually advances activity.
- Minimal fix proposal: encode a small pending-request descriptor per flow and
  reject responses unless they match the descriptor exactly; clear or advance
  the descriptor only after all range/version invariants are checked; for
  membership gossip, carry a small generation or epoch and ignore older
  join/leave/heartbeat state. Refresh remote liveness only after an accepted
  event advances state or emits work.

### RC-3: Backpressure is inconsistent across async boundaries

- Representative issues: ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-118,
  ISSUE-119, ISSUE-120, ISSUE-123, ISSUE-124, ISSUE-125, ISSUE-126,
  ISSUE-127, ISSUE-133, ISSUE-136, ISSUE-147, ISSUE-153, ISSUE-157,
  ISSUE-163, ISSUE-164, ISSUE-178, ISSUE-182, ISSUE-184, ISSUE-198,
  ISSUE-199.
- Pattern: some paths use bounded channels and drop on `try_send`, some await
  bounded sends from critical tasks, and others use unbounded queues or produce
  duplicate internal control work. Under load this causes silent data loss,
  head-of-line blocking, unreported total fanout failure for failed awaited or
  nonblocking sends, or unbounded memory. RPC fanout can also count failed
  local or remote delivery attempts as live destinations. Transport config can
  also admit unused stream classes that no application task drains. Repair
  state machines can emit duplicate in-flight repair requests without waiting
  for timeout or response.
- Minimal fix proposal: define channel policy by event class: lifecycle and
  route updates must use bounded retry/coalescing; service payload delivery must
  return explicit backpressure errors, including zero-recipient fanout errors;
  public and internal request/control queues need fixed admission limits and
  per-target coalescing; peer tasks must not await bounded lifecycle reporting
  before they can process traffic or
  cleanup. RPC paths should insert pending state only after at least one
  successful local or remote fanout. Disable unused QUIC stream classes or add
  explicit admission plus drain/reject handlers. Repair requests should keep a
  typed pending descriptor and suppress duplicates until timeout or a matching
  response changes the required range.

### RC-4: Timeouts are partial, coarse, or overflow-prone

- Representative issues: ISSUE-002, ISSUE-009, ISSUE-021, ISSUE-036,
  ISSUE-042, ISSUE-093, ISSUE-117, ISSUE-121, ISSUE-134, ISSUE-149,
  ISSUE-156, ISSUE-159, ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-176.
- Pattern: timeout checks often wrap only one await point, rely on unchecked
  timestamp arithmetic, use coarse global sweeps instead of per-operation
  deadlines, or complete one side of setup before the full end-to-end setup is
  still alive. Handshake tokens also lack nonce/challenge binding or replay
  caches.
- Minimal fix proposal: use `checked_add`/`saturating_add` for deadlines and
  wrap every protocol phase with one end-to-end setup timeout; for RPCs, track
  the exact deadline per request and wake on the nearest deadline; for relays,
  tie downstream setup to upstream cancellation and roll back downstream streams
  if upstream acknowledgement fails. Bind handshake responses to fresh request
  nonces and reject recently accepted tokens until expiry.

### RC-5: Application-level resource limits are missing

- Representative issues: ISSUE-010, ISSUE-024, ISSUE-027, ISSUE-035,
  ISSUE-041, ISSUE-043, ISSUE-045, ISSUE-046, ISSUE-100 through ISSUE-108,
  ISSUE-122, ISSUE-131, ISSUE-174, ISSUE-196.
- Pattern: protocol framing may limit packet size, but decoded service-level
  collections, pending maps, cache sets, tombstones, remote stores, retained
  channel state, and outbound event queues often have no item-count or lifetime
  cap.
- Minimal fix proposal: introduce small per-structure caps with deterministic
  eviction/rejection: max rows per message, max peers per alias/channel, max
  pending RPCs/finds, max tombstones/remotes, max queued outbound events, and
  prune empty channel state on teardown. Mutation APIs that can enqueue work
  should return explicit backpressure errors or coalesce superseded work.

### RC-6: Lifecycle cleanup and stale handles are not consistently modeled

- Representative issues: ISSUE-028, ISSUE-029, ISSUE-051, ISSUE-057,
  ISSUE-060, ISSUE-064, ISSUE-065, ISSUE-069 through ISSUE-076, ISSUE-108,
  ISSUE-128 through ISSUE-132, ISSUE-135, ISSUE-139, ISSUE-142, ISSUE-144,
  ISSUE-148, ISSUE-150, ISSUE-151, ISSUE-161, ISSUE-162, ISSUE-165,
  ISSUE-167, ISSUE-168, ISSUE-170, ISSUE-179, ISSUE-183, ISSUE-185,
  ISSUE-187, ISSUE-188, ISSUE-193, ISSUE-195.
- Pattern: requesters, services, peer aliases, channel state, and cached hints
  can outlive the owner they represent, while shutdown paths may panic, leak,
  emit false public events, or keep stale routes/cache entries. Remote
  membership events can also arrive before a local channel owner exists and be
  discarded instead of retained as peer-owned state. Some shutdown controls
  announce owner teardown without clearing local authority, and peer lifecycle
  events do not consistently reach service-owned membership or public
  network-event consumers. Teardown instrumentation can also reset metrics
  through the wrong metric kind or reset monotonic counters to zero, corrupting
  observability state for exporters that require stable metric kinds and
  non-decreasing counters.
- Minimal fix proposal: add generation or liveness tokens to cloned requesters
  and local handles, make closed channels return `Err` instead of panicking, and
  centralize owner teardown so aliases, metrics, routes, caches, and service ids
  are cleared together. Shutdown controls should also enter an explicit
  terminal state so later register/find operations are rejected or no-op.
  Peer stopped/disconnected events should be fanned out to services that own
  per-peer state and surfaced through the public network event API. Pubsub
  should retain bounded remote membership state even before local handles exist
  and replay it when the first local handle is created. Metric teardown should
  use the same metric kind as live emission for every metric name and must not
  reset monotonic counters.

### RC-7: Routing and discovery accept unstable or self-referential topology

- Representative issues: ISSUE-003, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-008, ISSUE-033, ISSUE-044, ISSUE-055, ISSUE-092, ISSUE-103,
  ISSUE-112 through ISSUE-114, ISSUE-160, ISSUE-161, ISSUE-164, ISSUE-167,
  ISSUE-177, ISSUE-180, ISSUE-181, ISSUE-190, ISSUE-192, ISSUE-197.
- Pattern: route/discovery inputs can include local ids, self seeds, stale
  addresses, overflowed metrics, over-hop routes, duplicate connection races, or
  explicit connect addresses that are ignored by peer-id-only fast paths, or
  tiny RTT jitter that changes active paths too aggressively. Malformed route
  or discovery syncs can also contain duplicate destination rows whose last
  value silently wins before validation. Stream setup and unicast forwarding
  also trust route choices without checking whether the next hop is the same
  ingress connection, and local advertise config can gossip non-dialable
  addresses.
- Minimal fix proposal: add route/discovery sanitization before insertion:
  reject local/self candidates and over-hop routes, pin authenticated direct
  paths for their peer ids, use checked metric math, ignore stale discovery
  timestamps, reject duplicate destination rows in a single route or discovery
  sync, coalesce duplicate connects, validate already-connected peer addresses,
  add a small hysteresis threshold before switching active paths, and reject
  relay stream or unicast hops that would forward back to their ingress
  connection. Validate configured local advertise addresses before gossiping
  them.

### RC-8: Public examples are not compile-checked against the API

- Representative issues: ISSUE-191.
- Pattern: documentation snippets can drift from the exported API because they
  are not compiled as examples, doctests, or compile tests. This lets onboarding
  code demonstrate invalid result handling or mutability even when maintained
  examples still compile.
- Minimal fix proposal: make README snippets executable examples or doctests,
  and add a focused compile gate for the getting-started path. Keep snippets
  using real `Result` handling and mutable bindings where the API requires
  mutation.

## Accepted Issues

### ISSUE-001: Forged third-party `PeerStopped` removes a live peer

- Category: security, correctness
- Score: 92/100
- Reviewer: `Leibniz`, confirmed. Also confirmed by `Bernoulli`, `Wegener`,
  and `Carver`.
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
  - Additional propagation evidence:
    `cargo test forged_peer_stopped_must_not_be_forwarded_to_other_neighbours -- --nocapture`
  - Failure summary: a relay forwards a forged stop for an unrelated victim,
    causing an observer to remove its route to that victim.

### ISSUE-002: Future-dated handshake timestamps are accepted

- Category: security
- Score: 74/100
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
- Score: 68/100
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
- Score: 86/100
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
- Score: 62/100
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
- Score: 70/100
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
- Score: 72/100
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
- Score: 58/100
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
- Score: 78/100
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
- Score: 84/100
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
- Score: 76/100
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
- Score: 78/100
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
- Score: 57/100
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
- Score: 94/100
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
- Score: 94/100
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
- Score: 72/100
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
- Score: 82/100
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
- Score: 93/100
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
- Score: 54/100
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
- Score: 88/100
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
- Score: 74/100
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
- Score: 72/100
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
- Score: 76/100
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
- Score: 70/100
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
- Score: 78/100
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
- Score: 66/100
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
- Score: 82/100
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
- Score: 58/100
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
- Score: 57/100
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
- Score: 52/100
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
- Score: 64/100
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
- Score: 61/100
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
- Score: 78/100
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
- Score: 83/100
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
- Score: 68/100
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
- Score: 55/100
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
- Score: 73/100
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
- Score: 70/100
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
- Score: 86/100
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
- Score: 45/100
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
- Score: 68/100
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
- Score: 52/100
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
- Score: 72/100
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
- Score: 74/100
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
- Score: 76/100
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
- Score: 78/100
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
- Score: 82/100
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
- Score: 86/100
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
- Score: 70/100
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
- Score: 68/100
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
- Score: 63/100
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
- Score: 56/100
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
- Score: 84/100
- Reviewer: `Hooke`, confirmed. Additional fuzz evidence confirmed by
  `Socrates the 2nd` and `Hilbert the 3rd`.
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
  - Additional fuzz evidence:
    `cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  - Failure summary: with default `P2P_FUZZ_NODES=5`,
    `P2P_FUZZ_STEPS=120`, and `P2P_FUZZ_SEED=0x5eed`, the random action
    harness reaches the broadcast variant of the same bug by injecting
    `PeerMessage::Broadcast(..., P2pServiceId::from(256), ...)`, panicking a
    background connection task at `src/ctx.rs:33`.
  - Additional churn fuzz evidence:
    `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=40 cargo test fuzz_random_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  - Churn fuzz failure summary: after the five-cycle no-new threshold, the
    node-churn random action harness also reaches the broadcast variant by
    injecting `PeerMessage::Broadcast(..., P2pServiceId::from(256), ...)`,
    panicking a background connection task at `src/ctx.rs:33`.

### ISSUE-054: Zero network tick interval panics node construction

- Category: correctness, configuration stability
- Score: 45/100
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
- Score: 64/100
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
- Score: 70/100
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
- Score: 67/100
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
- Score: 58/100
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
- Score: 80/100
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
- Score: 60/100
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
  - Additional reviewer `Turing the 2nd` accepted
    `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
    as closed-destination evidence for this stale service sender root cause.
  - Additional failure summary: after node2 drops its destination
    `P2pService`, node1's live service can still route a unicast to node2 and
    observe `Ok(())`; the stale registered sender means the closed destination
    remains addressable until delivery is later discarded.

### ISSUE-061: Visualization accepts unsolicited forged topology info

- Category: security, topology correctness
- Score: 74/100
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
- Score: 76/100
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
- Score: 72/100
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
  - Additional fuzz evidence:
    `cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  - Fuzz failure summary: with default `P2P_FUZZ_NODES=5`,
    `P2P_FUZZ_STEPS=300`, and `P2P_FUZZ_SEED=0x5eed`, valid random actions
    including duplicate connects, forged in-range unicast, streams, and
    `PeerStopped` messages panic background tasks at `src/router.rs:76`,
    rediscovering the stale `PeerData::Sync` route-missing crash without using
    the known invalid service-id shortcut.
  - Additional churn fuzz evidence, confirmed by `Laplace the 3rd`:
    `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=60 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  - Churn fuzz failure summary: valid-only shutdown/restart churn, duplicate
    connects, in-range raw messages, streams, broadcasts, and forged
    `PeerStopped` messages panic a background task at `src/router.rs:76`,
    rediscovering the stale `PeerData::Sync` route-missing crash without using
    an out-of-range service id.

### ISSUE-064: Stale peer stats events publish metrics for unknown connections

- Category: correctness, monitoring stability
- Score: 55/100
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
- Score: 54/100
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
- Score: 68/100
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
- Score: 73/100
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
- Score: 60/100
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
- Score: 66/100
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
- Score: 66/100
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

### ISSUE-071: Replicated KV retries stale FetchChanged after broadcasts fill the gap

- Category: correctness, high-load/backpressure stability
- Score: 58/100
- Reviewer: `Herschel`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::apply_pendings` stores `self.sending_req` when a version gap
    is detected.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` can later receive the missing change by
    broadcast and advance `self.version`, but does not clear the old pending
    `FetchChanged` request.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_tick` blindly resends `self.sending_req` after timeout.
- Impact: after normal broadcasts already fill a version gap and the remote
  store converges, the node continues retrying an obsolete `FetchChanged`
  request on every timeout. Under packet loss or high load this creates
  unnecessary repair traffic and can keep requesting stale ranges after the
  replica is already up to date. This is distinct from ISSUE-027 and ISSUE-046,
  which cover unbounded pending-change memory growth; this issue is stale
  request state after convergence.
- Evidence test:
  - `cargo test working_state_must_cancel_fetch_changed_when_broadcast_fills_gap -- --nocapture`
  - Failure summary: after `Changed(version=2)` queues
    `FetchChanged { from: Version(1), count: 1 }`, a later broadcast
    `Changed(version=1)` fills the gap and advances the state to `Version(2)`,
    but ticking after `REQUEST_TIMEOUT` still emits the obsolete
    `FetchChanged { from: Version(1), count: 1 }` instead of no retry.

### ISSUE-072: Dropped service requesters can still send unicast

- Category: correctness, service lifecycle stability
- Score: 70/100
- Reviewer: `Arendt`, confirmed.
- Affected code:
  - `src/service.rs`: `P2pServiceRequester` is cloneable and keeps only
    `service`, plus `SharedCtx`, so it outlives the owning `P2pService`.
  - `src/service.rs`: `P2pServiceRequester::send_unicast` delegates directly to
    `SharedCtx::send_unicast` without checking that the service receiver is
    still alive.
  - `src/ctx.rs`: `SharedCtx::send_unicast` sends `PeerMessage::Unicast` with
    the supplied service id as long as a route exists.
- Impact: after a `P2pService` is dropped, a previously cloned requester can
  still send unicast messages under that service id. Remote peers receive data
  from a service instance whose local owner has already been destroyed. This is
  distinct from ISSUE-060, which covers local service-id reservation after drop,
  and from ISSUE-069/070, which cover pubsub-specific publisher/subscriber
  requesters.
- Evidence test:
  - `cargo test dropped_service_requester_must_not_continue_sending_unicast -- --nocapture`
  - Failure summary: after dropping node1's `P2pService`, a cloned
    `P2pServiceRequester` sends `stale-service-unicast`, and node2's service
    receives `P2pServiceEvent::Unicast`.

### ISSUE-073: Dropped service requesters can still open streams

- Category: correctness, pipe lifecycle stability
- Score: 72/100
- Reviewer: `Parfit`, confirmed.
- Affected code:
  - `src/service.rs`: `P2pServiceRequester` is cloneable and keeps only
    `service`, plus `SharedCtx`, so it outlives the owning `P2pService`.
  - `src/service.rs`: `P2pServiceRequester::open_stream` delegates directly to
    `SharedCtx::open_stream` without checking that the service receiver is
    still alive.
  - `src/ctx.rs`: `SharedCtx::open_stream` opens a routed stream with the
    supplied service id as long as a route exists.
- Impact: after a `P2pService` is dropped, a previously cloned requester can
  still open streams under that service id. Remote peers receive
  `P2pServiceEvent::Stream` from a service instance whose local owner has
  already been destroyed. This is distinct from ISSUE-072, which covers stale
  requester unicast, and from ISSUE-011/012/013/056, which cover other stream
  delivery, panic, and backpressure failure modes.
- Evidence test:
  - `cargo test dropped_service_requester_must_not_continue_opening_streams -- --nocapture`
  - Failure summary: after dropping node1's `P2pService`, a cloned
    `P2pServiceRequester` opens `stale-service-stream`, and node2's service
    receives `P2pServiceEvent::Stream`.

### ISSUE-074: Dropped publisher requesters can still issue publish RPCs

- Category: correctness, pubsub lifecycle stability
- Score: 70/100
- Reviewer: `Galileo`, confirmed.
- Affected code:
  - `src/service/pubsub_service/publisher.rs`: `PublisherRequester` is
    cloneable and retains `local_id`, `channel_id`, and the service control
    sender after the owning `Publisher` is dropped.
  - `src/service/pubsub_service/publisher.rs`: `Publisher::drop` sends
    `InternalMsg::PublisherDestroyed(local_id, channel_id)`.
  - `src/service/pubsub_service.rs`:
    `InternalMsg::PublishRpc(_local_id, channel, data, method, tx, timeout)`
    ignores `_local_id`, delivers RPCs to local/remote subscribers, and inserts
    pending RPC state whenever the channel has destinations.
- Impact: after the last local publisher is dropped and subscribers observe
  `PeerLeaved(PeerSrc::Local)`, any cloned requester from that dropped
  publisher can still issue publish RPCs on the channel. This invokes
  subscriber RPC handlers and creates pending RPC state from a publisher the
  pubsub state already considers gone. This is distinct from ISSUE-069, which
  covers ordinary publishes, and from ISSUE-020/043/048, which cover other
  pubsub RPC answer, retention, and remote membership failure modes.
- Evidence test:
  - `cargo test dropped_publisher_requester_must_not_continue_publish_rpc -- --nocapture`
  - Failure summary: after dropping the publisher and receiving
    `SubscriberEvent::PeerLeaved(PeerSrc::Local)`, a cloned stale requester
    issues `publish_rpc("stale", b"stale-publish-rpc", ...)`, and the
    subscriber still receives `SubscriberEvent::PublishRpc`.

### ISSUE-075: Dropped subscriber requesters can still issue feedback RPCs

- Category: correctness, pubsub lifecycle stability
- Score: 70/100
- Reviewer: `Lorentz`, confirmed.
- Affected code:
  - `src/service/pubsub_service/subscriber.rs`: `SubscriberRequester` is
    cloneable and retains `local_id`, `channel_id`, and the service control
    sender after the owning `Subscriber` is dropped.
  - `src/service/pubsub_service/subscriber.rs`: `Subscriber::drop` sends
    `InternalMsg::SubscriberDestroyed(local_id, channel_id)`.
  - `src/service/pubsub_service.rs`:
    `InternalMsg::FeedbackRpc(_local_id, channel, data, method, tx, timeout)`
    ignores `_local_id`, delivers RPCs to local/remote publishers, and inserts
    pending RPC state whenever the channel has destinations.
- Impact: after the last local subscriber is dropped and publishers observe
  `PeerLeaved(PeerSrc::Local)`, any cloned requester from that dropped
  subscriber can still issue feedback RPCs on the channel. This invokes
  publisher RPC handlers and creates pending RPC state from a subscriber the
  pubsub state already considers gone. This is distinct from ISSUE-070, which
  covers ordinary feedback, and from ISSUE-074, which covers the publisher-side
  publish RPC mirror.
- Evidence test:
  - `cargo test dropped_subscriber_requester_must_not_continue_feedback_rpc -- --nocapture`
  - Failure summary: after dropping the subscriber and receiving
    `PublisherEvent::PeerLeaved(PeerSrc::Local)`, a cloned stale requester
    issues `feedback_rpc("stale", b"stale-feedback-rpc", ...)`, and the
    publisher still receives `PublisherEvent::FeedbackRpc`.

### ISSUE-076: Dropped service requesters can still send broadcast

- Category: correctness, service lifecycle stability
- Score: 69/100
- Reviewer: `Hume`, confirmed.
- Affected code:
  - `src/service.rs`: `P2pServiceRequester` is cloneable and keeps only
    `service`, plus `SharedCtx`, so it outlives the owning `P2pService`.
  - `src/service.rs`: `P2pServiceRequester::send_broadcast` delegates directly
    to `SharedCtx::send_broadcast` without checking that the service receiver
    is still alive.
  - `src/ctx.rs`: `SharedCtx::send_broadcast` fans out
    `PeerMessage::Broadcast` with the supplied service id to all current
    connections.
- Impact: after a `P2pService` is dropped, a previously cloned requester can
  still send broadcast messages under that service id. Remote peers receive
  broadcast data from a service instance whose local owner has already been
  destroyed. This is distinct from ISSUE-072, which covers unicast, and
  ISSUE-073, which covers stream setup.
- Evidence test:
  - `cargo test dropped_service_requester_must_not_continue_sending_broadcast -- --nocapture`
  - Failure summary: after dropping node1's `P2pService`, a cloned
    `P2pServiceRequester` sends `stale-service-broadcast`, and node2's service
    receives `P2pServiceEvent::Broadcast`.

### ISSUE-077: Replicated KV zero changed batch size returns false empty success

- Category: correctness, bad-network stability
- Score: 60/100
- Reviewer: `Peirce`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvService::new` exposes
    `max_compose_pkts` directly to callers without rejecting zero.
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::new` stores `compose_max_pkts` unchecked.
  - `src/service/replicate_kv_service/local_storage.rs`:
    `changeds_from_to` computes `to = from + count.min(compose_max_pkts)`.
    When `compose_max_pkts` is zero, `to == from`, the range checks pass, and
    an empty successful response is produced.
- Impact: a node can reply to `FetchChanged { from, count }` with
  `Ok(vec![])` even when the requested change exists. Receivers can treat the
  repair response as successful while making no version progress, causing
  changed-sync repair to stall under a valid but broken zero-batch
  configuration. This is distinct from ISSUE-032, which covers zero snapshot
  page size in full sync.
- Evidence test:
  - `cargo test zero_changed_batch_size_must_not_return_empty_success -- --nocapture`
  - Failure summary: with `compose_max_pkts = 0` and version `1` present,
    `FetchChanged { from: Version(1), count: 1 }` returns
    `RpcRes::FetchChanged(Ok(vec![]))` instead of rejecting the request or
    returning at least one change.

### ISSUE-078: Metrics service discloses metrics to arbitrary scan requests

- Category: security, monitoring integrity
- Score: 78/100
- Reviewer: `Euler`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: `MetricsService::recv` accepts
    `Message::Scan` from any unicast or broadcast sender.
  - `src/service/metrics_service.rs`: the `Message::Scan` branch collects
    `self.service.ctx.metrics()` and sends `Message::Info(metrics)` back to the
    sender without checking `is_collector` or any pending request state.
- Impact: any connected peer can send a metrics `Scan` frame to a non-collector
  node and force it to disclose local connection metrics. This is distinct from
  ISSUE-062, which covers accepting unsolicited forged `Info` and poisoning
  metrics output; this issue is unauthorized metrics disclosure in response to
  unsolicited `Scan`.
- Evidence test:
  - `cargo test metrics_scan_must_not_disclose_metrics_to_non_collector -- --nocapture`
  - Failure summary: node2 injects a metrics `Scan` into node1's metrics
    service, and node2's base service receives a unicast response from node1
    containing a metrics `Info` frame.

### ISSUE-079: Visualization service discloses topology to arbitrary scan requests

- Category: security, topology integrity
- Score: 76/100
- Reviewer: `Hypatia`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: `VisualizationService::recv`
    accepts `Message::Scan` from any unicast or broadcast sender.
  - `src/service/visualization_service.rs`: the `Message::Scan` branch gathers
    `requester.router().neighbours()` and sends `Message::Info(neighbours)`
    back to the sender without checking collector role or any pending request
    state.
- Impact: any connected peer can send a visualization `Scan` frame to a
  non-collector node and force it to disclose local topology/neighbour data.
  This is distinct from ISSUE-061, which covers accepting unsolicited forged
  `Info` and poisoning visualization output, and from ISSUE-078, which covers
  the same disclosure pattern in the metrics service.
- Evidence test:
  - `cargo test visualization_scan_must_not_disclose_topology_to_non_collector -- --nocapture`
  - Failure summary: node2 injects a visualization `Scan` into node1's
    visualization service, and node2's base service receives a unicast response
    from node1 containing a topology `Info` frame.

### ISSUE-080: Pubsub heartbeat does not remove stale remote publishers

- Category: bad-network correctness, pubsub stability
- Score: 68/100
- Reviewer: `Plato`, confirmed. Additional omitted-channel heartbeat evidence
  reviewed by `Bohr` as an ISSUE-080 variant.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `Heartbeat` adds remote publishers
    when `heartbeat.publish` is true.
  - `src/service/pubsub_service.rs`: the same `Heartbeat` branch never removes
    peers from `remote_publishers` when `heartbeat.publish` is false.
  - `src/service/pubsub_service.rs`: explicit `PublisherLeaved` removes and
    notifies local subscribers, but heartbeat repair cannot correct a missed
    leave event.
- Impact: after a lost `PublisherLeaved` message, local subscribers can keep
  believing a remote publisher is still present even after later heartbeats say
  that peer no longer publishes the channel. Subscribers do not receive
  `SubscriberEvent::PeerLeaved`, so bad-network state divergence can persist.
  This is distinct from ISSUE-026, which covers stale remote subscribers
  affecting local publishers.
- Evidence test:
  - `cargo test pubsub_heartbeat_must_remove_stale_remote_publisher -- --nocapture`
  - Failure summary: after injecting `PublisherJoined` from node2, a heartbeat
    with `publish=false` and `subscribe=false` does not produce
    `SubscriberEvent::PeerLeaved(PeerSrc::Remote(node2))`; the test times out
    waiting for the leave event.
  - `cargo test pubsub_empty_heartbeat_must_remove_omitted_stale_remote_publisher -- --nocapture`
  - Failure summary: after injecting `PublisherJoined` from node2, an empty
    heartbeat omitting that channel does not produce
    `SubscriberEvent::PeerLeaved(PeerSrc::Remote(node2))`; the test times out
    waiting for the leave event.

### ISSUE-081: Replicated KV full sync accepts an initial empty snapshot as nonzero complete state

- Category: correctness, security
- Score: 72/100
- Reviewer: `Turing`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts
    `RpcRes::FetchSnapshot(Some(snapshot), version)` without rejecting an empty
    initial page.
  - `src/service/replicate_kv_service/remote_storage.rs`: when
    `snapshot.next_key` is `None`, the state transitions to
    `WorkingState::new(version)` even if `snapshot.slots` is empty and
    `version` is nonzero.
- Impact: a malicious or malformed peer can claim full sync completed at a
  nonzero version while providing no snapshot data. The receiver then believes
  it is synchronized at that version with an empty remote store, causing silent
  data loss until later repairs happen to expose the gap. This is distinct from
  ISSUE-038, which covers empty continuation pages with `next_key`, and
  ISSUE-059, which covers `None` as a fake continuation response.
- Evidence test:
  - `cargo test full_sync_must_reject_initial_empty_snapshot_with_nonzero_version -- --nocapture`
  - Failure summary: an initial
    `RpcRes::FetchSnapshot(Some(SnapshotData { slots: vec![], next_key: None, ... }), Version(1))`
    sets `ctx.next_state` to `Working(Version(1))`; expected the invalid empty
    nonzero snapshot to be rejected.

### ISSUE-082: Replicated KV full sync accepts snapshot slots beyond biggest_key

- Category: correctness, security
- Score: 70/100
- Reviewer: `Erdos`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` trusts `SnapshotData.biggest_key` for
    pagination but does not validate returned slot keys against that bound.
  - `src/service/replicate_kv_service/remote_storage.rs`: every
    `snapshot.slots` entry is emitted and inserted into `ctx.slots` before any
    range validation.
- Impact: a malicious or malformed peer can send a snapshot page declaring
  `biggest_key = 1` while including a slot for key `2`. The receiver stores
  the out-of-range slot and can complete full sync with data that the advertised
  snapshot range says should not exist. This is distinct from ISSUE-034, which
  validates slot versions, and ISSUE-037, which covers invalid continuation
  bounds.
- Evidence test:
  - `cargo test full_sync_must_reject_snapshot_slot_past_biggest_key -- --nocapture`
  - Failure summary: a snapshot response with
    `slots = [(2, Slot::new(2, Version(1)))]` and `biggest_key = 1` inserts key
    `2` into `ctx.slots`; expected out-of-range snapshot slots to be rejected.

### ISSUE-083: Replicated KV full sync accepts continuation slots before requested key

- Category: correctness, security
- Score: 68/100
- Reviewer: `Jason`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` requests continuation snapshots from
    `snapshot.next_key`, but does not remember or validate that lower bound.
  - `src/service/replicate_kv_service/remote_storage.rs`: continuation
    `snapshot.slots` entries are emitted and inserted even if their keys are
    before the requested `next_key`.
- Impact: a malicious or malformed peer can answer a continuation request that
  should start at key `5` with a slot for key `4`. The receiver stores the
  out-of-request-range slot and can complete full sync using an invalid page.
  This is distinct from ISSUE-082, which covers slots past the advertised
  `biggest_key` upper bound, and from ISSUE-047, which covers continuation
  version mismatch.
- Evidence test:
  - `cargo test full_sync_must_reject_continuation_slot_before_requested_key -- --nocapture`
  - Failure summary: after a first page with `next_key = Some(5)`, a
    continuation page containing key `4` is inserted into `ctx.slots`; expected
    slots before the requested continuation key to be rejected.

### ISSUE-084: Replicated KV full sync accepts unsorted snapshot pages

- Category: correctness, security
- Score: 62/100
- Reviewer: `Aquinas`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`: valid snapshot pages
    are produced from a `BTreeMap` range, so slots are ordered by key.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` trusts remote `SnapshotData.slots` and inserts
    them without checking monotonic key order.
- Impact: a malicious or malformed peer can send a snapshot page such as
  `[(2, ...), (1, ...)]`. The receiver stores both slots and can complete full
  sync while accepting a page shape that a valid producer would not emit. This
  is distinct from ISSUE-082 and ISSUE-083, which cover upper/lower key bounds;
  this issue covers per-page key ordering.
- Evidence test:
  - `cargo test full_sync_must_reject_unsorted_snapshot_slots -- --nocapture`
  - Failure summary: a snapshot response with slots ordered as keys `2, 1` is
    accepted and stored; expected unsorted snapshot pages to be rejected.

### ISSUE-085: Replicated KV full sync accepts duplicate keys in one snapshot page

- Category: correctness, security
- Score: 64/100
- Reviewer: `Kepler`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`: valid snapshot pages
    are produced from a `BTreeMap` range, so a key can appear at most once per
    page.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` trusts remote `SnapshotData.slots`, emits each
    slot, and inserts into `ctx.slots` without rejecting duplicate keys.
- Impact: a malicious or malformed peer can send duplicate keys with
  conflicting values in one snapshot page. The receiver emits multiple
  `KvEvent::Set` events and silently keeps the last value while completing full
  sync. This is distinct from ISSUE-084, which covers per-page ordering, and
  from ISSUE-082/ISSUE-083, which cover upper/lower key bounds.
- Evidence test:
  - `cargo test full_sync_must_reject_duplicate_snapshot_keys -- --nocapture`
  - Failure summary: a snapshot response with duplicate key `1` is accepted;
    the second value overwrites the first in `ctx.slots`; expected duplicate-key
    snapshot pages to be rejected.

### ISSUE-086: Replicated KV applies unsolicited FetchChanged success responses

- Category: correctness, security
- Score: 80/100
- Reviewer: `Hegel`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: any deserialized unicast
    `RpcEvent::RpcRes` from a peer is forwarded to that peer's `RemoteStore`
    without a request id or request-correlation check.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` applies
    `RpcRes::FetchChanged(Ok(changeds))` without checking that
    `self.sending_req` is currently waiting for a `FetchChanged` response.
- Impact: a peer can send a single unsolicited `FetchChanged` success for the
  next version and force the receiver to advance its replicated version, mutate
  `ctx.slots`, and emit local `KvEvent` changes. This is distinct from
  ISSUE-046, which covers unbounded batch/resource exhaustion; this issue uses
  one next-version change and demonstrates request-correlation/integrity
  failure.
- Evidence test:
  - `cargo test working_state_must_reject_unsolicited_fetch_changed_success -- --nocapture`
  - Failure summary: with no outstanding `FetchChanged` request, an unsolicited
    response containing `Changed { version: Version(1), ... }` advances
    `state.version` from `Version(0)` to `Version(1)`; expected it to be
    rejected.

### ISSUE-087: Replicated KV accepts unsolicited FetchChanged errors as forced resyncs

- Category: correctness, security, stability
- Score: 78/100
- Reviewer: `Bacon`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: any deserialized unicast
    `RpcEvent::RpcRes` from a peer is forwarded to that peer's `RemoteStore`
    without request correlation.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` accepts
    `RpcRes::FetchChanged(Err(_))` without checking that `self.sending_req`
    is waiting for a `FetchChanged` response, and schedules `SyncFull`.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::init` clears all existing remote slots, emits deletes, and
    queues a full snapshot request.
- Impact: a peer can send a single unsolicited `FetchChanged` error and force
  the receiver to leave working state, delete the peer's replicated slots, emit
  local delete events, and start full-sync churn. This is distinct from
  ISSUE-086, which covers unsolicited success responses advancing/mutating
  state, and from ISSUE-046, which covers unbounded success-batch memory growth.
- Evidence test:
  - `cargo test working_state_must_reject_unsolicited_fetch_changed_error -- --nocapture`
  - Failure summary: with no outstanding `FetchChanged` request, an unsolicited
    `FetchChanged(Err(MissingData))` clears existing slot key `7`; expected the
    remote store to remain in working state with its slots intact.

### ISSUE-088: Replicated KV accepts duplicate versions in FetchChanged responses

- Category: correctness, security
- Score: 70/100
- Reviewer: `Helmholtz`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::changeds_from_to` emits changes from a `BTreeMap` range keyed
    by `Version`, so a legitimate response cannot contain duplicate versions.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` inserts every `FetchChanged(Ok(_))` item into
    `pendings: BTreeMap<Version, Changed<_, _>>`, so duplicate versions silently
    overwrite earlier entries before `apply_pendings`.
- Impact: after a real outstanding `FetchChanged` request, a malicious or
  malformed peer can send two conflicting `Changed` entries for the same
  version. The receiver silently keeps the later one, advances its working
  version, mutates slots, and emits a local `KvEvent`. This is distinct from
  ISSUE-085, which covers duplicate keys in snapshot pages; ISSUE-086/087,
  which cover unsolicited responses; and ISSUE-046, which covers unbounded
  response batches.
- Evidence test:
  - `cargo test working_state_must_reject_duplicate_fetch_changed_versions -- --nocapture`
  - Failure summary: after requesting `FetchChanged { from: Version(1), count:
    1 }`, a response with two `Changed { version: Version(1), ... }` entries is
    accepted and advances `state.version` to `Version(1)`; expected the
    duplicate-version response to be rejected.

### ISSUE-089: Replicated KV applies FetchChanged versions beyond the requested count

- Category: correctness, security
- Score: 72/100
- Reviewer: `Planck`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::changeds_from_to` computes `to = from + count` (bounded by
    compose size) and emits only `self.changeds.range(from..to)`.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` records a `FetchChanged { from, count }`
    request, but `WorkingState::on_rpc_res` does not validate returned
    `Changed.version` values against that requested range.
- Impact: after a real outstanding `FetchChanged { from: Version(1), count: 1
  }` request, a malicious or malformed peer can include both versions `1` and
  `2` in the response. The receiver applies the extra out-of-range version,
  advances to `Version(2)`, mutates slots, and emits local `KvEvent` changes.
  This is distinct from ISSUE-088, which covers duplicate versions, and from
  ISSUE-046, which covers unbounded response batches.
- Evidence test:
  - `cargo test working_state_must_reject_fetch_changed_versions_beyond_requested_count -- --nocapture`
  - Failure summary: after requesting one missing change, a response containing
    versions `1` and `2` is accepted and advances `state.version` to
    `Version(2)`; expected versions beyond the requested count to be rejected.

### ISSUE-090: Alias cached hint lookup accepts Found from unchecked peers

- Category: correctness, security
- Score: 70/100
- Reviewer: `Banach`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: cached alias lookup sends
    `AliasMessage::Check(alias)` only to peers in the cached hint set and stores
    that set in `FindRequestState::CheckHint`.
  - `src/service/alias_service.rs`: `AliasMessage::Found(alias)` unconditionally
    inserts the sender into the cache and completes any pending lookup for that
    alias, without checking whether the sender was one of the hinted peers that
    received `Check`.
- Impact: an unrelated peer can race `Found(alias)` into a cached-hint lookup,
  complete the caller with `AliasFoundLocation::Hint(unchecked_peer)`, and
  poison the alias hint cache. This is distinct from the existing alias issues
  about refcount overflow, shutdown cache eviction, stale requester panic,
  pending-find growth, and timeout arithmetic.
- Evidence test:
  - `cargo test cached_hint_find_must_ignore_found_from_unchecked_peer -- --nocapture`
  - Failure summary: after a cached lookup checks only `PeerId(1)`, a
    `Found(alias)` from `PeerId(2)` completes the lookup; expected unchecked
    peers to be ignored during `CheckHint`.

### ISSUE-091: Inbound out-of-range stream service ids panic accept tasks

- Category: security, stability
- Score: 84/100
- Reviewer: `Laplace`, confirmed.
- Affected code:
  - `src/msg.rs`: `StreamConnectReq.service` is a wire-controlled
    `P2pServiceId(u16)`.
  - `src/peer/peer_internal.rs`: `accept_bi` handles every inbound
    bidirectional stream and calls `ctx.get_service(&service)` for local stream
    destinations.
  - `src/ctx.rs`: `SharedCtxInternal::get_service` indexes the 256-slot service
    table with `services[*service_id as usize]` without validating the `u16`
    value.
- Impact: an authenticated peer can open a bidirectional stream with
  `P2pServiceId(256)` and trigger an out-of-bounds panic in the receiver's
  stream accept task. This is distinct from ISSUE-053, which covers
  out-of-range service ids on the main peer-message unicast/broadcast path; this
  issue covers the separate `StreamConnectReq`/bidirectional-stream path.
- Evidence test:
  - `cargo test inbound_out_of_range_stream_service_id_must_not_panic_accept_task -- --nocapture`
  - Failure summary: the test's panic hook observes an out-of-bounds panic at
    `src/ctx.rs` after opening a stream with service id `256`; expected the
    invalid stream request to be rejected without panicking in the accept task.

### ISSUE-092: Discovery accepts stale advertisements over newer peer addresses

- Category: correctness, security, stability
- Score: 68/100
- Reviewer: `Nash`, confirmed.
- Affected code:
  - `src/discovery.rs`: `PeerDiscovery::apply_sync` accepts any non-expired
    advertisement and unconditionally writes
    `self.remotes.insert(peer, (last_updated, address))`.
  - `src/discovery.rs`: `create_sync_for` propagates `last_updated` as freshness
    metadata, but the receiver does not compare it with the currently stored
    timestamp for the same peer.
- Impact: stale gossip can roll an active discovered peer back to an older
  address as long as the older advertisement has not timed out. This can cause
  reconnect churn, stale-address poisoning, and instability under reordered or
  delayed discovery syncs. This is distinct from ISSUE-009, which covers
  overflow/future timestamps, ISSUE-005, which covers local-id advertisements,
  and ISSUE-055, which covers configured seed ids.
- Evidence test:
  - `cargo test apply_sync_must_not_overwrite_newer_discovery_with_stale_advertisement -- --nocapture`
  - Failure summary: after learning peer `1` at timestamp `200` and address
    `127.0.0.1:9001`, a later sync containing timestamp `100` and address
    `127.0.0.1:9000` overwrites the newer address; expected the stale
    advertisement to be ignored.

### ISSUE-093: Discovery tombstones suppress fresh restart advertisements

- Category: correctness, stability under bad-network ordering
- Score: 64/100
- Reviewer: `Lagrange`, confirmed.
- Affected code:
  - `src/discovery.rs`: `PeerDiscovery::remove_remote` records a stop tombstone
    timestamp for non-seed peers.
  - `src/discovery.rs`: `PeerDiscovery::apply_sync` rejects every advertisement
    for a peer while the tombstone is fresh, without comparing the
    advertisement's `last_updated` timestamp against the tombstone timestamp.
- Impact: a non-seed node that gracefully stops and quickly restarts at a new
  address can remain undiscoverable for `TIMEOUT_AFTER` despite fresh gossip.
  This can delay reconnects and create churn in lossy or reordered networks.
  This is distinct from ISSUE-051, which covers neighbour cleanup after
  `PeerStopped`; ISSUE-055, which covers configured seed ids; and ISSUE-092,
  which covers stale advertisements overwriting newer active discovery records.
- Evidence test:
  - `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`
  - Failure summary: after learning peer `2` at timestamp `100`, recording a
    stop at timestamp `110`, and receiving a fresh restart advertisement at
    timestamp `120` with address `127.0.0.1:9002`, `remotes()` stays empty;
    expected the fresh restart advertisement to be accepted.

### ISSUE-094: Pubsub object helper panics on user serialization failure

- Category: correctness, API stability
- Score: 52/100
- Reviewer: `Averroes`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `PubsubServiceRequester::publish_as_guest_ob`
    returns `anyhow::Result<()>` but calls
    `bincode::serialize(&ob).expect("should serialize")` on caller-supplied
    `Serialize`.
  - `src/service/pubsub_service.rs`,
    `src/service/pubsub_service/publisher.rs`, and
    `src/service/pubsub_service/subscriber.rs`: similar public object helpers
    use `expect` around serialization for user-provided objects.
- Impact: a valid `Serialize` implementation can return an error, but these
  public async helpers unwind instead of propagating `Err`. That breaks API
  stability and lets local caller input panic pubsub code paths that are typed
  as recoverable `Result` APIs.
- Evidence test:
  - `cargo test pubsub_guest_object_publish_must_return_error_on_serialize_failure -- --nocapture`
  - Failure summary: `publish_as_guest_ob(...)` unwinds at
    `src/service/pubsub_service.rs:658` when serialization returns a custom
    error; expected the helper to return `Ok(Err(_))` rather than panic.

### ISSUE-095: Replicated KV duplicate future broadcasts overwrite pending changes

- Category: correctness, security
- Score: 74/100
- Reviewer: `Pauli`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` accepts future `BroadcastEvent::Changed`
    values and stores them with
    `self.pendings.insert(changed.version, changed)`.
- Impact: a malicious or malformed peer can send two conflicting future
  `Changed` broadcasts for the same version before the missing gap is filled.
  The later duplicate silently overwrites the first pending value, and the
  receiver later applies and emits the overwritten data as if it were the valid
  change for that version. This is distinct from ISSUE-027, which covers
  unbounded pending future broadcasts, and ISSUE-088/ISSUE-089, which cover
  `FetchChanged` response validation.
- Evidence test:
  - `cargo test working_state_must_reject_duplicate_pending_changed_broadcast_versions -- --nocapture`
  - Failure summary: duplicate pending broadcast `Version(2)` overwrites value
    `20` with `99`; after `Version(1)` fills the gap, the slot contains
    `Some(Slot { value: 99, version: Version(2) })` instead of preserving or
    rejecting the first accepted version.

### ISSUE-096: Replicated KV recv panics on user value serialization failure

- Category: correctness, API stability
- Score: 55/100
- Reviewer: `Meitner`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvService::recv`
    serializes outbound `BroadcastEvent` and `RpcEvent` with
    `bincode::serialize(...).expect("should serialize")`.
- Impact: caller-provided replicated-KV key/value types can implement
  `Serialize` and legitimately return an error. A local `set` can then make
  `recv()` unwind while producing outbound events instead of surfacing or
  containing the serialization failure, breaking service API stability. This is
  distinct from ISSUE-094, which covers pubsub object helper serialization.
- Evidence test:
  - `cargo test replicated_kv_recv_must_not_panic_on_value_serialize_failure -- --nocapture`
  - Failure summary: `service.recv()` panics at
    `src/service/replicate_kv_service.rs:163` when serializing a value whose
    `Serialize` implementation returns a custom error.

### ISSUE-097: QUIC object writer panics on serialization failure

- Category: correctness, API stability
- Score: 58/100
- Reviewer: `Poincare`, confirmed.
- Affected code:
  - `src/stream.rs`: `write_object` returns `anyhow::Result<()>` but calls
    `bincode::serialized_size(object).expect(...)` and
    `bincode::serialize(object).expect(...)`.
- Impact: a valid `Serialize` implementation can return an error, but the
  shared QUIC object writer unwinds instead of propagating `Err`. This can panic
  connection handshake or stream-control write paths that rely on the helper.
  This is distinct from ISSUE-094 and ISSUE-096, which cover service-level
  pubsub and replicated-KV serialization surfaces.
- Evidence test:
  - `cargo test write_object_must_return_error_on_serialize_failure -- --nocapture`
  - Failure summary: `write_object::<_, _, 1024>` panics at
    `src/stream.rs:109` when serialization returns a custom error; expected
    `Ok(Err(_))`.

### ISSUE-098: QUIC object writer truncates lengths above `u16::MAX`

- Category: correctness, API stability
- Score: 50/100
- Reviewer: `Nash the 2nd`, confirmed.
- Affected code:
  - `src/stream.rs`: `write_object` checks serialized size only against the
    generic `MAX_SIZE`, then writes `(data_buf.len() as u16).to_be_bytes()`.
  - `src/stream.rs`: `wait_object` reads a two-byte length prefix, so the
    object format cannot represent payloads larger than `u16::MAX`.
- Impact: if `write_object` is instantiated with `MAX_SIZE > u16::MAX`, it can
  accept a serialized object whose length cannot be represented by the wire
  prefix, return `Ok(())`, and emit a truncated length. A receiver then reads
  only the wrapped length, likely fails deserialization, and leaves trailing
  bytes in the stream, desynchronizing the helper protocol. Current in-tree
  production constants are below `u16::MAX`; the issue is in the public generic
  helper contract. This is distinct from ISSUE-024's main peer-message codec
  cap and ISSUE-097's serialization-error panic.
- Evidence test:
  - `cargo test write_object_must_reject_payloads_larger_than_u16_length_prefix -- --nocapture`
  - Failure summary: `write_object::<_, _, 100_000>` returns `Ok(())` for a
    70 KB payload even though the two-byte length prefix cannot represent it;
    expected a recoverable error.

### ISSUE-174: QUIC object writer can bypass `MAX_SIZE` with non-deterministic serialization

- Category: correctness, API stability, framing validation
- Score: 46/100
- Reviewer: `Hypatia the 3rd`, confirmed after `Locke the 3rd` discovery.
- Affected code:
  - `src/stream.rs`: `write_object` calls
    `bincode::serialized_size(object)` for the size check, then calls
    `bincode::serialize(object)` again and writes the second buffer.
  - `src/stream.rs`: the actual `data_buf.len()` is not rechecked against
    `MAX_SIZE` before writing the length prefix and payload.
  - `src/peer.rs` and `src/peer/peer_internal.rs`: handshake and stream setup
    paths rely on this shared object writer.
- Impact: a legal but stateful or otherwise non-deterministic `Serialize`
  implementation can produce a small payload for the estimate pass and a larger
  payload for the actual write pass. `write_object` can then return `Ok(())`
  and emit a frame larger than the configured protocol cap even when
  `MAX_SIZE <= u16::MAX`. This is distinct from ISSUE-097, which covers
  serialization errors panicking instead of returning `Err`, and ISSUE-098,
  which covers length-prefix truncation when `MAX_SIZE` itself exceeds the
  two-byte wire limit.
- Minimal fix proposal: serialize exactly once, map serialization failures to
  `Err`, then validate `data_buf.len() <= MAX_SIZE` and
  `data_buf.len() <= u16::MAX` before writing any bytes.
- Evidence test:
  - `cargo test write_object_must_recheck_actual_serialized_size -- --nocapture`
  - Failure summary: a test `Serialize` implementation emits a one-element
    sequence during `serialized_size` and a 32-element sequence during
    `serialize`; `write_object::<_, _, 16>` returns `Ok(())` instead of
    rejecting the actual oversized serialized payload.

### ISSUE-099: Replicated KV accepts zero-count FetchChanged as successful repair

- Category: correctness, bad-network stability
- Score: 64/100
- Reviewer: `Laplace the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::on_rpc_req` passes remote `RpcReq::FetchChanged { from,
    count }` directly to `changeds_from_to`.
  - `src/service/replicate_kv_service/local_storage.rs`:
    `changeds_from_to` computes `to = from + count.min(compose_max_pkts)`.
    With a remote-supplied `count = 0`, `to == from`; if `from` is retained,
    the range checks pass and the empty range returns `Ok(vec![])`.
- Impact: a malformed or malicious peer can make a node answer a repair request
  with a successful empty `FetchChanged` response that makes no version
  progress. This is distinct from ISSUE-077, which covers the local
  `compose_max_pkts = 0` configuration; here the compose budget is valid and
  the no-progress result is caused by unvalidated remote input.
- Evidence test:
  - `cargo test fetch_changed_with_zero_count_must_not_return_empty_success -- --nocapture`
  - Failure summary: with `compose_max_pkts = 2` and version `1` retained,
    `FetchChanged { from: Version(1), count: 0 }` returns
    `RpcRes::FetchChanged(Ok(vec![]))` instead of rejecting the no-progress
    request.

### ISSUE-100: Pubsub remote membership sets are unbounded per channel

- Category: high-load stability, resource exhaustion
- Score: 76/100
- Reviewer: `Ramanujan the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PublisherJoined` inserts
    `from_peer` into `state.remote_publishers`.
  - `src/service/pubsub_service.rs`: inbound `SubscriberJoined` inserts
    `from_peer` into `state.remote_subscribers`.
  - `src/service/pubsub_service.rs`: inbound `Heartbeat` can also insert
    remote publisher and subscriber membership for the advertised channel.
- Impact: pubsub retains remote membership peer ids in per-channel `HashSet`s
  with no cap, eviction policy, or admission limit. High churn or malformed
  traffic can grow `remote_publishers`/`remote_subscribers` without bound and
  increase later fanout work. Lower-layer source-identity spoofing can amplify
  the issue, but the primary failure is the missing resource bound on retained
  pubsub membership state. This is distinct from ISSUE-039/048's membership
  authorization bypass, ISSUE-026/080's stale heartbeat cleanup failures, and
  ISSUE-014/015's lower-layer source identity binding failures.
- Evidence test:
  - `cargo test remote_publisher_memberships_must_be_bounded -- --nocapture`
  - Failure summary: after 1,025 distinct `PublisherJoined` events for one
    channel, `remote_publishers.len()` is 1,025, exceeding the test cap of
    1,024.

### ISSUE-101: Alias cache peer hints are unbounded per alias

- Category: high-load stability, resource exhaustion
- Score: 70/100
- Reviewer: `Beauvoir the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasServiceInternal.cache` is an
    `LruCache<AliasId, HashSet<PeerId>>`, so the LRU bounds alias keys but not
    the number of peer hints stored under one alias.
  - `src/service/alias_service.rs`: inbound `AliasMessage::NotifySet` inserts
    the sender peer into the alias's `HashSet` without a per-alias cap.
  - `src/service/alias_service.rs`: inbound `AliasMessage::Found` also inserts
    the sender peer into the alias hint set.
  - `src/service/alias_service.rs`: a later cached `Find` iterates the whole
    retained peer set and queues one `Check` per peer.
- Impact: high churn or malformed alias traffic can grow one alias's cached
  peer-hint set without bound and amplify later lookup fanout. Spoofed peer
  identities make this worse, but the primary failure is missing per-alias
  admission or eviction for retained hint peers. This is distinct from
  ISSUE-035's duplicate local waiters, ISSUE-041's distinct pending lookups,
  ISSUE-090's unchecked `Found` lookup poisoning, ISSUE-022's shutdown cache
  eviction, and ISSUE-100's pubsub membership resource bound.
- Evidence test:
  - `cargo test cached_alias_peer_hints_must_be_bounded -- --nocapture`
  - Failure summary: after 1,025 `NotifySet` messages for one alias from
    distinct peer ids, the cached peer-hint set length is 1,025, exceeding the
    test cap of 1,024.

### ISSUE-102: Visualization remote peer state is unbounded

- Category: high-load stability, resource exhaustion
- Score: 66/100
- Reviewer: `Planck the 2nd`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: `VisualizationService.neighbours`
    is a `HashMap<PeerId, u64>` with no admission cap.
  - `src/service/visualization_service.rs`: inbound `Message::Info` inserts
    every sender peer id into `neighbours` and emits a joined or updated event.
  - `src/service/visualization_service.rs`: timeout cleanup runs only inside
    the ticker branch when `collect_interval` is `Some`; `new(None, ...)` has
    no expiry path for retained remote peers.
- Impact: malformed traffic, high churn, or spoofed source identities can grow
  retained visualization peer state without bound. With collection disabled via
  `collect_interval = None`, inserted peers never expire. This is distinct from
  ISSUE-061's unsolicited topology poisoning, ISSUE-079's topology disclosure
  on `Scan`, and the resource-bound issues in pubsub and alias services.
- Evidence test:
  - `cargo test visualization_remote_peers_must_be_bounded -- --nocapture`
  - Failure summary: after 1,025 `Info` frames from distinct peer ids,
    `VisualizationService.neighbours.len()` is 1,025, exceeding the test cap of
    1,024.

### ISSUE-103: Configured self seed is returned as a remote dial candidate

- Category: correctness, configuration stability
- Score: 58/100
- Reviewer: `Carson the 2nd`, confirmed.
- Affected code:
  - `src/discovery.rs`: `PeerDiscovery::enable_local` records the local peer id
    and address, but static seeds are stored separately.
  - `src/discovery.rs`: `PeerDiscovery::remotes()` chains learned remotes with
    `self.seeds.iter().cloned()` without filtering out the local peer id.
  - `src/lib.rs`: `P2pNetwork::process_tick` enqueues `ControlCmd::Connect`
    for every address returned by `discovery.remotes()`.
- Impact: if static seed configuration contains this node's own peer id,
  discovery returns that seed as a remote dial candidate. The learned discovery
  path already has an invariant rejecting local-peer advertisements, but the
  configured seed path bypasses it and can cause self-dial churn or self-route
  setup attempts. This is distinct from ISSUE-005's learned local-peer
  advertisements, ISSUE-055's learned advertisements for configured seed ids,
  and ISSUE-006's router acceptance of local-peer routes.
- Evidence test:
  - `cargo test configured_seed_with_local_peer_id_must_not_be_dial_candidate -- --nocapture`
  - Failure summary: after enabling local peer `1` with a configured seed
    `1@127.0.0.1:9000`, `PeerDiscovery::remotes()` still returns peer `1` as a
    remote candidate.

### ISSUE-104: Metrics `Info` batches have no service-level row cap

- Category: high-load stability, resource exhaustion
- Score: 62/100
- Reviewer: `Maxwell the 2nd`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: `Message::Info` carries a
    `Vec<(ConnectionId, PeerId, PeerConnectionMetric)>` with no semantic row
    bound.
  - `src/service/metrics_service.rs`: `MetricsService::recv` deserializes
    metrics service messages from unicast or broadcast payloads.
  - `src/service/metrics_service.rs`: `Message::Info(peer_metrics)` is
    forwarded directly as
    `MetricsServiceEvent::OnPeerConnectionMetric(from, peer_metrics)`.
- Impact: a peer can send a metrics `Info` frame containing a large number of
  metric rows, forcing deserialization and forwarding of an oversized batch.
  The outer peer frame codec may impose a byte limit, but there is no
  metrics-service row cap, validation, truncation, or rejection path. This is
  distinct from ISSUE-062's forged/unsolicited metrics content, ISSUE-078's
  unauthorized metrics disclosure via `Scan`, ISSUE-024's lower-level peer
  frame-size cap gap, and ISSUE-010's route/discovery sync vector growth.
- Evidence test:
  - `cargo test metrics_info_batches_must_be_bounded -- --nocapture`
  - Failure summary: a single metrics `Info` frame with 1,025 rows is delivered
    as one `OnPeerConnectionMetric` event, exceeding the test cap of 1,024
    rows.

### ISSUE-105: Visualization `Info` batches have no service-level row cap

- Category: high-load stability, resource exhaustion
- Score: 62/100
- Reviewer: `Galileo the 2nd`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: `Message::Info` carries a
    `Vec<(ConnectionId, PeerId, u16)>` with no semantic row bound.
  - `src/service/visualization_service.rs`: `VisualizationService::recv`
    deserializes visualization service messages from unicast or broadcast
    payloads.
  - `src/service/visualization_service.rs`: `Message::Info(neighbours)` is
    forwarded directly as `VisualizationServiceEvent::PeerJoined` or
    `PeerUpdated` with the full vector.
- Impact: a peer can send a visualization `Info` frame containing a large
  topology vector, forcing deserialization and forwarding of an oversized
  batch. Outer framing still has byte limits, but there is no
  visualization-service row cap, validation, truncation, or rejection path.
  This is distinct from ISSUE-061's forged/unsolicited topology content,
  ISSUE-079's topology disclosure through `Scan`, ISSUE-102's retained sender
  state growth, ISSUE-104's metrics row-cap issue, ISSUE-024's lower-level peer
  frame-size cap gap, and ISSUE-010's route/discovery sync vector growth.
- Evidence test:
  - `cargo test visualization_info_batches_must_be_bounded -- --nocapture`
  - Failure summary: a single visualization `Info` frame with 1,025 topology
    rows is delivered as one `PeerJoined` event, exceeding the test cap of
    1,024 rows.

### ISSUE-106: Pubsub heartbeat channel batches have no service-level row cap

- Category: high-load stability, resource exhaustion
- Score: 68/100
- Reviewer: `Boyle the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `PubsubMessage::Heartbeat` carries a
    `Vec<ChannelHeartbeat>` with no semantic channel-count bound.
  - `src/service/pubsub_service.rs`: `PubsubService::on_service` deserializes
    pubsub messages from unicast or broadcast payloads and iterates every
    heartbeat entry.
  - `src/service/pubsub_service.rs`: matching heartbeat entries can mutate
    `remote_publishers` or `remote_subscribers` and emit local join events.
- Impact: a peer can send one heartbeat frame containing a large channel vector
  and force the receiver to process every entry, mutate many channel states,
  and emit local events. Outer framing still has byte limits, but there is no
  pubsub heartbeat row cap, validation, truncation, or rejection path. This is
  distinct from ISSUE-026/080's stale heartbeat cleanup failures, ISSUE-100's
  per-channel membership growth, ISSUE-104/105's metrics and visualization row
  caps, ISSUE-024's lower-level peer frame-size cap gap, and ISSUE-010's
  route/discovery sync vector growth.
- Evidence test:
  - `cargo test pubsub_heartbeat_channel_batches_must_be_bounded -- --nocapture`
  - Failure summary: a single heartbeat with 1,025 channel entries updates
    1,025 channel states for one remote peer, exceeding the test cap of 1,024
    channels.

### ISSUE-107: Pubsub RPC method names have no service-level length cap

- Category: high-load stability, input validation
- Score: 56/100
- Reviewer: `Gibbs the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `PubsubMessage::GuestPublishRpc`,
    `PublishRpc`, `GuestFeedbackRpc`, and `FeedbackRpc` each carry a `String`
    method name with no semantic length bound.
  - `src/service/pubsub_service.rs`: `PubsubService::on_service` deserializes
    inbound pubsub RPC messages from unicast or broadcast payloads.
  - `src/service/pubsub_service.rs`: inbound RPC method names are cloned and
    forwarded unchanged to local `SubscriberEvent` or `PublisherEvent`
    handlers.
- Impact: a peer can send a pubsub RPC with an oversized method name and force
  the receiver to deserialize and deliver that method string to application
  handlers. Outer framing may still limit total message bytes, but the pubsub
  service has no method-name cap, truncation, validation, or rejection path.
  This is distinct from ISSUE-024's lower-level peer frame-size cap gap,
  ISSUE-039/048's pubsub membership authorization bypasses, ISSUE-043's pending
  RPC request retention, ISSUE-074/075's stale local requester issues,
  ISSUE-100's remote membership growth, and ISSUE-104/105/106's row and batch
  cap issues.
- Evidence test:
  - `cargo test pubsub_rpc_methods_must_be_bounded -- --nocapture`
  - Failure summary: a remote `PublishRpc` with a 1,025-byte method name is
    delivered unchanged to the local subscriber, exceeding the test cap of
    1,024 bytes.

### ISSUE-108: Empty pubsub channel state is retained after local handle teardown

- Category: high-load stability, lifecycle cleanup
- Score: 60/100
- Reviewer: `Euler the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `InternalMsg::PublisherDestroyed` uses
    `self.channels.entry(channel).or_default()` and removes only the local
    publisher id.
  - `src/service/pubsub_service.rs`: `InternalMsg::SubscriberDestroyed` uses
    `self.channels.entry(channel).or_default()` and removes only the local
    subscriber id.
  - `src/service/pubsub_service.rs`: neither branch prunes the channel entry
    after local publishers, local subscribers, remote publishers, and remote
    subscribers are all empty.
- Impact: local channel churn can retain one empty `PubsubChannelState` per
  channel id even after the final local handle is destroyed. Those empty
  entries consume memory and are still iterated by heartbeat generation, so
  repeated short-lived channels can grow retained state and periodic work. This
  is distinct from ISSUE-026/080's stale remote heartbeat cleanup failures,
  ISSUE-100's remote membership growth within a channel, ISSUE-106's heartbeat
  batch row cap, and ISSUE-107's RPC method length validation.
- Evidence test:
  - `cargo test empty_pubsub_channels_must_be_removed_after_last_local_handle_drops -- --nocapture`
  - Failure summary: after creating and destroying 1,025 distinct local
    subscriber channels, the service still retains 1,025 fully empty channel
    entries, exceeding the test cap of 1,024.

### ISSUE-109: Unsolicited alias `Found` messages create cache hints

- Category: correctness, security, cache poisoning
- Score: 66/100
- Reviewer: `Lorentz the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasServiceInternal::on_msg` handles
    `AliasMessage::Found(alias_id)` by inserting the sender into
    `self.cache[alias_id]`.
  - `src/service/alias_service.rs`: this insertion happens before checking
    whether a `FindRequest` exists for the alias.
  - `src/service/alias_service.rs`: with no pending lookup, the message still
    creates retained alias cache state for a future lookup.
- Impact: a peer can send `Found(alias)` without receiving a prior `Check` or
  `Scan` and seed this node's alias cache with an arbitrary alias-to-peer hint.
  Later lookups can spend work checking that poisoned hint and may be steered
  toward the unsolicited peer if it continues the protocol. This is distinct
  from ISSUE-090, which covers an unchecked peer completing an active
  cached-hint lookup, and from ISSUE-101, which covers unbounded per-alias
  hint-set growth.
- Evidence test:
  - `cargo test unsolicited_found_must_not_create_alias_cache_hint -- --nocapture`
  - Failure summary: a single unsolicited `Found(alias)` with no pending lookup
    creates a cache entry for that alias; expected unsolicited `Found` messages
    to be ignored.

### ISSUE-110: Replicated KV snapshots can terminally omit keys updated past max_version

- Category: correctness, bad-network/concurrent-write stability
- Score: 78/100
- Reviewer: `Bacon the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::snapshot` filters out current slots whose
    `slot.version > max_version`.
  - `src/service/replicate_kv_service/local_storage.rs`: `LocalStore` retains
    only the latest `Slot` per key in `slots`, not historical values for older
    snapshot versions.
  - `src/service/replicate_kv_service/local_storage.rs`: when all keys in the
    requested range are newer than `max_version`, the producer can return
    `SnapshotData { slots: vec![], next_key: None, ... }`.
- Impact: during paged full sync, the first page locks a snapshot version. If a
  key in a later requested range existed at that version but is updated before
  the continuation request, the producer no longer has the old value. It filters
  out the newer current slot and returns a terminal empty page, so the receiver
  can complete full sync while missing historical data. This is distinct from
  ISSUE-032's zero-size empty page with continuation, ISSUE-038's consumer
  acceptance of empty continuation pages, ISSUE-081's initial empty snapshot
  acceptance, ISSUE-034's future-version slot acceptance, and ISSUE-047's
  continuation version mismatch.
- Evidence test:
  - `cargo test snapshot_must_not_return_terminal_empty_page_for_newer_updated_keys -- --nocapture`
  - Failure summary: a bounded snapshot at `max_version = 2` over key `3`
    returns `slots: []` and `next_key: None` after key `3` was updated to a
    newer version, causing the test to fail because the page appears complete
    while omitting the historical key.

### ISSUE-111: Replicated KV consumer cancels FetchChanged repair on empty success

- Category: correctness, bad-network stability
- Score: 72/100
- Reviewer: `Kepler the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` handles `RpcRes::FetchChanged(Ok(changeds))`
    by inserting any versions newer than the current version.
  - `src/service/replicate_kv_service/remote_storage.rs`: after processing the
    response, it unconditionally sets `self.sending_req = None` before calling
    `apply_pendings(ctx)`.
  - `src/service/replicate_kv_service/remote_storage.rs`: if `changeds` is
    empty for an outstanding missing-version repair, no version advances, no
    pending entry is applied, no retry remains, and no full resync is scheduled.
- Impact: a malformed, stale, or malicious peer can answer an active
  missing-version repair with `FetchChanged(Ok(vec![]))`. The consumer clears
  the in-flight repair without advancing `version`, applying pendings, retrying,
  or starting full sync, so the remote replica can remain permanently behind.
  This is distinct from ISSUE-077 and ISSUE-099's producer-side empty-success
  bugs, ISSUE-086/087's unsolicited response handling, ISSUE-088/089's malformed
  non-empty response validation, and ISSUE-071's stale retry after broadcasts
  already filled a gap.
- Evidence test:
  - `cargo test working_state_must_not_cancel_repair_after_empty_fetch_changed_success -- --nocapture`
  - Failure summary: after requesting `FetchChanged { from: Version(1), count:
    1 }`, an empty successful response clears the in-flight repair; the next
    timeout tick emits no retry and does not transition to full resync.

### ISSUE-112: `connect()` accepts the node's own peer address

- Category: correctness, input validation, configuration stability
- Score: 70/100
- Reviewer: `Parfit the 2nd`, confirmed.
- Affected code:
  - `src/requester.rs`: `P2pNetworkRequester::connect` forwards any
    `PeerAddress` to the main loop.
  - `src/lib.rs`: `P2pNetwork::process_control` checks only
    `self.neighbours.has_peer(&addr.peer_id())` before dialing.
  - `src/lib.rs`: the connect path does not reject
    `addr.peer_id() == self.local_id`, so a node can dial its own advertised
    socket address.
- Impact: calling `connect()` with a `PeerAddress` whose peer id and socket
  address belong to the local node starts a self-dial and returns `Ok(())`
  instead of rejecting the input. The authenticated identity is the local node
  itself, which can create self connection/neighbour state, loopback route
  metrics, duplicate connection work, noisy events, and unstable behavior under
  poisoned discovery, bad configuration, or direct API misuse. This is distinct
  from ISSUE-005's learned local-peer advertisements, ISSUE-006's local route
  storage, ISSUE-013's local stream panic, ISSUE-016's early connect success
  before identity authentication, and ISSUE-103's configured self seed dial
  candidate.
- Evidence test:
  - `cargo test connect_to_own_peer_address_must_fail -- --nocapture`
  - Failure summary: `connect()` to the node's own advertised `PeerAddress`
    returns `Ok(())`; expected the self-connect target to be rejected.

### ISSUE-113: concurrent `connect()` calls to the same peer are not coalesced

- Category: correctness, high-load stability
- Score: 74/100
- Reviewer: `Dewey the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_control` suppresses a new
    `ControlCmd::Connect` only when `self.neighbours.has_peer(&addr.peer_id())`
    is true.
  - `src/neighbours.rs`: `NetworkNeighbours::has_peer` returns true only for
    neighbours whose `PeerConnection::is_connected()` is true.
  - `src/peer.rs`: outgoing attempts are inserted with `peer_id:
    Some(to_peer)` and `is_connected: false`, so additional connect commands
    for the same peer can start more QUIC connection attempts before the first
    one finishes.
- Impact: high-load callers, repeated seed ticks, or malicious API use can
  issue several `connect()` requests to the same peer while the first handshake
  is still pending. The node starts parallel outgoing QUIC connections, and
  each successful handshake can emit `PeerConnected`, install direct route
  state, register metrics, and trigger sync work. This can create duplicate
  direct connections, noisy route/path updates, duplicate connection events,
  and unnecessary work under churn. This is distinct from ISSUE-016's early
  success before identity authentication, ISSUE-057's stale connected event
  installing an unusable route, ISSUE-067's rebinding of an existing connection
  to a different peer, and ISSUE-112's self-connect acceptance. It is also
  distinct from the `stale_pending_outgoing_peer_does_not_suppress_reconnect`
  unit test: stale or failed unconnected attempts must not suppress reconnect
  forever, but live in-flight attempts should still be coalesced or otherwise
  bounded until they connect, fail, or time out.
- Evidence test:
  - `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`
  - Failure summary: four immediate `try_connect()` calls to the same peer
    produce four `PeerConnected` events; the test expected at most one
    connected event for that peer while duplicate attempts were in flight.

### ISSUE-114: Inbound duplicate connections from the same peer are not coalesced

- Category: correctness, high-load stability
- Score: 76/100
- Reviewer: `Fermat the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_incoming` accepts every inbound QUIC
    connection and inserts a `PeerConnection` before the remote peer id is
    known.
  - `src/peer.rs`: after the inbound handshake authenticates, `run_connection`
    sends `MainEvent::PeerConnected(conn_id, to_id, rtt_ms)` for the remote
    peer id.
  - `src/lib.rs`: `P2pNetwork::process_internal` handles every
    `MainEvent::PeerConnected` by calling `router.set_direct`,
    `neighbours.mark_connected`, and emitting a public `PeerConnected` event
    without checking whether that peer already has a live connection.
- Impact: a remote peer can open several inbound connections to the listener.
  After each authenticates as the same peer, the node emits another
  `PeerConnected` event and refreshes direct route/neighbour state instead of
  coalescing or bounding duplicates. This can create duplicate direct
  connections, repeated sync/metrics work, noisy route updates, and high-load
  instability. This is distinct from ISSUE-113's outbound/API-side duplicate
  `connect()` attempts, and from ISSUE-057/067's stale or mismatched internal
  `PeerConnected` event validation: this issue uses real live inbound
  connections with real authenticated peer ids.
- Evidence test:
  - `cargo test inbound_duplicate_connections_from_same_peer_must_be_coalesced -- --nocapture`
  - Failure summary: four inbound connections from peer `2` to node `1`
    produce four `PeerConnected` events on node `1`; the test expected at most
    one connected event for that already-connected peer.

### ISSUE-177: `connect()` reports success for a different address when peer id is already connected

- Category: correctness, API stability, topology stability
- Score: 38/100
- Reviewer: `Helmholtz the 3rd`, confirmed after `Aristotle the 3rd`
  discovery.
- Affected code:
  - `src/requester.rs`: `P2pNetworkRequester::connect` forwards a concrete
    `PeerAddress` to the main loop.
  - `src/lib.rs`: `P2pNetwork::process_control` returns `Ok(())` whenever
    `self.neighbours.has_peer(&addr.peer_id())` is true.
  - `src/neighbours.rs`: `NetworkNeighbours::has_peer` checks only
    authenticated peer id and has no record of the requested socket address.
- Impact: after a node is already authenticated to `PeerId(N)`,
  `connect(N@different_socket)` reports success without dialing or validating
  that endpoint. Callers can believe a specific address is reachable even when
  the live connection is to a different address, causing misleading state,
  address churn, or reconnect policy mistakes. This is distinct from ISSUE-016,
  which covers success before authentication; ISSUE-112, which covers
  self-connect input; and ISSUE-113/114, which cover duplicate connection work
  rather than the already-connected fast path.
- Minimal fix proposal: store the authenticated connection's remote network
  address in `PeerConnection` or `NetworkNeighbours`, and make the fast path
  return success only when both peer id and address match. If the peer id is
  connected at a different address, return an explicit
  `AlreadyConnectedDifferentAddress` error or enter a deliberate reconnect
  policy.
- Evidence test:
  - `cargo test connect_to_same_peer_id_at_different_address_must_not_report_success -- --nocapture`
  - Failure summary: after node2 has a live authenticated connection to node1,
    `process_control(ControlCmd::Connect(node1_peer_id@127.0.0.1:9, Some(tx)))`
    sends `Ok(())`; expected an error because the requested socket address does
    not match the existing connection.

### ISSUE-115: Local pubsub publish RPC answers are not bound to the subscriber handle

- Category: correctness, security, lifecycle stability
- Score: 72/100
- Reviewer: `Gauss the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service/subscriber.rs`:
    `SubscriberRequester::answer_publish_rpc` sends
    `InternalMsg::PublishRpcAnswer(rpc, source, data)` without including its
    `SubscriberLocalId`.
  - `src/service/pubsub_service.rs`: `InternalMsg::PublishRpcAnswer` carries
    only `RpcId`, `PeerSrc`, and data, so the service cannot verify which local
    subscriber handle is answering.
  - `src/service/pubsub_service.rs`: when `peer_src` is `PeerSrc::Local`,
    `PubsubService::on_internal` removes `publish_rpc_reqs[rpc_id]` and
    completes the pending publisher request by `RpcId` only.
- Impact: a requester cloned from a dropped subscriber can answer a local
  publish RPC handled by a different live subscriber when it knows the `RpcId`.
  `PubsubService` completes the pending publisher request by
  `RpcId`/`PeerSrc::Local` only, allowing stale or unauthorized local handles to
  inject RPC results. This is distinct from ISSUE-020's remote/inbound RPC
  answer binding gap, ISSUE-070/075's stale subscriber requesters initiating
  feedback traffic, and ISSUE-074's stale publisher requesters issuing publish
  RPCs.
- Evidence test:
  - `cargo test dropped_subscriber_requester_must_not_answer_publish_rpc -- --nocapture`
  - Failure summary: after a stale subscriber is dropped and a live subscriber
    receives a local `PublishRpc`, the stale requester answers with the same
    `RpcId` and completes the publisher's pending RPC with
    `stale-local-answer`.

### ISSUE-116: Local pubsub feedback RPC answers are not bound to the publisher handle

- Category: correctness, security, lifecycle stability
- Score: 82/100
- Reviewer: `Harvey the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service/publisher.rs`:
    `PublisherRequester::answer_feedback_rpc` sends
    `InternalMsg::FeedbackRpcAnswer(rpc, source, data)` without including its
    `PublisherLocalId`.
  - `src/service/pubsub_service.rs`: `InternalMsg::FeedbackRpcAnswer` carries
    only `RpcId`, `PeerSrc`, and data, so the service cannot verify which local
    publisher handle is answering.
  - `src/service/pubsub_service.rs`: when `peer_src` is `PeerSrc::Local`,
    `PubsubService::on_internal` removes `feedback_rpc_reqs[rpc_id]` and
    completes the pending subscriber request by `RpcId` only.
- Impact: a stale or otherwise unauthorized local publisher requester can
  complete a local feedback RPC if it learns the `RpcId`. The failing evidence
  uses a requester cloned from a dropped publisher to answer a feedback RPC
  handled by a different live publisher. `PubsubService` completes the pending
  subscriber request by `RpcId`/`PeerSrc::Local` only, allowing stale or
  unauthorized local handles to inject feedback RPC results. This is distinct
  from ISSUE-020's remote/inbound RPC answer binding gap, ISSUE-074's stale
  publisher requesters issuing publish RPCs, ISSUE-069's stale publisher
  ordinary publishes, and ISSUE-115's subscriber-side local publish RPC answer
  binding gap.
- Evidence test:
  - `cargo test dropped_publisher_requester_must_not_answer_feedback_rpc -- --nocapture`
  - Failure summary: after a stale publisher is dropped and a live publisher
    receives a local `FeedbackRpc`, the stale requester answers with the same
    `RpcId` and completes the subscriber's pending RPC with
    `stale-feedback-answer`.

### ISSUE-117: Idle inbound stream-connect handshakes are not admission bounded

- Category: stability, resource exhaustion under bad-network or malicious-peer
  conditions
- Score: 78/100
- Reviewer: `Plato the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerConnectionInternal::on_accept_bi`
    spawns one `accept_bi` task for every inbound bidirectional stream.
  - `src/peer/peer_internal.rs`: `accept_bi` immediately awaits
    `wait_object::<_, StreamConnectReq, MAX_CONTROL_STREAM_PKT>(&mut stream)`
    with no per-stream admission gate or stream-connect read timeout.
  - `src/quic.rs`: the transport is configured with
    `max_concurrent_bidi_streams(10_000)`, so the transport can accept many
    concurrent bidirectional streams before the application sees a valid
    stream-connect request.
- Impact: an authenticated connected peer can open many bidirectional streams
  and send no `StreamConnectReq`, leaving one idle `accept_bi` task/resource per
  stream. Under malicious peers, packet loss, or stalled clients, this can
  accumulate up to the large transport limit because the application has no
  lower admission cap or handshake timeout for idle stream-connect attempts.
  This is distinct from ISSUE-011/012, which cover successful `open_stream`
  responses when the local destination service cannot receive; ISSUE-056, which
  covers caller-side blocking on a full peer control queue before stream-open
  timeout; ISSUE-091, which covers an out-of-range service id panic after a
  stream request is sent; and ISSUE-024, which covers frame-size/resource caps
  on the peer message codec.
- Evidence test:
  - `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
  - Failure summary: a raw authenticated QUIC peer opens 17 inbound
    bidirectional streams and sends no `StreamConnectReq`; all 17 are
    transport-accepted, exceeding the test's admission threshold of 16 idle
    stream-connect attempts.

### ISSUE-118: Graceful shutdown waits one timeout per congested peer

- Category: stability, graceful shutdown reliability under high load or
  bad-network backpressure
- Score: 72/100
- Reviewer: `Bernoulli the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::shutdown_gracefully` collects
    `self.ctx.conns()` and iterates over the peer aliases sequentially.
  - `src/lib.rs`: each peer notification uses its own
    `timeout(Duration::from_secs(1), conn.send_wait(PeerMessage::PeerStopped(local_id)))`.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send_wait` first awaits
    admission to the bounded peer control queue, then waits for the connection
    task to write the message and answer.
- Impact: each congested peer control queue can consume its own one-second
  timeout before the next peer is attempted. With multiple congested or stalled
  peers, `shutdown_gracefully()` latency scales roughly with peer count and can
  delay `endpoint.close()` and process termination far beyond the apparent
  one-second notification timeout. This is distinct from ISSUE-049's broadcast
  fanout blocking, ISSUE-050's unicast blocking, ISSUE-056's stream-open queue
  blocking, ISSUE-051's received `PeerStopped` cleanup, and ISSUE-117's idle
  inbound stream-connect admission gap.
- Evidence test:
  - `cargo test shutdown_gracefully_must_not_wait_one_second_per_congested_peer -- --nocapture`
  - Failure summary: two synthetic peer aliases with full bounded control
    queues make `timeout(Duration::from_millis(1500), node.shutdown_gracefully())`
    expire before shutdown completes; expected graceful shutdown to use a global
    deadline or parallel/best-effort peer notification.

### ISSUE-119: Inbound unicast is silently dropped when the local service queue is full

- Category: correctness, reliability/stability under load, potential data loss
- Score: 76/100
- Reviewer: `Bohr the 2nd`, confirmed.
- Affected code:
  - `src/service.rs`: each `P2pService` uses a bounded
    `SERVICE_CHANNEL_SIZE` of 10 pending events.
  - `src/peer/peer_internal.rs`: inbound `PeerMessage::Unicast` for a local
    destination calls
    `service.try_send(P2pServiceEvent::Unicast(source, data)).print_on_err(...)`
    and does not retry, backpressure, close the stream, or signal delivery
    failure to the sender.
- Impact: when the destination local service is not draining quickly enough,
  the 11th ordinary inbound unicast is dropped after only a log message. The
  sender has already observed `send_unicast` success, so this creates silent
  data loss under local service backpressure. This is distinct from
  ISSUE-011/012, which cover stream delivery/open success under destination
  queue pressure; ISSUE-049/050, which cover outbound peer control queue
  blocking; ISSUE-053/091, which cover out-of-range service id handling;
  ISSUE-072/076, which cover stale requester lifecycle sends; and the sender
  identity binding issues. `PeerMessage::Broadcast` uses the same
  `try_send(...).print_on_err(...)` local-delivery pattern, but this issue's
  evidence is limited to ordinary unicast.
- Evidence test:
  - `cargo test inbound_unicast_must_not_drop_when_service_queue_is_full -- --nocapture`
  - Failure summary: after two connected nodes send 11 unicast messages to an
    unconsumed destination service, the receiver logs
    `send service msg got error no available capacity` and only 10 messages can
    be drained from the service queue; expected all 11 to be preserved or
    backpressured instead of silently dropped.
  - Additional reviewer `Turing the 2nd` accepted
    `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
    as closed-receiver evidence for the same ignored local-delivery failure
    pattern.
  - Additional failure summary: when the destination service receiver is closed,
    inbound local unicast delivery still only logs `try_send` failure, while
    the sender has already observed `send_unicast` success.
- Current audit status:
  - No-new cycle 22 reran
    `cargo test inbound_unicast_must_not_drop_when_service_queue_is_full -- --nocapture`;
    it now passes because the current local delivery path awaits bounded
    `service.send(...)` with a timeout instead of immediate `try_send`.
  - The additional closed-receiver evidence
    `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
    still fails at `src/tests/cross_nodes.rs:203:5`, so ISSUE-119 remains open
    for sender success reporting when local delivery cannot be completed.

### ISSUE-120: Inbound broadcast is silently dropped when the local service queue is full

- Category: correctness, reliability/stability under load, potential broadcast
  data loss
- Score: 74/100
- Reviewer: `Confucius the 2nd`, confirmed.
- Affected code:
  - `src/service.rs`: each `P2pService` uses a bounded
    `SERVICE_CHANNEL_SIZE` of 10 pending events.
  - `src/peer/peer_internal.rs`: inbound `PeerMessage::Broadcast` calls
    `service.try_send(P2pServiceEvent::Broadcast(source, data)).print_on_err(...)`
    after duplicate suppression, and does not retry, backpressure, disconnect,
    or signal local delivery failure.
- Impact: when the destination local service is not draining quickly enough,
  the 11th inbound broadcast is dropped after only a log message. The sender has
  already returned from `send_broadcast`, so this creates silent broadcast data
  loss under receiver-side local service backpressure. This is distinct from
  ISSUE-119 because broadcast uses a different inbound path with duplicate
  suppression and fanout semantics before local delivery. It is also distinct
  from ISSUE-011/012 stream queue pressure, ISSUE-049/050 outbound peer-control
  queue blocking, ISSUE-053/091 out-of-range service id panics, ISSUE-076 stale
  requester lifecycle broadcast, ISSUE-017 duplicate suppression behavior, and
  sender identity binding issues.
- Evidence test:
  - `cargo test inbound_broadcast_must_not_drop_when_service_queue_is_full -- --nocapture`
  - Failure summary: after two connected nodes send 11 broadcasts to an
    unconsumed destination service, the receiver logs
    `send service msg got error no available capacity` and only 10 broadcast
    events can be drained; expected all 11 to be preserved or backpressured
    instead of silently dropped.
- Current audit status:
  - No-new cycle 23 reran
    `cargo test inbound_broadcast_must_not_drop_when_service_queue_is_full -- --nocapture`;
    it now passes because current local broadcast delivery awaits bounded
    `service.send(...)` through `send_local_service_event` instead of immediate
    `try_send`.

### ISSUE-121: Short pubsub RPC timeouts wait for the global one-second sweep

- Category: correctness, bad-network stability, timeout reliability
- Score: 57/100
- Reviewer: `Leibniz the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `RPC_TICK_INTERVAL_MS` is fixed at 1,000
    ms.
  - `src/service/pubsub_service.rs`: `PubsubService::run_loop` only calls
    `on_rpc_tick` when that global interval ticks.
  - `src/service/pubsub_service.rs`: `on_rpc_tick` is the only path that checks
    `publish_rpc_reqs` and `feedback_rpc_reqs` against each request's
    caller-supplied timeout.
- Impact: callers that request short pubsub RPC deadlines can block far longer
  than the supplied timeout because pending RPCs are timed out only by the
  global one-second sweep. Under bad-network or no-answer conditions this
  violates API timeout expectations, ties up caller tasks and pending request
  state longer than requested, and can amplify load when many short-deadline
  RPCs are issued. This is distinct from ISSUE-043, which covers unbounded
  pending RPC retention; ISSUE-074/075, which cover stale requesters issuing
  RPCs after handle teardown; and ISSUE-115/116, which cover RPC answer
  authorization/binding.
- Evidence test:
  - `cargo test pubsub_publish_rpc_must_respect_short_timeout -- --nocapture`
  - Failure summary: a local `publish_rpc("slow", ..., Duration::from_millis(20))`
    delivers a `PublishRpc` event that is intentionally not answered, but the
    task still has not completed after a 200 ms outer timeout because the
    service waits for its one-second RPC sweep before returning timeout.

### ISSUE-122: Discovery records unbounded stop tombstones for unknown peers

- Category: security, high-load stability, discovery resource exhaustion
- Score: 68/100
- Reviewer: `Aristotle the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: inbound `PeerMessage::PeerStopped(peer_id)` is
    converted into `MainEvent::PeerStopped(self.conn_id, peer_id)`.
  - `src/lib.rs`: `P2pNetwork::process_internal` handles `PeerStopped` by
    calling `discovery.remove_remote(now_ms, &peer)`.
  - `src/discovery.rs`: `PeerDiscovery::remove_remote` inserts
    `self.stopped.insert(*peer, now_ms)` for every non-seed peer id before
    checking whether that peer was ever present in `remotes`.
  - `src/discovery.rs`: `clear_timeout` retains stop tombstones for
    `TIMEOUT_AFTER`.
- Impact: forged or repeated `PeerStopped` messages for arbitrary unknown
  non-seed peer ids can grow `PeerDiscovery::stopped` without a cardinality
  bound. Tombstones persist for `TIMEOUT_AFTER` and are inserted even when no
  remote discovery record existed, so a bad peer or noisy network can force
  avoidable memory growth and extra discovery filtering work. This is amplified
  by the existing forged third-party stop path because the sender does not need
  to prove ownership of the stopped peer id. This is distinct from ISSUE-001,
  which covers forged route/discovery removal of live peers; ISSUE-009, which
  covers discovery timestamp overflow/immortal-peer arithmetic; and ISSUE-093,
  which covers tombstones suppressing valid restart advertisements.
- Evidence test:
  - `cargo test graceful_stop_tombstones_must_be_bounded_for_unknown_peers -- --nocapture`
  - Failure summary: calling `PeerDiscovery::remove_remote` for 1,025 distinct
    unknown non-seed peer ids leaves 1,025 entries in `discovery.stopped`,
    exceeding the bounded-tombstone assertion.

### ISSUE-123: Local pubsub subscriber event queues are unbounded

- Category: high-load stability, resource exhaustion, pubsub reliability
- Score: 72/100
- Reviewer: `Heisenberg the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service/subscriber.rs`: `Subscriber::build` creates an
    unbounded `mpsc::unbounded_channel` for each local subscriber event stream.
  - `src/service/pubsub_service.rs`: remote and local pubsub delivery paths send
    `SubscriberEvent` values into every matching local subscriber queue with
    `sub_tx.send(...)` and no cardinality cap, backpressure, or drop/close
    policy.
- Impact: an undrained or slow local subscriber can accumulate one queued event
  per incoming publish, publish RPC, guest publish, heartbeat membership change,
  or lifecycle notification. Under high publish rate, bad-network replay, or a
  stalled application consumer, this can grow memory without bound inside the
  pubsub service even after the bounded P2P service queue has already accepted
  and processed the inbound message. This is distinct from ISSUE-043, which
  covers pending RPC request maps; ISSUE-100, which covers remote membership
  sets; ISSUE-106, which covers heartbeat batch input size; ISSUE-119/120, which
  cover bounded service ingress queue drops before pubsub handling; and the
  stale requester lifecycle issues.
- Evidence test:
  - `cargo test local_subscriber_event_backlog_must_be_bounded -- --nocapture`
  - Failure summary: sending 1,025 remote `Publish` messages to one registered
    but undrained local subscriber leaves `sub_rx.len() == 1025`; expected the
    local subscriber event backlog to be bounded, backpressured, or otherwise
    controlled.

### ISSUE-124: Local pubsub publisher event queues are unbounded

- Category: high-load stability, resource exhaustion, pubsub reliability
- Score: 72/100
- Reviewer: `Cicero the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service/publisher.rs`: `Publisher::build` creates an
    unbounded `mpsc::unbounded_channel` for each local publisher event stream.
  - `src/service/pubsub_service.rs`: feedback delivery paths send
    `PublisherEvent` values into every matching local publisher queue with
    `pub_tx.send(...)` and no cardinality cap, backpressure, or drop/close
    policy.
- Impact: an undrained or slow local publisher can accumulate one queued event
  per incoming feedback, feedback RPC, guest feedback, heartbeat membership
  change, or lifecycle notification. Under high feedback rate, bad-network
  replay, or a stalled application consumer, this can grow memory without bound
  inside the pubsub service even after the bounded P2P service queue has already
  accepted and processed the inbound message. This is symmetric with but
  distinct from ISSUE-123, which covers subscriber event queues fed by
  publish-side traffic; this issue covers publisher event queues fed by
  feedback-side traffic. It is also distinct from ISSUE-043, ISSUE-100,
  ISSUE-106, ISSUE-119/120, and the stale requester lifecycle issues for the
  same reasons.
- Evidence test:
  - `cargo test local_publisher_event_backlog_must_be_bounded -- --nocapture`
  - Failure summary: sending 1,025 remote `Feedback` messages to one registered
    but undrained local publisher leaves `pub_rx.len() == 1025`; expected the
    local publisher event backlog to be bounded, backpressured, or otherwise
    controlled.

### ISSUE-125: Requester connect commands can accumulate without bound

- Category: high-load stability, resource exhaustion, API backpressure
- Score: 70/100
- Reviewer: `Feynman the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::new` creates `control_tx/control_rx` with
    `tokio::sync::mpsc::unbounded_channel`.
  - `src/requester.rs`: `P2pNetworkRequester::try_connect` sends
    `ControlCmd::Connect(addr, None)` into that unbounded channel and returns
    immediately, with no admission cap, coalescing, or backpressure.
  - `src/requester.rs`: `P2pNetworkRequester::connect` uses the same unbounded
    channel before awaiting the per-request answer.
- Impact: any local caller holding a requester can enqueue connect work faster
  than the network main loop drains `control_rx`. Under high load, repeated seed
  churn, or misuse of the public API, pending connect commands can grow memory
  without bound before any dial attempt is processed. This is distinct from
  ISSUE-028, which covers panics after the receiver is closed; ISSUE-113, which
  covers duplicate in-flight connections after commands are processed; and
  ISSUE-114, which covers inbound duplicate connections. It is also distinct
  from peer/service/pubsub queue issues because this is the public
  requester-to-main-loop control queue.
- Evidence test:
  - `cargo test requester_connect_backlog_must_be_bounded -- --nocapture`
  - Failure summary: 1,025 `try_connect` calls for distinct target peers remain
    queued in `node.control_rx`, exceeding the bounded-backlog assertion.

### ISSUE-126: Pubsub internal control messages can accumulate without bound

- Category: high-load stability, resource exhaustion, API backpressure
- Score: 70/100
- Reviewer: `Banach the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `PubsubService::new` creates
    `internal_tx/internal_rx` with `tokio::sync::mpsc::unbounded_channel`.
  - `src/service/pubsub_service.rs`: `PubsubServiceRequester::publisher` and
    `subscriber` enqueue handle-registration messages through that unbounded
    channel and return handles without admission control.
  - `src/service/pubsub_service/publisher.rs` and
    `src/service/pubsub_service/subscriber.rs`: handle actions and drops use the
    same unbounded control sender for additional `InternalMsg` variants.
- Impact: a local caller holding a pubsub requester can enqueue service-control
  work faster than `PubsubService::run_loop` drains `internal_rx`. Under high
  handle churn or a stalled service loop, pending pubsub control messages can
  grow memory without bound before the service processes them. This is distinct
  from ISSUE-058, which covers dead-on-arrival handles after service drop;
  ISSUE-108, which covers retained empty channel state after processed teardown;
  ISSUE-123/124, which cover per-handle event output queues; ISSUE-125, which
  covers the network requester's control queue; and the pending RPC issues.
- Evidence test:
  - `cargo test pubsub_internal_control_backlog_must_be_bounded -- --nocapture`
  - Failure summary: 1,025 `PubsubServiceRequester::publisher(...)` calls leave
    1,025 pending messages in `service.internal_rx`, exceeding the bounded
    control-backlog assertion.

### ISSUE-127: Alias internal control messages can accumulate without bound

- Category: high-load stability, resource exhaustion, API backpressure
- Score: 70/100
- Reviewer: `Schrodinger the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasService::new` creates `tx/rx` with
    `tokio::sync::mpsc::unbounded_channel`.
  - `src/service/alias_service.rs`: `AliasServiceRequester::register`, `find`,
    and `shutdown` enqueue `AliasControl` messages through that unbounded
    channel without admission control or backpressure while the service is
    alive.
  - `src/service/alias_service.rs`: `AliasGuard::drop` uses the same unbounded
    sender for unregister control messages.
- Impact: a local caller holding an alias requester can enqueue alias
  service-control work faster than `AliasService::run_loop` drains `rx`. Under
  high alias registration churn or a stalled service loop, pending alias
  control messages can grow memory without bound before the service processes
  them. This is distinct from ISSUE-029, which covers stale requester panics
  after service drop; ISSUE-035/041, which cover processed alias lookup state;
  ISSUE-101, which covers cached peer hints; and ISSUE-125/126, which cover the
  same unbounded-channel class in different components.
- Evidence test:
  - `cargo test alias_internal_control_backlog_must_be_bounded -- --nocapture`
  - Failure summary: 1,025 `AliasServiceRequester::register(...)` calls leave
    1,025 pending messages in `service.rx`, exceeding the bounded
    control-backlog assertion.

### ISSUE-128: Metrics recv panics when the base service channel closes

- Category: correctness, shutdown stability, API stability
- Score: 56/100
- Reviewer: `Meitner the 2nd`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: `MetricsService::recv` returns
    `anyhow::Result<MetricsServiceEvent>`.
  - `src/service/metrics_service.rs`: the `event = self.service.recv()` branch
    calls `event.expect("should work")` when the underlying `P2pService`
    receiver returns `None`.
- Impact: if the base service channel closes during shutdown or teardown,
  `MetricsService::recv()` unwinds instead of returning `Err`. This breaks the
  public result-returning API and can turn an orderly service shutdown into a
  task panic. This is distinct from ISSUE-028/029 stale requester send panics,
  ISSUE-040 zero interval constructor panics, ISSUE-058 pubsub dead-on-arrival
  handles, ISSUE-096 replicated-KV serialization panics, and the existing
  metrics forged/disclosure/batch issues.
- Evidence test:
  - `cargo test metrics_recv_after_base_service_close_must_not_panic -- --nocapture`
  - Failure summary: after dropping the underlying `P2pService` sender,
    `MetricsService::recv()` panics at `src/service/metrics_service.rs` with
    `should work`; expected `Ok(Err(_))` from the public `Result` API.

### ISSUE-129: Visualization recv panics when the base service channel closes

- Category: correctness, shutdown stability, API stability
- Score: 56/100
- Reviewer: `McClintock the 2nd`, confirmed.
- Affected code:
  - `src/service/visualization_service.rs`: `VisualizationService::recv`
    returns `anyhow::Result<VisualizationServiceEvent>`.
  - `src/service/visualization_service.rs`: the
    `event = self.service.recv()` branch calls `event.expect("should work")`
    when the underlying `P2pService` receiver returns `None`.
- Impact: if the base service channel closes during shutdown or teardown,
  `VisualizationService::recv()` unwinds instead of returning `Err`. This
  breaks the public result-returning API and can turn an orderly visualization
  service shutdown into a task panic. This is a close sibling of ISSUE-128, but
  affects a separate public service type and shutdown path. It is distinct from
  ISSUE-040 zero interval constructor panics, ISSUE-061/079/102/105
  visualization forged/disclosure/resource issues, and stale requester send
  panics.
- Evidence test:
  - `cargo test visualization_recv_after_base_service_close_must_not_panic -- --nocapture`
  - Failure summary: after dropping the underlying `P2pService` sender,
    `VisualizationService::recv()` panics at
    `src/service/visualization_service.rs` with `should work`; expected
    `Ok(Err(_))` from the public `Result` API.

### ISSUE-130: Alias run loop panics when the base service channel closes

- Category: correctness, shutdown stability, API stability
- Score: 58/100
- Reviewer: `Ampere the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasService::run_loop` returns
    `anyhow::Result<()>`.
  - `src/service/alias_service.rs`: the `event = self.service.recv()` branch
    calls `event.expect("service channel should work")` when the underlying
    `P2pService` receiver returns `None`.
- Impact: if the base service channel closes during teardown, the alias service
  task unwinds instead of returning `Err` from its result-returning run loop.
  This turns a normal shutdown condition into a task panic and can make graceful
  service shutdown noisy or unstable. This is distinct from ISSUE-127, which
  covers alias control backlog, and from stale requester send panics, which
  involve closed control senders. It is a close sibling of ISSUE-128/129 but
  affects the alias service's run-loop lifecycle rather than metrics or
  visualization `recv()` APIs.
- Evidence test:
  - `cargo test alias_run_loop_after_base_service_close_must_not_panic -- --nocapture`
  - Failure summary: after dropping the underlying `P2pService` sender,
    `AliasService::run_loop()` panics at `src/service/alias_service.rs` with
    `service channel should work`; expected `Ok(Err(_))` from the public
    `Result` API.

### ISSUE-131: Replicated KV full-sync snapshot pages are unbounded

- Category: high-load stability, bad-network stability, resource exhaustion
- Score: 62/100
- Reviewer: `Erdos the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` handles
    `RpcRes::FetchSnapshot(Some(snapshot), version)`.
  - `src/service/replicate_kv_service/remote_storage.rs`: every
    `snapshot.slots` entry is emitted as `KvEvent::Set` and inserted into
    `ctx.slots` without an application-level per-page slot cap.
- Impact: a malformed or adversarial peer can send a single full-sync snapshot
  page with many small slots and force the receiver to allocate replicated
  state and enqueue one local event per slot. Transport frame limits reduce the
  absolute packet size but do not enforce the replicated-KV page budget or
  event count, so this remains a memory/event-queue pressure issue under bad
  network input. This is distinct from ISSUE-045, which covers unbounded remote
  store count, and from ISSUE-081 through ISSUE-085, which cover invalid
  snapshot contents such as empty, out-of-range, unsorted, or duplicate pages.
- Evidence test:
  - `cargo test full_sync_snapshot_pages_must_be_bounded -- --nocapture`
  - Failure summary: one `FetchSnapshot` response containing 1,025 slots is
    accepted into `ctx.slots`; expected full-sync snapshot pages to be capped at
    or below 1,024 slots.

### ISSUE-132: Alias run loop panics when the internal control channel closes

- Category: correctness, shutdown stability, API stability
- Score: 57/100
- Reviewer: `Halley the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasService::run_loop` returns
    `anyhow::Result<()>`.
  - `src/service/alias_service.rs`: the `control = self.rx.recv()` branch calls
    `control.expect("service channel should work")` when the alias control
    receiver returns `None`.
- Impact: if the alias service's internal control channel closes during
  teardown, `run_loop()` unwinds instead of returning `Err`. This turns a normal
  channel-close condition into a task panic and makes graceful alias service
  shutdown noisy or unstable. This is distinct from ISSUE-127, which covers
  alias control backlog while the channel is live; ISSUE-130, which covers the
  base `P2pService` receive path closing; and ISSUE-029, which covers stale
  requester/guard send panics after service drop.
- Evidence test:
  - `cargo test alias_run_loop_after_control_channel_close_must_not_panic -- --nocapture`
  - Failure summary: after closing the alias control sender paired with
    `service.rx`, `AliasService::run_loop()` panics at
    `src/service/alias_service.rs` with `service channel should work`; expected
    `Ok(Err(_))` from the public `Result` API.

### ISSUE-133: PeerStopped blocks the peer task when the main event queue is full

- Category: high-load stability, shutdown stability, head-of-line blocking
- Score: 71/100
- Reviewer: `Pasteur the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerConnectionInternal::on_msg` handles
    `PeerMessage::PeerStopped(peer_id)`.
  - `src/peer/peer_internal.rs`: after forwarding the stop signal to other
    peer aliases, the handler awaits
    `self.main_tx.send(MainEvent::PeerStopped(self.conn_id, peer_id)).await`
    on the bounded main event queue.
- Impact: when the main loop is saturated, a peer connection task can park
  inside `PeerStopped` handling and stop processing later messages from that
  connection. This turns graceful stop notification into head-of-line blocking
  under load or slow main-loop processing, delaying unrelated unicast/broadcast
  traffic and connection cleanup. This is distinct from ISSUE-001/004/051,
  which cover forged or legitimate stop semantics, and from ISSUE-118, which
  covers caller-side graceful shutdown latency while notifying congested peers.
- Evidence test:
  - `cargo test peer_stopped_must_not_block_connection_task_on_full_main_queue -- --nocapture`
  - Failure summary: after filling node2's bounded main event queue and sending
    `PeerStopped`, a later unicast over the same peer connection times out with
    `Err(Elapsed(()))`; expected the connection task to keep processing traffic
    instead of blocking on `main_tx.send`.

### ISSUE-134: Unauthenticated inbound QUIC connections are not admission bounded

- Category: security, high-load stability, resource exhaustion
- Score: 72/100
- Reviewer: `Rawls the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_incoming` accepts every inbound
    `Incoming`, creates `PeerConnection::new_incoming`, and inserts it into
    `neighbours` before the P2P handshake authenticates the peer.
  - `src/peer.rs`: `PeerConnection::new_incoming` awaits `incoming.await` and
    then `connection.accept_bi().await` without a node-level cap for pending
    unauthenticated connections or a P2P control-stream handshake timeout.
  - `src/quic.rs`: transport limits allow many concurrent connections/streams
    and rely only on QUIC idle timeout rather than P2P admission control.
- Impact: raw QUIC clients can connect to a node, never open or send the P2P
  main control stream, and remain as pending unauthenticated connection tasks.
  Under load or hostile traffic this can consume connection/task resources
  before authentication and before normal peer-level controls apply. This is
  distinct from ISSUE-117, which covers idle bidirectional stream-connect
  handshakes after an authenticated peer connection already exists.
- Evidence test:
  - `cargo test unauthenticated_inbound_connections_must_be_admission_bounded -- --nocapture`
  - Failure summary: 17 raw QUIC clients connect to a node and never open the
    P2P main control stream; all are accepted, exceeding the test's admission
    threshold of 16 pending unauthenticated connections.

### ISSUE-135: Stale PeerConnectError removes a live neighbour

- Category: correctness, async race stability, connection lifecycle
- Score: 62/100
- Reviewer: `Pauli the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerConnectError(conn, peer, err)`.
  - `src/lib.rs`: the handler unconditionally calls
    `self.neighbours.remove(&conn)` without checking whether `conn` is still a
    pending outgoing attempt or already authenticated and connected.
- Impact: a stale connect-error event can remove a live neighbour entry for the
  same connection id after authentication. The router and peer task can still
  contain direct route or alias state, while neighbour iteration no longer
  treats the peer as connected, corrupting connection lifecycle state and
  suppressing later neighbour-driven maintenance. This is distinct from
  ISSUE-057/063/064/065/066/067/068 because those cover other internal event
  types, not `PeerConnectError` removing an authenticated live neighbour.
- Evidence test:
  - `cargo test stale_peer_connect_error_must_not_remove_live_neighbour -- --nocapture`
  - Failure summary: after node2 connects to node1, injecting
    `MainEvent::PeerConnectError(live_conn, Some(node1), ...)` makes
    `node2.neighbours.has_peer(node1)` false; expected stale connect errors not
    to remove an already-live neighbour.

### ISSUE-136: PeerDisconnected can block alias cleanup when the main event queue is full

- Category: high-load stability, bad-network stability, connection lifecycle
- Score: 67/100
- Reviewer: `Dalton the 2nd`, confirmed.
- Affected code:
  - `src/peer.rs`: `run_connection` awaits
    `main_tx.send(MainEvent::PeerDisconnected(conn_id, to_id))` after the peer
    run loop ends.
  - `src/peer.rs`: `ctx.unregister_conn(&conn_id)` and metric cleanup run only
    after that bounded main-queue send completes.
- Impact: if a peer disconnects while the main event queue is saturated, the
  peer task can park before unregistering its alias and metrics. Local send,
  stream, graceful-shutdown, or maintenance paths can still observe a stale
  `ctx.conn(conn_id)` after the transport is gone, extending bad-network churn
  into resource and lifecycle corruption. This is distinct from ISSUE-133,
  which covers blocking while processing inbound `PeerStopped`; from
  ISSUE-065/066, which cover stale or mismatched disconnect events in the main
  loop; and from ISSUE-128/129/130/132, which cover service shutdown panics.
- Evidence test:
  - `cargo test peer_disconnected_must_not_block_alias_cleanup_on_full_main_queue -- --nocapture`
  - Failure summary: after node2 connects to node1, the test fills node2's
    bounded main event queue and closes node1; `node2.ctx.conn(live_conn)`
    remains present after disconnect cleanup should have unregistered the alias.

### ISSUE-137: Pending alias finds can resolve remote after the alias becomes local

- Category: correctness, alias lifecycle stability, bad-network race
- Score: 68/100
- Reviewer: `Volta the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasControl::Find` for a missing alias
    inserts a `find_reqs` entry in scan state.
  - `src/service/alias_service.rs`: `AliasControl::Register` increments local
    alias state and broadcasts `NotifySet`, but does not complete or remove an
    existing pending find for the same alias.
  - `src/service/alias_service.rs`: a later `AliasMessage::Found(alias)` then
    removes the pending request and answers `AliasFoundLocation::Scan(from)`
    even though the alias is now local.
- Impact: local alias registration can lose a race against an older pending
  scan. Callers waiting on `find(alias)` may be told to use a remote peer for an
  alias that is already local, causing wrong stream placement, avoidable remote
  traffic, and inconsistent alias ownership under churn or delayed network
  replies. This is distinct from ISSUE-090 and ISSUE-109, which cover unchecked
  or unsolicited remote `Found` trust and cache poisoning; this issue is the
  failure to reconcile pending find state when local ownership appears.
- Evidence test:
  - `cargo test pending_find_must_prefer_late_local_registration_over_remote_found -- --nocapture`
  - Failure summary: after a missing-alias find starts a scan, local
    registration of that alias does not complete the pending find; a later
    remote `Found` returns `Some(Scan(PeerId(2)))` instead of `Some(Local)`.

### ISSUE-138: Replicated KV snapshot producer declares live version for bounded continuation pages

- Category: correctness, replication consistency, bad-network stability
- Score: 70/100
- Reviewer: `Poincare the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::on_rpc_req` handles
    `RpcReq::FetchSnapshot { from, to, max_version }`.
  - `src/service/replicate_kv_service/local_storage.rs`: the response is built
    as `RpcRes::FetchSnapshot(self.snapshot(from, to, max_version),
    self.version)`, so the data is filtered by `max_version` but the declared
    response version is the producer's newer live version.
- Impact: during paged full sync, a consumer can lock snapshot version `2` and
  request a continuation with `max_version = Some(Version(2))` after the
  producer has advanced to version `3`. The producer returns version-2 data but
  labels it as version `3`, making an honest continuation page violate the
  snapshot-version contract. A fixed consumer would reject the page; an unfixed
  consumer can mix version labels and data. This is distinct from ISSUE-047,
  which covers consumer-side acceptance of mismatched continuation responses,
  and from ISSUE-110, which covers omitted historical keys after filtering by
  `max_version`.
- Evidence test:
  - `cargo test continuation_snapshot_response_must_preserve_requested_max_version -- --nocapture`
  - Failure summary: a continuation `FetchSnapshot` request bounded to
    `max_version = Version(2)` returns snapshot data for key `2` at
    `Version(2)` but declares the response version as `Version(3)`; expected
    the declared version to remain `Version(2)`.

### ISSUE-139: Early PeerConnectError reporting can panic after main loop shutdown

- Category: shutdown stability, bad-network stability, connection lifecycle
- Score: 63/100
- Reviewer: `Mill the 2nd`, confirmed. Additional churn fuzz evidence
  confirmed by `Huygens the 3rd`.
- Affected code:
  - `src/peer.rs`: `PeerConnection::new_incoming` reports early
    `incoming.await` and `connection.accept_bi()` failures with
    `main_tx.send(MainEvent::PeerConnectError(...)).await.expect("should send to main")`.
  - `src/peer.rs`: `PeerConnection::new_connecting` uses the same unchecked
    send pattern for early `connecting.await` and `connection.open_bi()`
    failures.
- Impact: if the main `P2pNetwork` loop is dropped or shutting down while an
  early QUIC connect/open failure is reported, the background peer task panics
  instead of exiting quietly. Bad-network connection failures during teardown
  can therefore surface as task panics and noisy instability. This is distinct
  from ISSUE-028, which covers public requester panics after network drop;
  ISSUE-128/129/130/132, which cover service shutdown panics; and ISSUE-135,
  which covers stale `PeerConnectError` state mutation in the main loop.
- Evidence test:
  - `cargo test incoming_connect_error_after_main_drop_must_not_panic_task`
  - Failure summary: a raw client connects and closes before opening the P2P
    control stream while `main_rx` is dropped; the spawned incoming peer task
    panics at `src/peer.rs:62` with `should send to main: SendError`, and the
    panic hook records the background panic.
  - Additional sanitized churn fuzz evidence:
    `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=120 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  - Sanitized churn fuzz failure summary: with out-of-range service ids and
    forged `PeerStopped` messages disabled, ordinary connect/stop/restart
    churn panics a background task at `src/peer.rs:106` while the outbound
    `new_connecting` path reports an early `PeerConnectError` to a closed main
    loop.

### ISSUE-140: Ignored replicated-KV RPC responses refresh stale remote activity

- Category: stability, resource cleanup, bad-network resilience
- Score: 55/100
- Reviewer: `Helmholtz the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `RemoteStore::on_rpc_res` updates `last_active` before dispatching the
    response to the current state.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` ignores `RpcRes::FetchSnapshot` responses, so
    an unsolicited or stale snapshot response can refresh activity without any
    accepted state transition or output.
  - `src/service/replicate_kv_service.rs`: remote cleanup depends on
    `remote.last_active().elapsed() < REMOTE_TIMEOUT_MS`.
- Impact: stale remote replicated-KV stores can be kept alive by ignored RPC
  response traffic. Under bad network replay, delayed packets, or adversarial
  unsolicited responses, timeout cleanup can be prevented even though the
  remote made no valid progress. This is distinct from ISSUE-045, which covers
  unbounded remote-store creation; ISSUE-086/087, which cover unsolicited
  `FetchChanged` responses mutating state or forcing resync; and ISSUE-131/138,
  which cover snapshot page size and version-contract issues.
- Evidence test:
  - `cargo test ignored_rpc_response_must_not_refresh_remote_activity`
  - Failure summary: a stale `WorkingState` remote receives an ignored
    `RpcRes::FetchSnapshot(None, Version(99))`; no output is produced, but
    `last_active` changes from the stale instant to `Instant::now()`, preventing
    timeout cleanup from recognizing the remote as inactive.

### ISSUE-141: Replicated KV drops remaining repair range after partial FetchChanged success

- Category: correctness, replication consistency, bad-network stability
- Score: 64/100
- Reviewer: `Darwin the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`:
    `LocalStore::changeds_from_to` caps a `FetchChanged` response to
    `count.min(self.compose_max_pkts as u64)`, so non-empty partial repair
    responses are valid producer behavior.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` requests the whole missing version range when
    it receives a higher `BroadcastEvent::Version`.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` applies returned `FetchChanged` entries and then
    unconditionally clears `self.sending_req`, without continuing the remaining
    requested range.
- Impact: if a receiver asks for versions `1..=5` and the producer returns a
  valid capped prefix such as versions `1` and `2`, the receiver advances to
  version `2`, clears the in-flight repair, and emits no follow-up request for
  versions `3..=5`. The replica can remain stale until another version
  broadcast happens to re-trigger repair. This is distinct from ISSUE-111,
  which covers empty successful responses with no progress; ISSUE-071, which
  covers stale retries after broadcasts already fill the gap; ISSUE-077/099,
  which cover producer-side empty success; and ISSUE-089, which covers applying
  versions beyond the requested range.
- Evidence test:
  - `cargo test working_state_must_continue_repair_after_partial_fetch_changed_success`
  - Failure summary: after `FetchChanged { from: Version(1), count: 5 }`
    receives a valid partial success containing versions `1` and `2`, the test
    expects a follow-up `FetchChanged { from: Version(3), count: 3 }`; current
    code emits `None`.

### ISSUE-142: New local pubsub handles miss already-known remote members

- Category: correctness, pubsub membership stability, bad-network lifecycle
- Score: 62/100
- Reviewer: `Aquinas the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PublisherJoined` and heartbeat
    repair add `remote_publishers` and notify only currently-existing local
    subscribers.
  - `src/service/pubsub_service.rs`: inbound `SubscriberJoined` and heartbeat
    repair add `remote_subscribers` and notify only currently-existing local
    publishers.
  - `src/service/pubsub_service.rs`: `InternalMsg::PublisherCreated` registers
    the new local publisher but only reports existing local subscribers, not
    already-known `remote_subscribers`.
  - `src/service/pubsub_service.rs`: `InternalMsg::SubscriberCreated` registers
    the new local subscriber but only reports existing local publishers, not
    already-known `remote_publishers`.
- Impact: after remote membership is learned and local handles churn or are
  created later, the new local publisher/subscriber can start with an incomplete
  membership view. The service may still route publish/feedback using the stored
  remote sets, but the handle's event stream omits expected
  `PeerJoined(Remote(...))`, causing applications to make stale presence
  decisions or miss remote availability until another remote join or heartbeat
  transition arrives. This is distinct from ISSUE-026/080, which cover stale
  remote membership removal; ISSUE-039/048, which cover membership
  authorization bypass; ISSUE-100, which covers unbounded remote membership
  sets; and ISSUE-108, which covers fully empty channel retention.
- Evidence test:
  - `cargo test new_local_pubsub_handles_must_observe_existing_remote_members`
  - Failure summary: a channel seeded with `remote_subscribers = {PeerId(3)}`
    and `remote_publishers = {PeerId(2)}` creates new local publisher/subscriber
    handles; the publisher event backlog contains only `PeerJoined(Local)` and
    no `PeerJoined(Remote(PeerId(3)))`, and the subscriber likewise misses the
    existing remote publisher.

### ISSUE-143: Replicated KV full sync accepts stale terminal snapshot responses

- Category: correctness, replication consistency, bad-network stability
- Score: 76/100
- Reviewer: `James the 2nd`, confirmed.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::on_rpc_res` accepts every
    `RpcRes::FetchSnapshot(Some(snapshot), version)` while full sync is active.
  - `src/service/replicate_kv_service/remote_storage.rs`: the first snapshot
    page locks `self.version` and `self.biggest_key`, and a partial page records
    an outstanding continuation request in `self.sending_req`.
  - `src/service/replicate_kv_service/remote_storage.rs`: any later
    `Some(snapshot)` with `next_key == None` transitions to `WorkingState`
    without checking that the response matches the outstanding continuation
    range.
- Impact: after a partial snapshot page requests
  `FetchSnapshot { from: Some(2), to: Some(3), max_version: Some(Version(3)) }`,
  a stale or reordered terminal response that looks like an older initial-page
  response can complete full sync. The replica can move to `WorkingState` at the
  locked version while silently missing keys that were still pending in the
  continuation range. This is distinct from ISSUE-047, which covers mismatched
  continuation response versions; ISSUE-059, which covers `FetchSnapshot(None)`
  as fake completion; ISSUE-083, which covers slots before the requested lower
  bound; ISSUE-110/138, which cover producer-side snapshot omissions/version
  labels; ISSUE-140, which covers ignored responses refreshing activity; and
  ISSUE-141, which covers partial `FetchChanged` repair.
- Evidence test:
  - `cargo test full_sync_must_reject_stale_terminal_snapshot_after_continuation_request`
  - Failure summary: after the first page for key `1` requests continuation
    `from=2..=3` at `Version(3)`, a stale terminal `Some(snapshot)` for only
    key `1` sets `ctx.next_state` to `Working(Version(3))`; expected full sync
    to reject or ignore the stale terminal page and keep the continuation
    outstanding.

### ISSUE-144: Peer alias leaks if main loop closes before PeerConnected delivery

- Category: shutdown stability, bad-network stability, connection lifecycle
- Score: 66/100
- Reviewer: `Tesla the 2nd`, confirmed.
- Affected code:
  - `src/peer.rs`: after authentication, `run_connection` creates a
    `PeerConnectionAlias` and `PeerConnectionInternal`, then registers the alias
    with `ctx.register_conn`.
  - `src/peer.rs`: if
    `main_tx.send(MainEvent::PeerConnected(conn_id, to_id, rtt_ms)).await`
    fails because the main loop has closed, `run_connection` returns `Ok(())`
    immediately.
  - `src/peer.rs`: normal cleanup via `ctx.unregister_conn(&conn_id)` and
    metric reset happens only after `internal.run_loop()` and
    `PeerDisconnected`, so the failed `PeerConnected` branch skips it.
- Impact: during shutdown or teardown, a peer task can successfully
  authenticate, register its alias, fail to notify the closed main loop, and
  leave the alias visible through `SharedCtx::conn`. Later local send,
  open-stream, graceful-shutdown, or maintenance paths can observe a stale
  connection that the main loop never accepted. This is distinct from
  ISSUE-057/063/064/065/067/068, which cover stale or malformed main-loop
  events after they reach `P2pNetwork::process_internal`; ISSUE-136, which
  covers cleanup blocked after `PeerDisconnected` on a full main queue; and
  ISSUE-139, which covers early `PeerConnectError` panic before authenticated
  alias registration.
- Evidence test:
  - `cargo test authenticated_peer_alias_must_be_cleaned_if_main_loop_closed_before_connected_event`
  - Failure summary: a valid incoming handshake runs with `main_rx` dropped;
    after the task tries to send `PeerConnected`, `ctx.conn(&conn_id)` remains
    `Some`, but expected cleanup should unregister the alias and leave it
    `None`.

### ISSUE-145: Mismatched PeerData can mutate routes on a live connection

- Category: correctness, internal-event integrity, route stability
- Score: 70/100
- Reviewer: `Newton the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: peer tasks emit
    `MainEvent::PeerData(self.conn_id, self.to_id, ...)`, so the event carries
    both the connection id and the authenticated peer id.
  - `src/lib.rs`: `P2pNetwork::process_internal` receives
    `MainEvent::PeerData(conn, peer, data)` but only logs `peer`.
  - `src/lib.rs`: the sync handler calls `self.router.apply_sync(conn, route)`
    and `self.discovery.apply_sync(now_ms, advertise)` without validating that
    the reported `peer` matches the live connection owner.
  - `src/router.rs`: `RouterTable::apply_sync` resolves the route source from
    `directs[conn]`, so a mismatched `PeerData` still mutates routes through the
    live connection.
- Impact: a stale, reordered, or malformed peer task event for a known live
  connection can apply route/discovery data even when its peer id no longer
  matches the connection owner. This can install or remove routes as if the data
  came from the real authenticated peer, causing route poisoning and noisy path
  state. This is distinct from ISSUE-063, which covers unknown/stale
  `PeerData` panicking without a direct route; ISSUE-066, which covers
  mismatched `PeerDisconnected`; ISSUE-067, which covers mismatched
  `PeerConnected`; ISSUE-068, which covers mismatched `PeerStats`; and
  ISSUE-135, which covers stale `PeerConnectError` removing a live neighbour.
- Evidence test:
  - `cargo test peer_data_must_validate_peer_matches_connection`
  - Failure summary: with live direct route `ConnectionId(10) -> PeerId(2)`,
    injecting `MainEvent::PeerData(ConnectionId(10), PeerId(99), Sync { ... })`
    with a route advertisement for `PeerId(4)` installs
    `PeerId(4) -> ConnectionId(10)`; expected mismatched `PeerData` to be
    ignored and leave no route to `PeerId(4)`.

### ISSUE-146: Shared-key handshake request tokens are replayable

- Category: security, authentication replay
- Score: 70/100
- Reviewer: `Jason the 2nd`, confirmed.
- Affected code:
  - `src/secure.rs`: `HandshakeData` contains only `from`, `to`, `timestamp`,
    and `is_initiator`.
  - `src/secure.rs`: `SharedKeyHandshake::generate_handshake` signs that
    deterministic payload with the shared key and static seed.
  - `src/secure.rs`: `SharedKeyHandshake::validate_handshake` checks timestamp
    freshness, peer ids, role, and hash, then returns `Ok(())` without nonce,
    server challenge, session binding, or replay cache.
  - `src/secure.rs`: `verify_request` delegates directly to that stateless
    validation.
- Impact: any captured valid request token can authenticate another connection
  as the same peer until `HANDSHAKE_TIMEOUT` expires. This is distinct from
  ISSUE-002, which covers future-dated tokens being accepted before their time
  and extending the replay window; ISSUE-021, which covers timestamp overflow;
  ISSUE-016, which covers `connect()` success before identity authentication;
  and ISSUE-113/114, which cover duplicate connection coalescing after or
  during connection establishment.
- Evidence test:
  - `cargo test request_handshake_tokens_must_not_be_replayable`
  - Failure summary: a request token created at timestamp `1000` verifies at
    `1005` and then verifies again at `1010`; expected the second use of the
    same request blob to be rejected.

### ISSUE-147: Valid route sync is dropped when the main event queue is full

- Category: bad-network stability, route correctness, backpressure
- Score: 73/100
- Reviewer: `Hypatia the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerConnectionInternal::on_msg` handles
    inbound `PeerMessage::Sync` by calling
    `self.main_tx.try_send(MainEvent::PeerData(...))`.
  - `src/peer/peer_internal.rs`: when the bounded main queue is full, the
    `PeerData` send failure is only logged as `queue main loop full`; the valid
    route/discovery sync is not queued, retried, or coalesced.
  - `src/lib.rs`: `P2pNetwork::process_internal` is the only path that applies
    `PeerMainData::Sync` through `router.apply_sync` and
    `discovery.apply_sync`.
  - `src/ctx.rs`: unicast and stream setup depend on current router state, so a
    dropped sync can leave a valid destination unreachable.
- Impact: under main-loop backpressure, an authenticated peer can send a valid
  route/discovery sync and have it silently lost before the network applies it.
  The advertised destination remains unreachable for later unicast or stream
  setup until another sync happens to arrive, creating avoidable route churn and
  intermittent `route not found` behavior in high load or bad network
  conditions. This is distinct from ISSUE-049/050/056, which cover outbound API
  blocking on congested peer control queues; ISSUE-119/120, which cover inbound
  service delivery drops; ISSUE-133/136, which cover lifecycle events blocked by
  a full main queue; and ISSUE-145, which covers malformed `PeerData` being
  accepted.
- Evidence test:
  - `cargo test valid_sync_must_survive_full_main_event_queue -- --nocapture`
  - Failure summary: the test authenticates a real QUIC peer, drains the initial
    `PeerConnected`, fills the one-slot main queue, and sends a valid
    `PeerMessage::Sync` advertising `PeerId(4)`. After draining the dummy event,
    no `MainEvent::PeerData` arrives within one second, so the advertised route
    cannot be applied and `PeerId(4)` remains unreachable.

### ISSUE-148: Alias shutdown leaves pending cached-hint lookups stuck

- Category: correctness, graceful-shutdown stability, alias failover
- Score: 62/100
- Reviewer: `Averroes the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: cached alias lookup sends
    `AliasMessage::Check(alias_id)` to cached peers and stores the pending
    request as `FindRequestState::CheckHint(now, slot.clone())`.
  - `src/service/alias_service.rs`: `AliasMessage::NotFound` removes the sender
    from the pending hint set and broadcasts `Scan(alias_id)` when no checked
    hint peers remain.
  - `src/service/alias_service.rs`: `AliasMessage::Shutdown` clears cache
    entries but does not update any in-flight `find_reqs` that are already
    waiting on the stopped peer.
- Impact: if an alias lookup is waiting on a cached hint and that peer
  gracefully shuts down, the lookup is not immediately failed over to a network
  scan or completed. It remains stuck until `HINT_TIMEOUT_MS`, adding avoidable
  lookup latency and churn during graceful shutdown or bad-network conditions.
  This is distinct from ISSUE-022, which covers shutdown evicting alias hints
  learned from unrelated peers; ISSUE-035/041, which cover unbounded pending
  find state; ISSUE-090, which covers unchecked `Found` replies; ISSUE-101,
  which covers unbounded per-alias hints; and ISSUE-109, which covers
  unsolicited cache poisoning.
- Evidence test:
  - `cargo test shutdown_from_cached_hint_must_unblock_pending_find -- --nocapture`
  - Failure summary: after a cached-hint lookup sends
    `Check(AliasId(1))` to `PeerId(2)`, `PeerId(2)` sends
    `AliasMessage::Shutdown`. The service emits no output, but expected it to
    immediately broadcast `Scan(AliasId(1))` or otherwise unblock the pending
    lookup instead of waiting for hint timeout.

### ISSUE-149: Stream open waits forever if the peer withholds `StreamConnectRes`

- Category: bad-network stability, stream setup, timeout correctness
- Score: 74/100
- Reviewer: `Ptolemy the 2nd`, confirmed.
- Affected code:
  - `src/peer/peer_internal.rs`: `open_bi` wraps only
    `connection.open_bi()` in `OPEN_BI_TIMEOUT`.
  - `src/peer/peer_internal.rs`: after writing `StreamConnectReq`, `open_bi`
    awaits `wait_object::<_, StreamConnectRes, ...>` with no timeout.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::open_stream` waits on the
    spawned `open_bi` task result, so callers can wait indefinitely for the
    missing response.
  - `src/peer/peer_internal.rs`: relay forwarding also awaits downstream
    `alias.open_stream(...)` before answering the upstream opener, so a
    downstream non-response can stall the upstream pipe setup too.
- Impact: a connected peer can accept a bidirectional stream, read the
  `StreamConnectReq`, and then keep the stream open without ever sending
  `StreamConnectRes`. The opener has no setup deadline after the QUIC stream is
  created, so `open_stream` can hang indefinitely and retain the spawned open
  task. This is distinct from ISSUE-056, which blocks before the peer task
  receives `OpenStream`; ISSUE-117, which covers inbound streams that never send
  `StreamConnectReq`; and ISSUE-011/012, which cover false success after
  destination service delivery fails.
- Evidence test:
  - `cargo test open_stream_must_timeout_when_peer_withholds_connect_response -- --nocapture`
  - Failure summary: a raw authenticated peer accepts the stream-open
    bidirectional stream and reads `StreamConnectReq`, but withholds
    `StreamConnectRes`. The caller's `open_stream` task does not return within
    2.5 seconds, so the test aborts it and fails; expected stream setup to
    return `Err` within the setup timeout instead of hanging.

### ISSUE-150: Stale pubsub destroy controls create phantom channel state

- Category: correctness, pubsub lifecycle stability, cleanup
- Score: 58/100
- Reviewer: `Sagan the 2nd`, confirmed.
- Affected code:
  - `src/service/pubsub_service.rs`: `InternalMsg::PublisherDestroyed` accepts
    any `(local_id, channel)` pair and calls
    `self.channels.entry(channel).or_default()`.
  - `src/service/pubsub_service.rs`: the publisher destroy branch ignores
    whether `state.local_publishers.remove(&local_id)` actually removed a live
    handle.
  - `src/service/pubsub_service.rs`: when the local publisher set is empty, it
    runs leave handling and broadcasts `PublisherLeaved(channel)` even if the
    destroy was stale or unknown.
  - `src/service/pubsub_service.rs`: `InternalMsg::SubscriberDestroyed` mirrors
    the same behavior for subscriber handles and `SubscriberLeaved(channel)`.
- Impact: a stale, duplicated, or malformed destroy control for a never-seen
  local handle creates phantom empty channel state and can broadcast false leave
  messages even though there was no prior local membership. This can add retained
  channel entries and noisy membership churn under handle-drop races or bad
  internal ordering. This is distinct from ISSUE-108, which covers empty channel
  state retained after a legitimate create/destroy lifecycle; this issue covers
  destroy controls that should have been no-ops because the handle was never
  registered.
- Evidence test:
  - `cargo test stale_pubsub_destroy_must_not_create_phantom_channel -- --nocapture`
  - Failure summary: on a fresh pubsub service, processing
    `PublisherDestroyed(rand, PubsubChannelId(77))` creates
    `PubsubChannelId(77)` in `service.channels`; expected unknown publisher and
    subscriber destroy controls to leave no phantom channel state.

### ISSUE-151: Stopped peer route is resurrected by connection RTT ticks

- Category: correctness, graceful-shutdown stability, route lifecycle
- Score: 68/100
- Reviewer: `Lagrange the 2nd`, confirmed.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerStopped` by removing discovery state and calling
    `router.del_peer(&peer)`.
  - `src/lib.rs`: the same branch does not stop the peer task or unregister its
    ability to mutate shared router state.
  - `src/peer/peer_internal.rs`: the peer connection task ticker independently
    calls `self.ctx.router().set_direct(self.conn_id, self.to_id, rtt_ms)` every
    second.
  - `src/router.rs`: `set_direct` unconditionally recreates direct route state
    for the connection and peer.
- Impact: after a graceful stop notification removes a peer route, the still-live
  connection task can recreate that route on its next RTT tick. The node can
  treat a stopped peer as routable again, causing route churn, reconnect
  suppression, and later unicast or stream attempts toward a peer that already
  announced it stopped. This is distinct from ISSUE-051, which covers the
  stopped peer remaining in neighbour/context state; this issue proves the
  route deletion itself is undone by an independent connection task writer.
- Evidence test:
  - `cargo test peer_stopped_route_must_not_be_resurrected_by_connection_ticker -- --nocapture`
  - Failure summary: after node2 processes
    `MainEvent::PeerStopped(stopped_conn, node1)`, `node2.router.action(node1)`
    becomes `None`; after 1.2 seconds it becomes `Some(Next(stopped_conn))`
    again because the connection ticker calls `set_direct`.

### ISSUE-152: Stale alias `NotFound` evicts cached hints without request correlation

- Category: correctness, alias cache stability, bad-network ordering
- Score: 63/100
- Reviewer: `Huygens the 2nd`, confirmed.
- Affected code:
  - `src/service/alias_service.rs`: `AliasMessage::NotFound` removes the sender
    from `self.cache[alias_id]` and can pop the alias entry before validating
    whether any matching lookup is active.
  - `src/service/alias_service.rs`: only after mutating cache does the handler
    inspect `find_reqs` and `FindRequestState::CheckHint`.
  - `src/service/alias_service.rs`: cached lookup later relies on
    `self.cache[alias_id]` to choose which peers receive `AliasMessage::Check`.
- Impact: a delayed or unsolicited `NotFound` from an older lookup can erase a
  currently valid cached alias hint even when there is no active `CheckHint`
  request for that alias. Later finds fall back to broadcast scan instead of
  checking the cached peer, adding avoidable latency and network noise under
  reordered messages or churn. This is distinct from ISSUE-090, which covers
  unchecked `Found` completing an active lookup; ISSUE-109, which covers
  unsolicited `Found` creating cache hints; and ISSUE-148, which covers
  `Shutdown` failing to advance an active cached-hint lookup.
- Evidence test:
  - `cargo test stale_not_found_must_not_evict_alias_cache_without_pending_check -- --nocapture`
  - Failure summary: after `NotifySet(AliasId(7))` caches `PeerId(2)` and no
    `find_reqs` entry exists, a `NotFound(AliasId(7))` from `PeerId(2)` removes
    the cache entry; expected stale `NotFound` without a matching pending
    `CheckHint` request to leave the valid cached hint intact.

### ISSUE-153: Discovery ticks enqueue duplicate connect commands without coalescing

- Category: high-load stability, resource exhaustion, discovery backpressure
- Score: 71/100
- Reviewer: `Curie the 2nd`, confirmed after `Carver the 2nd` discovery.
- Affected code:
  - `src/lib.rs`: `P2pNetwork::process_tick` iterates every
    `discovery.remotes()` entry and sends `ControlCmd::Connect` into
    `control_tx` on each tick.
  - `src/lib.rs`: `control_tx/control_rx` are an unbounded channel, so pending
    connect commands have no fixed admission cap.
  - `src/discovery.rs`: `PeerDiscovery::remotes()` returns seed remotes every
    tick so seeds are retried indefinitely.
  - `src/neighbours.rs`: duplicate suppression only observes already-connected
    peers; it does not account for queued or in-flight connect commands that
    have not yet been drained by the main loop.
- Impact: a node with an unreachable seed or repeatedly advertised remote can
  enqueue one duplicate `ControlCmd::Connect` per discovery tick if the control
  queue is not drained fast enough. Under slow main-loop processing, bad
  network conditions, or many seeds/remotes, this grows unbounded internal
  control backlog and later amplifies duplicate dial work. This is distinct from
  ISSUE-113, which covers duplicate outbound QUIC attempts after connect
  commands are processed while a dial is in flight, and ISSUE-125, which covers
  requester/API calls filling the same unbounded control queue. This issue is
  produced internally by normal discovery ticking with no public API spam.
- Minimal fix proposal: track pending discovery connect targets by peer id or
  address and skip enqueueing a new `ControlCmd::Connect` while one is already
  queued or in flight; clear that marker when the connect succeeds, fails, or
  reaches a bounded retry deadline.
- Evidence test:
  - `cargo test discovery_tick_connect_backlog_must_coalesce_duplicate_remotes -- --nocapture`
  - Failure summary: calling `process_tick` 1,025 times with one unreachable
    seed leaves `node.control_rx.len() == 1025`; expected discovery retries to
    coalesce to at most one pending connect per remote.

### ISSUE-154: Stale FetchChanged response cancels a newer replicated-KV repair

- Category: correctness, replicated-KV repair stability, bad-network ordering
- Score: 66/100
- Reviewer: `Curie the 2nd`, confirmed after `Linnaeus the 2nd` discovery.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` stores the current repair request in
    `self.sending_req`, but a later broadcast can replace it with a wider
    `FetchChanged` request.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` accepts any successful
    `RpcRes::FetchChanged(Ok(_))` without checking that it matches the current
    request's `from` and `count`.
  - `src/service/replicate_kv_service/remote_storage.rs`: after applying the
    response, `on_rpc_res` unconditionally clears `self.sending_req`.
- Impact: when a delayed response to an older narrow repair arrives after a
  newer wider repair has been requested, the stale response can advance only
  part of the gap and then clear the newer pending request. The replica remains
  stale for the rest of the missing version range until a future broadcast
  happens to restart repair. This is distinct from ISSUE-086, which covers
  unsolicited success without any request; ISSUE-111, which covers empty success
  canceling an active repair; ISSUE-141, which covers a partial response to the
  current wider request; and ISSUE-071, which covers retrying a stale request
  after broadcasts already filled the gap.
- Minimal fix proposal: store a typed pending `FetchChanged { from, count }`
  descriptor and reject or ignore responses that cannot be matched to that
  descriptor; after applying a matched response, keep or narrow the pending
  descriptor until the entire requested range is covered.
- Evidence test:
  - `cargo test working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair -- --nocapture`
  - Failure summary: after a `FetchChanged { from: Version(1), count: 1 }`
    request is superseded by `{ from: Version(1), count: 5 }`, a delayed
    response containing only `Version(1)` clears the wider repair. The timeout
    tick emits no follow-up `FetchChanged { from: Version(2), count: 4 }`.

### ISSUE-184: Replicated KV duplicates in-flight FetchChanged repairs for the same gap

- Category: bad-network stability, replicated-KV repair backpressure,
  duplicate control traffic
- Score: 57/100
- Reviewer: `Poincare the 3rd`, confirmed after `Planck the 3rd` discovery.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::apply_pendings` computes a missing range and always queues
    `RpcReq::FetchChanged { from, count }` when the first pending version is
    discontinuous.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` inserts every future
    `BroadcastEvent::Changed` into `pendings` and immediately calls
    `apply_pendings`.
  - `src/service/replicate_kv_service/remote_storage.rs`: `self.sending_req`
    is overwritten with the new request without checking whether an equivalent
    repair is already in flight.
- Impact: a remote that sends multiple future `Changed` broadcasts while the
  same lower version gap is still missing can make the receiver emit duplicate
  `FetchChanged` repairs before any timeout or response. Under packet loss or
  reordered broadcasts this creates avoidable duplicate control traffic and
  resets the single opaque `sending_req` timestamp. This is distinct from
  ISSUE-027, which covers unbounded pending future-change memory growth;
  ISSUE-071, which covers retrying stale repairs after broadcasts fill the gap;
  and ISSUE-111/141/154, which cover response-side repair cancellation or
  continuation bugs.
- Minimal fix proposal: track the in-flight repair as typed state, such as
  `Option<(from, count, sent_at)>`, and suppress emission when the computed gap
  is already covered by the active repair. Resend only from `on_tick`; replace
  or widen/narrow the descriptor only when the requested repair range truly
  changes.
- Evidence test:
  - `cargo test working_state_must_not_duplicate_inflight_fetch_changed_for_same_gap -- --nocapture`
  - Failure summary: after a future `Changed(version=10)` queues
    `FetchChanged { from: Version(1), count: 9 }`, a later
    `Changed(version=11)` before timeout/response queues the same
    `FetchChanged { from: Version(1), count: 9 }` again; expected no duplicate
    in-flight repair request.

### ISSUE-185: Pubsub keeps remote subscriber membership after graceful peer stop

- Category: correctness, graceful-shutdown stability, pubsub lifecycle
- Score: 56/100
- Reviewer: `Popper the 3rd`, confirmed after `Herschel the 3rd`
  discovery.
- Affected code:
  - `src/lib.rs`: `MainEvent::PeerStopped` updates discovery and router state
    only.
  - `src/service.rs`: `P2pServiceEvent` has no peer stopped or disconnected
    lifecycle variant for service state machines.
  - `src/service/pubsub_service.rs`: remote publisher/subscriber membership
    is removed only by pubsub protocol `PublisherLeaved` and
    `SubscriberLeaved` messages.
- Impact: when a remote subscriber gracefully stops, the network layer accepts
  `PeerStopped` and removes routing state, but `PubsubService` keeps that peer
  in `remote_subscribers` and never emits
  `PublisherEvent::PeerLeaved(PeerSrc::Remote(peer))`. Publishers continue to
  believe the stopped peer is a live destination, which can feed later
  delivery failures or RPC timeout behavior under churn. This is distinct from
  ISSUE-026 and ISSUE-080, which cover heartbeat repair after missed leave
  messages; from ISSUE-162 and ISSUE-165, which cover the same missing service
  lifecycle fanout in replicated-KV and visualization; and from ISSUE-163,
  which covers RPC behavior after stale remote sends fail.
- Minimal fix proposal: add a peer lifecycle event such as
  `P2pServiceEvent::PeerStopped(PeerId)` or
  `P2pServiceEvent::PeerDisconnected(PeerId)`, fan it out from accepted
  `P2pNetwork::process_internal` peer lifecycle events, and have pubsub remove
  that peer from every `remote_subscribers` and `remote_publishers` set while
  emitting corresponding local `PeerLeaved` events. Keep heartbeat cleanup as
  fallback for silent loss.
- Evidence test:
  - `cargo test pubsub_must_remove_remote_subscriber_on_graceful_peer_stop -- --nocapture`
  - Failure summary: node1 processes node2's graceful
    `PeerStopped`/disconnect after node1's publisher learns node2 as a remote
    subscriber, but the publisher does not receive
    `PublisherEvent::PeerLeaved(PeerSrc::Remote(node2))` before the prompt
    leave timeout; expected pubsub membership to be removed immediately.

### ISSUE-186: Ignored replicated-KV broadcasts refresh stale remote activity

- Category: stability, resource cleanup, bad-network resilience
- Score: 54/100
- Reviewer: `Nietzsche the 3rd`, confirmed after local discovery.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `RemoteStore::on_broadcast` sets `last_active = Instant::now()`
    unconditionally before dispatching to the current state.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_broadcast` ignores stale or equal
    `BroadcastEvent::Version` values when they do not advance the working
    version or create a repair request.
  - `src/service/replicate_kv_service.rs`: remote cleanup depends on
    `remote.last_active().elapsed() < REMOTE_TIMEOUT_MS`.
- Impact: stale, replayed, or noisy version broadcasts that make no state
  progress can keep an otherwise inactive remote store alive. Under bad network
  replay or adversarial traffic, timeout cleanup can be delayed or prevented
  even though no valid replicated-KV state changed. This is distinct from
  ISSUE-140, which covers ignored RPC responses refreshing activity through
  `RemoteStore::on_rpc_res`; this issue covers the broadcast ingress path.
- Minimal fix proposal: refresh `last_active` only when a broadcast is
  accepted and actionable. The smallest robust shape is for state handlers to
  return whether they consumed or progressed from an event, then update
  liveness only on true. A narrower local fix can compare state/output before
  and after dispatch and refresh only when there is a state transition,
  outbound event, version advance, or pending repair change.
- Evidence test:
  - `cargo test ignored_broadcast_must_not_refresh_remote_activity -- --nocapture`
  - Failure summary: a stale `WorkingState` remote already at `Version(5)`
    receives an ignored `BroadcastEvent::Version(Version(5))`; no output is
    produced, but `last_active` changes from the stale instant to
    `Instant::now()`, preventing timeout cleanup from recognizing the remote as
    inactive.

### ISSUE-187: Graceful PeerStopped is hidden from public network events

- Category: correctness, graceful-shutdown stability, public API lifecycle
- Score: 49/100
- Reviewer: `Mendel the 3rd`, confirmed after `Godel the 3rd` discovery.
- Affected code:
  - `src/lib.rs`: `P2pNetworkEvent` exposes `PeerConnected`,
    `PeerDisconnected`, and `Continue`, but has no explicit graceful-stop
    event.
  - `src/lib.rs`: `P2pNetwork::process_internal` handles
    `MainEvent::PeerStopped(conn, peer)` by updating discovery/router state
    and then returning `P2pNetworkEvent::Continue`.
- Impact: applications that maintain their own live-peer set from
  `P2pNetwork::recv()` cannot observe an accepted graceful shutdown when the
  stop control message is processed. They can keep the stopped peer live until
  a later transport disconnect event, or miss the graceful lifecycle transition
  entirely if they rely on `PeerStopped` as the authoritative shutdown signal.
  This is distinct from ISSUE-051, which covers internal neighbour cleanup;
  ISSUE-065 and ISSUE-066, which cover stale or mismatched disconnect events;
  ISSUE-162, ISSUE-165, and ISSUE-185, which cover service-specific lifecycle
  fanout; and ISSUE-001/004, which cover forged or seed `PeerStopped` handling.
- Minimal fix proposal: for a validated `MainEvent::PeerStopped(conn, peer)`,
  perform stopped-neighbour/context cleanup and return
  `P2pNetworkEvent::PeerDisconnected(conn, peer)`. If the API needs to
  distinguish graceful stop from transport loss, add
  `P2pNetworkEvent::PeerStopped(conn, peer)` and emit that instead.
- Evidence test:
  - `cargo test peer_stopped_must_emit_public_disconnect_event -- --nocapture`
  - Failure summary: after node2 establishes a real connection to node1,
    processing `MainEvent::PeerStopped(stopped_conn, node1)` returns
    `P2pNetworkEvent::Continue`; expected a visible
    `P2pNetworkEvent::PeerDisconnected(stopped_conn, node1)` lifecycle event
    for public consumers.

### ISSUE-155: Stale pubsub leave removes membership confirmed by newer heartbeat

- Category: bad-network correctness, pubsub membership stability
- Score: 64/100
- Reviewer: `Boole the 2nd`, confirmed after `Euclid the 2nd` discovery.
- Affected code:
  - `src/service/pubsub_service.rs`: `ChannelHeartbeat` carries only boolean
    publisher/subscriber state, with no generation, epoch, or timestamp.
  - `src/service/pubsub_service.rs`: inbound `PubsubMessage::Heartbeat` can add
    `from_peer` to `remote_publishers` when `publish` is true.
  - `src/service/pubsub_service.rs`: inbound
    `PubsubMessage::PublisherLeaved` later removes `from_peer` from
    `remote_publishers` without proving the leave is newer than the heartbeat.
  - `src/service/pubsub_service.rs`: `SubscriberLeaved` mirrors the same
    stale-removal risk for `remote_subscribers`.
- Impact: under delayed or reordered pubsub control messages, a newer heartbeat
  can confirm that a remote peer is still publishing a channel, then an older
  delayed leave can remove that live membership and emit a false
  `PeerLeaved(Remote(peer))` event to local subscribers. This causes membership
  flapping and delivery gaps in bad networks. This is distinct from ISSUE-026
  and ISSUE-080, which cover heartbeat failing to remove stale members after a
  missed leave; this issue is the inverse ordering bug where stale leave
  overrides newer heartbeat state. It is also distinct from ISSUE-150, which
  covers stale local destroy controls creating phantom local channel state.
- Minimal fix proposal: add per-peer/per-channel publisher and subscriber
  generation values to join, leave, and heartbeat messages; store the latest
  observed generation and ignore leave messages older than the stored live
  generation. Keep the first change scoped to pubsub membership lifecycle
  messages.
- Evidence test:
  - `cargo test stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat -- --nocapture`
  - Failure summary: after a heartbeat with `publish=true` inserts `PeerId(2)`
    into `remote_publishers`, a delayed `PublisherLeaved(channel)` from the same
    peer removes it; expected stale leave to preserve the heartbeat-confirmed
    membership and not emit `PeerLeaved`.

### ISSUE-156: Relay delivers orphan downstream stream after upstream setup closes

- Category: bad-network stability, stream setup, pipe reliability
- Score: 64/100
- Reviewer: `Herschel the 2nd`, confirmed after `Hooke the 2nd` discovery.
- Affected code:
  - `src/peer/peer_internal.rs`: in the relay branch of `accept_bi`,
    `alias.open_stream(service, source, dest, meta).await` opens and delivers
    the downstream stream before the upstream side has been acknowledged.
  - `src/peer/peer_internal.rs`: the relay writes `Ok(())` to the upstream
    stream only after downstream setup succeeds.
  - `src/peer/peer_internal.rs`: `copy_bidirectional` starts only after that
    late upstream acknowledgement, so a failed acknowledgement can leave the
    downstream stream already accepted by the destination service.
- Impact: if an upstream caller sends a relayed `StreamConnectReq` and then
  closes the setup response side before the relay writes `StreamConnectRes`, the
  relay can still open and deliver a downstream stream to the destination. The
  destination service observes an accepted stream that has no live upstream
  caller or usable pipe, causing confusing stream lifecycle events and wasted
  resources under cancellations or bad networks. This is distinct from
  ISSUE-011 and ISSUE-012, which cover local destination delivery failures;
  ISSUE-056, which blocks before stream setup reaches the peer task; ISSUE-117,
  which covers idle inbound stream-connect requests; and ISSUE-149, which covers
  a downstream peer withholding `StreamConnectRes`.
- Minimal fix proposal: in the relay branch, treat downstream setup as tentative
  until the upstream success acknowledgement is written. If writing `Ok(())` to
  the upstream stream fails, immediately close/reset the downstream stream and
  return an error; for a broader fix, pass an upstream cancellation token into
  downstream `open_stream`.
- Evidence test:
  - `cargo test relay_must_not_deliver_downstream_stream_after_upstream_setup_closes -- --nocapture`
  - Failure summary: a raw authenticated node sends a relayed
    `StreamConnectReq` to node2 for node3, finishes the request, and stops the
    response side before node2 writes setup success. Node3 still receives
    `P2pServiceEvent::Stream(PeerId(1), b"orphan-relay-stream", _)`; expected
    the relay not to deliver a downstream stream after upstream setup closed.

### ISSUE-157: PeerConnected backpressure stalls authenticated peer run loop

- Category: high-load stability, connection lifecycle, head-of-line blocking
- Score: 66/100
- Reviewer: `Avicenna the 2nd`, confirmed after `Zeno the 2nd` discovery.
- Affected code:
  - `src/peer.rs`: after authentication, `run_connection` registers the
    `PeerConnectionAlias` in shared context.
  - `src/peer.rs`: before entering `PeerConnectionInternal::run_loop`, it
    awaits `main_tx.send(MainEvent::PeerConnected(conn_id, to_id, rtt_ms))` on
    the bounded main event queue.
  - `src/lib.rs`: `main_tx/main_rx` is a bounded channel, so a live but
    saturated main loop can backpressure that send indefinitely.
- Impact: when the main event queue is full during connection setup, an
  authenticated peer task can park before it starts reading peer messages,
  streams, or stop notifications. The alias is already registered, so other
  code can enqueue traffic to that connection, but the peer task has not entered
  its run loop to process it. This stalls authenticated traffic and lifecycle
  handling under high load. This is distinct from ISSUE-133, which blocks inside
  `PeerStopped` handling after the run loop is active; ISSUE-136, which blocks
  after the run loop exits while sending `PeerDisconnected`; and ISSUE-144,
  which covers a closed main loop causing alias leak after `PeerConnected`
  delivery fails.
- Minimal fix proposal: do not await bounded main-loop delivery before starting
  the peer run loop. Register the alias, start traffic processing, and report
  `PeerConnected` through a non-blocking or coalesced lifecycle path; if the
  main queue is full, keep bounded retry state outside the peer traffic loop.
- Evidence test:
  - `cargo test peer_connected_must_not_block_authenticated_connection_run_loop_on_full_main_queue -- --nocapture`
  - Failure summary: after node1 accepts node2 and its bounded main queue is
    filled, node2 has a registered alias and enqueues a unicast to node1, but
    node1's service receive times out. The authenticated node1 peer task is
    parked on `main_tx.send(PeerConnected)` and has not entered `run_loop`.

### ISSUE-158: Stale alias NotifySet resurrects hint after newer NotifyDel

- Category: correctness, alias cache stability, bad-network ordering
- Score: 62/100
- Reviewer: `Dirac the 2nd`, confirmed after `Nietzsche the 2nd` discovery.
- Affected code:
  - `src/service/alias_service.rs`: `AliasMessage::NotifySet(AliasId)` and
    `NotifyDel(AliasId)` carry no epoch, generation, or timestamp.
  - `src/service/alias_service.rs`: `NotifySet` unconditionally inserts the
    sender into `self.cache[alias_id]`.
  - `src/service/alias_service.rs`: `NotifyDel` removes the sender from the
    cache, but stores no tombstone or freshness marker.
  - `src/service/alias_service.rs`: later `Find` trusts the resurrected cache
    entry and sends `Check(alias_id)` to the stale peer.
- Impact: under delayed or reordered alias lifecycle messages, an older
  `NotifySet` can arrive after a newer `NotifyDel` from the same peer and
  recreate a stale cache hint. Later lookups waste work checking that stale
  peer instead of doing a fresh scan, adding lookup latency and noisy failed
  checks during churn. This is distinct from ISSUE-022, which covers
  `Shutdown` clearing unrelated aliases; ISSUE-090 and ISSUE-109, which cover
  unchecked or unsolicited `Found`; ISSUE-148, which covers shutdown not
  advancing a pending cached-hint lookup; ISSUE-152, which covers stale
  `NotFound` evicting valid hints; and ISSUE-155, which covers the same
  freshness pattern in pubsub membership rather than alias hints.
- Minimal fix proposal: add per-peer/per-alias generation values to
  `NotifySet` and `NotifyDel`, store the latest observed generation, and ignore
  lifecycle messages older than the stored generation. As a smaller interim
  mitigation, keep a short-lived `(peer, alias)` delete tombstone and ignore
  same-peer `NotifySet` during that grace window.
- Evidence test:
  - `cargo test stale_notify_set_must_not_resurrect_alias_after_newer_notify_del -- --nocapture`
  - Failure summary: after `NotifySet(AliasId(7))` from `PeerId(2)` creates a
    cache hint and `NotifyDel(AliasId(7))` removes it, a delayed
    `NotifySet(AliasId(7))` from the same peer re-inserts the cache entry;
    expected the stale set to be ignored and a later find to broadcast
    `Scan(AliasId(7))` instead of checking the stale peer.

### ISSUE-159: Outbound peer setup hangs before main control stream opens

- Category: bad-network stability, connection lifecycle, setup timeout
- Score: 67/100
- Reviewer: `Pascal the 2nd`, confirmed after `Singer the 2nd` discovery.
- Affected code:
  - `src/peer.rs`: the outbound connection task awaits the QUIC connection and
    then awaits `connection.open_bi()` without a peer-setup timeout.
  - `src/peer.rs`: authentication and `ConnectReq`/`ConnectRes` exchange only
    start after the main P2P control stream opens.
  - `src/lib.rs`: an outbound pending neighbour is inserted before the control
    stream opens, and cleanup depends on receiving
    `MainEvent::PeerConnectError`.
  - `src/neighbours.rs`: pending outbound entries stay resident until an error
    event removes them.
- Impact: a remote QUIC endpoint can complete the transport connection but
  advertise no bidirectional stream credit, or otherwise hold the main control
  stream setup open. The outbound peer task then parks before authentication,
  no `PeerConnectError` reaches the main loop, and the pending neighbour remains
  until QUIC idle timeout or longer. Repeated attempts under bad networks or
  hostile peers can accumulate stuck setup tasks and noisy pending state. This
  is distinct from ISSUE-134, which covers inbound unauthenticated connections
  waiting on `accept_bi`, and ISSUE-149, which covers authenticated stream setup
  after a peer task is already running.
- Minimal fix proposal: wrap outbound setup after QUIC connect in one bounded
  deadline, at least around `connection.open_bi()`, and send
  `MainEvent::PeerConnectError` on timeout so the main loop removes the pending
  neighbour. The smallest robust version uses one setup timeout for main control
  stream open, `ConnectReq` write, and `ConnectRes` read.
- Evidence test:
  - `cargo test outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open -- --nocapture`
  - Failure summary: a raw QUIC server accepts the transport connection while
    advertising `max_concurrent_bidi_streams(0)`. After the normal node queues
    a connect and the test drives `recv()` for the cleanup window, the pending
    neighbour count remains `1`; expected setup timeout cleanup to remove it.

### ISSUE-160: Relayed route replaces direct authenticated peer route

- Category: correctness, route stability, bad-network path selection
- Score: 68/100
- Reviewer: `Wegener the 2nd`, confirmed after `Kierkegaard the 2nd`
  discovery.
- Affected code:
  - `src/router.rs`: `set_direct` inserts an authenticated direct peer path,
    but the direct path is stored as just another scored candidate.
  - `src/router.rs`: `apply_sync` accepts relayed advertisements for peers that
    already have a direct authenticated connection.
  - `src/router.rs`: `PeerMemory::select_best` chooses the lowest score only,
    with no direct-path ownership preference for the destination peer.
  - `src/peer/peer_internal.rs`: unicast and stream forwarding trust the
    selected router path.
- Impact: a relay can advertise a very low-cost path to a peer that is already
  directly authenticated, causing local traffic for that peer to leave over the
  relay instead of the peer's own connection. This makes active paths noisy,
  can break pipe setup when the relay path is stale or lossy, and contradicts
  the router invariant that direct RTT should dominate. This is distinct from
  ISSUE-003, which covers low-margin route flapping and equal-score instability,
  and ISSUE-044, which requires score arithmetic overflow.
- Minimal fix proposal: prefer the direct connection for its authenticated peer
  unless that direct connection is removed or explicitly marked unusable. The
  smallest implementation is to check `RouterTable.directs` during selection,
  or store direct ownership in `PeerMemory`, and make direct paths outrank
  relayed paths for the same destination.
- Evidence test:
  - `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`
  - Failure summary: after installing a direct route to `PeerId(2)` over
    `ConnectionId(2)`, a relay on `ConnectionId(1)` advertises a cheaper path
    to `PeerId(2)`. The router changes the active route to
    `Some(Next(ConnectionId(1)))`; expected it to keep the authenticated direct
    route `Some(Next(ConnectionId(2)))`.

### ISSUE-161: Stopped peer route resurrects through third-party route sync

- Category: correctness, graceful-shutdown stability, route lifecycle
- Score: 64/100
- Reviewer: `Epicurus the 2nd`, confirmed after `Locke the 2nd` discovery.
- Affected code:
  - `src/lib.rs`: `PeerData::Sync` applies router sync unconditionally before
    applying discovery sync.
  - `src/lib.rs`: `PeerStopped` records a discovery tombstone and removes the
    router peer once.
  - `src/discovery.rs`: stopped-peer tombstones suppress discovery addresses,
    but are not visible to route sync.
  - `src/router.rs`: `apply_sync` accepts learned route entries without checking
    whether a peer is currently stopped.
- Impact: after a graceful stop removes a peer's route, a different
  authenticated relay can re-advertise a stale route to that stopped peer and
  make it routable again during the discovery tombstone window. This keeps
  graceful shutdown noisy under relay gossip and can send unicast or stream
  setup toward a peer that already announced it stopped. This is distinct from
  ISSUE-151, where the stopped peer's own still-running connection ticker
  resurrects its direct route.
- Minimal fix proposal: filter route sync entries through stopped-peer
  tombstones before calling `router.apply_sync`, or move tombstone awareness
  into `RouterTable` so stopped peer ids are rejected until tombstone expiry or
  a fresh authenticated direct restart proves the peer is live again.
- Evidence test:
  - `cargo test stopped_peer_route_must_not_be_resurrected_by_third_party_sync -- --nocapture`
  - Failure summary: after `MainEvent::PeerStopped` removes `PeerId(2)`, a
    third-party relay `PeerId(3)` advertises a route to `PeerId(2)`. The local
    router changes from `None` to `Some(Next(ConnectionId(30)))`; expected the
    graceful-stop tombstone to keep the stopped peer unroutable.

### ISSUE-162: Replicated KV keeps stopped peer data until idle timeout

- Category: correctness, graceful-shutdown stability, service lifecycle
- Score: 63/100
- Reviewer: `Franklin the 2nd`, confirmed after `Noether the 2nd` discovery.
- Affected code:
  - `src/service/replicate_kv_service.rs`: remote KV data is deleted only when
    `ReplicatedKvStore::on_tick` sees `REMOTE_TIMEOUT_MS` expire.
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvService::recv` consumes
    service payload events and ticks, but receives no peer stopped or
    disconnected lifecycle event.
  - `src/lib.rs`: `shutdown_gracefully` sends `PeerStopped` to connected peers.
  - `src/lib.rs`: `PeerStopped` handling updates discovery and routing only,
    not service-owned replicated-KV state.
- Impact: after a peer explicitly announces graceful shutdown, its replicated-KV
  rows remain visible on other nodes until the 10-second idle timeout. The
  module comment promises disconnected-node data will be deleted from other
  nodes, but graceful stop does not promptly remove that application state. This
  is distinct from ISSUE-051, ISSUE-151, and ISSUE-161, which cover neighbour
  and route state after `PeerStopped`, and from ISSUE-140, which covers ignored
  RPC responses refreshing stale KV activity.
- Minimal fix proposal: deliver peer lifecycle events to services, or add a
  narrower replicated-KV hook that calls `RemoteStore::destroy()` immediately on
  `PeerStopped` or `PeerDisconnected`. Keep the 10-second idle timeout only as
  fallback for silent network loss.
- Evidence test:
  - `cargo test replicated_kv_must_delete_remote_data_when_peer_gracefully_stops -- --nocapture`
  - Failure summary: node2 receives `KvEvent::Set(Some(node1), 7, 70)`, then
    node1 calls `shutdown_gracefully()`. No
    `KvEvent::Del(Some(node1), 7)` is emitted within the 3-second graceful-stop
    window; expected replicated KV to remove explicitly stopped peer data
    promptly instead of waiting for `REMOTE_TIMEOUT_MS`.

### ISSUE-163: Pubsub RPC waits for timeout after every remote send fails

- Category: correctness, pubsub RPC stability, bad-network delivery failure
- Score: 61/100
- Reviewer: `Einstein the 2nd`, confirmed after `Descartes the 2nd` discovery.
- Affected code:
  - `src/service/pubsub_service.rs`: `GuestPublishRpc` and the mirrored RPC
    paths treat non-empty remote membership sets as valid destinations.
  - `src/service/pubsub_service.rs`: each remote RPC send calls `send_to`, but
    the caller does not learn whether delivery succeeded.
  - `src/service/pubsub_service.rs`: pending RPC state is inserted even if every
    remote send failed immediately.
  - `src/service/pubsub_service.rs`: `send_to` logs `send_unicast` errors and
    returns no status.
  - `src/ctx.rs`: `send_unicast` can fail synchronously with `route not found`
    or `peer not found`.
- Impact: stale remote membership can make a pubsub RPC appear to have a
  destination, but if all sends fail immediately, the caller still waits until
  the RPC timeout sweep instead of receiving `NoDestination`. This ties up caller
  tasks and pending RPC state under churn or bad network conditions. This is
  distinct from ISSUE-043, which covers unbounded pending RPC retention;
  ISSUE-121, which covers coarse timeout granularity; and ISSUE-026/155, which
  cover stale membership cleanup.
- Minimal fix proposal: make `send_to` return `anyhow::Result<()>`; for RPC
  paths, count reached local destinations plus successful remote sends, and
  send `PubsubRpcError::NoDestination` without inserting pending state when the
  success count is zero. Keep timeout behavior only for RPCs delivered to at
  least one destination.
- Evidence test:
  - `cargo test pubsub_rpc_must_return_no_destination_when_all_remote_sends_fail -- --nocapture`
  - Failure summary: a channel has only stale remote subscriber `PeerId(99)` and
    no route. `GuestPublishRpc` calls `send_unicast`, which fails immediately,
    but `rx.try_recv()` remains `Err(Empty)` and `publish_rpc_reqs` retains the
    request; expected immediate `Err(NoDestination)` and no pending RPC.

### ISSUE-178: Pubsub RPC treats closed local event channels as live destinations

- Category: correctness, pubsub RPC stability, lifecycle stability
- Score: 57/100
- Reviewer: `Russell the 3rd`, confirmed after `Carson the 3rd` discovery.
- Affected code:
  - `src/service/pubsub_service.rs`: `GuestPublishRpc` and the mirrored RPC
    paths treat non-empty local publisher/subscriber maps as valid
    destinations.
  - `src/service/pubsub_service.rs`: local RPC delivery ignores
    `sub_tx.send(...)` or `pub_tx.send(...)` failures.
  - `src/service/pubsub_service.rs`: pending RPC state is inserted even if
    every local event-channel send failed immediately.
  - `src/service/pubsub_service/publisher.rs` and
    `src/service/pubsub_service/subscriber.rs`: local event receivers can close
    when handles are dropped.
- Impact: a dropped or otherwise closed local pubsub handle can remain in
  channel state and make an RPC appear to have a destination. If every local
  event send fails immediately, the caller still waits until the RPC timeout
  sweep instead of receiving `NoDestination`, and pending RPC state is retained.
  This is distinct from ISSUE-163, which covers failed remote `send_unicast`
  fanout, and from stale requester/answer-binding issues that cover authority
  rather than destination accounting.
- Minimal fix proposal: for RPC paths, count only successful fanout deliveries:
  increment for local `send(...)` success and remote `send_to(...)` success. If
  the count is zero, send `PubsubRpcError::NoDestination` and do not insert
  pending state. Also prune local publisher/subscriber entries whose event
  sender is closed.
- Evidence test:
  - `cargo test pubsub_rpc_must_return_no_destination_when_all_local_sends_fail -- --nocapture`
  - Failure summary: a channel contains only a local subscriber sender whose
    receiver is closed. `GuestPublishRpc` ignores the failed local send,
    `rx.try_recv()` remains `Err(Empty)`, and `publish_rpc_reqs` retains the
    request; expected immediate `Err(NoDestination)` and no pending RPC.

### ISSUE-179: Local alias shutdown leaves pending find waiters alive

- Category: lifecycle stability, graceful shutdown, alias lookup
- Score: 49/100
- Reviewer: `Socrates the 3rd`, confirmed after `Ampere the 3rd` discovery.
- Affected code:
  - `src/service/alias_service.rs`: `AliasControl::Find` inserts missing-alias
    lookups into `AliasServiceInternal::find_reqs` and parks caller waiters
    until a response or timeout.
  - `src/service/alias_service.rs`: `AliasControl::Shutdown` only queues
    `Broadcast(AliasMessage::Shutdown)` and does not transition local pending
    lookup state.
  - `src/service/alias_service.rs`: pending finds are completed only by remote
    `Found` responses or the periodic timeout sweep.
- Impact: a local alias service can be asked to shut down while callers are
  blocked in `AliasServiceRequester::find`. Those waiters remain unresolved and
  `find_reqs` remains populated until the normal scan timeout, so graceful
  shutdown leaves local tasks and metrics alive longer than necessary. This is
  distinct from ISSUE-022, which covers remote shutdown evicting unrelated
  cache entries; ISSUE-148, which covers remote shutdown from a cached hint
  peer leaving hint lookup stuck; ISSUE-029/130/132, which cover panic and
  closed-channel lifecycle issues; and ISSUE-137, which covers pending find
  races with later local registration.
- Minimal fix proposal: in `AliasControl::Shutdown`, drain `find_reqs`, send
  `None` to every waiter, decrement `P2P_ALIAS_LIVE_FIND_REQUEST` for each
  removed request, then broadcast `AliasMessage::Shutdown`. Optionally add a
  local `shutting_down` flag so later `Find` controls fail immediately.
- Evidence test:
  - `cargo test service::alias_service::test::local_shutdown_must_fail_pending_alias_finds -- --nocapture`
  - Failure summary: after a missing-alias `Find` creates a pending scan,
    `AliasControl::Shutdown` broadcasts `Shutdown` but `rx.try_recv()` remains
    `Err(Empty)` and `find_reqs` still contains the alias; expected immediate
    `Ok(None)` and cleared pending state.

### ISSUE-183: Local alias shutdown keeps serving local aliases

- Category: lifecycle stability, graceful shutdown, alias authority
- Score: 53/100
- Reviewer: `Newton the 3rd`, confirmed after `Cicero the 3rd` discovery.
- Affected code:
  - `src/service/alias_service.rs`: `AliasControl::Shutdown` only queues
    `Broadcast(AliasMessage::Shutdown)`.
  - `src/service/alias_service.rs`: `AliasControl::Find` later checks
    `self.local.contains_key(&alias_id)` and returns
    `Some(AliasFoundLocation::Local)`.
  - `src/service/alias_service.rs`: `AliasControl::Register` remains accepted
    after shutdown, so new local alias ownership can be created after the
    service announces shutdown.
- Impact: a local alias service can broadcast a graceful shutdown notification
  but continue to act as the authoritative local owner for registered aliases.
  Later callers can resolve the alias as `Local` even though the service has
  announced that it stopped. This is distinct from ISSUE-179, which covers
  pending missing-alias waiters that already exist at shutdown; ISSUE-022/148,
  which cover remote `AliasMessage::Shutdown` cache and hint behavior; and
  ISSUE-029/130/132, which cover stale requester or run-loop panic surfaces.
- Minimal fix proposal: in `AliasControl::Shutdown`, clear `self.local`, drain
  pending finds with `None`, and set a `shutting_down` flag. Later `Register`
  should no-op or fail, and later `Find` should immediately return `None`,
  while preserving a single shutdown broadcast.
- Evidence test:
  - `cargo test local_shutdown_must_stop_serving_local_aliases -- --nocapture`
  - Failure summary: after registering an alias and processing
    `AliasControl::Shutdown`, a later `Find` returns `Ok(Some(Local))` and
    `self.local` still contains the alias; expected `Ok(None)` and cleared
    local ownership.

### ISSUE-180: Relay stream setup can forward back to the ingress peer

- Category: correctness, stream relay stability, route-loop handling
- Score: 64/100
- Reviewer: `Carver the 3rd`, confirmed after `Heisenberg the 3rd` discovery.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerConnectionInternal::on_accept_bi`
    spawns `accept_bi` without passing the ingress `ConnectionId`.
  - `src/peer/peer_internal.rs`: `accept_bi` handles
    `RouteAction::Next(next)` by blindly calling `alias.open_stream(...)`.
  - `src/router.rs`: `SharedRouterTable::action` returns only the selected
    next connection and cannot exclude the connection that delivered the
    current stream setup request.
- Impact: if route state forms a two-node loop for a destination, a relay can
  receive a `StreamConnectReq` and open the next stream back over the same peer
  direction that sent the request. The opener does not receive a prompt
  `route loop`/`route not found` error and instead waits while both peers
  recursively create relayed stream setup attempts. This is distinct from
  ISSUE-003/160/161 route selection and stale-route issues; ISSUE-007/008 route
  table loop acceptance/advertisement; ISSUE-011/012 local service delivery
  false success; ISSUE-117/149/159/169/172/173 setup timeout/stall issues; and
  ISSUE-156 downstream orphan delivery after upstream closes.
- Minimal fix proposal: pass the ingress `ConnectionId` into `accept_bi`; when
  `RouteAction::Next(next)` equals that ingress connection, write
  `Err("route loop")` to the upstream stream and return. A broader hardening
  can add a hop limit or visited-path token to `StreamConnectReq`.
- Evidence test:
  - `cargo test relay_stream_must_not_forward_back_to_ingress_peer -- --nocapture`
  - Failure summary: node1 and node2 are given route state where `PeerId(99)`
    routes through each other. `service1.open_stream(PeerId(99), ...)` does not
    return an error within 500 ms and the test fails with `Elapsed(())`;
    expected prompt rejection instead of recursive relay setup.

### ISSUE-181: Local advertise config can gossip unroutable wildcard addresses

- Category: correctness, discovery stability, config validation
- Score: 45/100
- Reviewer: `Nash the 3rd`, confirmed after `Schrodinger the 3rd` discovery.
- Affected code:
  - `src/lib.rs`: `P2pNetworkConfig::advertise` accepts any `NetworkAddress`.
  - `src/lib.rs`: `P2pNetwork::new` passes configured `advertise` directly to
    discovery with `discovery.enable_local(...)`.
  - `src/discovery.rs`: `PeerDiscovery::enable_local` stores the address
    unchanged, and `create_sync_for` gossips it to peers.
- Impact: a node can be configured with `0.0.0.0:0` or another non-dialable
  local advertise address and then publish that address through discovery sync.
  Other peers can learn a useless dial candidate and repeatedly attempt failed
  connections under churn. This is distinct from ISSUE-005/055 learned bad
  advertisements; ISSUE-092/093 stale discovery timestamps and tombstones;
  ISSUE-103/112/177 self or incorrect connect candidates; ISSUE-153 duplicate
  retry enqueueing; and ISSUE-180 stream relay loop handling.
- Minimal fix proposal: validate `cfg.advertise` before `enable_local`, or make
  `PeerDiscovery::enable_local` reject/suppress non-dialable addresses. At
  minimum reject `ip().is_unspecified()` and `port() == 0`; return a config
  error if strict behavior is preferred.
- Evidence test:
  - `cargo test local_sync_must_not_advertise_unroutable_wildcard_address -- --nocapture`
  - Failure summary: after `enable_local(PeerId(1), 0.0.0.0:0)`,
    `create_sync_for` returns
    `PeerDiscoverySync([(PeerId(1), 100, NetworkAddress(0.0.0.0:0))])`;
    expected empty sync so non-dialable local addresses are not gossiped.

### ISSUE-182: QUIC admits unused unidirectional streams

- Category: high-load stability, transport admission, resource exhaustion
- Score: 52/100
- Reviewer: `Pascal the 3rd`, confirmed after `Ohm the 3rd` discovery.
- Affected code:
  - `src/quic.rs`: `configure_server` sets
    `max_concurrent_uni_streams(10_000_u32.into())`.
  - `src/quic.rs`: `configure_client` sets
    `max_concurrent_uni_streams(10_000_u32.into())`.
  - Production peer code uses only bidirectional streams and never calls
    `accept_uni`.
- Impact: a raw QUIC peer can open many unidirectional streams that the P2P
  protocol never consumes. Those streams expose unused transport state and flow
  control surface under bad-network or malicious-peer load. This is distinct
  from ISSUE-117, which covers authenticated idle bidirectional stream-connect
  handshakes; ISSUE-134, which covers unauthenticated raw QUIC connections
  waiting on the P2P control stream; ISSUE-149/169/172/173, which cover stalled
  bidirectional setup read/write paths; and ISSUE-159, which covers outbound
  peer setup before the main control stream opens.
- Minimal fix proposal: set `max_concurrent_uni_streams(0_u32.into())` in both
  server and client transport config. If future protocol features need
  unidirectional streams, introduce a small explicit cap plus an application
  accept/reject/drain path.
- Evidence test:
  - `cargo test unused_unidirectional_streams_must_not_be_admitted -- --nocapture`
  - Failure summary: a raw client opens 17 unidirectional QUIC streams against
    the endpoint while the protocol has no `accept_uni` path; expected zero
    admitted unused unidirectional streams.

### ISSUE-164: Tick route/discovery sync is dropped when peer control queue is full

- Category: stability, routing/discovery freshness, high-load backpressure
- Score: 57/100
- Reviewer: `Archimedes the 2nd`, confirmed after `Turing the 2nd` discovery.
- Affected code:
  - `src/lib.rs`: `process_tick` builds a `PeerMessage::Sync` containing
    route and discovery state for each connected neighbour.
  - `src/lib.rs`: tick delivery uses `alias.try_send(...)` and only logs when
    the peer control queue is full.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::try_send` delegates to
    bounded `control_tx.try_send`, so full queues reject immediately.
  - `src/peer.rs`: inbound sync is what updates the remote main loop's route
    and discovery state.
- Impact: route/discovery maintenance sync is fire-and-forget. If a connected
  peer's control queue is briefly full during a tick, the latest route and
  discovery snapshot is silently lost with no pending-sync slot, coalescing, or
  retry marker. A later tick may repair the state, but sustained load or a slow
  peer loop can repeatedly drop maintenance sync and leave neighbours with stale
  topology, causing noisy active-path selection and failed pipe setup. This is
  distinct from caller API blocking on congested peer queues, graceful shutdown
  delay, inbound sync dropped on the main queue, and pubsub RPC timeout after
  failed sends.
- Minimal fix proposal: keep one bounded pending sync slot per peer/connection.
  When `try_send` fails because the queue is full, store the latest
  `(route, advertise)` and retry it on the next tick before creating a fresh
  sync, replacing older pending state with newer state.
- Evidence test:
  - `cargo test tick_sync_must_not_be_dropped_when_peer_control_queue_is_full -- --nocapture`
  - Failure summary: the test fills a connected peer's control queue, runs
    `process_tick`, then drains the queue. No `PeerMessage::Sync` is delivered
    afterward; expected route/discovery sync to be queued, coalesced, or retried
    instead of dropped during transient queue pressure.

### ISSUE-165: Visualization keeps gracefully stopped peer until timeout

- Category: correctness, graceful-shutdown stability, service lifecycle
- Score: 54/100
- Reviewer: `Hubble the 2nd`, confirmed after `Faraday the 2nd` discovery.
- Affected code:
  - `src/lib.rs`: `shutdown_gracefully` sends `PeerStopped` before closing the
    endpoint.
  - `src/lib.rs`: `PeerStopped` and `PeerDisconnected` handling updates
    discovery, routing, and neighbour state only.
  - `src/service.rs`: `P2pServiceEvent` has payload and stream variants, but no
    peer lifecycle event for services.
  - `src/service/visualization_service.rs`: `VisualizationService` emits
    `PeerLeaved` only from its timeout sweep.
- Impact: when a peer gracefully stops and the network loop processes
  `PeerStopped`/`PeerDisconnected`, visualization consumers do not learn that
  the peer left until the collection timeout expires. This keeps monitoring or
  topology UIs stale after an explicit shutdown and can mislead operators about
  active peers. This is close to ISSUE-162's replicated-KV lifecycle gap, but
  distinct because it covers `VisualizationServiceEvent::PeerLeaved` semantics;
  existing visualization issues cover forged topology, topology disclosure,
  resource bounds, timeout arithmetic, and channel-close panic.
- Minimal fix proposal: add a peer lifecycle event such as
  `P2pServiceEvent::PeerStopped(PeerId)` or `PeerDisconnected(PeerId)`, fan it
  out from `P2pNetwork::process_internal` when graceful stop/disconnect is
  accepted, and have visualization remove the peer and emit `PeerLeaved`
  immediately. Keep timeout cleanup as fallback for silent network loss.
- Evidence test:
  - `cargo test visualization_must_emit_peer_leaved_on_graceful_peer_stop -- --nocapture`
  - Failure summary: node1's network loop logs and processes node2's graceful
    `PeerStopped`/disconnect, but `VisualizationService` does not emit
    `PeerLeaved(node2)` within the prompt leave window; expected lifecycle
    propagation instead of waiting for the timeout sweep.

### ISSUE-166: Broadcast replay is accepted again after dedup cache eviction

- Category: security, replay resistance, bad-network correctness
- Score: 58/100
- Reviewer: `Avicenna the 3rd`, confirmed after `Bernoulli the 3rd`
  discovery.
- Affected code:
  - `src/msg.rs`: `BroadcastMsgId` is the only replay/duplicate token carried
    by broadcast frames.
  - `src/ctx.rs`: `received_broadcast_msg` is an
    `LruCache<BroadcastMsgId, ()>` with a fixed capacity.
  - `src/ctx.rs`: `check_broadcast_msg` accepts the same id again after it has
    been evicted from the LRU.
  - `src/peer/peer_internal.rs`: accepted broadcasts are forwarded and
    delivered locally after passing `check_broadcast_msg`.
- Impact: broadcast replay protection is pure cache residency. After enough
  other broadcast ids churn the LRU, an identical already-accepted broadcast id
  is treated as new and can be forwarded or delivered again. This is distinct
  from ISSUE-017, which is false duplicate suppression across different
  sources/services because the key omits source and service; this issue is the
  opposite failure mode, where the same replay becomes accepted again after
  eviction.
- Minimal fix proposal: replace pure LRU dedupe with freshness-aware replay
  state. At minimum, key by `(source, service, msg_id)` and enforce a bounded
  epoch/timestamp or per-source sequence window so old broadcasts are rejected
  after cache churn while memory remains bounded.
- Evidence test:
  - `cargo test broadcast_replay_must_not_be_accepted_after_dedup_cache_eviction -- --nocapture`
  - Failure summary: the test accepts `BroadcastMsgId(7)`, verifies the
    immediate duplicate is rejected, then inserts 8,192 distinct ids. The same
    original id is accepted again after eviction; expected replays inside the
    configured freshness window to remain rejected.

### ISSUE-167: Expired non-seed discovery entries remain routable

- Category: correctness, route lifecycle, bad-network topology stability
- Score: 56/100
- Reviewer: `Pasteur the 3rd`, confirmed after `Raman the 3rd` discovery.
- Affected code:
  - `src/lib.rs`: `process_tick` calls `discovery.clear_timeout(now_ms)` but
    does not remove router state for expired peers.
  - `src/discovery.rs`: `clear_timeout` removes expired non-seed discovery
    remotes without returning the removed peer ids.
  - `src/lib.rs`: route sync can install relayed routes independently through
    `router.apply_sync`.
  - `src/router.rs`: learned routes have no freshness or discovery-liveness
    check.
- Impact: after a non-seed peer's discovery address expires during a long
  outage, a previously learned relayed route to that peer can remain selected.
  The node can still send or open streams toward the expired peer and may
  continue advertising stale reachability. This is distinct from ISSUE-161,
  which involves graceful-stop tombstones; ISSUE-092/093, which cover discovery
  freshness only; ISSUE-160, which covers direct-vs-relay preference; and
  ISSUE-147/164, which cover sync delivery loss.
- Minimal fix proposal: make `PeerDiscovery::clear_timeout` return expired
  non-seed peer ids, and have `P2pNetwork::process_tick` call
  `router.del_peer(&peer)` for each expired non-seed. Preserve configured seed
  behavior. A broader fix can add route freshness per learned path.
- Evidence test:
  - `cargo test discovery_timeout_must_remove_route_to_expired_non_seed -- --nocapture`
  - Failure summary: after discovery timeout removes the non-seed peer's
    address, `router.action(expired)` still returns
    `Some(Next(ConnectionId(20)))`; expected the expired non-seed peer to be
    unroutable.

### ISSUE-168: Duplicate pubsub local ids detach live publisher handles

- Category: correctness, lifecycle stability
- Score: 44/100
- Reviewer: `Jason the 3rd`, confirmed after `Mill the 3rd` discovery.
- Affected code:
  - `src/service/pubsub_service/publisher.rs`: `PublisherLocalId::rand`
    creates an untracked random id.
  - `src/service/pubsub_service.rs`: `InternalMsg::PublisherCreated` calls
    `state.local_publishers.insert(local_id, tx)` and silently replaces any
    existing live publisher with the same local id.
  - `src/service/pubsub_service.rs`: `InternalMsg::SubscriberCreated` has the
    same collision shape for `local_subscribers`.
  - `src/service/pubsub_service.rs`: destroy and action controls identify
    local handles only by `(local_id, channel)`, so a collision can corrupt
    ownership.
- Impact: if two live local pubsub handles collide on their random local id,
  the later create replaces the first handle's event sender and disconnects the
  first live handle. Later destroy/action controls for the shared id can also
  affect the wrong live handle. Public reproduction is probabilistic because
  ids are random `u64`, but the service state machine has no collision
  handling once a duplicate id appears. This is distinct from ISSUE-069 through
  ISSUE-075, which cover stale requesters after drop, and ISSUE-150, which
  covers unknown destroy controls creating phantom channel state.
- Minimal fix proposal: allocate local handle ids collision-aware inside
  `PubsubService`, or have create retry/reject when the id already exists.
  Longer term, pair handle controls with a generation/ownership token so
  destroy and action messages cannot affect a different handle with the same
  local id.
- Evidence test:
  - `cargo test duplicate_publisher_local_id_must_not_detach_live_handle -- --nocapture`
  - Failure summary: after two `PublisherCreated` controls use the same
    `PublisherLocalId`, the first publisher receiver returns
    `Err(Disconnected)` instead of receiving `PeerJoined(Local)` from a later
    local subscriber join.
  - Additional reviewer `Noether the 2nd` accepted
    `cargo test duplicate_subscriber_local_id_must_not_detach_live_handle -- --nocapture`
    as subscriber-side evidence for the same issue.
  - Additional failure summary: after two `SubscriberCreated` controls use the
    same `SubscriberLocalId`, the first subscriber receiver returns
    `Err(Disconnected)` instead of receiving `PeerJoined(Local)` from a later
    local publisher join.

### ISSUE-169: Stream open hangs while writing connect request to stalled peer

- Category: bad-network stability, stream setup, timeout correctness
- Score: 68/100
- Reviewer: independent validation in this audit turn, confirmed after
  subagent `019ede01-2c64-7e11-af87-56677fa09649` discovery.
- Affected code:
  - `src/peer/peer_internal.rs`: `open_bi` wraps only
    `connection.open_bi()` in `OPEN_BI_TIMEOUT`.
  - `src/peer/peer_internal.rs`: after the QUIC stream is opened, the
    `write_object(StreamConnectReq)` call has no setup deadline.
  - `src/stream.rs`: `write_object` awaits `write_all` for the length and
    payload without any timeout.
  - `src/service.rs` and `src/ctx.rs`: public `open_stream` callers wait on
    this setup path.
- Impact: an authenticated peer can accept a stream-open bidirectional stream
  but stop reading. If the request metadata is large enough to hit QUIC
  flow-control backpressure, the opener can remain stuck in
  `write_object(StreamConnectReq)` until transport idle timeout or longer. This
  is distinct from ISSUE-149, which covers a peer that reads
  `StreamConnectReq` and withholds `StreamConnectRes`; ISSUE-056, which blocks
  before peer task stream setup; ISSUE-156, which covers relay orphan delivery;
  and ISSUE-159, which covers outbound peer setup before the main control
  stream opens.
- Minimal fix proposal: wrap the entire stream setup sequence in one deadline:
  `connection.open_bi()`, writing `StreamConnectReq`, and reading
  `StreamConnectRes`. Keep `OPEN_BI_TIMEOUT` as the setup timeout or rename it
  to make the broader scope explicit.
- Evidence test:
  - `cargo test open_stream_must_timeout_when_connect_request_write_stalls -- --nocapture`
  - Failure summary: a raw authenticated peer accepts the stream-open
    bidirectional stream with a tiny receive window and then never reads it.
    The caller's `open_stream` task does not return within 2.5 seconds, so the
    test aborts it and fails; expected stream setup to return `Err`.

### ISSUE-170: PeerStopped forwarding loops indefinitely in cyclic meshes

- Category: correctness, graceful-shutdown stability, mesh control traffic
- Score: 62/100
- Reviewer: `Banach the 3rd`, confirmed after `Lorentz the 3rd` discovery.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerMessage::PeerStopped(peer_id)` is
    forwarded to every other connection except the sender each time it is
    received.
  - `src/peer/peer_internal.rs`: stopped-peer forwarding has no dedupe, TTL, or
    tombstone check before re-forwarding.
  - `src/lib.rs`: `MainEvent::PeerStopped` cleanup is idempotent but does not
    suppress duplicate peer-task forwarding in a mesh cycle.
- Impact: one legitimate graceful stop can circulate repeatedly through a
  cyclic mesh, generating thousands of duplicate `PeerStopped` events and
  control messages in less than a second. This wastes bandwidth and CPU during
  shutdown churn and can amplify a normal stop into a control-plane storm. This
  is distinct from ISSUE-001, which covers forged stop authority; ISSUE-051,
  ISSUE-151, ISSUE-161, ISSUE-165, and ISSUE-167, which cover cleanup effects
  after a stop; and ISSUE-133, which covers blocking while reporting one stop.
- Minimal fix proposal: add a bounded recent-stop dedupe set keyed by stopped
  `PeerId` in the peer/network forwarding path. Process local cleanup
  idempotently, but suppress forwarding when the stopped peer id was already
  seen recently. A hop limit or TTL would also work, but dedupe is the smaller
  change.
- Evidence test:
  - `cargo test peer_stopped_forwarding_must_be_deduplicated_in_mesh -- --nocapture`
  - Failure summary: one `shutdown_gracefully` from a stopped node connected to
    a live `B-C-D-B` mesh produces thousands of
    `MainEvent::PeerStopped(_, stopped)` events in 500 ms; expected at most one
    observation per live node.

### ISSUE-171: Replicated KV full resync deletes visible data before replacement snapshot

- Category: correctness, replicated-KV resync stability, bad-network ordering
- Score: 60/100
- Reviewer: `Fermat the 3rd`, confirmed after `Franklin the 3rd` discovery.
- Affected code:
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::on_rpc_res` handles a solicited
    `FetchChanged(Err(MissingData))` by scheduling `SyncFullState`.
  - `src/service/replicate_kv_service/remote_storage.rs`: the wrapper
    initializes the next state immediately after `on_rpc_res`.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `SyncFullState::init` clears all current remote slots and emits
    `KvEvent::Del` before any replacement snapshot response arrives.
- Impact: when a legitimate repair request falls back to full resync because
  the producer no longer has the requested changelog range, consumers can see
  the remote peer's live data disappear before the replacement snapshot has
  completed. If the snapshot stalls or is delayed, the false deletes remain
  visible for the whole resync window. Public reproduction requires changelog
  rollover or a producer returning `MissingData` for a real requested gap. This
  is distinct from ISSUE-087, which covers an unsolicited error forcing resync;
  ISSUE-111, ISSUE-141, and ISSUE-154, which cover repair cancellation or stale
  response correlation; and ISSUE-162, which covers stopped-peer cleanup.
- Minimal fix proposal: for resync from an existing `WorkingState`, keep the
  old `ctx.slots` visible while fetching a replacement full snapshot. Stage the
  full-sync result in temporary storage, then atomically diff/apply `Set` and
  `Del` events only after the terminal snapshot page is accepted. First-time
  sync can still start from empty state.
- Evidence test:
  - `cargo test solicited_full_resync_must_not_delete_existing_slots_before_snapshot_completes -- --nocapture`
  - Failure summary: after a real `FetchChanged { from: Version(3), count: 3 }`
    request, a `FetchChanged(Err(MissingData))` fallback immediately clears
    existing slots `{1, 2}`; expected the old slots to remain visible until the
    replacement full snapshot completes.

### ISSUE-175: Replicated KV emits delete changes for keys that were never present

- Category: correctness, replicated-KV API semantics, noisy state changes
- Score: 42/100
- Reviewer: `Volta the 3rd`, confirmed after `Gibbs the 3rd` discovery.
- Affected code:
  - `src/service/replicate_kv_service/local_storage.rs`: `LocalStore::del`
    increments the local version, records and broadcasts
    `Changed { action: Del }`, and emits `KvEvent::Del(None, key)` before
    checking whether `slots.remove(&key)` removes anything.
  - `src/service/replicate_kv_service/remote_storage.rs`:
    `WorkingState::apply_pendings` emits `KvEvent::Del(Some(remote), key)` for
    a delete change before checking whether replicated state contained that key.
- Impact: deleting an absent key through the normal local API creates a
  replicated version bump, broadcasts a delete to peers, and emits a local
  delete event even though no visible value changed. Peers can observe delete
  events for keys they never had, creating noisy application events and
  avoidable replication churn. This is distinct from ISSUE-171, which deletes
  visible data during full resync; ISSUE-162, which delays stopped-peer data
  cleanup; ISSUE-086/087, which cover unsolicited RPC responses; and version
  overflow issues.
- Minimal fix proposal: make local deletes idempotent against current state:
  remove first, and only if a slot existed should the store increment version,
  record/broadcast `Changed(Del)`, and emit `KvEvent::Del`. On the remote side,
  still advance protocol version for valid ordered deletes, but emit
  `KvEvent::Del(Some(remote), key)` only when `ctx.slots.remove(&key)` returns
  `Some(_)`.
- Evidence test:
  - `cargo test deleting_absent_key_must_not_emit_delete_event -- --nocapture`
  - Failure summary: calling `LocalStore::del(99)` on an empty store queues
    `NetEvent::Broadcast(Changed { key: 99, version: Version(1), action: Del })`
    and advances version; expected no output and version `0` because the key
    was never present.

### ISSUE-172: Outbound peer setup hangs while writing ConnectReq to stalled peer

- Category: bad-network stability, connection lifecycle, setup timeout
- Score: 68/100
- Reviewer: `James the 3rd`, confirmed after `Ptolemy the 3rd` discovery.
- Affected code:
  - `src/peer.rs`: outbound `run_connection` writes `ConnectReq` with
    `write_object` after `connection.open_bi()` succeeds, but that write has no
    peer setup deadline.
  - `src/stream.rs`: `write_object` awaits `write_all` for the length and
    payload without any timeout.
  - `src/lib.rs` and `src/neighbours.rs`: outbound pending neighbour cleanup
    depends on receiving a later `MainEvent::PeerConnectError`.
- Impact: a remote endpoint can complete QUIC setup and accept the P2P main
  control stream, then stop reading. If `ConnectReq` is large enough to hit
  peer flow control, the outbound peer task can remain stuck while writing the
  request. No `PeerConnectError` reaches the main loop, so the pending
  neighbour remains resident until transport idle timeout or longer. This is
  distinct from ISSUE-159, which stalls before the main control stream opens;
  ISSUE-169, which covers post-auth service stream setup; ISSUE-016, which
  covers early connect success; ISSUE-134, which covers inbound unauthenticated
  admission; and ISSUE-113, which covers duplicate outbound connects.
- Minimal fix proposal: wrap the whole outbound peer admission sequence in one
  bounded deadline: `connection.open_bi()`, writing `ConnectReq`, reading
  `ConnectRes`, and verifying the response. On timeout, emit
  `MainEvent::PeerConnectError` so pending neighbour cleanup runs.
- Evidence test:
  - `cargo test outbound_peer_setup_must_timeout_when_connect_request_write_stalls -- --nocapture`
  - Failure summary: a raw peer accepts the P2P control stream with a tiny
    receive window and never reads it. After the setup window, the normal node
    still has one pending neighbour; expected timeout cleanup to remove it.

### ISSUE-173: Inbound peer setup hangs while writing ConnectRes to stalled peer

- Category: bad-network stability, connection lifecycle, setup timeout
- Score: 68/100
- Reviewer: `Peirce the 3rd`, confirmed after `Einstein the 3rd` discovery.
- Affected code:
  - `src/peer.rs`: incoming `run_connection` reads `ConnectReq`, verifies the
    request, then writes `ConnectRes` with `write_object` without a peer setup
    deadline.
  - `src/stream.rs`: `write_object` awaits `write_all` for the length and
    payload without any timeout.
  - `src/peer.rs`: `PeerConnectError` is sent only after `run_connection`
    returns, so a stalled response write leaves the inbound setup task parked.
- Impact: a raw client can complete QUIC setup, open the P2P control stream,
  send a valid `ConnectReq`, and then stop reading. If `ConnectRes` is large
  enough to hit peer flow control, the inbound peer task can remain stuck while
  writing the response, and no `PeerConnectError` is emitted promptly. This is
  distinct from ISSUE-172, which covers outbound `ConnectReq` writes;
  ISSUE-159, which stalls before the main control stream opens; ISSUE-134,
  which covers inbound connections that never complete the P2P control-stream
  handshake; ISSUE-139, which covers panic while reporting early errors; and
  ISSUE-144, which covers alias leak after authentication.
- Minimal fix proposal: wrap the entire inbound peer admission sequence in one
  bounded deadline: accepting/opening the control stream, reading `ConnectReq`,
  verifying auth, and writing `ConnectRes`. On timeout, emit
  `MainEvent::PeerConnectError(conn_id, None, err)` and close the QUIC
  connection.
- Evidence test:
  - `cargo test inbound_peer_setup_must_timeout_when_connect_response_write_stalls -- --nocapture`
  - Failure summary: a raw client with a tiny receive window sends a valid
    `ConnectReq` and does not read the large `ConnectRes`; no
    `PeerConnectError` arrives within 2.5 seconds, but setup should fail
    promptly.

### ISSUE-176: Shared-key handshake response tokens are replayable

- Category: security, authentication replay
- Score: 66/100
- Reviewer: `Harvey the 3rd`, confirmed after `Curie the 3rd` discovery.
- Affected code:
  - `src/secure.rs`: `SharedKeyHandshake::generate_handshake` signs only
    deterministic `{from, to, timestamp, is_initiator}` payload data.
  - `src/secure.rs`: `SharedKeyHandshake::validate_handshake` checks timestamp,
    peer ids, role, and hash, but has no nonce, challenge/session binding, or
    replay cache.
  - `src/secure.rs`: `verify_response` delegates directly to that stateless
    validation.
  - `src/peer.rs`: outbound setup accepts `ConnectRes` once
    `verify_response` succeeds.
- Impact: a captured valid response token can be reused to satisfy another
  outbound setup within `HANDSHAKE_TIMEOUT`. This is distinct from ISSUE-146,
  which covers replayed request tokens authenticating inbound connections; this
  is the response-side verifier accepting the same `ConnectRes` blob more than
  once.
- Minimal fix proposal: add a fresh nonce or challenge to `ConnectReq`, require
  `ConnectRes` to sign and echo that nonce, and cache recently accepted
  `(from, to, nonce/signature)` values until expiry so replayed response tokens
  are rejected.
- Evidence test:
  - `cargo test response_handshake_tokens_must_not_be_replayable -- --nocapture`
  - Failure summary: a response token created at timestamp `1000` verifies at
    `1005` and then verifies again at `1010`; expected the second use of the
    same response blob to be rejected.

### ISSUE-188: Pubsub drops early remote publisher joins before local channel creation

- Category: correctness, pubsub membership stability, bad-network ordering
- Score: 51/100
- Reviewer: `Noether the 3rd`, confirmed after `Archimedes the 3rd` discovery.
- Affected code:
  - `src/service/pubsub_service.rs`: inbound `PublisherJoined(channel)` only
    records `from_peer` when `self.channels.get_mut(&channel)` already returns
    a channel state.
  - `src/service/pubsub_service.rs`: `InternalMsg::SubscriberCreated` later
    creates the channel state, but the earlier remote publisher membership has
    already been discarded and cannot be replayed to the new subscriber.
  - `src/service/pubsub_service.rs`: inbound `SubscriberJoined(channel)` has
    the symmetric dropped-before-local-publisher risk.
- Impact: under ordinary message ordering races, a remote publisher can
  advertise a channel before this node creates its local subscriber. The join is
  silently dropped, so the later subscriber starts without the live remote
  publisher and receives no `PeerJoined(Remote(...))` until a later heartbeat or
  repeated join happens. This is distinct from ISSUE-142, which assumes remote
  membership was already stored but not replayed; here the inbound join is never
  stored. It is also distinct from ISSUE-026/080 stale heartbeat cleanup and
  ISSUE-185 graceful peer-stop cleanup.
- Minimal fix proposal: retain early remote `PublisherJoined` and
  `SubscriberJoined` state in bounded per-channel membership state even when no
  local handle exists yet, then replay existing remote members when the first
  local subscriber/publisher is created. Add per-channel and global caps so
  remote peers cannot create unbounded pubsub channel state.
- Evidence test:
  - `cargo test early_remote_publisher_join_must_survive_late_local_subscriber_creation -- --nocapture`
  - Failure summary: after an inbound `PublisherJoined` from `PeerId(2)` arrives
    before local channel creation, a later `SubscriberCreated` creates the
    channel with no retained `remote_publishers` entry and emits no
    `SubscriberEvent::PeerJoined(PeerSrc::Remote(PeerId(2)))`.

### ISSUE-189: Inbound handshake accepts a remote peer claiming the local peer id

- Category: security, peer admission, identity validation
- Score: 72/100
- Reviewer: `Zeno the 3rd`, confirmed after `Pauli the 3rd` discovery.
- Affected code:
  - `src/peer.rs`: inbound `run_connection` reads `ConnectReq` and verifies the
    request against peer-controlled `req.from` and `req.to`.
  - `src/peer.rs`: the inbound branch rejects only `req.to != local_id`; it
    never rejects `req.from == local_id`.
  - `src/peer.rs`: accepted inbound setup uses `req.from` as `to_id`, registers
    a `PeerConnectionAlias`, and emits `MainEvent::PeerConnected`.
  - `src/lib.rs`: `MainEvent::PeerConnected(conn, local_id, ...)` can then
    install neighbour/router state for the node's own identity.
- Impact: any peer with the shared key can open an inbound connection and send a
  fresh valid `ConnectReq { from: local_id, to: local_id, auth }`. The receiver
  accepts it because verification is asked to validate exactly those claimed
  ids, creating self-neighbour state, misleading public `PeerConnected` events,
  metrics/control noise, and unstable route or shutdown behavior. This is
  distinct from ISSUE-112, which covers the local `connect()` API self-dialing;
  ISSUE-005/006/103, which cover self identity injected through
  discovery/router/seed topology; ISSUE-113/114 duplicate connection races; and
  ISSUE-146/176 replayable handshake tokens.
- Minimal fix proposal: reject inbound `ConnectReq` when `req.from == local_id`
  before creating a successful `ConnectRes`, registering the alias, or emitting
  `PeerConnected`. Add the symmetric outbound guard if a response would bind
  the remote identity to `local_id`.
- Evidence test:
  - `cargo test inbound_handshake_must_reject_peer_claiming_local_id -- --nocapture`
  - Failure summary: a raw client sends a valid shared-key `ConnectReq` with
    `from == to == PeerId(1)` to a node whose local id is `PeerId(1)`;
    current code returns `ConnectRes { result: Ok(_) }`, so the test fails
    because the inbound handshake was accepted instead of rejected.

### ISSUE-190: Duplicate route-sync destinations silently keep the last metric

- Category: correctness, route stability, malformed-input handling
- Score: 43/100
- Reviewer: `Epicurus the 3rd`, confirmed by failing evidence.
- Affected code:
  - `src/router.rs`: `RouterTableSync` stores peer-supplied routes as
    `Vec<(PeerId, PathMetric)>`, so one sync can contain the same destination
    peer more than once.
  - `src/router.rs`: `RouterTable::apply_sync` converts that vector with
    `BTreeMap::<PeerId, PathMetric>::from_iter(sync.0)`, which silently keeps
    the last metric for duplicate peer ids.
  - `src/peer/peer_internal.rs` and `src/lib.rs`: authenticated peers can
    deliver raw `PeerMessage::Sync` route vectors to this path.
- Impact: a malformed or malicious peer can send two route rows for the same
  destination in one sync and choose which metric wins by ordering the
  duplicates. The receiver accepts the last row as authoritative before metric
  composition and best-path selection, so route state becomes order-dependent
  and can be biased toward an attacker-controlled low metric. This is distinct
  from ISSUE-010, which covers excessive route-sync entry count; ISSUE-033 and
  ISSUE-044, which cover metric arithmetic/overflow; ISSUE-003 and ISSUE-160,
  which cover best-path instability after accepted route state; and
  ISSUE-006/007/008, which cover local-id, over-hop, and split-horizon
  filtering.
- Minimal fix proposal: validate `RouterTableSync` before converting it into a
  `BTreeMap`. If a destination `PeerId` appears more than once, reject the whole
  sync or skip all duplicate rows, and preferably return `Result` from
  `apply_sync` so malformed syncs are logged and ignored without partial route
  mutation.
- Evidence test:
  - `cargo test route_sync_must_reject_duplicate_peer_entries -- --nocapture`
  - Failure summary: a sync from `ConnectionId(1)` contains two rows for
    `PeerId(9)`, first with RTT `500` and then RTT `1`. Current code keeps the
    last row and installs `Some((ConnectionId(1), PathMetric { relay_hops: 1,
    rtt_ms: 11 }))`; expected duplicate destination rows to be rejected so no
    route to `PeerId(9)` is installed.

### ISSUE-191: README getting-started public API example does not compile

- Category: documentation correctness, public API usability
- Score: 18/100
- Reviewer: `Halley the 3rd`, confirmed by failing compile-test evidence.
- Affected code:
  - `README.md`: the setup snippet stores
    `P2pNetwork::new(P2pNetworkConfig { ... }).await` in `network` without
    handling the returned `anyhow::Result`.
  - `README.md`: the following service snippet calls
    `network.create_service(1.into())`, but `create_service` exists on mutable
    `P2pNetwork`, not on `Result<P2pNetwork<_>, anyhow::Error>`.
  - `README.md`: the setup snippet also derives a `PeerId` from an address-like
    string literal, which is misleading for the public `PeerId(u64)` API even
    if type inference can hide it until runtime.
- Impact: a new user following the documented getting-started path cannot
  compile the public API example as written. This blocks onboarding and can
  produce wrong cargo guidance around `P2pNetwork::new` result handling and
  service creation mutability. This is distinct from ISSUE-030/052/054 runtime
  service API validation, ISSUE-112/177 connect-address behavior, and ISSUE-181
  advertise-address validation.
- Minimal fix proposal: update the README snippet to use a numeric peer id,
  bind `let mut network = P2pNetwork::new(...).await?;`, and make the snippet
  part of a doctest, example, or focused compile test so it cannot drift again.
- Evidence test:
  - `cargo test readme_getting_started_snippet_must_compile -- --nocapture`
  - Failure summary: the test runs `cargo check --example
    readme_getting_started` against a mirror of the README setup snippet.
    Current compilation fails with `no method named create_service found for
    enum Result<T, E>` because the snippet calls `create_service` on the
    unhandled `Result<P2pNetwork<SharedKeyHandshake>, anyhow::Error>`.

### ISSUE-192: Duplicate discovery-sync peers silently keep the last address

- Category: correctness, route/discovery stability, malformed-input handling
- Score: 39/100
- Reviewer: `Arendt the 3rd`, confirmed by independent forked review and
  failing test evidence.
- Affected code:
  - `src/discovery.rs`: `PeerDiscoverySync(Vec<(PeerId, u64, NetworkAddress)>)`
    allows repeated peer ids in one discovery sync message.
  - `src/discovery.rs`: `PeerDiscovery::apply_sync` iterates rows and calls
    `self.remotes.insert(peer, (last_updated, address))` for every live row, so
    the last duplicate in a single malformed sync wins regardless of timestamp.
- Impact: a malformed or malicious peer can send two address rows for the same
  discovered peer in one sync and make the receiver retain the attacker-chosen
  last address. This makes discovery state order-dependent, can replace a
  fresher address with a stale one inside one packet, and adds dial/route churn
  under bad-network conditions. This is distinct from ISSUE-092, which covers a
  later stale sync overwriting already-stored discovery state; ISSUE-055, which
  covers configured seed ids; and ISSUE-190, which covers duplicate route-sync
  destinations rather than discovery address rows.
- Minimal fix proposal: validate `PeerDiscoverySync` for duplicate `PeerId`
  rows before mutating `remotes` and reject the whole malformed sync. A slightly
  broader fix can pre-collapse duplicate rows by keeping only the highest
  `last_updated`, then apply the existing local/seed/stopped filters.
- Evidence test:
  - `cargo test discovery_sync_must_reject_duplicate_peer_entries -- --nocapture`
  - Failure summary: a single sync contains `PeerId(7)` twice, first with fresh
    `7@127.0.0.1:9001` at timestamp 200 and then stale
    `7@127.0.0.1:9000` at timestamp 100. Current discovery remotes become
    `[7@127.0.0.1:9000]`; expected rejection or newest-timestamp resolution
    leaving `[7@127.0.0.1:9001]`.

### ISSUE-193: Connection teardown emits RTT as both gauge and counter

- Category: correctness, observability stability, lifecycle cleanup
- Score: 31/100
- Reviewer: `Copernicus the 3rd`, confirmed by independent forked review and
  failing test evidence.
- Affected code:
  - `src/peer/peer_internal.rs`: live connection ticks emit
    `P2P_CONNECTION_RTT` with `gauge!(...).set(metrics.rtt as f64)`.
  - `src/stats.rs`: `P2P_CONNECTION_RTT` is described as a gauge.
  - `src/peer.rs`: connection teardown first resets
    `P2P_CONNECTION_RTT` as a gauge, then emits the same metric name with
    `counter!(P2P_CONNECTION_RTT).absolute(0)`.
- Impact: exporters or recorders that enforce one metric kind per metric name
  can observe `p2p_connection_rtt` registered as both a gauge and a counter.
  That can drop the reset, reject the metric family, or corrupt dashboards
  after disconnects. This is distinct from ISSUE-061/062 metrics-service forged
  protocol messages and ISSUE-064/068 stale or mismatched `PeerStats` state;
  this issue is the real connection teardown instrumentation path using the
  wrong metric kind.
- Minimal fix proposal: remove the teardown
  `counter!(P2P_CONNECTION_RTT).absolute(0)` call, or change it to the existing
  `gauge!(P2P_CONNECTION_RTT).set(0)` reset only. Keep uptime and byte/loss
  metrics on counter names.
- Evidence test:
  - `cargo test connection_teardown_must_not_emit_rtt_as_counter -- --nocapture`
  - Failure summary: the test installs a recorder that records metric kind per
    key, emits `p2p_connection_rtt` as a gauge like the live tick path, then
    exercises the teardown reset sequence. The recorder observes the same RTT
    key as both gauge and counter, so the test fails with the assertion that
    teardown must not emit RTT as a counter.

### ISSUE-194: Inbound handshake accepts arbitrary third-party peer-id claims

- Category: security, peer admission, identity validation
- Score: 88/100
- Reviewer: `Confucius the 3rd`, confirmed after `Euclid the 3rd` discovery.
- Affected code:
  - `src/secure.rs`: `SharedKeyHandshake` authenticates the caller-supplied
    `(from, to, timestamp, role)` tuple with the cluster shared key, not a
    per-peer credential.
  - `src/peer.rs`: inbound `run_connection` verifies `req.auth` against
    peer-controlled `req.from` and `req.to`, then accepts `req.from` as the
    authenticated `to_id`.
  - `src/peer.rs`: after accepting the claim, the peer task registers a
    `PeerConnectionAlias` and emits `MainEvent::PeerConnected(conn_id,
    req.from, rtt_ms)`.
- Impact: any node with the shared key can open an inbound connection and claim
  to be any third-party peer id, for example `PeerId(99)`, without replaying an
  old token or owning that identity. That poisons the authenticated connection
  identity before application messages are processed, causing neighbour,
  router, metrics, stream, and shutdown paths to treat the attacker as the
  claimed peer. This is distinct from ISSUE-014/015/018, which spoof
  application-message source fields after admission; ISSUE-146/176, which cover
  replayable handshake tokens; and ISSUE-189, which is the narrower
  `req.from == local_id` self-identity admission case.
- Minimal fix proposal: do not treat inbound `req.from` as authoritative merely
  because it is signed by the cluster-wide shared key. Reject unauthorized
  claimed peer ids before sending `ConnectRes { result: Ok(_) }`, registering
  aliases, or emitting `PeerConnected`; the smallest immediate mitigation is
  an expected-peer/admission check for inbound claims plus the existing self-id
  rejection, with the stronger fix being per-peer credentials or another
  identity proof that binds the transport peer to the claimed `PeerId`.
- Evidence test:
  - `cargo test inbound_handshake_must_reject_peer_claiming_third_party_id -- --nocapture`
  - Failure summary: a raw client sends a fresh valid shared-key `ConnectReq`
    with `from == PeerId(99)` and `to == PeerId(1)` to a node whose local id is
    `PeerId(1)`; current code returns `ConnectRes { result: Ok(_) }`, so the
    test fails because the claimed third-party id is admitted as the
    authenticated connection identity.

### ISSUE-195: Connection teardown resets monotonic counters to zero

- Category: correctness, observability stability, lifecycle cleanup
- Score: 42/100
- Reviewer: `Dalton the 3rd`, confirmed after `Feynman the 3rd` discovery.
- Affected code:
  - `src/stats.rs`: connection uptime, sent bytes, received bytes, lost bytes,
    lost packets, and congestion events are described as counters.
  - `src/peer/peer_internal.rs`: live connection ticks emit those metrics with
    `counter!(...).absolute(metrics...)`.
  - `src/peer.rs`: disconnect teardown emits `counter!(...).absolute(0)` for
    the same monotonic counter metric names and label set.
- Impact: metrics exporters or recorders that expect counters to be monotonic
  can observe a connection counter advance to a positive value and then
  decrease to zero during teardown. Under churn, this can make byte/loss/uptime
  dashboards and alerting noisy, rejected, or interpreted as a process restart.
  This is distinct from ISSUE-193, which covers `P2P_CONNECTION_RTT` being
  emitted as both gauge and counter; ISSUE-061/062 forged metrics-service
  messages; ISSUE-064/068 stale or mismatched `PeerStats` state; and
  ISSUE-128/129 service shutdown panics.
- Minimal fix proposal: remove teardown `counter!(...).absolute(0)` resets for
  monotonic connection counters in `src/peer.rs`. Keep gauge cleanup for live
  connection count and RTT. If zero-on-teardown semantics are required, model
  those values as gauges or include a connection-lifetime label so each
  connection creates a distinct time series.
- Evidence test:
  - `cargo test connection_teardown_must_not_reset_monotonic_counters -- --nocapture`
  - Failure summary: a test recorder observes positive absolute samples for
    `P2P_CONNECTION_UPTIME`, sent/received bytes, lost bytes/packets, and
    congestion counters, then observes teardown `absolute(0)` samples for the
    same counter names. The test fails because those monotonic counters
    decrease.

### ISSUE-196: Replicated-KV local mutations build an unbounded outbound event queue

- Category: high-load stability, resource bounds, backpressure
- Score: 47/100
- Reviewer: `Averroes the 3rd`, confirmed after `Boyle the 3rd` discovery.
- Affected code:
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvStore` retains
    outbound `Event` values in an unbounded `VecDeque`.
  - `src/service/replicate_kv_service.rs`: `ReplicatedKvStore::set` and `del`
    drain all local-store outputs into `self.outs` without capacity checks or
    backpressure.
  - `src/service/replicate_kv_service/local_storage.rs`: each local `set`
    queues both a broadcast `Changed` event and a local `KvEvent::Set`.
- Impact: a local high-load caller can call `ReplicatedKvService::set()` or
  `del()` faster than `recv()` drains and transmits events. The store retains
  every generated net/local event, so memory grows linearly with mutation rate
  under stalled polling, slow network sends, or bursty workloads. This is
  distinct from ISSUE-045 remote-store growth from many remote identities;
  ISSUE-046/131 malicious inbound response batches; ISSUE-096 serialization
  panic while draining events; ISSUE-119/120 service ingress drops;
  ISSUE-123/124/126 pubsub queues; and ISSUE-164 route/discovery sync drops.
- Minimal fix proposal: add a bounded pending-event capacity to
  `ReplicatedKvStore::outs`. On overflow, coalesce superseded local changes by
  key/version and drop redundant local `KvEvent`s where safe; otherwise make
  `set` and `del` return explicit backpressure errors instead of accepting
  unlimited writes.
- Evidence test:
  - `cargo test replicated_kv_local_outbound_event_queue_must_be_bounded -- --nocapture`
  - Failure summary: after `1025` local `set()` calls, current code retains
    `2050` events in `ReplicatedKvStore::outs` because each set queues one
    broadcast and one local KV event, so the bounded-queue assertion fails.

### ISSUE-197: Unicast relay can forward packets back to the ingress connection

- Category: correctness, unicast relay stability, route-loop handling
- Score: 64/100
- Reviewer: `Lagrange the 3rd`, confirmed after `Darwin the 3rd` discovery.
- Affected code:
  - `src/peer/peer_internal.rs`: `PeerConnectionInternal::on_msg` handles
    `PeerMessage::Unicast` by asking `ctx.router().action(&dest)` and
    forwarding to the selected `RouteAction::Next(next)`.
  - `src/peer/peer_internal.rs`: the unicast forwarding branch does not reject
    `next == self.conn_id`, so a packet can be sent back over the same
    connection that delivered it.
  - `src/router.rs`: `SharedRouterTable::action` returns only the selected
    next connection and cannot exclude the ingress connection for a specific
    forwarding decision.
- Impact: when route state forms a two-node loop for a destination, an inbound
  ordinary unicast frame can be forwarded back to the peer that sent it. That
  can bounce service traffic until hop/routing churn, queue pressure, or
  connection failure stops it. This is distinct from ISSUE-007 over-hop route
  acceptance, ISSUE-008 control-plane route advertisement back to the learned
  peer, ISSUE-147/164 route-sync delivery pressure, and ISSUE-180 stream setup
  relay loops; this issue is the ordinary unicast data-plane forwarding guard.
- Minimal fix proposal: pass or use the ingress `ConnectionId` in the unicast
  forwarding path. If `RouteAction::Next(next) == self.conn_id`, drop or reject
  the packet and log a route-loop warning instead of forwarding back to the
  sender. Longer term, add a shared relay guard for unicast and stream setup,
  plus TTL or visited-path loop control.
- Evidence test:
  - `cargo test unicast_relay_must_not_forward_back_to_ingress_peer -- --nocapture`
  - Failure summary: route state learned from the ingress peer makes
    `ctx.router().action(&PeerId(99))` return
    `Some(RouteAction::Next(ingress_conn))`, proving that the current unicast
    forwarding branch would send the packet back over the ingress connection.

### ISSUE-198: `try_send_broadcast` silently loses all copies under peer queue pressure

- Category: high-load stability, broadcast reliability, API backpressure
- Score: 54/100
- Reviewer: `Dewey the 3rd`, confirmed after `Goodall the 3rd` discovery.
- Affected code:
  - `src/ctx.rs`: `SharedCtx::try_send_broadcast` creates and marks a broadcast
    id as seen, iterates peer aliases, calls `conn_alias.try_send(...)`, and
    logs each failure without returning any delivery result.
  - `src/service.rs`: `P2pService::try_send_broadcast` and
    `P2pServiceRequester::try_send_broadcast` expose this as a public
    success-shaped `()` API.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::try_send` delegates to a
    bounded peer-control `try_send`, so a full peer queue rejects immediately.
- Impact: under high load or a stalled peer task, every connected peer queue
  can reject an outbound broadcast while the caller receives no error and no
  copy is preserved for retry. This is distinct from ISSUE-049, which covers
  awaited `send_broadcast` blocking on one congested peer queue; ISSUE-119/120,
  which cover inbound local service queue drops; ISSUE-164, which covers
  maintenance route/discovery sync drops; ISSUE-163/178, which cover pubsub RPC
  destination accounting; and ISSUE-196, which covers replicated-KV outbound
  queue growth.
- Minimal fix proposal: change `SharedCtx::try_send_broadcast` and public
  wrappers to return a delivery result such as `anyhow::Result<usize>` or a
  small enum. Count successful peer-queue admissions; if zero peers accept the
  broadcast, return an error. Keep per-peer logging as secondary diagnostics.
- Evidence test:
  - `cargo test try_send_broadcast_must_report_when_all_peer_queues_reject -- --nocapture`
  - Failure summary: two registered peer-control queues are prefilled, so
    `ctx.try_send_broadcast(...)` enqueues zero `PeerMessage::Broadcast` copies
    while returning no error. The test fails at `src/peer.rs:870` because no
    queue preserves the broadcast and the API cannot report the total fanout
    failure.

### ISSUE-199: `send_broadcast` silently succeeds when every peer send fails

- Category: high-load stability, broadcast reliability, API error reporting
- Score: 52/100
- Reviewer: `Maxwell the 3rd`, confirmed after `Chandrasekhar the 3rd`
  discovery.
- Affected code:
  - `src/ctx.rs`: `SharedCtx::send_broadcast` awaits each
    `conn_alias.send(...)` result, but calls `print_on_err(...)` and returns
    `()` even if every peer send fails.
  - `src/service.rs`: `P2pService::send_broadcast` and
    `P2pServiceRequester::send_broadcast` expose this success-shaped `()` API.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` can return an error
    when the peer-control receiver is closed, but broadcast fanout discards
    that error.
- Impact: when all connected peer control channels are already closed or
  otherwise immediately reject awaited sends, an outbound broadcast is delivered
  to no peer while the caller has no failure signal. This is distinct from
  ISSUE-049, which covers awaited `send_broadcast` blocking behind a full peer
  queue; ISSUE-198, which covers the nonblocking `try_send_broadcast` path
  losing all copies under queue pressure; ISSUE-119/120, which cover inbound
  local service delivery drops; and ISSUE-163/178, which cover pubsub RPC
  destination accounting.
- Minimal fix proposal: change `SharedCtx::send_broadcast` and public wrappers
  to return a delivery result such as `anyhow::Result<usize>` or a small enum.
  Count successful peer-control sends; if zero peers accept the broadcast,
  return an error. Keep per-peer logging as diagnostics.
- Evidence test:
  - `cargo test send_broadcast_must_report_when_all_peer_channels_are_closed -- --nocapture`
  - Failure summary: two registered peer-control receiver halves are dropped,
    so every `PeerConnectionAlias::send(...).await` fails immediately. The test
    fails at `src/peer.rs:858` because `ctx.send_broadcast(...).await` returns
    unit `()`, leaving callers unable to observe total fanout failure.

### ISSUE-200: metrics collector duplicates scan broadcasts behind hidden backpressure

- Category: high-load stability, collector backpressure, duplicate work
- Score: 58/100
- Reviewer: `Bohr the 3rd`, confirmed after `Meitner the 3rd` discovery.
- Affected code:
  - `src/service/metrics_service.rs`: each collector tick pushes local metrics
    and then `tokio::spawn`s a detached task that awaits
    `requester.send_broadcast(Message::Scan)`.
  - `src/ctx.rs`: `SharedCtx::send_broadcast` awaits bounded peer-control
    sends sequentially.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits the bounded
    peer-control channel.
- Impact: when a peer-control queue is full, the metrics collector loop remains
  responsive because the broadcast is hidden in a detached task. Subsequent
  ticks spawn more blocked scan tasks for the same periodic work, so once
  pressure clears the peer can receive duplicate stale scans. This is distinct
  from ISSUE-049/050, which cover callers directly blocking on awaited sends,
  and from ISSUE-198/199, which cover broadcast delivery/result reporting.
- Minimal fix proposal: add explicit in-flight scan state to `MetricsService`
  and skip or coalesce collector ticks while the previous scan broadcast is
  still pending. A small fix is a `JoinHandle` or flag tracked by the service;
  a longer-term fix should use a bounded, observable broadcast API so
  coalescing can react to delivery status.
- Evidence test:
  - `cargo test metrics_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured -- --nocapture`
  - Failure summary: the test keeps a synthetic peer-control queue full across
    eight 1 ms collector ticks, drains the filler item, and observes more than
    one queued `PeerMessage::Broadcast` scan. It fails at `src/peer.rs:895`
    with `got 2`, proving scans accumulate while a prior broadcast remains
    backpressured.

### ISSUE-201: visualization collector duplicates scan broadcasts behind hidden backpressure

- Category: high-load stability, visualization collector backpressure,
  duplicate work
- Score: 57/100
- Reviewer: `Plato the 3rd`, confirmed after `Tesla the 3rd` discovery.
- Affected code:
  - `src/service/visualization_service.rs`: each collection tick optionally
    emits a local topology update and then `tokio::spawn`s a detached task that
    awaits `requester.send_broadcast(Message::Scan)`.
  - `src/ctx.rs`: `SharedCtx::send_broadcast` awaits bounded peer-control
    sends sequentially.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits the bounded
    peer-control channel.
- Impact: when a peer-control queue is full, the visualization service loop
  remains responsive by hiding the blocked broadcast in a detached task.
  Subsequent collection ticks spawn more blocked topology scan tasks; when
  pressure clears, peers can receive duplicate stale scans. This is distinct
  from ISSUE-200, which covers `MetricsService`; ISSUE-049/050, which cover
  direct caller blocking; and ISSUE-198/199, which cover broadcast
  delivery/result reporting.
- Minimal fix proposal: add explicit in-flight scan state to
  `VisualizationService` and skip or coalesce collector ticks while the prior
  scan broadcast is pending. The smallest local fix is a pending `JoinHandle`
  or boolean flag cleared when the send task completes; longer term, use an
  observable bounded broadcast API.
- Evidence test:
  - `cargo test visualization_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured -- --nocapture`
  - Failure summary: the test keeps a synthetic peer-control queue full across
    eight 1 ms visualization collection ticks, drains the filler item, and
    observes more than one queued `PeerMessage::Broadcast` scan. It fails at
    `src/peer.rs:930` with `got 2`, proving visualization scans accumulate
    while a prior broadcast remains backpressured.

### ISSUE-202: metrics scan responses are dropped under peer-control backpressure

- Category: high-load stability, metrics response reliability, API
  backpressure
- Score: 55/100
- Reviewer: `Ramanujan the 3rd`, confirmed after `Aquinas the 3rd` discovery.
- Affected code:
  - `src/service/metrics_service.rs`: when a `Message::Scan` arrives, the
    service gathers metrics and spawns a detached response task.
  - `src/service/metrics_service.rs`: that task calls
    `requester.try_send_unicast(...)` and only logs errors.
  - `src/ctx.rs`: `SharedCtx::try_send_unicast` delegates to the next-hop
    peer alias and returns an error immediately if the bounded peer-control
    queue is full.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::try_send` uses bounded
    channel `try_send`.
- Impact: after a metrics `Scan` is accepted, a transiently full next-hop
  peer-control queue drops the `Info` response immediately. The sender gets no
  retry, backpressure, or observable failure from the metrics service. This is
  distinct from ISSUE-050, which covers direct awaited `send_unicast`
  blocking; ISSUE-119/120, which cover local service ingress queue drops;
  ISSUE-198, which covers broadcast `try_send_broadcast`; ISSUE-200/201, which
  cover duplicate collector scan scheduling; and ISSUE-078, which covers
  unauthorized metrics disclosure.
- Minimal fix proposal: do not use fire-and-forget `try_send_unicast` for
  metrics `Scan` responses. Use an awaited send with timeout, or keep a small
  bounded retry/coalescing state for pending metrics responses. Surface
  delivery failure so the service can retry, account for the drop, or apply
  caller-visible backpressure.
- Evidence test:
  - `cargo test metrics_scan_response_must_not_be_dropped_when_peer_control_queue_is_full -- --nocapture`
  - Failure summary: the test injects a metrics `Scan` while the selected
    next-hop peer-control queue is full, yields while the queue remains full,
    then drains the filler item. No `PeerMessage::Unicast` metrics response is
    observed afterward, and the test fails at `src/peer.rs:977`.

### ISSUE-203: visualization scan responses accumulate behind peer-control backpressure

- Category: high-load stability, visualization response reliability, duplicate
  work
- Score: 56/100
- Reviewer: `Sartre the 3rd`, confirmed after `Boole the 3rd` discovery.
- Affected code:
  - `src/service/visualization_service.rs`: when a `Message::Scan` arrives,
    the service gathers neighbour topology from the requester router.
  - `src/service/visualization_service.rs`: the response path spawns a
    detached task for every scan and awaits `requester.send_unicast(...)`.
  - `src/ctx.rs`: `SharedCtx::send_unicast` awaits the selected next-hop
    peer alias.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits the bounded
    peer-control channel.
- Impact: repeated visualization `Scan` messages can create multiple detached
  response tasks while the selected peer-control queue is full. When pressure
  clears, stale duplicate `Info` responses are released. This is distinct from
  ISSUE-050, which covers direct awaited unicast blocking; ISSUE-079, which
  covers unauthorized topology disclosure; ISSUE-119/120, which cover local
  service ingress drops; ISSUE-200/201, which cover periodic collector scan
  broadcast scheduling; and ISSUE-202, where metrics uses nonblocking
  `try_send_unicast` and drops the response instead of accumulating tasks.
- Minimal fix proposal: do not spawn unbounded detached response tasks for
  visualization `Scan`. Keep bounded in-flight response state per requester and
  coalesce duplicates, or await/send with a timeout and explicit failure
  accounting.
- Evidence test:
  - `cargo test visualization_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  - Failure summary: the test injects eight visualization `Scan` messages while
    the selected next-hop peer-control queue is full, yields while response
    tasks are blocked, then drains the filler item. More than one queued
    `PeerMessage::Unicast` response appears, and the test fails at
    `src/peer.rs:1021` with `got 2`.

### ISSUE-204: metrics scan responses accumulate behind peer-control backpressure

- Category: high-load stability, metrics response reliability, duplicate work
- Score: 56/100
- Reviewer: `Anscombe the 4th`, confirmed.
- Affected code:
  - `src/service/metrics_service.rs`: when a `Message::Scan` arrives, the
    service gathers metrics and spawns a detached task for every response.
  - `src/service/metrics_service.rs`: the response task awaits
    `requester.send_unicast(...)` with a timeout, but no per-requester
    in-flight state suppresses duplicate pending replies.
  - `src/ctx.rs`: `SharedCtx::send_unicast` awaits the selected next-hop
    peer alias.
  - `src/peer/peer_alias.rs`: `PeerConnectionAlias::send` awaits the bounded
    peer-control channel.
- Impact: repeated metrics `Scan` messages can create multiple detached
  response tasks while the selected peer-control queue is full. When pressure
  clears, stale duplicate `Info` responses are released. This is distinct from
  ISSUE-202, which covered the older dropped-response path; ISSUE-203, which
  covers the same accumulation class in `VisualizationService`; ISSUE-050,
  which covers direct awaited unicast blocking; ISSUE-078, which covers
  unauthorized metrics disclosure; and ISSUE-200/201, which cover periodic
  collector scan broadcast scheduling.
- Minimal fix proposal: add bounded in-flight response state for metrics scan
  replies, preferably keyed by requester peer. While a response to that peer is
  pending, skip or coalesce additional `Scan` replies; clear the marker when
  the send task completes or times out.
- Evidence test:
  - `cargo test metrics_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  - Failure summary: the test injects eight metrics `Scan` messages while the
    selected next-hop peer-control queue is full, yields while response tasks
    are blocked, then drains the filler item. More than one queued
    `PeerMessage::Unicast` response appears, and the test fails at
    `src/peer.rs:1092` with `got 2`.

## No-New-Issue Audit Cycles

### Cycle after ISSUE-204 no-new cycle 323: valid churn shutdown duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Ramanujan the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=323 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=323, nodes=8, steps=4800`.
- Evidence summary:
  - exit status 101; log had 53 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - two hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - invalid-service, stale-route, channel-closed, and endpoint-driver-dropped
    signatures were absent.
  - network churn context included seven `connection lost`, nine
    `closed by peer`, and three `aborted by peer` markers.
  - four `forward peer stopped over peer alias` / `no available capacity`
    markers were reviewed as too small and context-only for ISSUE-170 in this
    cycle; there was no sustained storm, channel-closed amplification, hang,
    leak, or independent failure mode.
- Duplicate mapping: ISSUE-139 for the shutdown/closed-main reporting panic.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted closed-main shutdown path.
- Smallest fix proposal: no summary fix change; keep ISSUE-139 fix proposal to
  replace shutdown-path `expect("should send to main")` calls at
  `src/peer.rs:89/92/130/133` with graceful closed-channel handling.

### Cycle after ISSUE-204 no-new cycle 322: steady valid clean pass with endpoint teardown logs

- Result: no accepted non-duplicate issue.
- Reviewer: `Boole the 7th`, forked subagent review, confirmed clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=322 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=5600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed`, `0 failed`.
- Evidence summary:
  - exit status 0; log had 11 lines; test result was `ok. 1 passed; 0 failed;
    0 ignored; 0 measured; 289 filtered out; finished in 35.94s`.
  - invalid-service, stale-route, shutdown-send, PeerStopped storm,
    channel-closed, internal-channel-error, and panic signatures were absent.
  - two `endpoint driver future was dropped` markers and one `connection lost`
    marker were reviewed as connection teardown/lifecycle noise because the
    run completed successfully and no failing assertion, panic, hang, leak, or
    data-loss proof followed.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause.
- Smallest fix proposal: none for this cycle; continue monitoring endpoint
  teardown logs for a reproducible behavioral impact or failing test.

### Cycle after ISSUE-204 no-new cycle 321: sanitized churn shutdown storm duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Darwin the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=321 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=321, nodes=8, steps=4800`.
- Evidence summary:
  - exit status 101; log had 8,867 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - eight hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - `forward peer stopped over peer alias` appeared 8,780 times, including
    8,533 `no available capacity` reports and 262 `channel closed` reports.
  - invalid-service and stale-route panic signatures were absent; network
    churn context included 13 `connection lost`, 11 `closed by peer`, and one
    `aborted by peer` marker.
  - one `endpoint driver future was dropped` marker was reviewed as lifecycle
    fallout after the task panic/storm context, with no independent failing
    assertion, panic origin, reproducible hang, leak, or data-loss proof.
- Duplicate mapping: ISSUE-139 for the shutdown/closed-main reporting panic;
  ISSUE-170 for the PeerStopped forwarding/capacity storm context.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted closed-main shutdown path and the storm
  volume as already accepted stopped-forwarding pressure behavior.
- Smallest fix proposal: no summary fix change; keep ISSUE-139 fix proposal to
  replace shutdown-path `expect("should send to main")` calls at
  `src/peer.rs:89/92/130/133` with graceful closed-channel handling, and keep
  ISSUE-170 fix proposal to dedupe/coalesce stopped forwarding with bounded
  retry/backpressure and TTL/tombstones.

### Cycle after ISSUE-204 no-new cycle 320: churn stale-sync duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Maxwell the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=320 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=320, nodes=8, steps=4800`.
- Evidence summary:
  - exit status 101; log had 29 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - one hard stale-route panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - invalid-service, shutdown-send, and PeerStopped capacity-storm signatures
    were absent.
  - network churn context included six `connection lost` markers, one
    `closed by peer` marker, and one `aborted by peer` marker.
- Duplicate mapping: ISSUE-063 for the stale-sync panic.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted stale-sync/direct-route race.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  guard stale route-sync application when the direct metric has already gone
  missing.

### Cycle after ISSUE-204 no-new cycle 319: valid action stale-sync storm duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Kuhn the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=319 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=319, nodes=8, steps=4800`.
- Evidence summary:
  - exit status 101; log had 8,115 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard stale-route panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - `forward peer stopped over peer alias` appeared 8,077 times, including
    7,158 `no available capacity` reports, 937 `channel closed` reports, and
    2 `broadcast data over peer alias` reports.
  - invalid-service and shutdown-send panic signatures were absent, as were
    connection-lost, closed-by-peer, and aborted-by-peer markers.
- Duplicate mapping: ISSUE-063 for the stale-sync panic; ISSUE-170 for the
  PeerStopped forwarding/capacity storm context.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted stale-sync/direct-route race, with the storm
  logs matching accepted stopped-forwarding pressure symptoms.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  guard stale route-sync application when the direct metric has already gone
  missing, and keep ISSUE-170 fix proposal to dedupe/coalesce stopped
  forwarding with bounded retry/backpressure and TTL/tombstones.

### Cycle after ISSUE-204 no-new cycle 318: steady valid clean pass

- Result: no accepted non-duplicate issue.
- Reviewer: `Franklin the 7th`, forked subagent review, confirmed clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=318 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed`, `0 failed`.
- Evidence summary:
  - exit status 0; log had 9 lines; test result was `ok. 1 passed; 0 failed;
    0 ignored; 0 measured; 289 filtered out; finished in 31.08s`.
  - invalid-service, stale-route, shutdown-send, PeerStopped storm,
    connection-lifecycle, and endpoint-driver-dropped signatures were absent.
  - one `answer open_bi got error internal channel error` log was reviewed as
    transient connection/task teardown context because the test completed
    successfully and no panic, failed assertion, leak/stall proof, or
    behavioral impact followed.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause.
- Smallest fix proposal: none for this cycle.

### Cycle after ISSUE-204 no-new cycle 317: sanitized churn shutdown storm duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Lagrange the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=317 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=317, nodes=8, steps=4800`.
- Evidence summary:
  - exit status 101; log had 14,071 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - nine hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - `forward peer stopped over peer alias` appeared 13,976 times, with 14,000
    `no available capacity` markers and 12 `broadcast data over peer alias`
    markers.
  - invalid-service and stale-route panic signatures were absent; network
    churn context included six `connection lost`, ten `closed by peer`, and
    eight `aborted by peer` markers.
- Duplicate mapping: ISSUE-139 for the shutdown/closed-main reporting panic;
  ISSUE-170 for the PeerStopped forwarding/capacity storm context.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted closed-main shutdown path and the storm
  volume as already accepted stopped-forwarding pressure behavior.
- Smallest fix proposal: no summary fix change; keep ISSUE-139 fix proposal to
  replace shutdown-path `expect("should send to main")` calls at
  `src/peer.rs:89/92/130/133` with graceful closed-channel handling, and keep
  ISSUE-170 fix proposal to dedupe/coalesce stopped forwarding with bounded
  retry/backpressure and TTL/tombstones.

### Cycle after ISSUE-204 no-new cycle 316: invalid service-id duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Godel the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=316 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=316, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 23 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard invalid-service panic at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - stale-route, shutdown-send, and PeerStopped capacity-storm signatures were
    absent.
  - minor network context included two `channel closed` markers and one
    `closed by peer` marker.
- Duplicate mapping: ISSUE-053 for the invalid/out-of-range service-id table
  indexing panic.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted invalid service-id dispatch path.
- Smallest fix proposal: no summary fix change; keep ISSUE-053 fix proposal to
  validate service IDs before indexing the context/service table and reject or
  drop values outside `0..256`.

### Cycle after ISSUE-204 no-new cycle 315: sanitized churn shutdown-send duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Kant the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=315 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=315, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 66 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - eight hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - invalid-service, stale-route, and PeerStopped capacity-storm signatures
    were absent.
  - network churn context included ten `closed by peer`, ten `aborted by peer`,
    and one `connection lost` marker.
- Duplicate mapping: ISSUE-139 for the shutdown/closed-main reporting panic.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure as the already accepted closed-main shutdown path.
- Smallest fix proposal: no summary fix change; keep ISSUE-139 fix proposal to
  replace shutdown-path `expect("should send to main")` calls at
  `src/peer.rs:89/92/130/133` with graceful closed-channel handling and
  terminal task exit.

### Cycle after ISSUE-204 no-new cycle 314: churn stale-sync storm duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Plato the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=314 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=314, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 30,787 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - five hard stale-route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - `forward peer stopped over peer alias` appeared 30,733 times, including
    25,450 `no available capacity` reports and 5,295 `channel closed` reports.
  - invalid-service and shutdown-send panic signatures were absent; connection
    lifecycle noise included four `connection lost` markers and six
    `closed by peer` markers.
- Duplicate mapping: ISSUE-063 for the stale-sync panic; ISSUE-170 for the
  PeerStopped forwarding/capacity storm context.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  panic as the already accepted stale-sync/direct-route race, with the storm
  logs matching accepted stopped-forwarding pressure symptoms.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  guard stale route-sync application when the direct metric has already gone
  missing, and keep ISSUE-170 fix proposal to dedupe/coalesce stopped
  forwarding with bounded retry/backpressure and TTL/tombstones.

### Cycle after ISSUE-204 no-new cycle 313: valid churn stale-sync storm duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `McClintock the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=313 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=313, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 10,170 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - four hard stale-route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - `forward peer stopped over peer alias` appeared 10,119 times, including
    7,437 `no available capacity` reports and 2,702 `channel closed` reports.
  - invalid-service and shutdown-send panic signatures were absent; connection
    lifecycle noise included four `connection lost` markers and one
    `closed by peer` marker.
- Duplicate mapping: ISSUE-063 for the stale-sync panic; ISSUE-170 for the
  PeerStopped forwarding/capacity storm context.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  panic as the already accepted stale-sync/direct-route race, with the storm
  logs matching accepted stopped-forwarding pressure symptoms.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  guard stale route-sync application when the direct metric has already gone
  missing, and keep ISSUE-170 fix proposal to dedupe/coalesce stopped
  forwarding with bounded retry/backpressure and TTL/tombstones.

### Cycle after ISSUE-204 no-new cycle 312: broad invalid service shutdown duplicates

- Result: no accepted non-duplicate issue.
- Reviewer: `Popper the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=312 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=312, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 25 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard invalid-service panic at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - one hard shutdown-send panic at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - stale-route and stopped-forwarding/capacity-storm counts were zero.
  - one `channel closed` and one `closed by peer` marker were reviewed as
    lifecycle fallout around the same failing run.
- Duplicate mapping: ISSUE-053 for the invalid-service panic; ISSUE-139 for
  the shutdown-send panic.
- Root-cause summary impact: no new root cause; reviewer classified both hard
  failure signatures as already accepted issues.
- Smallest fix proposal: no summary fix change; keep ISSUE-053 fix proposal to
  bounds-check service IDs before table indexing and ISSUE-139 fix proposal to
  gracefully handle closed peer-to-main reporting channels.

### Cycle after ISSUE-204 no-new cycle 311: valid churn stale sync duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Curie the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=311 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=311, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 20 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - one hard stale-sync route panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - invalid-service-id, shutdown-send, stopped-forwarding, capacity-storm,
    broadcast-alias, path-not-found, channel/connection lifecycle,
    closed-by-peer, aborted-by-peer, and endpoint-driver-dropped counts were
    zero.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure signature as the already accepted stale-sync route panic.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop or no-op stale route sync when the direct metric is gone.

### Cycle after ISSUE-204 no-new cycle 310: long steady valid clean pass

- Result: no accepted issue.
- Reviewer: `Halley the 7th`, forked subagent review, confirmed clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=310 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=5000 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 32.06s`.
  - no panic, failed assertion, invalid-service-id, stale-sync, shutdown-send,
    stopped-forwarding, capacity storm, broadcast-alias, path-not-found,
    channel/connection lifecycle, closed-by-peer, aborted-by-peer, or
    endpoint-driver-dropped evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; reviewer classified the longer
  steady valid-node fuzz run as clean.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 309: broad stale sync shutdown duplicates

- Result: no accepted non-duplicate issue.
- Reviewer: `Cicero the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=309 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=309, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 4156 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard stale-sync route panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - two hard shutdown-send panics at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - invalid-service-id count was zero.
  - the same run also showed duplicate ISSUE-170 pressure markers:
    `forward peer stopped over peer alias` 4114 times, `no available capacity`
    3655 times, `channel closed` 472 times, and `broadcast data over peer
    alias` 4 times.
  - two `connection lost` and one `endpoint driver future was dropped` markers
    were reviewed as lifecycle fallout around the same failing run.
- Duplicate mapping: ISSUE-063 for the stale-sync route panic; ISSUE-139 for
  the shutdown-send panics; ISSUE-170 for secondary stopped-forwarding/capacity
  storm noise.
- Root-cause summary impact: no new root cause; reviewer classified all hard
  failure signatures and secondary pressure as already accepted issues.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop stale route sync when the direct metric is gone, ISSUE-139 fix proposal
  to gracefully handle closed peer-to-main reporting channels, and ISSUE-170
  fix proposal to dedupe/coalesce stopped-peer forwarding.

### Cycle after ISSUE-204 no-new cycle 308: valid stale sync duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Dalton the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=308 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=308, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 24 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - two hard stale-sync route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - invalid-service-id, shutdown-send, stopped-forwarding, capacity-storm,
    broadcast-alias, path-not-found, channel-closed, closed-by-peer,
    aborted-by-peer, and endpoint-driver-dropped counts were zero.
  - two `connection lost` markers were reviewed as lifecycle fallout around
    the same failing run.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure signature as the already accepted stale-sync route panic.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop or no-op stale route sync when the direct metric is gone.

### Cycle after ISSUE-204 no-new cycle 307: sanitized churn stale sync storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Hegel the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=307 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=307, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 87185 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - two hard stale-sync route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - invalid-service-id and shutdown-send counts were zero.
  - the same run also showed duplicate ISSUE-170 pressure markers:
    `forward peer stopped over peer alias` 86857 times, `no available
    capacity` 85509 times, `channel closed` 1631 times, and `broadcast data
    over peer alias` 74 times.
  - four `connection lost`, nine `closed by peer`, and one
    `endpoint driver future was dropped` markers were reviewed as churn
    lifecycle fallout.
- Duplicate mapping: ISSUE-063 for the stale-sync route panics; ISSUE-170 for
  secondary stopped-forwarding/capacity storm noise.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure signature and secondary storm as already accepted issues.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop stale route sync when the direct metric is gone and ISSUE-170 fix
  proposal to dedupe/coalesce stopped-peer forwarding.

### Cycle after ISSUE-204 no-new cycle 306: steady valid clean pass

- Result: no accepted issue.
- Reviewer: `Mencius the 7th`, forked subagent review, confirmed
  clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=306 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=4200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 27.49s`.
  - no panic, failed assertion, invalid-service-id, stale-sync, shutdown-send,
    stopped-forwarding, capacity storm, broadcast-alias, path-not-found,
    channel/connection lifecycle, closed-by-peer, aborted-by-peer, or
    endpoint-driver-dropped evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; reviewer classified the steady
  valid-node fuzz run as clean.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 305: valid churn stale sync duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Mendel the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=305 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=305, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 8996 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - two hard stale-sync route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - invalid-service-id and shutdown-send counts were zero.
  - the same run also showed duplicate ISSUE-170 pressure markers:
    `forward peer stopped over peer alias` 8950 times, `no available capacity`
    6154 times, and `channel closed` 2796 times.
  - eight `connection lost` and nine `closed by peer` markers were reviewed as
    churn lifecycle fallout.
- Duplicate mapping: ISSUE-063 for the stale-sync route panics; ISSUE-170 for
  secondary stopped-forwarding/capacity storm noise.
- Root-cause summary impact: no new root cause; reviewer classified the hard
  failure signature and secondary pressure as already accepted issues.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop stale route sync when the direct metric is gone and ISSUE-170 fix
  proposal to dedupe/coalesce stopped-peer forwarding.

### Cycle after ISSUE-204 no-new cycle 304: broad invalid service duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Confucius the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=304 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=304, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 23 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard invalid-service panic at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - stale-route, shutdown-send, stopped-forwarding, capacity-storm,
    broadcast-alias, path-not-found, connection-lost, aborted-by-peer, and
    endpoint-driver-dropped counts were zero.
  - two `channel closed` and one `closed by peer` markers were reviewed as
    lifecycle fallout around the same failing run.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; reviewer classified the crash
  as the existing unchecked out-of-range `P2pServiceId(256)` service-table
  index.
- Smallest fix proposal: no summary fix change; keep ISSUE-053 fix proposal to
  bounds-check service IDs before table indexing and drop/reject out-of-range
  messages.

### Cycle after ISSUE-204 no-new cycle 303: valid stale sync and shutdown duplicates

- Result: no accepted non-duplicate issue.
- Reviewer: `Hypatia the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=303 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=303, nodes=8, steps=3600`.
- Evidence summary:
  - exit status 101; log had 4689 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - three hard stale-sync route panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - one hard shutdown-send panic at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - the same run also showed duplicate ISSUE-170 pressure markers:
    `forward peer stopped over peer alias` 4646 times, `no available capacity`
    4192 times, `channel closed` 465 times, and two
    `endpoint driver future was dropped` markers.
  - invalid-service-id and path-not-found counts were zero.
- Duplicate mapping: ISSUE-063 for the stale-sync route panics; ISSUE-139 for
  the shutdown-send panic; ISSUE-170 for secondary stopped-forwarding/capacity
  storm noise.
- Root-cause summary impact: no new root cause; reviewer classified all hard
  failure signatures as already accepted issues.
- Smallest fix proposal: no summary fix change; keep ISSUE-063 fix proposal to
  drop stale route sync when the direct metric is gone, ISSUE-139 fix proposal
  to gracefully handle closed peer-to-main reporting channels, and ISSUE-170
  fix proposal to dedupe/coalesce stopped-peer forwarding.

### Cycle after ISSUE-204 no-new cycle 302: sanitized churn shutdown send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Epicurus the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=302 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the harness reported `seed=302, nodes=8, steps=3600` because the
    test clamps `P2P_FUZZ_NODES` to `2..=8`.
- Evidence summary:
  - exit status 101; log had 29 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - three hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - two `connection lost` and three `aborted by peer` markers were reviewed as
    lifecycle fallout around the same churn/shutdown condition.
  - invalid-service, stale-route, capacity-storm, broadcast-alias,
    path-not-found, channel-closed, closed-by-peer, and endpoint-driver-dropped
    counts were zero.
- Duplicate mapping: ISSUE-139.
- Root-cause summary impact: no new root cause; reviewer classified the crash
  as the existing closed-main shutdown reporting panic.
- Smallest fix proposal: no summary fix change; keep ISSUE-139 fix proposal to
  replace shutdown-path `expect("should send to main")` calls in `src/peer.rs`
  with graceful closed-channel handling.

### Cycle after ISSUE-204 no-new cycle 301: twelve-node steady valid clean pass

- Result: no accepted issue.
- Reviewer: `Gibbs the 7th`, forked subagent review, confirmed clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=301 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=4200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 27.38s`.
  - no panic, failed assertion, invalid-service-id, stale-sync, shutdown-send,
    stopped-forwarding, capacity storm, broadcast-alias, path-not-found,
    channel/connection lifecycle, closed-by-peer, aborted-by-peer, or
    endpoint-driver-dropped evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; reviewer classified the
  twelve-node steady valid-node fuzz run as clean.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 300: broad invalid service duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `James the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=300 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3400 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=300, nodes=8, steps=3400`.
- Evidence summary:
  - exit status 101; log had 6937 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - four hard invalid-service panics at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - `src/router.rs:76`, `should have direct metric`, and `should send to main`
    counts were zero.
  - the same run also showed duplicate ISSUE-170 pressure markers:
    `forward peer stopped over peer alias` 6888 times, `no available capacity`
    3531 times, and `channel closed` 3373 times.
- Duplicate mapping: ISSUE-053 for the failing service-table panic; ISSUE-170
  for secondary stopped-forwarding/capacity storm noise.
- Root-cause summary impact: no new root cause; reviewer classified the crash
  as the existing unchecked out-of-range `P2pServiceId(256)` service-table
  index.
- Smallest fix proposal: no summary fix change; keep ISSUE-053 fix proposal to
  bounds-check service IDs before table indexing and drop/reject out-of-range
  messages.

### Cycle after ISSUE-204 no-new cycle 299: steady valid clean pass

- Result: no accepted issue.
- Reviewer: `Gauss the 7th`, forked subagent review, confirmed clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=299 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 24.35s`.
  - no panic, failed assertion, invalid-service-id, stale-sync, shutdown-send,
    stopped-forwarding, capacity storm, broadcast-alias, path-not-found,
    channel/connection lifecycle, or endpoint-driver-dropped evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; the steady valid-node action
  harness completed cleanly and showed no failed invariant.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 298: sanitized churn incoming shutdown send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hume the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=298 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3400 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=298, nodes=8, steps=3400`.
- Evidence summary:
  - exit status 101; log had 33 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - one hard shutdown-send panic at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - three connection-lost markers, four closed-by-peer markers, and one
    aborted-by-peer marker were reviewed as lifecycle fallout around the same
    shutdown/churn condition.
  - no invalid-service-id, stale-sync, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, or channel-closed evidence.
- Duplicate mapping: ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  shutdown/closed-main reporting evidence for the incoming `accept().await`
  error branch.
- Smallest fix proposal: replace the `expect("should send to main")` sends in
  peer connection error reporting with graceful closed-channel handling; if
  `main_tx.send(...)` fails because the main loop has stopped, return from the
  peer task without panicking, and keep seed `298` as regression evidence for
  the incoming `src/peer.rs:92` path.

### Cycle after ISSUE-204 no-new cycle 297: isolated valid stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Peirce the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=297 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=297, nodes=8, steps=3400`.
- Evidence summary:
  - exit status 101; log had 21 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard stale-sync panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - one closed-by-peer marker was reviewed as teardown fallout from the same
    disconnect/routing race.
  - no invalid-service-id, shutdown-send, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, connection-lost, channel-closed, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync race evidence without stopped-forwarding amplification.
- Smallest fix proposal: in `RouterTable::apply_sync`, replace the
  direct-metric `expect` with guarded stale-sync handling; if
  `self.directs.get(&conn)` is missing, drop or ignore that sync update and
  avoid mutating route state from it, and consider clearing queued sync state
  when a direct route is removed.

### Cycle after ISSUE-204 no-new cycle 296: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Helmholtz the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=296 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=296, nodes=8, steps=3200`.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard invalid-service panic at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed marker and one closed-by-peer marker were reviewed as
    teardown fallout after the background task panic.
  - no stale-sync, shutdown-send, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, connection-lost, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  unchecked inbound service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry service table; reject or drop inbound ids outside `0..256` before
  calling `get_service`, and keep seed `296` as regression evidence for the
  invalid-service broadcast path.

### Cycle after ISSUE-204 no-new cycle 295: steady valid clean pass with lifecycle logs

- Result: no accepted issue.
- Reviewer: `Goodall the 7th`, forked subagent review, confirmed
  clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=295 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 13 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 23.52s`.
  - two connection-lost markers and three endpoint-driver-dropped markers were
    reviewed as non-failing teardown/lifecycle noise.
  - no panic, invalid-service-id, stale-sync, shutdown-send, capacity storm,
    stopped-forwarding, broadcast-alias, path-not-found, channel-closed,
    closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; the steady valid-node action
  harness completed cleanly and showed no failed invariant.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 294: sanitized churn incoming shutdown send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Copernicus the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=294 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=294, nodes=8, steps=3200`.
- Evidence summary:
  - exit status 101; log had 29 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - one hard shutdown-send panic at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - seven connection-lost markers, one closed-by-peer marker, and one
    aborted-by-peer marker were reviewed as lifecycle fallout around the same
    churn/shutdown condition.
  - no invalid-service-id, stale-sync, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, or channel-closed evidence.
- Duplicate mapping: ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  shutdown/closed-main reporting evidence for the incoming `accept().await`
  error branch.
- Smallest fix proposal: replace the `expect("should send to main")` sends in
  peer connection error reporting with graceful closed-channel handling; if
  `main_tx.send(...)` fails because the main loop has stopped, return from the
  peer task without panicking, and keep seed `294` as regression evidence for
  the `src/peer.rs:92` incoming accept path.

### Cycle after ISSUE-204 no-new cycle 293: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Sagan the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=293 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=293, nodes=8, steps=3200`.
- Evidence summary:
  - exit status 101; log had 10,738 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - one hard stale-sync panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 10,707 forwarded-stop markers, including 10,577 `no available capacity`
    markers and 135 `channel closed` markers.
  - one broadcast-over-peer-alias marker was reviewed as insufficient for an
    independent root cause.
  - no invalid-service-id, shutdown-send, path-not-found, connection-lost,
    closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: primary ISSUE-063; secondary amplification ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync race evidence and stopped-peer forwarding storm evidence
  without adding a new issue.
- Smallest fix proposal: for ISSUE-063, replace the `apply_sync` direct-metric
  `expect` with guarded stale-sync handling that drops or ignores sync for a
  missing direct connection; for ISSUE-170, dedupe/coalesce forwarded
  `PeerStopped` events with bounded retry/backpressure behavior and TTL or
  tombstone suppression for cyclic topologies.

### Cycle after ISSUE-204 no-new cycle 292: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Pascal the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=292 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3000 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=292, nodes=8, steps=3000`.
- Evidence summary:
  - exit status 101; log had 49 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure.
  - seven hard invalid-service panics at `src/ctx.rs:34:9`, each with
    `index out of bounds: the len is 256 but the index is 256`.
  - one connection-lost marker, eight channel-closed markers, and six
    closed-by-peer markers were reviewed as teardown fallout after the
    background task panics.
  - no stale-sync, shutdown-send, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; the repeated panics strengthen
  existing unchecked inbound service-id validation evidence without adding a
  new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry service table; for inbound messages, reject or drop ids outside
  `0..256` and avoid calling `get_service` with invalid `P2pServiceId` values,
  keeping seed `292` as regression evidence for repeated invalid-service
  broadcast delivery.

### Cycle after ISSUE-204 no-new cycle 291: steady valid clean pass

- Result: no accepted issue.
- Reviewer: `Lovelace the 7th`, forked subagent review, confirmed
  clean/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=291 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test tests::fuzz::fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks ... ok`.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 20.93s`.
  - no panic, invalid-service-id, stale-sync, shutdown-send,
    stopped-forwarding, broadcast-alias, path-not-found, lifecycle, capacity,
    or channel-closed markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; the steady valid-node action
  harness completed cleanly and showed no failed invariant.
- Smallest fix proposal: no fix proposal change.

### Cycle after ISSUE-204 no-new cycle 290: sanitized churn shutdown send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Boyle the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=290 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=3000 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with `seed=290, nodes=8, steps=3000`.
- Evidence summary:
  - exit status 101; log had 69 lines; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` reported background connection/service task
    failure.
  - two hard shutdown-send panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - nine connection-lost markers, sixteen closed-by-peer markers, and nine
    aborted-by-peer markers were reviewed as lifecycle fallout around the same
    shutdown/churn condition.
  - no invalid-service-id, stale-sync, stopped-forwarding, broadcast-alias,
    path-not-found, no-capacity, or channel-closed evidence.
- Duplicate mapping: ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  shutdown/closed-main reporting evidence for the outgoing
  `connecting.await` error branch.
- Smallest fix proposal: replace the `expect("should send to main")` sends in
  peer connection error reporting with graceful closed-channel handling; if
  `main_tx.send(...)` fails because the main loop is already stopped, return
  from the peer task without panicking, and keep seed `290` as regression
  evidence for the `src/peer.rs:133` path.

### Cycle after ISSUE-204 no-new cycle 289: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Nash the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=289 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=289, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 6,440 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=289, nodes=8, steps=2600`.
  - one hard stale-sync panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 6,413 forwarded-stop markers, including 5,693 `no available capacity`
    markers and 726 `channel closed` markers.
  - three broadcast-over-peer-alias markers were reviewed as fallout.
  - no invalid-service-id, shutdown-send, path-not-found, connection-lost,
    closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: primary ISSUE-063; secondary amplification ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route race evidence and stopped-peer forwarding storm evidence without
  adding a new issue.
- Smallest fix proposal: for ISSUE-063, replace the `apply_sync` direct-metric
  `expect` with guarded stale-sync handling that drops or ignores sync for a
  missing direct connection; for ISSUE-170, dedupe/coalesce forwarded
  `PeerStopped` events with bounded retry/backpressure and TTL or tombstone
  suppression.

### Cycle after ISSUE-204 no-new cycle 288: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Raman the 7th`, forked subagent review, confirmed duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=288 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=288, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=288, nodes=8, steps=2600`.
  - one `src/ctx.rs:34:9` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one connection-lost and one channel-closed marker were reviewed as teardown
    fallout.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry table, reject or drop ids outside `0..256`, and keep seed `288` as
  regression evidence.

### Cycle after ISSUE-204 no-new cycle 287: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Huygens the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=287 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=287, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 7,941 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=287, nodes=8, steps=2400`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - 7,867 forwarded-stopped-peer markers, including 7,193 no-capacity markers
    and 721 channel-closed markers, were reviewed as duplicate
    stopped-forwarding storm evidence.
  - one open_bi internal-channel, five connection-lost, and one closed-by-peer
    marker were reviewed as fallout.
  - no invalid-service, shutdown-send, broadcast-data, path-not-found, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync and stopped-forwarding storm evidence without adding a new issue.
- Smallest fix proposal: guard or drop stale route sync when the direct metric
  is gone, and dedupe/coalesce `PeerStopped` forwarding with bounded
  retry/backpressure behavior.

### Cycle after ISSUE-204 no-new cycle 286: steady valid-action fuzz pass

- Result: no accepted issue.
- Reviewer: `Socrates the 7th`, forked subagent review, confirmed clean
  no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=286 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered
    out; finished in 23.70s`.
  - no stale-sync, invalid-service, shutdown-send, channel-closed,
    connection-lost, closed-by-peer, path-not-found, no-capacity,
    forwarded-stop, broadcast-data, open_bi, connect-answer, or
    aborted-by-peer evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this clean pass does not add
  issue evidence.
- Smallest fix proposal: none for this cycle.

### Cycle after ISSUE-204 no-new cycle 285: valid stale sync, shutdown send, and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Planck the 7th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=285 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=285, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 9,906 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=285, nodes=8, steps=2400`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - one shutdown-send panic marker at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - 9,760 forwarded-stopped-peer markers, including 8,770 no-capacity markers
    and 1,112 channel-closed markers, were reviewed as duplicate
    stopped-forwarding storm evidence.
  - 40 broadcast-alias and one connection-lost marker were reviewed as fallout.
  - no invalid-service, path-not-found, closed-by-peer, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync, shutdown-send, and stopped-forwarding storm evidence without
  adding a new issue.
- Smallest fix proposal: guard or drop stale route sync when the direct metric
  is gone, replace shutdown-path `expect("should send to main")` sends with
  graceful closed-channel handling, and dedupe/coalesce `PeerStopped`
  forwarding with bounded retry/backpressure behavior.

### Cycle after ISSUE-204 no-new cycle 284: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hypatia the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=284 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=284, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 30 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=284, nodes=8, steps=2600`.
  - three `src/ctx.rs:34:9` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - three connection-lost and three channel-closed markers were reviewed as
    teardown fallout.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry table, reject or drop ids outside `0..256`, and keep seed `284` as
  regression evidence.

### Cycle after ISSUE-204 no-new cycle 283: steady valid-action fuzz pass

- Result: no accepted issue.
- Reviewer: `Jason the 6th`, forked subagent review, confirmed clean no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=283 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered
    out; finished in 23.85s`.
  - no stale-sync, invalid-service, shutdown-send, channel-closed,
    connection-lost, closed-by-peer, path-not-found, no-capacity,
    forwarded-stop, broadcast-data, open_bi, connect-answer, or
    aborted-by-peer evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this clean pass does not add
  issue evidence.
- Smallest fix proposal: none for this cycle.

### Cycle after ISSUE-204 no-new cycle 282: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Pauli the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=282 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=282, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=282, nodes=8, steps=2600`.
  - one `src/ctx.rs:34:9` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed and one closed-by-peer marker were reviewed as teardown
    fallout.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-lost, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry table, reject or drop ids outside `0..256`, and keep seed `282` as
  regression evidence.

### Cycle after ISSUE-204 no-new cycle 281: valid stale sync, shutdown send, and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Nash the 6th`, forked subagent review, confirmed duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=281 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=281, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 1,417 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=281, nodes=8, steps=2400`.
  - two `src/router.rs:76:66` panic markers with
    `should have direct metric with apply_sync`.
  - two shutdown-send panic markers at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - 1,379 forwarded-stopped-peer markers, including 1,022 no-capacity markers
    and 361 channel-closed markers, were reviewed as duplicate
    stopped-forwarding storm evidence.
  - four connection-lost and two closed-by-peer markers were reviewed as
    teardown fallout.
  - no invalid-service, broadcast-data, path-not-found, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync, shutdown-send, and stopped-forwarding storm evidence without
  adding a new issue.
- Smallest fix proposal: guard or drop stale route sync when the direct metric
  is gone, replace shutdown-path `expect("should send to main")` sends with
  graceful closed-channel handling, and dedupe/coalesce `PeerStopped`
  forwarding with bounded retry/backpressure behavior.

### Cycle after ISSUE-204 no-new cycle 280: stale sync and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Bacon the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=280 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=280, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 31 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=280, nodes=8, steps=2600`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - one shutdown-send panic marker at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - six endpoint-driver-dropped internal-error markers and two connection-lost
    markers were reviewed as teardown fallout.
  - no invalid-service, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync and shutdown-send teardown evidence without adding a new issue.
- Smallest fix proposal: guard or drop stale route sync when the direct metric
  is gone, and replace shutdown-path `expect("should send to main")` sends
  with graceful closed-channel handling.

### Cycle after ISSUE-204 no-new cycle 279: steady valid-action fuzz pass

- Result: no accepted issue.
- Reviewer: `Carson the 6th`, forked subagent review, confirmed clean no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=279 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered
    out; finished in 23.33s`.
  - no stale-sync, invalid-service, shutdown-send, channel-closed,
    connection-lost, closed-by-peer, path-not-found, no-capacity,
    forwarded-stop, broadcast-data, open_bi, connect-answer, or
    aborted-by-peer evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this clean pass does not add
  issue evidence.
- Smallest fix proposal: none for this cycle.

### Cycle after ISSUE-204 no-new cycle 278: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Meitner the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=278 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=278, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 29,663 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=278, nodes=8, steps=2400`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - 29,375 forwarded-stopped-peer markers, including 28,621 no-capacity
    markers and 1,019 channel-closed markers, were reviewed as duplicate
    stopped-forwarding storm evidence.
  - 71 broadcast-alias, one open_bi internal-channel, and one connection-lost
    marker were reviewed as fallout.
  - no invalid-service, shutdown-send, path-not-found, closed-by-peer, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync and stopped-forwarding storm evidence without adding a new issue.
- Smallest fix proposal: guard or drop stale route sync when the direct metric
  is gone, and dedupe/coalesce `PeerStopped` forwarding with bounded
  retry/backpressure behavior.

### Cycle after ISSUE-204 no-new cycle 277: broad invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Laplace the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=277 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=277, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 30 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=277, nodes=8, steps=2600`.
  - three `src/ctx.rs:34:9` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - three `channel closed` and three closed-by-peer markers were reviewed as
    teardown fallout.
  - no stale-sync, shutdown-send, open_bi, connect-answer, no-capacity,
    forwarded-stop, broadcast-data, path-not-found, connection-lost, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry table, reject or drop ids outside `0..256`, and keep seed `277` as
  regression evidence.

### Cycle after ISSUE-204 no-new cycle 276: steady valid-action fuzz pass

- Result: no accepted issue.
- Reviewer: `Dalton the 6th`, forked subagent review, confirmed clean no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=276 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered
    out; finished in 23.09s`.
  - no stale-sync, invalid-service, shutdown-send, channel-closed,
    connection-lost, closed-by-peer, path-not-found, no-capacity,
    forwarded-stop, broadcast-data, open_bi, connect-answer, or
    aborted-by-peer evidence.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this clean pass does not add
  issue evidence.
- Smallest fix proposal: none for this cycle.

### Cycle after ISSUE-204 no-new cycle 275: valid stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Aristotle the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=275 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=275, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 21 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=275, nodes=8, steps=2400`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - one connection-closed log line was reviewed as fallout.
  - no invalid-service, shutdown-send, channel-closed, connection-lost,
    closed-by-peer, path-not-found, no-capacity, forwarded-stop,
    broadcast-data, open_bi, connect-answer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync evidence without adding a new issue.
- Smallest fix proposal: guard `Router::apply_sync` against missing direct
  metrics, drop or ignore queued stale sync when the direct route is gone, and
  keep seed `275` as regression evidence.

### Cycle after ISSUE-204 no-new cycle 274: broad invalid service and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Parfit the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=274 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=274, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 33 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=274, nodes=8, steps=2600`.
  - three `src/ctx.rs:34:9` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one shutdown-send panic marker at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - three `channel closed` and three closed-by-peer markers were reviewed as
    teardown fallout.
  - no stale-sync, open_bi, connect-answer, no-capacity, forwarded-stop,
    broadcast-data, path-not-found, connection-lost, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation and shutdown-send teardown evidence without
  adding a new issue.
- Smallest fix proposal: validate service ids before indexing the fixed
  256-entry table, and replace shutdown-path `expect("should send to main")`
  calls with graceful closed-main handling.

### Cycle after ISSUE-204 no-new cycle 273: valid stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kant the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=273 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=273, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 20 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=273, nodes=8, steps=2400`.
  - one `src/router.rs:76:66` panic marker with
    `should have direct metric with apply_sync`.
  - no invalid-service, shutdown-send, channel-closed, connection-lost,
    closed-by-peer, path-not-found, no-capacity, forwarded-stop,
    broadcast-data, open_bi, connect-answer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing stale
  route-sync evidence without adding a new issue.
- Smallest fix proposal: guard `Router::apply_sync` against missing direct
  metrics, drop or ignore queued stale sync when the direct route is gone, and
  keep seed `273` as regression evidence.

### Cycle after ISSUE-204 no-new cycle 272: broad invalid service and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Ampere the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=272 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=272, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 49 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=272, nodes=8, steps=2600`.
  - six `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one shutdown-send panic marker at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - six `channel closed`, four connection-lost, four closed-by-peer, and one
    endpoint-internal-error marker were reviewed as teardown fallout.
  - no stale-sync, open_bi, connect-answer, no-capacity, forwarded-stop,
    broadcast-data, path-not-found, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation and shutdown-send teardown evidence without
  adding a new issue.
- Smallest fix proposal: validate service ids before indexing, and replace
  shutdown-path `expect("should send to main")` calls with graceful
  closed-main handling when reporting peer lifecycle events.

### Cycle after ISSUE-204 no-new cycle 271: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Archimedes the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=271 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=271, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 14,337 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=271, nodes=8, steps=2400`.
  - one router panic marker at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 14,271 `forward peer stopped over peer alias` markers, including 14,080
    `no available capacity` markers and 235 `channel closed` markers.
  - seven `broadcast data over peer alias` markers were reviewed as fallout
    inside the same known stopped-peer storm.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    path-not-found, connection-lost, closed-by-peer, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 270: steady valid fuzz pass

- Result: no accepted issue.
- Reviewer: `Plato the 6th`, forked subagent review, confirmed no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=270 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 24.34s`.
  - no invalid-service-id, stale-sync, shutdown-send, open_bi,
    connect-answer, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-loss, closed-by-peer, aborted-by-peer, or panic
    markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this is clean steady-valid
  churn evidence only.
- Smallest fix proposal: none for this cycle; continue alternating clean
  steady-valid sampling with failure-producing broad and valid-action fuzz
  runs until accepted issues are fixed.

### Cycle after ISSUE-204 no-new cycle 269: broad invalid service id panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Popper the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=269 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=269, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 38 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=269, nodes=8, steps=2600`.
  - five `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - five `channel closed`, four closed-by-peer, and one connection-lost marker
    were reviewed as teardown fallout in the invalid-service context.
  - no stale-sync, shutdown-send, open_bi, connect-answer, no-capacity,
    forwarded-stop, broadcast-data, path-not-found, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 268: compact valid stale sync panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Franklin the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=268 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=268, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=268, nodes=8, steps=2400`.
  - two router panic markers at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    no-capacity, forwarded-stop, broadcast-data, path-not-found,
    connection-lost, closed-by-peer, aborted-by-peer, or channel-closed
    evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation evidence without adding a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and invalidate or drop queued route sync
  work when its direct metric source disappears.

### Cycle after ISSUE-204 no-new cycle 267: steady valid fuzz pass

- Result: no accepted issue.
- Reviewer: `Averroes the 6th`, forked subagent review, confirmed no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=267 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 24.47s`.
  - no invalid-service-id, stale-sync, shutdown-send, open_bi,
    connect-answer, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-loss, closed-by-peer, aborted-by-peer, or panic
    markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this is clean steady-valid
  churn evidence only.
- Smallest fix proposal: none for this cycle; continue alternating clean
  steady-valid sampling with failure-producing broad and valid-action fuzz
  runs until accepted issues are fixed.

### Cycle after ISSUE-204 no-new cycle 266: compact broad invalid service id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hubble the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=266 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=266, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=266, nodes=8, steps=2600`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` marker and one closed-by-peer marker were reviewed as
    teardown fallout in the invalid-service context.
  - no stale-sync, shutdown-send, open_bi, connect-answer, no-capacity,
    forwarded-stop, broadcast-data, path-not-found, connection-lost, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 265: valid stale sync and large stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Curie the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=265 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=265, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 15,361 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=265, nodes=8, steps=2400`.
  - four router panic markers at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 15,273 `forward peer stopped over peer alias` markers, including 14,531
    `no available capacity` markers and 804 `channel closed` markers.
  - 13 `broadcast data over peer alias` markers were reviewed as fallout
    inside the same known stopped-peer storm.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    path-not-found, connection-lost, closed-by-peer, or aborted-by-peer
    evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 264: steady valid fuzz pass

- Result: no accepted issue.
- Reviewer: `Peirce the 6th`, forked subagent review, confirmed no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=264 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 24.27s`.
  - no invalid-service-id, stale-sync, shutdown-send, open_bi,
    connect-answer, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-loss, closed-by-peer, aborted-by-peer, or panic
    markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this is clean steady-valid
  churn evidence only.
- Smallest fix proposal: none for this cycle; continue alternating clean
  steady-valid sampling with failure-producing broad and valid-action fuzz
  runs until accepted issues are fixed.

### Cycle after ISSUE-204 no-new cycle 263: compact broad stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Erdos the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=263 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=263, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 20 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=263, nodes=8, steps=2600`.
  - one router panic marker at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    no-capacity, forwarded-stop, broadcast-data, path-not-found,
    connection-lost, closed-by-peer, aborted-by-peer, or channel-closed
    evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation evidence without adding a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and invalidate or drop queued route sync
  work when its direct metric source disappears.

### Cycle after ISSUE-204 no-new cycle 262: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Sagan the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=262 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=262, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 6,169 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=262, nodes=8, steps=2400`.
  - two router panic markers at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 6,100 `forward peer stopped over peer alias` markers, with 6,133
    `no available capacity` markers and 12 `channel closed` markers.
  - five `broadcast data over peer alias` markers, one `connection lost`
    marker, and one internal endpoint marker were reviewed as fallout inside
    the same stale-sync/stopped-peer storm context.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    path-not-found, closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 261: broad invalid, shutdown, and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Gauss the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=261 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=261, nodes=8, steps=2600`.
- Evidence summary:
  - exit status 101; log had 4,757 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=261, nodes=8, steps=2600`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one shutdown-send panic marker at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - 4,704 `forward peer stopped over peer alias` markers, including 3,791
    `no available capacity` markers and 941 `channel closed` markers.
  - 17 `broadcast data over peer alias` markers and one closed-by-peer marker
    were reviewed as fallout inside the same invalid-service/shutdown/stopped
    context.
  - no stale-sync, open_bi, connect-answer, path-not-found, connection-lost, or
    aborted-by-peer evidence.
- Duplicate mapping: ISSUE-053, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service validation, graceful shutdown reporting, and stopped-peer
  storm evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, replace
  shutdown-path `expect("should send to main")` calls with graceful closed-main
  handling, and suppress duplicate `PeerStopped` forwarding with per-peer
  tombstones, TTL, or coalescing before retrying onto bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 260: long steady valid fuzz pass

- Result: no accepted issue.
- Reviewer: `Newton the 6th`, forked subagent review, confirmed no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=260 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 24.49s`.
  - no invalid-service-id, stale-sync, shutdown-send, open_bi,
    connect-answer, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-loss, closed-by-peer, aborted-by-peer, or panic
    markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this is clean steady-valid
  churn evidence only.
- Smallest fix proposal: none for this cycle; keep alternating clean
  steady-valid sampling with failure-producing broad and valid-action fuzz
  runs until accepted issues are fixed.

### Cycle after ISSUE-204 no-new cycle 259: compact valid stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kierkegaard the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=259 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=259, nodes=8, steps=2200`.
- Evidence summary:
  - exit status 101; log had 21 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=259, nodes=8, steps=2200`.
  - one router panic marker at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - one `P2pNetwork connection ... outgoing: None error closed` line was
    reviewed as insufficient to establish a distinct issue.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    no-capacity, forwarded-stop, broadcast-data, path-not-found,
    connection-lost, closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation evidence without adding a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and invalidate or drop queued route sync
  work when its direct metric source disappears.

### Cycle after ISSUE-204 no-new cycle 258: broad invalid service and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `James the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=258 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=258, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 53 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=258, nodes=8, steps=2400`.
  - seven `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one shutdown-send panic marker at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - seven `channel closed`, six closed-by-peer, one connection-lost, and two
    aborted-by-peer markers were reviewed as teardown fallout in the same
    invalid-service/shutdown context.
  - no stale-sync, open_bi, connect-answer, no-capacity, forwarded-stop,
    broadcast-data, or path-not-found evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation and shutdown-send teardown evidence without
  adding a new issue.
- Smallest fix proposal: validate service ids before indexing, and replace
  shutdown-path `expect("should send to main")` calls with graceful
  closed-main handling when reporting peer lifecycle events.

### Cycle after ISSUE-204 no-new cycle 257: steady valid fuzz pass

- Result: no accepted issue.
- Reviewer: `Raman the 6th`, forked subagent review, confirmed no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=257 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; log had 8 lines.
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289 filtered out; finished in 21.71s`.
  - no invalid-service-id, stale-sync, shutdown-send, open_bi,
    connect-answer, no-capacity, forwarded-stop, broadcast-data,
    path-not-found, connection-loss, closed-by-peer, aborted-by-peer, or panic
    markers.
- Duplicate mapping: none.
- Root-cause summary impact: no new root cause; this is clean steady-valid
  churn evidence only.
- Smallest fix proposal: none for this cycle; keep using varied steady-valid
  fuzz runs to sample lifecycle stability after fixes to the accepted issues.

### Cycle after ISSUE-204 no-new cycle 256: valid stale sync and massive stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Aquinas the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=256 P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed; the test assertion reported `seed=256, nodes=8, steps=2400`.
- Evidence summary:
  - exit status 101; log had 79,069 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=256, nodes=8, steps=2400`.
  - one router panic marker at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 78,653 `forward peer stopped over peer alias` markers, including 78,397
    `no available capacity` markers and 648 `channel closed` markers.
  - one `answer open_bi got error internal channel error`, 55
    `broadcast data over peer alias` markers, and two `connection lost`
    markers were reviewed as fallout inside the same stopped-peer storm.
  - no invalid-service-id, shutdown-send, connect-answer, path-not-found,
    closed-by-peer, or aborted-by-peer evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 255: broad invalid service id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hooke the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=255 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=255, nodes=8, steps=1800`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send marker and one closed-by-peer marker were
    reviewed as teardown fallout in the invalid-service-id context.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 254: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Faraday the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=254 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 13,735 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=254, nodes=8, steps=1800`.
  - two router panic markers at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 13,654 `forward peer stopped over peer alias` markers, including 11,854
    `no available capacity` markers and 1,857 `channel closed` markers.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer,
    broadcast-alias, or path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 253: valid stale sync and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Fermat the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=253 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 48,458 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=253, nodes=8, steps=1800`.
  - one router panic marker at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - 47,925 `forward peer stopped over peer alias` markers, including 47,896
    `no available capacity` markers and 542 `channel closed` markers.
  - 42 `broadcast data over peer alias` markers were reviewed as low-count
    fallout inside the same stopped-peer storm.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer, or
    path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale route-sync invalidation and stopped-peer storm evidence without adding
  a new issue.
- Smallest fix proposal: guard stale route-sync application when the direct
  metric has already been removed, and suppress duplicate `PeerStopped`
  forwarding with per-peer tombstones, TTL, or coalescing before retrying onto
  bounded peer-alias queues.

### Cycle after ISSUE-204 no-new cycle 252: broad repeated invalid service id panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Epicurus the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=252 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 38 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=252, nodes=8, steps=1800`.
  - five `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - five connection closed/lost markers and five `channel closed` send markers
    were reviewed as fallout in the invalid-service-id context.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 251: valid stale sync and large stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Einstein the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=251 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 23102 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failures with `seed=251, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 22884 `forward peer stopped over peer alias` errors, with 19876
    `no available capacity` markers and 3195 `channel closed` markers in the
    log.
  - 53 `broadcast data over peer alias` markers and 11 connection
    lost/closed/internal markers were reviewed as fallout in the stopped-storm
    context.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer, or
    path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 250: broad invalid service id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Locke the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=250 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=250, nodes=8, steps=1800`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one closed-by-peer marker and one `channel closed` send marker were
    reviewed as fallout in the invalid-service-id context.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 249: valid stale sync and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Ohm the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=249 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 27 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=249, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` panic marker with
    `should send to main: SendError { .. }`.
  - two closed-by-peer markers were reviewed as teardown fallout in the same
    compact stale-sync/shutdown-send context.
  - no invalid-service-id, no-capacity, channel-closed, forwarded-stop,
    broadcast-data, open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and shutdown-reporting evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-139, replace `expect("should send to main")` with
    shutdown-aware non-panicking handling.

### Cycle after ISSUE-204 no-new cycle 248: broad stale sync and stopped storm with broadcast fallout

- Result: no accepted non-duplicate issue.
- Reviewer: `Schrodinger the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=248 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 5855 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failure with `seed=248, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 5808 `forward peer stopped over peer alias` errors, with 3434
    `no available capacity` markers and 2387 `channel closed` markers in the
    log.
  - nine `broadcast data over peer alias` no-capacity markers and thirteen
    connection lost/closed/internal markers were reviewed as fallout in the
    stopped-storm context.
  - no invalid-service-id, shutdown-send, open_bi, connect-answer, or
    path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 247: valid stale sync, shutdown send, and large stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Linnaeus the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=247 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 17227 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failures with `seed=247, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:133` panic marker with
    `should send to main: SendError { .. }`.
  - 17050 `forward peer stopped over peer alias` errors, with 17119
    `no available capacity` markers and 83 `channel closed` markers in the
    log.
  - 34 `broadcast data over peer alias` no-capacity markers and two
    connection/internal markers were reviewed as fallout in the stopped-storm
    context.
  - no invalid-service-id, open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync, shutdown-reporting, and PeerStopped storm evidence without
  adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-139, replace `expect("should send to main")` with
    shutdown-aware non-panicking handling.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 246: broad invalid, stale sync, stopped storm, and broadcast fallout

- Result: no accepted non-duplicate issue.
- Reviewer: `Poincare the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=246 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 2741 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failures with `seed=246, nodes=8, steps=1800`.
  - three `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 2698 `forward peer stopped over peer alias` errors, with 2106
    `no available capacity` markers and 603 `channel closed` markers in the
    log.
  - eight `broadcast data over peer alias` no-capacity markers and four
    connection lost/closed/internal markers were reviewed as fallout in the
    stopped-storm context.
  - no shutdown-send, open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053, ISSUE-063, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id, stale-route-sync, and PeerStopped storm evidence without
  adding a new issue.
- Smallest fix proposal:
  - for ISSUE-053, validate service ids before indexing, preferably at decode
    or inbound dispatch, and reject, drop, or log out-of-range ids.
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 245: valid stale sync panic, stopped storm, and open_bi fallout

- Result: no accepted non-duplicate issue.
- Reviewer: `Lorentz the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=245 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 5348 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=245, nodes=8, steps=1800`.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 5273 `forward peer stopped over peer alias` errors, with 4224
    `no available capacity` markers and 1088 `channel closed` markers in the
    log.
  - one `answer open_bi got error internal channel error` marker and eleven
    connection lost/closed/internal markers were reviewed as fallout in the
    stale-sync/stopped-storm context.
  - no invalid-service-id, shutdown-send, broadcast-data, connect-answer, or
    path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 244: broad stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Sartre the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=244 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 20 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=244, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - no invalid-service-id, shutdown-send, no-capacity, channel-closed,
    forwarded-stop, broadcast-data, open_bi, connect-answer, connection-lost,
    or path-not-found evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync evidence without adding a new issue.
- Smallest fix proposal: guard the direct-route lookup, ignore stale sync for
  unknown direct connections, and clear queued sync when direct connection state
  is removed.

### Cycle after ISSUE-204 no-new cycle 243: valid stale sync panic and large stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Pascal the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=243 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 7591 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=243, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 7555 `forward peer stopped over peer alias` errors, with 5874
    `no available capacity` markers and 1693 `channel closed` markers in the
    log.
  - four connection lost/closed/internal markers were reviewed as fallout in
    the stale-sync/stopped-storm context.
  - no invalid-service-id, shutdown-send, broadcast-data, open_bi,
    connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 242: broad invalid service id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Feynman the 6th`, forked subagent review, confirmed
  duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=242 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 30 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=242, nodes=8, steps=1800`.
  - three `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - three connection closed/lost markers and three `channel closed` send
    markers were reviewed as fallout in the invalid-service-id context.
  - no stale-sync, shutdown-send, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id validation evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids.

### Cycle after ISSUE-204 no-new cycle 241: valid stale sync panic and stopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Arendt the 6th`, forked subagent review, confirmed duplicate/no-new.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=241 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 5262 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=241, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 5229 `forward peer stopped over peer alias` errors, with 5225
    `no available capacity` markers and 12 `channel closed` markers in the
    log.
  - three connection/internal endpoint-driver-dropped markers were reviewed as
    fallout in the stale-sync/stopped-storm context.
  - no invalid-service-id, shutdown-send, broadcast-data, open_bi,
    connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 240: broad mixed invalid, stale sync, shutdown, and storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Rawls the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=240 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 1599 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=240, nodes=8, steps=1800`.
  - seven `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` panic marker with
    `should send to main: SendError { .. }`.
  - 1545 `forward peer stopped over peer alias` errors, with 1086
    `no available capacity` markers and 469 `channel closed` markers in the
    log.
  - three `broadcast data over peer alias` errors and connection closed/lost
    lines were reviewed as fallout in the same panic/storm context.
  - no open_bi, connect-answer, or path-not-found evidence.
- Duplicate mapping: ISSUE-053, ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id, stale-route-sync, shutdown-reporting, and PeerStopped
  storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-053, validate service ids before indexing, preferably at decode
    or inbound dispatch, and reject, drop, or log out-of-range ids.
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-139, replace `expect("should send to main")` with
    shutdown-aware non-panicking handling.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 239: isolated valid stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Bohr the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=239 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 20 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=239, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, no-capacity, channel-closed, forwarded-stop,
    broadcast-data, open_bi, connect-answer, path-not-found, connection-lost,
    or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync evidence without adding a new issue.
- Smallest fix proposal: replace the direct-route `expect` with a guarded
  lookup, drop stale sync for unknown direct connections, and clear queued sync
  state when direct connection state is removed.

### Cycle after ISSUE-204 no-new cycle 238: broad invalid service and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Gibbs the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=238 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 50 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=238, nodes=8, steps=1800`.
  - seven `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/peer.rs:133` panic marker with
    `should send to main: SendError { .. }`.
  - eight `channel closed` send errors and seven connection-loss/closed-peer
    lines were reviewed as teardown fallout after the panics.
  - no `src/router.rs:76` stale-sync evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id and shutdown-reporting evidence without adding a new
  issue.
- Smallest fix proposal:
  - for ISSUE-053, validate service ids before indexing, preferably at decode
    or inbound dispatch, then reject, drop, or log out-of-range ids.
  - for ISSUE-139, replace `expect("should send to main")` in shutdown and
    error-report paths with shutdown-aware non-panicking handling, such as
    log-and-return or ignoring send failure after the main receiver closes.

### Cycle after ISSUE-204 no-new cycle 237: valid stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Carver the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=237 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 3734 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=237, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 3709 `forward peer stopped over peer alias` errors, with 3067
    `no available capacity` markers and 642 `channel closed` markers in the
    log.
  - one `send connect answer got error Ok(())` line was reviewed as
    fallout/noise in the immediate stale-sync and storm context.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, broadcast-data, open_bi, path-not-found, or
    invalid-service evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 236: broad invalid service and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Maxwell the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=236 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 4409 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=236, nodes=8, steps=1800`.
  - five `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - 4371 `forward peer stopped over peer alias` errors, with 2351
    `no available capacity` markers and 2025 `channel closed` markers in the
    log.
  - no `src/router.rs:76` stale-sync evidence.
  - no `should send to main`, broadcast-data, open_bi, connect-answer,
    path-not-found, or stale-sync evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-053, validate service ids before indexing, preferably at decode
    or inbound dispatch, then reject, drop, or log out-of-range ids.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 235: valid mixed stale sync, shutdown send, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Zeno the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=235 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 9150 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=235, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - two `src/peer.rs:92` panic markers with
    `should send to main: SendError { .. }`.
  - 9048 `forward peer stopped over peer alias` errors, with 9082
    `no available capacity` markers and 38 `channel closed` markers in the
    log.
  - 35 `broadcast data over peer alias` errors and endpoint-driver dropped
    internal errors were reviewed as storm or teardown fallout.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no open_bi, connect-answer, path-not-found, or invalid-service evidence.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync, shutdown-reporting, and PeerStopped storm evidence without
  adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct connection state is
    removed.
  - for ISSUE-139, replace `expect("should send to main")` with
    shutdown-aware handling such as log-and-return or ignoring send failure
    after the main receiver closes.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and rate-limit or suppress repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 234: isolated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Descartes the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=234 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=234, nodes=8, steps=1800`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `closed by peer` line and one `channel closed` send error were reviewed
    as teardown noise after the invalid-service panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `should send to main`, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids so
  downstream service-table accesses only receive valid ids.

### Cycle after ISSUE-204 no-new cycle 233: valid stale sync and large PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Pasteur the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=233 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 16791 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=233, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 16745 `forward peer stopped over peer alias` errors, including 14030
    `no available capacity` errors and 2738 `channel closed` errors.
  - one `broadcast data over peer alias` error was reviewed as storm fallout.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, open_bi, connect-answer, path-not-found, or
    invalid-service evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync state when direct connection
    state is removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 232: repeated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Boole the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=232 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 34 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=232, nodes=8, steps=1800`.
  - four `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - four `channel closed` send errors and four connection-loss/closed-peer
    lines were reviewed as teardown noise after the invalid-service panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `should send to main`, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range service ids
  so downstream service-table accesses only receive valid ids.

### Cycle after ISSUE-204 no-new cycle 231: valid stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Copernicus the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=231 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 10293 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=231, nodes=8, steps=1800`.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 10231 `forward peer stopped over peer alias` errors, including 9476
    `no available capacity` errors and 790 `channel closed` errors.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, broadcast-data, open_bi, connect-answer,
    path-not-found, or invalid-service evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync state when direct connection
    state is removed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 230: isolated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kepler the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=230 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=230, nodes=8, steps=1800`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `closed by peer` line and one `channel closed` send error were reviewed
    as teardown noise after the invalid-service panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `should send to main`, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing, preferably at
  decode or inbound dispatch, and reject, drop, or log out-of-range ids so
  downstream service-table accesses only receive valid ids.

### Cycle after ISSUE-204 no-new cycle 229: valid stale sync, shutdown send, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Avicenna the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=229 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 4229 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=229, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - two `src/peer.rs:92` panic markers with
    `should send to main: SendError { .. }`.
  - 4178 `forward peer stopped over peer alias` errors, including 3860
    `no available capacity` errors and 339 `channel closed` errors.
  - six `broadcast data over peer alias` errors were reviewed as storm fallout.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no open_bi, connect-answer, path-not-found, or invalid-service evidence.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync, shutdown-reporting, and PeerStopped storm evidence without
  adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync state when direct route state is
    removed.
  - for ISSUE-139, replace `expect("should send to main")` with
    shutdown-aware non-panicking handling, such as log-and-return or ignoring
    send failure once the main receiver is closed.
  - for ISSUE-170, add per-event dedupe or tombstones, bound forwarded stop
    propagation with TTL, and suppress or rate-limit repeated send failures
    during shutdown.

### Cycle after ISSUE-204 no-new cycle 228: broad invalid service and stale sync panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Hume the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=228 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 55 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=228, nodes=8, steps=1800`.
  - eight `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - eight `channel closed` send errors and eleven connection-loss/closed-peer
    lines were reviewed as teardown noise after the panics.
  - no `should send to main`, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053 and ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id and stale-route-sync evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-053, validate service ids before indexing, preferably at decode
    or inbound dispatch, and reject, drop, or log out-of-range ids.
  - for ISSUE-063, guard the direct-route lookup, ignore stale sync for unknown
    direct connections, and clear queued sync when direct route state is
    removed.

### Cycle after ISSUE-204 no-new cycle 227: valid stale sync with PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Harvey the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=227 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 10839 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=227, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 10779 `forward peer stopped over peer alias` errors, including 9867
    `no available capacity` errors and 949 `channel closed` errors.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, broadcast-data, open_bi, connect-answer,
    path-not-found, or invalid-service evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync state when direct route state is
    removed.
  - for ISSUE-170, add per-peer/event dedupe or tombstones, bound forwarded
    stop propagation with TTL, and suppress or rate-limit repeated send
    failures during shutdown.

### Cycle after ISSUE-204 no-new cycle 226: isolated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Lovelace the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=226 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=226, nodes=8, steps=1800`.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `closed by peer` line and one `channel closed` send error were reviewed
    as teardown noise after the invalid-service panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `should send to main`, no-capacity, forwarded-stop, broadcast-data,
    open_bi, connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  invalid-service-id evidence without adding a new issue.
- Smallest fix proposal: validate service ids before indexing in the inbound
  dispatch path, rejecting or dropping out-of-range ids at decode or message
  admission so downstream service-table accesses only receive valid ids.

### Cycle after ISSUE-204 no-new cycle 225: valid stale sync and peer send shutdown

- Result: no accepted non-duplicate issue.
- Reviewer: `Wegener the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=225 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 25 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=225, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:133` panic marker with
    `should send to main: SendError { .. }`.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, open_bi,
    connect-answer, path-not-found, connection-lost, or PeerStopped storm
    evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and shutdown-reporting evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    direct connections, and clear queued sync state when direct route state is
    removed.
  - for ISSUE-139, replace peer-task `expect("should send to main")` calls with
    shutdown-aware handling such as log-and-return or ignoring send failure once
    the main receiver is closed.

### Cycle after ISSUE-204 no-new cycle 224: broad stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Turing the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_alias.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=224 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 8159 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=224, nodes=8, steps=1800`.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 8098 `forward peer stopped over peer alias` errors, including 6597
    `no available capacity` errors and 1540 `channel closed` errors.
  - 11 `broadcast data over peer alias` errors were reviewed as storm fallout.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no `should send to main`, open_bi, connect-answer, path-not-found, or
    connection-lost evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and PeerStopped storm evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, guard the direct-route lookup, drop stale sync for unknown
    connections, and clear queued sync when direct connection state is removed.
  - for ISSUE-170, add per-peer/event dedupe or tombstones for forwarded stop
    notifications, bound propagation with TTL, and suppress or rate-limit
    repeated send failures during shutdown.

### Cycle after ISSUE-204 no-new cycle 223: valid stale sync and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Anscombe the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=223 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 27 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic with `seed=223, nodes=8, steps=1800`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - two `src/peer.rs:92` panic markers with
    `should send to main: SendError { .. }`.
  - no `src/ctx.rs:34` invalid-service evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, open_bi,
    connect-answer, path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-route-sync and shutdown-reporting evidence without adding a new issue.
- Smallest fix proposal:
  - for ISSUE-063, replace the direct-route `expect` with a guarded lookup that
    drops stale sync for unknown connections and invalidate queued sync when a
    direct route is removed.
  - for ISSUE-139, replace peer-task `expect("should send to main")` shutdown
    reports with non-panicking closed-main handling.

### Cycle after ISSUE-204 no-new cycle 222: broad isolated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Dirac the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=222 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one `closed by peer` log were
    reviewed as teardown fallout after the service task panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 221: valid mixed stale sync, shutdown send, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Godel the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=221 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92:104` panic marker with
    `should send to main: SendError { .. }`.
  - 8,478 `forward peer stopped over peer alias` send failures: 7,423
    `no available capacity` and 1,113 `channel closed`.
  - 22 `broadcast data over peer alias` no-capacity logs were reviewed as
    storm fallout from the same ISSUE-170 pattern.
  - two connection lost/internal endpoint logs were reviewed as teardown
    fallout.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation, shutdown-send-panic, and PeerStopped storm evidence
  without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 220: broad repeated invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Mencius the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=220 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 35 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - four `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - four `channel closed` send errors, three `closed by peer` logs, one
    `connection lost` log, and one later connection closed log were reviewed
    as teardown fallout after the service task panics.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 219: valid stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Noether the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=219 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 13,993 `forward peer stopped over peer alias` send failures: 12,094
    `no available capacity` and 1,990 `channel closed`.
  - three connection-lost logs were reviewed as teardown fallout.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 218: broad duplicate stale sync panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Euclid the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=218 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - one `forward peer stopped over peer alias got error no available capacity`
    line was reviewed as too small to classify as ISSUE-170 storm/backpressure
    evidence by itself.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no channel-closed, broadcast-data, open_bi, connect-answer,
    path-not-found, connection-loss, or WARN logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 217: valid isolated stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Heisenberg the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=217 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, open_bi,
    connect-answer, path-not-found, connection-loss, teardown, or WARN logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 216: broad isolated invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Dewey the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=216 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 22 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one `closed by peer` log were
    reviewed as teardown fallout after the background service panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 215: valid isolated stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `McClintock the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=215 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, open_bi,
    connect-answer, path-not-found, connection-loss, or WARN logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 214: broad stale sync and alias storm fallout

- Result: no accepted non-duplicate issue.
- Reviewer: `Ptolemy the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=214 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 12,627 `forward peer stopped over peer alias` send failures: 12,506
    `no available capacity` and 172 `channel closed`.
  - 11 `broadcast data over peer alias` no-capacity logs were reviewed as
    storm fallout from the same ISSUE-170 pattern.
  - one endpoint internal-error log was reviewed as teardown fallout.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 213: valid repeated stale sync and massive PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Halley the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=213 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    failures.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 33,435 `forward peer stopped over peer alias` send failures: 29,915
    `no available capacity` and 3,705 `channel closed`.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found,
    connection-loss, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 212: broad repeated invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Tesla the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=212 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 42 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - six `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - six `channel closed` send errors, five `closed by peer` logs, and one
    `connection lost` log were reviewed as teardown fallout after the
    background panics.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 211: valid stale sync and large alias storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Chandrasekhar the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=211 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 10,725 `forward peer stopped over peer alias` send failures: 9,411
    `no available capacity` and 1,421 `channel closed`.
  - 32 `broadcast data over peer alias` no-capacity/channel-closed logs were
    reviewed as storm fallout from the same ISSUE-170 pattern.
  - three endpoint internal-error logs were reviewed as teardown fallout after
    the panic/assertion.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 210: broad duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Bernoulli the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=210 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 23 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - two `channel closed` send errors and one `closed by peer` log were
    reviewed as teardown fallout after the background panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 209: valid stale sync and alias storm fallout

- Result: no accepted non-duplicate issue.
- Reviewer: `Volta the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=209 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 9,175 `forward peer stopped over peer alias` send failures: 8,523
    `no available capacity` and 826 `channel closed`.
  - 38 `broadcast data over peer alias` no-capacity logs and one
    `answer open_bi got error internal channel error` log were reviewed as
    storm/lifecycle fallout from the same ISSUE-170 pattern.
  - three connection lost/internal endpoint logs were reviewed as teardown
    noise.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 208: broad duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: forked RED-team reviewer review, confirmed `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=208 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; log had 23 lines; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - two `channel closed` send errors and one `closed by peer` log were
    reviewed as teardown fallout after the background panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or PeerStopped storm evidence.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens the existing
  unchecked inbound service-id validation evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 207: valid duplicate stale sync and shutdown send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Mill the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=207 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92:104` panic marker with
    `should send to main: SendError { .. }`.
  - one endpoint internal-error log was reviewed as fallout from the router
    panic or dropped endpoint driver.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, channel-closed, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and shutdown-send-panic evidence without adding a
  new issue.

### Cycle after ISSUE-204 no-new cycle 206: broad duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Huygens the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=206 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; the fuzz assertion at `src/tests/fuzz.rs:183:5`
    reported a background connection/service task panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one `connection lost` log were
    reviewed as teardown fallout after the background panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 205: valid repeated stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Leibniz the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=205 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 6,627 `forward peer stopped over peer alias` send failures: 6,102
    `no available capacity` and 534 `channel closed`.
  - two connection-lost logs were reviewed as network teardown fallout.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 204: broad duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Mendel the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=204 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; the fuzz assertion at `src/tests/fuzz.rs:183:5`
    reported a background connection/service task panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one `closed by peer` log were
    reviewed as fallout after the background panic/connection teardown.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 203: valid duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Planck the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=203 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported a background connection/service task
    panic.
  - one `src/router.rs:76` panic with
    `should have direct metric with apply_sync`.
  - 7,647 `forward peer stopped over peer alias` send failures: 7,184
    `no available capacity` and 484 `channel closed`.
  - three connection lost/closed/internal endpoint logs were reviewed as
    network teardown fallout.
  - no `src/ctx.rs:34` invalid-service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  stale-sync invalidation and PeerStopped storm evidence without adding a new
  issue.

### Cycle after ISSUE-204 no-new cycle 202: compact duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hegel the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=202 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; the fuzz assertion at `src/tests/fuzz.rs:183:5`
    reported a background connection/service task panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one `closed by peer` log were
    reviewed as teardown/lifecycle fallout after the same background panic.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 201: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Hilbert the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=201 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - five `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - five `channel closed` send errors plus five closed/lost connection logs
    were reviewed as teardown/lifecycle noise in the same failure context.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 200: valid random duplicate stale sync and extreme PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Kuhn the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=200 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported the background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 107,853 forwarded-stop peer-alias errors were present, including 107,461
    `no available capacity` logs and 1,801 `channel closed` logs.
  - 175 `broadcast data over peer alias` logs and two
    `answer open_bi got error internal channel error` logs were reviewed as
    storm-context fallout under ISSUE-170.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, connection-lost, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 199: broad random duplicate invalid service and stale sync panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Lagrange the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=199 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one connection-lost log and one channel-closed send error were reviewed as
    lifecycle/teardown noise in the same failure context.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053 and ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id and ISSUE-063 stale-sync evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 198: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Socrates the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=198 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` reported background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 6,326 forwarded-stop peer-alias errors were present, including 4,881
    `no available capacity` logs and 1,449 `channel closed` logs.
  - seven connection-lost/closed/aborted/internal-error signals were reviewed
    as lifecycle noise in the same storm context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 197: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Goodall the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=197 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected the background connection/service task
    panic.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, open_bi,
    connect-answer, connection-lost, path-not-found, ERROR, or WARN logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 196: valid random duplicate stale sync and large PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Helmholtz the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=196 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 10,170 forwarded-stop peer-alias errors were present, including 8,671
    `no available capacity` logs and 1,545 `channel closed` logs.
  - 21 `broadcast data over peer alias` logs were reviewed as storm-context
    fallout under ISSUE-170.
  - eight connection-lost/closed/aborted/internal-error signals were reviewed
    as lifecycle noise in the same failure context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 195: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Euler the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=195 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - three `channel closed` send errors plus connection-lost and closed-by-peer
    logs were reviewed as teardown/lifecycle noise in the same failure
    context.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 194: valid random duplicate stale sync and shutdown send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Banach the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=194 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:133` panic marker with
    `should send to main: SendError`.
  - one connection-lost log and one internal endpoint error log were reviewed
    as lifecycle noise in the same failure context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    channel-closed, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-139.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-139 shutdown send evidence without adding a
  new issue.

### Cycle after ISSUE-204 no-new cycle 193: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Herschel the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=193 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - two `channel closed` send errors plus closed-by-peer logs were reviewed as
    teardown/lifecycle noise in the same failure context.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    connection-lost, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 192: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Galileo the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=192 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 7,774 forwarded-stop peer-alias errors were present, including 6,915
    `no available capacity` logs and 900 `channel closed` logs.
  - two connection-lost/closed/aborted/internal-error signals were reviewed as
    lifecycle noise in the same storm context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 191: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Singer the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=191 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - four `channel closed` send errors plus closed-by-peer and connection-lost
    logs were reviewed as teardown/lifecycle noise in the same failure
    context.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 190: valid random duplicate stale sync, shutdown send, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Cicero the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=190 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` panic marker with
    `should send to main: SendError`.
  - 6,708 forwarded-stop peer-alias errors were present, including 6,405
    `no available capacity` logs and 343 `channel closed` logs.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no open_bi, connect-answer, broadcast-data, connection-lost,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063, ISSUE-139, and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync, ISSUE-139 shutdown send, and ISSUE-170 forwarded-stop
  storm evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 189: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Nietzsche the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=189 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `channel closed` send error and one closed-by-peer log were reviewed
    as teardown/lifecycle noise in the same failure context.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, open_bi, connect-answer,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 188: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Confucius the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=188 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 5,669 forwarded-stop peer-alias errors were present, including 5,514
    `no available capacity` logs and 157 `channel closed` logs.
  - two connection-lost/closed/aborted/internal-error signals were reviewed as
    lifecycle noise in the same storm context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 187: broad random duplicate invalid service and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Beauvoir the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=187 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - 14,822 forwarded-stop peer-alias errors were present, including 12,972
    `no available capacity` logs and 2,007 `channel closed` logs.
  - 55 `broadcast data over peer alias got error no available capacity` logs
    were reviewed as storm-context fallout under ISSUE-170.
  - no `src/router.rs:76` stale-sync evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, connection-lost, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-053 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 invalid-service-id and ISSUE-170 forwarded-stop storm evidence
  without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 186: valid random duplicate stale sync and large PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Darwin the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=186 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 10,425 forwarded-stop peer-alias errors were present, including 8,605
    `no available capacity` logs and 1,840 `channel closed` logs.
  - eight connection-lost/closed/aborted/internal-error signals were reviewed
    as lifecycle noise in the same failure context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 185: broad random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Russell the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=185 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` panic markers with
    `should have direct metric with apply_sync`.
  - 5,973 forwarded-stop peer-alias errors were present, including 4,843
    `no available capacity` logs and 1,132 `channel closed` logs.
  - one connection-lost/closed/aborted/internal-error signal was reviewed as
    lifecycle noise in the same failure context.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 184: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Ramanujan the 6th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=184 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` panic marker with
    `should have direct metric with apply_sync`.
  - 5,921 forwarded-stop peer-alias errors were present, including 5,732
    `no available capacity` logs and 206 `channel closed` logs.
  - no `src/ctx.rs:34` invalid service-id evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no open_bi, connect-answer, broadcast-data, connection-lost,
    path-not-found, or WARN logs.
- Duplicate mapping: ISSUE-063 and ISSUE-170.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 stale-sync and ISSUE-170 forwarded-stop storm evidence without
  adding a new issue.

### Cycle after ISSUE-204 no-new cycle 183: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Dalton the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=183 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - two channel-closed logs and two closed-by-peer logs were reviewed as
    lifecycle fallout after connection-task panics, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 182: valid random duplicate stale sync and large PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Maxwell the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=182 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 34,201 `forward peer stopped over peer alias` logs, 31,775
    `no available capacity` logs, and 2,759 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 88 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - three transport lifecycle logs were reviewed as storm/teardown fallout with
    no independent failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 181: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Descartes the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=181 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - six channel-closed logs and four connection-lost logs were reviewed as
    lifecycle fallout after connection-task panics, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 180: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Rawls the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=180 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - four connection-lost logs were reviewed as lifecycle fallout around the
    same disconnect/routing race, with no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 179: broad random duplicate invalid service and send-to-main panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Galileo the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=179 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/peer.rs:92` panic marker with
    `should send to main: SendError { .. }`.
  - two channel-closed logs and two closed-by-peer logs were reviewed as
    lifecycle fallout after task panics, with no separate failed invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-053, unchecked inbound out-of-range service ids can index
    past the fixed service table.
  - secondary: ISSUE-139, peer connect-error reporting can panic when the main
    receiver is already closed during shutdown.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 and ISSUE-139 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 178: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Averroes the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=178 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 7,029 `forward peer stopped over peer alias` logs, 5,976
    `no available capacity` logs, and 1,068 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - five transport lifecycle logs were reviewed as storm/teardown fallout with
    no independent failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 177: broad random duplicate invalid service panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Dewey the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=177 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - five `src/ctx.rs:34` panic markers with
    `index out of bounds: the len is 256 but the index is 256`.
  - five channel-closed logs and five connection-lost or closed-by-peer logs
    were reviewed as lifecycle fallout after the connection task panics, with
    no separate failed invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 176: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Wegener the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=176 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 6,862 `forward peer stopped over peer alias` logs, 4,800
    `no available capacity` logs, and 2,071 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - two connection-lost lifecycle logs were reviewed as storm/teardown fallout
    with no independent failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 175: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Boole the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=175 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed log and one connection-lost log were reviewed as
    lifecycle fallout after the connection task panic, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 174: valid random duplicate stale sync and send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Lorentz the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=174 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` panic marker with
    `should send to main: SendError { .. }`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-139, peer connect-error reporting can panic when the main
    receiver is already closed during shutdown.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-139 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 173: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Avicenna the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=173 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed log and one connection-lost log were reviewed as
    lifecycle fallout after the connection task panic, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 172: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Meitner the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=172 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 5,131 `forward peer stopped over peer alias` logs and 5,135
    `no available capacity` logs show the known peer-alias
    stop-forwarding/backpressure storm.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no channel-closed, broadcast-data, connect-answer, open_bi,
    endpoint/transport lifecycle, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 171: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Russell the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=171 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed log and one connection-lost log were reviewed as
    lifecycle fallout after the connection task panic, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 170: valid random duplicate stale sync and large PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Leibniz the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=170 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 155,208 `forward peer stopped over peer alias` logs, 152,036
    `no available capacity` logs, and 5,006 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 401 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - four `answer open_bi got error internal channel error` logs were reviewed
    as lifecycle/storm fallout with no independent failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, endpoint/transport lifecycle, WARN, or path-not-found
    logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 169: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Peirce the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=169 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34` panic marker with
    `index out of bounds: the len is 256 but the index is 256`.
  - one channel-closed log and one connection-lost log were reviewed as
    lifecycle fallout after the connection task panic, with no separate failed
    invariant.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 168: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Dirac the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=168 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - two connection-lost logs were reviewed as lifecycle fallout around the same
    disconnect/routing race, with no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 167: broad random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Popper the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=167 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 4,153 `forward peer stopped over peer alias` logs and 4,154
    `no available capacity` logs show the known peer-alias
    stop-forwarding/backpressure storm.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no channel-closed, broadcast-data, connect-answer, open_bi,
    endpoint/transport lifecycle, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 166: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Mendel the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=166 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - 7,848 `forward peer stopped over peer alias` logs, 7,151
    `no available capacity` logs, and 732 `channel closed` logs show the known
    peer-alias stop-forwarding/backpressure storm.
  - six `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping:
  - primary: ISSUE-063, stale `PeerData::Sync` can outlive the direct metric
    and panic in `RouterTable::apply_sync`.
  - secondary: ISSUE-170, forwarded `PeerStopped` messages can storm through
    peer aliases without dedupe, TTL, or tombstone suppression.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 165: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Godel the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=165 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 164: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Plato the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=164 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 163: broad random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Anscombe the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=163 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 4,100 `forward peer stopped over peer alias` logs, 3,974
    `no available capacity` logs, and 134 `channel closed` logs show the known
    peer-alias stop-forwarding/backpressure storm.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, connect-answer, open_bi, WARN, or path-not-found logs;
    two endpoint/transport lifecycle lines are teardown fallout without a
    separate invariant.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 162: valid random duplicate stale sync, send-to-main, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Harvey the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=162 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` send-to-main panic marker with
    `should send to main: SendError`.
  - 17,201 `forward peer stopped over peer alias` logs, 17,436
    `no available capacity` logs, and 32 `channel closed` logs show the known
    peer-alias stop-forwarding/backpressure storm.
  - 45 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the `src/peer.rs:92` panic maps directly to ISSUE-139: early
    `PeerConnectError` reporting can panic on `main_tx` send after the main
    loop has already shut down.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063, ISSUE-139, and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 161: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Boyle the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=161 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one invalid-service panic marker at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - one closed-by-peer log and one channel-closed log were reviewed as
    teardown fallout after the background panic.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 160: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Curie the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=160 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 159: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Carson the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=159 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - two closed-by-peer logs and two channel-closed logs were reviewed as
    teardown fallout after the background panics.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 158: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Carver the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=158 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 14,052 `forward peer stopped over peer alias` logs, 13,158
    `no available capacity` logs, and 955 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 18 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs; two
    endpoint/transport lifecycle lines are teardown fallout without a separate
    invariant.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 157: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `James the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=157 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one invalid-service panic marker at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - one closed-by-peer log and one channel-closed log were reviewed as
    teardown fallout after the background panic.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 156: valid random duplicate stale sync and send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `McClintock the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=156 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:92` send-to-main panic marker with
    `should send to main: SendError`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the `src/peer.rs:92` panic maps directly to ISSUE-139: early
    `PeerConnectError` reporting can panic on `main_tx` send after the main
    loop has already shut down.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-139 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 155: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Laplace the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=155 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76` stale-sync panic marker with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, endpoint/transport lifecycle, WARN, or
    path-not-found logs.
- Duplicate mapping: ISSUE-063.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 154: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Volta the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=154 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 12,921 `forward peer stopped over peer alias` logs, 11,442
    `no available capacity` logs, and 1,599 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 52 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs; four
    endpoint/transport lifecycle lines are teardown fallout without a separate
    invariant.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 153: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hilbert the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=153 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - 14 invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; seven channel-closed and seven closed/lost
    peer lines are lifecycle teardown fallout without storm markers or a
    separate invariant.
- Duplicate mapping: ISSUE-053.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 152: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Tesla the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=152 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 12,523 `forward peer stopped over peer alias` logs, 11,163
    `no available capacity` logs, and 1,401 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 9 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, path-not-found, or endpoint lifecycle
    logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 151: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Ohm the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=151 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - eight invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the four `channel closed` and four
    closed-by-peer lines are lifecycle teardown fallout without storm markers
    or a separate failed invariant.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 150: valid random duplicate stale sync, send-to-main, and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Hume the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=150 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - six `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - two send-to-main panic markers at `src/peer.rs:92:104` with
    `should send to main: SendError`.
  - 6,994 `forward peer stopped over peer alias` logs, 6,332
    `no available capacity` logs, and 719 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 21 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the `src/peer.rs:92` panic maps directly to ISSUE-139:
    background peer tasks still `expect` successful main-loop reporting after
    the main receiver is closed during shutdown.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063, ISSUE-139, and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 149: broad random duplicate invalid service and send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Bacon the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=149 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - two send-to-main panic markers at `src/peer.rs:133:113` with
    `should send to main: SendError`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the single `channel closed` and single
    closed-by-peer line are lifecycle teardown fallout without storm markers
    or a separate failed invariant.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
  - secondary: the `src/peer.rs:133` panic maps directly to ISSUE-139:
    background peer tasks still `expect` successful main-loop reporting after
    the main receiver is closed during shutdown.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 and ISSUE-139 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 148: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kant the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=148 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, WARN, or path-not-found logs; the six
    closed-by-peer lines are lifecycle teardown fallout without storm markers
    or a separate failed invariant.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 147: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Raman the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=147 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, ERROR, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 146: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Noether the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=146 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 21,216 `forward peer stopped over peer alias` logs, 17,777
    `no available capacity` logs, and 3,535 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 145: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Darwin the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=145 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the single `channel closed` and single
    closed-by-peer line are lifecycle fallout without storm markers or a
    separate failed invariant.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 144: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Copernicus the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=144 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 15,791 `forward peer stopped over peer alias` logs, 14,552
    `no available capacity` logs, and 1,346 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 20 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 143: broad random duplicate invalid service and stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Pasteur the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=143 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two invalid-service panic markers at `src/ctx.rs:34` with
    `index out of bounds: the len is 256 but the index is 256`.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the single `channel closed` and single
    closed-by-peer line are lifecycle fallout without storm markers or a
    separate failed invariant.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
  - secondary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 and ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 142: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Schrodinger the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=142 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - 8,080 `forward peer stopped over peer alias` logs, 7,110
    `no available capacity` logs, and 1,011 `channel closed` logs show the
    known peer-alias stop-forwarding/backpressure storm.
  - 23 `broadcast data over peer alias` logs were reviewed as storm fallout
    under ISSUE-170 because they appeared inside the same peer-alias
    backpressure storm and had no separate failed invariant.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no connect-answer, open_bi, WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
  - secondary: the forwarded-stop/no-capacity/channel-closed storm maps
    directly to ISSUE-170: `PeerStopped` forwarding has no dedupe, TTL, or
    tombstone suppression and amplifies under peer-alias backpressure.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 141: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Pauli the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=141 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76` stale-sync panic markers with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` invalid-service panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, WARN, or path-not-found logs; the seven
    connection-lost/internal-error lines are teardown fallout, not a separate
    accepted invariant.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` route state can outlive the direct metric required by
    `RouterTable::apply_sync`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 140: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Franklin the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=140 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - five `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the five `channel closed` and five
    closed/lost peer lines are teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 139: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Lovelace the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=139 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - eight `src/router.rs:76` / `should have direct metric with apply_sync`
    markers from four direct stale-sync panics.
  - 7,151 `no available capacity`, 2,258 `channel closed`, and 9,402
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 138: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Feynman the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=138 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the two `channel closed` and two
    closed-peer lines are teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 137: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Kuhn the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=137 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - four `src/router.rs:76` / `should have direct metric with apply_sync`
    markers from two direct stale-sync panics.
  - 34,373 `no available capacity`, 829 `channel closed`, and 34,841
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - one `answer open_bi got error internal channel error` line was reviewed as
    lifecycle/storm fallout, not a standalone new issue, because there is no
    separate panic site, assertion, data-loss invariant, or distinct root
    cause.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 136: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Aquinas the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=136 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the two `channel closed` and two
    closed-peer lines are teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 135: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Beauvoir the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=135 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - eight `src/router.rs:76` / `should have direct metric with apply_sync`
    markers from four direct stale-sync panics.
  - 7,399 `no available capacity`, 1,380 `channel closed`, and 8,779
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, open_bi, connect-answer, path-not-found, or WARN logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 134: broad random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Fermat the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=134 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - two `src/router.rs:76` / `should have direct metric with apply_sync`
    markers.
  - 169,759 `no available capacity`, 644 `channel closed`, and 168,451
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - 113 `broadcast data over peer alias` backpressure lines were reviewed as
    duplicate storm fallout under ISSUE-170, not a standalone new issue,
    because there is no distinct invariant failure or root cause in this
    evidence.
  - six `answer open_bi got error internal channel error` lines were reviewed
    as lifecycle/teardown noise accompanying the storm, not a new issue.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 133: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Hypatia the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=133 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - four `src/router.rs:76` / `should have direct metric with apply_sync`
    markers from two direct panic sites plus the fuzz assertion.
  - 8,057 `no available capacity`, 799 `channel closed`, and 8,836
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - seven `broadcast data over peer alias` backpressure lines were reviewed as
    duplicate storm fallout, not a standalone new issue, because there is no
    separate failed invariant, panic site, data-loss assertion, or distinct
    root cause in this evidence.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 132: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Banach the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=132 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the four `channel closed` and four
    closed/lost peer lines are teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 131: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Heisenberg the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=131 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - four `src/router.rs:76` / `should have direct metric with apply_sync`
    markers, including a second tokio worker panic.
  - 10,935 `no available capacity`, 136 `channel closed`, and 11,061
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 130: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Singer the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=130 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the single `channel closed` and single
    `closed by peer` lines are teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 129: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Erdos the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=129 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/task panic.
  - two `src/router.rs:76` / `should have direct metric with apply_sync`
    markers.
  - 5,636 `no available capacity`, 1,125 `channel closed`, and 6,732
    `forward peer stopped over peer alias` logs show the same forwarded-stop
    storm shape.
  - no `src/ctx.rs:34` out-of-range service-id panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` can survive after a direct route is removed, then
    `apply_sync` asserts that the missing direct metric exists.
  - secondary: the stop-forwarding capacity/channel-closed storm maps to
    ISSUE-170: forwarded `PeerStopped` events lack dedupe, TTL, or tombstone
    suppression and can amplify under churn.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 128: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Planck the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=128 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the two `channel closed` and one
    `closed by peer` lines are small teardown fallout, not ISSUE-170 storm
    evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 127: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Gauss the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=127 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 9,040 `no available capacity`,
    1,397 `channel closed`, 10,324 `forward peer stopped over peer alias` logs,
    19 `broadcast data over peer alias` logs, and 10,437 ERROR lines.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no WARN, `path not found`, connect-answer, open_bi, or connection teardown
    logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 126: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Sartre the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=126 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the three `channel closed`, two
    `connection lost`, and one `closed by peer` lines are small teardown
    fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panics map directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 125: valid random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Goodall the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=125 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data,
    connect-answer, open_bi, connection teardown, ERROR, WARN, or
    path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 124: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Archimedes the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=124 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, open_bi,
    WARN, or path-not-found logs; the two `channel closed`, two
    `closed by peer`, and one `aborted by peer` lines are small teardown
    fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panics map directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 123: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Lagrange the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=123 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 11,573 `no available capacity`,
    665 `channel closed`, 12,190 `forward peer stopped over peer alias` logs,
    and 12,239 ERROR lines.
  - one `[PeerConnectionInternal] answer open_bi got error internal channel
    error` line from `src/peer/peer_internal.rs:167`; reviewer classified it
    as a dropped internal response channel during cancellation/shutdown, not
    distinct accepted issue evidence.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no WARN, `broadcast data over peer alias`, `path not found`, or
    connect-answer logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 122: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Nietzsche the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=122 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, WARN, or
    path-not-found logs; the one `connection lost` and one `channel closed` log
    are small teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 121: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Poincare the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=121 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 5,639 `no available capacity`, 816
    `channel closed`, 6,441 `forward peer stopped over peer alias` logs, and
    6,460 ERROR lines.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no broadcast-data, connect-answer, WARN, or `path not found` logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 120: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Sagan the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=120 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, connect-answer, WARN, or
    path-not-found logs; the one `connection lost` and one `channel closed` log
    are small teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 119: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Newton the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `src/lib.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=119 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 26,040 `no available capacity`,
    2,358 `channel closed`, 28,230 `forward peer stopped over peer alias`
    logs, 10 `broadcast data over peer alias` logs, and 28,403 ERROR lines.
  - one `[P2pNetwork] send connect answer got error Ok(())` line from
    `src/lib.rs:326`; reviewer classified it as dropped requester oneshot
    lifecycle noise, not distinct accepted issue evidence.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no WARN or `path not found` logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 118: broad random duplicate invalid service panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Faraday the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=118 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - no `src/router.rs:76` stale-sync panic evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, forwarded-stop, broadcast-data, WARN, or path-not-found
    logs; the three `channel closed` and connection lost/closed logs are small
    teardown fallout, not ISSUE-170 storm evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panics map directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 117: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Epicurus the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=117 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 4,286 `no available capacity`, 423
    `channel closed`, 4,670 `forward peer stopped over peer alias` logs, 3
    `broadcast data over peer alias` logs, and 4,714 ERROR lines.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no WARN or `path not found` logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 116: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Aristotle the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=116 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected a background connection/service task
    panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - no `src/ctx.rs:34` index-out-of-bounds evidence.
  - no `src/peer.rs:89/92/130/133` `should send to main` evidence.
  - no no-capacity, channel-closed, forwarded-stop, broadcast-data, ERROR,
    WARN, or path-not-found logs.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 115: valid random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Helmholtz the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=115 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - heavy `PeerStopped` backpressure storm: 4,854 `no available capacity`, 31
    `channel closed`, 4,885 `forward peer stopped over peer alias` logs, and
    4,888 ERROR lines.
  - no `src/ctx.rs:34` index-out-of-bounds evidence and no
    `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 114: broad random duplicate stale sync and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Hooke the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=114 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - three `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - heavy backpressure storm: 6,202 `no available capacity`, 1,454
    `channel closed`, 7,615 `forward peer stopped over peer alias` logs, plus
    30 `broadcast data over peer alias` logs.
  - no `src/ctx.rs:34` index-out-of-bounds evidence and no
    `src/peer.rs:89/92/130/133` `should send to main` evidence.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reached `RouterTable::apply_sync` after the direct route
    for that connection had been removed.
  - secondary: the forwarded `PeerStopped` no-capacity/channel-closed storm maps
    to ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding a new issue.

### Cycle after ISSUE-204 no-new cycle 113: broad random duplicate invalid service and send panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Chandrasekhar the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=113 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - one `src/peer.rs:92:104` panic and one `src/peer.rs:133:113` panic with
    `should send to main: SendError { .. }`.
  - reviewer found no `src/router.rs:76` stale-sync panic, no forwarded
    `PeerStopped` no-capacity/channel-closed storm, and classified the six
    ordinary `channel closed` network send logs as shutdown fallout rather than
    separate ISSUE-170 evidence.
- Duplicate mapping:
  - primary: the `src/ctx.rs:34` panics map directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
  - secondary: the `src/peer.rs:92` incoming `incoming.await` error path and
    `src/peer.rs:133` outgoing `connecting.await` error path map to ISSUE-139:
    early `PeerConnectError` reporting after main-loop shutdown uses unchecked
    `main_tx.send(...).await.expect("should send to main")`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 and ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 112: valid-action duplicate stale sync send panic and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Ptolemy the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=112 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - one `src/peer.rs:133:113` panic with
    `should send to main: SendError { .. }`.
  - the same log contains 5,278 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 440 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic and no separate
    new accepted issue.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - secondary: the `src/peer.rs:133` outgoing `connecting.await` error path
    maps to ISSUE-139: early `PeerConnectError` reporting after main-loop
    shutdown uses an unchecked `main_tx.send(...).await.expect("should send to main")`.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063, ISSUE-139, and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 111: sanitized churn duplicate outgoing send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Bohr the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=111 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/peer.rs:133:113` panic with
    `should send to main: SendError { .. }`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/router.rs:76` stale-sync panic, no forwarded `PeerStopped`
    no-capacity/channel-closed storm, and no separate new accepted issue.
- Duplicate mapping:
  - the `src/peer.rs:133` outgoing `connecting.await` error path maps to
    ISSUE-139: early `PeerConnectError` reporting after main-loop shutdown
    uses an unchecked `main_tx.send(...).await.expect("should send to main")`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 110: steady-valid clean pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Herschel the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=110 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed; 0 failed`; the steady-valid fuzz run completed
    in 15.60s.
  - no `panicked at` lines, no failed test result, no `ERROR` or `WARN`
    markers, and no known noisy failure markers: invalid service-id panic,
    stale-sync panic, send-to-main panic, no-capacity/channel-closed
    forwarded-stop storm, or path-not-found marker.
- Duplicate mapping:
  - none; no accepted issue evidence and no ISSUE-205.
- Root-cause summary impact: no new root cause and no fix proposal change; this
  is additional clean steady-valid coverage for 8 nodes and 2400 valid random
  actions.

### Cycle after ISSUE-204 no-new cycle 109: valid churn duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Turing the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=109 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no `src/peer.rs`
    send-to-main panic, no forwarded `PeerStopped` no-capacity/channel-closed
    storm, no path-not-found marker, and no separate new accepted issue.
- Duplicate mapping:
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 108: broad random duplicate invalid service-id panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Ramanujan the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=108 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - four `src/ctx.rs:34:9` panics with
    `index out of bounds: the len is 256 but the index is 256`.
  - reviewer found no `src/router.rs:76` stale-sync panic, no `src/peer.rs`
    send-to-main panic, no no-capacity/path-not-found storm, and classified the
    four ordinary `channel closed` send logs as shutdown fallout rather than
    separate ISSUE-139 or ISSUE-170 evidence.
- Duplicate mapping:
  - the `src/ctx.rs:34` panics map directly to ISSUE-053: inbound out-of-range
    `P2pServiceId(256)` reaches unchecked service table indexing in
    `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 107: valid-action duplicate stale sync and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Zeno the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=107 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - the same log contains 7,115 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 1,782 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic and no
    `src/peer.rs` send-to-main panic in this log.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 106: sanitized churn duplicate send panic and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Halley the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=106 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/peer.rs:92:104` panic with
    `should send to main: SendError { .. }`.
  - the same log contains 6,511 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 46 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic and no
    `src/router.rs:76` stale-sync panic in this log.
- Duplicate mapping:
  - primary: the `src/peer.rs:92` incoming `incoming.await` error path maps to
    ISSUE-139: early `PeerConnectError` reporting after main-loop shutdown uses
    an unchecked `main_tx.send(...).await.expect("should send to main")`.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 105: steady-valid clean pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Ampere the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=105 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed; 0 failed`; the steady-valid fuzz run completed
    in 16.20s.
  - no `panicked at` lines, no failed test result, no `ERROR` or `WARN`
    markers, and no known noisy failure markers: invalid service-id panic,
    stale-sync panic, send-to-main panic, no-capacity/channel-closed
    forwarded-stop storm, or path-not-found marker.
- Duplicate mapping:
  - none; no accepted issue evidence and no ISSUE-205.
- Root-cause summary impact: no new root cause and no fix proposal change; this
  is additional clean steady-valid coverage for 8 nodes and 2400 valid random
  actions.

### Cycle after ISSUE-204 no-new cycle 104: valid churn duplicate incoming send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Mencius the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=104 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/peer.rs:92:104` panic with
    `should send to main: SendError { .. }`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/router.rs:76` stale-sync panic, and no forwarded `PeerStopped`
    no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/peer.rs:92` incoming `incoming.await` error path maps to
    ISSUE-139: early `PeerConnectError` reporting after main-loop shutdown
    uses an unchecked `main_tx.send(...).await.expect("should send to main")`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 103: broad random duplicate invalid service-id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kepler the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=103 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - reviewer found no `src/router.rs:76` stale-sync panic, no `src/peer.rs`
    send-to-main panic, no no-capacity/path-not-found storm, and only one
    ordinary `channel closed` log that is not separate ISSUE-139 or ISSUE-170
    evidence.
- Duplicate mapping:
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` reaches unchecked service table indexing
    in `SharedCtxInternal::get_service`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 102: sanitized churn duplicate incoming send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Gibbs the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=102 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/peer.rs:89:112` panic with
    `should send to main: SendError { .. }`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/router.rs:76` stale-sync panic, no `src/peer.rs:92`,
    `src/peer.rs:130`, or `src/peer.rs:133` send-to-main panic, and no
    forwarded `PeerStopped` no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/peer.rs:89` incoming `accept_bi()` error path maps to ISSUE-139:
    early `PeerConnectError` reporting after main-loop shutdown uses an
    unchecked `main_tx.send(...).await.expect("should send to main")`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 101: valid-action duplicate stale sync and stop-forwarding storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Einstein the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - the same log contains 4,644 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 682 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic and no
    `src/peer.rs` send-to-main panic in this log.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 100: steady-valid pass with teardown noise

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Mill the 5th`, forked subagent review, confirmed `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=100 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed; 0 failed`; the steady-valid fuzz run completed
    in 17.04s.
  - no `panicked at` lines, no failed test result, no `WARN` markers, and no
    known noisy failure markers: `no available capacity`, `channel closed`, or
    `path not found`.
  - six `ERROR` lines appeared during teardown/internal endpoint handling: two
    `answer open_bi got error internal channel error`, one peer
    `connection lost`, and three `endpoint driver future was dropped` lines.
    Reviewer found these insufficient for a new accepted issue without
    failing-test evidence tying them to an invariant violation, data loss,
    panic, retry storm, or harness failure.
- Duplicate mapping:
  - none; no accepted issue evidence and no ISSUE-205.
- Root-cause summary impact: no new root cause and no fix proposal change; this
  is additional steady-valid pass coverage with reviewed lifecycle noise.

### Cycle after ISSUE-204 no-new cycle 99: broad random duplicate invalid service-id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Parfit the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=99 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - reviewer found no `src/router.rs:76` stale-sync panic, no `src/peer.rs`
    send-to-main panic, and no forwarded `PeerStopped`
    no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    `P2pServiceId(256)` is not rejected before indexing the fixed-size service
    context array.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 98: sanitized churn duplicate outgoing send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Arendt the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=98 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - one `src/peer.rs:133:113` panic with
    `should send to main: SendError { .. }`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/router.rs:76` stale-sync panic, no `src/peer.rs:92` or
    `src/peer.rs:130` send-to-main panic, and no forwarded `PeerStopped`
    no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/peer.rs:133` panic maps directly to ISSUE-139: early outgoing
    `PeerConnectError` reporting can panic after the main loop receiver is
    closed.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 97: valid-action duplicate stale sync and large stop-forwarding storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Hegel the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=97 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - the same log contains 12,403 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 1,318 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - five `broadcast data over peer alias got error channel closed` entries
    appeared near shutdown; reviewer found them insufficient to establish a
    separate new root cause without an isolated delivery-loss or panic test.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/peer.rs` send-to-main panic, and no `should send to main` evidence in
    this log.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 96: clean steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Confucius the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=96 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed; 0 failed`; the steady-valid fuzz run completed
    in 16.26s.
  - no `panicked at` lines, no failed test result, no `ERROR`/`WARN` markers,
    and no known noisy failure markers: `no available capacity`,
    `channel closed`, or `path not found`.
- Duplicate mapping:
  - none; the reviewer found no failing evidence to map to ISSUE-053,
    ISSUE-063, ISSUE-139, ISSUE-170, or a new ISSUE-205.
- Root-cause summary impact: no new root cause and no fix proposal change; this
  is additional clean steady-valid fuzz coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 95: broad random duplicate stale sync and stop-forwarding storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Locke the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=95 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183:5` detected background connection/service task
    panics.
  - two `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - the same log contains 4,496 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 260 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/peer.rs` send-to-main panic, and no `should send to main` evidence in
    this log.
- Duplicate mapping:
  - primary: the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 94: sanitized churn duplicate peer-connect panic and stop-forwarding storm

- Result: no accepted non-duplicate issue.
- Reviewer: cycle 94 reviewer task, confirmed `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `src/utils.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=94 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372:5` detected background connection/service task
    panics.
  - two `src/peer.rs:133:113` panics with
    `should send to main: SendError { .. }`.
  - the same log contains 207 exact
    `forward peer stopped over peer alias got error no available capacity`
    entries and 9 exact
    `forward peer stopped over peer alias got error channel closed` entries.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/router.rs:76` stale-sync panic, and no `src/peer.rs:92` or
    `src/peer.rs:130` send-to-main panic in this log.
- Duplicate mapping:
  - primary: the `src/peer.rs:133` panics map directly to ISSUE-139: early
    outgoing `PeerConnectError` reporting can panic after the main loop
    receiver is closed.
  - secondary: the forwarded-stop no-capacity/channel-closed storm maps to
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes and can amplify shutdown churn into repeated failed control sends.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 and ISSUE-170 sanitized-churn evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 93: valid-action duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: cycle 93 reviewer task, confirmed `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=93 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected background connection/service task
    panics.
  - three `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - reviewer found no invalid-service `src/ctx.rs:34` panic, no
    `src/peer.rs` send-to-main panic, and no forwarded `PeerStopped`
    no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 92: clean steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Bernoulli the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=92 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed; 0 failed`; the steady-valid fuzz run completed
    in 15.95s.
  - no `panicked at` lines, no failed test result, no `ERROR`/`WARN` markers,
    and no known noisy failure markers: `no available capacity`,
    `channel closed`, or `path not found`.
- Duplicate mapping:
  - none; the reviewer found no failing evidence to map to ISSUE-053,
    ISSUE-063, ISSUE-139, ISSUE-170, or a new ISSUE-205.
- Root-cause summary impact: no new root cause and no fix proposal change; this
  is additional clean steady-valid fuzz coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 91: broad random duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Euler the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=91 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected background task panics.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - two closed-by-peer `ERROR` logs appeared, but reviewer found no
    invalid-service `src/ctx.rs` panic, no `src/peer.rs` send-to-main panic,
    and no forwarded `PeerStopped` no-capacity/channel-closed storm in this
    log.
- Duplicate mapping:
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 90: sanitized churn duplicate outgoing send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kierkegaard the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=90 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372` detected background connection task panics.
  - three `src/peer.rs:133:113` panics and one `src/peer.rs:130:121` panic,
    all with `should send to main: SendError { .. }`.
  - reviewer found no evidence for ISSUE-053, ISSUE-063, or ISSUE-170 in this
    log.
- Duplicate mapping:
  - the `src/peer.rs:133` and `src/peer.rs:130` panics map directly to
    ISSUE-139: early outbound `PeerConnectError` reporting can panic after the
    main event loop has closed because the spawned connection task uses
    `main_tx.send(...).await.expect("should send to main")`.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 89: valid-action duplicate stale sync without stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Euclid the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=89 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected background task panics.
  - two `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - reviewer found no `src/ctx.rs` invalid-service panic, no `src/peer.rs`
    send-to-main panic, and no forwarded `PeerStopped`
    no-capacity/channel-closed storm in this log.
- Duplicate mapping:
  - the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 88: clean steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Cicero the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=88 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no failed assertion,
    no `ERROR` or `WARN` logs, and no no-capacity, channel-closed, or
    path-not-found markers.
- Root-cause summary impact: no new root cause; this clean steady-valid fuzz
  run is additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 87: broad random duplicate invalid service id

- Result: no accepted non-duplicate issue.
- Reviewer: `Jason the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=87 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected a background task panic.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - one later `try send message ... error channel closed` line was reviewed as
    follow-on lifecycle/error-send noise, not a distinct accepted issue.
- Duplicate mapping:
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` indexes the fixed service table.
  - reviewer found no ISSUE-063, ISSUE-139, or ISSUE-170 mapping for this
    cycle.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 86: sanitized churn duplicate stale sync and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Nash the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=86 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372` detected a background task panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - the log also contains 44,181 forwarded `PeerStopped` `no available
    capacity` errors and 2,266 forwarded `PeerStopped` `channel closed` errors.
  - reviewer found no `src/peer.rs` send-to-main panic and no `src/ctx.rs`
    invalid-service panic in this log.
- Duplicate mapping:
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - the forwarded `PeerStopped` storm maps to ISSUE-170's missing
    dedupe/TTL/tombstone suppression for stop forwarding in cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 85: valid-action duplicate stale sync with large stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Linnaeus the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=85 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected a background task panic.
  - one `src/router.rs:76:66` panic with
    `should have direct metric with apply_sync`.
  - the log also contains 247,968 forwarded `PeerStopped` `no available
    capacity` errors and 81 forwarded `PeerStopped` `channel closed` errors.
- Duplicate mapping:
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - the forwarded `PeerStopped` storm maps to ISSUE-170's missing
    dedupe/TTL/tombstone suppression for stop forwarding in cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 84: steady-valid fuzz pass with endpoint teardown noise

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Hubble the 5th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=84 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no failed assertion,
    and no no-capacity, channel-closed, or path-not-found markers.
  - one `ERROR` teardown/lifecycle line reported endpoint internal error
    because the endpoint driver future was dropped.
  - reviewer classified this as lifecycle noise because no fuzz invariant
    failed and prior steady-valid entries have treated similar endpoint teardown
    logs as pass/no-new evidence.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 83: broad random duplicate invalid service, stale sync, and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Huygens the 5th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=83 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected background task panics.
  - one `src/ctx.rs:34:9` panic with
    `index out of bounds: the len is 256 but the index is 256`.
  - three `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - the log also contains 7,029 forwarded `PeerStopped` `no available
    capacity` errors and 744 forwarded `PeerStopped` `channel closed` errors.
- Duplicate mapping:
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` indexes the fixed service table.
  - the `src/router.rs:76` panics map directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after direct route
    state is gone.
  - the forwarded `PeerStopped` storm maps to ISSUE-170's missing
    dedupe/TTL/tombstone suppression for stop forwarding in cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-053, ISSUE-063, and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 82: sanitized churn duplicate outgoing peer-connect panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Lagrange the 4th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=82 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:372` detected background connection task panics.
  - the log contains three `src/peer.rs:133:113` panics with
    `should send to main: SendError { .. }`.
  - no forwarded-stop `no available capacity` or `channel closed` storm markers
    appeared in this run.
- Duplicate mapping:
  - the `src/peer.rs:133` panics map directly to ISSUE-139: early outgoing
    `PeerConnectError` reporting can panic after the main event loop has
    closed because the spawned connection task uses
    `main_tx.send(...).await.expect("should send to main")`.
  - reviewer explicitly found no ISSUE-170 mapping for this cycle because the
    forwarded-stop storm markers were absent.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 81: valid-action fuzz duplicate stale sync and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Ohm the 4th`, forked subagent review, confirmed
  `DUPLICATE/NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=81 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Evidence summary:
  - exit status 101; `0 passed; 1 failed`; the fuzz assertion at
    `src/tests/fuzz.rs:183` detected a background task panic.
  - the log contains two `src/router.rs:76:66` panics with
    `should have direct metric with apply_sync`.
  - the log also contains 6,250 forwarded `PeerStopped` `no available capacity`
    errors and 2,077 forwarded `PeerStopped` `channel closed` errors.
- Duplicate mapping:
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after the direct
    route/metric has already been removed.
  - the forwarded `PeerStopped` error storm maps to ISSUE-170's missing
    dedupe/TTL/tombstone suppression for stop forwarding in cyclic meshes.
- Root-cause summary impact: no new root cause; this strengthens existing
  ISSUE-063 and ISSUE-170 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 80: steady-valid fuzz pass with teardown noise

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Laplace the 4th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=80 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no `FAILED` lines, and
    no no-capacity, channel-closed, or path-not-found markers.
  - three `ERROR` teardown lines reported endpoint-driver drop, connection
    lost, and closed-by-peer conditions; reviewer classified them as teardown
    noise because no fuzz invariant failed and no distinct existing issue
    mapping was established.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 79: sanitized churn duplicate outgoing send panic and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Meitner the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/peer/peer_internal.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=79 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - five background panics occurred at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - 221,765 `forward peer stopped over peer alias got error no available
    capacity` logs and 2,660 `forward peer stopped over peer alias got error
    channel closed` logs were emitted.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - the `src/peer.rs:133` panics map directly to ISSUE-139: early outgoing
    `PeerConnectError` reporting can panic after main-loop shutdown because
    `PeerConnection::new_connecting` reports early errors with
    `main_tx.send(...).await.expect("should send to main")`. Existing score:
    63/100.
  - the PeerStopped forwarding storm is secondary amplification evidence for
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes. Existing score: 62/100.
  - sanitized churn excludes invalid service ids and forged `PeerStopped`
    messages, so this run narrows the primary failure to shutdown
    send-to-main panic plus already-known stop forwarding amplification.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 78: valid-action fuzz duplicate stale sync and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Kepler the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=78 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - 5,826 `forward peer stopped over peer alias got error no available
    capacity` logs and 351 `forward peer stopped over peer alias got error
    channel closed` logs were emitted.
  - one background panic occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after the direct
    route/metric has already been removed. Existing score: 72/100.
  - the PeerStopped forwarding storm is secondary amplification evidence for
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes. Existing score: 62/100.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 77: broad random fuzz duplicate invalid service id

- Result: no accepted non-duplicate issue.
- Reviewer: `Feynman the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/ctx.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=77 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-053: inbound out-of-range
    `P2pServiceId(256)` indexes the fixed service table and kills the peer
    connection task. Existing score: 84/100.
  - the later `try send message ... error channel closed` log is consistent
    with the connection task dying after the invalid-service panic and does not
    establish a distinct root cause.
- Root-cause summary impact: no new root cause; this broad random fuzz run
  strengthens existing ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 76: steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Mendel the 4th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=76 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no `FAILED` lines, no
    `ERROR`/`WARN` log lines, and no no-capacity, channel-closed, or
    path-not-found markers.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional clean baseline coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 75: valid-action fuzz duplicate stale sync and incoming send panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hume the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=75 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - one Tokio worker panic occurred at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after the direct
    route/metric has already been removed. Existing score: 72/100.
  - the `src/peer.rs:92` panic maps directly to ISSUE-139: early incoming
    `PeerConnectError` reporting can panic after main-loop shutdown. Existing
    score: 63/100.
  - the connection lost/closed logs are consistent with those lifecycle races
    and do not establish a distinct root cause.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 and ISSUE-139 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 74: sanitized churn duplicate incoming send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `McClintock the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=74 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - 18 `ERROR` log lines around closed, connection-lost, shutdown, and refused
    connections were emitted during churn.
  - one background panic occurred at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - this maps directly to ISSUE-139: early `PeerConnectError` reporting can
    panic after main-loop shutdown because
    `PeerConnection::new_incoming` reports early connection failures with
    `main_tx.send(...).await.expect("should send to main")`. Existing score:
    63/100.
  - sanitized churn excludes invalid service ids and forged `PeerStopped`
    messages, so the surrounding closed/refused/lost connection logs narrow to
    the same shutdown/error-reporting lifecycle race rather than a distinct
    root cause.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 73: valid churn duplicate incoming send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `James the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=73 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - this maps directly to ISSUE-139: early `PeerConnectError` reporting can
    panic after main-loop shutdown because
    `PeerConnection::new_incoming` reports early errors with
    `main_tx.send(...).await.expect("should send to main")`. Existing score:
    63/100.
  - the surrounding closed/refused/aborted connection logs are consistent with
    the same shutdown/error-reporting path and do not establish a distinct root
    cause.
- Root-cause summary impact: no new root cause; this valid-churn fuzz run
  strengthens existing ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 72: broad random fuzz duplicate invalid service and stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Aquinas the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/ctx.rs`
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=72 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - one background panic occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` indexes the fixed service table. Existing
    score: 84/100.
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after the direct
    route/metric has already been removed. Existing score: 72/100.
  - the single `try send message ... error channel closed` log is secondary
    noise and does not establish a distinct root cause.
- Root-cause summary impact: no new root cause; this broad random fuzz run
  strengthens existing ISSUE-053 and ISSUE-063 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 71: steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Copernicus the 4th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=71 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no `FAILED` lines, no
    `ERROR`/`WARN` log lines, and no no-capacity, channel-closed, or
    path-not-found markers.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional clean baseline coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 70: sanitized churn duplicate stale sync and stop storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Mencius the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=70 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - 56,948 `forward peer stopped over peer alias got error no available
    capacity` logs were emitted before the panic.
  - one background panic occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - the `src/router.rs:76` panic maps directly to ISSUE-063: stale
    `PeerData::Sync` reaches `RouterTable::apply_sync` after the direct
    route/metric has already been removed. Existing score: 72/100.
  - the no-capacity `PeerStopped` storm is secondary amplification evidence for
    ISSUE-170: stop forwarding lacks dedupe/TTL/tombstone suppression in cyclic
    meshes. Existing score: 62/100.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-063 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 69: valid-action fuzz duplicate stale sync panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Kuhn the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=69 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` reaches
    `RouterTable::apply_sync` after the direct route/metric has already been
    removed. Existing score: 72/100.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 68: broad random fuzz duplicate invalid service and shutdown panics

- Result: no accepted non-duplicate issue.
- Reviewer: `Aristotle the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/ctx.rs`
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=68 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - one background panic occurred at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - one Tokio worker panic occurred at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - the `src/ctx.rs:34` panic maps directly to ISSUE-053: inbound
    out-of-range `P2pServiceId(256)` indexes the fixed service table. Existing
    score: 84/100.
  - the `src/peer.rs:133` panic maps directly to ISSUE-139: early
    `PeerConnectError` reporting can panic after main-loop shutdown. Existing
    score: 53/100.
- Root-cause summary impact: no new root cause; this broad random fuzz run
  strengthens existing ISSUE-053 and ISSUE-139 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 67: steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Linnaeus the 4th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=67 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines, no `FAILED` lines, and
    no warnings.
  - one `answer open_bi got error internal channel error` log was treated as
    non-fatal lifecycle noise because no fuzz invariant failed.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 66: valid-action fuzz duplicate stale sync storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Dalton the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=66 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - four background panics occurred at `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` reaches
    `RouterTable::apply_sync` after the direct route/metric has already been
    removed. Existing score: 72/100.
  - 25,635 `forward peer stopped ... no available capacity` logs and 571
    `channel closed` logs are secondary ISSUE-170-style amplification evidence,
    not the primary failing invariant for this run.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 65: sanitized churn duplicate incoming send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Erdos the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=65 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - the primary background panic was `src/peer.rs:92:104` with
    `should send to main: SendError { .. }`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - this maps directly to ISSUE-139: early `PeerConnectError` reporting can
    panic after main loop shutdown. Existing score: 53/100.
  - 8,545 `forward peer stopped ... no available capacity` logs and 1,221
    `channel closed` logs are secondary ISSUE-170-style amplification evidence,
    not the primary failing invariant for this run.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 64: broad random fuzz duplicate invalid service id

- Result: no accepted non-duplicate issue.
- Reviewer: `Euler the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/ctx.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=64 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - two background panics occurred at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-053: inbound out-of-range service ids kill peer
    connection tasks. Existing score: 84/100.
  - two channel-closed send logs were secondary shutdown fallout, not the
    primary failing invariant for this run.
- Root-cause summary impact: no new root cause; this broad random fuzz run
  strengthens existing ISSUE-053 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 63: clean steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Schrodinger the 4th`, forked subagent review, confirmed
  `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=63 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines and no `FAILED` lines.
  - captured log had no `ERROR` or `WARN` lines.
  - reviewer confirmed no accepted new or duplicate issue evidence was present.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 62: valid-action fuzz duplicate stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Carson the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=62 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - the background panic was `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` reaches
    `RouterTable::apply_sync` after the direct route/metric has already been
    removed. Existing score: 72/100.
  - no secondary capacity, channel-closed, or path-not-found storm was observed
    in the extracted counts for this run.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 61: sanitized churn duplicate send-to-main panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Lorentz the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=61 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - the primary background panic was `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:372:5`.
  - this maps directly to ISSUE-139: early `PeerConnectError` reporting can
    panic after main loop shutdown. Existing score: 53/100.
  - 10,003 `forward peer stopped ... no available capacity` logs and 610
    `channel closed` logs are secondary ISSUE-170-style amplification evidence,
    not the primary failing invariant for this run.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 60: broad random fuzz duplicate stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Boole the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=60 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - exit status 101.
  - the background panic was `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` reaches
    `RouterTable::apply_sync` after the direct route/metric has already been
    removed. Existing score: 72/100.
- Root-cause summary impact: no new root cause; this broad random fuzz run
  strengthens existing ISSUE-063 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 59: steady-valid fuzz pass

- Result: pass/no-new; no accepted issue evidence.
- Reviewer: `Singer the 4th`, forked subagent review, confirmed `PASS_NO_NEW`.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=59 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed.
- Evidence summary:
  - exit status 0; `1 passed`; no `panicked at` lines and no `FAILED` lines.
  - teardown logs included one `answer open_bi got error internal channel error`,
    peer connection closed/internal endpoint-dropped errors, and four
    `forward peer stopped ... channel closed` lines.
  - reviewer classified those teardown logs as non-fatal lifecycle noise
    overlapping existing ISSUE-170/RC-6 shutdown and PeerStopped forwarding
    areas, not accepted failing evidence.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  additional pass coverage without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 58: valid-action fuzz duplicate stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Einstein the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=58 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - deterministic rerun exited with status 101.
  - the primary background panics were two `src/router.rs:76:66` failures with
    `should have direct metric with apply_sync`.
  - the fuzz harness then failed at `src/tests/fuzz.rs:183:5`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` reaches
    `RouterTable::apply_sync` after the direct route/metric has already been
    removed. Existing score: 72/100.
  - 63,420 `forward peer stopped ... no available capacity` logs and 838
    `channel closed` logs are secondary ISSUE-170-style amplification evidence,
    not the primary failing invariant for this run.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 57: sanitized churn duplicate open_bi connect-error panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Parfit the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=57 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed.
- Duplicate or too-close symptoms rejected:
  - sanitized churn closed the main loop while an outgoing connection task was
    reporting an early `PeerConnectError`.
  - the primary background panic was `src/peer.rs:130:121` with
    `should send to main: SendError { .. }`, followed by the fuzz harness
    assertion at `src/tests/fuzz.rs:372:5`.
  - this is the `connection.open_bi().await` error branch in
    `PeerConnection::new_connecting`; it shares the same unchecked
    `main_tx.send(...).await.expect("should send to main")` root cause as the
    previously observed `src/peer.rs:133` branch.
  - this maps directly to ISSUE-139: early `PeerConnectError` reporting can
    panic after main loop shutdown. Existing score: 53/100.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 56: pubsub stale leave duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Averroes the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat -- --nocapture`
    failed at `src/service/pubsub_service.rs:1167:9`.
- Duplicate or too-close symptoms rejected:
  - a newer heartbeat confirms `PeerId(2)` as a live remote publisher for the
    channel.
  - a later-delivered stale `PublisherLeaved(channel)` from the same peer
    removes that heartbeat-confirmed membership because pubsub membership
    messages have no freshness, generation, or epoch comparison.
  - this maps directly to ISSUE-155: stale pubsub leave removes membership
    confirmed by newer heartbeat. Existing score: 64/100.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-155 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 55: pubsub stale destroy duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Maxwell the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test stale_pubsub_destroy_must_not_create_phantom_channel -- --nocapture`
    failed at `src/service/pubsub_service.rs:1077:9`.
- Duplicate or too-close symptoms rejected:
  - a stale `PublisherDestroyed` control for an unknown publisher handle
    materializes `PubsubChannelId(77)` in `PubsubService::channels`.
  - destroy handling creates or retains channel bookkeeping instead of treating
    unknown local handles as no-ops.
  - this maps directly to ISSUE-150: stale pubsub destroy controls create
    phantom channel state. Existing score: 58/100.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-150 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 54: pubsub empty channel duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Arendt the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test empty_pubsub_channels_must_be_removed_after_last_local_handle_drops -- --nocapture`
    failed at `src/service/pubsub_service.rs:1056:9`.
- Duplicate or too-close symptoms rejected:
  - repeated local subscriber create/drop cycles over distinct channel ids
    leave fully empty `PubsubChannelState` entries in `PubsubService::channels`.
  - after 1,025 such cycles the service still retained 1,025 empty channel
    entries, exceeding the bounded-resource assertion.
  - this maps directly to ISSUE-108: empty pubsub channel state is retained
    after local handle teardown. Existing score: 60/100.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-108 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 53: pubsub heartbeat batch duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Darwin the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test pubsub_heartbeat_channel_batches_must_be_bounded -- --nocapture`
    failed at `src/service/pubsub_service.rs:1027:9`.
- Duplicate or too-close symptoms rejected:
  - a single inbound `PubsubMessage::Heartbeat` can carry 1,025
    `ChannelHeartbeat` rows.
  - `PubsubService::on_service` deserializes the frame and processes every
    heartbeat entry without a semantic channel-count cap, rejection path, or
    truncation.
  - the run updated 1,025 channel states for one remote peer, exceeding the
    bounded-resource assertion.
  - this maps directly to ISSUE-106: pubsub heartbeat channel batches have no
    service-level row cap. Existing score: 68/100.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-106 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 52: pubsub early remote join duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Gauss the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test early_remote_publisher_join_must_survive_late_local_subscriber_creation -- --nocapture`
    failed at `src/service/pubsub_service.rs:985:9`.
- Duplicate or too-close symptoms rejected:
  - an inbound `PublisherJoined` from `PeerId(2)` arrives before any local
    channel state exists.
  - `PubsubMessage::PublisherJoined` is handled only when
    `self.channels.get_mut(&channel)` succeeds, so the early remote publisher
    membership is silently discarded.
  - later local subscriber creation creates the channel, but the previous
    `remote_publishers` state is gone and no `PeerJoined(Remote(...))` event is
    emitted.
  - this maps directly to ISSUE-188: pubsub drops early remote publisher joins
    before local channel creation.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-188 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 51: pubsub existing remote member duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Leibniz the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test new_local_pubsub_handles_must_observe_existing_remote_members -- --nocapture`
    failed at `src/service/pubsub_service.rs:958:9`.
- Duplicate or too-close symptoms rejected:
  - the test seeds `remote_publishers` and `remote_subscribers`, then creates
    new local publisher/subscriber handles.
  - the new publisher observes only `[PeerJoined(Local)]` and misses the
    already-known remote subscriber.
  - this maps directly to ISSUE-142: `InternalMsg::PublisherCreated` and
    `InternalMsg::SubscriberCreated` replay only local peer presence to the new
    handle and never replay `state.remote_subscribers` or
    `state.remote_publishers`.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-142 evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 50: steady-valid fuzz pass

- Result: no accepted issue and no failing assertion.
- Reviewer: `Epicurus the 4th`, forked subagent review, confirmed
  pass/no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=50 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed; 0 failed`.
- Duplicate or too-close symptoms rejected:
  - no panic, failing assertion, or error log was observed under this
    steady-valid seed.
  - the run covered live-node randomized connect, unicast, broadcast,
    `open_stream`, and raw valid-service traffic.
  - reviewer confirmed this does not exercise churn/shutdown families such as
    ISSUE-139 and ISSUE-170, invalid-service/wire-input handling such as
    ISSUE-053, or stale-route-after-disconnect behavior such as ISSUE-063.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  pass/no-new evidence only. It does not prove absence of latency bugs,
  stream-open correctness, graceful shutdown behavior, invalid input safety,
  route convergence under churn, or high-load backpressure failure modes.

### Cycle after ISSUE-204 no-new cycle 49: sanitized churn duplicate outbound connect panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Carver the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=49 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed at `src/tests/fuzz.rs:372:5`.
- Duplicate or too-close symptoms rejected:
  - the primary background panic was `src/peer.rs:133:113` with
    `should send to main: SendError { .. }`.
  - this maps directly to ISSUE-139: `PeerConnection::new_connecting` still
    reports early outbound `connecting.await` failure through
    `main_tx.send(...).await.expect("should send to main")`, so sanitized
    churn can close the main loop before the spawned connection task reports
    failure.
  - the same run produced 8,610
    `forward peer stopped over peer alias got error no available capacity` logs
    and 548 `... channel closed` logs; reviewer mapped that secondary noise to
    ISSUE-170's missing dedupe/TTL/tombstone suppression for stop forwarding in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this sanitized-churn fuzz run
  strengthens existing ISSUE-139 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 48: valid-action fuzz duplicate stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Pauli the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/router.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=48 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed at `src/tests/fuzz.rs:183:5`.
- Duplicate or too-close symptoms rejected:
  - the primary background panic was `src/router.rs:76:66` with
    `should have direct metric with apply_sync`.
  - this maps directly to ISSUE-063: stale `PeerData::Sync` can reach
    `P2pNetwork::process_internal` after the direct route for that connection
    has been removed; `RouterTable::apply_sync` still expects the direct
    metric to exist and panics.
  - the same run produced 8,753
    `forward peer stopped over peer alias got error no available capacity` logs
    and 161 `... channel closed` logs; reviewer mapped that secondary noise to
    ISSUE-170's missing dedupe/TTL/tombstone suppression for stop forwarding in
    cyclic meshes.
- Root-cause summary impact: no new root cause; this valid-action fuzz run
  strengthens existing ISSUE-063 and ISSUE-170 evidence without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 47: steady-valid fuzz pass

- Result: no accepted issue and no failing assertion.
- Reviewer: `Volta the 4th`, forked subagent review, confirmed
  pass/no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer/peer_internal.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=47 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed; 0 failed`.
- Duplicate or too-close symptoms rejected:
  - no panic or failing assertion was observed under this steady-valid seed.
  - the non-fatal log
    `[PeerConnectionInternal] answer open_bi got error internal channel error`
    maps to a timed-out/dropped requester receiver while the spawned
    `open_bi` task tries to send its result at `src/peer/peer_internal.rs:167`.
  - this is adjacent to existing stream setup/backpressure issues such as
    ISSUE-056, ISSUE-149, and ISSUE-169, but this run has no failing evidence
    for a new accepted issue.
- Root-cause summary impact: no new root cause; this steady-valid fuzz run is
  pass/no-new evidence only. It does not prove stream-open correctness, latency
  bounds, clean cancellation, absence of noisy logs, or invalid-wire/churn
  coverage.

### Cycle after ISSUE-204 no-new cycle 46: invalid-service fuzz duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Cicero the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/ctx.rs`
  - `RUST_LOG=error P2P_FUZZ_SEED=6 P2P_FUZZ_NODES=7 P2P_FUZZ_STEPS=900 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed at `src/tests/fuzz.rs:183:5`.
- Duplicate or too-close symptoms rejected:
  - the fuzz harness injected an invalid wire service id with
    `PeerMessage::Broadcast(..., P2pServiceId::from(256), ...)`.
  - inbound handling reached `SharedCtxInternal::get_service`, which indexes
    the fixed 256-slot service array with `service_id as usize`.
  - the background task panicked at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256`.
  - this maps directly to ISSUE-053: inbound out-of-range service ids still
    kill peer connection tasks through unchecked service-table indexing.
- Root-cause summary impact: no new root cause; this fuzz cycle strengthens
  existing ISSUE-053 evidence under RC-7 without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 45: sanitized churn duplicate peer-connect panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Pasteur the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/fuzz.rs`
  - `src/peer.rs`
  - `P2P_FUZZ_SEED=1 P2P_FUZZ_NODES=6 P2P_FUZZ_STEPS=700 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed at `src/tests/fuzz.rs:372:5`.
- Duplicate or too-close symptoms rejected:
  - sanitized random churn closed or dropped owning network loops while spawned
    connection tasks were still reporting QUIC setup failures.
  - background tasks panicked at `src/peer.rs:92:104` and
    `src/peer.rs:133:113` with `should send to main: SendError { .. }`.
  - this maps directly to ISSUE-139: early incoming/outgoing
    `PeerConnectError` reporting still sends to `main_tx` with
    `.expect("should send to main")` after the main event receiver may already
    be closed.
- Root-cause summary impact: no new root cause; this fuzz cycle strengthens
  existing ISSUE-139 churn evidence under RC-6 without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 44: inbound ConnectRes write-stall duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Curie the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/stream.rs`
  - `cargo test inbound_peer_setup_must_timeout_when_connect_response_write_stalls -- --nocapture`
    failed at `src/peer.rs:855:9`.
- Duplicate or too-close symptoms rejected:
  - a raw client with a tiny receive window sends a valid `ConnectReq` and then
    does not read the large `ConnectRes`; no `PeerConnectError(_, None, _)`
    arrives within the setup window.
  - this maps directly to ISSUE-173: inbound `run_connection` writes
    `ConnectRes` through `write_object`, whose `write_all` calls have no setup
    timeout, so stalled receive-side flow control can hang setup and prevent
    error reporting.
- Root-cause summary impact: no new root cause; this focused inbound setup
  timeout cycle strengthens existing ISSUE-173 evidence under RC-4 without
  adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 43: outbound ConnectReq write-stall duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Kant the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/peer.rs`
  - `src/stream.rs`
  - `cargo test outbound_peer_setup_must_timeout_when_connect_request_write_stalls -- --nocapture`
    failed at `src/peer.rs:821:9` with one pending neighbour left.
- Duplicate or too-close symptoms rejected:
  - a raw peer accepts the P2P control stream with a tiny receive window and
    never reads it; the normal node's outbound setup remains stuck and pending
    neighbour cleanup does not run within the setup window.
  - this maps directly to ISSUE-172: outbound `run_connection` writes
    `ConnectReq` with `write_object`, whose `write_all` calls have no setup
    timeout, so no `MainEvent::PeerConnectError` is emitted while the write is
    stalled behind peer flow control.
- Root-cause summary impact: no new root cause; this focused outbound setup
  timeout cycle strengthens existing ISSUE-172 evidence under RC-4 without
  adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 42: outbound control-stream setup duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Pascal the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/security.rs`
  - `src/peer.rs`
  - `cargo test outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open -- --nocapture`
    failed at `src/tests/security.rs:544:5` with one pending neighbour left.
- Duplicate or too-close symptoms rejected:
  - a raw QUIC server accepts the transport connection while advertising zero
    bidirectional streams; outbound setup never opens the P2P control stream
    and the pending neighbour remains after the cleanup window.
  - this maps directly to ISSUE-159: outbound setup awaits
    `connection.open_bi().await` without a setup timeout, so no
    `MainEvent::PeerConnectError` is emitted for pending-neighbour cleanup.
- Root-cause summary impact: no new root cause; this focused outbound setup
  timeout cycle strengthens existing ISSUE-159 evidence under RC-4 without
  adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 41: unauthenticated inbound connection duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Newton the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/lib.rs`
  - `src/peer.rs`
  - `cargo test unauthenticated_inbound_connections_must_be_admission_bounded -- --nocapture`
    failed at `src/tests/stream.rs:607:5`.
- Duplicate or too-close symptoms rejected:
  - 17 raw QUIC clients connect to a node and never open the P2P main control
    stream; all are accepted, exceeding the test's pending unauthenticated
    connection threshold of 16.
  - this maps directly to ISSUE-134: `process_incoming` accepts/inserts inbound
    connections before authentication, and `PeerConnection::new_incoming`
    awaits the P2P control stream without a node-level unauthenticated cap or
    control-stream timeout.
- Root-cause summary impact: no new root cause; this focused transport
  admission cycle strengthens existing ISSUE-134 evidence under RC-4/RC-5
  unauthenticated setup admission without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 40: unused unidirectional stream duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Hegel the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/quic.rs`
  - `cargo test unused_unidirectional_streams_must_not_be_admitted -- --nocapture`
    failed at `src/quic.rs:128:9`; the raw client opened 17 unidirectional
    streams while the expected admitted count was zero.
- Duplicate or too-close symptoms rejected:
  - both client and server transport configs still allow
    `max_concurrent_uni_streams(10_000_u32.into())`.
  - this maps directly to ISSUE-182 because production P2P code has no
    `accept_uni` path, so the transport admits unused unidirectional streams
    that no application task drains or rejects.
- Root-cause summary impact: no new root cause; this focused transport
  admission cycle strengthens existing ISSUE-182 evidence under RC-3 resource
  admission/backpressure without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 39: idle inbound stream admission duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Avicenna the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
    failed at `src/tests/stream.rs:575:5`.
- Duplicate or too-close symptoms rejected:
  - a raw authenticated peer opens 17 inbound bidirectional streams without
    sending `StreamConnectReq`; all are transport accepted, exceeding the
    test's threshold of 16 idle stream-connect attempts.
  - this maps directly to ISSUE-117: the peer loop accepts every inbound
    bidirectional stream and spawns `accept_bi(...)` without an admission cap or
    stream-connect read timeout.
- Root-cause summary impact: no new root cause; this focused stream admission
  cycle strengthens existing ISSUE-117 evidence under RC-4 without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 38: orphan relay stream duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Archimedes the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test relay_must_not_deliver_downstream_stream_after_upstream_setup_closes -- --nocapture`
    failed at `src/tests/stream.rs:521:5`.
- Duplicate or too-close symptoms rejected:
  - a raw authenticated upstream sends a relayed `StreamConnectReq`, closes the
    upstream response side before setup acknowledgement, and the destination
    still receives the downstream stream event.
  - this maps directly to ISSUE-156: the relay branch opens and delivers the
    downstream stream with `alias.open_stream(...)` before proving the upstream
    setup acknowledgement is writable/live.
- Root-cause summary impact: no new root cause; this focused relay
  cancellation cycle strengthens existing ISSUE-156 evidence under RC-4
  end-to-end setup cancellation without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 37: stalled stream request write duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Bohr the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `src/stream.rs`
  - `cargo test open_stream_must_timeout_when_connect_request_write_stalls -- --nocapture`
    failed at `src/tests/stream.rs:455:5`.
- Duplicate or too-close symptoms rejected:
  - a raw authenticated peer with a tiny stream receive window accepts the
    stream-open bidirectional stream and never reads it; the caller opens with
    large metadata and does not return `Ok(Err(_))` within 2.5 seconds.
  - this maps directly to ISSUE-169: `open_bi` times out only
    `connection.open_bi()`, while `write_object(StreamConnectReq)` and the
    subsequent `StreamConnectRes` wait have no whole-setup deadline.
- Root-cause summary impact: no new root cause; this focused stream setup
  timeout cycle strengthens existing ISSUE-169 evidence under RC-4 without
  adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 36: withheld stream response duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Halley the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test open_stream_must_timeout_when_peer_withholds_connect_response -- --nocapture`
    failed at `src/tests/stream.rs:380:5`.
- Duplicate or too-close symptoms rejected:
  - a raw authenticated peer accepts the stream-open bidirectional stream,
    reads `StreamConnectReq`, withholds `StreamConnectRes`, and the caller's
    `open_stream` does not return `Ok(Err(_))` within 2.5 seconds.
  - this maps directly to ISSUE-149: `open_bi` times out only
    `connection.open_bi()`, writes `StreamConnectReq`, and then awaits
    `wait_object::<_, StreamConnectRes, ...>` without a setup deadline.
- Root-cause summary impact: no new root cause; this focused stream setup
  timeout cycle strengthens existing ISSUE-149 evidence under RC-4 without
  adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 35: unicast ingress-loop fixed

- Result: no accepted non-duplicate issue.
- Reviewer: `Mill the 4th`, forked subagent review, confirmed
  existing-issue fixed/no-new classification.
- Source and test evidence reviewed:
  - `src/peer/peer_internal.rs`
  - `cargo test unicast_relay_must_not_forward_back_to_ingress_peer -- --nocapture`
    passed with `1 passed; 0 failed`.
- Existing issue status:
  - ISSUE-197's focused unicast ingress-loop evidence is fixed: the unicast
    relay path now calls `unicast_route_decision(..., self.conn_id)`, maps
    `RouteAction::Next(next)` where `next == ingress` to
    `UnicastRouteDecision::DropIngressLoop`, and drops/logs instead of
    forwarding the packet back to the sender.
  - this does not prove broader route-loop handling is fixed for stream setup
    or other control-plane paths; ISSUE-180 remains separately evidenced.
- Root-cause summary impact: no new root cause; this cycle records a focused
  fix for ISSUE-197's unicast ingress-loop case without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 34: relay stream ingress-loop duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Socrates the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test relay_stream_must_not_forward_back_to_ingress_peer -- --nocapture`
    failed at `src/tests/stream.rs:161:10` with `Elapsed(())`.
- Duplicate or too-close symptoms rejected:
  - with route state where a ghost peer routes through the two connected nodes,
    `service1.open_stream(PeerId(99), ...)` still fails to return a prompt
    route-loop error and instead times out while relayed setup recurses.
  - this maps directly to ISSUE-180: `accept_bi` still handles
    `RouteAction::Next(next)` by blindly calling `alias.open_stream(...)`
    without knowing or rejecting the ingress connection.
- Root-cause summary impact: no new root cause; this focused relay
  pipe-reliability cycle strengthens existing ISSUE-180 evidence under RC-7
  route-loop handling without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 33: local open-stream panic duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Helmholtz the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/ctx.rs`
  - `cargo test open_stream_to_local_returns_error_not_panic -- --nocapture`
    failed with a panic at `src/ctx.rs:235:17` and final assertion at
    `src/tests/stream.rs:110:5`.
- Duplicate or too-close symptoms rejected:
  - `SharedCtx::open_stream` still maps `RouteAction::Local` to
    `panic!("unsupported open_stream to local node")` instead of returning a
    recoverable `Err`.
  - this maps directly to ISSUE-013; the current panic line is
    `src/ctx.rs:235` after source-line drift from the original ledger entry.
- Root-cause summary impact: no new root cause; this focused stream API
  stability cycle strengthens existing ISSUE-013 evidence under API stability
  and local-route validation without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 32: queue-full stream duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Wegener the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test open_stream_does_not_succeed_when_destination_service_queue_is_full -- --nocapture`
    failed at `src/tests/stream.rs:97:5`.
- Duplicate or too-close symptoms rejected:
  - after ten held accepted streams fill the destination service acceptor
    queue, the next `open_stream` still reports success for a pipe that no
    destination service can consume.
  - this maps directly to ISSUE-012, where the local `open_stream` path ignores
    bounded destination service acceptor `try_send` failure and still returns a
    successful `StreamConnectRes`.
- Root-cause summary impact: no new root cause; this focused high-load
  pipe-reliability cycle strengthens existing ISSUE-012 evidence under RC-3
  delivery backpressure without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 31: closed-receiver stream duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Rawls the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test open_stream_fails_when_destination_service_receiver_is_closed -- --nocapture`
    failed at `src/tests/stream.rs:68:5`.
- Duplicate or too-close symptoms rejected:
  - `open_stream` still reports success after the destination service receiver
    is closed, leaving the opener with an apparently valid pipe that no
    destination service can accept.
  - this maps directly to ISSUE-011, where local stream delivery ignores the
    `service_acceptor.try_send(...)` result and still sends successful
    `StreamConnectRes` to the opener.
- Root-cause summary impact: no new root cause; this focused pipe-reliability
  cycle strengthens existing ISSUE-011 evidence under RC-3/RC-7 delivery and
  routing reliability patterns without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 30: active-path jitter duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Galileo the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/router.rs`
  - `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`
    failed at `src/router.rs:445:9`; the active route switched from
    `ConnectionId(1)` to `ConnectionId(2)` on a tiny RTT improvement.
  - `cargo test should_keep_existing_best_path_on_equal_score -- --nocapture`
    failed at `src/router.rs:420:9`; an equal-cost update switched the active
    route from `ConnectionId(2)` to `ConnectionId(1)`.
- Duplicate or too-close symptoms rejected:
  - both failures map directly to ISSUE-003, where `PeerMemory::select_best`
    immediately reselects the lowest score with no stickiness or hysteresis for
    the current active path.
- Root-cause summary impact: no new root cause; this targeted route-stability
  cycle strengthens existing ISSUE-003 evidence under RC-7 without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 29: sanitized churn duplicate outbound peer-connect panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Galileo the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `RUST_LOG=error P2P_FUZZ_SEED=2182001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed at `src/tests/fuzz.rs:372:5` after background connection tasks
    panicked.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }` map directly to ISSUE-139's
    unchecked outbound `PeerConnectError` reporting after the main loop is
    closed.
  - repeated `forward peer stopped over peer alias got error no available
    capacity` and `try send message ... no available capacity` logs are churn
    pressure/noise overlapping existing ISSUE-170 and RC-3/RC-6 lifecycle
    backpressure patterns, but this run's accepted failing assertion is the
    already-recorded ISSUE-139 panic.
- Root-cause summary impact: no new root cause; this fuzz cycle strengthens
  existing ISSUE-139 sanitized-churn evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 28: steady-valid fuzz pass

- Result: no accepted issue; fuzz command passed.
- Reviewer: `Galileo the 4th`, forked subagent review, classified this cycle
  as `NO_NEW_PASS`; no failing test evidence exists in this cycle, so there is
  no accepted issue to add.
- Fuzz evidence reviewed:
  - `RUST_LOG=error P2P_FUZZ_SEED=2181001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed; 0 failed; 289 filtered out; finished in 16.84s`.
  - `src/tests/fuzz.rs:378-456` runs steady-valid random actions across live
    nodes: connects, unicast, try_unicast, broadcast, open_stream, and raw
    valid unicast, then asserts no background connection/service task panic.
- Root-cause summary impact: no new root cause and no existing failing issue
  strengthened; this is continued pass coverage for steady valid-node action
  fuzzing without failing evidence for ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 27: sanitized churn duplicate peer-connect panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Ptolemy the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `RUST_LOG=error P2P_FUZZ_SEED=2180001 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-139. The churn harness clamps the
    requested node count to 8, so the assertion reported `nodes=8`.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }` map directly to ISSUE-139's
    unchecked `PeerConnectError` reporting after the main loop is closed.
  - connection-loss, `queue main loop full`, and `forward peer stopped ... no
    available capacity` logs are churn pressure/noise overlapping existing
    ISSUE-170 and RC-3/RC-6 shutdown-lifecycle patterns, but this run's
    accepted failing assertion is the already-recorded ISSUE-139 panic.
- Root-cause summary impact: no new root cause; this fuzz cycle strengthens
  existing ISSUE-139 sanitized-churn evidence without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 26: discovery fresh-restart tombstone duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Confucius the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/discovery.rs`
  - `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`
    failed with duplicate evidence for ISSUE-093.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/discovery.rs:328:9` leaves `remotes()` empty after a
    stopped peer restarts with a newer advertisement timestamp and address.
  - `PeerDiscovery::apply_sync` still rejects all advertisements while a stop
    tombstone is fresh without comparing `last_updated` against the tombstone
    timestamp; this is exact accepted source behavior/failing evidence for
    ISSUE-093.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-093 evidence under RC-4 timeout/freshness gaps
  without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 25: discovery tombstone resource duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Hubble the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/discovery.rs`
  - `cargo test graceful_stop_tombstones_must_be_bounded_for_unknown_peers -- --nocapture`
    failed with duplicate evidence for ISSUE-122.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/discovery.rs:280:9` leaves 1,025 stopped-peer
    tombstones for unknown non-seed peer ids, exceeding the bounded-tombstone
    assertion.
  - `PeerDiscovery::remove_remote` still inserts into `stopped` for every
    non-seed peer id, even when that peer was never present in `remotes`; this
    is the exact accepted source behavior and failing evidence for ISSUE-122.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-122 evidence under RC-5 application-level
  resource limits without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 24: pubsub short RPC timeout duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Dirac the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/pubsub.rs`
  - `src/service/pubsub_service.rs`
  - `cargo test pubsub_publish_rpc_must_respect_short_timeout -- --nocapture`
    failed with duplicate evidence for ISSUE-121.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/tests/pubsub.rs:618:10` shows a caller-supplied 20 ms
    `publish_rpc` timeout still waiting past the 200 ms outer test timeout.
  - `RPC_TICK_INTERVAL_MS` remains fixed at 1,000 ms, and `on_rpc_tick` remains
    the only path that expires `publish_rpc_reqs` and `feedback_rpc_reqs`
    against caller-supplied timeouts; this is the exact accepted source
    behavior and failing evidence for ISSUE-121.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-121 evidence under RC-4 timeout granularity gaps
  without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 23: broadcast queue-full fixed

- Result: no accepted non-duplicate issue.
- Reviewer: `Hilbert the 4th`, forked subagent review, confirmed
  existing-issue fixed/no-new classification.
- Source and test evidence reviewed:
  - `src/tests/cross_nodes.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test inbound_broadcast_must_not_drop_when_service_queue_is_full -- --nocapture`
    passed.
- Duplicate, fixed, or too-close symptoms rejected:
  - the queue-full broadcast subcase no longer reproduces ISSUE-120 because
    `send_local_service_event` now wraps bounded `service.send(event)` in
    `LOCAL_SERVICE_DELIVERY_TIMEOUT`, and the 11-broadcast assertion drains all
    expected events.
  - this is fixed evidence for ISSUE-120's local queue-full silent-drop case
    only; it is not a broader RC-3 fix and does not add ISSUE-205.
- Root-cause summary impact: no new root cause; this cycle records fixed
  evidence for ISSUE-120 under RC-3 while other RC-3 backpressure issues remain
  open.

### Cycle after ISSUE-204 no-new cycle 22: unicast queue-full partial fix, closed receiver duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Noether the 4th`, forked subagent review, confirmed
  existing-issue partial-fix/no-new classification.
- Source and test evidence reviewed:
  - `src/tests/cross_nodes.rs`
  - `src/peer/peer_internal.rs`
  - `cargo test inbound_unicast_must_not_drop_when_service_queue_is_full -- --nocapture`
    passed.
  - `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
    failed with duplicate evidence for ISSUE-119.
- Duplicate, fixed, or too-close symptoms rejected:
  - the queue-full subcase no longer reproduces ISSUE-119 because
    `send_local_service_event` now wraps bounded `service.send(event)` in
    `LOCAL_SERVICE_DELIVERY_TIMEOUT`, and the 11-message assertion drains all
    expected messages.
  - the closed-receiver subcase still fails at `src/tests/cross_nodes.rs:203:5`;
    `send_unicast` reports success even when the destination service receiver
    is gone. This is already accepted under ISSUE-119 and does not add
    ISSUE-205.
- Root-cause summary impact: no new root cause; this cycle records partial
  fixed evidence for ISSUE-119 under RC-3 while the closed-receiver reporting
  subcase remains open.

### Cycle after ISSUE-204 no-new cycle 21: visualization info batch duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Popper the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/visualization_service.rs`
  - `cargo test visualization_info_batches_must_be_bounded -- --nocapture`
    failed with duplicate evidence for ISSUE-105.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/service/visualization_service.rs:248:9` delivered
    1,025 topology rows from one `Info` frame, exceeding the test cap of 1,024
    rows.
  - `VisualizationService::recv` still forwards `Message::Info(neighbours)`
    directly as `VisualizationServiceEvent::PeerJoined` or `PeerUpdated`, which
    is the exact accepted source behavior and failing evidence for ISSUE-105.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-105 evidence under RC-5 application-level
  resource limits without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 20: alias internal-control backlog duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Euclid the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/alias_service.rs`
  - `cargo test alias_internal_control_backlog_must_be_bounded -- --nocapture`
    failed with duplicate evidence for ISSUE-127.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/service/alias_service.rs:484:9` observed 1,025
    pending alias internal control messages, exceeding the bounded-backlog
    assertion.
  - `AliasService::new` still uses an unbounded `tx/rx` channel, and
    `AliasServiceRequester::register` can enqueue registration messages without
    admission control or backpressure; this is the exact accepted source
    behavior and failing evidence for ISSUE-127.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-127 evidence under RC-3 backpressure policy gaps
  without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 19: pubsub internal-control backlog duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Nietzsche the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/pubsub_service.rs`
  - `cargo test pubsub_internal_control_backlog_must_be_bounded -- --nocapture`
    failed with duplicate evidence for ISSUE-126.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/service/pubsub_service.rs:754:9` observed 1,025
    pending pubsub internal control messages, exceeding the bounded-backlog
    assertion.
  - `PubsubService::new` still uses an unbounded `internal_tx/internal_rx`
    channel, and requester handle creation can enqueue registration messages
    without admission control; this is the exact accepted source behavior and
    failing evidence for ISSUE-126.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-126 evidence under RC-3 backpressure policy gaps
  without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 18: metrics info batch duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Bernoulli the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/metrics_service.rs`
  - `cargo test metrics_info_batches_must_be_bounded -- --nocapture`
    failed with duplicate evidence for ISSUE-104.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/service/metrics_service.rs:67:9` delivered 1,025
    metrics rows from one `Info` frame, exceeding the test cap of 1,024 rows.
  - `MetricsService::recv` still forwards `Message::Info(peer_metrics)`
    directly as `MetricsServiceEvent::OnPeerConnectionMetric(from,
    peer_metrics)`, which is the exact accepted source behavior and failing
    evidence for ISSUE-104.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-104 evidence under RC-5 application-level
  resource limits without adding ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 17: discovery graceful-stop duplicate noise

- Result: no accepted non-duplicate issue.
- Reviewer: `Gibbs the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/discovery.rs`
  - `src/discovery.rs`
  - `cargo test graceful_shutdown_removes_stopped_non_seed -- --nocapture`
    passed.
- Duplicate or too-close symptoms rejected:
  - the test confirmed node3 eventually removed the gracefully stopped non-seed
    peer from its neighbour set, so this run did not produce failing evidence
    for ISSUE-205.
  - repeated `P2pNetwork connecting to 2@...` logs before final route removal
    map to existing discovery/lifecycle issues: ISSUE-153 duplicate discovery
    connect attempts, ISSUE-051/ISSUE-167 stopped or expired non-seed cleanup,
    ISSUE-118 graceful-shutdown congestion, ISSUE-170 stop propagation
    amplification, and ISSUE-185/ISSUE-187 lifecycle event gaps.
- Root-cause summary impact: no new root cause; this source/test cycle maps to
  RC-6 lifecycle cleanup and RC-7 unstable routing/discovery without adding
  ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 16: stream idle admission duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Beauvoir the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/tests/stream.rs`
  - `src/stream.rs`
  - `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
    failed with duplicate evidence for ISSUE-117.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/tests/stream.rs:575:5` shows a raw authenticated QUIC
    peer opening 17 bidirectional streams without sending `StreamConnectReq`,
    exceeding the test's admission threshold of 16 idle stream-connect
    attempts.
  - the same test, assertion, and root cause are already the accepted evidence
    for ISSUE-117 and RC-4's incomplete stream setup admission/timeout pattern.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-117 evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 15: handshake third-party identity duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Zeno the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/secure.rs`
  - `src/peer.rs`
  - `src/tests/security.rs`
  - `src/tests/stream.rs`
  - `cargo test inbound_handshake_must_reject_peer_claiming_third_party_id -- --nocapture`
    failed with duplicate evidence for ISSUE-194.
- Duplicate or too-close symptoms rejected:
  - the failure at `src/peer.rs:683:9` shows inbound handshake admission
    accepting a shared-key holder's caller-supplied third-party `PeerId` claim.
  - the same test, assertion, and root cause are already the accepted evidence
    for ISSUE-194 and RC-1's non-authoritative authenticated identity pattern.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-194 evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 14: replicated-KV stale terminal snapshot duplicate

- Result: no accepted non-duplicate issue.
- Reviewer: `Sagan the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Source and test evidence reviewed:
  - `src/service/replicate_kv_service/remote_storage.rs`
  - `src/service/replicate_kv_service/local_storage.rs`
  - `src/service/replicate_kv_service/messages.rs`
  - `src/service/replicate_kv_service.rs`
  - `cargo test full_sync_must_reject_stale_terminal_snapshot_after_continuation_request -- --nocapture`
    failed with duplicate evidence for ISSUE-143.
- Duplicate or too-close symptoms rejected:
  - the failure at
    `src/service/replicate_kv_service/remote_storage.rs:919:9` shows
    `SyncFullState` accepting a stale terminal snapshot and setting
    `ctx.next_state` to `Working(Version(3))` while a continuation range is
    outstanding.
  - the same test, assertion, state transition, and root cause are already the
    accepted evidence for ISSUE-143.
- Root-cause summary impact: no new root cause; this source/test cycle
  strengthens existing ISSUE-143 evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 13: invalid churn duplicate service-id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Nash the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=2179001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1400 cargo test fuzz_random_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-053.
- Duplicate or too-close symptoms rejected:
  - the background panic at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256` maps directly to
    ISSUE-053, where inbound out-of-range `P2pServiceId(256)` indexes the
    fixed 256-service array.
  - `try send message to peer 6 ... error channel closed` logs after the
    panic were reviewed as consequential churn/lifecycle symptoms rather than
    a distinct issue.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-053 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 12: sanitized churn duplicate shutdown and PeerStopped storm

- Result: no accepted non-duplicate issue.
- Reviewer: `Plato the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=2178001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-139, ISSUE-170, RC-3, and
    shutdown-congestion overlap covered by ISSUE-118.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/peer.rs:92:104` and `src/peer.rs:133:113` with
    `should send to main: SendError { .. }` map directly to ISSUE-139's
    unchecked `PeerConnectError` reporting after the main loop is closed.
  - 9,813 `forward peer stopped over peer alias got error ...` logs, including
    8,556 `no available capacity` and 1,269 `channel closed` failures, map to
    ISSUE-170's duplicate `PeerStopped` forwarding amplification in cyclic
    meshes.
  - 529 `queue main loop full` warnings map to RC-3 peer-control backpressure
    and ISSUE-118-style congested graceful-shutdown overlap.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-139 and ISSUE-170 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 11: valid-action fuzz duplicate stale sync and shutdown panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Godel the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=2177001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2500 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-063 and ISSUE-139.
- Duplicate or too-close symptoms rejected:
  - the primary background panic at `src/router.rs:76:66` with
    `should have direct metric with apply_sync` followed 20
    `reported peer 4 stopped` logs and maps directly to ISSUE-063's stale
    peer data / stopped-peer route-missing crash.
  - the later background panic at `src/peer.rs:92:104` with
    `should send to main: SendError { .. }` maps directly to ISSUE-139's
    unchecked `PeerConnectError` reporting after the main loop is closed. The
    source line differs from earlier fuzz notes because this run hit the
    incoming-connection error branch.
  - 14 `queue main loop full` warnings map to RC-3 peer-control backpressure
    rather than a new issue.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-063 and ISSUE-139 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 10: eight-node steady-valid fuzz pass

- Result: no accepted non-duplicate issue.
- Reviewer: `Fermat the 4th`, forked subagent review, confirmed no-new
  classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=2176001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with `1 passed; 0 failed`.
- Duplicate or too-close symptoms rejected:
  - 335 route reselection logs map to ISSUE-003 and RC-7 route instability.
  - 20 `queue main loop full` warnings map to RC-3 peer-control backpressure,
    including ISSUE-118, ISSUE-164, ISSUE-198, ISSUE-199, ISSUE-203, and
    ISSUE-204 depending on the affected send path.
  - 2 transient `path to ... not found` warnings map to existing stale or
    unavailable-route entries rather than a new issue.
- Root-cause summary impact: no new root cause; the run produced no panic,
  failing assertion, or stronger correctness/security evidence for ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 9: invalid-wire action fuzz duplicate service-id panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Hooke the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x205301 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1000 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-053. The harness reported
    `seed=24301` because the current fuzz env parser falls back to its default
    when given a hex seed literal.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/ctx.rs:34:9` with
    `index out of bounds: the len is 256 but the index is 256` map directly to
    ISSUE-053, where an inbound out-of-range `P2pServiceId(256)` indexes the
    fixed 256-service array.
  - route churn, temporary path lookup failures, and channel-close logs map to
    existing route/lifecycle entries rather than a new issue.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-053 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 8: sanitized churn duplicate connect-error panic

- Result: no accepted non-duplicate issue.
- Reviewer: `Faraday the 4th`, forked subagent review, confirmed
  duplicate-only no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x205201 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1200 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-139.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/peer.rs:133:113` with
    `should send to main: SendError { .. }` map directly to ISSUE-139, where
    early `PeerConnectError` reporting uses `expect("should send to main")`
    after the main loop may already be shut down.
  - shutdown, refused connection, and endpoint-close churn are the bad-network
    lifecycle conditions ISSUE-139 already covers.
  - temporary `path to X not found`, connection-lost, and endpoint shutdown
    logs map to existing route/lifecycle entries rather than a new issue.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-139 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 7: valid-action fuzz duplicate stale sync

- Result: no accepted non-duplicate issue.
- Reviewer: `Planck the 4th`, forked subagent review, confirmed duplicate-only
  no-new classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x205101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1200 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    failed with duplicate evidence for ISSUE-063.
- Duplicate or too-close symptoms rejected:
  - background panics at `src/router.rs:76:66` with
    `should have direct metric with apply_sync` map directly to ISSUE-063,
    where stale `PeerData::Sync` reaches `RouterTable::apply_sync` after the
    direct connection route has already been removed.
  - repeated `queue main loop full` warnings map to RC-3 backpressure issues.
  - route reselection/path noise maps to ISSUE-003 and RC-7 route instability.
  - repeated PeerStopped forwarding pressure maps to existing graceful-stop and
    lifecycle entries, including ISSUE-170 where applicable.
- Root-cause summary impact: no new root cause; this run strengthens existing
  ISSUE-063 fuzz evidence but does not add ISSUE-205.

### Cycle after ISSUE-204 no-new cycle 6: fifteen-node post-stop-condition fuzz

- Result: no accepted non-duplicate issue.
- Reviewer: `Peirce the 4th`, forked subagent review, confirmed no-new
  classification after the prior 5/5 stop-condition threshold.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x205001 P2P_FUZZ_NODES=15 P2P_FUZZ_STEPS=5000 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
- Duplicate or too-close symptoms rejected:
  - route reselection/path-jumping noise maps to ISSUE-003 and RC-7 route
    instability.
  - `queue main loop full` warnings under high load map to RC-3, including
    ISSUE-118, ISSUE-164, ISSUE-198, ISSUE-199, ISSUE-203, and ISSUE-204.
  - temporary `path to X not found` maps to existing stale/unavailable route
    and stopped-peer availability issues.
  - stream `open_bi` and local processing logs completed successfully and
    produced no failing evidence.
- Root-cause summary impact: no new root cause; fuzz output produced no panic
  or failing assertion for a fresh accepted issue.

### Cycle after ISSUE-204 no-new cycle 5: fourteen-node steady-valid fuzz

- Result: no accepted non-duplicate issue.
- Reviewer: `Harvey the 4th`, forked subagent review, confirmed no-new
  classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x204207 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=4200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
- Duplicate or too-close symptoms rejected:
  - route reselection/path-jumping noise maps to ISSUE-003 and RC-7 route
    instability.
  - `queue main loop full` warnings under load map to RC-3, including
    ISSUE-118, ISSUE-164, ISSUE-198, ISSUE-199, ISSUE-203, and ISSUE-204.
  - temporary `path to X not found` maps to existing stale/unavailable route
    and stopped-peer availability issues.
  - stream `open_bi` and local processing logs completed successfully and
    produced no failing evidence.
- Root-cause summary impact: no new root cause; fuzz output produced no panic
  or failing assertion for a fresh accepted issue.

### Cycle after ISSUE-204 no-new cycle 4: thirteen-node steady-valid fuzz

- Result: no accepted non-duplicate issue.
- Reviewer: `Jason the 4th`, forked subagent review, confirmed no-new
  classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x204206 P2P_FUZZ_NODES=13 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
- Duplicate or too-close symptoms rejected:
  - active route reselection/path-jumping noise maps to ISSUE-003 and RC-7
    route instability.
  - `queue main loop full` warnings under high load map to RC-3, including
    ISSUE-118, ISSUE-164, ISSUE-198, ISSUE-199, ISSUE-203, and ISSUE-204.
  - temporary `path to X not found` and stale unavailable-route symptoms map
    to existing stale-route/stopped-peer availability root causes.
  - successful stream `open_bi` and local processing logs produced no failing
    evidence.
- Root-cause summary impact: no new root cause; fuzz output produced no panic
  or failing assertion for a fresh accepted issue.

### Cycle after ISSUE-204 no-new cycle 3: twelve-node steady-valid fuzz

- Result: no accepted non-duplicate issue.
- Reviewer: `Locke the 4th`, forked subagent review, confirmed no-new
  classification.
- Fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x204205 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
- Duplicate or too-close symptoms rejected:
  - active route reselection noise maps to ISSUE-003 and RC-7 route
    instability.
  - `queue main loop full` and bounded backpressure warnings map to RC-3,
    including ISSUE-118, ISSUE-164, ISSUE-198, ISSUE-199, ISSUE-203, and
    ISSUE-204 depending on the path.
  - endpoint-driver-drop and connection-lost teardown logs at test end map to
    existing lifecycle and graceful-shutdown entries in RC-6, especially
    ISSUE-187 and related PeerStopped/teardown cases.
- Root-cause summary impact: no new root cause; fuzz output produced no panic
  or failing assertion for a fresh accepted issue.

### Cycle after ISSUE-204 no-new cycle 2: security, transport, stream codec, config

- Result: no accepted non-duplicate issue.
- Reviewer: `Turing the 4th`, forked subagent review, confirmed no-new
  classification.
- Local/source areas reviewed:
  - `src/secure.rs`, `src/quic.rs`, `src/stream.rs`, `src/lib.rs`
  - public network and service constructors, plus existing stream/security
    tests and ledger entries.
- Duplicate or too-close candidates rejected:
  - shared-key future timestamp, overflow, replay, self-id, and third-party-id
    handshake variants map to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
    ISSUE-189, and ISSUE-194.
  - unused QUIC unidirectional streams, stream admission/setup stalls, relay
    stream loops, and upstream/downstream cancellation gaps map to ISSUE-117,
    ISSUE-134, ISSUE-149, ISSUE-156, ISSUE-159, ISSUE-169, ISSUE-172,
    ISSUE-173, ISSUE-180, and ISSUE-182.
  - codec frame limits and QUIC object writer serialization/size bugs map to
    ISSUE-010, ISSUE-024, ISSUE-097, ISSUE-098, and ISSUE-174.
  - zero network tick and metrics/visualization collection interval panics map
    to ISSUE-040 and ISSUE-054.
  - seed, advertise, discovery timestamp, and malformed route/discovery sync
    config/input behavior maps to ISSUE-009, ISSUE-033, ISSUE-044, ISSUE-055,
    ISSUE-092, ISSUE-103, ISSUE-167, ISSUE-181, ISSUE-190, and ISSUE-192.
- Root-cause summary impact: no new root cause; rejected candidates map to
  existing RC-3, RC-4, RC-6, and RC-7 patterns.

### Cycle after ISSUE-204 no-new cycle 1: pubsub, replicated-KV, alias, route/discovery

- Result: no accepted non-duplicate issue.
- Reviewer: `Russell the 4th`, forked subagent review, confirmed no-new
  classification.
- Local/source areas reviewed:
  - `src/service/pubsub_service.rs`,
    `src/service/pubsub_service/publisher.rs`,
    `src/service/pubsub_service/subscriber.rs`
  - `src/service/replicate_kv_service.rs`,
    `src/service/replicate_kv_service/local_storage.rs`,
    `src/service/replicate_kv_service/remote_storage.rs`,
    `src/service/replicate_kv_service/messages.rs`
  - `src/service/alias_service.rs`, `src/lib.rs`, `src/discovery.rs`,
    `src/router.rs`, and existing fuzz harnesses.
- Duplicate or too-close candidates rejected:
  - pubsub directed response/fanout/backpressure paths map to ISSUE-115,
    ISSUE-116, ISSUE-163, ISSUE-178, and related RC-3/RC-4 entries.
  - replicated-KV malformed snapshot, repair, resource-bound, and lifecycle
    paths map to ISSUE-081 through ISSUE-089, ISSUE-110, ISSUE-138,
    ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-171, ISSUE-175, ISSUE-184, and
    ISSUE-186.
  - alias lookup/shutdown/find backlog and cache mutation paths map to
    ISSUE-022, ISSUE-090, ISSUE-109, ISSUE-148, ISSUE-152, ISSUE-158,
    ISSUE-179, and ISSUE-183.
  - route/discovery/stopped-peer/stale-event behavior maps to ISSUE-003,
    ISSUE-055, ISSUE-092, ISSUE-103, ISSUE-160, ISSUE-167, ISSUE-177,
    ISSUE-181, ISSUE-190, ISSUE-192, and ISSUE-197.
- Additional fuzz evidence reviewed:
  - `P2P_FUZZ_SEED=0x204204 P2P_FUZZ_NODES=11 P2P_FUZZ_STEPS=2800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue. Output again showed active route reselection
    noise and endpoint-driver-drop shutdown logs at test end, but no panic or
    failing assertion.
- Root-cause summary impact: no new root cause; rejected candidates map to
  existing RC-2, RC-3, RC-4, RC-5, RC-6, and RC-7 patterns.

### Cycle after ISSUE-193 no-new cycle 4: public API, examples, fuzz harness, config

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Kierkegaard the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/lib.rs`, `src/requester.rs`, `src/service.rs`, `src/ctx.rs`,
    `src/quic.rs`, `src/secure.rs`, `src/utils.rs`, `src/discovery.rs`, and
    `src/router.rs`.
  - `README.md`, `examples/simple.rs`, `examples/kv.rs`,
    `examples/benchmark.rs`, `examples/readme_getting_started.rs`,
    `src/tests.rs`, `src/tests/fuzz.rs`, and cross-node/config/security tests.
- Validation:
  - Forked reviewer ran example checks and reported that `cargo check
    --example simple`, `cargo check --example kv`, and `cargo check --example
    benchmark` pass.
  - `cargo check --examples` still fails only on
    `examples/readme_getting_started.rs`, which is existing ISSUE-191
    evidence.
- Duplicate or too-close candidates rejected:
  - README/getting-started compile drift: ISSUE-191.
  - duplicate, out-of-range, and dropped service registration/lifecycle
    failures: ISSUE-050, ISSUE-052, ISSUE-053, ISSUE-060, and ISSUE-091.
  - stale requester panic after network shutdown/drop: ISSUE-028 and
    ISSUE-125.
  - zero tick interval constructor panic: ISSUE-054.
  - seed and non-seed stop, timeout, stale advertise, duplicate advertise, and
    route/discovery lifecycle behavior: ISSUE-004, ISSUE-055, ISSUE-092,
    ISSUE-093, ISSUE-103, ISSUE-122, ISSUE-153, ISSUE-167, ISSUE-181, and
    ISSUE-192.
  - graceful shutdown latency across congested peers: ISSUE-118.
  - QUIC stream admission/setup stalls and unused stream classes:
    ISSUE-117, ISSUE-134, ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-172,
    ISSUE-173, and ISSUE-182.
  - existing fuzz harness panic surfaces map to known RC-1, RC-3, RC-6, and
    RC-7 issues rather than a fresh root cause.
- Root-cause summary impact: no new root cause; rejected candidates map to
  existing RC-1, RC-3, RC-6, RC-7, and RC-8.
- Threshold impact: this is the fifth consecutive no-new issue cycle after
  ISSUE-193, so the audit switches to randomized node-action fuzzing as
  requested.

### Cycle after ISSUE-193 no-new cycle 3: alias, metrics, visualization, service boundaries

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Bacon the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/service/alias_service.rs`,
    `src/service/metrics_service.rs`,
    `src/service/visualization_service.rs`, and existing alias, metrics, and
    visualization tests.
  - `src/service.rs`, `src/ctx.rs`, `src/msg.rs`, `src/stream.rs`,
    `src/peer/peer_alias.rs`, and existing service boundary, stream codec, and
    peer-alias evidence tests.
- Duplicate or too-close candidates rejected:
  - alias requester/guard panics after close, refcount overflow, find waiter
    growth, unique pending-find growth, internal control backlog, and run-loop
    closed-channel panics: ISSUE-019, ISSUE-029, ISSUE-035, ISSUE-041,
    ISSUE-127, ISSUE-130, and ISSUE-132.
  - alias cache poisoning, stale cache mutation via `Found`, `NotFound`,
    `NotifySet`, and `Shutdown`, plus shutdown still serving aliases or leaving
    pending find waiters alive: ISSUE-022, ISSUE-090, ISSUE-109, ISSUE-148,
    ISSUE-152, ISSUE-158, ISSUE-179, and ISSUE-183.
  - metrics and visualization zero-interval or closed-service panics, forged
    `Info`, arbitrary `Scan`, retained state, oversized batches, and graceful
    stop lag: ISSUE-040, ISSUE-061, ISSUE-062, ISSUE-078, ISSUE-079,
    ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-128, ISSUE-129, and ISSUE-165.
  - repeated collector scans building up behind congested peer queues has since
    been accepted as ISSUE-200 after focused failing evidence showed detached
    periodic scan tasks accumulating rather than direct caller blocking.
  - service boundary stale requesters, dropped services, local delivery queue
    loss, out-of-range service ids, broadcast replay, and open-stream-to-local
    panic: ISSUE-013, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076,
    ISSUE-091, ISSUE-119, ISSUE-120, and ISSUE-181.
  - stream codec serialization failure, length-prefix truncation, actual-size
    mismatch, setup stalls, and peer-alias blocking/backpressure:
    ISSUE-097, ISSUE-098, ISSUE-117, ISSUE-118, ISSUE-134, ISSUE-149,
    ISSUE-156, ISSUE-159, ISSUE-169, ISSUE-172, ISSUE-173, and ISSUE-174.
- Root-cause summary impact: no new root cause; rejected candidates map to
  RC-1, RC-2, RC-3, RC-4, RC-5, RC-6, and RC-7.

### Cycle after ISSUE-193 no-new cycle 2: pubsub lifecycle and replicated-KV storage

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Galileo the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/service/pubsub_service.rs`,
    `src/service/pubsub_service/publisher.rs`,
    `src/service/pubsub_service/subscriber.rs`, and existing pubsub lifecycle,
    membership, RPC, requester, and resource-limit tests.
  - `src/service/replicate_kv_service.rs`,
    `src/service/replicate_kv_service/messages.rs`,
    `src/service/replicate_kv_service/local_storage.rs`,
    `src/service/replicate_kv_service/remote_storage.rs`, and replicated-KV
    storage/message evidence tests.
- Duplicate or too-close candidates rejected:
  - new local pubsub handles missing existing remote members: ISSUE-142.
  - early remote pubsub joins dropped before local channel creation: ISSUE-188.
  - stale pubsub leave overriding newer heartbeat: ISSUE-155.
  - unknown destroy creating phantom pubsub channels and empty channel
    retention: ISSUE-150 and ISSUE-108.
  - unbounded pubsub event queues, internal controls, RPC maps, membership,
    heartbeat, and method sizes: ISSUE-043, ISSUE-100, ISSUE-106, ISSUE-107,
    ISSUE-123, ISSUE-124, and ISSUE-133.
  - pubsub RPC destination accounting for failed local or remote fanout:
    ISSUE-163 and ISSUE-178.
  - stale cloned pubsub requesters and RPC answer authority gaps:
    ISSUE-069, ISSUE-070, ISSUE-074, ISSUE-075, ISSUE-115, and ISSUE-116.
  - forged pubsub source or membership bypass paths:
    ISSUE-014, ISSUE-015, ISSUE-039, and ISSUE-048.
  - replicated-KV snapshot producer/consumer mismatch, terminal omissions,
    malformed snapshot pages, version arithmetic, stale or duplicate
    `FetchChanged`, empty or partial repair responses, premature resync
    deletes, absent-key deletes, and ignored broadcast liveness refresh:
    ISSUE-034, ISSUE-047, ISSUE-081 through ISSUE-089, ISSUE-110, ISSUE-138,
    ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-171, ISSUE-175, ISSUE-184, and
    ISSUE-186.
  - replicated-KV remote store/resource growth and graceful-stop lifecycle:
    ISSUE-045, ISSUE-102, ISSUE-162, and related RC-5/RC-6 entries.
- Root-cause summary impact: no new root cause; rejected candidates map to
  RC-1, RC-2, RC-3, RC-5, and RC-6.

### Cycle after ISSUE-193 no-new cycle 1: public control, routing/discovery, transport admission

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Mencius the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/lib.rs`, especially `P2pNetwork::recv`, `process_tick`,
    `process_internal`, `process_control`, `shutdown`, and
    `shutdown_gracefully`.
  - `src/router.rs`, `src/discovery.rs`, `src/neighbours.rs`,
    `src/requester.rs`, and existing route/discovery/security tests.
  - `src/quic.rs`, `src/secure.rs`, `src/peer.rs`,
    `src/peer/peer_internal.rs`, and existing handshake/stream admission
    evidence tests.
- Duplicate or too-close candidates rejected:
  - zero network tick panic: ISSUE-054.
  - slow or blocking graceful shutdown notify: ISSUE-118.
  - stale requester panic after network drop: ISSUE-028.
  - `connect()` success before authentication or for a different address:
    ISSUE-016 and ISSUE-177.
  - duplicate discovery tick connect backlog: ISSUE-153.
  - hidden graceful `PeerStopped` from public events: ISSUE-187.
  - stale or mismatched internal lifecycle events:
    ISSUE-057, ISSUE-063 through ISSUE-068, ISSUE-135, and ISSUE-145.
  - unauthenticated QUIC admission, idle stream admission, and unused
    unidirectional streams: ISSUE-117, ISSUE-134, and ISSUE-182.
  - shared-key future timestamp, replay, overflow, and self-id handshake
    problems: ISSUE-002, ISSUE-146, ISSUE-176, and ISSUE-189.
  - local stream panic, relay loop, stalled setup, and upstream cancellation
    failures: ISSUE-013, ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-169, and
    ISSUE-180.
  - route/discovery jitter, duplicate rows, direct-route replacement, seed
    handling, stale advertisements, stopped tombstones, and overflow
    candidates were already covered by existing route/discovery ledger entries.
- Root-cause summary impact: no new root cause; rejected candidates map to
  RC-3, RC-4, RC-6, and RC-7.

### Cycle after ISSUE-193: metrics/visualization, fuzz, malformed wire paths, alias state

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Kuhn the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/service/metrics_service.rs`,
    `src/service/visualization_service.rs`, `src/service.rs`,
    `src/msg.rs`, `src/stream.rs`, `src/ctx.rs`
  - `src/peer/peer_internal.rs`, `src/lib.rs`, `src/tests/fuzz.rs`,
    `src/tests/metrics.rs`, `src/tests/visualization.rs`
  - `src/service/alias_service.rs`, `src/tests/alias.rs`, and existing
    alias/metrics/visualization/malformed-wire ledger entries.
- Duplicate or too-close candidates rejected:
  - metrics and visualization forged `Info` frames: ISSUE-061 and ISSUE-062.
  - metrics and visualization arbitrary `Scan` disclosure: ISSUE-078 and
    ISSUE-079; broadcast scan reflection composes with ISSUE-015.
  - metrics and visualization row/resource growth and retained sender state:
    ISSUE-102, ISSUE-104, and ISSUE-105.
  - metrics and visualization closed-channel or zero-interval panics:
    ISSUE-040, ISSUE-128, and ISSUE-129.
  - visualization graceful leave lag: ISSUE-165.
  - out-of-range service ids on unicast/stream paths: ISSUE-053 and ISSUE-091.
  - stream source spoofing, relay loops, idle/stalled setup, and upstream
    cancellation gaps: ISSUE-018, ISSUE-117, ISSUE-149, ISSUE-156,
    ISSUE-169, and ISSUE-180.
  - alias stale requester, cache poisoning/freshness, shutdown, refcount,
    backlog, and pending-find limit candidates: ISSUE-019, ISSUE-022,
    ISSUE-029, ISSUE-036, ISSUE-090, ISSUE-109, ISSUE-127, ISSUE-130,
    ISSUE-132, ISSUE-137, ISSUE-148, ISSUE-152, ISSUE-158, ISSUE-179, and
    ISSUE-183.
  - fuzz harness panic surfaces mapped to existing malformed internal event,
    source-spoofing, out-of-range service id, and PeerStopped issues rather
    than a fresh root cause.
- Root-cause summary impact: no new root cause; rejected candidates map to
  RC-1, RC-2, RC-3, RC-4, RC-5, and RC-6.

### Cycle after ISSUE-165: requester/control, lifecycle, route/discovery

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Mendel the 2nd`.
- Local/source areas reviewed:
  - `src/requester.rs`, `src/lib.rs`, `src/ctx.rs`,
    `src/peer/peer_alias.rs`
  - `src/peer.rs`, `src/peer/peer_internal.rs`, `src/neighbours.rs`
  - `src/router.rs`, `src/discovery.rs`, stream-control tests, metrics tests,
    and existing ledger entries.
- Duplicate or too-close candidates rejected:
  - stale `P2pNetworkRequester` panic after network drop: ISSUE-028.
  - self-connect and local-route failures: ISSUE-112, ISSUE-013,
    ISSUE-005/006/103.
  - peer control queue blocking or dropped maintenance sync:
    ISSUE-049/050/056/118/164.
  - duplicate connection races: ISSUE-113/114.
  - stale internal lifecycle event validation: ISSUE-057/063/064/065/066/067/068/135/136/145.
  - stream setup/admission failures: ISSUE-011/012/117/134/149/156/159.
  - service lifecycle propagation variants without distinct impact from
    ISSUE-162 or ISSUE-165.
- Root-cause summary impact: no change; rejected candidates map to existing
  RC-3, RC-4, RC-6, and RC-7 patterns.

### Cycle after no-new cycle 1: serialization, parsing, public API boundaries

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Hegel the 3rd`.
- Local/source areas reviewed:
  - `src/stream.rs`, `src/msg.rs`, `src/peer.rs`,
    `src/peer/peer_internal.rs`
  - `src/service/pubsub_service.rs`, `src/service/replicate_kv_service.rs`,
    `src/service/replicate_kv_service/local_storage.rs`,
    `src/service/replicate_kv_service/remote_storage.rs`
  - existing pubsub, replicated-KV, metrics, visualization, stream, and
    malformed-input ledger entries.
- Duplicate or too-close candidates rejected:
  - pubsub object helper serialization panics: ISSUE-094.
  - QUIC object writer serialization and length-prefix failures:
    ISSUE-097/098.
  - oversized main/service frames and service-level batch/resource gaps:
    ISSUE-010, ISSUE-024, ISSUE-100 through ISSUE-108, ISSUE-122.
  - alias requester/guard drop panics and stale alias cache states: ISSUE-029
    plus alias stale/cache issues.
  - pubsub RPC answer binding and forgery surfaces:
    ISSUE-020, ISSUE-115, ISSUE-116.
  - replicated-KV malformed snapshot/change/version cases already covered by
    local and remote storage evidence tests.
- Root-cause summary impact: no change; rejected candidates map to existing
  RC-2, RC-3, RC-5, and RC-6 patterns.

### Cycle after ISSUE-173: pubsub handle collisions and closed-service delivery

- Result: no accepted non-duplicate issue; added supplemental failing evidence
  to existing issues.
- Reviewer/explorer: `Kepler the 3rd`, `Noether the 2nd`, `Turing the 2nd`.
- Local/source areas reviewed:
  - `src/service/pubsub_service.rs`,
    `src/service/pubsub_service/subscriber.rs`,
    `src/service/pubsub_service/publisher.rs`
  - `src/service.rs`, `src/ctx.rs`, `src/peer/peer_internal.rs`,
    `src/tests/cross_nodes.rs`
  - existing pubsub lifecycle, service lifecycle, stream false-success, and
    service delivery-loss ledger entries.
- Duplicate or too-close candidates rejected:
  - duplicate subscriber local ids detach live subscriber handles: accepted by
    `Noether the 2nd` as additional ISSUE-168 evidence, not distinct from the
    existing pubsub local-id collision root cause.
  - `send_unicast` returns success when the destination service receiver is
    closed: accepted by `Turing the 2nd` as additional ISSUE-060/ISSUE-119
    evidence, not a standalone issue because it composes stale service
    registration with ignored local-delivery failure.
- Root-cause summary impact: no new root cause; supplemental evidence maps to
  RC-3 and RC-6.

### Cycle after ISSUE-189: replicated-KV, service/control, and malformed service inputs

- Result: no accepted non-duplicate issue.
- Reviewer/explorer: `Erdos the 3rd`; local review by main agent.
- Local/source areas reviewed:
  - `src/service/replicate_kv_service.rs`,
    `src/service/replicate_kv_service/local_storage.rs`,
    `src/service/replicate_kv_service/remote_storage.rs`,
    `src/service/replicate_kv_service/messages.rs`
  - `src/requester.rs`, `src/service.rs`, `src/ctx.rs`, `src/lib.rs`,
    `src/msg.rs`, `src/quic.rs`, `src/stream.rs`
  - existing security, stream, replicated-KV, malformed-input, and service
    lifecycle tests and ledger entries.
- Duplicate or too-close candidates rejected:
  - arbitrary inbound replicated-KV RPC can create `RemoteStore` and queue full
    sync: ISSUE-045.
  - pubsub `Publish`/`Feedback` accepts unauthenticated membership state:
    ISSUE-039 and ISSUE-048.
  - `StreamConnectReq.source` is trusted without connection binding:
    ISSUE-018.
  - alias hint timeout arithmetic can overflow near `u64::MAX`: ISSUE-036.
  - metrics and visualization accept unsolicited `Info`/`Scan` updates:
    ISSUE-061, ISSUE-062, ISSUE-078, and ISSUE-079.
  - replicated-KV malformed snapshot/change/version/repair candidates map to
    existing evidence under ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-047,
    ISSUE-081 through ISSUE-089, ISSUE-111, ISSUE-138, ISSUE-141,
    ISSUE-143, ISSUE-154, ISSUE-171, ISSUE-175, ISSUE-184, and ISSUE-186.
  - public service/control stale requester, panic, service-id, and serialization
    candidates map to ISSUE-013, ISSUE-028, ISSUE-030, ISSUE-052,
    ISSUE-053, ISSUE-054, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076,
    ISSUE-091, ISSUE-096, ISSUE-097, ISSUE-098, and ISSUE-174.
- Root-cause summary impact: no new root cause; rejected candidates map to
  RC-1, RC-2, RC-3, RC-5, RC-6, and RC-7.

### Fuzz-phase no-new cycle: path flapping, pipe reliability, and graceful-stop hints

- Result: no accepted non-duplicate issue.
- Reviewer: `Rawls the 3rd`, forked subagent review, confirmed no-new
  classification.
- Local/source areas reviewed:
  - `src/router.rs`, `src/ctx.rs`, `src/peer/peer_internal.rs`,
    `src/service.rs`, `src/discovery.rs`, `src/lib.rs`
  - existing router, stream, discovery, and graceful-stop tests and ledger
    entries.
- Duplicate or too-close candidates rejected:
  - active route jumping/noisy path selection: ISSUE-003. Representative
    failing evidence remains
    `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`.
  - pipe/stream setup false success, blocking, and relay-loop failure modes:
    ISSUE-011, ISSUE-012, ISSUE-056, ISSUE-117, ISSUE-149, ISSUE-156,
    ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-180, and ISSUE-182.
    Representative failing evidence remains
    `cargo test open_stream_does_not_succeed_when_destination_service_queue_is_full -- --nocapture`.
  - non-seed expiry, seed preservation, and graceful-stop cleanup: ISSUE-004,
    ISSUE-051, ISSUE-118, ISSUE-167, ISSUE-170, ISSUE-185, and ISSUE-187.
    Representative failing evidence remains
    `cargo test discovery_timeout_must_remove_route_to_expired_non_seed -- --nocapture`
    and
    `cargo test peer_stopped_for_seed_must_not_remove_active_seed_route -- --nocapture`.
- Additional fuzz evidence reviewed:
  - `P2P_FUZZ_NODES=6 P2P_FUZZ_STEPS=800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
  - `P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1500 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
    passed with no new issue.
- Root-cause summary impact: no new root cause; candidates map to existing
  RC-3, RC-4, RC-6, and RC-7 patterns.
