# Issue Summary

Short review copy for the RED-team issue ledger. The detailed evidence,
reviewer decisions, scores, and failing tests remain in `docs/found_issues.md`.

## Audit Status

- Accepted issues: 207
- Missing issue scores: 0
- Current consecutive no-new-issue cycles: 0
- Stop condition: continue until 5 consecutive cycles find no new accepted
  issue; currently 0/5 after ISSUE-207.
- Fix phase status: ISSUE-001, ISSUE-003, ISSUE-004, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-002, ISSUE-008, ISSUE-009, ISSUE-010, ISSUE-011, ISSUE-012, ISSUE-013, ISSUE-014, ISSUE-015, ISSUE-017, ISSUE-020, ISSUE-021, ISSUE-024, ISSUE-027, ISSUE-033, ISSUE-034, ISSUE-039, ISSUE-047, ISSUE-048, ISSUE-055, ISSUE-059, ISSUE-103, ISSUE-115, ISSUE-116, ISSUE-118, ISSUE-119, ISSUE-120, ISSUE-122, ISSUE-123,
  ISSUE-124, ISSUE-125, ISSUE-126, ISSUE-127, ISSUE-128, ISSUE-129, ISSUE-130,
  ISSUE-131, ISSUE-132, ISSUE-133, ISSUE-134, ISSUE-135, ISSUE-136, ISSUE-137,
  ISSUE-140, ISSUE-143, ISSUE-145, ISSUE-147, ISSUE-148, ISSUE-150, ISSUE-151,
  ISSUE-152, ISSUE-153, ISSUE-154, ISSUE-155, ISSUE-156, ISSUE-157, ISSUE-158,
  ISSUE-159, ISSUE-160, ISSUE-161, ISSUE-163, ISSUE-164, ISSUE-053, ISSUE-063, ISSUE-086, ISSUE-091, ISSUE-139, ISSUE-146, ISSUE-168, ISSUE-170,
  ISSUE-149, ISSUE-169, ISSUE-174, ISSUE-176, ISSUE-181, ISSUE-189, ISSUE-190, ISSUE-191, ISSUE-192, ISSUE-193,
  ISSUE-194, ISSUE-195, ISSUE-196, ISSUE-197, ISSUE-198, ISSUE-199,
  ISSUE-200, ISSUE-201, ISSUE-202, ISSUE-203, ISSUE-204, ISSUE-205, ISSUE-206, ISSUE-207, ISSUE-097, ISSUE-098, and ISSUE-018 have focused
  fixes committed.
  ISSUE-003 is fixed by `cfc8e57`;
  ISSUE-001 and ISSUE-004 are covered by the ISSUE-170 ownership-validation follow-up
  `87cf6ce`; earlier fixes are `648cfd0`, `2cbf096`, `15b788c`, and
  `4997404`.

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
  ISSUE-110, ISSUE-111, ISSUE-143,
  ISSUE-166, ISSUE-171, ISSUE-175,
  ISSUE-186, ISSUE-205, ISSUE-206.
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

- Representative issues: ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-123,
  ISSUE-124, ISSUE-125, ISSUE-126,
  ISSUE-127, ISSUE-136, ISSUE-153,
  ISSUE-178, ISSUE-182, ISSUE-184, ISSUE-198, ISSUE-199,
  ISSUE-200, ISSUE-201, ISSUE-202, ISSUE-203, ISSUE-204.
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
  ISSUE-042, ISSUE-093, ISSUE-117, ISSUE-121, ISSUE-149,
  ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-176, ISSUE-207.
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
  ISSUE-148, ISSUE-150, ISSUE-151, ISSUE-161, ISSUE-165,
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
  ISSUE-112 through ISSUE-114, ISSUE-164, ISSUE-167,
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

## Recently Fixed Issues

- ISSUE-118: fixed by making `P2pNetwork::shutdown_gracefully` attempt
  peer-stopped notifications concurrently with `futures::future::join_all`
  under one outer one-second timeout. Root cause was a sequential per-peer
  timeout around `send_wait`, which made shutdown latency scale by congested
  peer count. The fix preserves best-effort logging, avoids detached tasks, and
  still closes the endpoint after completion or the global deadline.
  Verification:
  `cargo test shutdown_gracefully_must_not_wait_one_second_per_congested_peer -- --nocapture`.
- ISSUE-119: fixed by adding a direct-route unicast service-admission ack path.
  Root cause was `send_unicast` reporting success after peer-alias enqueue
  while the destination peer could later fail local service delivery. Direct
  next-hop sends now use `PeerMessage::UnicastWithAck`/`UnicastAck` with
  bounded pending ack tracking, and the receiver sends the ack only after local
  `send_local_service_event(...)` returns. Missing, closed, full, or timed-out
  direct destination service delivery is now sender-visible. Relayed routes
  keep existing enqueue-to-next-hop semantics, and `try_send_unicast` remains
  fire-and-forget. Verification:
  `cargo test unicast_must_not_report_success_when_destination_service_receiver_is_closed -- --nocapture`
  and
  `cargo test inbound_unicast_must_not_drop_when_service_queue_is_full -- --nocapture`.
- ISSUE-120: fixed by replacing inbound broadcast local delivery's lossy
  bounded-service `try_send` with awaited `send_local_service_event(...).await`
  over bounded `service.send(...)` plus timeout/backpressure. This preserves
  receiver-side broadcast events under transient local service queue pressure.
  Caveat: broadcast senders still do not get per-recipient local service
  consumption acknowledgements. Verification:
  `cargo test inbound_broadcast_must_not_drop_when_service_queue_is_full -- --nocapture`.
- ISSUE-147: fixed by changing inbound `PeerMessage::Sync` handling in
  `PeerConnectionInternal::on_msg` from lossy `try_send` to bounded
  `send().await`. A briefly full main queue now backpressures the peer
  connection task and still delivers valid route/discovery sync; telemetry,
  lifecycle, outbound alias sync, and peer-stopped paths remain unchanged.
  Verification:
  `cargo test valid_sync_must_survive_full_main_event_queue -- --nocapture`.
- ISSUE-148: fixed by making remote `AliasMessage::Shutdown` remove the sender
  from pending cached-hint lookup state after cache eviction. When that was the
  last hint peer, the request now transitions to scan state and broadcasts
  `Scan(alias_id)` while keeping waiters pending; local `AliasControl::Shutdown`
  behavior remains unchanged. Verification:
  `cargo test service::alias_service::test::shutdown_from_cached_hint_must_unblock_pending_find -- --nocapture`.
- ISSUE-150: fixed by `73c63ec` (`fix: isolate pubsub local handle ownership`).
  Pubsub destroy controls now return on unknown channels and on exact-handle
  removal misses, so stale publisher/subscriber destroy controls do not create
  phantom channel state or broadcast false leave events. Verification:
  `cargo test stale_pubsub_destroy_must_not_create_phantom_channel -- --nocapture`.
- ISSUE-151: fixed by `87cf6ce` (`fix: validate peer stopped ownership`).
  Accepted `PeerStopped` events now remove the neighbour and tombstone removed
  direct connection ids in the router, so a still-running connection ticker
  cannot recreate the stopped peer's direct route. Verification:
  `cargo test peer_stopped_route_must_not_be_resurrected_by_connection_ticker -- --nocapture`.
- ISSUE-152: fixed by correlating `AliasMessage::NotFound` with an active
  cached-hint lookup before mutating alias cache. Only pending
  `FindRequestState::CheckHint` peers can evict their own hint and trigger scan
  failover; unsolicited or `Scan`-state `NotFound` messages are ignored.
  Verification:
  `cargo test stale_not_found_must_not_evict_alias_cache_without_pending_check -- --nocapture`,
  `cargo test test_find_cached_alias_not_found -- --nocapture`, and
  `cargo test shutdown_from_cached_hint_must_unblock_pending_find -- --nocapture`.
- ISSUE-153: fixed by coalescing connect work against existing neighbour
  connection attempts. `NetworkNeighbours::has_peer_connection_attempt` matches
  connected peers and pending outbound attempts for the target peer id, but not
  unauthenticated inbound attempts. Best-effort discovery/requester connects
  coalesce duplicate in-flight `endpoint.connect` work, while awaited duplicate
  connects return an error instead of synthetic success for a potentially
  different socket address. Immediate connect errors and later
  `PeerConnectError` cleanup still allow retry.
  Verification:
  `cargo test discovery_tick_connect_backlog_must_coalesce_duplicate_remotes -- --nocapture`,
  `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`,
  `cargo test stale_pending_outgoing_peer_does_not_suppress_reconnect -- --nocapture`,
  `cargo test requester_connect_backlog_must_be_bounded -- --nocapture`, and
  `cargo test awaited_connect_must_error_while_same_peer_connect_is_pending -- --nocapture`,
  plus
  `cargo test connect_to_same_peer_id_at_different_address_must_not_report_success -- --nocapture`.
- ISSUE-154: fixed by `55b79e5` (`fix: continue partial kv repair
  responses`). `WorkingState::on_rpc_res` now accepts `FetchChanged` success
  only for an active pending `FetchChanged { from, count }`, validates returned
  versions against the pending range, rejects duplicates and zero-count pending
  repairs, and sends a follow-up request for the remaining range after a valid
  partial response. This also closes ISSUE-086 because unsolicited
  `FetchChanged` success responses are rejected before slot/version mutation or
  event emission. Verification:
  `cargo test working_state_must_reject_unsolicited_fetch_changed_success -- --nocapture`,
  `cargo test working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair -- --nocapture`
  and
  `cargo test working_state_must_continue_repair_after_partial_fetch_changed_success -- --nocapture`.
- ISSUE-155: fixed by adding per-role pubsub membership generations to
  publisher/subscriber join, leave, and heartbeat messages. Remote membership
  now stores the latest generation plus an active flag, keeping tombstones so
  stale leave messages cannot remove newer heartbeat-confirmed members and
  stale active messages cannot resurrect newer leaves. Local first-join and
  last-leave transitions increment their role generation before broadcasting.
  This changes pubsub control-message wire serialization. Verification:
  `cargo test stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat -- --nocapture`,
  `cargo test pubsub_remote_heartbeat_restore -- --nocapture`, and
  `cargo test pubsub_remote_single_pair_pub_first -- --nocapture`.
- ISSUE-163: fixed by `c979cea2cfdcbc49c1cb6fdafa212bd115524d3a`
  (`c979cea`, `fix: bound pubsub subscriber events`), with later
  active-membership support from `f9f3c81` (`fix: version pubsub membership
  updates`). Root cause: pubsub RPC fanout treated stale remote membership as a
  live destination even when every `send_unicast` failed immediately, inserted
  pending RPC state, and waited for timeout. Smallest fix: make `send_to`
  return success/failure, count only successful local/remote fanout in
  `GuestPublishRpc`, and return `PubsubRpcError::NoDestination` without
  pending RPC state when `delivered == 0`. Verification:
  `cargo test pubsub_rpc_must_return_no_destination_when_all_remote_sends_fail -- --nocapture`.
  This closes failed remote pubsub RPC send fanout only; local
  analogs/backpressure issues such as ISSUE-178 and ISSUE-124 remain separate.
- ISSUE-164: fixed by adding `P2pNetwork::pending_sync_tasks`, one guarded
  async route/discovery retry task per connection. Root cause was
  `process_tick` dropping maintenance sync on peer-control queue pressure after
  a failed `try_send`. Smallest fix: try immediate sync delivery first, replace
  any unfinished retry with the latest sync when queue pressure blocks the
  immediate send, remove completed retry handles on later ticks, abort older
  retry work after successful immediate delivery, and abort/remove stale
  retries on `PeerStopped`, `PeerDisconnected`, `PeerConnectError`, or missing
  aliases. Caveat: only one retry task is kept per connection, so the latest
  tick sync replaces older pending retry state. Verification:
  `cargo test tick_sync_must_not_be_dropped_when_peer_control_queue_is_full -- --nocapture`.
- ISSUE-156: fixed by writing the upstream relay setup `Ok(())` before opening
  the downstream stream. A closed upstream response side now fails that write
  and returns before `alias.open_stream` can deliver an orphan downstream
  service event. After a successful upstream acknowledgement, downstream open
  failure is observed as stream failure/EOF rather than a second setup
  response; preserving downstream setup errors would require a broader
  two-phase relay protocol. Verification:
  `cargo test relay_must_not_deliver_downstream_stream_after_upstream_setup_closes -- --nocapture`.
- ISSUE-157: fixed by replacing the awaited startup `PeerConnected` send with a
  nonblocking send plus one abortable retry task. A full main event queue no
  longer parks the authenticated peer task before `run_loop`; temporary
  backpressure still preserves eventual `PeerConnected` delivery, while an
  already-closed main loop keeps the existing immediate alias cleanup path. A
  pending connected retry is aborted during peer teardown before alias cleanup
  and `PeerDisconnected` reporting; if the retry already completed just before
  teardown, the main loop may still observe `PeerConnected` followed by
  `PeerDisconnected`. Verification:
  `cargo test peer_connected_must_not_block_authenticated_connection_run_loop_on_full_main_queue -- --nocapture`,
  `cargo test authenticated_peer_alias_must_be_cleaned_if_main_loop_closed_before_connected_event -- --nocapture`, and
  `cargo test peer_disconnected_must_not_block_alias_cleanup_on_full_main_queue -- --nocapture`.
- ISSUE-158: fixed by adding per-alias generations to alias lifecycle
  `NotifySet` and `NotifyDel` messages, plus bounded remote lifecycle state
  keyed by `(AliasId, PeerId)`. Stale lower-generation sets/deletes are ignored,
  accepted deletes leave an inactive tombstone, and higher-generation sets can
  still restore legitimate re-registrations. Local register/unregister
  broadcasts now carry the current alias generation. This changes alias
  control-message wire serialization, and tombstone protection is bounded by
  the lifecycle cache size. Verification:
  `cargo test stale_notify_set_must_not_resurrect_alias_after_newer_notify_del -- --nocapture`,
  `cargo test newer_notify_set_must_restore_alias_after_notify_del -- --nocapture`,
  `cargo test stale_not_found_must_not_evict_alias_cache_without_pending_check -- --nocapture`, and
  `cargo test shutdown_from_cached_hint_must_unblock_pending_find -- --nocapture`.
- ISSUE-159: fixed by `f62d6a6a`. Root cause was outbound peer setup inserting
  pending neighbour state before the main control stream existed, while
  `PeerConnection::new_connecting` could await QUIC connect or the initial
  `connection.open_bi()` indefinitely without emitting cleanup. The smallest
  fix wraps those setup awaits in `PEER_SETUP_TIMEOUT` and reports
  `MainEvent::PeerConnectError(conn_id, Some(to_peer), err)` on timeout. This
  closes pre-main-control-stream outbound setup hangs; later authenticated
  stream setup and per-service `open_bi` stalls remain separate issues.
  Verification:
  `cargo test outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open -- --nocapture`
  and
  `cargo test outbound_peer_setup_must_timeout_when_connect_request_write_stalls -- --nocapture`.
- ISSUE-145: fixed by validating `MainEvent::PeerData(conn, peer, ...)`
  against the router's live direct `(ConnectionId, PeerId)` binding before
  applying route sync or discovery advertisements. Stale or mismatched peer-data
  events now return `Continue` without mutating topology. Verification:
  `cargo test peer_data_must_validate_peer_matches_connection -- --nocapture`
  and
  `cargo test stale_peer_data_event_must_not_panic_without_direct_route -- --nocapture`.
- ISSUE-144: fixed by cleaning up the just-registered peer alias and connection
  teardown metrics when `run_connection` cannot deliver `PeerConnected` because
  the main loop is closed. The branch returns without emitting
  `PeerDisconnected`, since the main loop never accepted the connection.
  Verification:
  `cargo test authenticated_peer_alias_must_be_cleaned_if_main_loop_closed_before_connected_event -- --nocapture`.
- ISSUE-141: fixed by deriving the remaining requested `FetchChanged` range
  from the pending repair request and validating the response before mutation.
  Zero-count requests, duplicate returned versions, and versions outside the
  requested range are ignored without clearing the pending repair. Successful
  partial responses now emit a follow-up request for any versions still below
  the original inclusive target, unless `apply_pendings` already started another
  repair. Verification:
  `cargo test working_state_must_continue_repair_after_partial_fetch_changed_success -- --nocapture`.
- ISSUE-140: fixed by making replicated-KV remote state handlers return whether
  an RPC response or broadcast was accepted. `RemoteStore` now refreshes
  `last_active` only for accepted events, using one timestamp for dispatch,
  transition init, and liveness. Ignored working-state snapshots, unsolicited
  `FetchChanged` responses without a pending request, stale/equal version
  broadcasts, sync-full broadcasts, and destroy-state events no longer keep
  remotes alive. This also covers ISSUE-186. Verification:
  `cargo test ignored_rpc_response_must_not_refresh_remote_activity -- --nocapture`
  and
  `cargo test ignored_broadcast_must_not_refresh_remote_activity -- --nocapture`.
- ISSUE-137: fixed by completing and removing any pending alias find when the
  same alias is registered locally. The pending lookup now resolves as
  `AliasFoundLocation::Local`, the live-find gauge is decremented, and the
  existing `NotifySet` broadcast is preserved. Verification:
  `cargo test pending_find_must_prefer_late_local_registration_over_remote_found -- --nocapture`.
- ISSUE-138: fixed by using one effective snapshot version for bounded
  `FetchSnapshot` requests: the producer now caps requested future versions at
  the live version, filters the page with that value, and declares the same
  value in `RpcRes::FetchSnapshot`. Verification:
  `cargo test continuation_snapshot_response_must_preserve_requested_max_version -- --nocapture`.
- ISSUE-047: fixed by continuation `max_version` validation in
  `SyncFullState::on_rpc_res`. A continuation response must declare the same
  version that the pending `FetchSnapshot { max_version: Some(..) }` requested;
  mismatched pages are rejected before slot insertion, event emission, or
  state transition. Verification:
  `cargo test full_sync_must_reject_continuation_snapshot_version_mismatch -- --nocapture`.
- ISSUE-059: fixed by rejecting `FetchSnapshot(None, version)` terminal
  responses while a full-sync continuation request is pending. The pending
  request stays intact for timeout retry, so a partial snapshot cannot be
  silently completed with missing data. Verification:
  `cargo test full_sync_must_reject_none_continuation_after_partial_snapshot -- --nocapture`.
- ISSUE-136: fixed by moving `ctx.unregister_conn(&conn_id)` and teardown
  metric cleanup before the awaited
  `main_tx.send(MainEvent::PeerDisconnected(...))` lifecycle report in
  `run_connection`, so a full bounded main event queue cannot delay alias or
  metric cleanup. Verified with
  `cargo test peer_disconnected_must_not_block_alias_cleanup_on_full_main_queue -- --nocapture`.
- ISSUE-135: fixed by ignoring stale `PeerConnectError` events for connection
  ids that are already authenticated and connected in `NetworkNeighbours`.
  Pending or unknown connect errors still remove/no-op as before. Verified with
  `cargo test stale_peer_connect_error_must_not_remove_live_neighbour -- --nocapture`.
- ISSUE-134: fixed by refusing inbound QUIC `Incoming` attempts once 16
  unauthenticated inbound neighbours are already pending, and by bounding
  pre-authentication peer setup with `PEER_SETUP_TIMEOUT`. Excess raw clients
  are rejected before another pending peer task is inserted. Verified with
  `cargo test unauthenticated_inbound_connections_must_be_admission_bounded -- --nocapture`,
  `cargo test inbound_peer_setup_must_timeout_when_connect_response_write_stalls -- --nocapture`,
  `cargo test outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open -- --nocapture`,
  and
  `cargo test outbound_peer_setup_must_timeout_when_connect_request_write_stalls -- --nocapture`.
- ISSUE-133: fixed by `4997404` (`fix: deduplicate peer stopped forwarding`).
  `PeerConnectionInternal::on_msg` now uses
  `self.main_tx.try_send(MainEvent::PeerStopped(self.conn_id, peer_id))` for
  best-effort lifecycle reporting, so a full bounded main event queue cannot
  block the peer connection task while handling `PeerStopped`. Verified with
  `cargo test peer_stopped_must_not_block_connection_task_on_full_main_queue -- --nocapture`.
- ISSUE-131: fixed by capping full-sync snapshot pages at
  `MAX_SNAPSHOT_SLOTS_PER_PAGE` (`1024`) before applying slots in
  `SyncFullState::on_rpc_res`. Oversized pages are rejected without mutating
  local replicated state or emitting per-slot events. Fix commit: `d2dfbf7`
  (`fix: cap full sync snapshot pages`). Verified with
  `cargo test full_sync_snapshot_pages_must_be_bounded -- --nocapture`.
