# Issue Summary

Short review copy for the RED-team issue ledger. The detailed evidence,
reviewer decisions, scores, and failing tests remain in `docs/found_issues.md`.

## Audit Status

- Accepted issues: 204
- Missing issue scores: 0
- Current consecutive no-new-issue cycles: 29
- Stop condition: continue until 5 consecutive cycles find no new accepted
  issue; currently 29/5 after ISSUE-204.

## Root Cause Summary

### RC-1: Authenticated identity is not authoritative

- Representative issues: ISSUE-001, ISSUE-004, ISSUE-014, ISSUE-015,
  ISSUE-018, ISSUE-020, ISSUE-039, ISSUE-048, ISSUE-066, ISSUE-067,
  ISSUE-068, ISSUE-090, ISSUE-115, ISSUE-116, ISSUE-145, ISSUE-189,
  ISSUE-194.
- Pattern: message payloads and internal events carry peer ids, RPC ids, or
  source identities that are trusted without binding them to the live
  authenticated connection, local handle, expected responder, channel role, or
  the invariant that a shared-key holder may not authenticate as the local node
  or an arbitrary third-party peer.
- Minimal fix proposal: derive source identity from authenticated connections,
  validate `(ConnectionId, PeerId)` before processing main events, reject
  self-identity and unauthorized third-party peer admission before aliases are
  registered, and store expected responder/handle metadata before accepting
  answers.

### RC-2: Protocol state machines lack correlation/freshness checks

- Representative issues: ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-047,
  ISSUE-059, ISSUE-071, ISSUE-081 through ISSUE-089, ISSUE-095, ISSUE-099,
  ISSUE-110, ISSUE-111, ISSUE-138, ISSUE-141, ISSUE-143, ISSUE-152,
  ISSUE-154, ISSUE-155, ISSUE-158, ISSUE-166, ISSUE-171, ISSUE-175,
  ISSUE-186.
- Pattern: replicated-KV, alias, metrics, visualization, and pubsub flows accept
  stale, unsolicited, reordered, or mismatched responses or broadcasts because
  handlers do not verify request shape, bounds, version, continuation key,
  expected phase, membership generation, or whether an event actually advances
  activity.
- Minimal fix proposal: keep a small pending-request descriptor per flow and
  reject responses unless they match; for membership gossip, carry a generation
  or epoch and ignore older join/leave/heartbeat state. Refresh remote liveness
  only after an accepted event advances state or emits work.

### RC-3: Backpressure is inconsistent across async boundaries

- Representative issues: ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-118,
  ISSUE-119, ISSUE-120, ISSUE-123, ISSUE-124, ISSUE-125, ISSUE-126,
  ISSUE-127, ISSUE-133, ISSUE-136, ISSUE-147, ISSUE-153, ISSUE-157,
  ISSUE-163, ISSUE-164, ISSUE-178, ISSUE-182, ISSUE-184, ISSUE-198,
  ISSUE-199, ISSUE-200, ISSUE-201, ISSUE-202, ISSUE-203, ISSUE-204.
- Pattern: some paths drop on `try_send`, some await bounded sends from
  critical tasks, and others use unbounded queues or duplicate internal control
  work. Under load this causes silent loss, head-of-line blocking, unreported
  total fanout failure for failed awaited or nonblocking sends, or unbounded
  memory. RPC fanout can also count failed local or remote delivery attempts as
  live destinations. Transport config can also admit unused stream classes that
  no application task drains. Repair state machines can also duplicate
  in-flight repair requests before timeout or response.
- Minimal fix proposal: define a channel policy by event class; lifecycle and
  route updates need bounded retry/coalescing, service payload delivery needs
  explicit backpressure errors including zero-recipient fanout errors, and peer
  tasks must not await bounded lifecycle reporting before they can process
  traffic or cleanup. RPC paths should insert pending state only after at least
  one successful local or remote fanout. Disable unused QUIC stream classes or
  add explicit admission plus drain/reject handlers. Repair requests need typed
  pending descriptors and duplicate suppression until timeout or a matching
  response changes the range. Periodic metrics and visualization collectors
  should keep explicit in-flight scan state and coalesce ticks while an earlier
  scan broadcast is still backpressured. Metrics scan responses need a retry,
  timeout, or observable backpressure policy instead of fire-and-forget
  nonblocking unicast, plus bounded in-flight response state instead of
  duplicate detached awaited unicast tasks. Visualization scan responses need
  bounded in-flight response state instead of unbounded detached awaited
  unicast tasks.

