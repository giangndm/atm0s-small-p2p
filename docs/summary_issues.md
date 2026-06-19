# Issue Summary

Short review copy for the RED-team issue ledger. The detailed evidence,
reviewer decisions, scores, and failing tests remain in `docs/found_issues.md`.

## Audit Status

- Accepted issues: 173
- Missing issue scores: 0
- Current consecutive no-new-issue cycles: 0
- Stop condition: continue until 5 consecutive cycles find no new accepted
  issue; after that, continue with randomized fuzz tests over node actions.

## Root Cause Summary

### RC-1: Authenticated identity is not authoritative

- Representative issues: ISSUE-001, ISSUE-004, ISSUE-014, ISSUE-015,
  ISSUE-018, ISSUE-020, ISSUE-039, ISSUE-048, ISSUE-066, ISSUE-067,
  ISSUE-068, ISSUE-090, ISSUE-115, ISSUE-116, ISSUE-145.
- Pattern: message payloads and internal events carry peer ids, RPC ids, or
  source identities that are trusted without binding them to the live
  authenticated connection, local handle, expected responder, or channel role.
- Minimal fix proposal: derive source identity from authenticated connections,
  validate `(ConnectionId, PeerId)` before processing main events, and store
  expected responder/handle metadata before accepting answers.

### RC-2: Protocol state machines lack correlation/freshness checks

- Representative issues: ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-047,
  ISSUE-059, ISSUE-071, ISSUE-081 through ISSUE-089, ISSUE-095, ISSUE-099,
  ISSUE-110, ISSUE-111, ISSUE-138, ISSUE-141, ISSUE-143, ISSUE-152,
  ISSUE-154, ISSUE-155, ISSUE-158, ISSUE-166, ISSUE-171.
- Pattern: replicated-KV, alias, metrics, visualization, and pubsub flows accept
  stale, unsolicited, reordered, or mismatched responses because handlers do
  not verify request shape, bounds, version, continuation key, expected phase,
  or membership generation.
- Minimal fix proposal: keep a small pending-request descriptor per flow and
  reject responses unless they match; for membership gossip, carry a generation
  or epoch and ignore older join/leave/heartbeat state.

### RC-3: Backpressure is inconsistent across async boundaries

- Representative issues: ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-118,
  ISSUE-119, ISSUE-120, ISSUE-123, ISSUE-124, ISSUE-125, ISSUE-126,
  ISSUE-127, ISSUE-133, ISSUE-136, ISSUE-147, ISSUE-153, ISSUE-157,
  ISSUE-163, ISSUE-164.
- Pattern: some paths drop on `try_send`, some await bounded sends from
  critical tasks, and others use unbounded queues or duplicate internal control
  work. Under load this causes silent loss, head-of-line blocking, or unbounded
  memory.
- Minimal fix proposal: define a channel policy by event class; lifecycle and
  route updates need bounded retry/coalescing, service payload delivery needs
  explicit backpressure errors, and peer tasks must not await bounded lifecycle
  reporting before they can process traffic or cleanup.

### RC-4: Timeouts and setup cancellation are incomplete

- Representative issues: ISSUE-002, ISSUE-009, ISSUE-021, ISSUE-036,
  ISSUE-042, ISSUE-093, ISSUE-117, ISSUE-121, ISSUE-134, ISSUE-149,
  ISSUE-156, ISSUE-159, ISSUE-169, ISSUE-172, ISSUE-173.
- Pattern: timeouts wrap only one await point, rely on unchecked timestamp
  arithmetic, use coarse global sweeps, or complete one side of setup before
  proving the end-to-end setup is still alive.
- Minimal fix proposal: use checked/saturating deadline math, wrap every
  protocol phase in an end-to-end timeout, and tie relay downstream setup to
  upstream cancellation.

### RC-5: Application-level resource limits are missing

- Representative issues: ISSUE-010, ISSUE-024, ISSUE-027, ISSUE-035,
  ISSUE-041, ISSUE-043, ISSUE-045, ISSUE-046, ISSUE-100 through ISSUE-108,
  ISSUE-122, ISSUE-131.
- Pattern: decoded service-level collections, pending maps, cache sets,
  tombstones, remote stores, and retained channel state often have no item-count
  or lifetime cap.
- Minimal fix proposal: add per-structure caps with deterministic
  eviction/rejection: max rows per message, max peers per alias/channel, max
  pending RPCs/finds, max tombstones/remotes, and prune empty channel state on
  teardown.

### RC-6: Lifecycle cleanup and stale handles are inconsistent

- Representative issues: ISSUE-028, ISSUE-029, ISSUE-051, ISSUE-057,
  ISSUE-060, ISSUE-064, ISSUE-065, ISSUE-069 through ISSUE-076, ISSUE-108,
  ISSUE-128 through ISSUE-132, ISSUE-135, ISSUE-139, ISSUE-142, ISSUE-144,
  ISSUE-148, ISSUE-150, ISSUE-151, ISSUE-161, ISSUE-162, ISSUE-165,
  ISSUE-167, ISSUE-168, ISSUE-170.
- Pattern: requesters, services, peer aliases, channel state, and cached hints
  can outlive the owner they represent; shutdown paths can panic, leak, emit
  false public events, or keep stale routes/cache entries.
- Minimal fix proposal: add generation or liveness tokens to cloned requesters
  and local handles, make closed channels return `Err`, and centralize teardown
  for aliases, metrics, routes, caches, and service ids.

### RC-7: Routing/discovery accepts unstable topology

- Representative issues: ISSUE-003, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-008, ISSUE-033, ISSUE-044, ISSUE-055, ISSUE-092, ISSUE-103,
  ISSUE-112 through ISSUE-114, ISSUE-160, ISSUE-161, ISSUE-164, ISSUE-167.
- Pattern: route/discovery inputs can include local ids, self seeds, stale
  addresses, overflowed metrics, over-hop routes, duplicate connection races,
  or tiny RTT jitter that changes active paths too aggressively.
- Minimal fix proposal: sanitize before insertion: reject local/self candidates
  and over-hop routes, pin authenticated direct paths for their peer ids, use
  checked metric math, ignore stale discovery timestamps, coalesce duplicate
  connects, and add hysteresis before switching active paths.

## Recent Accepted Issues

- ISSUE-168, score 44: duplicate pubsub local ids detach live publisher
  handles. Reviewer: Jason the 3rd.
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

## Next Candidate To Validate

- None queued. ISSUE-173 reset the no-new counter to 0. Continue fresh source
  review; if five consecutive cycles find no issue, switch to randomized fuzz
  tests over node actions.