- ISSUE-122: fixed by bounding `PeerDiscovery::stopped` at 1,024 entries and
  evicting the oldest stopped tombstone after non-seed insertion, with peer-id
  tie-breaking for deterministic timestamp ties. Configured seeds still skip
  tombstone insertion and remain retryable. Verified with
  `cargo test graceful_stop_tombstones_must_be_bounded_for_unknown_peers -- --nocapture`,
  `cargo test graceful_stop_tombstones_evict_oldest_deterministically -- --nocapture`,
  `cargo test graceful_stop_tombstone -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-027: fixed by adding a 1,024-entry cap to replicated-KV
  `WorkingState::pendings` through a shared pending-change admission helper.
  Duplicate pending versions are ignored without overwrite, and cap overflow
  clears incremental repair state and falls back to full sync instead of
  silently evicting incremental data. Verification:
  `cargo test working_state_must_cap_pending_future_changes -- --nocapture`.
- ISSUE-181: fixed by validating local discovery advertise addresses in
  `PeerDiscovery::enable_local`. Addresses with an unspecified IP or port zero
  are now warned about, suppressed, and clear any previous local advertise
  address; valid addresses still produce local discovery sync entries. Verified
  with `cargo test local_sync_must_not_advertise -- --nocapture`.
- ISSUE-010: fixed by adding a 1,024-entry application-level cap to route and
  discovery syncs. Oversized inbound syncs are rejected before per-entry work,
  outbound sync creation is capped, and discovery logs only the sync count
  instead of the full payload. Verified with
  `cargo test should_reject_excessive_route_sync_entries -- --nocapture`,
  `cargo test discovery_sync_must_reject_excessive_entries -- --nocapture`,
  `cargo test create_sync_must_cap_outbound_route_entries -- --nocapture`, and
  `cargo test create_sync_for_must_cap_outbound_discovery_entries -- --nocapture`.
- ISSUE-024: fixed by using a 60 KB max-frame `BincodeCodec` for peer
  main-stream messages. Oversized `PeerMessage` frames are rejected before
  framing or inbound allocation. Verified with
  `cargo test peer_message_codec -- --nocapture`.
- ISSUE-174: fixed by the serialize-once validation in `write_object`. The
  actual encoded buffer is now checked against `MAX_SIZE` before any frame
  bytes are written, so non-deterministic serialization cannot bypass the cap.
  Verified with
  `cargo test write_object_must_recheck_actual_serialized_size -- --nocapture`.
- ISSUE-098: fixed by the concrete-buffer validation in `write_object`. Actual
  serialized payloads larger than `u16::MAX` are now rejected before writing the
  two-byte length prefix. Verified with
  `cargo test write_object_must_reject_payloads_larger_than_u16_length_prefix -- --nocapture`.
- ISSUE-097: fixed by making `write_object` serialize once and return
  serialization failures as `Err` before writing any frame bytes. Verified with
  `cargo test write_object_must_return_error_on_serialize_failure -- --nocapture`.
- ISSUE-169: fixed by the full stream setup timeout in `0a48ec7`, which also
  bounds `StreamConnectReq` writes. A peer that stops reading after accepting
  the stream-open bidirectional stream now makes `open_stream` return `Err(_)`
  within the setup timeout. Verified with
  `cargo test open_stream_must_timeout_when_connect_request_write_stalls -- --nocapture`.
- ISSUE-149: fixed by applying `OPEN_BI_TIMEOUT` to the whole outbound stream
  setup sequence, including `StreamConnectReq` write and `StreamConnectRes`
  read. A peer that withholds the setup response now makes `open_stream`
  return `Err(_)` instead of hanging. Verified with
  `cargo test open_stream_must_timeout_when_peer_withholds_connect_response -- --nocapture`.
- ISSUE-012: fixed by the same destination service queue reservation used for
  ISSUE-011. A full destination service queue now prevents a successful stream
  setup response, so the opener receives `Err(_)` instead of an orphan pipe.
  Verified with
  `cargo test open_stream_does_not_succeed_when_destination_service_queue_is_full -- --nocapture`.
- ISSUE-011: fixed by reserving destination service queue capacity before
  sending a successful stream setup response. Closed destination service
  receivers now produce an error response to the opener instead of a successful
  orphan pipe. Verified with
  `cargo test open_stream_fails_when_destination_service_receiver_is_closed -- --nocapture`.
- ISSUE-013: fixed by making `SharedCtx::open_stream` return
  `Err("unsupported open_stream to local node")` for `RouteAction::Local`
  instead of panicking. Verified with
  `cargo test open_stream_to_local_returns_error_not_panic -- --nocapture`.
- ISSUE-001: fixed by validating stopped-peer ownership before accepting or
  propagating graceful stop notifications. Inbound `PeerStopped(peer)` must
  name the authenticated direct peer for that connection, and
  `MainEvent::PeerStopped(conn, peer)` is ignored unless `conn` owns the direct
  route for `peer`. Verified with
  `cargo test forged_peer_stopped_must_not_remove_third_party_route -- --nocapture`,
  `cargo test forged_peer_stopped_must_not_be_forwarded_to_other_neighbours -- --nocapture`,
  and
  `cargo test peer_stopped_for_seed_must_not_remove_active_seed_route -- --nocapture`.
- ISSUE-018: fixed by treating the accepted bidirectional stream's
  authenticated ingress peer as the authoritative stream source. Decoded
  `StreamConnectReq.source` is normalized to that peer before local
  `P2pServiceEvent::Stream` delivery or relay `open_stream(...)`, preserving
  the existing wire/API shape. Verified with
  `cargo test stream_source_must_be_bound_to_authenticated_connection_peer -- --nocapture`,
  `cargo test relayed_stream_source_must_be_bound_to_previous_hop_peer -- --nocapture`,
  and `cargo fmt -- --check`. ISSUE-156 remains separate for relay setup
  cancellation.
- ISSUE-017: fixed by keying broadcast duplicate suppression on the trusted
  tuple `(authenticated/effective source, service_id, msg_id)` instead of the
  message id alone. Inbound broadcasts use the normalized authenticated source,
  and local broadcasts mark the local source plus service. Verified with
  `cargo test broadcast_dedup_must_include_authenticated_source_and_service -- --nocapture`
  and
  `cargo test broadcast_dedup_must_ignore_forged_claimed_source -- --nocapture`.
- ISSUE-020: fixed by binding pending pubsub RPCs to the responders that
  actually received each request. Pending publish and feedback RPC requests now
  record `expected_responders`, inbound remote answers are accepted only when
  `PeerSrc::Remote(from_peer)` is expected, and unexpected answers leave the
  request pending for the legitimate responder or timeout. Verified with
  `cargo test pubsub_publish_rpc_answer_must_be_bound_to_expected_responder -- --nocapture`,
  `cargo test pubsub_feedback_rpc_answer_must_be_bound_to_expected_responder -- --nocapture`,
  `cargo test pubsub_publish_rpc_remote -- --nocapture`,
  `cargo test pubsub_publish_rpc_local -- --nocapture`,
  `cargo test pubsub_feedback_rpc_remote -- --nocapture`, and
  `cargo test pubsub_feedback_rpc_local -- --nocapture`.
- ISSUE-115: fixed by carrying `SubscriberHandleId` on local
  `PublishRpcAnswer` control messages and recording which local subscriber
  handles actually received each publish RPC. Stale or unrelated subscriber
  requesters now leave the pending request open for the legitimate responder or
  timeout. Verified with
  `cargo test dropped_subscriber_requester_must_not_answer_publish_rpc -- --nocapture`.
- ISSUE-116: fixed by carrying `PublisherHandleId` on local
  `FeedbackRpcAnswer` control messages and recording which local publisher
  handles actually received each feedback RPC. Stale or unrelated publisher
  requesters now leave the pending request open for the legitimate responder or
  timeout. Verified with
  `cargo test dropped_publisher_requester_must_not_answer_feedback_rpc -- --nocapture`.
- ISSUE-039: fixed by authorizing ordinary pubsub member traffic against
  tracked channel membership. Inbound `Publish` now requires the sender to be
  an active remote publisher for the channel, and inbound `Feedback` requires
  the sender to be an active remote subscriber. Verified with
  `cargo test pubsub_publish_must_require_remote_publisher_membership -- --nocapture`,
  `cargo test pubsub_feedback_must_require_remote_subscriber_membership -- --nocapture`,
  `cargo test pubsub_remote_single_pair_pub_first -- --nocapture`, and
  `cargo test pubsub_remote_single_pair_sub_first -- --nocapture`. Guest
  traffic remains intentionally non-member traffic, and ISSUE-048 remains
  separate for RPC member traffic.
- ISSUE-048: fixed by applying the same channel-membership authorization to
  member RPC request frames. Inbound `PublishRpc` now requires the sender to be
  an active remote publisher for the channel, and inbound `FeedbackRpc`
  requires the sender to be an active remote subscriber. Verified with
  `cargo test pubsub_publish_rpc_must_require_remote_publisher_membership -- --nocapture`,
  `cargo test pubsub_feedback_rpc_must_require_remote_subscriber_membership -- --nocapture`,
  `cargo test pubsub_publish_must_require_remote_publisher_membership -- --nocapture`,
  `cargo test pubsub_feedback_must_require_remote_subscriber_membership -- --nocapture`,
  `cargo test pubsub_publish_rpc_remote -- --nocapture`, and
  `cargo test pubsub_feedback_rpc_remote -- --nocapture`. Guest RPC traffic
  remains intentionally non-member traffic, and RPC answers remain covered by
  ISSUE-020's expected-responder binding.
- ISSUE-014: fixed by binding inbound unicast sender identity to the
  authenticated immediate peer before delivery or forwarding. Message-body
  `source` can no longer spoof service-visible unicast sender ids; relayed
  unicasts now expose previous-hop identity until a future authenticated
  end-to-end origin protocol exists. Verified with
  `cargo test unicast_source_must_be_bound_to_authenticated_connection_peer -- --nocapture`,
  `cargo test forwarded_unicast_source_must_be_bound_to_ingress_peer -- --nocapture`,
  `cargo test send_relay -- --nocapture`, and `cargo fmt -- --check`.
- ISSUE-194: fixed by `InboundPeerBindings`, whose default is a strict static
  binding set. Inbound shared-key `ConnectReq.from` claims are now rejected
  unless the claimed peer id is explicitly bound to the observed remote address;
  legacy open-cluster admission requires the explicit
  `InboundPeerBindings::insecure_open_cluster()` opt-out. Verified with
  `cargo test inbound_handshake_must_reject_peer_claiming_third_party_id -- --nocapture`,
  `cargo test inbound_handshake_must_accept_bound_peer_claim -- --nocapture`,
  `cargo check --tests`, `cargo check --examples`, and
  `cargo fmt -- --check`.
- ISSUE-168: fixed by internal `PublisherHandleId` and `SubscriberHandleId`
  generation tokens. Pubsub local publisher/subscriber maps are now keyed by
  exact handle id, duplicate public local ids no longer replace another live
  handle, destroy removes only the exact handle, and publish/feedback controls
  require the exact live handle before acting. Verified with
  `cargo test duplicate_publisher_local_id_must_not_detach_live_handle -- --nocapture`,
  `cargo test duplicate_subscriber_local_id_must_not_detach_live_handle -- --nocapture`,
  and `cargo fmt -- --check`. Stale requesters, phantom channel creation,
  remote membership retention/removal, RPC timeout/backpressure, and
  unauthorized/stale remote membership remain separate issues.
- ISSUE-123: fixed by bounding local subscriber event streams with
  `LOCAL_SUBSCRIBER_EVENT_QUEUE_SIZE = 1024` and routing subscriber fanout
  through `try_send_subscriber_event`. Full or closed local subscriber queues no
  longer accumulate unbounded events, and local publish RPC fanout returns
  `NoDestination` without pending RPC state when every local subscriber delivery
  fails. Verified with
  `cargo test local_subscriber_event_backlog_must_be_bounded -- --nocapture`,
  `cargo test pubsub_rpc_must_return_no_destination_when_all_local_sends_fail -- --nocapture`,
  and
  `cargo test pubsub_rpc_must_return_no_destination_when_all_local_subscriber_queues_are_full -- --nocapture`.
- ISSUE-124: fixed by bounding local publisher event streams with
  `LOCAL_PUBLISHER_EVENT_QUEUE_SIZE = 1024` and routing publisher fanout
  through `try_send_publisher_event`. Full or closed local publisher queues no
  longer accumulate unbounded events, and local feedback RPC fanout returns
  `NoDestination` without pending RPC state when every local publisher delivery
  fails. Verified with
  `cargo test local_publisher_event_backlog_must_be_bounded -- --nocapture`,
  `cargo test feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full -- --nocapture`,
  `cargo test guest_feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-125: fixed by replacing the network requester control inbox with a
  bounded `NETWORK_CONTROL_QUEUE_SIZE = 1024` channel. Best-effort
  `try_connect` now drops on full or closed queues, awaited `connect` returns an
  immediate error when admission fails, and discovery tick retries process
  connects directly instead of self-enqueueing into the bounded inbox. Verified
  with `cargo test requester_connect_backlog_must_be_bounded -- --nocapture`,
  `cargo test requester_connect_returns_error_when_control_queue_full -- --nocapture`,
  `cargo test requester_connect_after_network_drop_returns_error_not_panic -- --nocapture`,
  `cargo test requester_try_connect_after_network_drop_must_not_panic -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-126: fixed by replacing the pubsub internal control inbox with bounded
  `PUBSUB_INTERNAL_CONTROL_QUEUE_SIZE = 1024` admission. Existing
  Result-returning guest, publisher, and subscriber request APIs now fail fast
  on full or closed queues instead of enqueueing without bound or awaiting
  response channels that were never admitted. Handle registration and drop
  paths use best-effort `try_send` with debug logging to preserve the current
  direct-handle API; dead-on-arrival handles when registration cannot be
  admitted remain tracked separately by ISSUE-058/API correctness follow-up.
  Verified with `cargo test pubsub_internal_control_backlog_must_be_bounded -- --nocapture`,
  `cargo test pubsub_guest_publish_returns_error_when_internal_queue_full -- --nocapture`,
  `cargo test pubsub_guest_publish_rpc_returns_error_when_internal_queue_full -- --nocapture`,
  `cargo test pubsub_guest_feedback_returns_error_when_internal_queue_full -- --nocapture`,
  `cargo test pubsub_guest_feedback_rpc_returns_error_when_internal_queue_full -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-127: fixed by replacing the alias internal control inbox with bounded
  `ALIAS_CONTROL_QUEUE_SIZE = 1024` admission. `find` now returns `None` when
  the queue is full or closed, and `register`, `shutdown`, and
  `AliasGuard::drop` use best-effort `try_send` with debug logging to preserve
  their current non-`Result` APIs. The remaining dead-on-arrival guard behavior
  under overload is an API-breaking follow-up. Verified with
  `cargo test alias_internal_control_backlog_must_be_bounded -- --nocapture`,
  `cargo test alias_find_returns_none_when_control_queue_full -- --nocapture`,
  `cargo test alias_shutdown_when_control_queue_full_must_not_panic -- --nocapture`,
  `cargo test alias_guard_drop_when_control_queue_full_must_not_panic -- --nocapture`,
  `cargo test alias_find_after_service_drop_returns_none_not_panic -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-128: fixed by `c83321c`, which makes `MetricsService::recv` return
  `Err(_)` when the underlying base service channel closes instead of panicking
  on `expect("should work")`. Verified with
  `cargo test metrics_recv_after_base_service_close_must_not_panic -- --nocapture`
  and `cargo fmt -- --check`.
- ISSUE-129: fixed by `c83321c`, which makes `VisualizationService::recv`
  return `Err(_)` when the underlying base service channel closes instead of
  panicking on `expect("should work")`. Verified with
  `cargo test visualization_recv_after_base_service_close_must_not_panic -- --nocapture`
  and `cargo fmt -- --check`.
- ISSUE-130: fixed by `e78c190` (`fix: return errors when alias channels
  close`), which makes `AliasService::run_loop` return `Err(_)` when the
  underlying base service channel closes instead of panicking on
  `expect("service channel should work")`. Verified with
  `cargo test alias_run_loop_after_base_service_close_must_not_panic -- --nocapture`.
- ISSUE-132: fixed by `e78c190` (`fix: return errors when alias channels
  close`), which makes `AliasService::run_loop` return `Err(_)` when the
  internal alias control channel closes instead of panicking on
  `expect("service channel should work")`. Verified with
  `cargo test alias_run_loop_after_control_channel_close_must_not_panic -- --nocapture`.
- ISSUE-204: fixed by `MetricsService::pending_scan_responses` plus bounded
  `requester.send_unicast(...)` response tasks, so duplicate metrics scans from
  one requester coalesce while a response is still backpressured. Verified with
  `cargo test metrics_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-202 remains separate for immediate dropped
  responses, ISSUE-203 remains separate for visualization response
  accumulation, ISSUE-200/201 remain separate for periodic scan-broadcast
  coalescing, and ISSUE-078/related issues remain separate for unauthorized
  metrics disclosure.
- ISSUE-203: fixed by `VisualizationService::pending_scan_responses` plus
  bounded `requester.send_unicast(...)` response tasks, so duplicate
  visualization scans from one peer coalesce while a response is still
  backpressured. Verified with
  `cargo test visualization_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-201 remains separate for periodic
  visualization scan-broadcast coalescing, ISSUE-204 remains separate for
  metrics response accumulation, and ISSUE-079/related issues remain separate
  for unauthorized topology disclosure.
- ISSUE-202: fixed by awaiting `requester.send_unicast(...)` with
  `SCAN_RESPONSE_SEND_TIMEOUT` and tracking `pending_scan_responses`, so a
  metrics scan response waits through transient peer-control backpressure
  instead of being dropped immediately. Verified with
  `cargo test metrics_scan_response_must_not_be_dropped_when_peer_control_queue_is_full -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-204 remains separate for duplicate
  response-task accumulation under sustained backpressure, and broad metrics
  correctness remains covered by other accepted issues.
- ISSUE-201: fixed by `VisualizationService::pending_scan_broadcast`, which
  keeps one active scan-broadcast task and skips/coalesces visualization
  collection ticks until that task completes. Verified with
  `cargo test visualization_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-203 remains separate for visualization
  scan-response backpressure, and unauthorized visualization scan/disclosure
  issues remain separate.
- ISSUE-200: fixed by `MetricsService::pending_scan_broadcast`, which keeps one
  active scan-broadcast task and skips/coalesces collector ticks until that task
  completes. Verified with
  `cargo test metrics_collector_must_not_spawn_duplicate_scans_when_previous_broadcast_is_backpressured -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-201 remains separate for visualization scan
  broadcasts, and ISSUE-202/203/204 remain separate for scan-response
  backpressure.
- ISSUE-198: fixed by changing `try_send_broadcast` to return
  `anyhow::Result<usize>` and report zero accepted peer-control queue
  admissions as an error. Verified with
  `cargo test try_send_broadcast_must_report_when_all_peer_queues_reject -- --nocapture`
  and `cargo test broadcast_direct -- --nocapture`. ISSUE-049 and ISSUE-199
  remain separate.
- ISSUE-199: fixed by changing `send_broadcast` to return
  `anyhow::Result<usize>` and report zero accepted awaited peer-control
  admissions as an error. Verified with
  `cargo test send_broadcast_must_report_when_all_peer_channels_are_closed -- --nocapture`
  and `cargo test broadcast_relay -- --nocapture`. ISSUE-049 and ISSUE-198
  remain separate.
- ISSUE-197: fixed by routing ordinary unicast relay decisions through
  `DropIngressLoop` when the selected next hop is the ingress connection. The
  relay path now logs and drops instead of forwarding back to the sender.
  Verified with
  `cargo test unicast_relay_must_not_forward_back_to_ingress_peer -- --nocapture`
  and `cargo test send_relay -- --nocapture`. Stream relay loop handling
  remains separate under ISSUE-180.
- ISSUE-196: fixed by bounding `ReplicatedKvStore::outs` at `1024` pending
  outbound events with drop-oldest admission. This preserves the public
  `set`/`del` API and contains overload at the outer service queue, retaining
  the newest work under sustained local mutation bursts. Verified with
  `cargo test replicated_kv_local_outbound_event_queue_must_be_bounded -- --nocapture`.
- ISSUE-195: fixed by leaving monotonic connection counters untouched during
  connection teardown. `emit_connection_teardown_metrics` now only decrements
  `P2P_LIVE_CONNECTION_COUNT` and resets `P2P_CONNECTION_RTT` with
  `gauge!(...).set(0)`, so uptime, byte, loss, packet, and congestion counters
  no longer receive teardown `absolute(0)` samples. Verified with
  `cargo test connection_teardown_must_not_reset_monotonic_counters -- --nocapture`,
  `cargo test connection_teardown_must_not_emit_rtt_as_counter -- --nocapture`,
  and `cargo fmt -- --check`. ISSUE-193 remains the separate RTT metric-kind
  collision issue.
- ISSUE-193: fixed by keeping connection teardown on the RTT gauge metric kind.
  `emit_connection_teardown_metrics` now decrements live connection count and
  resets `P2P_CONNECTION_RTT` with `gauge!(...).set(0)` only, so RTT is not
  emitted as both a gauge and counter. Verified with
  `cargo test connection_teardown_must_not_emit_rtt_as_counter -- --nocapture`,
  `cargo test connection_teardown_must_not_reset_monotonic_counters -- --nocapture`,
  and `cargo fmt -- --check`. ISSUE-195 is tracked separately for monotonic
  counter reset behavior.