### RC-4: Timeouts and setup cancellation are incomplete

- Representative issues: ISSUE-002, ISSUE-009, ISSUE-021, ISSUE-036,
  ISSUE-042, ISSUE-093, ISSUE-117, ISSUE-121, ISSUE-134, ISSUE-149,
  ISSUE-156, ISSUE-159, ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-176.
- Pattern: timeouts wrap only one await point, rely on unchecked timestamp
  arithmetic, use coarse global sweeps, or complete one side of setup before
  proving the end-to-end setup is still alive. Handshake tokens also lack
  nonce/challenge binding or replay caches.
- Minimal fix proposal: use checked/saturating deadline math, wrap every
  protocol phase in an end-to-end timeout, and tie relay downstream setup to
  upstream cancellation. Bind handshake responses to fresh request nonces and
  reject recently accepted tokens until expiry.

### RC-5: Application-level resource limits are missing

- Representative issues: ISSUE-010, ISSUE-024, ISSUE-027, ISSUE-035,
  ISSUE-041, ISSUE-043, ISSUE-045, ISSUE-046, ISSUE-100 through ISSUE-108,
  ISSUE-122, ISSUE-131, ISSUE-174, ISSUE-196.
- Pattern: decoded service-level collections, pending maps, cache sets,
  tombstones, remote stores, retained channel state, and outbound event queues
  often have no item-count or lifetime cap.
- Minimal fix proposal: add per-structure caps with deterministic
  eviction/rejection: max rows per message, max peers per alias/channel, max
  pending RPCs/finds, max tombstones/remotes, max queued outbound events, and
  prune empty channel state on teardown. Mutation APIs that enqueue work should
  return backpressure errors or coalesce superseded work.

### RC-6: Lifecycle cleanup and stale handles are inconsistent

- Representative issues: ISSUE-028, ISSUE-029, ISSUE-051, ISSUE-057,
  ISSUE-060, ISSUE-064, ISSUE-065, ISSUE-069 through ISSUE-076, ISSUE-108,
  ISSUE-128 through ISSUE-132, ISSUE-135, ISSUE-139, ISSUE-142, ISSUE-144,
  ISSUE-148, ISSUE-150, ISSUE-151, ISSUE-161, ISSUE-162, ISSUE-165,
  ISSUE-167, ISSUE-168, ISSUE-170, ISSUE-179, ISSUE-183, ISSUE-185,
  ISSUE-187, ISSUE-188, ISSUE-193, ISSUE-195.
- Pattern: requesters, services, peer aliases, channel state, and cached hints
  can outlive the owner they represent; shutdown paths can panic, leak, emit
  false public events, keep stale routes/cache entries, announce shutdown while
  local authority remains active, or drop remote membership that arrives before
  local channel ownership exists. Peer lifecycle events also do not consistently
  reach service-owned per-peer membership or public network-event consumers.
  Connection teardown can also reset metric names through the wrong metric kind
  or reset monotonic counters to zero.
- Minimal fix proposal: add generation or liveness tokens to cloned requesters
  and local handles, make closed channels return `Err`, and centralize teardown
  for aliases, metrics, routes, caches, and service ids. Shutdown controls
  should enter an explicit terminal state so later register/find operations are
  rejected or no-op. Fan out accepted peer stopped/disconnected events to
  services that own per-peer state and surface them through the public network
  event API. Retain bounded pubsub remote membership even before local handles
  exist, then replay it when local handles are created. Keep each metric name on
  one metric kind during live emission and teardown resets, and do not reset
  monotonic counters during teardown.

### RC-7: Routing/discovery accepts unstable topology

- Representative issues: ISSUE-003, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-008, ISSUE-033, ISSUE-044, ISSUE-055, ISSUE-092, ISSUE-103,
  ISSUE-112 through ISSUE-114, ISSUE-160, ISSUE-161, ISSUE-164, ISSUE-167,
  ISSUE-177, ISSUE-180, ISSUE-181, ISSUE-190, ISSUE-192, ISSUE-197.
- Pattern: route/discovery inputs can include local ids, self seeds, stale
  addresses, overflowed metrics, over-hop routes, duplicate connection races,
  explicit connect addresses that are ignored by peer-id-only fast paths, or
  tiny RTT jitter that changes active paths too aggressively. Malformed route
  or discovery syncs can also contain duplicate destination rows whose last
  value silently wins before validation. Stream relay setup and unicast
  forwarding can also forward back to the ingress connection when route state
  forms a loop, and local advertise config can gossip non-dialable addresses.
- Minimal fix proposal: sanitize before insertion: reject local/self candidates
  and over-hop routes, pin authenticated direct paths for their peer ids, use
  checked metric math, ignore stale discovery timestamps, reject duplicate
  destination rows in one route or discovery sync, coalesce duplicate connects,
  validate already-connected peer addresses, add hysteresis before switching
  active paths, and reject relay stream or unicast hops that point back to the
  ingress connection. Validate configured local advertise addresses before
  gossiping them.

### RC-8: Public examples are not compile-checked

- Representative issues: ISSUE-191.
- Pattern: documentation snippets can drift from the exported API because they
  are not compiled as examples, doctests, or compile tests. This can leave the
  getting-started path with invalid result handling, mutability, or type usage
  even when maintained examples still compile.
- Minimal fix proposal: make README snippets executable examples or doctests,
  and add a focused compile gate for the getting-started path. Keep snippets
  using real `Result` handling and mutable bindings where the API requires
  mutation.

## Recent Accepted Issues

- ISSUE-168, score 44: duplicate pubsub local ids detach live publisher and
  subscriber handles. Reviewers: Jason the 3rd, Noether the 2nd.
- ISSUE-169, score 68: stream open hangs while writing connect request to a
  flow-control-stalled peer. Reviewer: independent validation after subagent
  `019ede01-2c64-7e11-af87-56677fa09649`.
- ISSUE-170, score 62: PeerStopped forwarding loops indefinitely in cyclic
  meshes. Reviewer: Banach the 3rd.
- ISSUE-171, score 60: replicated KV full resync deletes visible data before
  replacement snapshot. Reviewer: Fermat the 3rd.
- ISSUE-172, score 68: outbound peer setup hangs while writing `ConnectReq` to
  a stalled peer. Reviewer: James the 3rd.
- ISSUE-173, score 68: inbound peer setup hangs while writing `ConnectRes` to
  a stalled peer. Reviewer: Peirce the 3rd.
- ISSUE-174, score 46: QUIC object writer can bypass `MAX_SIZE` with
  non-deterministic serialization. Reviewer: Hypatia the 3rd.
- ISSUE-175, score 42: replicated KV emits delete changes for keys that were
  never present. Reviewer: Volta the 3rd.
- ISSUE-176, score 66: shared-key handshake response tokens are replayable.
  Reviewer: Harvey the 3rd.
- ISSUE-177, score 38: `connect()` reports success for a different address
  when the peer id is already connected. Reviewer: Helmholtz the 3rd.
- ISSUE-178, score 57: pubsub RPC treats closed local event channels as live
  destinations. Reviewer: Russell the 3rd.
- ISSUE-179, score 49: local alias shutdown leaves pending find waiters alive.
  Reviewer: Socrates the 3rd.
- ISSUE-180, score 64: relay stream setup can forward back to the ingress peer.
  Reviewer: Carver the 3rd.
- ISSUE-181, score 45: local advertise config can gossip unroutable wildcard
  addresses. Reviewer: Nash the 3rd.
- ISSUE-182, score 52: QUIC admits unused unidirectional streams. Reviewer:
  Pascal the 3rd.
- ISSUE-183, score 53: local alias shutdown keeps serving local aliases.
  Reviewer: Newton the 3rd.
- ISSUE-184, score 57: replicated KV duplicates in-flight FetchChanged repairs
  for the same gap. Reviewer: Poincare the 3rd.
- ISSUE-185, score 56: pubsub keeps remote subscriber membership after graceful
  peer stop. Reviewer: Popper the 3rd.
- ISSUE-186, score 54: ignored replicated-KV broadcasts refresh stale remote
  activity. Reviewer: Nietzsche the 3rd.
- ISSUE-187, score 49: graceful PeerStopped is hidden from public network
  events. Reviewer: Mendel the 3rd.
- ISSUE-188, score 51: pubsub drops early remote publisher joins before local
  channel creation. Reviewer: Noether the 3rd.
- ISSUE-189, score 72: inbound handshake accepts a remote peer claiming the
  local peer id. Reviewer: Zeno the 3rd.
- ISSUE-190, score 43: duplicate route-sync destinations silently keep the
  last metric. Reviewer: Epicurus the 3rd.
- ISSUE-191, score 18: README getting-started public API example does not
  compile. Reviewer: Halley the 3rd.
- ISSUE-192, score 39: duplicate discovery-sync peers silently keep the last
  address. Reviewer: Arendt the 3rd.