- ISSUE-189: fixed by the existing inbound `run_connection` self-identity
  guard, which rejects `ConnectReq` messages with `req.from == local_id` before
  emitting a successful handshake response or installing alias/neighbour state.
  Verified with
  `cargo test inbound_handshake_must_reject_peer_claiming_local_id -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-194 remains open for arbitrary third-party
  peer-id claims.
- ISSUE-191: fixed by updating the README getting-started snippet and its
  compile-mirror example to use a numeric `PeerId`, propagate
  `P2pNetwork::new(...).await?`, bind a mutable network, and create the service
  after successful construction. Verified with
  `cargo test readme_getting_started_snippet_must_compile -- --nocapture`,
  `cargo check --example readme_getting_started`, and
  `cargo fmt -- --check`. Reviewer `James the 8th` accepted.
- ISSUE-002: fixed by bounded future-skew validation in
  `SharedKeyHandshake::validate_handshake`. Handshake timestamps more than
  1 second ahead of verifier time are rejected with checked
  `current_ts + HANDSHAKE_MAX_FUTURE_SKEW` arithmetic, while the existing small
  clock-skew compatibility case remains valid. Verified with
  `cargo test rejects_arbitrarily_future_request_timestamp -- --nocapture`,
  `cargo test test_handshake_timeout -- --nocapture`,
  `cargo test rejects_overflowing_request_timestamp_without_panic -- --nocapture`,
  `cargo test test_handshake_flow -- --nocapture`, and
  `cargo fmt -- --check`. Reviewer `Pauli the 8th` accepted.
- ISSUE-003: fixed with stable route selection hysteresis in
  `PeerMemory::select_best`, direct-path priority over relayed candidates, and
  widened route score math. Verified with
  `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`,
  `cargo test should_keep_existing_best_path_on_equal_score -- --nocapture`,
  `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`,
  `cargo test should_not_overflow_score_during_best_path_selection -- --nocapture`,
  `cargo test should_remove_relay_path_after_disconnect -- --nocapture`, and
  `cargo test should_remove_stopped_peer_path -- --nocapture`. Reviewer
  `Chandrasekhar the 7th` approved the revised patch.
- ISSUE-160: fixed by `cfc8e57` as part of the ISSUE-003 route-stability work.
  Root cause was that `PeerMemory::select_best` treated authenticated direct
  paths as ordinary scored candidates, so a lower-cost relayed advertisement
  could replace the direct route. The smallest shipped fix gives direct paths
  (`relay_hops == 0`) priority over relayed candidates whenever a direct path
  exists, using directness by path metric rather than a separate ownership
  lookup. Verified with
  `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`.
- ISSUE-161: fixed by making discovery's live graceful-stop tombstones visible
  to route sync application. Root cause was that `PeerStopped` removed the
  route once and recorded only a discovery tombstone, while third-party
  `RouterTableSync` rows were applied without consulting that tombstone. The
  smallest fix applies discovery sync first, then filters route-sync
  destinations through `PeerDiscovery::is_stopped`, so stale relay routes stay
  suppressed during the tombstone window while fresh restart advertisements can
  clear the tombstone before route application. Verified with
  `cargo test stopped_peer_route_must_not_be_resurrected_by_third_party_sync -- --nocapture`,
  `cargo test graceful_stop_tombstone -- --nocapture`, and
  `cargo test router -- --nocapture`.
- ISSUE-162: fixed by fanning accepted peer-disconnect lifecycle events to
  registered services and teaching replicated KV to destroy the matching remote
  store immediately. Root cause was that graceful `PeerStopped` cleanup updated
  discovery and routing only; service-owned remote KV state kept waiting for
  the 10-second idle timeout. The smallest fix adds
  `P2pServiceEvent::PeerDisconnected(peer)`, sends it nonblocking after
  accepted peer stop/disconnect handling, and reuses `RemoteStore::destroy()`
  to emit `KvEvent::Del(Some(peer), key)` rows. Full or closed service queues
  still fall back to timeout cleanup. Verified with
  `cargo test replicated_kv_must_delete_remote_data_when_peer_gracefully_stops -- --nocapture`,
  `cargo test single_node -- --nocapture`, and
  `cargo test peer_stopped_must_emit_public_disconnect_event -- --nocapture`.
- ISSUE-004: fixed by the ISSUE-170 ownership-validation follow-up `87cf6ce`.
  `MainEvent::PeerStopped(conn, peer)` is ignored unless `conn` is the direct
  authenticated connection for `peer`, so a third-party stop cannot delete a
  configured seed route. Verified with
  `cargo test peer_stopped_for_seed_must_not_remove_active_seed_route -- --nocapture`,
  `cargo test forged_peer_stopped_must_not_remove_third_party_route -- --nocapture`,
  `cargo test peer_stopped_must_remove_stopped_neighbour_immediately -- --nocapture`,
  and `cargo test peer_stopped -- --nocapture`.
- ISSUE-005: fixed by rejecting advertised rows for the enabled local peer id
  in `PeerDiscovery::apply_sync`, after timestamp validation and before
  tombstone handling. Verified with
  `cargo test apply_sync_rejects_local_peer_advertisement -- --nocapture`,
  `cargo test apply_sync_rejects_overflowing_future_timestamp -- --nocapture`,
  `cargo test graceful_stop_tombstone_ignores_stale_non_seed_advertise -- --nocapture`,
  `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`,
  `cargo test apply_sync_must_not_overwrite_newer_discovery_with_stale_advertisement -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Locke the 7th` approved. The
  configured-seed local-id dial-candidate failure remains separate.
- ISSUE-006: fixed by filtering local-peer routes at router sync ingress and
  egress. `RouterTable::apply_sync` now drops incoming rows for `self.peer_id`
  before creating route memory, and `create_sync` defensively excludes any
  stored local-peer route from advertisements. Verified with
  `cargo test should_not_store_or_advertise_route_to_local_peer -- --nocapture`,
  `cargo test create_correct_direct_sync -- --nocapture`,
  `cargo test apply_correct_direct_sync -- --nocapture`,
  `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`,
  `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Arendt the 7th` approved.
- ISSUE-007: fixed by rejecting composed over-`MAX_HOPS` route metrics during
  router sync ingestion. `RouterTable::apply_sync` now adds the direct
  connection metric before storage and drops rows whose resulting
  `relay_hops > MAX_HOPS`, causing existing paths for that connection to be
  removed through the normal deletion path. Verified with
  `cargo test should_reject_over_max_hops_for_forwarding -- --nocapture`,
  `cargo test dont_create_sync_over_max_hops -- --nocapture`,
  `cargo test should_remove_relay_path_after_disconnect -- --nocapture`,
  `cargo test should_not_store_or_advertise_route_to_local_peer -- --nocapture`,
  `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`,
  `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Schrodinger the 7th` approved.
- ISSUE-008: fixed by split-horizon filtering in `RouterTable::create_sync`.
  Routes whose current best next-hop connection reaches the sync destination
  are not advertised back to that destination, while direct routes remain
  advertised to other peers. Verified with
  `cargo test should_not_advertise_route_back_to_next_hop -- --nocapture`,
  `cargo test apply_correct_direct_sync -- --nocapture`,
  `cargo test create_correct_direct_sync -- --nocapture`,
  `cargo test should_not_store_or_advertise_route_to_local_peer -- --nocapture`,
  `cargo test should_reject_over_max_hops_for_forwarding -- --nocapture`,
  `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`,
  `cargo test direct_peer_route_must_not_be_replaced_by_relayed_path -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Harvey the 7th` approved.
- ISSUE-009: fixed by checked discovery timestamp liveness. Discovery timeout
  cleanup and sync ingestion now use `checked_add(TIMEOUT_AFTER)` plus
  `timestamp <= now_ms`, reject stale/future/overflowing advertisements, and
  allow fresh restart advertisements newer than live stopped tombstones.
  Verified with
  `cargo test apply_sync_rejects_overflowing_future_timestamp -- --nocapture`,
  `cargo test apply_sync_timeout -- --nocapture`,
  `cargo test clear_timeout -- --nocapture`,
  `cargo test graceful_stop_tombstone_ignores_stale_non_seed_advertise -- --nocapture`,
  `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`,
  `cargo test non_seed_discovered_peer_ages_out_but_seed_remains_retryable -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Dewey the 7th` approved. Separate
  accepted discovery failures for local-peer advertisements and configured-seed
  overrides still reproduce and remain outside this fix.
- ISSUE-021: fixed by checked handshake expiry arithmetic in
  `SharedKeyHandshake::validate_handshake`. Peer-controlled timestamps now use
  `checked_add(HANDSHAKE_TIMEOUT)` and overflow returns `Err` before timeout
  comparison, avoiding debug panics and release wrapping. Verified with
  `cargo test rejects_overflowing_request_timestamp_without_panic -- --nocapture`,
  `cargo test test_handshake_timeout -- --nocapture`,
  `cargo test test_handshake_flow -- --nocapture`, and
  `cargo fmt -- --check`. Reviewer `Singer the 8th` accepted.
- ISSUE-146, ISSUE-176, and ISSUE-207: fixed by adding a signed random nonce
  and scoped replay cache to `SharedKeyHandshake`. Accepted token hashes are
  recorded only after full validation, expired entries are pruned, duplicate
  request or response tokens are rejected within their verified
  `(from, to, is_initiator)` scope while present, and oldest-token eviction
  keeps the replay cache globally bounded across many unique scopes without
  denying unrelated fresh handshakes. Inbound setup performs cheap local
  admission checks before recording request tokens. Verified with
  `cargo test request_handshake_tokens_must_not_be_replayable -- --nocapture`,
  `cargo test response_handshake_tokens_must_not_be_replayable -- --nocapture`,
  `cargo test replay_cache_exhaustion_must_not_reject_fresh_valid_handshake -- --nocapture`,
  `cargo test replay_cache_many_scopes_must_remain_bounded -- --nocapture`,
  `cargo test test_handshake_flow -- --nocapture`,
  `cargo test test_invalid_handshake -- --nocapture`,
  `cargo test test_handshake_timeout -- --nocapture`,
  `cargo test rejects_arbitrarily_future_request_timestamp -- --nocapture`,
  `cargo test rejects_overflowing_request_timestamp_without_panic -- --nocapture`,
  `cargo test --lib tests::cross_nodes::send_direct -- --nocapture`, and
  `cargo fmt -- --check`. Reviewer `Copernicus the 8th` accepted. This changes
  the handshake wire payload; compatibility/versioning remains separate.
- ISSUE-033: fixed by checked route metric composition in
  `RouterTable::apply_sync`. Peer-advertised metrics are combined with the
  direct-link metric through `PathMetric::checked_add`, and overflowing hop or
  RTT rows are rejected before they can update route memory or active paths.
  Verified with
  `cargo test should_reject_overflowing_route_sync_metric_without_panic -- --nocapture`,
  `cargo test should_not_overflow_score_during_best_path_selection -- --nocapture`,
  `cargo test should_reject_over_max_hops_for_forwarding -- --nocapture`,
  `cargo test apply_correct_direct_sync -- --nocapture`,
  `cargo test create_correct_direct_sync -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer accepted.
- ISSUE-034: fixed by full-sync snapshot validation in
  `SyncFullState::on_rpc_res`. Snapshot pages now reject any
  `slot.version > version` before inserting into `ctx.slots`, emitting
  `KvEvent::Set`, or transitioning to `WorkingState(version)`. Verified with
  `cargo test full_sync_must_reject_snapshot_slot_newer_than_declared_version -- --nocapture`.
- ISSUE-055: fixed by rejecting discovery sync rows whose peer id is already
  configured as a seed, preserving static seed addresses as authoritative.
  Verified with
  `cargo test apply_sync_must_not_duplicate_or_override_configured_seed -- --nocapture`,
  `cargo test apply_sync_rejects_local_peer_advertisement -- --nocapture`,
  `cargo test apply_sync_rejects_overflowing_future_timestamp -- --nocapture`,
  `cargo test graceful_stop_tombstone_ignores_stale_non_seed_advertise -- --nocapture`,
  `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`,
  `cargo test non_seed_discovered_peer_ages_out_but_seed_remains_retryable -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Singer the 7th` approved. The
  configured-seed-with-local-peer-id dial-candidate failure remains separate.