- ISSUE-193, score 31: connection teardown emits RTT as both gauge and counter.
  Reviewer: Copernicus the 3rd.
- ISSUE-194, score 88: inbound handshake accepts arbitrary third-party peer-id
  claims. Reviewer: Confucius the 3rd.
- ISSUE-195, score 42: connection teardown resets monotonic counters to zero.
  Reviewer: Dalton the 3rd.
- ISSUE-196, score 47: replicated-KV local mutations build an unbounded
  outbound event queue. Reviewer: Averroes the 3rd.
- ISSUE-197, score 64: unicast relay can forward packets back to the ingress
  connection. Reviewer: Lagrange the 3rd.
- ISSUE-198, score 54: `try_send_broadcast` silently loses all copies under
  peer queue pressure. Reviewer: Dewey the 3rd.
- ISSUE-199, score 52: `send_broadcast` silently succeeds when every peer send
  fails. Reviewer: Maxwell the 3rd.
- ISSUE-200, score 58: metrics collector duplicates scan broadcasts behind
  hidden backpressure. Reviewer: Bohr the 3rd.
- ISSUE-201, score 57: visualization collector duplicates scan broadcasts
  behind hidden backpressure. Reviewer: Plato the 3rd.
- ISSUE-202, score 55: metrics scan responses are dropped under peer-control
  backpressure. Reviewer: Ramanujan the 3rd.
- ISSUE-203, score 56: visualization scan responses accumulate behind
  peer-control backpressure. Reviewer: Sartre the 3rd.
- ISSUE-204, score 56: metrics scan responses accumulate behind peer-control
  backpressure. Reviewer: Anscombe the 4th.

## Next Candidate To Validate

- Continue RED-team review around directed service replies, pubsub/alias
  response paths, and high-load backpressure. Randomized node-action churn
  fuzzing has already started and should continue from the steady-valid passing
  baseline when five consecutive no-new cycles accumulate again.

## Recent Fuzz Evidence

- Sanitized churn fuzz duplicate:
  `RUST_LOG=error P2P_FUZZ_SEED=2182001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. The accepted failure was the
  outbound `PeerConnectError` path panicking at `src/peer.rs:133` with
  `should send to main: SendError`; repeated peer-stopped/backpressure logs
  overlap existing ISSUE-170 and RC-3/RC-6 churn noise without adding a new
  root cause.
- Steady-valid random action fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=2181001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with `1 passed; 0 failed; 289 filtered out; finished in 16.84s`.
  Classification was `NO_NEW_PASS`; because the cycle had no failing evidence,
  it adds no accepted issue and no root-cause impact.
- Extended sanitized churn fuzz:
  `RUST_LOG=error P2P_FUZZ_SEED=2180001 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Ptolemy the 4th`
  confirmed the `src/peer.rs:92` send-to-main panic is the already-accepted
  early `PeerConnectError` reporting panic after main-loop shutdown; churn
  queue/stop-forwarding logs were supporting noise, not a new issue.
- Focused discovery freshness source/test review:
  `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`
  failed with duplicate evidence for ISSUE-093. Reviewer `Confucius the 4th`
  confirmed the `src/discovery.rs:328` assertion is the already-accepted fresh
  restart advertisement suppression by stale stop tombstone under RC-4.
- Focused discovery tombstone source/test review:
  `cargo test graceful_stop_tombstones_must_be_bounded_for_unknown_peers -- --nocapture`
  failed with duplicate evidence for ISSUE-122. Reviewer `Hubble the 4th`
  confirmed the `src/discovery.rs:280` assertion is the already-accepted
  unbounded stopped-peer tombstone resource flaw under RC-5.
- Focused pubsub timeout source/test review:
  `cargo test pubsub_publish_rpc_must_respect_short_timeout -- --nocapture`
  failed with duplicate evidence for ISSUE-121. Reviewer `Dirac the 4th`
  confirmed the `src/tests/pubsub.rs:618` timeout is the already-accepted
  one-second pubsub RPC sweep granularity flaw under RC-4.
- Focused cross-node broadcast delivery review:
  `cargo test inbound_broadcast_must_not_drop_when_service_queue_is_full -- --nocapture`
  now passes. Reviewer `Hilbert the 4th` classified this as existing-issue
  fixed/no-new evidence for ISSUE-120 under RC-3: local queue-full broadcast
  silent drop is fixed by awaited local delivery, without proving broader RC-3
  fixes.
- Focused cross-node unicast delivery review:
  `cargo test inbound_unicast_must_not_drop_when_service_queue_is_full -- --nocapture`
  now passes, but
  `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
  still fails at `src/tests/cross_nodes.rs:203`. Reviewer `Noether the 4th`
  classified this as existing-issue partial-fix/no-new evidence for ISSUE-119
  under RC-3: queue-full silent drop is fixed by awaited local delivery, while
  closed-receiver success reporting remains open.
- Focused visualization resource-bound source/test review:
  `cargo test visualization_info_batches_must_be_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-105. Reviewer `Popper the 4th`
  confirmed the `src/service/visualization_service.rs:248` assertion is the
  already-accepted missing service-level row cap for visualization `Info`
  batches under RC-5.
- Focused alias backpressure source/test review:
  `cargo test alias_internal_control_backlog_must_be_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-127. Reviewer `Euclid the 4th`
  confirmed the `src/service/alias_service.rs:484` assertion is the
  already-accepted unbounded alias internal control backlog under RC-3.
- Focused pubsub backpressure source/test review:
  `cargo test pubsub_internal_control_backlog_must_be_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-126. Reviewer `Nietzsche the 4th`
  confirmed the `src/service/pubsub_service.rs:754` assertion is the
  already-accepted unbounded pubsub internal control backlog under RC-3.
- Focused metrics resource-bound source/test review:
  `cargo test metrics_info_batches_must_be_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-104. Reviewer `Bernoulli the 4th`
  confirmed the `src/service/metrics_service.rs:67` assertion is the
  already-accepted missing service-level row cap for metrics `Info` batches
  under RC-5.
- Focused discovery graceful-stop source/test review:
  `cargo test graceful_shutdown_removes_stopped_non_seed -- --nocapture`
  passed. Reviewer `Gibbs the 4th` confirmed the repeated reconnect and route
  cleanup logs are duplicate/no-new symptoms mapped to ISSUE-153,
  ISSUE-051/ISSUE-167, ISSUE-118, ISSUE-170, ISSUE-185, ISSUE-187, RC-6, and
  RC-7 rather than failing evidence for ISSUE-205.
- Focused stream admission source/test review:
  `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-117. Reviewer `Beauvoir the 4th`
  confirmed the `src/tests/stream.rs:575` assertion is the already-accepted
  idle inbound stream-connect admission gap covered by RC-4.
- Focused handshake source/test review:
  `cargo test inbound_handshake_must_reject_peer_claiming_third_party_id -- --nocapture`
  failed with duplicate evidence for ISSUE-194. Reviewer `Zeno the 4th`
  confirmed the `src/peer.rs:683` assertion is the already-accepted
  shared-key third-party `PeerId` admission flaw covered by RC-1.
- Focused replicated-KV source/test review:
  `cargo test full_sync_must_reject_stale_terminal_snapshot_after_continuation_request -- --nocapture`
  failed with duplicate evidence for ISSUE-143. Reviewer `Sagan the 4th`
  confirmed the panic at `remote_storage.rs:919` is the already-accepted stale
  terminal snapshot case where full sync incorrectly transitions to
  `Working(Version(3))` while a continuation range is outstanding.
- Extended invalid churn fuzz:
  `P2P_FUZZ_SEED=2179001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1400 cargo test fuzz_random_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Nash the 4th`
  confirmed the `src/ctx.rs:34` fixed service-array bounds panic is the same
  out-of-range `P2pServiceId(256)` root cause; channel-closed sends after the
  panic were consequential churn/lifecycle symptoms.
- Extended sanitized churn fuzz:
  `P2P_FUZZ_SEED=2178001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139 and ISSUE-170. Reviewer
  `Plato the 4th` confirmed the `src/peer.rs:92`/`src/peer.rs:133`
  send-to-main panics are ISSUE-139, while the 9,813 forwarded-stop alias
  errors strengthen ISSUE-170 without adding a new root cause.
- Extended valid-action fuzz:
  `P2P_FUZZ_SEED=2177001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2500 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `Godel the 4th` confirmed the `src/router.rs:76` stale-sync panic after
  repeated `PeerStopped` reports and the later `src/peer.rs:92` send-to-main
  panic are already-covered root causes.
- Extended steady-valid fuzz:
  `P2P_FUZZ_SEED=2176001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with `1 passed; 0 failed`. Reviewer `Fermat the 4th` confirmed the
  non-fatal route reselection, `path not found`, and `queue main loop full`
  warnings map to existing RC-7 and RC-3 entries.