- ISSUE-103: fixed by filtering configured seeds whose peer id equals the
  enabled local peer id in `PeerDiscovery::remotes()`, while preserving normal
  configured seed retry behavior. Verified with
  `cargo test configured_seed_with_local_peer_id_must_not_be_dial_candidate -- --nocapture`,
  `cargo test non_seed_discovered_peer_ages_out_but_seed_remains_retryable -- --nocapture`,
  `cargo test graceful_stop_tombstone_keeps_seed_retry_address -- --nocapture`,
  `cargo test apply_sync_rejects_local_peer_advertisement -- --nocapture`,
  `cargo test apply_sync_must_not_duplicate_or_override_configured_seed -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer `Leibniz the 7th` approved.
- ISSUE-190: fixed by rejecting route syncs with duplicate non-local
  destination peer ids before route memory or path state is updated. This
  removes order-dependent last-row-wins behavior for malformed route-sync
  packets. Verified with
  `cargo test route_sync_must_reject_duplicate_peer_entries -- --nocapture`,
  `cargo test apply_correct_direct_sync -- --nocapture`,
  `cargo test create_correct_direct_sync -- --nocapture`,
  `cargo test should_reject_overflowing_route_sync_metric_without_panic -- --nocapture`,
  `cargo test should_reject_over_max_hops_for_forwarding -- --nocapture`,
  `cargo test should_not_store_or_advertise_route_to_local_peer -- --nocapture`,
  and `cargo fmt -- --check`. Reviewer accepted.
- ISSUE-192: fixed by the ISSUE-009 discovery freshness handling.
  `PeerDiscovery::apply_sync` ignores rows whose timestamp is not newer than
  existing remote discovery state, so duplicate discovery rows in one sync
  resolve to the newest timestamp instead of the last row. Verified with
  `cargo test discovery_sync_must_reject_duplicate_peer_entries -- --nocapture`.
- ISSUE-053: fixed by `648cfd0` with range-checked service table indexing.
  Verified with `cargo test ctx::tests -- --nocapture` and seed-340
  invalid-service fuzz passing without `src/ctx.rs` panics.
- ISSUE-091: fixed by the same range-checked service table lookup. Root cause
  was the inbound stream accept path trusting wire-decoded
  `StreamConnectReq.service` before looking up the fixed 256-slot service
  table. `SharedCtxInternal::get_service` now returns `None` for
  `P2pServiceId >= 256`, so invalid stream opens receive the normal
  service-not-found error and the accept task survives. Verification:
  `cargo test inbound_out_of_range_stream_service_id_must_not_panic_accept_task -- --nocapture`
  and
  `cargo test get_service_must_reject_out_of_range_id_without_panicking -- --nocapture`.
- ISSUE-063: fixed by `2cbf096` with stale router sync ignored after direct
  disconnect. Verified with
  `cargo test router::tests::should_ignore_stale_sync_after_direct_disconnect -- --nocapture`;
  the seed-341 reproducer no longer reports the `should have direct metric`
  signature.
- ISSUE-139: fixed by `15b788c` with closed-main tolerant connect-error
  reporting. Verified with focused peer tests and seed-341 churn fuzz no
  longer reporting `should send to main`.
- ISSUE-170: fixed by `4997404` plus corrective follow-up `87cf6ce`.
  `4997404` added per-context stopped-peer dedupe and nonblocking main-loop
  reporting; `87cf6ce` validates stopped-peer ownership against the direct
  authenticated route, rejects forged stops before dedupe/forwarding, emits
  public disconnects for legitimate graceful stops, removes stopped neighbours,
  and bounds removed-connection tombstones. Verified with
  `cargo test peer_stopped -- --nocapture`, seed-341 churn fuzz showing zero
  forwarded-stop/capacity/channel-closed storm markers, and seed-340 broad fuzz
  passing without service-id, stale-sync, or shutdown-send panic signatures.

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
- ISSUE-194, score 88: inbound handshake accepted arbitrary third-party peer-id
  claims; strict static inbound peer bindings are now the default, with legacy
  open-cluster admission behind an explicit insecure opt-out. Reviewer:
  Confucius the 3rd.
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
- ISSUE-205, score 62: pubsub membership generations reset on restart, so an
  old inactive tombstone can suppress a fresh publisher/subscriber join from
  the same peer id. Reviewer: Noether the 5th.
- ISSUE-206, score 60: alias lifecycle generations reset on restart, so an
  inactive alias tombstone can suppress a fresh `NotifySet` from the same peer
  id. Reviewer: Curie the 6th.
- ISSUE-207, score 58: shared-key replay cache exhaustion can reject unrelated
  fresh valid handshakes. Reviewer: Turing the 7th.

## Next Candidate To Validate

- Continue RED-team review around directed service replies, pubsub/alias
  response paths, and high-load backpressure. Randomized node-action churn
  fuzzing has already started and should continue from the steady-valid passing
  baseline when five consecutive no-new cycles accumulate again.

## Recent Fuzz Evidence

- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=222 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Dirac the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The single `channel closed` send error and `closed by peer` log were
  reviewed as teardown fallout after the service task panic, and the fuzz
  harness assertion only reported that background panic. No ISSUE-063,
  ISSUE-139, or ISSUE-170 evidence was present, and no new invariant appeared.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=221 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063, ISSUE-139, and ISSUE-170.
  Reviewer `Godel the 6th` confirmed the three `src/router.rs:76` panics with
  `should have direct metric with apply_sync` are the existing stale-sync root
  cause; the `src/peer.rs:92:104` `should send to main: SendError { .. }`
  panic is the existing shutdown reporting race; and the 8,478 forwarded-stop
  alias errors with 7,423 no-capacity and 1,113 channel-closed logs plus 22
  `broadcast data over peer alias` failures are the existing PeerStopped storm
  fallout. The connection lost/internal endpoint logs were teardown fallout.
  No ISSUE-053 evidence was present, and no new invariant appeared. The
  smallest fix proposals remain unchanged: guard/drop stale sync when the
  direct metric is gone and invalidate queued sync state on direct-route
  removal; replace peer main-channel `expect` calls with normal teardown
  handling when the main receiver is closed; add dedupe/TTL/tombstone
  suppression for forwarded `PeerStopped` and rate-limit repeated `try_send`
  failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=220 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Mencius the 6th` confirmed the four `src/ctx.rs:34` panics with index
  `256` into len `256` are the existing unchecked inbound service-id root
  cause. The channel-closed, closed-by-peer, connection-lost, and connection
  closed logs were reviewed as teardown fallout after the service task panics,
  and the fuzz harness assertion only reported those background panics. No
  ISSUE-063, ISSUE-139, or ISSUE-170 evidence was present, and no new
  invariant appeared. The smallest fix proposal remains unchanged: validate
  decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=219 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Noether the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the 13,993 forwarded-stop alias errors with 12,094 no-capacity
  and 1,990 channel-closed logs are the existing PeerStopped forwarding storm.
  The three connection-lost logs were reviewed as teardown fallout. No
  ISSUE-053 or ISSUE-139 evidence was present, and no new invariant appeared.
  The smallest fix proposals remain unchanged: guard/drop stale sync when the
  direct metric is gone and invalidate queued sync state on direct-route
  removal; add dedupe/TTL/tombstone suppression for forwarded `PeerStopped`
  and rate-limit repeated `try_send` failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=218 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Euclid the 6th` confirmed the three `src/router.rs:76` panics with
  `should have direct metric with apply_sync` are the existing stale-sync root
  cause. The single `forward peer stopped over peer alias got error no
  available capacity` line was reviewed as too small to classify as ISSUE-170
  storm/backpressure evidence by itself. No ISSUE-053 or ISSUE-139 evidence
  was present, and no new invariant appeared. The smallest fix proposal remains
  unchanged: guard/drop stale sync when the direct metric is gone and
  invalidate queued sync state on direct-route removal. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=217 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Heisenberg the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause. The fuzz harness assertion only reported that background-task panic.
  No invalid service ID, send-to-main shutdown race, PeerStopped storm, alias
  backpressure, transport teardown, path-not-found evidence, or new invariant
  appeared. The smallest fix proposal remains unchanged: guard/drop stale sync
  when the direct metric is gone and invalidate queued sync state on
  direct-route removal. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=216 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Dewey the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The single `channel closed` send error and `closed by peer` log were
  reviewed as teardown fallout after the background service panic, and the
  fuzz harness assertion only reported that background-task panic. No
  ISSUE-063, ISSUE-139, or ISSUE-170 evidence was present, and no new
  invariant appeared. The smallest fix proposal remains unchanged: validate
  decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=215 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `McClintock the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause. The fuzz harness assertion only reported that background-task panic.
  No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present, and no new
  invariant appeared. The smallest fix proposal remains unchanged: guard/drop
  stale sync when the direct metric is gone and invalidate queued sync state on
  direct-route removal. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=214 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Ptolemy the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the 12,627 forwarded-stop alias errors with 12,506 no-capacity
  and 172 channel-closed logs are the existing PeerStopped forwarding storm.
  The 11 `broadcast data over peer alias` logs were reviewed as storm fallout,
  and the endpoint internal-error log was teardown fallout. No ISSUE-053 or
  ISSUE-139 evidence was present, and no new invariant appeared. The smallest
  fix proposals remain unchanged: guard/drop stale sync when the direct metric
  is gone and invalidate queued sync state on direct-route removal; add
  dedupe/TTL/tombstone suppression for forwarded `PeerStopped` and rate-limit
  repeated `try_send` failures. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=213 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Halley the 6th` confirmed the three `src/router.rs:76` panics with
  `should have direct metric with apply_sync` are the existing stale-sync root
  cause, and the 33,435 forwarded-stop alias errors with 29,915 no-capacity
  and 3,705 channel-closed logs are the existing PeerStopped forwarding storm.
  The fuzz harness assertion only reported background task failure. No
  ISSUE-053 or ISSUE-139 evidence was present, and no new invariant appeared.
  The smallest fix proposals remain unchanged: guard/drop stale sync when the
  direct metric is gone and invalidate queued sync state on direct-route
  removal; add dedupe/TTL/tombstone suppression for forwarded `PeerStopped`
  and rate-limit repeated `try_send` failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=212 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Tesla the 6th` confirmed the six `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause. The
  six `channel closed` send errors, five `closed by peer` logs, and one
  `connection lost` log were reviewed as teardown fallout after the background
  panics, and the fuzz harness assertion only reported those background-task
  panics. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was present, and no
  new invariant appeared. The smallest fix proposal remains unchanged:
  validate decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=211 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Chandrasekhar the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the 10,725 forwarded-stop alias errors with 9,411 no-capacity and
  1,421 channel-closed logs are the existing PeerStopped forwarding storm. The
  32 `broadcast data over peer alias` logs were reviewed as storm fallout, and
  the endpoint internal-error logs were teardown fallout after the
  panic/assertion. No ISSUE-053 or ISSUE-139 evidence was present, and no new
  invariant appeared. The smallest fix proposals remain unchanged: guard/drop
  stale sync when the direct metric is gone and invalidate queued sync state on
  direct-route removal; add dedupe/TTL/tombstone suppression for forwarded
  `PeerStopped` and rate-limit repeated `try_send` failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=210 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Bernoulli the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The two `channel closed` send errors and one `closed by peer` log
  were reviewed as teardown fallout after the background panic, and the fuzz
  harness assertion only reported that background-task panic. No ISSUE-063,
  ISSUE-139, or ISSUE-170 evidence was present, and no new invariant appeared.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=209 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Volta the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the 9,175 forwarded-stop alias errors with 8,523 no-capacity and
  826 channel-closed logs are the existing PeerStopped forwarding storm. The
  38 `broadcast data over peer alias` logs and one
  `answer open_bi got error internal channel error` log were reviewed as
  storm/lifecycle fallout; the connection lost/internal endpoint logs were
  teardown noise. No ISSUE-053 or ISSUE-139 evidence was present, and no new
  invariant appeared. The smallest fix proposals remain unchanged: guard/drop
  stale sync when the direct metric is gone and invalidate queued sync state on
  direct-route removal; add dedupe/TTL/tombstone suppression for forwarded
  `PeerStopped` and rate-limit repeated `try_send` failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=208 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. The forked RED-team
  reviewer confirmed the single `src/ctx.rs:34` panic with index `256` into
  len `256` is the existing unchecked inbound service-id root cause. The two
  `channel closed` send errors and one `closed by peer` log were reviewed as
  teardown fallout after the background panic, and the fuzz harness assertion
  only reported that background-task panic. No ISSUE-063, ISSUE-139, or
  ISSUE-170 evidence was present, and no new invariant appeared. The smallest
  fix proposal remains unchanged: validate decoded `P2pServiceId` before
  indexing the fixed service table and reject/drop out-of-bounds remote ids.
  No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=207 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `Mill the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the `src/peer.rs:92:104` `should send to main: SendError { .. }`
  panic is the existing shutdown reporting race. The endpoint internal-error
  log was reviewed as fallout from the router panic or dropped endpoint driver.
  No ISSUE-053 or ISSUE-170 evidence was present, and no new invariant
  appeared. The smallest fix proposals remain unchanged: guard/drop stale sync
  when the direct metric is gone and invalidate queued sync state on
  direct-route removal; replace peer main-channel `expect` calls with normal
  teardown handling when the main receiver is closed. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=206 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Huygens the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The single `channel closed` send error and `connection lost` log were
  reviewed as teardown fallout after the background panic, and the fuzz harness
  assertion only reported that background-task panic. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present, and no new invariant appeared. The
  smallest fix proposal remains unchanged: validate decoded `P2pServiceId`
  before indexing the fixed service table and reject/drop out-of-bounds remote
  ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=205 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Leibniz the 6th` confirmed the three `src/router.rs:76` panics with
  `should have direct metric with apply_sync` are the existing stale-sync root
  cause, and the 6,627 forwarded-stop alias errors with 6,102 no-capacity and
  534 channel-closed logs are the existing PeerStopped forwarding storm. The
  two connection-lost logs were reviewed as network teardown fallout. No
  ISSUE-053 or ISSUE-139 evidence was present, and no new invariant appeared.
  The smallest fix proposals remain unchanged: guard/drop stale sync when the
  direct metric is gone and invalidate queued sync state on direct-route
  removal; add dedupe/TTL/tombstone suppression for forwarded `PeerStopped`
  and rate-limit repeated `try_send` failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=204 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Mendel the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The single `channel closed` send error and `closed by peer` log were
  reviewed as fallout after the background panic/connection teardown, and the
  fuzz harness assertion only reported that background-task panic. No
  ISSUE-063, ISSUE-139, or ISSUE-170 evidence was present, and no new
  invariant appeared. The smallest fix proposal remains unchanged: validate
  decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=203 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Planck the 6th` confirmed the single `src/router.rs:76` panic with
  `should have direct metric with apply_sync` is the existing stale-sync root
  cause, and the 7,647 forwarded-stop alias errors with 7,184 no-capacity and
  484 channel-closed logs are the existing PeerStopped forwarding storm. The
  three connection lost/closed/internal endpoint logs were reviewed as network
  teardown fallout. No ISSUE-053 or ISSUE-139 evidence was present, and no new
  invariant appeared. The smallest fix proposals remain unchanged: guard/drop
  stale sync when the direct metric is gone and invalidate queued sync state on
  direct-route removal; add dedupe/TTL/tombstone suppression for forwarded
  `PeerStopped` and rate-limit repeated `try_send` failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=202 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Hegel the 6th` confirmed the single `src/ctx.rs:34` panic with index
  `256` into len `256` is the existing unchecked inbound service-id root
  cause. The single `channel closed` send error and `closed by peer` log were
  reviewed as teardown/lifecycle fallout, and the fuzz harness assertion only
  reported the background-task panic. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present, and no new invariant appeared beyond the invalid
  service-id panic. The smallest fix proposal remains unchanged: validate
  decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=201 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Hilbert the 6th` confirmed the five `src/ctx.rs:34` panics with index
  `256` into len `256` are the existing unchecked inbound service-id root
  cause. The `channel closed` send errors and closed/lost connection logs were
  reviewed as teardown/lifecycle noise, and the fuzz harness assertion only
  reported the background-task panic. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present, and no new invariant appeared beyond the invalid
  service-id panic. The smallest fix proposal remains unchanged: validate
  decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=200 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Kuhn the 6th` confirmed the `src/router.rs:76` direct-metric panic is the
  existing stale `PeerData::Sync` root cause, and the 107,853 forwarded-stop
  logs with 107,461 no-capacity and 1,801 channel-closed logs are the existing
  peer-alias stop-forwarding/backpressure storm. The 175 `broadcast data over
  peer alias` logs and two `answer open_bi got error internal channel error`
  logs were reviewed as storm-context fallout under ISSUE-170. No ISSUE-053
  or ISSUE-139 evidence was present, and no new invariant appeared beyond
  stale sync panic plus forwarded-stop storm. Smallest fixes remain unchanged:
  guard/drop stale sync without a direct metric, clear queued sync state when
  direct routes are removed, and add dedupe/TTL/tombstone suppression plus
  rate-limited repeated `try_send` errors for forwarded `PeerStopped`. No new
  issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=199 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-063. Reviewer
  `Lagrange the 6th` confirmed the `src/ctx.rs:34` panic is the existing
  unchecked inbound service-id root cause, and the `src/router.rs:76`
  direct-metric panic is the existing stale `PeerData::Sync` root cause. The
  connection-lost and channel-closed logs were reviewed as lifecycle/teardown
  noise, and the fuzz harness panic only reported background task failure. No
  ISSUE-139 or ISSUE-170 evidence was present, and no new invariant appeared
  in this cycle. Smallest fixes remain unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table, reject/drop
  out-of-bounds remote ids, guard/drop stale sync without a direct metric, and
  clear queued sync state when direct routes are removed. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=198 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Socrates the 6th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` root cause, and the 6,326
  forwarded-stop logs with 4,881 no-capacity and 1,449 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. The seven
  connection-lost/closed/aborted/internal-error signals were reviewed as
  lifecycle noise in the same storm context. No ISSUE-053 or ISSUE-139
  evidence was present, and no new invariant appeared beyond stale sync panic
  plus forwarded-stop storm. Smallest fixes remain unchanged: guard/drop stale
  sync without a direct metric, clear queued sync state when direct routes are
  removed, and add dedupe/TTL/tombstone suppression plus rate-limited repeated
  `try_send` errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=197 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Goodall the 6th` confirmed the `src/router.rs:76` direct-metric panic is
  the existing stale `PeerData::Sync` root cause. The second panic is the fuzz
  harness assertion reporting the background-task panic, not a separate issue.
  No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present, and no new
  invariant appeared in this cycle. The smallest fix proposal remains
  unchanged: guard/drop stale sync without a direct metric and clear queued
  sync state when direct routes are removed. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=196 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Helmholtz the 6th` confirmed the four `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` root cause, and the 10,170
  forwarded-stop logs with 8,671 no-capacity and 1,545 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. The 21
  `broadcast data over peer alias` logs were reviewed as storm-context fallout
  under ISSUE-170, and the eight connection-lost/closed/aborted/internal-error
  signals were reviewed as lifecycle noise. No ISSUE-053 or ISSUE-139
  evidence was present, and no new invariant appeared beyond stale sync panic
  plus forwarded-stop storm. Smallest fixes remain unchanged: guard/drop stale
  sync without a direct metric, clear queued sync state when direct routes are
  removed, and add dedupe/TTL/tombstone suppression plus rate-limited repeated
  `try_send` errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=195 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Euler the 6th` confirmed the three `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause. The
  `channel closed`, connection-lost, and closed-by-peer logs were reviewed as
  teardown/lifecycle noise. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present, and no new invariant appeared beyond the invalid service-id panic.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=194 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `Banach the 6th` confirmed the `src/router.rs:76` direct-metric panic is the
  existing stale `PeerData::Sync` root cause, and the `src/peer.rs:133`
  `should send to main: SendError` panic is the existing shutdown-send root
  cause. The connection-lost and internal endpoint error logs were reviewed as
  lifecycle noise. No ISSUE-053 or ISSUE-170 evidence was present, and no new
  invariant appeared beyond stale sync panic plus shutdown send panic.
  Smallest fixes remain unchanged: guard/drop stale sync without a direct
  metric, clear queued sync state when direct routes are removed, and treat
  closed main receivers as normal teardown in peer shutdown/error reporting.
  No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=193 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Herschel the 6th` confirmed the two `src/ctx.rs:34` panics with index
  `256` into len `256` are the existing unchecked inbound service-id root
  cause. The `channel closed` and closed-by-peer logs were reviewed as
  teardown/lifecycle noise. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present, and no new invariant appeared beyond the invalid service-id panic.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=192 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Galileo the 6th` confirmed the two `src/router.rs:76` direct-metric panics
  are the existing stale `PeerData::Sync` root cause, and the 7,774
  forwarded-stop logs with 6,915 no-capacity and 900 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. Two
  connection-lost/closed/aborted/internal-error signals were reviewed as
  lifecycle noise in the same storm context. No ISSUE-053 or ISSUE-139
  evidence was present, and no new invariant appeared beyond stale sync panic
  plus forwarded-stop storm. Smallest fixes remain unchanged: guard/drop stale
  sync without a direct metric, clear queued sync state when direct routes are
  removed, and add dedupe/TTL/tombstone suppression plus rate-limited repeated
  `try_send` errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=191 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Singer the 6th` confirmed the four `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause. The
  `channel closed`, closed-by-peer, and connection-lost logs were reviewed as
  teardown/lifecycle noise. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present, and no new invariant appeared beyond the invalid service-id panic.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=190 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063, ISSUE-139, and ISSUE-170.
  Reviewer `Cicero the 6th` confirmed the `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` root cause, the
  `src/peer.rs:92` `should send to main: SendError` panic is the existing
  shutdown-send root cause, and the 6,708 forwarded-stop logs with 6,405
  no-capacity and 343 channel-closed logs are the existing peer-alias
  stop-forwarding/backpressure storm. No ISSUE-053 evidence was present, and
  no new invariant appeared beyond stale sync panic, shutdown send panic, and
  forwarded-stop storm. Smallest fixes remain unchanged: guard/drop stale sync
  without a direct metric, clear queued sync state when direct routes are
  removed, treat closed main receivers as normal teardown in peer
  shutdown/error reporting, and add dedupe/TTL/tombstone suppression plus
  rate-limited repeated `try_send` errors for forwarded `PeerStopped`. No new
  issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=189 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Nietzsche the 6th` confirmed the `src/ctx.rs:34` panic with index `256`
  into len `256` is the existing unchecked inbound service-id root cause. The
  single `channel closed` send error and closed-by-peer log were reviewed as
  teardown/lifecycle noise. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present, and no new invariant appeared beyond the invalid service-id panic.
  The smallest fix proposal remains unchanged: validate decoded
  `P2pServiceId` before indexing the fixed service table and reject/drop
  out-of-bounds remote ids. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=188 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Confucius the 6th` confirmed the `src/router.rs:76` direct-metric panic is
  the existing stale `PeerData::Sync` root cause, and the 5,669
  forwarded-stop logs with 5,514 no-capacity and 157 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. Two
  connection-lost/closed/aborted/internal-error signals were reviewed as
  lifecycle noise in the same storm context. No ISSUE-053 or ISSUE-139
  evidence was present, and no new invariant appeared beyond stale sync panic
  plus forwarded-stop storm. Smallest fixes remain unchanged: guard/drop stale
  sync without a direct metric, clear queued sync state when direct routes are
  removed, and add dedupe/TTL/tombstone suppression plus rate-limited repeated
  `try_send` errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=187 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-170. Reviewer
  `Beauvoir the 6th` confirmed the `src/ctx.rs:34` panic with index `256`
  into len `256` is the existing unchecked inbound service-id root cause, and
  the 14,822 forwarded-stop logs with 12,972 no-capacity and 2,007
  channel-closed logs are the existing peer-alias stop-forwarding/backpressure
  storm. The 55 `broadcast data over peer alias got error no available
  capacity` logs were reviewed as storm-context fallout under ISSUE-170. No
  ISSUE-063 or ISSUE-139 evidence was present, and no new invariant appeared
  beyond invalid service-id panic plus forwarded-stop storm. Smallest fixes
  remain unchanged: validate decoded `P2pServiceId` before indexing the fixed
  service table, reject/drop out-of-bounds remote ids, and add
  dedupe/TTL/tombstone suppression plus rate-limited repeated `try_send`
  errors for forwarded `PeerStopped`. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=186 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Darwin the 6th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` root cause, and the 10,425
  forwarded-stop logs with 8,605 no-capacity and 1,840 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. Eight
  connection-lost/closed/aborted/internal-error signals were reviewed as
  lifecycle noise. No ISSUE-053 or ISSUE-139 evidence was present, and no new
  invariant appeared beyond the stale sync panic plus forwarded-stop storm.
  Smallest fixes remain unchanged: guard/drop stale sync without a direct
  metric, clear queued sync state when direct routes are removed, and add
  dedupe/TTL/tombstone suppression plus rate-limited repeated `try_send`
  errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=185 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Russell the 6th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` root cause, and the 5,973
  forwarded-stop logs with 4,843 no-capacity and 1,132 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. One
  connection-lost/closed/aborted/internal-error signal was reviewed as
  lifecycle noise. No ISSUE-053 or ISSUE-139 evidence was present, and no
  separate invariant appeared beyond the stale sync panic plus forwarded-stop
  storm. Smallest fixes remain unchanged: guard/drop stale sync without a
  direct metric, invalidate queued sync when direct routes are removed, and add
  dedupe/TTL/tombstone suppression plus rate-limited repeated `try_send`
  errors for forwarded `PeerStopped`. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=184 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Ramanujan the 6th` confirmed the `src/router.rs:76` direct-metric panic is
  the existing stale `PeerData::Sync` root cause, and the 5,921 forwarded-stop
  logs with 5,732 no-capacity and 206 channel-closed logs are the existing
  peer-alias stop-forwarding/backpressure storm. No ISSUE-053 or ISSUE-139
  evidence was present, and no separate invariant appeared beyond the stale
  sync panic plus forwarded-stop storm. Smallest fixes remain unchanged:
  replace the direct-route `expect` with a guarded lookup/drop for stale sync,
  invalidate queued sync when direct routes are removed, and add
  dedupe/TTL/tombstone suppression plus rate-limited repeated `try_send`
  errors for forwarded `PeerStopped`. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=183 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Dalton the 5th` confirmed the two `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause. The
  two channel-closed logs and two closed-by-peer logs were reviewed as
  lifecycle fallout after connection-task panics. No ISSUE-063, ISSUE-139, or
  ISSUE-170 evidence was present. The smallest fix proposal remains unchanged:
  validate decoded `P2pServiceId` before indexing the fixed service table and
  reject/drop remote ids outside the table bounds. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=182 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Maxwell the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, and the 34,201
  forwarded-stop logs with 31,775 no-capacity and 2,759 channel-closed logs
  are the existing peer-alias stop-forwarding/backpressure storm. The 88
  `broadcast data over peer alias` logs and three transport lifecycle logs
  were reviewed as storm/teardown fallout with no independent failed
  invariant. No ISSUE-053 or ISSUE-139 evidence was present. Smallest fixes
  remain unchanged: replace the direct-route `expect` with a guarded
  lookup/drop for stale sync, and add dedupe/TTL/tombstone suppression for
  forwarded `PeerStopped` aliases. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=181 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Descartes the 5th` confirmed the four `src/ctx.rs:34` panics with index
  `256` into len `256` are the existing unchecked inbound service-id root
  cause. The six channel-closed logs and four connection-lost logs were
  reviewed as lifecycle fallout after connection-task panics. No ISSUE-063,
  ISSUE-139, or ISSUE-170 evidence was present. The smallest fix proposal
  remains unchanged: validate decoded `P2pServiceId` before indexing the fixed
  service table and reject/drop remote ids outside the table bounds. No new
  issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=180 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Rawls the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. The four
  connection-lost logs were reviewed as lifecycle fallout around the same
  disconnect/routing race. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was
  present; there were no backpressure storm, WARN, or path-not-found logs. The
  smallest fix proposal remains unchanged: replace the direct-route `expect`
  with a checked direct-metric lookup and drop or recompute stale sync entries
  when the direct metric is absent. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=179 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-139. Reviewer
  `Galileo the 5th` confirmed the two `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause, and
  the `src/peer.rs:92` `should send to main: SendError` panic is the existing
  peer-connect-error reporting after main-loop shutdown root cause. The
  channel-closed and closed-by-peer logs were reviewed as lifecycle fallout
  after task panics. No ISSUE-063 or ISSUE-170 evidence was present. Smallest
  fixes remain unchanged: validate decoded service ids before indexing the
  fixed table, and replace peer connect-error reporting `expect` calls with
  normal teardown handling when the main receiver is closed. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=178 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Averroes the 5th` confirmed the three `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` root cause, and the
  7,029 forwarded-stop logs with 5,976 no-capacity and 1,068 channel-closed
  logs are the existing peer-alias stop-forwarding/backpressure storm. The
  five transport lifecycle logs were reviewed as storm/teardown fallout with
  no independent failed invariant. No ISSUE-053 or ISSUE-139 evidence was
  present. Smallest fixes remain unchanged: replace the direct-route `expect`
  with a guarded lookup/drop for stale sync, and add dedupe/TTL/tombstone
  suppression for forwarded `PeerStopped` aliases. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=177 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Dewey the 5th` confirmed the five `src/ctx.rs:34` panics with index `256`
  into len `256` are the existing unchecked inbound service-id root cause. The
  channel-closed, connection-lost, and closed-by-peer logs were reviewed as
  lifecycle fallout after the connection task panics. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present. The smallest fix proposal remains
  unchanged: validate decoded `P2pServiceId` before indexing the fixed service
  table and reject/drop remote ids outside the table bounds. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=176 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Wegener the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, and the 6,862
  forwarded-stop logs with 4,800 no-capacity and 2,071 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. The two
  connection-lost lifecycle logs were reviewed as storm/teardown fallout with
  no independent failed invariant. No ISSUE-053 or ISSUE-139 evidence was
  present. Smallest fixes remain unchanged: replace the direct-route `expect`
  with a guarded lookup/drop for stale sync, and add dedupe/TTL/tombstone
  suppression for forwarded `PeerStopped` aliases. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=175 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Boole the 5th` confirmed the `src/ctx.rs:34` panic with index `256` into
  len `256` is the existing unchecked inbound service-id root cause. The
  single channel-closed and single connection-lost logs were reviewed as
  lifecycle fallout after the connection task panic. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present. The smallest fix proposal remains
  unchanged: validate decoded `P2pServiceId` before indexing the fixed service
  table and reject/drop remote ids outside the table bounds. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=174 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `Lorentz the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, and the
  `src/peer.rs:92` `should send to main: SendError` panic is the existing
  peer-connect-error reporting after main-loop shutdown root cause. No
  ISSUE-053 or ISSUE-170 evidence was present, and there were no backpressure
  storm, lifecycle, open_bi, connect-answer, WARN, or path-not-found logs.
  Smallest fixes remain unchanged: replace the direct-route `expect` with a
  guarded lookup/drop for stale sync, and replace peer connect-error reporting
  `expect` calls with normal teardown handling when the main receiver is
  closed. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=173 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Avicenna the 5th` confirmed the `src/ctx.rs:34` panic with index `256`
  into len `256` is the existing unchecked inbound service-id root cause. The
  single channel-closed and single connection-lost logs were reviewed as
  lifecycle fallout after the connection task panic. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present. The smallest fix proposal remains
  unchanged: validate decoded `P2pServiceId` before indexing the fixed service
  table and reject/drop remote ids outside the table bounds. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=172 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Meitner the 5th` confirmed the three `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` root cause, and the
  5,131 forwarded-stop logs with 5,135 no-capacity logs are the existing
  peer-alias stop-forwarding/backpressure storm. No ISSUE-053 or ISSUE-139
  evidence was present, and there were no separate transport, path, open_bi,
  connect-answer, channel-closed, broadcast-data, or WARN invariants. Smallest
  fixes remain unchanged: replace the direct-route `expect` with a guarded
  lookup/drop for stale sync, and add dedupe/TTL/tombstone suppression for
  forwarded `PeerStopped` aliases. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=171 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Russell the 5th` confirmed the `src/ctx.rs:34` panic with index `256`
  into len `256` is the existing unchecked inbound service-id root cause. The
  single channel-closed and single connection-lost logs were reviewed as
  lifecycle fallout after the connection task panic. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present. The smallest fix proposal remains
  unchanged: validate decoded `P2pServiceId` before indexing the fixed service
  table and reject/drop remote ids outside the table bounds. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=170 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Leibniz the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, and the 155,208
  forwarded-stop logs with 152,036 no-capacity and 5,006 channel-closed logs
  are the existing peer-alias stop-forwarding/backpressure storm. The 401
  `broadcast data over peer alias` logs and four `answer open_bi got error
  internal channel error` logs were reviewed as storm/lifecycle fallout because
  they had no separate failed invariant. No ISSUE-053 or ISSUE-139 evidence
  was present. Smallest fixes remain unchanged: replace the direct-route
  `expect` with a guarded lookup/drop for stale sync, and add
  dedupe/TTL/tombstone suppression for forwarded `PeerStopped` aliases. No new
  issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=169 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Peirce the 5th` confirmed the `src/ctx.rs:34` panic with index `256` into
  len `256` is the existing unchecked inbound service-id root cause. The
  single channel-closed and single connection-lost logs were reviewed as
  lifecycle fallout after the connection task panic. No ISSUE-063, ISSUE-139,
  or ISSUE-170 evidence was present. The smallest fix proposal remains
  unchanged: validate decoded `P2pServiceId` before indexing the fixed service
  table and reject/drop remote ids outside the table bounds. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=168 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Dirac the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. The two
  connection-lost logs were reviewed as lifecycle fallout around the same
  disconnect/routing race. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was
  present; there were no backpressure storm, WARN, or path-not-found logs. The
  smallest fix proposal remains unchanged: replace the direct-route `expect`
  with a checked direct-metric lookup and drop or recompute stale sync entries
  when the direct metric is absent. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=167 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Popper the 5th` confirmed the three `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, and the 4,153
  forwarded-stop logs with 4,154 no-capacity logs are the existing peer-alias
  stop-forwarding/backpressure storm. No ISSUE-053 or ISSUE-139 evidence was
  present, and no separate transport, path, open_bi, connect-answer, channel
  closed, broadcast-data, or WARN invariant appeared. Smallest fixes remain
  unchanged: replace the direct-route `expect` with a guarded lookup/drop for
  stale sync, and add dedupe/TTL/tombstone suppression for forwarded
  `PeerStopped` aliases. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=166 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Mendel the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause, and the 7,848
  forwarded-stop logs with 7,151 no-capacity and 732 channel-closed logs are
  the existing peer-alias stop-forwarding/backpressure storm. The six
  `broadcast data over peer alias` logs were reviewed as storm fallout because
  they had no separate failed invariant. No ISSUE-053 or ISSUE-139 evidence
  was present. Smallest fixes remain unchanged: replace the direct-route
  `expect` with a guarded lookup/drop for stale sync, and add
  dedupe/TTL/tombstone suppression for forwarded `PeerStopped` aliases. No new
  issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=165 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Godel the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. No ISSUE-053,
  ISSUE-139, or ISSUE-170 evidence was present; there were no lifecycle,
  backpressure storm, WARN, or path-not-found logs. The smallest fix proposal
  remains unchanged: replace the direct-route `expect` with a checked
  direct-metric lookup and drop or recompute stale sync entries when the direct
  metric is absent. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=164 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Plato the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. No ISSUE-053,
  ISSUE-139, or ISSUE-170 evidence was present; there were no lifecycle,
  backpressure storm, WARN, or path-not-found logs. The smallest fix proposal
  remains unchanged: replace the direct-route `expect` with a checked
  direct-metric lookup and drop or recompute stale sync entries when the direct
  metric is absent. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=163 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Anscombe the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, while the 4,100
  forwarded-stop, 3,974 no-capacity, and 134 channel-closed logs are the
  existing stop-forwarding amplification root cause. No ISSUE-053 or ISSUE-139
  evidence was present; two transport lifecycle lines were teardown fallout
  without a separate invariant. The smallest fix proposals remain unchanged:
  replace the direct-route `expect` with checked stale-sync handling, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=162 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063, ISSUE-139, and ISSUE-170.
  Reviewer `Harvey the 5th` confirmed the one `src/router.rs:76`
  direct-metric panic marker is the existing stale `PeerData::Sync` root
  cause, the one `src/peer.rs:92` `should send to main: SendError` panic is
  the existing peer-connect shutdown reporting root cause, and the 17,201
  forwarded-stop, 17,436 no-capacity, and 32 channel-closed logs are the
  existing stop-forwarding amplification root cause. The 45 broadcast-data
  lines were classified as ISSUE-170 storm fallout. No ISSUE-053 evidence was
  present. The smallest fix proposals remain unchanged: replace the
  direct-route `expect` with checked stale-sync handling, replace
  `PeerConnectError` reporting `expect` sends with checked sends that treat
  closed main-loop receivers as normal shutdown, and bound `PeerStopped`
  propagation with dedupe/tombstones or TTL while suppressing repeated
  forwarding after capacity/channel-closed failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=161 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Boyle the 5th` confirmed the one `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic marker is the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; one closed-by-peer and one channel-closed line were
  teardown fallout without storm markers. The smallest fix proposal remains
  unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=160 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Curie the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. No ISSUE-053,
  ISSUE-139, or ISSUE-170 evidence was present; there were no lifecycle,
  backpressure storm, WARN, or path-not-found logs. The smallest fix proposal
  remains unchanged: replace the direct-route `expect` with a checked
  direct-metric lookup and drop or recompute stale sync entries when the direct
  metric is absent. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=159 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Carson the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; two closed-by-peer and two channel-closed lines were
  teardown fallout without storm markers. The smallest fix proposal remains
  unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=158 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Carver the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, while the 14,052
  forwarded-stop, 13,158 no-capacity, and 955 channel-closed logs are the
  existing stop-forwarding amplification root cause. The 18 broadcast-data
  lines were classified as ISSUE-170 storm fallout. No ISSUE-053 or ISSUE-139
  evidence was present. The smallest fix proposals remain unchanged: replace
  the direct-route `expect` with checked stale-sync handling, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=157 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `James the 5th` confirmed the one `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic marker is the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; one closed-by-peer and one channel-closed line were
  teardown fallout without storm markers. The smallest fix proposal remains
  unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=156 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `McClintock the 5th` confirmed the one `src/router.rs:76` direct-metric
  panic marker is the existing stale `PeerData::Sync` root cause, and the one
  `src/peer.rs:92` `should send to main: SendError` panic is the existing
  peer-connect shutdown reporting root cause. No ISSUE-053 or ISSUE-170
  evidence was present; there were no lifecycle, backpressure storm, WARN, or
  path-not-found logs. The smallest fix proposals remain unchanged: replace
  the direct-route `expect` with checked stale-sync handling, and replace
  `PeerConnectError` reporting `expect` sends with checked sends that treat
  closed main-loop receivers as normal shutdown. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=155 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Laplace the 5th` confirmed the one `src/router.rs:76` direct-metric panic
  marker is the existing stale `PeerData::Sync` root cause. No ISSUE-053,
  ISSUE-139, or ISSUE-170 evidence was present; there were no lifecycle,
  backpressure storm, WARN, or path-not-found logs. The smallest fix proposal
  remains unchanged: replace the direct-route `expect` with a checked
  direct-metric lookup and drop or recompute stale sync entries when the direct
  metric is absent. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=154 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Volta the 5th` confirmed the three `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, while the 12,921
  forwarded-stop, 11,442 no-capacity, and 1,599 channel-closed logs are the
  existing stop-forwarding amplification root cause. The 52 broadcast-data
  lines were classified as ISSUE-170 storm fallout. No ISSUE-053 or ISSUE-139
  evidence was present. The smallest fix proposals remain unchanged: replace
  the direct-route `expect` with checked stale-sync handling, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=153 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Hilbert the 5th` confirmed the 14 `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; seven channel-closed and seven closed/lost peer lines
  were lifecycle fallout without storm markers. The smallest fix proposal
  remains unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=152 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Tesla the 5th` confirmed the four `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` root cause, while the 12,523
  forwarded-stop, 11,163 no-capacity, and 1,401 channel-closed logs are the
  existing stop-forwarding amplification root cause. The 9 broadcast-data lines
  were classified as ISSUE-170 storm fallout. No ISSUE-053 or ISSUE-139
  evidence was present. The smallest fix proposals remain unchanged: discard
  or invalidate stale sync route entries when the direct metric is missing, and
  bound `PeerStopped` propagation with dedupe/tombstones or TTL while
  suppressing repeated forwarding after capacity/channel-closed failures. No
  new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=151 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Ohm the 5th` confirmed the eight `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; the four channel-closed and four closed-by-peer lines
  were lifecycle fallout without storm markers. The smallest fix proposal
  remains unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=150 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063, ISSUE-139, and ISSUE-170.
  Reviewer `Hume the 5th` confirmed the six `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` root cause, the two
  `src/peer.rs:92` send-to-main panic markers are the existing peer background
  shutdown-reporting root cause, and the 6,994 forwarded-stop, 6,332
  no-capacity, and 719 channel-closed logs are the existing stop-forwarding
  amplification root cause. The 21 broadcast-data lines were classified as
  ISSUE-170 storm fallout. No ISSUE-053 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing, replace peer background
  `expect("should send to main")` sends with graceful shutdown handling, and
  bound `PeerStopped` propagation with dedupe/tombstones or TTL while
  suppressing repeated forwarding after capacity/channel-closed failures. No
  new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=149 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-139. Reviewer
  `Bacon the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause, and the two `src/peer.rs:133` send-to-main
  panic markers are the existing background peer task shutdown-reporting root
  cause. No ISSUE-063 or ISSUE-170 evidence was present; the single
  channel-closed and closed-by-peer lines were lifecycle fallout without storm
  markers. The smallest fix proposals remain unchanged: reject/drop inbound
  packets with service ids outside the registered service table before
  indexing, and replace peer background `expect(\"should send to main\")`
  sends with graceful handling when the main-loop receiver is closed. No new
  issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=148 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Kant the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` after direct-route removal
  root cause. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present; the
  six closed-by-peer lines were lifecycle teardown fallout without storm
  markers. The smallest fix proposal remains unchanged: discard or invalidate
  stale sync route entries when the direct metric is missing instead of
  asserting, using a checked lookup that removes or ignores the sync-derived
  route update. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=147 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Raman the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` after direct-route removal
  root cause. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present, and
  no transport lifecycle noise appeared. The smallest fix proposal remains
  unchanged: discard or invalidate stale sync route entries when the direct
  metric is missing instead of asserting, using a checked lookup that removes
  or ignores the sync-derived route update. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=146 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Noether the 5th` confirmed the four `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 21,216 forwarded-stop, 17,777 no-capacity, and 3,535
  channel-closed logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=145 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Darwin the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170
  evidence was present; the single channel-closed and closed-by-peer lines were
  lifecycle fallout without storm markers. The smallest fix proposal remains
  unchanged: reject/drop inbound packets with service ids outside the
  registered service table before indexing, using a bounds-checked lookup and
  treating unknown service ids as invalid remote input. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=144 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Copernicus the 5th` confirmed the four `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` after direct-route
  removal root cause, while the 15,791 forwarded-stop, 14,552 no-capacity, and
  1,346 channel-closed logs are the existing stop-forwarding amplification
  root cause. The 20 broadcast-data lines were classified as ISSUE-170 storm
  fallout. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=143 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-063. Reviewer
  `Pasteur the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic markers are the existing unchecked inbound
  service-id indexing root cause, and the two `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` after direct-route
  removal root cause. No ISSUE-139 or ISSUE-170 evidence was present; the
  single channel-closed and closed-by-peer lines were lifecycle fallout without
  storm markers. The smallest fix proposals remain unchanged: reject/drop
  inbound packets with service ids outside the registered service table before
  indexing, and discard or invalidate stale sync route entries when the direct
  metric is missing instead of asserting. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=142 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Schrodinger the 5th` confirmed the four `src/router.rs:76` direct-metric
  panic markers are the existing stale `PeerData::Sync` after direct-route
  removal root cause, while the 8,080 forwarded-stop, 7,110 no-capacity, and
  1,011 channel-closed logs are the existing stop-forwarding amplification
  root cause. The 23 broadcast-data lines were classified as ISSUE-170 storm
  fallout. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=141 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Pauli the 5th` confirmed the two `src/router.rs:76` direct-metric panic
  markers are the existing stale `PeerData::Sync` after direct-route removal
  root cause. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present; the
  connection-lost/internal-error lines were teardown fallout without a separate
  invariant. The smallest fix proposal remains unchanged: discard or invalidate
  stale sync route entries when the direct metric is missing instead of
  asserting, using a checked lookup that removes or ignores the sync-derived
  route update. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=140 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Franklin the 5th` confirmed the five `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed and peer-closed/lost lines were teardown
  fallout. The smallest fix proposal remains unchanged: reject or drop inbound
  packets with service ids outside the registered service table before
  indexing, using a bounds-checked lookup and treating unknown service ids as
  invalid remote input. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=139 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Lovelace the 5th` confirmed the four `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 7,151 no-capacity, 2,258 channel-closed, and 9,402
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=138 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Feynman the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed and closed-peer lines were teardown fallout. The
  smallest fix proposal remains unchanged: reject or drop inbound packets with
  service ids outside the registered service table before indexing, using a
  bounds-checked lookup and treating unknown service ids as invalid remote
  input. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=137 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Kuhn the 5th` confirmed the two `src/router.rs:76` direct-metric panics
  are the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 34,373 no-capacity, 829 channel-closed, and 34,841
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. The single `answer open_bi got error internal channel error` line was
  classified as lifecycle/storm fallout, not a standalone new issue. No
  ISSUE-053 or ISSUE-139 evidence was present. The smallest fix proposals
  remain unchanged: discard or invalidate stale sync route entries when the
  direct metric is missing instead of asserting, and bound `PeerStopped`
  propagation with dedupe/tombstones or TTL while suppressing repeated
  forwarding after capacity/channel-closed failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=136 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Aquinas the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed and closed-peer lines were teardown fallout. The
  smallest fix proposal remains unchanged: reject or drop inbound packets with
  service ids outside the registered service table before indexing, using a
  bounds-checked lookup and treating unknown service ids as invalid remote
  input. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=135 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Beauvoir the 5th` confirmed the four `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 7,399 no-capacity, 1,380 channel-closed, and 8,779
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=134 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Fermat the 5th` confirmed the `src/router.rs:76` direct-metric panic is
  the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 169,759 no-capacity, 644 channel-closed, and 168,451
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. The 113 `broadcast data over peer alias` backpressure lines were
  classified as duplicate storm fallout under ISSUE-170, and the six
  `answer open_bi got error internal channel error` lines were classified as
  lifecycle/teardown noise accompanying the storm. No ISSUE-053 or ISSUE-139
  evidence was present. The smallest fix proposals remain unchanged: discard
  or invalidate stale sync route entries when the direct metric is missing
  instead of asserting, and bound `PeerStopped` propagation with
  dedupe/tombstones or TTL while suppressing repeated forwarding after
  capacity/channel-closed failures. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=133 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Hypatia the 5th` confirmed the `src/router.rs:76` direct-metric panics are
  the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 8,057 no-capacity, 799 channel-closed, and 8,836 forwarded-stop
  logs are the existing stop-forwarding amplification root cause. The seven
  `broadcast data over peer alias` backpressure lines were classified as
  duplicate storm fallout, not a standalone new issue, because there was no
  separate failed invariant, panic site, data-loss assertion, or distinct root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=132 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Banach the 5th` confirmed the four `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed and peer-closed/lost lines were teardown
  fallout. The smallest fix proposal remains unchanged: reject or drop inbound
  packets whose service id is outside the registered service table before
  indexing, using a bounds-checked lookup and treating unknown service ids as
  invalid remote input. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=131 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Heisenberg the 5th` confirmed the `src/router.rs:76` direct-metric panics
  are the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 10,935 no-capacity, 136 channel-closed, and 11,061
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard or invalidate stale sync route entries
  when the direct metric is missing instead of asserting, and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL while suppressing
  repeated forwarding after capacity/channel-closed failures. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=130 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Singer the 5th` confirmed the single `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic is the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the single channel-closed and closed-by-peer lines were teardown
  fallout. The smallest fix proposal remains unchanged: reject or drop inbound
  packets with service ids outside the registered service table before
  indexing, using a bounds-checked lookup and treating unknown service ids as
  invalid remote input. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=129 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Erdos the 5th` confirmed the `src/router.rs:76` direct-metric panic is the
  existing stale `PeerData::Sync` after direct-route removal root cause, while
  the 5,636 no-capacity, 1,125 channel-closed, and 6,732 forwarded-stop logs
  are the existing stop-forwarding amplification root cause. No ISSUE-053 or
  ISSUE-139 evidence was present. The smallest fix proposals remain unchanged:
  discard or invalidate stale sync route entries when the direct metric is
  missing instead of asserting, and bound `PeerStopped` propagation with
  dedupe/tombstones or TTL while suppressing repeated forwarding after
  capacity/channel-closed failures. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=128 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Planck the 5th` confirmed the single `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic is the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed and closed-by-peer lines were teardown fallout.
  The smallest fix proposal remains unchanged: reject or ignore out-of-range
  service ids before indexing by using a bounds-checked lookup and dropping
  unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=127 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Gauss the 5th` confirmed the single `src/router.rs:76` direct-metric panic
  is the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 9,040 no-capacity, 1,397 channel-closed, and 10,324 forwarded-stop
  logs are the existing stop-forwarding amplification root cause. No ISSUE-053
  or ISSUE-139 evidence was present. The smallest fix proposals remain
  unchanged: discard stale sync for missing direct metrics and bound
  `PeerStopped` propagation with dedupe/tombstones or TTL so a stopped-peer
  notification is forwarded at most once per live connection. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=126 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Sartre the 5th` confirmed the three `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed, connection-lost, and closed-by-peer lines were
  teardown fallout. The smallest fix proposal remains unchanged: reject or
  ignore out-of-range service ids before indexing by using a bounds-checked
  lookup and dropping unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=125 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Goodall the 5th` confirmed the single `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present. The
  smallest fix proposal remains unchanged: discard stale sync for missing direct
  metrics. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=124 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Archimedes the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the channel-closed, closed-by-peer, and aborted-by-peer lines were
  teardown fallout. The smallest fix proposal remains unchanged: reject or
  ignore out-of-range service ids before indexing by using a bounds-checked
  lookup and dropping unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=123 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Lagrange the 5th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 11,573 no-capacity, 665 channel-closed, and 12,190
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. The one `src/peer/peer_internal.rs:167` dropped `open_bi` response
  channel line was classified as lifecycle noise, not distinct accepted issue
  evidence. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard stale sync for missing direct metrics,
  bound `PeerStopped` propagation with dedupe/tombstones or TTL so a stopped-peer
  notification is forwarded at most once per live connection, and optionally
  downgrade dropped `open_bi` response sends during cancellation/shutdown to
  debug-level lifecycle handling. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=122 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Nietzsche the 5th` confirmed the single `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic is the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the one connection-lost and one channel-closed log were teardown
  fallout. The smallest fix proposal remains unchanged: reject or ignore
  out-of-range service ids before indexing by using a bounds-checked lookup and
  dropping unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=121 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Poincare the 5th` confirmed the single `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 5,639 no-capacity, 816 channel-closed, and 6,441
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard stale sync for missing direct metrics and
  bound `PeerStopped` propagation with dedupe/tombstones or TTL so a stopped-peer
  notification is forwarded at most once per live connection. No new issue was
  created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=120 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Sagan the 5th` confirmed the single `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic is the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the one connection-lost and one channel-closed log were teardown
  fallout. The smallest fix proposal remains unchanged: reject or ignore
  out-of-range service ids before indexing by using a bounds-checked lookup and
  dropping unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=119 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Newton the 5th` confirmed the single `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 26,040 no-capacity, 2,358 channel-closed, and 28,230
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. The one `src/lib.rs:326` dropped connect-answer oneshot log was
  classified as lifecycle noise, not distinct accepted issue evidence. No
  ISSUE-053 or ISSUE-139 evidence was present. The smallest fix proposals
  remain unchanged: discard stale sync for missing direct metrics, bound
  `PeerStopped` propagation with dedupe/tombstones or TTL so a stopped-peer
  notification is forwarded at most once per live connection, and optionally
  downgrade dropped connect-answer sends during shutdown/cancelled requests to
  debug-level lifecycle handling. No new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=118 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 only. Reviewer
  `Faraday the 5th` confirmed the three `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause. No ISSUE-063, ISSUE-139, or ISSUE-170 evidence was
  present; the three channel-closed and connection lost/closed logs were
  teardown fallout. The smallest fix proposal remains unchanged: reject or
  ignore out-of-range service ids before indexing by using a bounds-checked
  lookup and dropping unknown-service traffic. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=117 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Epicurus the 5th` confirmed the single `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 4,286 no-capacity, 423 channel-closed, and 4,670
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard stale sync for missing direct metrics,
  and bound `PeerStopped` propagation with dedupe/tombstones or TTL so a
  stopped-peer notification is forwarded at most once per live connection. No
  new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=116 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 only. Reviewer
  `Aristotle the 5th` confirmed the single `src/router.rs:76` direct-metric
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause. No ISSUE-053, ISSUE-139, or ISSUE-170 evidence was present. The
  smallest fix proposal remains unchanged: discard stale sync for missing direct
  metrics. No new issue was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=115 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Helmholtz the 5th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 4,854 no-capacity, 31 channel-closed, and 4,885
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard stale sync for missing direct metrics,
  and bound `PeerStopped` propagation with dedupe/tombstones or TTL so a
  stopped-peer notification is forwarded at most once per live connection. No
  new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=114 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Hooke the 5th` confirmed the three `src/router.rs:76` direct-metric
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause, while the 6,202 no-capacity, 1,454 channel-closed, and 7,615
  forwarded-stop logs are the existing stop-forwarding amplification root
  cause. No ISSUE-053 or ISSUE-139 evidence was present. The smallest fix
  proposals remain unchanged: discard stale sync for missing direct metrics,
  and bound `PeerStopped` propagation with dedupe/tombstones or TTL so a
  stopped-peer notification is forwarded at most once per live connection. No
  new issue was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=113 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-139. Reviewer
  `Chandrasekhar the 5th` confirmed the two `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panics are the existing unchecked inbound service-id
  indexing root cause, while the `src/peer.rs:92` and `src/peer.rs:133`
  send-to-main panics are the existing early `PeerConnectError` reporting after
  main-loop shutdown root cause. The six channel-closed network send logs were
  shutdown fallout, not separate ISSUE-170 evidence. The smallest fix proposals
  remain unchanged: reject or ignore out-of-range service ids before indexing,
  and treat main-channel closure during shutdown as terminal instead of
  panicking. No ISSUE-205 was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=112 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063, ISSUE-139, and ISSUE-170.
  Reviewer `Ptolemy the 5th` confirmed the two `src/router.rs:76`
  direct-metric panics are the existing stale `PeerData::Sync` after
  direct-route removal root cause, the single `src/peer.rs:133` outgoing
  send-to-main panic is the existing early `PeerConnectError` reporting after
  main-loop shutdown root cause, and the 5,278 no-capacity plus 440
  channel-closed forwarded-stop logs are the existing stop-forwarding
  amplification root cause. The smallest fix proposals remain unchanged:
  validate or ignore sync for removed direct routes, treat main-channel closure
  during shutdown as terminal, and add dedupe/TTL/tombstone suppression for
  forwarded stop notifications. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=111 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Bohr the 5th`
  confirmed the single `src/peer.rs:133` outgoing `connecting.await`
  send-to-main panic is the same early `PeerConnectError` reporting after
  main-loop shutdown root cause. No ISSUE-053, ISSUE-063, ISSUE-170, or new
  ISSUE-205 evidence was present. The smallest fix proposal remains unchanged:
  make peer connection tasks treat main-channel closure during shutdown as a
  terminal lifecycle event instead of panicking.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=110 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, `1 passed; 0 failed`, no panic lines, no failed
  assertion, no `ERROR` or `WARN` logs, and no invalid-service, stale-sync,
  send-to-main, no-capacity/channel-closed, or path-not-found markers.
  Reviewer `Herschel the 5th` classified it as `PASS_NO_NEW`; no new root cause
  or fix proposal change was identified, and no ISSUE-205 was created.
- Valid node churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=109 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Turing the 5th`
  confirmed the single `src/router.rs:76` direct-metric panic is the existing
  stale `PeerData::Sync` after direct-route removal root cause. No ISSUE-053,
  ISSUE-139, ISSUE-170, or new ISSUE-205 evidence was present. The smallest fix
  proposal remains unchanged: ignore or validate sync for removed direct routes
  before indexing `directs`.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=108 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Ramanujan the 5th`
  confirmed the four `src/ctx.rs:34` out-of-range `P2pServiceId(256)` panics
  are the existing unchecked inbound service-id indexing root cause. The four
  ordinary channel-closed send logs were shutdown fallout, not separate
  ISSUE-139 or ISSUE-170 evidence. The smallest fix proposal remains unchanged:
  reject or ignore out-of-range service ids before indexing, and treat malformed
  inbound service ids as invalid input rather than panicking. No ISSUE-205 was
  created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=107 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-170. Reviewer
  `Zeno the 5th` confirmed the single `src/router.rs:76` direct-metric panic is
  the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 7,115 no-capacity and 1,782 channel-closed forwarded-stop logs are
  the existing stop-forwarding amplification root cause. The smallest fix
  proposals remain unchanged: ignore or validate sync for removed direct routes
  before indexing `directs`, and add dedupe/TTL/tombstone suppression for
  forwarded stop notifications. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=106 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139 and ISSUE-170. Reviewer
  `Halley the 5th` confirmed the single `src/peer.rs:92` incoming
  `incoming.await` send-to-main panic is the same early `PeerConnectError`
  reporting after main-loop shutdown root cause, while the 6,511 no-capacity
  and 46 channel-closed forwarded-stop logs are the existing stop-forwarding
  amplification root cause. The smallest fix proposals remain unchanged: treat
  main-channel closure during shutdown as terminal instead of panicking, and add
  dedupe/TTL/tombstone suppression for forwarded stop notifications. No
  ISSUE-205 was created.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=105 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, `1 passed; 0 failed`, no panic lines, no failed
  assertion, no `ERROR` or `WARN` logs, and no invalid-service, stale-sync,
  send-to-main, no-capacity/channel-closed, or path-not-found markers.
  Reviewer `Ampere the 5th` classified it as `PASS_NO_NEW`; no new root cause
  or fix proposal change was identified, and no ISSUE-205 was created.
- Valid node churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=104 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Mencius the 5th`
  confirmed the single `src/peer.rs:92` incoming `incoming.await`
  send-to-main panic is the same early `PeerConnectError` reporting after
  main-loop shutdown root cause. No ISSUE-053, ISSUE-063, or ISSUE-170
  evidence was present. The smallest fix proposal remains unchanged: make peer
  connection tasks treat main-channel closure during shutdown as a terminal
  lifecycle event instead of panicking. No ISSUE-205 was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=103 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Kepler the 5th`
  confirmed the single `src/ctx.rs:34` out-of-range `P2pServiceId(256)` panic
  is the existing unchecked inbound service-id indexing root cause. The single
  ordinary channel-closed log was not separate ISSUE-139 or ISSUE-170 evidence.
  The smallest fix proposal remains unchanged: reject or ignore out-of-range
  service ids before indexing, and treat malformed inbound service ids as
  invalid input rather than panicking. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=102 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Gibbs the 5th`
  confirmed the single `src/peer.rs:89` incoming `accept_bi()` send-to-main
  panic is the same early `PeerConnectError` reporting after main-loop shutdown
  root cause. No ISSUE-053, ISSUE-063, or ISSUE-170 evidence was present. The
  smallest fix proposal remains unchanged: make peer connection tasks treat
  main-channel closure during shutdown as a terminal lifecycle event instead of
  panicking. No ISSUE-205 was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170. Reviewer
  `Einstein the 5th` confirmed two `src/router.rs:76` direct-metric panics are
  the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 4,644 no-capacity and 682 channel-closed forwarded-stop logs are
  the existing stop-forwarding amplification root cause. The smallest fix
  proposals remain unchanged: make `apply_sync` tolerate missing direct metrics
  by discarding or rebuilding stale sync entries, and suppress duplicate
  forwarded-stop propagation under churn. No ISSUE-205 was created.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=100 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no `WARN`
  logs, and no no-capacity/channel-closed/path-not-found markers. Reviewer
  `Mill the 5th` classified the six teardown/internal-channel `ERROR` lines as
  lifecycle noise without failing-test evidence. No new root cause or fix
  proposal change was identified, and no ISSUE-205 was created.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=99 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Parfit the 5th`
  confirmed the single `src/ctx.rs:34` out-of-range `P2pServiceId(256)` panic
  is the existing missing inbound service-id validation root cause. The
  smallest fix proposal remains unchanged: reject or drop service ids outside
  the valid `0..256` range before dispatch. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=98 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Arendt the 5th`
  confirmed the single `src/peer.rs:133` send-to-main panic is the existing
  early `PeerConnectError` reporting after main-loop shutdown root cause. No
  ISSUE-053, ISSUE-063, or ISSUE-170 evidence was present for this run. The
  smallest fix proposal remains unchanged: make peer connection tasks treat
  main-channel closure during shutdown as a terminal lifecycle event instead of
  panicking. No ISSUE-205 was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=97 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170. Reviewer
  `Hegel the 5th` confirmed the single `src/router.rs:76` direct-metric panic
  is the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 12,403 no-capacity and 1,318 channel-closed forwarded-stop logs are
  the existing stop-forwarding amplification root cause. The 5 broadcast-data
  channel-closed tail lines are insufficient as a separate root cause. The
  smallest fix proposals remain unchanged: make `apply_sync` tolerate missing
  direct metrics by discarding or rebuilding stale sync entries, and suppress
  duplicate forwarded-stop propagation under churn. No ISSUE-205 was created.
- Clean steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=96 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no
  `ERROR`/`WARN` logs, and no no-capacity/channel-closed/path-not-found
  markers. Reviewer `Confucius the 5th` classified it as `PASS_NO_NEW`; no
  new root cause or fix proposal change was identified.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=95 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170. Reviewer
  `Locke the 5th` confirmed two `src/router.rs:76` direct-metric panics are
  the existing stale `PeerData::Sync` after direct-route removal root cause,
  while the 4,496 no-capacity and 260 channel-closed forwarded-stop logs are
  the existing stop-forwarding amplification root cause. The smallest fix
  proposals remain unchanged: make `apply_sync` tolerate missing direct metrics
  by discarding or rebuilding stale sync entries, and suppress duplicate
  forwarded-stop propagation under churn. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=94 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139 and secondary ISSUE-170. The
  cycle 94 reviewer task confirmed two `src/peer.rs:133` send-to-main panics
  are the existing early `PeerConnectError` reporting after main-loop shutdown
  root cause, while the 207 no-capacity and 9 channel-closed forwarded-stop
  logs are the existing stop-forwarding amplification root cause. The smallest
  fix proposals remain unchanged: make peer error reporting tolerate closed
  main receivers during shutdown, and dedupe or bound forwarded `PeerStopped`
  propagation with TTL/tombstones. No ISSUE-205 was created.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=93 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. The cycle 93 reviewer task
  confirmed three `src/router.rs:76` direct-metric panics are the existing
  stale `PeerData::Sync` after direct-route removal root cause. The smallest
  fix proposal is unchanged: make `apply_sync` tolerate missing direct metrics
  by discarding or rebuilding stale sync entries instead of panicking. No
  ISSUE-205 was created.
- Clean steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=92 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no
  `ERROR`/`WARN` logs, and no no-capacity/channel-closed/path-not-found
  markers. Reviewer `Bernoulli the 5th` classified it as `PASS_NO_NEW`; no
  new root cause or fix proposal change was identified.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=91 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Euler the 5th`
  confirmed the single `src/router.rs:76` direct-metric panic is the existing
  stale `PeerData::Sync` after direct-route removal root cause. The smallest
  fix proposal is unchanged: make `apply_sync` tolerate missing direct metrics
  by discarding stale sync entries instead of panicking. No ISSUE-205 was
  created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=90 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Kierkegaard the 5th`
  confirmed three `src/peer.rs:133` panics and one `src/peer.rs:130` panic
  with `should send to main` are the existing outbound `PeerConnectError`
  reporting panic after main-loop shutdown. No ISSUE-053, ISSUE-063, or
  ISSUE-170 mapping was present for this cycle.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=89 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Euclid the 5th`
  confirmed two `src/router.rs:76` `should have direct metric with apply_sync`
  panics are the existing stale `PeerData::Sync` after direct-route removal
  root cause. No invalid-service, send-to-main, or forwarded-stop storm
  mapping was present for this cycle.
- Clean steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=88 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no
  `ERROR`/`WARN` logs, and no no-capacity/channel-closed/path-not-found
  markers. Reviewer `Cicero the 5th` classified it as `PASS_NO_NEW`.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=87 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Jason the 5th`
  confirmed the single `src/ctx.rs:34` out-of-range `P2pServiceId(256)` panic
  is the existing fixed-service-table indexing root cause. The later
  channel-closed send line was follow-on lifecycle noise, not ISSUE-063,
  ISSUE-139, ISSUE-170, or a new ISSUE-205.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=86 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170
  amplification. Reviewer `Nash the 5th` confirmed the single
  `src/router.rs:76` `should have direct metric with apply_sync` panic is the
  existing stale `PeerData::Sync` after direct-route removal root cause, while
  44,181 no-capacity and 2,266 channel-closed forwarded-stop logs are the
  existing stop-forwarding storm pattern.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=85 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170
  amplification. Reviewer `Linnaeus the 5th` confirmed the single
  `src/router.rs:76` `should have direct metric with apply_sync` panic is the
  existing stale `PeerData::Sync` after direct-route removal root cause, while
  247,968 no-capacity and 81 channel-closed forwarded-stop logs are the
  existing stop-forwarding storm pattern.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=84 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, and no
  no-capacity/channel-closed/path-not-found markers. Reviewer `Hubble the 5th`
  classified the single endpoint-driver-dropped `ERROR` line as pass/no-new
  lifecycle noise because no fuzz invariant failed.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=83 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053, ISSUE-063, and secondary
  ISSUE-170 amplification. Reviewer `Huygens the 5th` confirmed one
  `src/ctx.rs:34` out-of-range `P2pServiceId(256)` panic, three
  `src/router.rs:76` stale direct-route panics, and 7,029 no-capacity plus 744
  channel-closed forwarded-stop logs. No ISSUE-205 was created.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=82 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Lagrange the 4th`
  confirmed three `src/peer.rs:133` `should send to main` panics are the
  existing outgoing `PeerConnectError` reporting panic after main-loop
  shutdown. No forwarded-stop no-capacity/channel-closed storm appeared, so
  reviewer found no ISSUE-170 mapping for this cycle.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=81 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170
  amplification. Reviewer `Ohm the 4th` confirmed two `src/router.rs:76`
  `should have direct metric with apply_sync` panics are the existing stale
  `PeerData::Sync` after direct-route removal root cause, while the 6,250
  no-capacity and 2,077 channel-closed forwarded-stop logs are the existing
  stop-forwarding storm pattern.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=80 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, and no
  no-capacity/channel-closed/path-not-found markers. Reviewer `Laplace the 4th`
  classified three teardown `ERROR` logs as pass/no-new lifecycle noise because
  no fuzz invariant failed and no distinct issue mapping was established.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=79 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139 and secondary ISSUE-170
  amplification. Reviewer `Meitner the 4th` confirmed five `src/peer.rs:133`
  `should send to main` panics are the existing outgoing `PeerConnectError`
  shutdown race, while the 221,765 no-capacity and 2,660 channel-closed
  `PeerStopped` forwarding logs are the existing stop-forwarding storm pattern.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=78 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170
  amplification. Reviewer `Kepler the 4th` confirmed the `src/router.rs:76`
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 5,826 no-capacity and 351 channel-closed `PeerStopped`
  forwarding logs are the existing stop-forwarding storm pattern.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=77 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Feynman the 4th`
  confirmed the `src/ctx.rs:34` panic is the existing out-of-range
  `P2pServiceId(256)` fixed-table indexing issue; the later channel-closed
  send error was downstream noise from the killed connection task.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=76 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no
  `ERROR`/`WARN` log lines, and no no-capacity/channel-closed/path-not-found
  markers. Reviewer `Mendel the 4th` classified it as clean pass/no-new
  baseline coverage.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=75 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and ISSUE-139. Reviewer
  `Hume the 4th` confirmed the `src/router.rs:76` panic is the existing stale
  `PeerData::Sync` after direct-route removal root cause, and the
  `src/peer.rs:92` panic is the existing incoming `PeerConnectError`
  send-to-main shutdown race.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=74 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `McClintock the 4th`
  confirmed the `src/peer.rs:92` `should send to main` panic is the existing
  early `PeerConnectError` reporting panic after main-loop shutdown; sanitized
  churn excludes invalid service ids and forged `PeerStopped`, so the closed
  and refused connection logs stay within the same lifecycle root cause.
- Valid churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=73 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `James the 4th`
  confirmed the `src/peer.rs:92` `should send to main` panic is the existing
  early `PeerConnectError` reporting panic after main-loop shutdown; the
  surrounding closed/refused connection logs were the same shutdown path, not a
  new root cause.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=72 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-063. Reviewer
  `Aquinas the 4th` confirmed the `src/ctx.rs:34` invalid-service panic is
  the existing out-of-range `P2pServiceId(256)` fixed-table indexing issue,
  and the `src/router.rs:76` panic is the existing stale `PeerData::Sync`
  after direct-route removal root cause. The single channel-closed send error
  was secondary noise.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=71 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2400 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, no
  `ERROR`/`WARN` log lines, and no no-capacity/channel-closed/path-not-found
  markers. Reviewer `Copernicus the 4th` classified it as clean pass/no-new
  baseline coverage.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=70 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063 and secondary ISSUE-170
  amplification. Reviewer `Mencius the 4th` confirmed the `src/router.rs:76`
  panic is the existing stale `PeerData::Sync` after direct-route removal root
  cause, while the 56,948 no-capacity `PeerStopped` logs are the existing
  stop-forwarding storm pattern.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=69 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Kuhn the 4th`
  confirmed the `src/router.rs:76` panic is the existing stale
  `PeerData::Sync` after direct-route removal root cause.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=68 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053 and ISSUE-139. Reviewer
  `Aristotle the 4th` confirmed the `src/ctx.rs:34` out-of-range
  `P2pServiceId(256)` panic is the existing fixed-service-table indexing issue,
  and the `src/peer.rs:133` `should send to main` panic is the existing
  shutdown-reporting panic after main-loop closure.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=67 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, and no
  warnings. Reviewer `Linnaeus the 4th` classified the single `answer open_bi`
  internal-channel error log as non-fatal lifecycle noise.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=66 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Dalton the 4th`
  confirmed four `src/router.rs:76` panics are the existing stale
  `PeerData::Sync` after route removal root cause; the no-capacity/channel-closed
  `PeerStopped` storm was secondary ISSUE-170-style evidence.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=65 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Erdos the 4th`
  confirmed the `src/peer.rs:92` panic is the existing unchecked incoming
  `PeerConnectError` send-to-main path after main-loop shutdown; the
  no-capacity/channel-closed `PeerStopped` storm was secondary ISSUE-170-style
  evidence.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=64 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Euler the 4th`
  confirmed the two `src/ctx.rs:34` panics are the existing unchecked
  fixed-service-array index for inbound `P2pServiceId(256)`.
- Clean steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=63 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, no failed assertion, and no
  `ERROR`/`WARN` log lines. Reviewer `Schrodinger the 4th` classified it as
  `PASS_NO_NEW`.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=62 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Carson the 4th`
  confirmed the `src/router.rs:76` panic is the existing stale
  `PeerData::Sync` after route removal root cause, with no secondary
  capacity/channel/path storm observed.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=61 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Lorentz the 4th`
  confirmed the `src/peer.rs:133` panic is the existing unchecked
  `PeerConnectError` send-to-main path after main-loop shutdown; the
  no-capacity/channel-closed `PeerStopped` storm was secondary ISSUE-170-style
  evidence.
- Broad random fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=60 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Boole the 4th`
  confirmed the `src/router.rs:76` panic is the existing stale
  `PeerData::Sync` after route removal root cause.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=59 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with exit status 0, no panic lines, and no failed assertion. Reviewer
  `Singer the 4th` classified teardown channel-closed/internal endpoint-dropped
  logs as non-fatal lifecycle noise overlapping existing ISSUE-170/RC-6 areas,
  not accepted issue evidence.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=58 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Einstein the 4th`
  confirmed two `src/router.rs:76` panics are the existing stale
  `PeerData::Sync` after route removal root cause. The large forwarded-stop
  no-capacity/channel-closed storm was secondary ISSUE-170-style evidence.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=57 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Parfit the 4th`
  confirmed the `src/peer.rs:130` `connection.open_bi().await` error branch is
  the same unchecked early `PeerConnectError` send-to-main root cause as the
  earlier `src/peer.rs:133` churn failures.
- Focused pubsub stale-leave ordering review:
  `cargo test stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat -- --nocapture`
  failed with duplicate evidence for ISSUE-155. Reviewer `Averroes the 4th`
  confirmed pubsub membership controls still lack freshness/version comparison,
  so a delayed stale `PublisherLeaved` can remove membership confirmed by a
  newer heartbeat. No root-cause summary change was needed.
- Focused pubsub stale-destroy lifecycle review:
  `cargo test stale_pubsub_destroy_must_not_create_phantom_channel -- --nocapture`
  failed with duplicate evidence for ISSUE-150. Reviewer `Maxwell the 4th`
  confirmed unknown local-handle destroy controls still create phantom channel
  bookkeeping instead of being no-ops. No root-cause summary change was needed.
- Focused pubsub empty-channel lifecycle review:
  `cargo test empty_pubsub_channels_must_be_removed_after_last_local_handle_drops -- --nocapture`
  failed with duplicate evidence for ISSUE-108. Reviewer `Arendt the 4th`
  confirmed pubsub channel bookkeeping still leaves empty channel state in
  `PubsubService::channels` after the last local handle drops; 1,025 create/drop
  cycles retained 1,025 empty entries. No root-cause summary change was needed.
- Focused pubsub heartbeat resource-bound review:
  `cargo test pubsub_heartbeat_channel_batches_must_be_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-106. Reviewer `Darwin the 4th`
  confirmed `PubsubMessage::Heartbeat` still accepts and processes every
  `ChannelHeartbeat` row without a semantic channel-count cap, so one frame
  updated 1,025 channel states for one remote peer. No root-cause summary
  change was needed.
- Focused pubsub early-join review:
  `cargo test early_remote_publisher_join_must_survive_late_local_subscriber_creation -- --nocapture`
  failed with duplicate evidence for ISSUE-188. Reviewer `Gauss the 4th`
  confirmed inbound `PublisherJoined` is still ignored when no local channel
  state exists, so a later local subscriber misses that remote publisher.
- Focused pubsub local-handle membership review:
  `cargo test new_local_pubsub_handles_must_observe_existing_remote_members -- --nocapture`
  failed with duplicate evidence for ISSUE-142. Reviewer `Leibniz the 4th`
  confirmed new local pubsub handles still replay only local peer presence and
  miss already-known remote publishers/subscribers.
- ISSUE-142 fixed: newly-created local pubsub publishers now replay known
  remote subscribers, and newly-created local subscribers now replay known
  remote publishers, without notifying existing local handles again or changing
  first-local broadcasts. Verification:
  `cargo test new_local_pubsub_handles_must_observe_existing_remote_members -- --nocapture`.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=50 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no failing assertion or error logs. Reviewer `Epicurus the 4th`
  classified it as `PASS_NO_NEW`; it covers live-node randomized valid traffic
  but not churn/shutdown, invalid wire inputs, stale-route-after-disconnect, or
  high-load backpressure failure modes.
- Sanitized churn fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=49 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Carver the 4th`
  confirmed the `src/peer.rs:133` panic is the existing unchecked outbound
  `PeerConnectError` send-to-main path after main-loop shutdown; 8,610
  no-capacity and 548 channel-closed forwarded-stop logs were secondary
  ISSUE-170 amplification evidence rather than a new root cause.
- Valid-action fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=48 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-063. Reviewer `Pauli the 4th`
  confirmed the `src/router.rs:76` panic is the existing stale
  `PeerData::Sync` after route removal; 8,753 no-capacity and 161
  channel-closed forwarded-stop logs were secondary ISSUE-170 amplification
  evidence rather than a new root cause.
- Steady-valid fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=47 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with no failing assertion. Reviewer `Volta the 4th` classified it as
  `PASS_NO_NEW`; the single `answer open_bi got error internal channel error`
  log maps to a dropped requester receiver around `src/peer/peer_internal.rs:167`
  and is adjacent to existing stream setup/backpressure issues, not new failing
  evidence.
- Invalid-service fuzz review:
  `RUST_LOG=error P2P_FUZZ_SEED=6 P2P_FUZZ_NODES=7 P2P_FUZZ_STEPS=900 cargo test fuzz_random_node_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-053. Reviewer `Cicero the 4th`
  confirmed the `src/ctx.rs:34` panic is the existing unchecked
  fixed-service-array index for inbound `P2pServiceId(256)`.
- Sanitized churn fuzz review:
  `P2P_FUZZ_SEED=1 P2P_FUZZ_NODES=6 P2P_FUZZ_STEPS=700 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Pasteur the 4th`
  confirmed the panics at `src/peer.rs:92` and `src/peer.rs:133` are the
  existing unchecked early `PeerConnectError` send-to-main path after the main
  loop has closed during churn.
- Focused inbound ConnectRes write-stall review:
  `cargo test inbound_peer_setup_must_timeout_when_connect_response_write_stalls -- --nocapture`
  failed with duplicate evidence for ISSUE-173. Reviewer `Curie the 4th`
  confirmed inbound `run_connection` still writes `ConnectRes` through
  `write_object` without a setup timeout, so stalled receive-side flow control
  can hang setup and prevent `PeerConnectError`.
- Focused outbound ConnectReq write-stall review:
  `cargo test outbound_peer_setup_must_timeout_when_connect_request_write_stalls -- --nocapture`
  failed with duplicate evidence for ISSUE-172. Reviewer `Kant the 4th`
  confirmed outbound setup still writes `ConnectReq` with `write_object`
  without a setup timeout, so a stalled peer can block the write and prevent
  `PeerConnectError`/pending-neighbour cleanup.
- Focused outbound control-stream setup review:
  `cargo test outbound_peer_setup_must_timeout_when_main_control_stream_cannot_open -- --nocapture`
  failed with duplicate evidence for ISSUE-159. Reviewer `Pascal the 4th`
  confirmed outbound setup still awaits `connection.open_bi().await` without a
  setup timeout, so no `PeerConnectError` is emitted and the pending neighbour
  remains uncleared.
- Focused unauthenticated inbound connection review:
  `cargo test unauthenticated_inbound_connections_must_be_admission_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-134. Reviewer `Newton the 4th`
  confirmed inbound QUIC connections are still accepted/inserted before
  authentication and can wait for the P2P control stream without a node-level
  unauthenticated cap or control-stream timeout.
- Focused unused unidirectional stream review:
  `cargo test unused_unidirectional_streams_must_not_be_admitted -- --nocapture`
  failed with duplicate evidence for ISSUE-182. Reviewer `Hegel the 4th`
  confirmed QUIC still admits unused uni streams because transport config
  allows `max_concurrent_uni_streams(10_000)` while the P2P protocol has no
  `accept_uni` path.
- Focused idle inbound stream admission review:
  `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
  failed with duplicate evidence for ISSUE-117. Reviewer `Avicenna the 4th`
  confirmed the peer loop still accepts every inbound bidirectional stream and
  spawns `accept_bi(...)` without an admission cap or stream-connect read
  timeout.
- Focused orphan relay stream review:
  `cargo test relay_must_not_deliver_downstream_stream_after_upstream_setup_closes -- --nocapture`
  failed with duplicate evidence for ISSUE-156. Reviewer `Archimedes the 4th`
  confirmed the relay still opens the downstream stream with
  `alias.open_stream(...)` before proving the upstream setup acknowledgement is
  writable/live, so upstream cancellation can leave an orphan downstream pipe.
- Focused stalled stream-request write review:
  `cargo test open_stream_must_timeout_when_connect_request_write_stalls -- --nocapture`
  failed with duplicate evidence for ISSUE-169. Reviewer `Bohr the 4th`
  confirmed the same missing whole-setup deadline: `open_bi` times out only
  `connection.open_bi()`, while `write_object(StreamConnectReq)` and the later
  `StreamConnectRes` wait can hang behind peer flow control.
- Focused withheld stream-response review:
  `cargo test open_stream_must_timeout_when_peer_withholds_connect_response -- --nocapture`
  failed with duplicate evidence for ISSUE-149. Reviewer `Halley the 4th`
  confirmed `open_bi` still times out only `connection.open_bi()`, then awaits
  `StreamConnectRes` without a setup deadline after writing `StreamConnectReq`.
- Focused unicast ingress-loop review:
  `cargo test unicast_relay_must_not_forward_back_to_ingress_peer -- --nocapture`
  now passes. Reviewer `Mill the 4th` classified this as existing-issue
  fixed/no-new evidence for ISSUE-197: unicast relay now detects
  `RouteAction::Next(next) == ingress` via `DropIngressLoop` and drops/logs
  instead of forwarding back to the sender. This does not prove stream relay
  loop handling is fixed; ISSUE-180 remains open.
- Focused relay stream ingress-loop review:
  `cargo test relay_stream_must_not_forward_back_to_ingress_peer -- --nocapture`
  failed with duplicate evidence for ISSUE-180. Reviewer `Socrates the 4th`
  confirmed `accept_bi` still blindly relays `RouteAction::Next(next)` with
  `alias.open_stream(...)` and cannot reject forwarding back to the ingress
  peer/connection, so recursive relay setup times out instead of returning a
  prompt route-loop error.
- Focused local open-stream API review:
  `cargo test open_stream_to_local_returns_error_not_panic -- --nocapture`
  failed with duplicate evidence for ISSUE-013. Reviewer `Helmholtz the 4th`
  confirmed `SharedCtx::open_stream` still panics at `src/ctx.rs:235` for
  `RouteAction::Local` instead of returning a recoverable error.
- Focused queue-full stream review:
  `cargo test open_stream_does_not_succeed_when_destination_service_queue_is_full -- --nocapture`
  failed with duplicate evidence for ISSUE-012. Reviewer `Wegener the 4th`
  confirmed the local `open_stream` path still ignores bounded destination
  service acceptor `try_send` failure and reports success for an unconsumed
  pipe after the acceptor queue is full.
- Focused closed-receiver stream review:
  `cargo test open_stream_fails_when_destination_service_receiver_is_closed -- --nocapture`
  failed with duplicate evidence for ISSUE-011. Reviewer `Rawls the 4th`
  confirmed `open_stream` still reports success after the destination service
  receiver is closed because local stream delivery ignores the bounded
  `try_send` failure.
- Focused router active-path stability review:
  `cargo test active_path_should_not_jump_for_tiny_rtt_jitter -- --nocapture`
  and
  `cargo test should_keep_existing_best_path_on_equal_score -- --nocapture`
  both failed with duplicate evidence for ISSUE-003. Reviewer `Galileo the
  4th` confirmed the active route still changes on tiny RTT jitter or
  equal-cost updates because route selection has no stickiness/hysteresis; this
  remains under RC-7 with no new root cause.
- Sanitized churn fuzz duplicate:
  `RUST_LOG=error P2P_FUZZ_SEED=2182001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_sanitized_node_churn_actions_must_not_panic_connection_tasks -- --nocapture`
  failed with duplicate evidence for ISSUE-139. Reviewer `Galileo the 4th`
  confirmed the accepted failure was the outbound `PeerConnectError` path
  panicking at `src/peer.rs:133` with `should send to main: SendError`;
  repeated peer-stopped/backpressure logs overlap existing ISSUE-170 and
  RC-3/RC-6 churn noise without adding a new root cause.
- Steady-valid random action fuzz pass:
  `RUST_LOG=error P2P_FUZZ_SEED=2181001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2600 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed with `1 passed; 0 failed; 289 filtered out; finished in 16.84s`.
  Reviewer `Galileo the 4th` classified it as `NO_NEW_PASS`; because the cycle
  had no failing evidence, it adds no accepted issue and no root-cause impact.
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

- Cycle after ISSUE-204 no-new cycle 341 ran a valid node-churn fuzz pass with
  forked reviewer `Avicenna the 7th`. The run failed with exit code 101 and
  assertion `seed=341, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` panic with
  `should have direct metric with apply_sync`. Invalid-service, shutdown-send,
  PeerStopped storm, no-capacity, channel-closed, broadcast-failure,
  endpoint-driver-dropped, and internal-channel-error signatures were absent.
  Five `connection lost` markers and one `closed by peer` marker were reviewed
  as churn context. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 340 ran a broad random node-action fuzz
  pass with forked reviewer `Nietzsche the 7th`. The run failed with exit code
  101 and assertion `seed=340, nodes=8, steps=5200`. The hard failure was
  duplicate ISSUE-053 evidence: two `src/ctx.rs:34:9` panics with
  `index out of bounds: the len is 256 but the index is 256`. Shutdown-send,
  stale-route, PeerStopped storm, no-capacity, broadcast-failure,
  endpoint-driver-dropped, and internal-channel-error signatures were absent.
  One `closed by peer`, one `connection lost`, and two try-send
  `channel closed` markers were reviewed as lifecycle context. No accepted
  issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 339 ran a sanitized node-churn fuzz pass
  with forked reviewer `Bohr the 7th`. The run failed with exit code 101 and
  assertion `seed=339, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-139 evidence: one `src/peer.rs:92:104` incoming shutdown-send panic
  with `should send to main: SendError { .. }`. Invalid-service, stale-route,
  PeerStopped storm, no-capacity, channel-closed, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. One `aborted by peer` marker
  was reviewed as lifecycle context. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 338 ran a steady valid-node fuzz pass
  with forked reviewer `Heisenberg the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 36.35s. No panic,
  invalid-service, stale-route, shutdown-send, PeerStopped storm,
  connection-lifecycle, channel-closed, endpoint-driver-dropped, or
  internal-channel-error signatures were present. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 337 ran a valid node-churn fuzz pass with
  forked reviewer `Jason the 7th`. The run failed with exit code 101 and
  assertion `seed=337, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-139 evidence: one `src/peer.rs:133:113` outgoing shutdown-send panic
  with `should send to main: SendError { .. }`. Invalid-service, stale-route,
  PeerStopped storm, no-capacity, channel-closed, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. One `aborted by peer` marker
  was reviewed as lifecycle context. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 336 ran a broad random node-action fuzz
  pass with forked reviewer `Bernoulli the 7th`. The run failed with exit code
  101 and assertion `seed=336, nodes=8, steps=5200`. The hard failures were
  duplicate ISSUE-053 evidence, six `src/ctx.rs:34:9` panics with
  `index out of bounds: the len is 256 but the index is 256`, plus duplicate
  ISSUE-139 evidence, one `src/peer.rs:133:113` outgoing shutdown-send panic
  with `should send to main: SendError { .. }`. Stale-route, PeerStopped
  storm, no-capacity, broadcast-failure, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. One `connection lost`, five
  `closed by peer`, six try-send `channel closed`, and three `aborted by peer`
  markers were reviewed as lifecycle context. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 335 ran a sanitized node-churn fuzz pass
  with forked reviewer `Ptolemy the 7th`. The run failed with exit code 101
  and assertion `seed=335, nodes=8, steps=5200`. The hard failure was
  duplicate ISSUE-139 evidence: one `src/peer.rs:133:113` outgoing
  shutdown-send panic with `should send to main: SendError { .. }`.
  Invalid-service, stale-route, PeerStopped storm, no-capacity, channel-closed,
  endpoint-driver-dropped, and internal-channel-error signatures were absent.
  Twelve `connection lost`, 21 closed-by-peer shutdown, and five
  `aborted by peer` markers were reviewed as lifecycle context. No accepted
  issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 334 ran a steady valid-node fuzz pass
  with forked reviewer `Newton the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 36.20s. No panic,
  invalid-service, stale-route, shutdown-send, PeerStopped storm,
  connection-lifecycle, channel-closed, endpoint-driver-dropped, or
  internal-channel-error signatures were present. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 333 ran a valid node-churn fuzz pass with
  forked reviewer `Bacon the 7th`. The run failed with exit code 101 and
  assertion `seed=333, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-063 evidence, one `src/router.rs:76:66` stale-sync panic with
  `should have direct metric with apply_sync`; the 986 forwarded-stop alias
  `no available capacity` errors were duplicate ISSUE-170 backpressure/storm
  evidence. Invalid-service, shutdown-send, broadcast-failure, channel-closed,
  endpoint-driver-dropped, and internal-channel-error signatures were absent.
  One `connection lost`, two `closed by peer`, and two `aborted by peer`
  markers were reviewed as lifecycle context. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 332 ran a broad random node-action fuzz
  pass with forked reviewer `Faraday the 7th`. The run failed with exit code
  101 and assertion `seed=332, nodes=8, steps=5200`. The hard failures were
  duplicate ISSUE-063 evidence, two `src/router.rs:76:66` stale-sync panics
  with `should have direct metric with apply_sync`, plus duplicate ISSUE-170
  evidence, 4,090 forwarded-stop alias errors with 3,072
  `no available capacity` and 1,022 `channel closed` markers.
  Invalid-service, shutdown-send, broadcast-failure, endpoint-driver-dropped,
  and internal-channel-error signatures were absent. One closed-by-peer marker
  was reviewed as churn context. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 331 ran a sanitized node-churn fuzz pass
  with forked reviewer `Erdos the 7th`. The run failed with exit code 101 and
  assertion `seed=331, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-139 evidence: two `src/peer.rs:92:104` incoming shutdown-send panics
  with `should send to main: SendError { .. }`. Invalid-service, stale-route,
  PeerStopped storm, no-capacity, channel-closed, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. Seven `connection lost`,
  eight closed-by-peer shutdown, and five `aborted by peer` markers were
  reviewed as lifecycle context. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 330 ran a steady valid-node fuzz pass
  with forked reviewer `Mill the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 36.23s. No panic,
  invalid-service, stale-route, shutdown-send, PeerStopped storm,
  connection-lifecycle, channel-closed, endpoint-driver-dropped, or
  internal-channel-error signatures were present. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 329 ran a valid node-churn fuzz pass with
  forked reviewer `Galileo the 7th`. The run failed with exit code 101 and
  assertion `seed=329, nodes=8, steps=5200`. The hard failures were duplicate
  ISSUE-063 evidence, two `src/router.rs:76:66` stale-sync panics with
  `should have direct metric with apply_sync`, plus duplicate ISSUE-170
  evidence, 28,465 forwarded-stop alias errors with 27,353
  `no available capacity` and 1,156 `channel closed` markers.
  Invalid-service, shutdown-send, broadcast-failure, endpoint-driver-dropped,
  and internal-channel-error signatures were absent. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 328 ran a broad random node-action fuzz
  pass with forked reviewer `Kierkegaard the 7th`. The run failed with exit
  code 101 and assertion `seed=328, nodes=8, steps=5200`. The hard failures
  were duplicate ISSUE-053 evidence, six `src/ctx.rs:34:9` panics with
  `index out of bounds: the len is 256 but the index is 256`, plus duplicate
  ISSUE-139 evidence, one `src/peer.rs:133:113` outgoing shutdown-send panic
  with `should send to main: SendError { .. }`. Stale-route, PeerStopped
  storm, no-capacity, broadcast-failure, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. Five `closed by peer`, one
  `connection lost`, and six try-send `channel closed` markers were reviewed
  as churn context. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 327 ran a sanitized node-churn fuzz pass
  with forked reviewer `Einstein the 7th`. The run failed with exit code 101
  and assertion `seed=327, nodes=8, steps=5200`. The hard failure was
  duplicate ISSUE-139 evidence: one `src/peer.rs:92:104` incoming
  shutdown-send panic with `should send to main: SendError { .. }`.
  Invalid-service, stale-route, PeerStopped storm, no-capacity,
  channel-closed, endpoint-driver-dropped, and internal-channel-error
  signatures were absent. Three `aborted by peer` connection errors to
  `PeerId(7)` were reviewed as churn context. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 326 ran a valid node-churn fuzz pass with
  forked reviewer `Wegener the 7th`. The run failed with exit code 101 and
  assertion `seed=326, nodes=8, steps=5200`. The hard failure was duplicate
  ISSUE-139 evidence: one `src/peer.rs:130:121` shutdown-send panic with
  `should send to main: SendError { .. }`. Invalid-service, stale-route,
  PeerStopped storm, channel-closed, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. One `connection lost`, five
  `closed by peer`, and one `aborted by peer` marker were reviewed as churn
  context. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 325 ran a steady valid-node fuzz pass with
  forked reviewer `Volta the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 36.44s. No panic,
  invalid-service, stale-route, shutdown-send, PeerStopped storm,
  connection-lifecycle, channel-closed, endpoint-driver-dropped, or
  internal-channel-error signatures were present. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 324 ran a broad invalid-action fuzz pass
  with forked reviewer `Herschel the 7th`. The run failed with exit code 101
  and assertion `seed=324, nodes=8, steps=4800`. The hard failure was
  duplicate ISSUE-053 evidence: two `src/ctx.rs:34:9` service-table panics
  with `index out of bounds: the len is 256 but the index is 256`.
  Stale-route, shutdown-send, PeerStopped storm, endpoint-driver-dropped, and
  internal-channel-error signatures were absent. Two `channel closed` and two
  `closed by peer` markers were reviewed as minor network context. No accepted
  issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 323 ran a valid node-churn fuzz pass with
  forked reviewer `Ramanujan the 7th`. The run failed with exit code 101 and
  assertion `seed=323, nodes=8, steps=4800`. The hard failure was duplicate
  ISSUE-139 evidence: two `src/peer.rs:133:113` shutdown-send panics with
  `should send to main: SendError { .. }`. Invalid-service, stale-route,
  channel-closed, and endpoint-driver-dropped signatures were absent. Four
  stopped-forwarding/no-capacity markers were reviewed as too small and
  context-only for ISSUE-170 in this cycle. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 322 ran a steady valid-node fuzz pass with
  forked reviewer `Boole the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 35.94s. No panic,
  invalid-service, stale-route, shutdown-send, PeerStopped storm,
  channel-closed, or internal-channel-error signatures were present. Two
  `endpoint driver future was dropped` markers and one `connection lost`
  marker were reviewed as teardown/lifecycle noise without failing assertion,
  panic, hang, leak, or data-loss proof. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 321 ran a sanitized node-churn fuzz pass
  with forked reviewer `Darwin the 7th`. The run failed with exit code 101 and
  assertion `seed=321, nodes=8, steps=4800`. The hard failure was duplicate
  ISSUE-139 evidence: eight `src/peer.rs:133:113` shutdown-send panics with
  `should send to main: SendError { .. }`. The run also showed duplicate
  ISSUE-170 storm context: 8,780 `forward peer stopped over peer alias`
  reports, including 8,533 `no available capacity` and 262 `channel closed`
  reports. Invalid-service and stale-route panic signatures were absent. One
  `endpoint driver future was dropped` marker was reviewed as lifecycle
  fallout without independent failure proof. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 320 ran a broad node-churn fuzz pass with
  forked reviewer `Maxwell the 7th`. The run failed with exit code 101 and
  assertion `seed=320, nodes=8, steps=4800`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with
  `should have direct metric with apply_sync`. Invalid-service,
  shutdown-send, and PeerStopped capacity-storm signatures were absent. Six
  `connection lost` markers, one `closed by peer`, and one `aborted by peer`
  marker were reviewed as churn context. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 319 ran a valid-action fuzz pass with
  forked reviewer `Kuhn the 7th`. The run failed with exit code 101 and
  assertion `seed=319, nodes=8, steps=4800`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with
  `should have direct metric with apply_sync`. The run also showed duplicate
  ISSUE-170 storm context: 8,077 `forward peer stopped over peer alias`
  reports, including 7,158 `no available capacity`, 937 `channel closed`, and
  2 `broadcast data over peer alias` reports. Invalid-service,
  shutdown-send, connection-lost, closed-by-peer, and aborted-by-peer
  signatures were absent. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 318 ran a steady valid-node fuzz pass with
  forked reviewer `Franklin the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, `289 filtered out`, finished in 31.08s. No panic,
  failed assertion, invalid-service, stale-route, shutdown-send, PeerStopped
  storm, connection-lifecycle, or endpoint-driver-dropped signatures were
  present. The lone `answer open_bi got error internal channel error` log was
  reviewed as transient teardown context with no proven behavioral impact. No
  accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 317 ran a sanitized node-churn fuzz pass
  with forked reviewer `Lagrange the 7th`. The run failed with exit code 101
  and assertion `seed=317, nodes=8, steps=4800`. The hard failure was
  duplicate ISSUE-139 evidence: nine `src/peer.rs:133:113` shutdown-send
  panics with `should send to main: SendError { .. }`. The run also showed
  duplicate ISSUE-170 storm context: 13,976 `forward peer stopped over peer
  alias` reports, 14,000 `no available capacity` markers, and 12
  `broadcast data over peer alias` markers. Invalid-service and stale-route
  panic signatures were absent. Six `connection lost`, ten `closed by peer`,
  and eight `aborted by peer` markers were reviewed as churn context. No
  accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 316 ran a broad invalid-action fuzz pass
  with forked reviewer `Godel the 7th`. The run failed with exit code 101 and
  assertion `seed=316, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` service-table panic with
  `index out of bounds: the len is 256 but the index is 256`. Stale-route,
  shutdown-send, and PeerStopped capacity-storm signatures were absent. Two
  `channel closed` markers and one `closed by peer` marker were reviewed as
  minor network context. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 315 ran a sanitized node-churn fuzz pass
  with forked reviewer `Kant the 7th`. The run failed with exit code 101 and
  assertion `seed=315, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-139 evidence: eight `src/peer.rs:133:113` shutdown-send panics with
  `should send to main: SendError { .. }`. Invalid-service, stale-route, and
  PeerStopped capacity-storm signatures were absent. Ten `closed by peer`, ten
  `aborted by peer`, and one `connection lost` marker were reviewed as churn
  context. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 314 ran a valid node-churn fuzz pass with
  forked reviewer `Plato the 7th`. The run failed with exit code 101 and
  assertion `seed=314, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: five `src/router.rs:76:66` stale-sync panics with
  `should have direct metric with apply_sync`. The run also showed duplicate
  ISSUE-170 storm context: 30,733 `forward peer stopped over peer alias`
  reports, including 25,450 `no available capacity` and 5,295
  `channel closed` reports. Invalid-service and shutdown-send panic signatures
  were absent. Four `connection lost` markers and six `closed by peer` markers
  were reviewed as lifecycle fallout. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 313 ran a valid churn fuzz pass with
  forked reviewer `McClintock the 7th`. The run failed with exit code 101 and
  assertion `seed=313, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: four `src/router.rs:76:66` stale-sync panics with
  `should have direct metric with apply_sync`. The run also showed duplicate
  ISSUE-170 storm context: 10,119 `forward peer stopped over peer alias`
  reports, including 7,437 `no available capacity` and 2,702 `channel closed`
  reports. Invalid-service and shutdown-send panic signatures were absent.
  Four `connection lost` markers and one `closed by peer` marker were
  reviewed as lifecycle fallout. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 312 ran a broad invalid-action fuzz pass
  with forked reviewer `Popper the 7th`. The run failed with exit code 101 and
  assertion `seed=312, nodes=8, steps=3600`. The hard failures were duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` service-table panic with
  `index out of bounds: the len is 256 but the index is 256`, and duplicate
  ISSUE-139 evidence: one `src/peer.rs:133:113` shutdown-send panic with
  `should send to main: SendError { .. }`. Stale-route and stopped-forwarding
  capacity-storm counts were zero. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 311 ran a valid churn fuzz pass with
  forked reviewer `Curie the 7th`. The run failed with exit code 101 and
  assertion `seed=311, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with `should
  have direct metric with apply_sync`. Invalid-service-id, shutdown-send,
  stopped-forwarding/capacity storm, broadcast-alias, path-not-found,
  channel/connection lifecycle, closed-by-peer, aborted-by-peer, and
  endpoint-driver-dropped counts were zero. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 310 ran a longer steady valid-node fuzz
  pass with forked reviewer `Halley the 7th`. The run passed cleanly with exit
  code 0: `1 passed`, `0 failed`. No panic, failed assertion, invalid
  service-id, stale-route, shutdown-send, PeerStopped forwarding/capacity
  storm, broadcast-alias, path-not-found, channel/connection lifecycle,
  closed-by-peer, aborted-by-peer, or endpoint-driver-dropped evidence was
  present. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 309 ran a broad invalid-action fuzz pass
  with forked reviewer `Cicero the 7th`. The run failed with exit code 101 and
  assertion `seed=309, nodes=8, steps=3600`. The hard failures were duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with `should
  have direct metric with apply_sync`, and duplicate ISSUE-139 evidence: two
  `src/peer.rs:92:104` shutdown-send panics with `should send to main:
  SendError { .. }`. The same log had duplicate ISSUE-170 stopped-peer pressure
  (`forward peer stopped over peer alias` 4114, `no available capacity` 3655,
  `channel closed` 472). Invalid-service-id count was zero. No accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 308 ran a valid-action fuzz pass with
  forked reviewer `Dalton the 7th`. The run failed with exit code 101 and
  assertion `seed=308, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: two `src/router.rs:76:66` stale-sync panics with `should
  have direct metric with apply_sync`. Invalid-service-id, shutdown-send,
  stopped-forwarding/capacity storm, broadcast-alias, path-not-found,
  channel-closed, closed-by-peer, aborted-by-peer, and endpoint-driver-dropped
  counts were zero. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 307 ran a sanitized churn fuzz pass with
  forked reviewer `Hegel the 7th`. The run failed with exit code 101 and
  assertion `seed=307, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: two `src/router.rs:76:66` stale-sync panics with `should
  have direct metric with apply_sync`. The same log had severe duplicate
  ISSUE-170 stopped-peer pressure (`forward peer stopped over peer alias`
  86857, `no available capacity` 85509, `channel closed` 1631) plus 74
  broadcast-alias send errors. Invalid-service-id and shutdown-send counts were
  zero. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 306 ran a steady valid-node fuzz pass with
  forked reviewer `Mencius the 7th`. The run passed cleanly with exit code 0:
  `1 passed`, `0 failed`. No panic, failed assertion, invalid service-id,
  stale-route, shutdown-send, PeerStopped forwarding/capacity storm,
  broadcast-alias, path-not-found, channel/connection lifecycle,
  closed-by-peer, aborted-by-peer, or endpoint-driver-dropped evidence was
  present. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 305 ran a valid churn fuzz pass with
  forked reviewer `Mendel the 7th`. The run failed with exit code 101 and
  assertion `seed=305, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-063 evidence: two `src/router.rs:76:66` stale-sync panics with `should
  have direct metric with apply_sync`. The same log had duplicate ISSUE-170
  stopped-peer pressure (`forward peer stopped over peer alias` 8950,
  `no available capacity` 6154, `channel closed` 2796). Invalid-service-id and
  shutdown-send counts were zero. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 304 ran a broad invalid-action fuzz pass
  with forked reviewer `Confucius the 7th`. The run failed with exit code 101
  and assertion `seed=304, nodes=8, steps=3600`. The hard failure was duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` service-table panic with
  `index out of bounds: the len is 256 but the index is 256`. The two
  channel-closed and one closed-by-peer markers were reviewed as lifecycle
  fallout. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 303 ran a valid-action fuzz pass with
  forked reviewer `Hypatia the 7th`. The run failed with exit code 101 and
  assertion `seed=303, nodes=8, steps=3600`. The hard failures were duplicate
  ISSUE-063 evidence: three `src/router.rs:76:66` stale-sync panics with
  `should have direct metric with apply_sync`, and duplicate ISSUE-139
  evidence: one `src/peer.rs:92:104` shutdown-send panic with `should send to
  main: SendError { .. }`. The same log had duplicate ISSUE-170 stopped-peer
  pressure (`forward peer stopped over peer alias` 4646, `no available
  capacity` 4192, `channel closed` 465) plus two endpoint-driver-dropped
  markers. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 302 ran a sanitized churn fuzz pass with
  forked reviewer `Epicurus the 7th`. The run failed with exit code 101; the
  command requested `P2P_FUZZ_NODES=10`, and the harness reported
  `seed=302, nodes=8, steps=3600` because the test clamps nodes to 8. The hard
  failure was duplicate ISSUE-139 evidence: three `src/peer.rs:133:113`
  shutdown-send panics with `should send to main: SendError { .. }`. The two
  connection-lost and three aborted-by-peer markers were reviewed as lifecycle
  fallout. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 301 ran a twelve-node steady valid-node
  fuzz pass with forked reviewer `Gibbs the 7th`. The run passed cleanly with
  exit code 0: `1 passed`, `0 failed`. No panic, failed assertion, invalid
  service-id, stale-route, shutdown-send, PeerStopped forwarding/capacity
  storm, broadcast-alias, path-not-found, channel/connection lifecycle,
  closed-by-peer, aborted-by-peer, or endpoint-driver-dropped evidence was
  present. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 300 ran a broad invalid-action fuzz pass
  with forked reviewer `James the 7th`. The run failed with exit code 101 and
  assertion `seed=300, nodes=8, steps=3400`. The hard failure was duplicate
  ISSUE-053 evidence: four `src/ctx.rs:34:9` service-table panics with
  `index out of bounds: the len is 256 but the index is 256`. The same log had
  duplicate ISSUE-170 stopped-forwarding pressure (`forward peer stopped over
  peer alias` 6888, `no available capacity` 3531, `channel closed` 3373), but
  no stale-route or shutdown-send signature. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 299 ran a steady valid-node fuzz pass with
  forked reviewer `Gauss the 7th`. The run passed cleanly with exit code 0:
  `1 passed`, `0 failed`. No panic, failed assertion, invalid service-id,
  stale-route, shutdown-send, PeerStopped forwarding/capacity storm,
  channel/connection lifecycle, or endpoint-driver-dropped evidence was
  present. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 298 ran a sanitized churn fuzz pass with
  forked reviewer `Hume the 7th`. The run failed with exit code 101 and
  assertion `seed=298, nodes=8, steps=3400`. The hard failure was duplicate
  ISSUE-139 evidence: one `src/peer.rs:92:104` incoming shutdown-send panic
  with `should send to main: SendError { .. }`. The three connection-lost,
  four closed-by-peer, and one aborted-by-peer markers were reviewed as
  lifecycle fallout around the same shutdown/churn condition. No accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 297 ran a valid-action fuzz pass with
  forked reviewer `Peirce the 7th`. The run failed with exit code 101 and
  assertion `seed=297, nodes=8, steps=3400`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with `should
  have direct metric with apply_sync`. The single closed-by-peer marker was
  reviewed as teardown fallout from the same disconnect/routing race. No
  stopped-forwarding, capacity storm, invalid service-id, or shutdown-send
  evidence was present. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 296 ran a broad invalid-action fuzz pass
  with forked reviewer `Helmholtz the 7th`. The run failed with exit code 101
  and assertion `seed=296, nodes=8, steps=3200`. The hard failure was duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` invalid-service panic with
  `index out of bounds: the len is 256 but the index is 256`. The channel-closed
  and closed-by-peer markers were reviewed as teardown fallout. No accepted
  issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 295 ran a steady valid-node fuzz pass with
  forked reviewer `Goodall the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`. The two connection-lost and three
  endpoint-driver-dropped markers were reviewed as non-failing
  teardown/lifecycle noise. No panic, failed assertion, stale-route marker,
  invalid service-id marker, shutdown-send panic, capacity storm, or
  PeerStopped forwarding evidence was present. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 294 ran a sanitized churn fuzz pass with
  forked reviewer `Copernicus the 7th`. The run failed with exit code 101 and
  assertion `seed=294, nodes=8, steps=3200`. The hard failure was duplicate
  ISSUE-139 evidence: one `src/peer.rs:92:104` incoming shutdown-send panic
  with `should send to main: SendError { .. }`. The seven connection-lost, one
  closed-by-peer, and one aborted-by-peer markers were reviewed as lifecycle
  fallout around the same churn/shutdown condition. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 293 ran a valid-action fuzz pass with
  forked reviewer `Sagan the 7th`. The run failed with exit code 101 and
  assertion `seed=293, nodes=8, steps=3200`. The hard failure was duplicate
  ISSUE-063 evidence: one `src/router.rs:76:66` stale-sync panic with `should
  have direct metric with apply_sync`. The 10,707 stopped-peer forwarding
  markers, including 10,577 no-capacity markers and 135 channel-closed markers,
  were reviewed as duplicate ISSUE-170 amplification. The single
  broadcast-over-peer-alias marker was not enough to establish an independent
  root cause. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 292 ran a broad invalid-action fuzz pass
  with forked reviewer `Pascal the 7th`. The run failed with exit code 101 and
  assertion `seed=292, nodes=8, steps=3000`. The hard failure was duplicate
  ISSUE-053 evidence: seven `src/ctx.rs:34:9` invalid-service panics with
  `index out of bounds: the len is 256 but the index is 256`. The
  connection-lost, channel-closed, and closed-by-peer markers were reviewed as
  teardown fallout after background task panics. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 291 ran a steady valid-node fuzz pass with
  forked reviewer `Lovelace the 7th`. The run passed with exit code 0:
  `1 passed`, `0 failed`, and no panic, invalid-service-id, stale-sync,
  shutdown-send, stopped-forwarding, broadcast-alias, path-not-found, lifecycle,
  capacity, or channel-closed markers. No accepted issue or summary root-cause
  change was recorded.
- Cycle after ISSUE-204 no-new cycle 290 ran a sanitized churn fuzz pass with
  forked reviewer `Boyle the 7th`. The run failed with exit code 101 and
  assertion `seed=290, nodes=8, steps=3000`. The only hard failure was
  duplicate ISSUE-139 evidence: two `src/peer.rs:133:113` outgoing
  shutdown-send panics with `should send to main: SendError { .. }`. The nine
  connection-lost, sixteen closed-by-peer, and nine aborted-by-peer markers
  were reviewed as lifecycle fallout around the same shutdown/churn condition.
  No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 289 ran a valid-action fuzz pass with
  forked reviewer `Nash the 7th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=289, nodes=8, steps=2600`. The run
  failed with exit code 101. The hard failure was duplicate ISSUE-063 evidence:
  one `src/router.rs:76:66` stale-sync panic with `should have direct metric
  with apply_sync`. The 6,413 stopped-peer forwarding markers, including 5,693
  no-capacity markers and 726 channel-closed markers, were reviewed as
  duplicate ISSUE-170 amplification. The broadcast-over-peer-alias markers were
  reviewed as fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 288 ran a broad invalid-action fuzz pass
  with forked reviewer `Raman the 7th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=288, nodes=8, steps=2600`. The
  run failed with exit code 101, but the only hard failure was duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` invalid-service panic with
  `index out of bounds: the len is 256 but the index is 256`. The
  connection-lost and channel-closed markers were reviewed as teardown fallout.
  No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 287 ran a valid-action fuzz pass with
  forked reviewer `Huygens the 7th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=287, nodes=8, steps=2400`. The
  run failed with exit code 101, but the hard failure was duplicate ISSUE-063
  evidence, `src/router.rs:76:66` stale-sync panic. The 7,867 stopped-peer
  forwarding markers, including 7,193 no-capacity markers and 721
  channel-closed markers, were reviewed as duplicate ISSUE-170 evidence. The
  open_bi internal-channel, connection-lost, and closed-by-peer markers were
  reviewed as fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 286 ran a steady valid-action fuzz pass
  with forked reviewer `Socrates the 7th`. The command set
  `P2P_FUZZ_NODES=12` and `P2P_FUZZ_STEPS=3600`; the run passed with exit code
  0 and `1 passed; 0 failed`. All tracked accepted-issue signatures were
  absent, including stale route-sync, invalid service-id, shutdown-send, and
  stopped-peer storm/backpressure markers. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 285 ran a valid-action fuzz pass with
  forked reviewer `Planck the 7th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=285, nodes=8, steps=2400`. The run
  failed with exit code 101, but the hard failures were duplicate ISSUE-063
  evidence, `src/router.rs:76:66` stale-sync panic, and duplicate ISSUE-139
  evidence, `src/peer.rs:92:104` shutdown-send panic. The 9,760 stopped-peer
  forwarding markers, including 8,770 no-capacity markers and 1,112
  channel-closed markers, were reviewed as duplicate ISSUE-170 evidence. The
  broadcast-alias and connection-lost markers were reviewed as fallout. No
  accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 284 ran a broad invalid-action fuzz pass
  with forked reviewer `Hypatia the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=284, nodes=8, steps=2600`. The
  run failed with exit code 101, but the only hard failure was duplicate
  ISSUE-053 evidence: three `src/ctx.rs:34:9` invalid-service panics with
  `index out of bounds: the len is 256 but the index is 256`. The
  connection-lost and channel-closed markers were reviewed as teardown fallout.
  No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 283 ran a steady valid-action fuzz pass
  with forked reviewer `Jason the 6th`. The command set `P2P_FUZZ_NODES=12`
  and `P2P_FUZZ_STEPS=3600`; the run passed with exit code 0 and `1 passed; 0
  failed`. All tracked accepted-issue signatures were absent, including stale
  route-sync, invalid service-id, shutdown-send, and stopped-peer
  storm/backpressure markers. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 282 ran a broad invalid-action fuzz pass
  with forked reviewer `Pauli the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=282, nodes=8, steps=2600`. The
  run failed with exit code 101, but the only hard failure was duplicate
  ISSUE-053 evidence: one `src/ctx.rs:34:9` invalid-service panic with
  `index out of bounds: the len is 256 but the index is 256`. The
  channel-closed and closed-by-peer markers were reviewed as teardown fallout.
  No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 281 ran a valid-action fuzz pass with
  forked reviewer `Nash the 6th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=281, nodes=8, steps=2400`. The run
  failed with exit code 101, but the hard failures were duplicate ISSUE-063
  evidence, two `src/router.rs:76:66` stale-sync panics, and duplicate
  ISSUE-139 evidence, two `src/peer.rs:92:104` shutdown-send panics. The
  1,379 stopped-peer forwarding markers, including 1,022 no-capacity markers
  and 361 channel-closed markers, were reviewed as duplicate ISSUE-170
  evidence. Connection-lost and closed-by-peer markers were reviewed as
  teardown fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 280 ran a broad invalid-action fuzz pass
  with forked reviewer `Bacon the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=280, nodes=8, steps=2600`. The
  run failed with exit code 101, but the hard failures were duplicate
  ISSUE-063 evidence, `src/router.rs:76:66` panicked with
  `should have direct metric with apply_sync`, and duplicate ISSUE-139
  evidence, `src/peer.rs:92:104` panicked with
  `should send to main: SendError { .. }`. The endpoint-driver-dropped and
  connection-lost markers were reviewed as teardown fallout. No accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 279 ran a steady valid-action fuzz pass
  with forked reviewer `Carson the 6th`. The command set
  `P2P_FUZZ_NODES=12` and `P2P_FUZZ_STEPS=3600`; the run passed with exit code
  0 and `1 passed; 0 failed`. All tracked known-failure signatures were
  absent, including stale route-sync, invalid service-id, shutdown-send, and
  stopped-peer storm/backpressure markers. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 278 ran a valid-action fuzz pass with
  forked reviewer `Meitner the 6th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=278, nodes=8, steps=2400`. The
  run failed with exit code 101, but the hard failure was duplicate ISSUE-063
  evidence: `src/router.rs:76:66` panicked with
  `should have direct metric with apply_sync`. The 29,375 stopped-peer
  forwarding markers, including 28,621 no-capacity markers and 1,019
  channel-closed markers, were reviewed as duplicate ISSUE-170 evidence. The
  broadcast-alias, open_bi internal-channel, and connection-lost markers were
  reviewed as fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 277 ran a broad invalid-action fuzz pass
  with forked reviewer `Laplace the 6th`. The command set
  `P2P_FUZZ_NODES=10`, while the failing assertion reported `seed=277,
  nodes=8, steps=2600`. The run failed with exit code 101, but the only hard
  failure was duplicate ISSUE-053 evidence: three `src/ctx.rs:34:9`
  invalid-service panics with `index out of bounds: the len is 256 but the
  index is 256`. The channel-closed and closed-by-peer markers were reviewed
  as teardown fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 276 ran a steady valid-action fuzz pass
  with forked reviewer `Dalton the 6th`. The command set
  `P2P_FUZZ_NODES=12` and `P2P_FUZZ_STEPS=3600`; the run passed with exit code
  0 and `1 passed; 0 failed`. All tracked known-failure signatures were
  absent, including stale route-sync, invalid service-id, shutdown-send, and
  stopped-peer storm markers. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 275 ran a valid-action fuzz pass with
  forked reviewer `Aristotle the 6th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=275, nodes=8, steps=2400`. The
  run failed with exit code 101, but the only hard failure was duplicate
  ISSUE-063 evidence: `src/router.rs:76:66` panicked with
  `should have direct metric with apply_sync`. One connection-closed log line
  was reviewed as fallout. No invalid-service, shutdown-send, channel-closed,
  connection-lost, closed-by-peer, path-not-found, no-capacity,
  forwarded-stop, broadcast-data, open_bi, connect-answer, or aborted-by-peer
  evidence appeared. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 274 ran a broad invalid-action fuzz pass
  with forked reviewer `Parfit the 6th`. The command set
  `P2P_FUZZ_NODES=10`, while the failing assertion reported `seed=274,
  nodes=8, steps=2600`. The run failed with exit code 101, but the hard
  failures were duplicate ISSUE-053 evidence, three `src/ctx.rs:34:9`
  invalid-service panics, and duplicate ISSUE-139 evidence, one
  `src/peer.rs:92:104` shutdown-send panic. The channel-closed and
  closed-by-peer markers were reviewed as teardown fallout. No accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 273 ran a valid-action fuzz pass with
  forked reviewer `Kant the 6th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=273, nodes=8, steps=2400`. The run
  failed with exit code 101, but the only hard failure was duplicate ISSUE-063
  evidence: `src/router.rs:76:66` panicked with
  `should have direct metric with apply_sync`. No invalid-service,
  shutdown-send, channel-closed, connection-lost, closed-by-peer,
  path-not-found, no-capacity, forwarded-stop, broadcast-data, open_bi,
  connect-answer, or aborted-by-peer evidence appeared. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 272 ran a broad invalid-action fuzz pass
  with forked reviewer `Ampere the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=272, nodes=8, steps=2600`. The
  run failed with exit code 101, but the hard failures were duplicate
  ISSUE-053 evidence, six `src/ctx.rs:34:9` invalid-service panics, and
  duplicate ISSUE-139 evidence, one `src/peer.rs:92:104` shutdown-send panic.
  The channel-closed, connection-lost, closed-by-peer, and endpoint-internal
  markers were reviewed as teardown fallout. No accepted issue or summary
  root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 271 ran a valid-action fuzz pass with
  forked reviewer `Archimedes the 6th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=271, nodes=8, steps=2400`. The
  run failed with exit code 101, but the hard failure was duplicate ISSUE-063
  evidence: `src/router.rs:76:66` panicked with `should have direct metric with
  apply_sync`. The 14,271 stopped-peer forwarding errors, including 14,080
  no-capacity markers and 235 channel-closed markers, were duplicate ISSUE-170
  evidence. The seven broadcast-alias errors were reviewed as fallout. No
  accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 270 ran a fourteen-node steady-valid fuzz
  pass with forked reviewer `Plato the 6th`. The run passed with exit code 0
  and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289
  filtered out; finished in 24.34s`. All tracked panic, route-sync,
  invalid-service, shutdown-send, stopped-peer storm, transport, and path
  fallout signatures were zero. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 269 ran a broad invalid-action fuzz pass
  with forked reviewer `Popper the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=269, nodes=8, steps=2600`. The
  run failed with exit code 101, but the hard failures were duplicate ISSUE-053
  evidence: five `src/ctx.rs:34:9` invalid-service panics. The five
  channel-closed, four closed-by-peer, and one connection-lost markers were
  reviewed as teardown fallout. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 268 ran a valid-action fuzz pass with
  forked reviewer `Franklin the 6th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=268, nodes=8, steps=2400`. The
  run failed with exit code 101, but the only hard invariant failures were
  duplicate ISSUE-063 evidence: two `src/router.rs:76:66` stale direct-metric
  panics. All other tracked invalid-service, shutdown-send, stopped-peer
  storm, transport, and path signatures were zero. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 267 ran a fourteen-node steady-valid fuzz
  pass with forked reviewer `Averroes the 6th`. The run passed with exit code 0
  and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289
  filtered out; finished in 24.47s`. All tracked panic, route-sync,
  invalid-service, shutdown-send, stopped-peer storm, transport, and path
  fallout signatures were zero. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 266 ran a broad invalid-action fuzz pass
  with forked reviewer `Hubble the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=266, nodes=8, steps=2600`. The
  run failed with exit code 101, but the hard failure was duplicate ISSUE-053
  evidence: `src/ctx.rs:34:9` indexed service id 256 into the 256-entry service
  table. The one channel-closed marker and one closed-by-peer marker were
  reviewed as teardown fallout. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 265 ran a valid-action fuzz pass with
  forked reviewer `Curie the 6th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=265, nodes=8, steps=2400`. The run
  failed with exit code 101, but the hard failures were duplicate ISSUE-063
  evidence, four `src/router.rs:76:66` stale direct-metric panics. The 15,273
  stopped-peer forwarding errors, including 14,531 no-capacity markers and 804
  channel-closed markers, were duplicate ISSUE-170 evidence. The 13
  broadcast-alias errors were reviewed as fallout. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 264 ran a fourteen-node steady-valid fuzz
  pass with forked reviewer `Peirce the 6th`. The run passed with exit code 0
  and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289
  filtered out; finished in 24.27s`. All tracked panic, route-sync,
  invalid-service, shutdown-send, stopped-peer storm, transport, and path
  fallout signatures were zero. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 263 ran a broad invalid-action fuzz pass
  with forked reviewer `Erdos the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=263, nodes=8, steps=2600`. The
  run failed with exit code 101, but the only hard invariant failure was
  duplicate ISSUE-063 evidence: `src/router.rs:76:66` panicked with `should
  have direct metric with apply_sync`. All other tracked invalid-service,
  shutdown-send, stopped-peer storm, transport, and path signatures were zero.
  No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 262 ran a valid-action fuzz pass with
  forked reviewer `Sagan the 6th`. The command set `P2P_FUZZ_NODES=9`, while
  the failing assertion reported `seed=262, nodes=8, steps=2400`. The run
  failed with exit code 101, but the hard failures were duplicate ISSUE-063
  evidence, two `src/router.rs:76:66` stale direct-metric panics. The 6,100
  stopped-peer forwarding errors, including 6,133 no-capacity markers and 12
  channel-closed markers, were duplicate ISSUE-170 evidence. The five
  broadcast-alias errors and one connection-lost/internal endpoint marker were
  reviewed as fallout. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 261 ran a broad invalid-action fuzz pass
  with forked reviewer `Gauss the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=261, nodes=8, steps=2600`. The
  run failed with exit code 101, but the hard failures were duplicate
  ISSUE-053 evidence, one `src/ctx.rs:34:9` invalid-service panic, and
  duplicate ISSUE-139 evidence, one `src/peer.rs:133:113` shutdown-send panic.
  The 4,704 stopped-peer forwarding errors, including 3,791 no-capacity
  markers and 941 channel-closed markers, were duplicate ISSUE-170 evidence.
  The 17 broadcast-alias errors and one closed-by-peer marker were reviewed as
  fallout. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 260 ran a fourteen-node steady-valid fuzz
  pass with forked reviewer `Newton the 6th`. The run passed with exit code 0
  and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289
  filtered out; finished in 24.49s`. All tracked panic, route-sync,
  invalid-service, shutdown-send, stopped-peer storm, transport, and path
  fallout signatures were zero. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 259 ran a valid-action fuzz pass with
  forked reviewer `Kierkegaard the 6th`. The command set `P2P_FUZZ_NODES=9`,
  while the failing assertion reported `seed=259, nodes=8, steps=2200`. The
  run failed with exit code 101, but the only hard invariant failure was
  duplicate ISSUE-063 evidence: `src/router.rs:76:66` panicked with `should
  have direct metric with apply_sync`. The single `P2pNetwork connection ...
  outgoing: None error closed` line was not enough to establish a distinct
  issue. No accepted issue or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 258 ran a broad invalid-action fuzz pass
  with forked reviewer `James the 6th`. The command set `P2P_FUZZ_NODES=10`,
  while the failing assertion reported `seed=258, nodes=8, steps=2400`. The
  run failed with exit code 101, but the hard failures were duplicate
  ISSUE-053 evidence, seven `src/ctx.rs:34:9` invalid-service panics, and
  duplicate ISSUE-139 evidence, one `src/peer.rs:133:113` shutdown-send panic.
  The seven channel-closed, six closed-by-peer, one connection-lost, and two
  aborted-by-peer markers were reviewed as teardown fallout. No accepted issue
  or summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 257 ran a twelve-node steady-valid fuzz
  pass with forked reviewer `Raman the 6th`. The run passed with exit code 0
  and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 289
  filtered out; finished in 21.71s`. All tracked panic, route-sync,
  invalid-service, shutdown-send, stopped-peer storm, transport, and path
  fallout signatures were zero. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 256 ran a larger valid-action fuzz pass
  with forked reviewer `Aquinas the 6th`. The command set
  `P2P_FUZZ_NODES=10`, while the failing assertion reported `seed=256,
  nodes=8, steps=2400`. The run failed with exit code 101, but the hard
  invariant was duplicate ISSUE-063 evidence: `src/router.rs:76:66` panicked
  with `should have direct metric with apply_sync`. The 78,653 stopped-peer
  forwarding errors, including 78,397 no-capacity markers and 648
  channel-closed markers, were duplicate ISSUE-170 storm evidence. The lone
  open_bi internal-channel error, 55 broadcast-alias errors, and two
  connection-lost markers were reviewed as fallout. No accepted issue or
  summary root-cause change was recorded.
- Cycle after ISSUE-204 no-new cycle 255 ran an eight-node broad invalid-action
  fuzz pass with forked reviewer `Hooke the 6th`. The run failed with exit
  code 101, but the hard failure was duplicate ISSUE-053 evidence:
  `src/ctx.rs:34:9` indexed service id 256 into the 256-entry service table.
  The one channel-closed send marker and one closed-by-peer marker were
  reviewed as teardown fallout. No accepted issue or summary root-cause change
  was recorded.
- Cycle after ISSUE-204 no-new cycle 254 ran an eight-node valid-action fuzz
  pass with forked reviewer `Faraday the 6th`. The run failed with exit code
  101, but both hard invariant failures were duplicate ISSUE-063 evidence:
  `src/router.rs:76:66` panicked twice with `should have direct metric with
  apply_sync`. The 13,654 stopped-peer forwarding errors, including 11,854
  no-capacity markers and 1,857 channel-closed markers, were duplicate
  ISSUE-170 storm evidence. No accepted issue or summary root-cause change was
  recorded.
- Cycle after ISSUE-204 no-new cycle 253 ran an eight-node valid-action fuzz
  pass with forked reviewer `Fermat the 6th`. The run failed with exit code
  101, but the only hard invariant was duplicate ISSUE-063 evidence:
  `src/router.rs:76:66` panicked with `should have direct metric with
  apply_sync`. The 47,925 stopped-peer forwarding errors, including 47,896
  no-capacity markers and 542 channel-closed markers, were duplicate ISSUE-170
  storm evidence; the 42 broadcast-alias errors were reviewed as fallout. No
  accepted issue or summary root-cause change was recorded.
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