- Extended invalid-wire action fuzz:
  `P2P_FUZZ_SEED=0x205301 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1000 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/ctx.rs:34`, duplicate evidence for ISSUE-053. Reviewer
  `Hooke the 4th` confirmed the invalid service-id action hits the same fixed
  service-array bounds bug; the harness reported `seed=24301` because hex env
  seeds fall back to the current default.
- Churn fuzz with invalid wire actions:
  `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=40 cargo test fuzz_random_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/ctx.rs:33`, duplicate evidence for ISSUE-053.
- Valid-only churn fuzz:
  `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=60 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/router.rs:76`, duplicate evidence for ISSUE-063.
- Extended valid-action fuzz:
  `P2P_FUZZ_SEED=0x205101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1200 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/router.rs:76`, duplicate evidence for ISSUE-063. Reviewer
  `Planck the 4th` confirmed this is stale `PeerData::Sync` after direct route
  removal, not a new accepted issue.
- Sanitized churn fuzz:
  `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=120 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/peer.rs:106`, duplicate evidence for ISSUE-139.
- Extended sanitized churn fuzz:
  `P2P_FUZZ_SEED=0x205201 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1200 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  panics at `src/peer.rs:133`, duplicate evidence for ISSUE-139. Reviewer
  `Faraday the 4th` confirmed this is the outbound early `PeerConnectError`
  path using `expect("should send to main")` after main-loop shutdown, not a
  new accepted issue.
- Steady valid fuzz:
  `P2P_FUZZ_NODES=3 P2P_FUZZ_STEPS=150 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed after bypassing invalid service ids, forged `PeerStopped`, and
  stop/restart churn.
- Extended steady valid fuzz:
  `P2P_FUZZ_NODES=4 P2P_FUZZ_STEPS=300 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue.
- Extended steady valid fuzz:
  `P2P_FUZZ_NODES=5 P2P_FUZZ_STEPS=500 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue.
- Extended steady valid fuzz:
  `P2P_FUZZ_NODES=6 P2P_FUZZ_STEPS=800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Output still shows route reselection noise and
  bounded queue pressure warnings, but no failing test evidence for a new
  accepted issue.
- Extended steady valid fuzz:
  `P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1500 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. The run again produced heavy active-path
  reselection logging and some bounded queue pressure warnings, but no panic or
  failing assertion.
- Alternate-seed extended steady valid fuzz:
  `P2P_FUZZ_SEED=0x600df00d P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1500 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. The alternate action ordering reproduced route
  reselection noise and bounded queue pressure warnings without a new failing
  assertion.
- Ten-node extended steady valid fuzz:
  `P2P_FUZZ_SEED=0xdecafbad P2P_FUZZ_NODES=10 P2P_FUZZ_STEPS=2500 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Output still showed active-path reselection noise
  and bounded queue pressure warnings, but no panic or failing assertion.
- Eleven-node extended steady valid fuzz:
  `P2P_FUZZ_SEED=0x204204 P2P_FUZZ_NODES=11 P2P_FUZZ_STEPS=2800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Output again showed active route reselection noise
  and endpoint-driver-drop shutdown logs at test end, but no panic or failing
  assertion.
- Twelve-node extended steady valid fuzz:
  `P2P_FUZZ_SEED=0x204205 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Reviewer `Locke the 4th` mapped route reselection
  noise, main-loop pressure warnings, and teardown logs to existing RC-3,
  RC-6, and RC-7 entries.
- Thirteen-node extended steady valid fuzz:
  `P2P_FUZZ_SEED=0x204206 P2P_FUZZ_NODES=13 P2P_FUZZ_STEPS=3600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Reviewer `Jason the 4th` mapped route reselection,
  queue-full pressure, and temporary unavailable-route symptoms to existing
  RC-3 and RC-7/stale-route entries.
- Fourteen-node extended steady valid fuzz:
  `P2P_FUZZ_SEED=0x204207 P2P_FUZZ_NODES=14 P2P_FUZZ_STEPS=4200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Reviewer `Harvey the 4th` mapped route
  reselection, high-load queue pressure, temporary path-not-found, and
  successful stream-processing logs to existing RC-3 and RC-7/stale-route
  entries.
- Fifteen-node post-stop-condition steady valid fuzz:
  `P2P_FUZZ_SEED=0x205001 P2P_FUZZ_NODES=15 P2P_FUZZ_STEPS=5000 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no new issue. Reviewer `Peirce the 4th` mapped route
  reselection, high-load queue pressure, temporary path-not-found, and
  successful stream-processing logs to existing RC-3 and RC-7/stale-route
  entries.

## Recent No-New Audit

- Cycle after ISSUE-204 no-new cycle 15 reviewed handshake/authentication and
  peer admission with forked reviewer `Zeno the 4th`. The focused
  third-party-id handshake test failed with exit code 101, but it was
  duplicate evidence for ISSUE-194: inbound `run_connection` trusts a
  shared-key holder's caller-supplied `req.from` as the authenticated peer
  identity. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 14 reviewed replicated-KV remote/local
  storage and message handling with forked reviewer `Sagan the 4th`. The
  focused stale-terminal-snapshot test failed with exit code 101, but it was
  duplicate evidence for ISSUE-143: a stale terminal snapshot completed full
  sync into `Working(Version(3))` while a continuation range was still
  outstanding. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 13 ran an eight-node invalid churn fuzz
  pass with forked reviewer `Nash the 4th`. The run failed with exit code 101,
  but the failure was duplicate evidence for ISSUE-053: an invalid service id
  reached the fixed service array and panicked at `src/ctx.rs:34`. Follow-on
  channel-closed send logs were reviewed as churn/lifecycle consequences, so
  no accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 12 ran an eight-node sanitized churn fuzz
  pass with forked reviewer `Plato the 4th`. The run failed, but the failure
  was duplicate evidence for ISSUE-139 and ISSUE-170: early
  `PeerConnectError` reporting panicked at `src/peer.rs:92` and
  `src/peer.rs:133` after the main loop closed, and the run produced 9,813
  forwarded-stop alias errors from duplicate `PeerStopped` amplification.
  Queue-full warnings mapped to RC-3 backpressure and ISSUE-118-style
  congested shutdown overlap, so no accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 11 ran an eight-node valid-action fuzz
  pass with forked reviewer `Godel the 4th`. The run failed, but the failure
  was duplicate evidence for ISSUE-063 and ISSUE-139: stale sync reached
  `RouterTable::apply_sync` after repeated `PeerStopped` reports and panicked
  at `src/router.rs:76`, then an incoming `PeerConnectError` report panicked
  at `src/peer.rs:92` because the main loop was already closed. Queue-full
  warnings mapped to RC-3 backpressure, so no accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 10 ran an eight-node steady-valid fuzz
  pass with forked reviewer `Fermat the 4th`. The run passed with no panic or
  failing assertion. The reviewer mapped 335 route reselections to
  ISSUE-003/RC-7, 20 `queue main loop full` warnings to RC-3 backpressure, and
  2 transient `path to ... not found` warnings to existing stale/unavailable
  route entries, so no accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 9 ran an eight-node invalid-wire action
  fuzz pass with forked reviewer `Hooke the 4th`. The run failed, but the
  failure was duplicate evidence for ISSUE-053: an invalid service id
  `P2pServiceId(256)` reached the fixed service array and panicked at
  `src/ctx.rs:34`. The hex seed literal was ignored by the current fuzz env
  parser and the harness reported default `seed=24301`; this was recorded as
  an evidence-handling observation, not a new accepted issue.
- Cycle after ISSUE-204 no-new cycle 8 ran an eight-node sanitized churn fuzz
  pass with forked reviewer `Faraday the 4th`. The run failed, but the failure
  was duplicate evidence for ISSUE-139: the outbound early
  `PeerConnectError` path panicked at `src/peer.rs:133` while sending to a
  closed main loop. Shutdown/refused-connect churn and temporary route/lifecycle
  logs mapped to existing RC-6 and RC-7 entries, so no accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 7 ran an eight-node valid-action fuzz
  pass with forked reviewer `Planck the 4th`. The run failed, but the failure
  was duplicate evidence for ISSUE-063: stale `PeerData::Sync` reached
  `RouterTable::apply_sync` after the direct route was gone and panicked at
  `src/router.rs:76`. Queue pressure, route noise, and PeerStopped forwarding
  symptoms mapped to existing RC-3, RC-7, and lifecycle entries, so no accepted
  issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 6 ran a fifteen-node steady-valid fuzz
  pass after the prior 5/5 threshold, with forked reviewer `Peirce the 4th`.
  The run passed; observed route reselection/path-jumping noise, high-load
  queue pressure, temporary path-not-found warnings, and successful stream
  processing mapped to existing RC-3 and RC-7/stale-route patterns, so no
  accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 5 ran a fourteen-node steady-valid fuzz
  pass with forked reviewer `Harvey the 4th`. The run passed; observed route
  reselection/path-jumping noise, high-load queue pressure, temporary
  path-not-found warnings, and successful stream processing mapped to existing
  RC-3 and RC-7/stale-route patterns, so no accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 4 ran a thirteen-node steady-valid fuzz
  pass with forked reviewer `Jason the 4th`. The run passed; observed route
  reselection/path-jumping noise, high-load queue pressure, and temporary
  unavailable-route symptoms mapped to existing RC-3 and RC-7/stale-route
  patterns, so no accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 3 ran a twelve-node steady-valid fuzz
  pass with forked reviewer `Locke the 4th`. The run passed; observed route
  reselection noise, bounded main-loop pressure warnings, and endpoint teardown
  logs mapped to existing RC-3, RC-6, and RC-7 patterns, so no accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 2 reviewed shared-key handshake
  validation, QUIC transport admission, stream/object codec helpers, and
  public constructor/config validation with forked reviewer `Turing the 4th`.
  Rejected candidates mapped to existing RC-3, RC-4, RC-6, and RC-7 patterns;
  no accepted issue or summary root-cause change.
- Cycle after ISSUE-204 no-new cycle 1 reviewed pubsub directed
  response/fanout paths, replicated-KV malformed snapshot/repair/resource and
  lifecycle paths, alias lookup/shutdown/find backlog behavior, and
  route/discovery/stopped-peer/stale-event behavior with forked reviewer
  `Russell the 4th`. Rejected candidates mapped to existing RC-2, RC-3, RC-4,
  RC-5, RC-6, and RC-7 patterns; no accepted issue or summary root-cause
  change.
- Fuzz-phase no-new cycle reviewed path flapping, pipe/stream reliability, and
  non-seed/seed graceful-stop hints with forked reviewer `Rawls the 3rd`.
  Rejected candidates mapped to ISSUE-003, ISSUE-004, ISSUE-011, ISSUE-012,
  ISSUE-051, ISSUE-056, ISSUE-117, ISSUE-118, ISSUE-149, ISSUE-156,
  ISSUE-167, ISSUE-169, ISSUE-170, ISSUE-172, ISSUE-173, ISSUE-180,
  ISSUE-182, ISSUE-185, and ISSUE-187. Root causes remain RC-3, RC-4, RC-6,
  and RC-7; no accepted issue or summary root-cause change.
- Cycle after ISSUE-193 no-new cycle 4 reviewed public API/node lifecycle,
  config/transport/security helpers, discovery/router config edges,
  README/examples, and the fuzz harness. Rejected candidates mapped to existing
  RC-1, RC-3, RC-6, RC-7, and RC-8 patterns, so no root-cause summary change
  was needed. This reached the five-cycle threshold and moves the audit into
  randomized node-action fuzzing.
- Cycle after ISSUE-193 no-new cycle 3 reviewed alias lifecycle/cache/finder,
  metrics and visualization collector paths, service/context boundaries,
  stream codec helpers, and peer-alias control wrappers. Rejected candidates
  mapped to existing RC-1 through RC-7 patterns, so no root-cause summary
  change was needed.
- Cycle after ISSUE-193 no-new cycle 2 reviewed pubsub lifecycle, publisher
  and subscriber handle/requester behavior, pubsub RPC/member accounting, and
  replicated-KV service, message, local-storage, and remote-storage paths.
  Rejected candidates mapped to existing RC-1, RC-2, RC-3, RC-5, and RC-6
  patterns, so no root-cause summary change was needed.
- Cycle after ISSUE-193 no-new cycle 1 reviewed public network
  control/shutdown, router/discovery/neighbour tick paths, requesters,
  QUIC/secure handshake admission, and peer stream admission. Rejected
  candidates mapped to existing RC-3, RC-4, RC-6, and RC-7 patterns, so no
  root-cause summary change was needed.
- Cycle after ISSUE-193 reviewed metrics/visualization internals, the fuzz
  harness, malformed wire paths, and alias state. Rejected candidates mapped to
  existing RC-1, RC-2, RC-3, RC-4, RC-5, and RC-6 patterns, so no root-cause
  summary change was needed.
- Cycle after ISSUE-189 reviewed replicated-KV, service/control, and malformed
  service-input paths. Rejected candidates mapped to existing RC-1, RC-2,
  RC-3, RC-5, RC-6, and RC-7 patterns, so no root-cause summary change was
  needed.
