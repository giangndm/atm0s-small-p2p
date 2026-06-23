# Issue Summary

Short review copy for the RED-team issue ledger. The detailed evidence,
reviewer decisions, scores, and failing tests remain in `docs/found_issues.md`.

## Audit Status

- Accepted issues: 247
- Missing issue scores: 0
- Current consecutive no-new-issue cycles: 16
- Current audit continuation: critical-only high-load resource cap,
  backpressure, and unbounded-state review found no new score-80+ issue with
  concrete failing-test evidence.
- Critical-only no-new cycle 16 after ISSUE-247 reviewed `src/secure.rs`,
  `src/discovery.rs`, `src/router.rs`, `src/lib.rs`, `src/ctx.rs`,
  `src/peer.rs`, `src/peer/peer_internal.rs`, `src/msg.rs`,
  `src/requester.rs`, alias, pubsub, metrics, visualization, replicated-KV,
  remote storage, and bounded, cap, full, backpressure, pending, overflow,
  resource, queue, and adversarial fuzz tests. Local bounded, cap, full,
  backpressure, pending, overflow, queue, and 50-node 3000-step adversarial
  fuzz seed `105049` checks passed; `resource` had zero matching tests.
  Reviewer `Sagan the 2nd` returned `NO_NEW_CRITICAL` after independently
  reviewing the same high-load resource-cap/backpressure slice, then passing
  bounded, task, full, and 36-node 1500-step adversarial fuzz seed `105049`.
  Rejected candidates mapped handshake replay cache, freshness, timestamp
  overflow, replay pressure, and open-cluster exposure to ISSUE-002,
  ISSUE-021, ISSUE-146, ISSUE-176, ISSUE-189, ISSUE-194, ISSUE-207,
  ISSUE-223, ISSUE-244, RC-1, RC-3, and RC-4; discovery/router sync caps,
  stale syncs, stopped tombstones, seed/non-seed cleanup, route loops,
  duplicate route entries, and path stability to ISSUE-001, ISSUE-003,
  ISSUE-004, ISSUE-167, ISSUE-170, ISSUE-211 through ISSUE-225, ISSUE-231,
  RC-6, and RC-7; control queues, requester backlog, pending connects, sync
  retry tasks, unauthenticated inbound admission, pending unicast acks, stream
  admission, setup/open timeouts, failed pipe delivery, and relay task pressure
  to ISSUE-011, ISSUE-012, ISSUE-013, ISSUE-056, ISSUE-117, ISSUE-149,
  ISSUE-156, ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-180, ISSUE-217,
  ISSUE-220, ISSUE-229, ISSUE-230, ISSUE-238, ISSUE-246, RC-3, RC-4, and
  RC-6; service local queues, full/closed channels, stale requesters,
  false-success delivery paths, alias waiters, and stale observer data to
  ISSUE-043, ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073,
  ISSUE-076, ISSUE-091, ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121,
  ISSUE-123 through ISSUE-126, ISSUE-217 through ISSUE-230, ISSUE-234,
  ISSUE-235, ISSUE-246, RC-1, RC-3, RC-5, and RC-6; pubsub pending RPC,
  responder binding, remote channel/member/tombstone caps, heartbeat chunk
  bounds, and local publisher/subscriber queues to ISSUE-020, ISSUE-039,
  ISSUE-048, ISSUE-080, ISSUE-108, ISSUE-115, ISSUE-116, ISSUE-155,
  ISSUE-205, ISSUE-206, ISSUE-228, ISSUE-236, ISSUE-240 through ISSUE-243,
  ISSUE-246, RC-1, RC-2, RC-3, and RC-6; and metrics/visualization,
  replicated-KV, refused connects, duplicate connection churn, endpoint drops,
  frame-size errors, and shutdown noise to existing observability,
  replicated-KV, and high-load fuzz/churn families under RC-3, RC-5, RC-6,
  and RC-7. No distinct score-80+ map, queue, task, cache, tombstone,
  admission, route/discovery cap, service-local queue, replicated-KV cap, or
  high-load bad-network resource issue had concrete failing-test evidence.
  Smallest future fix proposal if this family regresses: pin the exact map,
  queue, task, cache, tombstone, admission path, route/discovery sync,
  service-local buffer, replicated-KV state, or fuzz seed with one focused
  failing test, then patch only the failed cap, admission, eviction, timeout,
  cleanup, or backpressure boundary.
- Critical-only no-new cycle 15 after ISSUE-247 reviewed `src/lib.rs`,
  `src/quic.rs`, `src/peer.rs`, `src/secure.rs`, `src/discovery.rs`,
  `src/requester.rs`, README/examples, and config, discovery, connect, QUIC,
  readme, requester, handshake, unauthenticated inbound, address, duplicate,
  seed, advertise, and fuzz tests. Local config, discovery, connect, QUIC,
  readme, examples, and 48-node 2800-step adversarial fuzz seed `104049`
  checks passed; `static` had zero matching tests, so direct binding coverage
  came through connect/auth tests and reviewer cross-checks. `cargo
  check --examples` passed with existing warnings only. Reviewer `Curie the
  2nd` returned `NO_NEW_CRITICAL` after independently reviewing the same
  config/binding/endpoint slice, then passing address, peer-id,
  own-peer-address, duplicate, seed, advertise, QUIC, requester,
  inbound-handshake, unauthenticated, readme, source, forged, and examples
  checks; `binding/open_cluster` had zero matching tests, and one invalid
  combined Cargo command was rerun as separate valid filters. Rejected
  candidates mapped static inbound binding, wrong peer-id/address,
  self-connect, duplicate connect, reconnect coalescing, and open-cluster
  posture to ISSUE-189, ISSUE-194, ISSUE-244, RC-1, RC-4, RC-6, and cycles 10
  and 14; advertised-address validation, non-dialable addresses,
  seed/non-seed cleanup, stale discovery, stopped tombstones, duplicate
  discovery rows, local-peer advertisements, and configured seed behavior to
  ISSUE-001, ISSUE-003, ISSUE-004, ISSUE-167, ISSUE-170, ISSUE-211 through
  ISSUE-225, ISSUE-231, RC-6, and RC-7; QUIC/TLS setup, control-stream
  open/write timeouts, stream admission, unused unidirectional stream
  admission, unauthenticated inbound pressure, requester/control backpressure,
  and endpoint drop/refusal behavior to ISSUE-117, ISSUE-172, ISSUE-173,
  ISSUE-217, ISSUE-220 through ISSUE-223, ISSUE-238, ISSUE-246, RC-3, and
  RC-4; and source/remote authorization, forged source handling, peer-data
  peer-id validation, and direct-route ownership checks to ISSUE-014,
  ISSUE-015, ISSUE-017, ISSUE-018, ISSUE-039, ISSUE-115, ISSUE-116,
  ISSUE-197, ISSUE-226, RC-1, and RC-2. No distinct score-80+ public config,
  static binding, address parsing, advertise/seed cleanup, QUIC/TLS endpoint
  setup, unauthenticated admission, requester/connect, or high-load
  bad-network stability issue had concrete failing-test evidence. Smallest
  future fix proposal if this family regresses: pin the exact config value,
  claimed peer/address, advertised row, seed/tombstone transition, endpoint
  setup step, or admission queue with one focused failing test, then patch only
  the failed parse/validation, binding comparison, discovery admission,
  connection coalescing, setup timeout, or admission-cap boundary.
- Critical-only no-new cycle 14 after ISSUE-247 reviewed `src/secure.rs`,
  `src/discovery.rs`, `src/router.rs`, `src/lib.rs`, `src/ctx.rs`,
  `src/msg.rs`, `src/peer/peer_internal.rs`, pubsub, alias, requester, and
  related security, route, source, pubsub, stream, requester, discovery,
  alias, and fuzz tests. Local source, forged, identity, replay, route,
  pubsub, requester, and 46-node 2600-step adversarial fuzz seed `103049`
  checks passed. Reviewer `Euclid the 2nd` returned `NO_NEW_CRITICAL` after
  independently reviewing the same identity/source-binding and authorization
  slice, then passing secure, source, router, pubsub, discovery, stream,
  alias, requester, security, and 36-node 1600-step adversarial fuzz seed
  `103049`. Rejected candidates mapped handshake identity/role binding,
  replay, freshness, future/overflow timestamps, replay-cache pressure, and
  open-cluster exposure to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
  ISSUE-189, ISSUE-194, ISSUE-207, ISSUE-223, ISSUE-244, RC-1, RC-3, and
  RC-4; forged source/service delivery across unicast, acked unicast,
  broadcast, stream setup, relayed stream setup, and pubsub RPC responder
  binding to ISSUE-014, ISSUE-015, ISSUE-017, ISSUE-018, ISSUE-039,
  ISSUE-115, ISSUE-116, ISSUE-197, ISSUE-226, RC-1, and RC-2; discovery and
  route stale sync, stopped-peer tombstones, direct-vs-relay route priority,
  route resurrection, route loops, duplicate route entries, and seed/non-seed
  lifecycle behavior to ISSUE-001, ISSUE-003, ISSUE-004, ISSUE-167,
  ISSUE-170, ISSUE-211 through ISSUE-225, ISSUE-231, RC-6, and RC-7;
  queue/backpressure, requester/service liveness, local service delivery,
  direct/relay stream false success, full/closed channels, and failed pipe
  delivery to ISSUE-043, ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072,
  ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-100 through ISSUE-105, ISSUE-119,
  ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-217 through ISSUE-230,
  ISSUE-234, ISSUE-235, ISSUE-238, ISSUE-246, RC-3, RC-4, and RC-6; and
  pubsub membership, RPC responder checks, stale heartbeat/chunk handling,
  alias hint/found behavior, and stale alias lifecycle to existing
  pubsub/alias lifecycle, spoofing, resource, and responder-binding families
  noted in cycles 8, 9, 12, and 13. No distinct score-80+ handshake,
  identity/source-binding, route-sync authorization, pubsub/alias
  responder-binding, requester liveness, or high-load bad-network stability
  issue had concrete failing-test evidence. Smallest future fix proposal if
  this family regresses: pin the exact authenticated peer, claimed source,
  route sync, pubsub responder, alias reply, or fuzz seed with one focused
  failing test, then patch only the failed ingress/source normalization,
  direct-route ownership check, responder correlation, route admission, or
  lifecycle cleanup boundary.
- Critical-only no-new cycle 13 after ISSUE-247 reviewed `src/lib.rs`,
  `src/peer.rs`, `src/peer/peer_internal.rs`, `src/ctx.rs`,
  `src/requester.rs`, `src/service.rs`, metrics, visualization, alias,
  pubsub, replicated-KV service modules, `src/utils.rs`, and panic, drop,
  shutdown, lifecycle, stale, task, backpressure, and fuzz tests. Local panic,
  drop, closed, full, shutdown, lifecycle, stale, task, and 44-node 2400-step
  adversarial fuzz seed `102049` checks passed. Reviewer `Schrodinger the
  2nd` returned `NO_NEW_CRITICAL` after independently reviewing the same
  panic-boundary/task-lifecycle slice, then passing shutdown, dropped-service,
  lifecycle, requester, metrics, visualization, pubsub, replicate, and
  36-node 1800-step adversarial fuzz seed `102049`; one invalid combined
  Cargo command was rerun as separate valid filters. Rejected candidates mapped
  production panic/unwrap, serialization, overflow, and time arithmetic to
  ISSUE-024, ISSUE-094, ISSUE-097, ISSUE-098, ISSUE-174, ISSUE-189,
  ISSUE-194, ISSUE-207, RC-5, and RC-6; shutdown, graceful stop,
  stopped-peer propagation, non-seed cleanup, stale lifecycle, and
  `JoinHandle` abort/cleanup to ISSUE-001, ISSUE-004, ISSUE-170, ISSUE-215
  through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7; full/closed queues,
  stale requesters, dropped services, local service pressure, and false-success
  backpressure paths to ISSUE-043, ISSUE-052, ISSUE-053, ISSUE-060,
  ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-100 through ISSUE-105,
  ISSUE-119, ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-217 through
  ISSUE-230, ISSUE-234, ISSUE-235, ISSUE-246, RC-3, and RC-6; stream/open_bi
  task admission, setup/open timeout, retry task pressure, and failed pipe
  delivery to ISSUE-011, ISSUE-012, ISSUE-013, ISSUE-056, ISSUE-117,
  ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-180, ISSUE-217, ISSUE-220,
  ISSUE-229, ISSUE-230, ISSUE-238, RC-3, RC-4, and RC-6; metrics and
  visualization scan cleanup to existing observability spoofing/resource/
  lifecycle families, RC-1, RC-3, and RC-6; pubsub pending RPC, responder
  binding, queue, and requester lifecycle behavior to ISSUE-020, ISSUE-039,
  ISSUE-048, ISSUE-080, ISSUE-108, ISSUE-115, ISSUE-116, ISSUE-155,
  ISSUE-205, ISSUE-206, ISSUE-228, ISSUE-236, ISSUE-240 through ISSUE-243,
  ISSUE-246, RC-1, RC-3, and RC-6; and replicated-KV bounds, unsolicited
  responses, serialization failure, full-sync tasks, and graceful-stop cleanup
  to existing replicated-KV resource/divergence/overflow/lifecycle families,
  RC-3, RC-5, RC-6, and RC-7. No distinct score-80+ production panic,
  background task, graceful shutdown, queue backpressure, service lifecycle,
  replicated-KV cleanup, or high-load bad-network stability issue had concrete
  failing-test evidence. Smallest future fix proposal if this family regresses:
  pin the exact panic, task, queue, drop, or shutdown event with one focused
  failing test or fuzz seed, then patch only the failed boundary by replacing
  the panic with `Result` or logged drop, propagating timeout/backpressure,
  aborting or joining the owned task, or adding the missing lifecycle cleanup.
- Critical-only no-new cycle 12 after ISSUE-247 reviewed `Cargo.toml`,
  `Cargo.lock`, `.github/workflows/*`, `src/stream.rs`, `src/msg.rs`,
  `src/secure.rs`, `src/quic.rs`, `src/discovery.rs`, `src/router.rs`, and
  codec, object, handshake, discovery, QUIC, stream, overflow, bounded, and
  fuzz tests. Local stream, secure, discovery, codec, overflow, duplicate
  dependency tree, and 40-node 2000-step adversarial fuzz seed `101049` checks
  passed; `malformed` had zero matching tests; `cargo audit` was unavailable
  because `cargo-audit` is not installed. `cargo fmt --all -- --check` failed
  on import ordering/line wrapping in existing source, and `cargo clippy
  --all-targets --all-features -- -D warnings` failed on unused imports, dead
  code, and style lints; both were classified as non-critical CI hygiene, not
  score-80+ correctness/security evidence. Reviewer `Chandrasekhar the 2nd`
  returned `NO_NEW_CRITICAL` after independently reviewing the same slice, then
  passing codec, object, secure, malformed, router, discovery, QUIC, stream,
  overflow, bounded, duplicate dependency tree, and locked metadata checks, with
  `cargo-audit` also unavailable. Rejected candidates mapped framing/object
  size, bincode failures, malformed frames, and oversized frames to ISSUE-024,
  ISSUE-094, ISSUE-097, ISSUE-098, ISSUE-174, and RC-5; handshake replay,
  freshness, identity/role binding, timestamp overflow, future timestamps, and
  replay-cache pressure to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
  ISSUE-189, ISSUE-194, ISSUE-207, ISSUE-223, ISSUE-244, RC-1, RC-3, and
  RC-4; QUIC stream caps, setup/open-stream timeouts, and unauthenticated
  admission to ISSUE-117, ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220 through
  ISSUE-223, ISSUE-238, RC-3, and RC-4; route/discovery caps, stale syncs,
  stopped peers, direct-route priority, and route stability to ISSUE-001,
  ISSUE-003, ISSUE-004, ISSUE-167, ISSUE-170, ISSUE-211 through ISSUE-225,
  ISSUE-231, RC-6, and RC-7; and queue/backpressure plus service-id bounds to
  ISSUE-043, ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121,
  ISSUE-123 through ISSUE-126, ISSUE-217 through ISSUE-230, ISSUE-234,
  ISSUE-235, and ISSUE-246. No distinct score-80+ build/dependency,
  serialization/framing, handshake, QUIC, discovery, malformed-frame, or
  high-churn stability issue had concrete failing-test evidence. Smallest
  future fix proposal if this family regresses: pin the exact dependency,
  workflow gate, serialized object, frame, handshake token, or discovery sync
  with one focused failing test or reproducing command, then patch only the
  dependency pin/gate, size check, decode admission, timestamp/replay guard, or
  sync cap boundary that failed.
- Critical-only no-new cycle 11 after ISSUE-247 reviewed `src/router.rs`,
  `src/ctx.rs`, `src/lib.rs`, `src/msg.rs`, `src/peer/peer_internal.rs`, and
  route, cross-node, stream, unicast, security, and fuzz tests. Local router,
  cross-nodes, stream, unicast, and 42-node 2600-step adversarial fuzz seed
  `100049` checks passed. Reviewer `Huygens the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing the same route/path stability
  and pipe/unicast delivery slice, then passing route, router tests,
  open-stream, unicast, cross-nodes, stream, source, relay-stream, a zero-match
  `PeerData` filter, 36-node 1200-step adversarial fuzz seed `100049`, and
  36-node 1200-step sanitized churn fuzz seed `100050`. Rejected candidates
  mapped active path jumping, equal-cost route stability, tiny RTT jitter, and
  direct-route priority to ISSUE-003 and RC-7; failed pipes, `open_stream`
  false success, stream relay commit ordering, setup/open-stream timeout,
  connection admission, and queue pressure to ISSUE-011, ISSUE-012, ISSUE-013,
  ISSUE-056, ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-180,
  ISSUE-217, ISSUE-220, ISSUE-229, ISSUE-230, ISSUE-238, RC-3, RC-4, and
  RC-6; unicast and acked-unicast delivery, ingress-loop rejection,
  destination service closure, and local service queue pressure to ISSUE-119,
  ISSUE-224, ISSUE-225, ISSUE-229, ISSUE-230, RC-3, and RC-6; and stale or
  forged route sync, stopped-peer cleanup, discovery expiry, seed retention,
  and lifecycle route cleanup to ISSUE-001, ISSUE-004, ISSUE-167, ISSUE-170,
  ISSUE-211 through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7. No distinct
  score-80+ route stability, pipe/open-stream, unicast relay, stale route sync,
  or high-churn route delivery issue had concrete failing-test evidence.
  Smallest future fix proposal if this family regresses: pin the exact
  route-change, route-sync, send, or stream-open event with one focused
  failing test, then patch only the route selection hysteresis, route-sync
  admission, live-alias lookup, ack propagation, or stream relay commit
  boundary that failed.
- Critical-only no-new cycle 10 after ISSUE-247 reviewed `src/lib.rs`,
  `src/ctx.rs`, `src/peer.rs`, `src/peer/peer_internal.rs`,
  `src/requester.rs`, `src/neighbours.rs`, `src/stats.rs`, `src/utils.rs`,
  tests, README/examples, Cargo config, workflow config, and the issue
  ledgers. Local security, requester, readme, lifecycle, config,
  zero-tick-config, connect, shutdown, dropped-service, examples, and 38-node
  2200-step adversarial fuzz seed `99049` checks passed; the examples check
  emitted warnings only. Reviewer `Lorentz the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing the same slice, then passing
  shutdown, requester, service-id, readme, config, drop, closed, stale,
  lifecycle, examples, and 32-node 1200-step adversarial fuzz seed `99031`.
  Rejected candidates mapped public requester false success, stale requester
  after drop, and full/closed control queues to ISSUE-052, ISSUE-053,
  ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-234,
  ISSUE-235, ISSUE-246, and RC-6; shutdown/graceful stop, stopped-peer
  propagation, non-seed cleanup, and stale lifecycle to ISSUE-001, ISSUE-004,
  ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7;
  configured seed lifecycle, seed retention/removal, and seed advertisement to
  ISSUE-004, ISSUE-167, ISSUE-211 through ISSUE-213, and RC-7; connection
  admission, duplicate connects, self-connect, setup timeouts, and open-stream
  timeouts to ISSUE-117, ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220 through
  ISSUE-223, ISSUE-238, RC-3, and RC-4; service-id bounds, dropped services,
  local queue pressure, and full/closed-channel false success to ISSUE-043,
  ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121, ISSUE-123 through
  ISSUE-126, ISSUE-217 through ISSUE-230, ISSUE-234, ISSUE-235, ISSUE-246,
  RC-3, RC-4, and RC-6; README/examples open-cluster/default demo behavior to
  the existing public config/example family, ISSUE-244, and RC-1; and
  bad-network/high-load churn, abort/restart, refused connections, forged
  stop/raw frames, and endpoint drops to existing fuzz-cycle families, RC-3,
  RC-6, and RC-7. No distinct score-80+ public API/lifecycle/config/example
  issue had concrete failing-test evidence. Smallest future fix proposal if
  this family regresses: pin the public method/config/example or lifecycle
  event, reproduce the false success/panic/leak with one focused test, then
  patch only the public admission, handle-liveness, shutdown propagation, or
  config validation boundary that failed.
- Critical-only no-new cycle 9 after ISSUE-247 reviewed `src/service.rs`,
  alias, pubsub, replicated-KV, metrics, visualization service modules, their
  tests, fuzz coverage, and the issue ledgers. Local alias, metrics,
  visualization, pubsub, replicate, service, and 36-node 2000-step
  adversarial fuzz seed `98049` checks passed. Reviewer `Socrates the 2nd`
  returned `NO_NEW_CRITICAL` after independently reviewing the same
  service-layer slice, then passing alias, pubsub, replicate-KV, metrics,
  visualization, and 32-node 1200-step adversarial fuzz seed `98031`.
  Rejected candidates mapped alias control queues, stale find waiters,
  cache/hint bounds, lifecycle spoofing, shutdown handling, and alias peer
  cleanup to existing alias/resource/lifecycle families, RC-3, RC-6, and
  RC-7; pubsub stale membership, tombstones, heartbeat chunks, RPC
  correlation, queue/full-channel behavior, dropped requesters, and source
  role binding to ISSUE-020, ISSUE-039, ISSUE-048, ISSUE-080, ISSUE-108,
  ISSUE-115, ISSUE-116, ISSUE-155, ISSUE-205, ISSUE-206, ISSUE-228,
  ISSUE-236, ISSUE-240 through ISSUE-243, ISSUE-246, RC-1, RC-2, RC-3, and
  RC-6; replicated-KV remote caps, unsolicited responses, version overflow,
  snapshot/page validation, pending changed bounds, serialization failure, and
  graceful-stop cleanup to existing replicated-KV overflow, resource,
  divergence, and lifecycle families, RC-3, RC-5, RC-6, and RC-7; metrics and
  visualization unsolicited `Info`, non-collector `Scan` disclosure, stale
  info after disconnect, row caps, scan task accumulation, and base-service
  close handling to existing observability spoofing/resource/lifecycle
  families, RC-1, RC-3, and RC-6; and bad-network churn, forged service
  frames, queue pressure, graceful stop, duplicate/refused connections, and
  endpoint drops to existing fuzz-cycle families, ISSUE-001, ISSUE-004,
  ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7.
  No distinct score-80+ service-layer issue had concrete failing-test
  evidence. Smallest future fix proposal if this family regresses: pin the
  exact service message, peer/source, queue state, timeout, or fuzz seed;
  determine whether the missing guard belongs at service admission, role/source
  correlation, request lifecycle cleanup, remote snapshot validation, or
  observability scan gating; patch that boundary only and add one focused
  regression.
- Critical-only no-new cycle 8 after ISSUE-247 reviewed `src/secure.rs`,
  `src/stream.rs`, `src/msg.rs`, `src/quic.rs`, `src/lib.rs`,
  `src/peer.rs`, `src/peer/peer_internal.rs`, `src/tests/security.rs`,
  `src/tests/stream.rs`, `src/tests/fuzz.rs`, and the issue ledgers. Local
  secure, stream, codec, unauthenticated, object, malformed, overflow,
  inbound-handshake, and 40-node 2400-step adversarial fuzz seed `97049`
  checks passed; `cargo audit` was unavailable because `cargo-audit` is not
  installed. Reviewer `Feynman the 2nd` returned `NO_NEW_CRITICAL` after
  independently reviewing transport/auth/framing/resource-boundary paths, then
  passing secure, stream, security, bounded, malformed, and 34-node 1400-step
  adversarial fuzz seed `97031`. Rejected candidates mapped handshake
  replay/freshness/identity, role, timestamp overflow, replay pressure, and
  replay false-positive DoS to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
  ISSUE-189, ISSUE-194, ISSUE-207, ISSUE-223, ISSUE-244, RC-1, RC-3, and
  RC-4; malformed/oversized frames, bincode/object serialization,
  length-prefix caps, and service payload limits to ISSUE-024, ISSUE-094,
  ISSUE-097, ISSUE-098, ISSUE-174, and RC-5; QUIC caps, unauthenticated
  admission, setup/open_bi/control stream timeouts, endpoint drops, and
  shutdown noise to ISSUE-117, ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220
  through ISSUE-223, ISSUE-238, RC-3, and RC-4; peer-control queue pressure,
  ack caps/timeouts, local service delivery backpressure, service-id bounds,
  and closed/full queues to ISSUE-043, ISSUE-100 through ISSUE-105,
  ISSUE-119, ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-217 through
  ISSUE-230, ISSUE-234, ISSUE-235, ISSUE-238, ISSUE-246, RC-3, RC-4, and
  RC-6; forged source/identity binding after auth to ISSUE-014, ISSUE-015,
  ISSUE-017, ISSUE-018, ISSUE-039, ISSUE-115, ISSUE-116, ISSUE-197,
  ISSUE-226, RC-1, and RC-2; and high-load churn/noisy duplicate/refused/
  graceful-stop behavior to ISSUE-001, ISSUE-004, ISSUE-170, ISSUE-215
  through ISSUE-225, ISSUE-231, existing fuzz-cycle families, RC-3, RC-6, and
  RC-7. No distinct score-80+ transport/auth/framing/resource-boundary issue
  had concrete failing-test evidence. Smallest future fix proposal if this
  family regresses: pin the exact malformed object/frame/seed/action sequence,
  determine whether the missing guard is handshake/auth, frame/object cap,
  transport admission, queue result propagation, or source binding, then patch
  that boundary only and add a focused regression.
- Critical-only no-new cycle 7 after ISSUE-247 reviewed `src/router.rs`,
  `src/discovery.rs`, `src/lib.rs`, `src/ctx.rs`, `src/requester.rs`,
  `src/peer.rs`, `src/peer/peer_internal.rs`, `src/tests/fuzz.rs`,
  `src/tests/cross_nodes.rs`, `src/tests/stream.rs`,
  `src/tests/security.rs`, and the issue ledgers. Local route, discovery,
  stopped-peer, open-stream, 44-node 2200-step adversarial fuzz seed `96049`,
  and 44-node 2200-step sanitized churn fuzz seed `96050` checks passed.
  Reviewer `Planck the 2nd` returned `NO_NEW_CRITICAL` after independently
  reviewing route/discovery/network/send/open-stream paths, then passing
  `router::tests`, discovery, peer-stopped, open-stream, unicast, route,
  36-node 1200-step adversarial fuzz seed `96031`, and 36-node 1200-step
  sanitized churn seed `96032`. Rejected candidates mapped active path
  jumping, equal-cost jitter, tiny RTT jitter, direct route priority, and
  route advertisement loops to ISSUE-003 and RC-7; non-seed timeout/removal,
  seed retention/retry, stopped tombstones, and stale discovery
  advertisements to ISSUE-004, ISSUE-167, ISSUE-211 through ISSUE-213, and
  RC-7; graceful `PeerStopped`, stopped route cleanup/resurrection, stopped
  fanout/dedup/backpressure, and service disconnect visibility to ISSUE-001,
  ISSUE-004, ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6,
  and RC-7; failed pipes, direct and relayed `open_stream` false success,
  stream delivery commits, upstream/downstream stalls, local service queue
  pressure, and stream admission limits to ISSUE-011, ISSUE-012, ISSUE-013,
  ISSUE-056, ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-180,
  ISSUE-217, ISSUE-220, ISSUE-229, ISSUE-230, ISSUE-238, RC-3, RC-4, and
  RC-6; unicast and acked-unicast relay delivery, ingress-loop rejection,
  queue pressure, destination-service closure, and false success to
  ISSUE-119, ISSUE-224, ISSUE-225, ISSUE-229, ISSUE-230, RC-3, and RC-6;
  and duplicate connects, refused connections, shutdown/abort/restart churn,
  endpoint drops, and noisy bad-network behavior to existing fuzz-cycle
  families, RC-3, RC-6, and RC-7. No distinct score-80+ route/discovery/
  pipe-stability correctness, security, or high-load stability issue had
  concrete failing-test evidence. Smallest future fix proposal if this family
  regresses: pin the exact topology/seed/action sequence, prove whether the
  failure is route selection, stale discovery, peer-lifecycle cleanup, or
  stream/unicast admission, then patch only that boundary with a focused
  regression test before widening behavior.
- Critical-only no-new cycle 6 after ISSUE-247 added and reviewed
  `fuzz_random_adversarial_node_actions_must_not_panic_connection_tasks` in
  `src/tests/fuzz.rs`. The new harness honors `P2P_FUZZ_NODES`,
  `P2P_FUZZ_STEPS`, and `P2P_FUZZ_SEED`, and randomizes duplicate connects,
  awaited connects, unicast/broadcast/send/open-stream calls, forged
  `Unicast`, forged `UnicastWithAck`, stray `UnicastAck`, forged
  `PeerStopped`, self-connects, oversized raw frames, graceful stop, abort,
  restart, and reconnect churn. Local 12-node 250-step seed `95001` and
  40-node 1800-step seed `95049` adversarial fuzz runs passed. `cargo fmt
  --check` still reports pre-existing formatting drift in `src/discovery.rs`,
  `src/lib.rs`, `src/router.rs`, and `src/stream.rs`; the touched fuzz file
  was formatted with the repo rustfmt config. Reviewer `James the 2nd`
  returned `NO_NEW_CRITICAL` after independently reviewing fuzz and network
  paths, then passing 42-node 2000-step valid-action seed `95001`, 42-node
  2000-step sanitized churn seed `95002`, 36-node 1800-step churn seed
  `95003`, and 40-node 2000-step adversarial seed `95004`. Rejected
  candidates mapped graceful stop, forged `PeerStopped`, stopped cleanup,
  abort/restart, and bad-network churn to ISSUE-001, ISSUE-004, ISSUE-170,
  ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6, RC-7, and existing
  fuzz-cycle families; duplicate connects, stale connect events, false
  connect success, and self-connect behavior to ISSUE-052, ISSUE-053,
  ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-234,
  ISSUE-235, ISSUE-246, and RC-6; forged sources, raw unicast, acked unicast,
  and stray ack handling to ISSUE-014, ISSUE-015, ISSUE-017, ISSUE-018,
  ISSUE-039, ISSUE-115, ISSUE-116, ISSUE-197, ISSUE-226, RC-1, and RC-2;
  and invalid service IDs, oversized frames, open-stream timeouts, queue
  pressure, and full peer/local queues to ISSUE-024, ISSUE-094, ISSUE-097,
  ISSUE-098, ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121, ISSUE-123
  through ISSUE-126, ISSUE-174, ISSUE-217 through ISSUE-230, ISSUE-238,
  RC-3, RC-4, RC-5, and RC-6. No distinct score-80+ correctness, security,
  or stability issue had concrete failing-test evidence. Smallest future fix
  proposal if this harness exposes a failure: pin the exact seed/node/step
  tuple, minimize to the first failing action sequence, then add only the
  missing boundary guard, admission check, queue result propagation, or
  source/liveness validation proven by that minimized regression.
- Critical-only no-new cycle 5 after ISSUE-247 reviewed `src/msg.rs`,
  `src/stream.rs`, `src/secure.rs`, `src/quic.rs`, `src/utils.rs`,
  `src/lib.rs`, `src/ctx.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, service serialization/resource-limit paths,
  `src/tests/security.rs`, `src/tests/stream.rs`, `src/tests/fuzz.rs`, and
  the issue ledgers. Local secure, stream, codec, object, malformed,
  overflow, security, bounded, and 38-node 1600-step sanitized churn fuzz
  checks passed. Reviewer `Cicero the 2nd` returned `NO_NEW_CRITICAL` after
  independently running handshake, stream, object, codec, security,
  service-id, timeout, queue, malformed, 40-node 1800-step valid-action fuzz,
  and 36-node 1800-step sanitized churn fuzz slices. Rejected candidates
  mapped malformed/oversized frames, object length-prefix handling, bincode
  failures, and service payload size limits to ISSUE-024, ISSUE-094,
  ISSUE-097, ISSUE-098, ISSUE-174, and RC-5; handshake replay/freshness/
  identity, timestamp overflow, replay pressure, and unauthenticated
  admission to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176, ISSUE-189,
  ISSUE-194, ISSUE-207, ISSUE-244, RC-1, RC-3, and RC-4; QUIC setup, stalled
  control/app streams, timeout behavior, and open-stream false success to
  ISSUE-117, ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220 through ISSUE-223,
  ISSUE-238, RC-3, and RC-4; service IDs, stale/dropped requesters,
  duplicate services, closed/full queues, local delivery, and ack admission
  to ISSUE-043, ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073,
  ISSUE-076, ISSUE-091, ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121,
  ISSUE-123 through ISSUE-126, ISSUE-217 through ISSUE-230, ISSUE-234,
  ISSUE-235, ISSUE-238, ISSUE-246, RC-3, RC-4, and RC-6; source forgery and
  previous-hop binding to ISSUE-014, ISSUE-015, ISSUE-017, ISSUE-018,
  ISSUE-039, ISSUE-115, ISSUE-116, ISSUE-197, ISSUE-226, RC-1, and RC-2;
  and graceful stop, `PeerStopped`, bad-network churn, duplicate connections,
  and endpoint/drop noise to ISSUE-001, ISSUE-004, ISSUE-170, ISSUE-215
  through ISSUE-225, ISSUE-231, RC-3, RC-6, RC-7, and existing fuzz families.
  No distinct score-80+ panic/error-boundary, serialization/framing,
  handshake/auth, timeout-overflow, queue/resource, source-binding,
  graceful-stop, or high-load bad-network issue had concrete failing-test
  evidence. Smallest future fix proposal if a new issue appears in this
  family: add the narrow missing fallible-result handling, size/admission cap,
  checked arithmetic, or source/liveness validation at the exact boundary
  where the failing regression reproduces; avoid broad rewrites unless the
  failing proof crosses module boundaries.
- Critical-only no-new cycle 4 after ISSUE-247 reviewed `src/lib.rs`,
  `src/ctx.rs`, `src/service.rs`, `src/requester.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/peer/peer_alias.rs`,
  `src/tests/security.rs`, `src/tests/cross_nodes.rs`,
  `src/tests/stream.rs`, `src/tests/fuzz.rs`, and the issue ledgers. Local
  requester, service, service-id, shutdown, dropped, stale, queue, bounded,
  and 36-node 1500-step valid-action fuzz checks passed. Reviewer `Dewey the
  2nd` returned `NO_NEW_CRITICAL` after independently reviewing lifecycle and
  public API control paths and running requester, service, shutdown,
  service-id, dropped, stale, queue, and the same 36-node fuzz slice. Rejected
  candidates mapped graceful shutdown, `PeerStopped`, stopped-peer cleanup,
  full main queues, and stopped route cleanup to ISSUE-001, ISSUE-004,
  ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7;
  public requester/control queue full or closed behavior, stale `connect()`,
  duplicate connection attempts, false connect success, duplicate services,
  out-of-range service IDs, dropped/stale requesters, and closed receivers to
  ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076,
  ISSUE-091, ISSUE-234, ISSUE-235, ISSUE-246, and RC-6; local/peer queue
  pressure, control-frame backpressure, full service delivery,
  unicast/open-stream false success, and ack admission to ISSUE-043,
  ISSUE-100 through ISSUE-105, ISSUE-119, ISSUE-121, ISSUE-123 through
  ISSUE-126, ISSUE-217 through ISSUE-230, ISSUE-238, RC-3, RC-4, and RC-6;
  and bad-network churn, duplicate connections, endpoint drops, timed-out
  stream setup, and noisy closed-channel delivery to existing fuzz families
  and RC-3, RC-6, and RC-7. No distinct score-80+ lifecycle, public API
  control, service registration, stale-event, queue/backpressure,
  graceful-shutdown, duplicate/stopped-peer, false-success, or high-load
  bad-network churn issue had concrete failing-test evidence. Smallest future
  fix proposal if a new issue appears in this root-cause family: add the
  narrow missing admission/liveness/stale-event guard at the boundary where
  the false success or unbounded wait is observed, then pin it with a focused
  failing regression test before broad refactor.
- Critical-only no-new cycle 3 after ISSUE-247 reviewed
  `src/service/pubsub_service.rs`,
  `src/service/pubsub_service/publisher.rs`,
  `src/service/pubsub_service/subscriber.rs`, `src/tests/pubsub.rs`,
  `src/tests/security.rs`, `src/tests/fuzz.rs`, and the issue ledgers. Local
  pubsub, heartbeat, tombstone, chunk, RPC, bounded, full-channel, stale, and
  36-node 1500-step sanitized churn fuzz checks passed. Reviewer `Maxwell the
  2nd` returned `NO_NEW_CRITICAL` after independently running pubsub,
  heartbeat, RPC, tombstone, chunk, membership, requester, bounded, full,
  stale, backpressure, and the same 36-node fuzz slice. Rejected candidates
  mapped RPC answer correlation and forged/stale answers to ISSUE-020,
  ISSUE-115, ISSUE-116, ISSUE-236, and RC-2; requester/drop/service-drop and
  phantom channels to ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-108, ISSUE-234,
  ISSUE-235, ISSUE-246, and RC-6; membership authorization to ISSUE-039,
  ISSUE-048, RC-1, and RC-2; pending RPC caps, timeout overflow,
  no-destination behavior, internal queue pressure, and full local queues to
  ISSUE-043, ISSUE-100 through ISSUE-105, ISSUE-121, ISSUE-123 through
  ISSUE-126, ISSUE-178, ISSUE-228, ISSUE-231, ISSUE-240 through ISSUE-243,
  ISSUE-246, RC-3, RC-5, and RC-6; heartbeat omission/chunk/stale snapshot,
  channel/member/tombstone caps, and restart generation reset to ISSUE-080,
  ISSUE-155, ISSUE-205, ISSUE-206, ISSUE-228, ISSUE-240 through ISSUE-243,
  and RC-6; malformed serialization and oversized method/batch handling to
  ISSUE-094, ISSUE-097, ISSUE-098, ISSUE-174, and RC-5; and graceful stop,
  disconnect cleanup, full-channel behavior, and high-load churn to
  ISSUE-001, ISSUE-004, ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231,
  RC-3, RC-6, and RC-7. No distinct score-80+ pubsub protocol,
  state-machine, resource-bound, queue/backpressure, lifecycle, graceful-stop,
  or high-load churn issue had concrete failing-test evidence.
- Critical-only no-new cycle 2 after ISSUE-247 reviewed `src/router.rs`,
  `src/discovery.rs`, `src/neighbours.rs`, `src/lib.rs`, `src/ctx.rs`,
  `src/peer.rs`, `src/peer/peer_internal.rs`, `src/tests/discovery.rs`,
  `src/tests/security.rs`, and `src/tests/fuzz.rs`. Local router,
  discovery, `PeerStopped`, stale-event, integration discovery, and 36-node
  1500-step valid churn fuzz checks passed. Reviewer `Franklin the 2nd`
  returned `NO_NEW_CRITICAL` after independently running router, discovery,
  stopped, route, disconnect, stale, and 36-node churn fuzz slices. Rejected
  candidates mapped active-path jumping, equal-cost jitter, and direct-route
  priority to ISSUE-003 and RC-7; non-seed expiry/removal and seed retention
  to ISSUE-004, ISSUE-167, ISSUE-211 through ISSUE-213, and RC-7;
  graceful-stop notification, `PeerStopped` validation/dedup/fanout, stopped
  route resurrection, and shutdown cleanup to ISSUE-001, ISSUE-004,
  ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231, RC-3, RC-6, and RC-7;
  stale/duplicate connection events, stale sync/data/stats/disconnect, and
  full-queue lifecycle behavior to ISSUE-117, ISSUE-156, ISSUE-217 through
  ISSUE-225, ISSUE-230, ISSUE-238, RC-3, RC-4, and RC-6; and bad-network
  churn to existing fuzz cycles under RC-3/RC-6/RC-7. No distinct score-80+
  route, discovery, neighbour lifecycle, graceful-stop, shutdown, duplicate
  connection, stale-sync, or high-load churn issue had concrete failing-test
  evidence.
- Critical-only no-new cycle after ISSUE-247 reviewed `src/stream.rs`,
  `src/secure.rs`, `src/quic.rs`, `src/msg.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/ctx.rs`, `src/service.rs`,
  `src/tests/security.rs`, `src/tests/stream.rs`, fuzz tests, and ledgers.
  Local stream codec/object, handshake, service-id, stream setup, timeout,
  unicast, ack, malformed, inbound-handshake, unauthenticated, bounded, and
  34-node 1450-step valid-action fuzz checks passed. Reviewer `Gauss the 2nd`
  returned `NO_NEW_CRITICAL` after independently running secure, stream,
  QUIC, peer, security, object, service-id, full-channel, and fuzz slices.
  Rejected candidates mapped malformed/oversized frames and object limits to
  ISSUE-024/094/097/098/174 and RC-5; handshake replay/freshness/identity and
  replay pressure to ISSUE-002/021/146/176/189/194/207/244 and RC-1; QUIC
  admission and stalled setup/open timeouts to ISSUE-117/172/173/217/220
  through ISSUE-223/238 and RC-3/RC-4; service-id, stale requester, duplicate
  service, and closed/full queues to ISSUE-052/053/060/072/073/076/091/234/
  235/246 and RC-6; source forgery and previous-hop binding to ISSUE-014/015/
  017/018/039/115/116/197/226 and RC-1/RC-2; and graceful-stop/high-load churn
  to ISSUE-001/004/170/215 through ISSUE-225/231, RC-3/RC-6/RC-7, and fuzz
  cycles 20/24/28/31/32/34/36/38/39/40/41/42/43/44/45/47. No distinct
  score-80+ transport, auth, stream/framing, service-id, peer-control,
  source-binding, queue/backpressure, or high-load churn issue had concrete
  failing-test evidence.
- ISSUE-247, score 88: fixed replicated-KV cumulative full-sync snapshot
  staging. Root cause: `SyncFullState` capped each snapshot page but retained
  all accepted non-terminal pages in `staged_slots` until terminal commit
  without a total cap. A malicious authenticated peer could keep sending valid
  advancing continuation pages and grow memory unbounded before the snapshot
  completed. Smallest fix: add `MAX_STAGED_SNAPSHOT_SLOTS` and reject a page
  before mutation when `staged_slots.len() + snapshot.slots.len()` would exceed
  the cap. Verification:
  `cargo test full_sync_staged_snapshot_slots_must_be_bounded_across_pages --lib -- --nocapture`,
  `cargo test replicate_kv --lib -- --nocapture`,
  `cargo test snapshot --lib -- --nocapture`,
  `cargo test fetch_changed --lib -- --nocapture`,
  `cargo test full --lib -- --nocapture`,
  `cargo test stale --lib -- --nocapture --test-threads=1`, and
  `P2P_FUZZ_NODES=34 P2P_FUZZ_STEPS=1450 P2P_FUZZ_SEED=89049 cargo test --lib fuzz_random_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`.
- Critical-only no-new cycle 48 reviewed `src/service/alias_service.rs`,
  `src/service/metrics_service.rs`, `src/service/visualization_service.rs`,
  `src/stats.rs`, `src/service.rs`, `src/ctx.rs`, `src/requester.rs`,
  service registration/requester paths in `src/lib.rs`, and scoped alias,
  metrics, visualization, security, bounded, stale, full-channel, service-id,
  and fuzz coverage. Local alias, metrics, visualization, service-drop,
  requester, and bounded checks passed. Reviewer `Sartre the 2nd` returned
  `NO_NEW_CRITICAL` after independently running alias, metrics,
  visualization, service-drop, requester, service-id, stale, bounded,
  malformed, full-channel, and 38-node 1450-step valid-action fuzz slices.
  Rejected candidates mapped alias forged/stale lifecycle, hint poisoning,
  shutdown, bounded pending finds/waiters/cache, and control queue behavior to
  ISSUE-028/035/041/053/060/072/073/076/090/091/101/109/125/127/152/158/179/
  183/206/235/239 and RC-2/RC-3/RC-5/RC-6; metrics and visualization forged
  `Info`, scan disclosure, stale disconnect cleanup, oversized row batches,
  duplicate scan suppression, and full peer-control queues to ISSUE-064/068/
  078/079/102/104/105/128/129/165/200 through ISSUE-204/226/232 and
  RC-1/RC-3/RC-5/RC-6; service/requester stale liveness, duplicate or
  out-of-range service IDs, service-drop reuse, full local service queues, and
  false success to ISSUE-052/053/060/072/073/076/091/234/235/246 and RC-6;
  graceful stop, `PeerStopped`, disconnect notification under full queues, and
  churn behavior to ISSUE-001/004/170/215 through ISSUE-225/231,
  RC-3/RC-6/RC-7, and fuzz cycles 20/24/28/31/32/34/36/38/39/40/41/42/43/44/
  45/47. No distinct score-80+ alias, observability, stats, service-boundary,
  queue/backpressure, graceful-stop, or high-load churn issue had concrete
  failing-test evidence.
- Critical-only no-new cycle 47 reviewed `src/router.rs`,
  `src/discovery.rs`, `src/neighbours.rs`, `src/ctx.rs`, `src/peer.rs`,
  `src/peer/peer_alias.rs`, `src/peer/peer_internal.rs`, `src/stream.rs`,
  `src/service.rs`, `src/tests/discovery.rs`, `src/tests/cross_nodes.rs`,
  `src/tests/stream.rs`, and fuzz coverage. Local router, route, relay,
  stream, cross-node, discovery, peer-stopped, unicast, and 36-node
  1500-step valid-churn fuzz checks passed. Reviewer `Mendel the 2nd`
  returned `NO_NEW_CRITICAL` after independently running discovery,
  router, stream, unicast, relay, and 38-node 1450-step valid-action fuzz
  slices. Rejected candidates mapped active-path jumping/noisy route
  selection to ISSUE-003 and RC-7; pipe/relay unsuccessful or false-success
  behavior to ISSUE-011/012/013/056/117/149/156/169/180/217/220/229/230/
  238 and RC-3/RC-4/RC-6; relayed unicast delivery/backpressure to
  ISSUE-119/224/225/229/230 and RC-3/RC-6; discovery/graceful-stop/stale
  route behavior to ISSUE-001/004/051/063/164/167/170/211 through ISSUE-225/
  231 and RC-5/RC-6/RC-7; and high-load churn to fuzz cycles
  20/24/28/31/32/34/36/38/39/40/41/42/43/44/45 and RC-3/RC-6/RC-7. No
  distinct score-80+ route-selection, stream/pipe relay, unicast relay,
  discovery, graceful-stop, stale-route, or high-load churn issue had
  concrete failing-test evidence.
- Critical-only no-new cycle 46 reviewed `src/service/pubsub_service.rs`,
  `src/service/pubsub_service/publisher.rs`,
  `src/service/pubsub_service/subscriber.rs`, base service/requester
  interactions, pubsub tests, and fuzz coverage. Local pubsub, RPC,
  heartbeat, tombstone, chunk, membership, service-drop, and 38-node
  1450-step valid-action fuzz checks passed. Reviewer `Ampere the 2nd`
  returned `NO_NEW_CRITICAL` after independently running pubsub, heartbeat,
  RPC, requester, bounded, full, stale, backpressure, and 34-node 1400-step
  sanitized churn fuzz slices. Rejected candidates mapped to pubsub RPC
  answer correlation, forged/stale answers, and local requester binding
  issues ISSUE-020/115/116/236, RC-2; publisher/subscriber requester
  lifecycle and phantom-channel cleanup ISSUE-072/073/076/108/234/235/246,
  RC-6; remote membership authorization ISSUE-039/048, RC-1/RC-2; pending
  RPC caps, timeout overflow, no-destination behavior, internal control
  backlog, and full local queues ISSUE-043/100 through ISSUE-105/121/123
  through ISSUE-126/178/228/231/240 through ISSUE-243/246, RC-3/RC-5/RC-6;
  heartbeat stale cleanup, chunk sequence/reassembly, tombstone caps, and
  restart generation handling ISSUE-080/155/205/206/228/240 through
  ISSUE-243, RC-6; malformed serialization and oversized method/batch caps
  ISSUE-094/097/098/174, RC-5; and graceful stop/high-load bad-network churn
  ISSUE-001/004/170/215 through ISSUE-225/231, RC-3/RC-6/RC-7, and fuzz
  cycles 20/24/28/31/32/34/36/38/39/40/41/42/43/44/45. No distinct
  score-80+ pubsub protocol, state-machine, resource-cap, queue/full-channel,
  lifecycle, or high-load churn issue had concrete failing-test evidence.
- Critical-only no-new cycle 45 reviewed concurrency/task lifecycle and
  closed/full-channel behavior in `src/lib.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/peer/peer_alias.rs`, `src/ctx.rs`,
  `src/service.rs`, service requesters, security/cross-node/stream/pubsub
  tests, and fuzz coverage. Local task, backpressure, peer-stopped, closed,
  delivery, ack, disconnect, stale, and 38-node 1400-step valid-churn fuzz
  checks passed. Reviewer `Newton the 2nd` returned `NO_NEW_CRITICAL` after
  independently running shutdown, drop, requester, channel, closed, full,
  task, metrics, service_drop, stale, backpressure, and 34-node 1400-step
  sanitized churn fuzz slices. Rejected candidates mapped to concurrency/task
  lifecycle, pending sync retry, peer-control closure, helper task drop, and
  main-loop closure issues ISSUE-117/156/217/220/221/222/223/238,
  RC-3/RC-4/RC-6, and cycles 20/24/32/34/36/38/42/44; closed/full channels,
  local delivery pressure, acked-unicast pressure, full peer-control queues,
  and false success to ISSUE-043/100 through ISSUE-105/119/121/123 through
  ISSUE-126/218 through ISSUE-230/234 through ISSUE-236/240 through
  ISSUE-243/246, RC-3/RC-4/RC-6, and cycles 30/33/34/36/38/40/42/44;
  requester/service/drop liveness to ISSUE-052/053/060/072/073/076/091/
  234/235/246 and RC-6; metrics cleanup, stale stats, scan retry/
  backpressure, and duplicate scan suppression to ISSUE-064/068/102/104/105/
  128/129/165/200 through ISSUE-204/226/232, RC-1/RC-3/RC-6, and cycles
  29/34/36/38/40/43; graceful shutdown, stopped-peer delivery, disconnect
  notification under full queues, route/service cleanup, duplicate connection
  cleanup, and stop-delivery backpressure to ISSUE-001/004/170/215 through
  ISSUE-225/231, RC-3/RC-6/RC-7, and cycles 18/24/32/34/36/38/39/41/42/44;
  and high-load bad-network churn to fuzz cycles 20/24/28/31/32/34/36/38/39/
  40/41/42/43/44. No distinct score-80+ concurrency, lifecycle, closed/full
  channel, spawned retry task, requester/drop liveness, metrics cleanup,
  graceful shutdown, or high-load backpressure issue had concrete failing-test
  evidence.
- Critical-only no-new cycle 44 reviewed public network lifecycle, config,
  examples, and malformed external-input behavior in `src/lib.rs`,
  `src/requester.rs`, `src/quic.rs`, `src/secure.rs`, `src/msg.rs`,
  `src/stream.rs`, `src/peer.rs`, `examples/simple.rs`,
  `examples/benchmark.rs`, `examples/kv.rs`,
  `examples/readme_getting_started.rs`, `README.md`, and security/readme
  tests. Local focused security, secure-handshake, stream codec, QUIC, readme,
  example-build, and 36-node 1300-step valid-action fuzz checks passed.
  Reviewer `Hume the 2nd` returned `NO_NEW_CRITICAL` after independently
  running secure, QUIC, codec/object/stream, requester, unauthenticated,
  shutdown/graceful, inbound, readme, examples, security, service-id, and
  30-node 1200-step sanitized churn fuzz slices; `malformed` matched no
  tests. Rejected candidates mapped to README/example open-cluster/default
  credential demos, ISSUE-244, RC-1, and cycles 19/23/35/37; requester,
  service-id, stale requester, duplicate service, and public control behavior
  to ISSUE-052/053/060/072/073/076/091/234/235/246 and RC-6; QUIC setup,
  unauthenticated admission, bidirectional/unidirectional stream limits,
  stalled setup, and main control-stream timeout behavior to ISSUE-117/172/
  173/217/220/221/222/223/238 and RC-3/RC-4; shared-key freshness, replay,
  timestamp overflow/future skew, peer-id/role binding, and inbound static
  binding to ISSUE-002/021/146/176/189/194/207/244 and RC-1; malformed
  frames, bincode/object size limits, oversized payloads, length-prefix
  overflow, and out-of-range service IDs to ISSUE-024/052/053/060/091/094/
  097/098/174/234/235 and RC-5/RC-6; source forgery and previous-hop binding
  to ISSUE-014/015/017/018/039/115/116/197/226 and RC-1/RC-2; shutdown,
  graceful stop, stopped-peer tombstones, stale lifecycle events, duplicate
  cleanup, and stop-delivery backpressure to ISSUE-001/004/170/215 through
  ISSUE-225/231 and RC-3/RC-6/RC-7; and high-load bad-network churn to fuzz
  cycles 20/24/28/31/32/34/36/38/39/40/41/42/43. No distinct score-80+
  public lifecycle, config/example, malformed input, handshake, QUIC setup,
  requester/control, service-id, shutdown, or high-load churn issue had
  concrete failing-test evidence.
- Critical-only no-new cycle 43 reviewed stateful-service and observability
  behavior in `src/service/replicate_kv_service.rs`,
  `src/service/replicate_kv_service/local_storage.rs`,
  `src/service/replicate_kv_service/remote_storage.rs`,
  `src/service/replicate_kv_service/messages.rs`,
  `src/service/alias_service.rs`, `src/service/metrics_service.rs`,
  `src/service/visualization_service.rs`, and `src/stats.rs`. Local
  replicated-KV working-state repair tests passed. Reviewer `Arendt the 2nd`
  returned `NO_NEW_CRITICAL` after running focused replicated-KV, metrics,
  visualization, alias, stale, bounded, scan, and 32-node 1300-step sanitized
  churn fuzz slices. Rejected candidates mapped to replicated-KV issues
  ISSUE-023/025/027/031/034/037/038/059/081 through ISSUE-089/110/131/138/
  141/154/171/175/184/186/196/233/237/245, RC-3/RC-5/RC-6, and cycles
  28/36/40; metrics and visualization issues ISSUE-064/068/078/079/102/104/
  105/128/129/165/200 through ISSUE-204/226/232, RC-1/RC-3/RC-5/RC-6, and
  cycles 29/34/36/38/40; alias issues ISSUE-028/035/041/053/060/072/073/076/
  090/091/101/109/125/127/152/158/179/183/206/235/239, RC-2/RC-3/RC-5/RC-6,
  and cycles 30/35/36/38/40; stats ingestion to ISSUE-064/068/226/232,
  RC-1/RC-6, and cycles 29/34/35/38/40; and high-load stale/disconnect/
  backpressure churn to fuzz cycles 20/24/28/31/32/34/36/38/39/40/41/42 and
  RC-3/RC-6/RC-7. No distinct score-80+ replicated-state, alias lifecycle,
  metrics/visualization ingestion, stale disconnect, queue/backpressure,
  stats, or high-load bad-network churn issue had concrete failing-test
  evidence.
- Critical-only no-new cycle 42 reviewed transport, stream, and backpressure
  behavior in `src/peer.rs`, `src/peer/peer_internal.rs`,
  `src/peer/peer_alias.rs`, `src/stream.rs`, `src/msg.rs`, `src/ctx.rs`,
  `src/service.rs`, `src/tests/stream.rs`, `src/tests/cross_nodes.rs`,
  `src/tests/security.rs`, and `src/tests/fuzz.rs`. Focused stream, unicast,
  backpressure, codec, object, malformed, control, queue, a 36-node 1500-step
  sanitized churn fuzz seed, a 32-node 1300-step steady fuzz seed, and a
  28-node 1000-step valid-action fuzz seed passed locally; `malformed`
  matched no tests. Reviewer `Halley the 2nd` returned `NO_NEW_CRITICAL`
  after independently reviewing transport, stream setup/relay, peer control,
  unicast ack, local service delivery, codec limits, and high-load
  backpressure. Rejected candidates mapped to ISSUE-001/002/117/172/173/189/
  194/219/221 through ISSUE-223/244, RC-1/RC-3/RC-4/RC-6, and cycles
  33/34/36/38/39 for transport/handshake admission, identity binding,
  duplicate/coalesced connections, pending unauthenticated inbound caps, and
  setup timeouts; ISSUE-011/012/013/056/117/149/156/169/180/217/220/238,
  RC-3/RC-4/RC-5/RC-6, and cycles 32/33/34/36/38/40 for stream setup,
  relayed open success, upstream/downstream commit ordering, route-loop relay
  rejection, stalled setup responses, deferred delivery reservations, and
  closed/full destination service behavior; ISSUE-043, ISSUE-100 through
  ISSUE-105, ISSUE-117/119/121, ISSUE-123 through ISSUE-126, ISSUE-156,
  ISSUE-217 through ISSUE-230, ISSUE-238, RC-3/RC-4/RC-6, and cycles
  30/32/33/34/36/38/40 for unicast ack, pending ack caps, local service
  delivery queues, closed receivers, full service queues, relay reporting, and
  false success; ISSUE-014/015/018/017, RC-1/RC-2, and cycles 33/39 for
  forged unicast/stream source normalization and authenticated previous-hop
  binding; ISSUE-024/094/097/098/174, RC-5, and cycles 33/36/40 for frame
  caps, object length-prefix limits, bincode errors, oversized payloads, and
  malformed raw frames; ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-231,
  RC-3/RC-6/RC-7, and cycles 20/24/32/34/36/38/39/41 for `PeerStopped`
  delivery during full queues, graceful shutdown forwarding, stopped-peer
  route resurrection, and peer-control closure noise; and fuzz cycles
  20/24/28/31/32/34/36/38/39/40/41 for high-load bad-network refused connects,
  deadlines, duplicate closures, transient route churn, endpoint-drop logs,
  and internal `open_bi` answer races. No distinct score-80+ transport,
  stream setup/relay, peer-control, unicast-ack, local-delivery, codec-limit,
  malformed-payload, queue/backpressure, or high-load bad-network churn issue
  had concrete failing-test evidence.
- Critical-only no-new cycle 41 reviewed route, discovery, and
  graceful-shutdown behavior in `src/router.rs`, `src/discovery.rs`,
  `src/neighbours.rs`, `src/lib.rs`, `src/ctx.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/tests/discovery.rs`,
  `src/tests/cross_nodes.rs`, `src/tests/fuzz.rs`, and stopped-peer security
  coverage. Focused discovery, router, graceful non-seed shutdown,
  peer-stopped, stopped-peer, a 34-node 1400-step sanitized churn fuzz seed,
  and a 30-node 1200-step steady fuzz seed passed locally; the initial
  uppercase `PeerStopped` filter matched no tests and was rerun with lowercase
  filters. Reviewer `Turing the 2nd` returned `NO_NEW_CRITICAL` after
  independently reviewing route/discovery/graceful-shutdown/stopped-peer
  lifecycle behavior under bad network and high churn. Rejected candidates
  mapped to ISSUE-003/009/010/063/103/164/211 through ISSUE-214, RC-5,
  RC-7, and cycles 32/36/39 for route jitter, direct-route priority, stale
  sync, route loop suppression, advertise validity, and active-path stability;
  ISSUE-001/004/215 through ISSUE-225/231, RC-6, and cycles
  18/24/32/34/36/38/39 for graceful stop, stopped-peer lifecycle, stale route
  resurrection, duplicate connection cleanup, and stop forwarding; ISSUE-170,
  ISSUE-215 through ISSUE-225, RC-3/RC-6, and cycles 20/24/32/34/36/38/39
  for `PeerStopped` forgery, dedup, stale-event confusion, and queue
  backpressure; ISSUE-167, ISSUE-211 through ISSUE-213, RC-7, and cycles
  32/37/39 for seed retention, non-seed timeout/removal, non-dialable
  advertise rejection, and discovery backlog coalescing; and fuzz cycles
  20/24/28/31/32/34/36/38/39/40 for high-load route/discovery churn. No
  distinct score-80+ route, discovery, graceful-shutdown, stopped-peer,
  seed/non-seed lifecycle, active-path, stale-sync, queue/backpressure, or
  high-load bad-network churn issue had concrete failing-test evidence.
- Critical-only no-new cycle 40 reviewed service payload, RPC, and resource
  boundaries in `src/service.rs`, `src/ctx.rs`, pubsub service and handles,
  alias/metrics/visualization services, replicated-KV service/storage/messages,
  `src/msg.rs`, `src/stream.rs`, and service-focused tests. Focused pubsub,
  replicated-KV, bounded, RPC, alias, metrics, visualization, malformed,
  service-id, codec, queue, serial stale-event filters, and a 32-node
  1300-step sanitized churn fuzz seed passed locally. Reviewer `Peirce the
  2nd` returned `NO_NEW_CRITICAL` after independently reviewing base service
  and context boundaries, pubsub service/handles, alias/metrics/visualization
  services, replicated-KV service/storage/messages, message framing, stream
  object helpers, and scoped tests. Rejected candidates mapped to
  ISSUE-052/053/060/072/073/076/091/234/235/246, RC-6, and cycles
  33/35/36/38 for service-id bounds, dropped/closed service false success,
  stale requesters, and duplicate service handles; ISSUE-024/094/097/098/174,
  RC-5, and cycles 33/36 for payload/frame oversize, object length prefixes,
  bincode malformed decode, and peer-message caps; pubsub issues
  ISSUE-020/039/043/080/094/100/115/116/121/123 through ISSUE-126,
  ISSUE-155, ISSUE-178, ISSUE-205/206, ISSUE-228, ISSUE-231, ISSUE-234
  through ISSUE-236, ISSUE-240 through ISSUE-243, ISSUE-246, RC-2/RC-3/RC-4/
  RC-5/RC-6, and cycles 20/24/31/32/34/36/38 for RPC responder binding,
  pending RPC caps, method caps, queue/full fanout, requester drop, internal
  control backlog, heartbeat chunking, stale snapshot cleanup, member/channel/
  tombstone caps, and generation behavior; alias issues ISSUE-028/035/041/
  053/060/072/073/076/090/091/101/109/125/127/152/158/179/183/206/235/239,
  RC-2/RC-3/RC-5/RC-6, and cycle 30; metrics/visualization issues
  ISSUE-064/068/078/079/102/104/105/128/129/165/200 through ISSUE-204,
  ISSUE-226/232, RC-1/RC-3/RC-5/RC-6, and cycles 29/34/36/38; replicated-KV
  issues ISSUE-023/025/027/031/034/037/038/059/081 through ISSUE-089/110/
  131/138/141/154/171/175/184/186/196/233/237/245, RC-3/RC-5/RC-6, and
  cycles 28/36; and fuzz cycles 20/24/28/31/32/34/36/38/39 for high-load
  service/resource churn. No distinct score-80+ service payload, RPC,
  resource-bound, malformed-input, stale-response, queue/backpressure,
  replicated-state, scan-ingestion, or high-load churn issue had concrete
  failing-test evidence.
- Critical-only no-new cycle 39 reviewed authentication, identity, and
  source-binding surfaces in `src/secure.rs`, `src/quic.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/msg.rs`, `src/ctx.rs`, `src/router.rs`,
  `src/discovery.rs`, `src/tests/security.rs`, `src/tests/cross_nodes.rs`,
  and stream/security source-binding tests. Focused auth, source, forged,
  handshake, peer-id, connect, duplicate, and a 30-node 1200-step sanitized
  churn fuzz seed passed locally. Reviewer `Parfit the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing `SharedKeyHandshake`
  peer-id/role binding, timestamp skew/timeout, replay cache and bounded
  replay pressure, QUIC setup, authenticated `PeerId` to `ConnectionId`
  binding, inbound binding checks, source normalization, stale connection
  events, message claims, broadcast dedup, route/discovery sync admission,
  stopped tombstones, and stale route cleanup. Rejected candidates mapped to
  ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176, ISSUE-207, ISSUE-244, RC-1,
  and cycle 33 for shared-key freshness/replay/role/peer-id binding;
  ISSUE-189, ISSUE-194, ISSUE-244, cycle 33, and existing inbound
  handshake/connect tests for inbound static/open-cluster peer binding and
  wrong-address/wrong-id rejection; ISSUE-014, ISSUE-015, ISSUE-018,
  ISSUE-017, RC-2, cycle 33, and existing source-binding tests for forged
  unicast/broadcast/stream source claims; ISSUE-001, ISSUE-004, ISSUE-215
  through ISSUE-225, ISSUE-231, RC-6, and cycles 18/24/32/34/36/38 for
  PeerStopped forgery, stale connection confusion, stop-route resurrection,
  duplicate connection cleanup, and graceful-stop forwarding; ISSUE-003,
  ISSUE-009, ISSUE-010, ISSUE-063, ISSUE-103, ISSUE-164, ISSUE-211 through
  ISSUE-213, RC-5, RC-7, and cycles 28/32/37 for route/discovery admission,
  caps, stale sync, direct-route priority, non-dialable advertise addresses,
  stopped tombstones, and seed behavior; and fuzz cycles 20/24/32/34/36/38
  for high-load authentication/identity churn. No distinct score-80+
  authentication, identity, source-binding, admission, stale-event,
  route/discovery, or high-load churn issue had concrete failing-test
  evidence.
- Critical-only no-new cycle 38 reviewed cancellation, drop, and task
  lifetime surfaces in `src/lib.rs`, `src/peer.rs`,
  `src/peer/peer_internal.rs`, `src/ctx.rs`, `src/quic.rs`,
  `src/service.rs`, `src/service/metrics_service.rs`, and
  `src/service/visualization_service.rs`. Focused shutdown/drop/channel
  closure tests, stale requester/service liveness tests, graceful/disconnect
  tests, task-related tests, and a 28-node 1100-step sanitized churn fuzz
  seed passed locally; an initial concurrent `stale` filter port collision
  was rerun serially and passed. Reviewer `Ohm the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing detached spawn/task
  ownership, shutdown and graceful-shutdown behavior, endpoint close behavior,
  pending sync task aborts, `PeerConnectionInternal` helper-task `Drop`,
  connection close paths, peer-control channel closure, stale requester/service
  liveness, retry/timer loops, scan response/broadcast tasks, stream
  open/setup, pending unicast ack expiry, peer disconnect notifications, and
  high-load churn. Rejected candidates mapped to ISSUE-215 through ISSUE-225,
  ISSUE-231, RC-6, and cycles 18/24/32/34/36 for graceful stop/stopped-peer
  behavior; RC-3/RC-4, ISSUE-117, ISSUE-156, ISSUE-217 through ISSUE-225,
  ISSUE-230, ISSUE-238, and cycles 20/24/32/34/36 for peer-control,
  queues/backpressure, pending ack, sync retry, and stream setup behavior;
  ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-234, ISSUE-235, ISSUE-246, RC-6,
  and cycles 24/30/34/35 for requester/service/network drop semantics;
  ISSUE-064, ISSUE-068, ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-226,
  ISSUE-232, RC-3/RC-6, and cycles 29/34/36 for metrics/visualization scan,
  retry, stale peer cleanup, and base-service close behavior; ISSUE-117,
  ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220, ISSUE-238, RC-3/RC-4, and
  cycles 33/34/36 for QUIC accept/setup and connection close concerns; and
  fuzz cycles 20/24 plus critical-only cycles 32/34/36 for high-load churn,
  refused connections, duplicate closures, endpoint-drop noise, and live work
  after shutdown/drop. No distinct score-80+ cancellation/drop/task-lifetime/
  live-work-after-shutdown issue had concrete failing-test evidence.
- Critical-only no-new cycle 37 reviewed build, package, feature-gating,
  platform, and release-profile surfaces in `Cargo.toml`, package contents,
  examples, README build contract, public exports in `lib.rs`, crate-local
  `cfg`/feature assumptions, dev/prod dependency split, release behavior, and
  downstream consumer compileability. `cargo metadata`, `cargo package
  --list`, `cargo package --allow-dirty`, `cargo tree -e features`, release
  lib/example checks, all-target checks, a downstream release consumer smoke,
  release panic tests, release overflow tests, and release README tests passed
  with existing warnings only. Reviewer `Singer the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing dependency/dev-dependency
  split, empty feature set, feature unification, package metadata and
  contents, examples, README build commands, public exports, release-profile
  panic behavior, downstream consumer compileability, and platform/cfg
  surfaces. Rejected candidates mapped to cycles 19/23/35, ISSUE-244, and
  RC-1 for demo certs, shared-key strings, open-cluster examples, and package
  assumptions; cycle 35 for dependency/default-feature and downstream
  consumer concerns; ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073,
  ISSUE-076, ISSUE-091, ISSUE-234, ISSUE-235, ISSUE-246, RC-6, and cycles
  33/35 for public API/config/service misuse; cycle 36 plus panic/overflow
  tests for release panic/overflow concerns; and ISSUE-211 through ISSUE-213,
  RC-7, and cycle 32 for seed/discovery config behavior. Missing manifest
  repository/homepage/documentation metadata and packaged `Cargo.toml.orig`
  had no concrete score-80+ failing-test evidence. No distinct score-80+
  build, package, feature-gating, platform, downstream consumer, all-target,
  or release-profile issue had concrete failing-test evidence.
- Critical-only no-new cycle 36 reviewed production panic, overflow, and
  resource-bound surfaces in non-test code across `secure.rs`, `router.rs`,
  `discovery.rs`, `stream.rs`, `ctx.rs`, `peer.rs`, `peer_internal.rs`,
  service modules, `utils.rs`, and `quic.rs`. Focused panic, overflow,
  bounded, queue, malformed, serialize, deserialize, timeout, and a 24-node
  900-step sanitized churn fuzz seed passed locally. Reviewer `Faraday the
  2nd` returned `NO_NEW_CRITICAL` after independently reviewing non-test
  panic/unwrap/expect surfaces, resource bounds in alias/pubsub/metrics/
  visualization/replicated-KV/discovery/router/peer task paths, and bincode
  serialization/deserialization across peer frames, service payloads, object
  helpers, pubsub, alias, metrics/visualization, and replicated-KV messages.
  Rejected candidates mapped to ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
  ISSUE-207, ISSUE-244, RC-1, and cycle 33 for handshake/timestamp/replay;
  ISSUE-024, ISSUE-094, ISSUE-097, ISSUE-098, ISSUE-174, RC-5, and cycle 33
  for frame/object bincode caps; ISSUE-003, ISSUE-009, ISSUE-010, ISSUE-063,
  ISSUE-103, ISSUE-164, ISSUE-211 through ISSUE-213, RC-5, RC-7, and cycle
  32 for router/discovery overflow and active-path behavior; ISSUE-052,
  ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091,
  ISSUE-234, ISSUE-235, ISSUE-246, RC-6, and cycles 33/35 for public service
  API misuse; ISSUE-043, ISSUE-100 through ISSUE-105, ISSUE-117, ISSUE-119,
  ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-156, ISSUE-217 through
  ISSUE-225, ISSUE-228, ISSUE-230, ISSUE-238, RC-3, RC-4, RC-6, and cycles
  30/33/34 for queue/backpressure/pending state; ISSUE-023, ISSUE-025,
  ISSUE-027, ISSUE-031, ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-059,
  ISSUE-081 through ISSUE-089, ISSUE-110, ISSUE-131, ISSUE-138, ISSUE-141,
  ISSUE-154, ISSUE-233, ISSUE-237, ISSUE-245, and cycle 28 for replicated-KV
  snapshot/repair/cap/serialization behavior; pubsub issues through
  ISSUE-246 plus RC-2/RC-3/RC-5/RC-6 and cycle 31 for pubsub bounds/RPC/
  shutdown cleanup; and ISSUE-064, ISSUE-068, ISSUE-078, ISSUE-079,
  ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-128, ISSUE-129, ISSUE-165,
  ISSUE-200 through ISSUE-204, ISSUE-226, ISSUE-232, RC-1, RC-3, RC-5,
  RC-6, and cycle 29 for metrics/visualization bounds. No distinct score-80+
  production panic, overflow, serialization, resource-bound, malformed-input,
  deadline, or high-load churn issue had concrete failing-test evidence.
- Critical-only no-new cycle 35 reviewed public API, config, and docs/spec
  boundaries in `README.md`, `Cargo.toml`, examples, `readme.rs`, `utils.rs`,
  `stats.rs`, `requester.rs`, `service.rs`, `ctx.rs`, and public exports in
  `lib.rs`. `cargo check --lib`, an external downstream consumer compile
  check, `cargo check --examples`, focused readme, zero-value, address,
  service-id, dropped-service, requester tests, and a 24-node 900-step
  valid-action fuzz seed passed. Reviewer `Hilbert the 2nd` returned
  `NO_NEW_CRITICAL` after independently reviewing README/spec promises,
  package/dependency assumptions, config validation, dev cert assumptions,
  static/open-cluster binding behavior, address parsing/display,
  service/requester drop behavior, and metrics/stats helpers. Rejected
  candidates mapped to cycles 19/23, ISSUE-244, and RC-1 for demo certs,
  shared-key strings, and explicit open-cluster examples; ISSUE-001,
  ISSUE-004, ISSUE-170, ISSUE-189, ISSUE-194, ISSUE-223, ISSUE-244, RC-1,
  and cycle 33 for inbound binding defaults and identity admission; ISSUE-054,
  ISSUE-211 through ISSUE-213, RC-7, and cycle 32 for zero config,
  advertise/seed validity, and discovery defaults; ISSUE-153, ISSUE-189,
  ISSUE-194, RC-1, and RC-6 for peer-id/address mismatch and self/duplicate
  connects; ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-234, ISSUE-235,
  ISSUE-246, RC-6, and cycles 24/30/34 for requester/service drop and connect
  backlogs; ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-234,
  ISSUE-235, RC-6, and cycle 33 for service-id misuse; and ISSUE-064,
  ISSUE-068, ISSUE-226, ISSUE-232, RC-1, RC-6, and cycles 29/34 for
  stats/metrics helper behavior. No distinct score-80+ public API, config,
  docs/spec, dependency, or helper issue had concrete failing-test evidence.
- Critical-only no-new cycle 34 reviewed runtime lifecycle and QUIC boundary in
  `lib.rs`, `quic.rs`, `requester.rs`, `neighbours.rs`, and `peer.rs`.
  Focused shutdown, requester, stale, duplicate, control, QUIC,
  peer-connected, peer-disconnected, unauthenticated-admission, and a 28-node
  1000-step churn fuzz seed passed. Reviewer `McClintock the 2nd` returned
  `NO_NEW_CRITICAL` after independently covering QUIC, graceful, stopped,
  unauthenticated, disconnected, backpressure, metrics, cross-node, and two
  24-node 900-step churn fuzz seeds. Rejected candidates mapped to ISSUE-215
  through ISSUE-225, ISSUE-231, RC-6, and cycles 18/24/32 for graceful
  shutdown, `PeerStopped`, seed/non-seed lifecycle, and stop tombstones;
  ISSUE-014, ISSUE-015, ISSUE-018, ISSUE-039, ISSUE-115, ISSUE-116,
  ISSUE-197, ISSUE-226, ISSUE-232, and RC-1 for forged or stale runtime event
  authority checks; ISSUE-221 through ISSUE-223 and cycle 33 for
  unauthenticated admission and authenticated alias release; ISSUE-117,
  ISSUE-156, ISSUE-172, ISSUE-173, ISSUE-217, ISSUE-220, ISSUE-238, RC-3,
  RC-4, and cycle 33 for QUIC stream caps, stream admission, stalled setup,
  and false stream success; ISSUE-218 through ISSUE-225, ISSUE-227,
  ISSUE-230, RC-3, and RC-6 for control/sync/local-delivery backpressure and
  ack bounds; ISSUE-226, ISSUE-232, cycle 29, cycle 33, and metrics stale
  tests for metrics/reporting admission; and cycles 18/24/32/33 plus fuzz
  coverage for high-load churn. No distinct score-80+ runtime lifecycle, QUIC
  boundary, main-loop event admission, or bad-network churn issue had concrete
  failing-test evidence.
- Critical-only no-new cycle 33 reviewed auth, framing, and service boundaries
  in `secure.rs`, `msg.rs`, `stream.rs`, `peer.rs`, `peer_internal.rs`,
  `lib.rs`, `service.rs`, and `ctx.rs`. Focused handshake, service-id, codec,
  object, stream, source-binding, unauthenticated-admission, and a 24-node
  900-step churn fuzz seed passed. Reviewer `Leibniz the 2nd` returned
  `NO_NEW_CRITICAL`. Rejected candidates mapped to ISSUE-146, ISSUE-176,
  ISSUE-207, ISSUE-244, and RC-1 for handshake freshness/replay/role binding;
  ISSUE-001, ISSUE-004, ISSUE-170, ISSUE-189, ISSUE-194, ISSUE-223,
  ISSUE-244, and RC-1 for self/third-party identity and inbound binding;
  ISSUE-117, ISSUE-156, ISSUE-217, ISSUE-220, ISSUE-221 through ISSUE-223,
  ISSUE-238, RC-3, and RC-4 for unauthenticated/stream admission and false
  setup success; ISSUE-024, ISSUE-094, ISSUE-097, ISSUE-098, ISSUE-174, and
  RC-5 for frame/object caps and malformed raw messages; ISSUE-052,
  ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091,
  ISSUE-234, ISSUE-235, RC-5, and RC-6 for service-id bounds; ISSUE-043,
  ISSUE-119, ISSUE-123 through ISSUE-126, ISSUE-218 through ISSUE-225,
  ISSUE-230, RC-3, and RC-6 for service queue false success and
  backpressure/ack bounds; and ISSUE-014, ISSUE-015, ISSUE-018, ISSUE-039,
  ISSUE-115, ISSUE-116, ISSUE-197, ISSUE-226, and RC-1 for authenticated
  source forgery. No distinct score-80+ auth, framing, service-boundary, or
  stream-admission issue had concrete failing-test evidence.
- Critical-only no-new cycle 32 reviewed route/discovery active-path and
  stream/pipe stability in `router.rs`, `discovery.rs`, `lib.rs`,
  `peer_internal.rs`, and `stream.rs`. Focused route, discovery, active-path,
  direct-route, peer-stopped, stale-peer, discovery-timeout, graceful-shutdown,
  duplicate, relayed-open-stream, relay-orphan-delivery, relay-loop,
  open-stream, send-relay, unicast-relay, sync-backpressure,
  peer-connected-backpressure, stream, relay, seed, stopped, and a 28-node
  churn fuzz seed passed. Reviewer `Laplace the 2nd` returned
  `NO_NEW_CRITICAL`. Rejected candidates mapped to ISSUE-003/RC-7 for active
  path flapping and direct-route priority, ISSUE-215 through ISSUE-225,
  ISSUE-231, and RC-6 for stale route resurrection and lifecycle cleanup,
  ISSUE-004, ISSUE-167, ISSUE-211 through ISSUE-213, and RC-7 for seed versus
  non-seed discovery lifecycle, ISSUE-063, ISSUE-103, ISSUE-164, RC-5, and
  RC-7 for route/discovery sync validation and metric/timestamp bounds,
  ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-217, ISSUE-220, ISSUE-229,
  ISSUE-230, RC-3, and RC-4 for relay loops, false stream/unicast success, and
  orphan pipe delivery, plus ISSUE-218, ISSUE-219, ISSUE-224, ISSUE-225,
  RC-3, and RC-6 for queue backpressure cleanup. No distinct score-80+
  route/discovery active-path or stream/pipe failure had concrete failing-test
  evidence.
- Critical-only no-new cycle 31 reviewed pubsub lifecycle and RPC behavior in
  `pubsub_service.rs`, `publisher.rs`, and `subscriber.rs`. Focused pubsub,
  heartbeat, RPC, stale, tombstone, bounded, queue, pending, disconnect,
  shutdown, malformed-payload filters, chunk, drop, and a 24-node valid-action
  fuzz seed passed. Reviewers `Avicenna the 2nd` and `Bacon the 2nd` returned
  `NO_NEW_CRITICAL`. Rejected candidates mapped to RC-2 for stale lifecycle
  and RPC-correlation issues, RC-3/RC-5 for queue, channel, member, tombstone,
  heartbeat, method, and pending-RPC caps, RC-6 for requester drop, service
  drop, shutdown, graceful-stop, and disconnect cleanup, ISSUE-020,
  ISSUE-039, ISSUE-043, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-080,
  ISSUE-094, ISSUE-100, ISSUE-102 through ISSUE-105, ISSUE-115, ISSUE-116,
  ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-155, ISSUE-178, ISSUE-205,
  ISSUE-206, ISSUE-228, ISSUE-231, ISSUE-234 through ISSUE-236,
  ISSUE-240 through ISSUE-243, ISSUE-246, and existing pubsub lifecycle,
  bounded, chunked-heartbeat, requester-drop, shutdown, malformed-input, and
  fuzz coverage. No distinct score-80+ pubsub lifecycle or RPC failure had
  concrete failing-test evidence.
- Critical-only no-new cycle 30 reviewed alias service and requester lifecycle
  behavior in `alias_service.rs`, `requester.rs`, and `service.rs`. Focused
  alias, requester, stale, bounded, pending, shutdown, dropped, unsolicited,
  malformed-payload filters, and a 24-node churn fuzz seed passed. Reviewer
  `Pauli the 2nd` returned `NO_NEW_CRITICAL`. Rejected candidates mapped to
  RC-2 for `Found`/`NotFound` correlation and stale alias lifecycle issues,
  RC-3/RC-5 for pending find, waiter, hint, cache, and queue caps, RC-6 for
  requester liveness plus shutdown/drop/disconnect cleanup, ISSUE-028,
  ISSUE-035, ISSUE-041, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073,
  ISSUE-076, ISSUE-090, ISSUE-091, ISSUE-101, ISSUE-109, ISSUE-125,
  ISSUE-127, ISSUE-152, ISSUE-158, ISSUE-179, ISSUE-183, ISSUE-206,
  ISSUE-215 through ISSUE-225, ISSUE-234, ISSUE-235, ISSUE-239, and existing
  alias/requester stale, bounded, shutdown, unsolicited, and drop coverage. No
  distinct score-80+ alias or requester lifecycle failure had concrete
  failing-test evidence.
- Critical-only no-new cycle 29 reviewed metrics and visualization ingestion in
  `metrics_service.rs` and `visualization_service.rs`. Focused metrics,
  visualization, scan, info, stale, bounded, disconnect, malformed-payload
  filters, and a 24-node high-load fuzz seed passed. Reviewer `Einstein the
  2nd` returned `NO_NEW_CRITICAL`. Rejected candidates mapped to RC-1/RC-2 for
  unauthorized scan disclosure and responder-correlation issues, RC-3 for
  scan-response backpressure and duplicate task coalescing, RC-5 for row and
  retained-peer caps, RC-6 for stale disconnect cleanup and base-service close
  behavior, ISSUE-053, ISSUE-061, ISSUE-062, ISSUE-064, ISSUE-068, ISSUE-078,
  ISSUE-079, ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-128, ISSUE-129,
  ISSUE-165, ISSUE-200 through ISSUE-204, ISSUE-226, ISSUE-232, and existing
  metrics/visualization stale, bounded, scan, and malformed-input coverage. No
  distinct score-80+ metrics or visualization ingestion failure had concrete
  failing-test evidence.
- Critical-only no-new cycle 28 reviewed replicated-KV state sync and repair in
  `replicate_kv_service.rs`, `local_storage.rs`, `remote_storage.rs`, and
  `messages.rs`. Focused replicated-KV, snapshot, `fetch_changed`,
  `fetch_snapshot`, graceful-stop deletion, malformed-payload filters, and a
  24-node high-load fuzz seed passed. Rejected candidates mapped to RC-2 for
  stale/unsolicited RPC response and request-correlation issues, RC-3/RC-5 for
  resource caps and bounded snapshot/repair handling, RC-6 for disconnect and
  tombstone cleanup, ISSUE-023, ISSUE-027, ISSUE-031, ISSUE-038, ISSUE-059,
  ISSUE-081 through ISSUE-089, ISSUE-110, ISSUE-131, ISSUE-138, ISSUE-141,
  ISSUE-154, ISSUE-205, ISSUE-206, ISSUE-233, ISSUE-237, ISSUE-245, and
  existing replicated-KV overflow, snapshot, repair, and graceful-stop deletion
  tests. No distinct score-80+ replicated-KV failure had concrete failing-test
  evidence.
- Fix phase status: ISSUE-001, ISSUE-003, ISSUE-004, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-002, ISSUE-008, ISSUE-009, ISSUE-010, ISSUE-011, ISSUE-012, ISSUE-013, ISSUE-014, ISSUE-015, ISSUE-017, ISSUE-020, ISSUE-021, ISSUE-023, ISSUE-024, ISSUE-025, ISSUE-027, ISSUE-033, ISSUE-034, ISSUE-039, ISSUE-045, ISSUE-046, ISSUE-047, ISSUE-048, ISSUE-055, ISSUE-059, ISSUE-103, ISSUE-110, ISSUE-111, ISSUE-115, ISSUE-116, ISSUE-117, ISSUE-118, ISSUE-119, ISSUE-120, ISSUE-122, ISSUE-123,
  ISSUE-124, ISSUE-125, ISSUE-126, ISSUE-127, ISSUE-128, ISSUE-129, ISSUE-130,
  ISSUE-131, ISSUE-132, ISSUE-133, ISSUE-134, ISSUE-135, ISSUE-136, ISSUE-137,
  ISSUE-140, ISSUE-143, ISSUE-145, ISSUE-147, ISSUE-148, ISSUE-150, ISSUE-151,
  ISSUE-152, ISSUE-153, ISSUE-154, ISSUE-155, ISSUE-156, ISSUE-157, ISSUE-158,
  ISSUE-159, ISSUE-160, ISSUE-161, ISSUE-163, ISSUE-164, ISSUE-053, ISSUE-062, ISSUE-063, ISSUE-086, ISSUE-087, ISSUE-088, ISSUE-089, ISSUE-090, ISSUE-091, ISSUE-092, ISSUE-093, ISSUE-139, ISSUE-146, ISSUE-168, ISSUE-170,
  ISSUE-094, ISSUE-095, ISSUE-096,
  ISSUE-149, ISSUE-169, ISSUE-174, ISSUE-176, ISSUE-177, ISSUE-078, ISSUE-079, ISSUE-080, ISSUE-081, ISSUE-082, ISSUE-083, ISSUE-084, ISSUE-085, ISSUE-181, ISSUE-184, ISSUE-185, ISSUE-189, ISSUE-190, ISSUE-191, ISSUE-192, ISSUE-193,
  ISSUE-194, ISSUE-195, ISSUE-196, ISSUE-197, ISSUE-198, ISSUE-199,
  ISSUE-200, ISSUE-201, ISSUE-202, ISSUE-203, ISSUE-204, ISSUE-205, ISSUE-206, ISSUE-207, ISSUE-208, ISSUE-097, ISSUE-098, ISSUE-099, ISSUE-100, ISSUE-101, ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-106, ISSUE-107, ISSUE-108, ISSUE-109, ISSUE-112, ISSUE-018, ISSUE-022, ISSUE-061, ISSUE-042, ISSUE-016, ISSUE-073, ISSUE-072, ISSUE-076, ISSUE-052, ISSUE-030, and ISSUE-060 have focused
  fixes committed.
  ISSUE-211 is fixed by gossiping configured seeds in outbound discovery sync
  and dropping stale relayed broadcast copies after mesh convergence.
  ISSUE-212 is fixed by placing local advertise before learned remotes in
  capped outbound discovery syncs.
  ISSUE-213 is fixed by deduplicating configured seed entries by peer id before
  applying the outbound sync cap.
  Post-ISSUE-213 high-load steady and sanitized-churn fuzz passed as no-new
  cycle 1.
  ISSUE-214 is fixed by prioritizing direct routes before learned routes in
  capped outbound router syncs.
  Post-ISSUE-214 high-load steady and sanitized-churn fuzz passed as no-new
  cycle 1.
  ISSUE-215 is fixed by marking `PeerStopped` dedup only after local main-loop
  admission succeeds, so backpressured retries are not suppressed.
  ISSUE-216 is fixed by clearing stale `PeerStopped` dedup state when a new
  connection lifecycle is registered for the same peer id.
  ISSUE-217 is fixed by rejecting an already-stopped upstream relay setup side
  before opening downstream, then delaying upstream success until downstream
  stream setup is accepted.
  ISSUE-218 is fixed by moving inbound sync main-queue retry into one bounded
  coalescing worker per connection, keeping the peer read loop unblocked.
  ISSUE-219 is fixed by bounding post-auth live main-control writes and closing
  stalled connections instead of parking the peer read/control loop.
  ISSUE-220 is fixed by bounding accept-side stream setup response writes so
  stalled peers cannot hold all
  accept permits for one authenticated connection when the peer stops reading
  `StreamConnectRes`.
  ISSUE-221 is fixed by closing and exiting the connection task after an
  admitted graceful stop so a stopped peer cannot continue sending traffic over
  the old authenticated connection.
  ISSUE-222 is fixed by unregistering the `SharedCtx` connection alias during
  accepted graceful-stop handling, so local fanout cannot target a peer after it
  has already been reported disconnected.
  ISSUE-223 is fixed by excluding already-authenticated inbound aliases from the
  unauthenticated admission cap while `PeerConnected` is backpressured.
  ISSUE-224 is fixed by moving fire-and-forget local service delivery behind a
  bounded per-connection worker so full service queues do not park the peer
  read loop before later graceful-stop frames.
  ISSUE-225 is fixed by queuing acked local unicast delivery into the same
  bounded per-connection worker and emitting `UnicastAck` only after local
  service admission succeeds or fails.
  ISSUE-226 is fixed by `a736bae`: metrics and visualization now deny inbound
  broadcast scan responses by default unless the sender is configured in
  `trusted_scan_collectors`; legitimate collectors opt in with
  `with_trusted_scan_collectors(...)`.
  ISSUE-227 is fixed by `c0d7616`: awaited broadcast fanout now admits
  peer-alias sends concurrently under the existing bounded timeout instead of
  waiting one timeout per congested peer.
  ISSUE-228 is fixed by `f9fd337`: outbound pubsub heartbeats are chunked to
  `MAX_HEARTBEAT_CHANNELS_PER_BATCH`, matching the inbound heartbeat cap.
  ISSUE-229 is fixed by `f87c6dc`: public relayed `send_unicast` now uses
  end-to-end `UnicastWithAck` propagation instead of reporting success after
  first-hop enqueue.
  ISSUE-230 is fixed by `2358c31`: per-connection pending unicast
  acknowledgements are capped before writing another `UnicastWithAck` frame.
  ISSUE-231 is fixed by `ed8f4fb`: absent-channel pubsub leaves now write
  bounded remote-role tombstones so delayed older joins cannot resurrect remote
  publisher/subscriber membership.
  ISSUE-232 is fixed by `8339384`: metrics clears pending responder and
  scan-response state on `PeerDisconnected`, so delayed stale `Info` cannot
  publish metrics for a disconnected peer.
  ISSUE-233 is fixed by `c7aa3f5`: replicated-KV rejects response-only
  unicast messages from unknown peers before creating remote state or queuing
  full-sync work.
  ISSUE-234 is fixed by `54f1118`: duplicate or rejected service creation now
  returns a disabled service handle whose outbound service/requester paths fail
  instead of publishing as an unregistered owner.
  ISSUE-235 is fixed by `5b0fc47`: alias registration now reports bounded
  control admission failure instead of returning a dead-on-arrival
  `AliasGuard`.
  ISSUE-236 is fixed by `13f3a67`: pubsub publish/feedback RPC deadline
  scheduling now uses checked `Instant` arithmetic, so huge caller-supplied
  timeouts cannot panic the service loop.
  ISSUE-237 is fixed by `abe7e37`: replicated-KV full-sync snapshot pages now
  reject continuation cursors that do not advance past the last accepted key.
  ISSUE-156 regression evidence is fixed by `fedfa0e`: relayed stream delivery
  now uses a two-phase downstream commit so final services do not receive a
  relayed stream until the original upstream setup acknowledgement succeeds.
  ISSUE-238 is fixed by `d340a7b`: deferred local stream delivery reservations
  are capped per connection to `SERVICE_CHANNEL_SIZE - 1`, so one authenticated
  peer cannot fill every local service queue slot by forging relay-only
  `defer_delivery` requests while legitimate relayed final-hop capacity and
  reserve-before-success semantics are preserved.
  ISSUE-239 is fixed, score 66: alias scan-mode `Found` responses now require
  active advertised `(AliasId, PeerId)` lifecycle proof before completing a
  lookup or caching the responder. Scan responders send `NotifySet` before
  `Found`, preserving legitimate scan discovery while rejecting standalone
  forged `Found` replies. Reviewers `Jason the 2nd` and `Zeno the 2nd`
  accepted the issue and implementation. Verification:
  `RUST_LOG=error cargo test scan_found_must_require_advertised_alias_lifecycle --lib -- --nocapture`
  and `RUST_LOG=error cargo test alias --lib -- --nocapture`.
  Post-ISSUE-239 no-new cycle 1 passed route, discovery, unicast, stale
  pubsub, full-sync, fetch-changed, and pending-resource focused regressions
  under reviewer `Descartes the 2nd`; no distinct issue survived duplicate
  mapping.
  Post-ISSUE-239 no-new cycle 2 passed focused handshake, inbound-handshake,
  requester, stream-codec, QUIC unidirectional-stream, serial stream, and
  eight-node fuzz checks under reviewer `Meitner the 2nd`; no distinct issue
  survived duplicate mapping.
  ISSUE-240 is fixed, score 64: pubsub multi-batch outbound heartbeats now use
  `HeartbeatChunk { channels, is_last }`, receivers accumulate seen channels
  per peer across chunks, and omitted-role cleanup runs only after the final
  chunk. `Heartbeat(Vec<_>)` remains the complete-snapshot message for empty
  and single-batch heartbeats, and the new enum variant is appended to preserve
  existing bincode tags. Reviewer `Archimedes the 2nd` accepted the issue.
  Verification:
  `RUST_LOG=error cargo test pubsub_chunked_heartbeat_must_not_remove_roles_from_previous_chunk --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_heartbeat --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_outbound_heartbeat_batches_must_respect_inbound_cap --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs`, and
  `git diff --check`.
  ISSUE-241 is fixed, score 60: pubsub chunked heartbeat pending state is now
  keyed by a heartbeat `snapshot_id`. A later multi-batch heartbeat resets any
  stale pending seen-channel accumulator from an abandoned earlier snapshot,
  so omitted-role cleanup uses only channels seen in the current complete
  snapshot. Reviewer `Erdos the 2nd` accepted the issue and confirmed the red
  regression. Verification:
  `RUST_LOG=error cargo test pubsub_new_chunked_heartbeat_must_not_reuse_stale_pending_seen_channels --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_chunked_heartbeat_must_not_remove_roles_from_previous_chunk --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_outbound_heartbeat_batches_must_respect_inbound_cap --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_heartbeat --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs`, and
  `git diff --check`.
  ISSUE-242 is fixed, score 62: pubsub chunked heartbeats now carry
  `chunk_index` and `chunks_count`, and pending receiver state tracks received
  chunk indexes. A final chunk triggers omitted-role cleanup only after every
  chunk in that snapshot has arrived, so a lone final chunk cannot remove
  roles from an incomplete view. Reviewer `Carver the 2nd` accepted the issue
  and confirmed the red regression. Verification:
  `RUST_LOG=error cargo test pubsub_final_heartbeat_chunk_without_prior_chunks_must_not_remove_omitted_roles --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_new_chunked_heartbeat_must_not_reuse_stale_pending_seen_channels --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_chunked_heartbeat_must_not_remove_roles_from_previous_chunk --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_outbound_heartbeat_batches_must_respect_inbound_cap --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_heartbeat --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs`, and
  `git diff --check`.
  ISSUE-243 is fixed, score 66: pubsub chunked heartbeats now reject
  `chunks_count` values above `MAX_HEARTBEAT_CHUNKS_PER_SNAPSHOT`, so a peer
  cannot grow `PendingHeartbeatChunks.seen_chunks` by sending endless sparse
  empty chunks for one snapshot. Reviewer `Aristotle the 2nd` accepted the
  issue as distinct from ISSUE-240, ISSUE-241, and ISSUE-242 and supplied the
  red regression. Verification:
  `RUST_LOG=error cargo test pubsub_sparse_heartbeat_chunk_indexes_must_not_grow_pending_unbounded --lib`,
  `RUST_LOG=error cargo test pubsub_chunked_heartbeat --lib`,
  `RUST_LOG=error cargo test pubsub_heartbeat --lib`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs`, and
  `git diff --check`.
  ISSUE-244 is fixed, score 72: `SharedKeyHandshake` now records accepted
  token hashes in a compact rotating replay window in addition to the bounded
  exact replay cache, so evicting the oldest exact entry under cache pressure
  cannot make a still-live token replayable. Reviewer `Locke the 2nd`
  accepted the issue as distinct from ISSUE-146, ISSUE-176, ISSUE-207, and
  ISSUE-166 and supplied the red regression. Verification:
  `cargo test handshake_replay_must_not_be_accepted_after_replay_cache_eviction_pressure --lib`,
  `cargo test secure::tests --lib`,
  `rustfmt --edition 2021 --check src/secure.rs`, and `git diff --check`.
  ISSUE-245 is fixed, score 58: replicated-KV initial full sync now stages
  snapshot pages until the terminal page is accepted, matching replacement
  resync atomicity. A partial first page no longer mutates `ctx.slots` or emits
  visible `KvEvent::Set`; stale terminal pages after a continuation request
  leave the initial view empty instead of exposing an incomplete prefix.
  Reviewer `Galileo the 2nd` accepted the issue as distinct from ISSUE-171,
  ISSUE-237, ISSUE-131, ISSUE-143, ISSUE-083, ISSUE-038, and ISSUE-037.
  Verification:
  `RUST_LOG=error cargo test initial_full_sync_must_not_emit_partial_snapshot_before_terminal_page --lib`,
  `RUST_LOG=error cargo test replicate_kv --lib`,
  `RUST_LOG=error cargo test full_sync --lib`,
  `RUST_LOG=error cargo test snapshot --lib`,
  `rustfmt --edition 2021 --check src/service/replicate_kv_service/remote_storage.rs`,
  and `git diff --check`.
  Post-ISSUE-245 no-new cycle 1 reviewed transport/request framing, stream
  setup, service/requester admission, peer alias control, unicast acking, and
  local service delivery under reviewer `Aquinas the 2nd`. `cargo test stream
  --lib -- --nocapture` passed. Rejected candidates mapped to ISSUE-011,
  ISSUE-012, ISSUE-014, ISSUE-018, ISSUE-028, ISSUE-053, ISSUE-060,
  ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-117, ISSUE-119,
  ISSUE-125, ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-180, ISSUE-217,
  ISSUE-220, ISSUE-224, ISSUE-225, ISSUE-229, ISSUE-230, ISSUE-234,
  ISSUE-235, ISSUE-238, RC-3, RC-4, and RC-6; no distinct issue survived
  duplicate mapping.
  ISSUE-246 is fixed, score 54: pubsub publisher/subscriber constructors now
  remember whether their bounded lifecycle registration control was admitted.
  Requester actions from never-registered handles fail immediately, and drops
  do not enqueue destruction controls for handles that never entered service
  state. Verification:
  `RUST_LOG=error cargo test pubsub_publisher_registration_overflow_must_not_return_silent_handle --lib -- --nocapture`,
  `RUST_LOG=error cargo test pubsub_internal_control --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs src/service/pubsub_service/publisher.rs src/service/pubsub_service/subscriber.rs`,
  and `git diff --check`.
  Post-ISSUE-246 no-new critical cycle 1 reviewed shared-key handshake,
  inbound identity binding, inbound `PeerMessage` trust, service-id bounds,
  stream setup/admission, unicast ack backpressure, sync/discovery caps,
  routing/relay loops, and high-load queue behavior under reviewer
  `Linnaeus the 2nd`. `RUST_LOG=error cargo test handshake --lib`,
  `RUST_LOG=error cargo test inbound_handshake --lib`,
  `RUST_LOG=error cargo test stream --lib`, and
  `RUST_LOG=error cargo test service_id --lib` passed. The ledger check found
  21 score-80+ issues and all 21 are marked fixed; no new critical issue was
  accepted.
  Post-ISSUE-246 no-new critical cycle 2 reviewed pubsub and replicated-KV
  state machines under reviewer `Russell the 2nd`. `RUST_LOG=error cargo test
  pubsub --lib`, `RUST_LOG=error cargo test replicate_kv --lib`,
  `RUST_LOG=error cargo test full_sync --lib`, and `RUST_LOG=error cargo test
  rpc --lib` passed. Pubsub RPC binding, membership churn, tombstones, channel
  caps, heartbeat chunks, replicated-KV unsolicited responses, full-sync
  validation, fetch-changed correlation, pending caps, liveness refresh, and
  high-version arithmetic mapped to existing fixed issues or noncritical
  families; no new critical issue was accepted.
  Post-ISSUE-246 no-new critical cycle 3 reviewed route, discovery, and alias
  lifecycle behavior under reviewer `Averroes the 2nd`.
  `RUST_LOG=error cargo test route --lib`, `RUST_LOG=error cargo test
  discovery --lib`, `RUST_LOG=error cargo test alias --lib`, and
  `RUST_LOG=error cargo test peer_stopped --lib` passed. Route poisoning,
  local/self routes, loops, direct-route preference, discovery stale/seed
  retention, tombstones, non-dialable addresses, alias cache/lifecycle,
  pending find bounds, request correlation, admission, and cleanup mapped to
  existing fixed issues or noncritical families; no new critical issue was
  accepted.
  Post-ISSUE-246 no-new critical cycle 4 reviewed metrics, visualization,
  requester, and service-boundary behavior under reviewer `Pasteur the 2nd`.
  `RUST_LOG=error cargo test metrics --lib`,
  `RUST_LOG=error cargo test visualization --lib`,
  `RUST_LOG=error cargo test requester --lib`, and
  `RUST_LOG=error cargo test service --lib` passed. Scan trust, responder
  correlation, stale disconnect cleanup, service registration, requester
  liveness, service-id validation, all-failed fanout reporting, graceful
  shutdown, and backpressure behavior mapped to existing fixed issues or
  noncritical families; no new critical issue was accepted.
  Post-ISSUE-246 no-new critical cycle 5 reviewed transport, framing,
  handshake, neighbour, stats, and network-lifecycle behavior under reviewer
  `Bohr the 2nd`. `RUST_LOG=error cargo test quic --lib`,
  `RUST_LOG=error cargo test stream --lib`, `RUST_LOG=error cargo test
  handshake --lib`, `RUST_LOG=error cargo test inbound_handshake --lib`,
  `RUST_LOG=error cargo test connect --lib`, and `RUST_LOG=error cargo test
  stats --lib` passed. QUIC caps, codec bounds, malformed objects, stream
  setup stalls, handshake replay/timestamp/identity binding, inbound static
  authorization, pending connect cleanup, stale ownership checks, metrics
  ownership, graceful shutdown, and bad-network behavior mapped to existing
  fixed issues or noncritical families; no new critical issue was accepted.
  Five consecutive no-new critical cycles after ISSUE-246 are now recorded
  under the current critical-only rule.
  ISSUE-043 is fixed by bounding pending pubsub publish/feedback RPC request
  maps before responder fanout.
  ISSUE-054 is fixed by rejecting zero network tick intervals before endpoint
  and ticker construction.
  ISSUE-056 is fixed by fail-fast stream-open control admission.
  ISSUE-064 is fixed by validating PeerStats direct-peer ownership before
  metrics export.
  ISSUE-065 is fixed by validating disconnect ownership before cleanup and
  public event emission.
  ISSUE-068 is fixed by validating PeerStats ownership before metrics export.
  ISSUE-019 is fixed by the ISSUE-208 alias refcount widening to `usize`.
  ISSUE-165 is fixed by emitting visualization leaves from peer disconnects.
  ISSUE-166 is fixed by a two-generation broadcast replay dedup window.
  ISSUE-167 is fixed by pruning router routes for expired discovered peers.
  ISSUE-003 is fixed by `cfc8e57`;
  ISSUE-090 is fixed by the alias `Found` request-correlation guard.
  ISSUE-001 and ISSUE-004 are covered by the ISSUE-170 ownership-validation follow-up
  `87cf6ce`; earlier fixes are `648cfd0`, `2cbf096`, `15b788c`, and
  `4997404`.
  ISSUE-040 is fixed by normalizing zero metrics and visualization collection
  intervals before constructing Tokio timers.

## Root Cause Summary

### RC-1: Authenticated identity is not authoritative

- Representative issues: ISSUE-001, ISSUE-004, ISSUE-014, ISSUE-015,
  ISSUE-018, ISSUE-020, ISSUE-039, ISSUE-048, ISSUE-066, ISSUE-067,
  ISSUE-068, ISSUE-090, ISSUE-115, ISSUE-116, ISSUE-145, ISSUE-189,
  ISSUE-194, ISSUE-226.
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
- ISSUE-226, score 73: fixed by `a736bae`. Metrics and visualization receivers
  now bind broadcast scan responses to an explicit trusted-collector allowlist;
  default constructors deny inbound scan responses.

### RC-2: Protocol state machines lack correlation/freshness checks

- Representative issues: ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-047,
  ISSUE-059, ISSUE-071, ISSUE-081 through ISSUE-089, ISSUE-095, ISSUE-099,
  ISSUE-110, ISSUE-111, ISSUE-143,
  ISSUE-166, ISSUE-171, ISSUE-175,
  ISSUE-186, ISSUE-205, ISSUE-206, ISSUE-231, ISSUE-232, ISSUE-233,
  ISSUE-237, ISSUE-239, ISSUE-240, ISSUE-241, ISSUE-242, ISSUE-243,
  ISSUE-245.
- Pattern: replicated-KV, alias, metrics, visualization, and pubsub flows accept
  stale, unsolicited, reordered, or mismatched responses or broadcasts because
  handlers do not verify request shape, bounds, version, continuation key,
  expected phase, membership generation, or whether an event actually advances
  activity.
- Minimal fix proposal: keep a small pending-request descriptor per flow and
  reject responses unless they match; for membership gossip, carry a generation
  or epoch and ignore older join/leave/heartbeat state. Refresh remote liveness
  only after an accepted event advances state or emits work.
- ISSUE-231, score 57: fixed by `ed8f4fb`. Pubsub now records bounded
  remote-role tombstones when publisher/subscriber leaves arrive for absent
  channel state, so a delayed older join cannot recreate a stale active remote
  membership. Verification:
  `cargo test unknown_publisher_leave_must_tombstone_stale_join -- --nocapture`,
  `cargo test unknown_subscriber_leave_must_tombstone_stale_join -- --nocapture`,
  `cargo test reclaimed_remote -- --nocapture`, and
  `cargo test stale_pubsub_leave_must_not_remove_membership_after_newer_heartbeat -- --nocapture`.
- ISSUE-232, score 58: fixed by `8339384`. Metrics now clears pending
  responder and scan-response state when a peer disconnects, so a delayed
  stale `Info` cannot satisfy an earlier request and publish disconnected-peer
  metrics. Verification:
  `cargo test metrics_stale_info_after_peer_disconnected_must_be_ignored -- --nocapture`,
  `cargo test metrics_info_must_not_be_accepted_without_scan_request -- --nocapture`,
  `cargo test metrics_info_batches_must_be_bounded -- --nocapture`,
  `cargo test visualization_stale_info_after_peer_disconnected_must_be_ignored -- --nocapture`,
  and `cargo test metrics -- --nocapture`.
- ISSUE-233, score 63: fixed by `c7aa3f5`. Replicated-KV now rejects
  unknown-peer RPC responses before remote-store admission, so a response-only
  first packet cannot allocate remote state or queue full-sync work.
  Verification:
  `cargo test unsolicited_rpc_response_from_unknown_peer_must_not_create_remote_store -- --nocapture`,
  `cargo test unsolicited_fetch_changed -- --nocapture`,
  `cargo test working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair -- --nocapture`,
  `cargo test service::replicate_kv_service::tests:: --lib -- --nocapture`,
  and `cargo test replicate_kv -- --nocapture`.
- ISSUE-237, score 61: fixed by `abe7e37`. Replicated-KV full sync now rejects
  non-empty snapshot pages whose `next_key` does not advance past the last
  accepted slot key, avoiding duplicate application and non-progressing
  continuation loops from malformed peers. Verification:
  `cargo test full_sync_must_reject_snapshot_next_key_that_does_not_advance -- --nocapture`,
  `cargo test replicate_kv -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/replicate_kv_service/remote_storage.rs`,
  and `git diff --check`.
- ISSUE-240, score 64: fixed pubsub chunked heartbeat correlation. Root cause:
  omission cleanup treated each inbound heartbeat frame as a complete peer
  snapshot even after outbound heartbeats were split into capped batches.
  Smallest fix: keep legacy `Heartbeat(Vec<_>)` as complete snapshots, add an
  explicit final-marked chunk variant for multi-batch heartbeats, accumulate
  per-peer seen channels across chunks, and run omitted cleanup only on the
  final chunk while preserving the per-chunk inbound cap.
- ISSUE-241, score 60: fixed pubsub chunked heartbeat snapshot lifecycle.
  Root cause: the chunk accumulator was keyed only by peer id, so an abandoned
  partial snapshot could leak its seen-channel set into a later complete
  snapshot. Smallest fix: add a chunk `snapshot_id`, increment it per outbound
  multi-batch heartbeat, store it with pending receiver state, and clear stale
  seen channels when the snapshot id changes.
- ISSUE-242, score 62: fixed pubsub chunked heartbeat completeness. Root
  cause: final-chunk cleanup trusted `is_last` without verifying that earlier
  chunks for the same snapshot had arrived. Smallest fix: add `chunk_index`
  and `chunks_count`, track received chunk indexes per pending snapshot, and
  run omitted-role cleanup only when the receiver has seen the full chunk set.
- ISSUE-243, score 66: fixed pubsub chunked heartbeat pending-state growth.
  Root cause: the receiver bounded rows per chunk but trusted peer-supplied
  `chunks_count` for the snapshot-level sequence space, so sparse chunk indexes
  could grow `seen_chunks` without ever completing cleanup. Smallest fix:
  reject `chunks_count` values above `MAX_HEARTBEAT_CHUNKS_PER_SNAPSHOT` and
  clear that peer's pending snapshot on malformed chunk metadata.
- ISSUE-245, score 58: fixed replicated-KV initial full-sync atomicity. Root
  cause: `SyncFullState::default()` started with `staged_slots: None`, so the
  first full sync wrote partial snapshot pages directly into visible remote
  slots, while only replacement resyncs used atomic staging. Smallest fix:
  initialize default full sync with an empty staging map and commit staged
  slots only after the terminal snapshot page is accepted.
- ISSUE-238, score 58: fixed by `d340a7b`. A peer-controlled
  `StreamConnectReq.defer_delivery` could reserve all destination service queue
  slots while waiting for a relay commit, temporarily denying legitimate stream
  delivery. Root cause: relay final-hop ordering reused the service queue
  reservation as an unbounded deferred-delivery hold. Smallest fix: keep
  final-hop reserve-before-success behavior, but cap deferred local reservations
  per connection to one less than the service queue capacity. Verification:
  `cargo test direct_stream_must_not_reserve_service_queue_while_deferred_delivery_waits --lib -- --nocapture`,
  `cargo test relayed_open_stream_does_not_succeed_when_final_service_queue_is_full --lib -- --nocapture`,
  `cargo test concurrent_relayed_open_streams_should_use_available_final_service_capacity --lib -- --nocapture`,
  `cargo test stream --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/peer/peer_internal.rs src/service.rs src/tests/stream.rs`,
  and `git diff --check`.
- Cycle after ISSUE-238 no-new cycle 1 reviewed route/discovery lifecycle and
  path stability with forked reviewer `Kierkegaard`. Focused route hysteresis,
  direct-route preference, sync cap prioritization, stopped tombstone,
  seed/non-seed expiry, `PeerStopped`, tick-sync retry, and graceful shutdown
  checks passed. Rejected candidates mapped to ISSUE-003/RC-7, ISSUE-004,
  ISSUE-051, ISSUE-055, ISSUE-063, ISSUE-103, ISSUE-118, ISSUE-151,
  ISSUE-167, ISSUE-170, ISSUE-211, ISSUE-213, ISSUE-215 through ISSUE-225,
  RC-3, RC-6, and RC-7.
- Cycle after ISSUE-238 no-new cycle 2 reviewed metrics, visualization, and
  alias service state/backpressure with forked reviewer `Dalton`. Focused alias
  suite, stale metrics/visualization `Info`, local alias shutdown, and alias
  disconnect lifecycle checks passed. Rejected candidates mapped to ISSUE-090,
  ISSUE-127, ISSUE-179, ISSUE-183, ISSUE-202, ISSUE-203, ISSUE-204, ISSUE-208,
  ISSUE-226, ISSUE-232, and ISSUE-235.
- Cycle after ISSUE-238 no-new cycle 3 reviewed the red broad metrics and
  visualization suites with forked reviewer `Russell`. The failing
  scan-response backpressure evidence was stale after ISSUE-234 requester
  liveness registration, not a new library issue. `src/peer.rs` test harnesses
  now register direct `P2pService::build(...)` services the same way
  `P2pNetwork::create_service` does. Verification:
  `rustfmt --edition 2021 --check src/peer.rs`,
  `RUST_LOG=error cargo test metrics --lib -- --nocapture`,
  `RUST_LOG=error cargo test visualization --lib -- --nocapture`, and
  `git diff --check`.
- Cycle after ISSUE-238 no-new cycle 4 reviewed P2pNetwork/service lifecycle,
  requester admission, duplicate service boundaries, graceful shutdown, and
  pending cleanup with forked reviewer `Tesla`. Focused requester, service
  requester, duplicate service, and graceful shutdown filters passed; reviewer
  cross-checks also passed `peer_stopped_`, `pending`, and `disconnect`
  filters serially where needed. Rejected candidates mapped to ISSUE-028,
  ISSUE-030, ISSUE-051, ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-063,
  ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-118, ISSUE-125,
  ISSUE-164, ISSUE-167, ISSUE-170, ISSUE-215 through ISSUE-225, ISSUE-234,
  RC-3, and RC-6.
- Cycle after ISSUE-238 no-new cycle 5 reviewed fuzz harness coverage and
  transitioned the next audit phase to configured node-count randomized fuzzing
  with forked reviewer `Raman`. `P2P_FUZZ_NODES` is honored, six random fuzz
  tests are listed, and 6-node plus 12-node deterministic steady/churn/random
  fuzz runs passed. Rejected candidates mapped to ISSUE-053, ISSUE-060,
  ISSUE-091, ISSUE-209, ISSUE-215 through ISSUE-225, ISSUE-218 through
  ISSUE-230, ISSUE-234, RC-3, RC-6, and RC-7. Next phase should run and extend
  configured-node fuzzing, accepting only distinct reviewed failures with test
  evidence.
- Fuzz phase no-new cycle 1 ran configured-node randomized fuzzing with forked
  reviewer `Boyle`. Local 10- and 12-node steady, valid-churn,
  sanitized-churn, and malformed-action fuzz runs passed; reviewer 12- and
  14-node cross-checks passed. Churn/refused-connection/endpoint-drop noise
  mapped to ISSUE-003, ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-209,
  ISSUE-215 through ISSUE-225, ISSUE-218 through ISSUE-230, ISSUE-234, RC-3,
  RC-6, and RC-7.
- Fuzz phase no-new cycle 2 continued configured-node randomized fuzzing with
  forked reviewer `Chandrasekhar`. Local 14- and 16-node runs covered every
  random fuzz entry point and passed; reviewer 12-, 14-, 16-, and 18-node
  cross-checks passed. Duplicate connection, endpoint-drop, shutdown,
  peer-stopped, deadline, and delivery-ack noise mapped to ISSUE-003,
  ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-113, ISSUE-114, ISSUE-139,
  ISSUE-144, ISSUE-170, ISSUE-177, ISSUE-180, ISSUE-193, ISSUE-197,
  ISSUE-215 through ISSUE-225, ISSUE-218 through ISSUE-230, ISSUE-234, RC-3,
  RC-4, RC-6, and RC-7.
- Fuzz phase no-new cycle 3 raised local configured-node fuzzing to 18- and
  20-node real-input runs plus 16-node malformed/forged churn with forked
  reviewer `Faraday`. Local and reviewer cross-checks passed. Duplicate
  connection, endpoint/control closed, stopped/restart/refused, deadline,
  route-loop stream rejection, open-bi channel, and delivery-ack noise mapped
  to existing lifecycle, backpressure, stream setup, and routing families:
  ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-156, ISSUE-180, ISSUE-217,
  ISSUE-220, ISSUE-234, ISSUE-238, RC-3, RC-6, and RC-7.
- Fuzz phase no-new cycle 4 extended configured-node randomized fuzzing to
  18-, 20-, and 22-node local runs with forked reviewer `Nash the 2nd`.
  Local and reviewer cross-checks passed, including malformed/forged action
  and churn coverage. Duplicate connection, endpoint/control closed,
  peer-stopped/refused-reconnect/deadline, `open_bi` channel, delivery-ack,
  route-not-found, and capacity/backpressure noise mapped to existing
  lifecycle, stream setup, malformed-input, ownership, and routing families:
  ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-156, ISSUE-180, ISSUE-217,
  ISSUE-220, ISSUE-234, ISSUE-238, RC-3, RC-6, and RC-7.
- Fuzz phase no-new cycle 5 pushed high-load configured-node randomized
  fuzzing to 20-, 22-, and 24-node local runs with forked reviewer
  `Boyle the 2nd`. Local and reviewer cross-checks passed, including
  malformed/forged action, sanitized churn, raw churn, stream-heavy valid
  action, and steady valid action coverage. Duplicate connection,
  endpoint/control closed, peer-stopped/refused-reconnect/deadline,
  handshake-close, `open_bi`/route setup, delivery-ack, and
  capacity/backpressure noise mapped to existing lifecycle, stream setup,
  malformed-input, ownership, async backpressure, and routing families:
  ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-156, ISSUE-180, ISSUE-217,
  ISSUE-220, ISSUE-224, ISSUE-225, ISSUE-227, ISSUE-229, ISSUE-230,
  ISSUE-234, ISSUE-238, RC-3, RC-6, and RC-7.
- Focused source-review no-new cycle 1 reviewed lifecycle/discovery/router
  graceful-stop and route-stability behavior with forked reviewer
  `Nietzsche the 2nd`. Local focused tests and reviewer cross-checks passed
  for active path stability, direct route preference, seed retention,
  non-seed timeout/removal, graceful-stop tombstones, shutdown notification
  latency, and pubsub/replicated-KV/visualization graceful-stop propagation.
  Rejected candidates mapped to ISSUE-003, ISSUE-004, ISSUE-051, ISSUE-063,
  ISSUE-156, ISSUE-167, ISSUE-170, ISSUE-180, ISSUE-215 through ISSUE-225,
  ISSUE-231, ISSUE-238, RC-3, RC-6, and RC-7.
- Focused source-review no-new cycle 2 reviewed security/transport handshake,
  inbound identity binding, setup timeouts, QUIC stream admission, and stale
  event route binding with forked reviewer `Raman the 2nd`. Local focused
  tests and reviewer cross-checks passed for handshake freshness, future
  timestamp rejection, replay-cache bounds, inbound identity binding,
  authenticated inbound admission accounting, unidirectional stream rejection,
  outbound/inbound setup timeout cleanup, and stale connected/connect-error/data
  event handling. Rejected candidates mapped to ISSUE-002, ISSUE-021,
  ISSUE-117, ISSUE-146, ISSUE-172, ISSUE-173, ISSUE-176, ISSUE-189,
  ISSUE-194, ISSUE-207, RC-3, RC-6, and RC-7.
- Focused source-review no-new cycle 3 reviewed route/discovery lifecycle,
  path stability, graceful-stop cleanup, stopped-peer tombstones, seed
  retryability, bounded sync admission, pending sync cleanup, and stale
  event guards with forked reviewer `Poincare the 2nd`. Local focused tests
  and reviewer cross-checks passed for route/discovery tables,
  peer-stopped/stale-peer handling, discovery timeout cleanup, and duplicate
  connect backlog coalescing. Rejected candidates mapped to ISSUE-003,
  ISSUE-004, ISSUE-010, ISSUE-051, ISSUE-055, ISSUE-063, ISSUE-156,
  ISSUE-167, ISSUE-170, ISSUE-180, ISSUE-190, ISSUE-192, ISSUE-211,
  ISSUE-212, ISSUE-213, ISSUE-214, ISSUE-217, ISSUE-220, ISSUE-229,
  ISSUE-230, ISSUE-238, RC-3, RC-6, and RC-7.
- Focused source-review no-new cycle 4 reviewed stream/pipe setup, relayed
  delivery, unicast acknowledgement, service queue backpressure, deferred
  delivery reservations, source binding, ingress relay loops, stale dropped
  requesters, and cross-node direct/relay delivery with forked reviewer
  `Confucius the 2nd`. Local focused tests and reviewer cross-checks passed
  for stream setup, relayed open, orphan relay rejection, unicast,
  open-stream timeout/error handling, cross-node delivery, broadcast/unicast
  backpressure, dropped requesters, source binding, and service queue pressure.
  Rejected candidates mapped to ISSUE-003, ISSUE-011, ISSUE-012, ISSUE-014,
  ISSUE-018, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-119, ISSUE-149,
  ISSUE-156, ISSUE-169, ISSUE-180, ISSUE-197, ISSUE-217, ISSUE-224,
  ISSUE-225, ISSUE-229, ISSUE-230, ISSUE-234, ISSUE-238, RC-3, RC-4,
  RC-6, and RC-7.
- Focused source-review no-new cycle 5 reviewed pubsub RPC correlation,
  stale membership generations/tombstones, pending/deadline/resource caps,
  replicated-KV unsolicited response rejection, accepted-event liveness,
  full-sync snapshot validation, and `FetchChanged` repair correlation with
  forked reviewer `Godel the 2nd`. Local focused tests and reviewer
  cross-checks passed for publish/feedback RPCs, stale pubsub updates,
  heartbeats, full-sync, unsolicited responses, pending caps, snapshots,
  repair, stale fetch handling, and duplicate rejection. Rejected candidates
  mapped to ISSUE-020, ISSUE-023, ISSUE-025, ISSUE-027, ISSUE-034,
  ISSUE-037, ISSUE-038, ISSUE-043, ISSUE-046, ISSUE-047, ISSUE-059,
  ISSUE-071, ISSUE-074, ISSUE-075, ISSUE-080, ISSUE-081 through ISSUE-089,
  ISSUE-095, ISSUE-099, ISSUE-100, ISSUE-106, ISSUE-110, ISSUE-111,
  ISSUE-115, ISSUE-116, ISSUE-121, ISSUE-123 through ISSUE-126, ISSUE-131,
  ISSUE-138, ISSUE-140, ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-155,
  ISSUE-163, ISSUE-171, ISSUE-175, ISSUE-178, ISSUE-184, ISSUE-186,
  ISSUE-196, ISSUE-205, ISSUE-206, ISSUE-228, ISSUE-231, ISSUE-233,
  ISSUE-237, RC-1, RC-2, RC-3, and RC-6. Five focused source-review no-new
  cycles are now recorded after the prior fuzz phase; the next audit phase
  should return to configured-node randomized fuzzing with reviewer-confirmed
  failing evidence required for any accepted issue.
- Fuzz phase no-new cycle 6 returned to configured-node randomized fuzzing
  after five focused source-review no-new cycles with forked reviewer
  `Hubble the 2nd`. Local fuzz coverage passed the node-count configuration
  check, the 6-test fuzz inventory check, and deterministic seeds
  28001 through 28006 across steady valid actions, valid random actions,
  valid churn, sanitized churn, malformed/random raw actions, and
  malformed/churn actions. Reviewer cross-check seeds 28101 through 28103
  also passed. Rejected noise mapped to duplicate-connect/route churn and
  lifecycle families, RC-6, RC-7, stream/open path backpressure families
  RC-3/RC-7, ISSUE-156, ISSUE-180, ISSUE-217, ISSUE-220, ISSUE-238,
  malformed-input/ownership families including ISSUE-053, ISSUE-060,
  ISSUE-091, ISSUE-234, RC-1, RC-6, and fixed ISSUE-209.
- Fuzz phase no-new cycle 7 raised configured-node randomized fuzzing to
  20-, 24-, 26-, and 28-node deterministic runs with forked reviewer
  `Kepler the 2nd`. Local seeds 29001 through 29006 passed across steady
  valid actions, valid random actions, valid churn, sanitized churn,
  malformed/random raw actions, and malformed/churn actions. Reviewer
  cross-check seeds 29101 through 29103 also passed. Rejected noise mapped to
  duplicate-connect/route-churn/lifecycle families, RC-7, RC-6,
  ISSUE-215 through ISSUE-225, stream/open path backpressure families
  RC-3/RC-7, ISSUE-156, ISSUE-180, ISSUE-217, ISSUE-220, and ISSUE-238.
  Malformed/random and malformed/churn coverage did not produce failing
  evidence for a new malformed-input issue.
- Fuzz phase no-new cycle 8 raised configured-node randomized fuzzing to a
  30-node steady run and 22- to 28-node valid, churn, sanitized, malformed,
  and raw-action runs with forked reviewer `Ramanujan the 2nd`. Local seeds
  30001 through 30006 passed across all six fuzz entry points, and reviewer
  cross-check seeds 300101 through 300105 also passed. Rejected noise mapped
  to fixed ISSUE-209, malformed-input/ownership families ISSUE-053,
  ISSUE-060, ISSUE-091, ISSUE-234, RC-1, RC-6, graceful-stop/lifecycle
  families ISSUE-215 through ISSUE-225, and route/stream/backpressure
  families ISSUE-003, ISSUE-156, ISSUE-180, ISSUE-217, ISSUE-220,
  ISSUE-238, RC-3, and RC-7. No reproducible fuzz failure supported a
  distinct score-80+ issue.
- Fuzz phase no-new cycle 9 continued critical-only configured-node fuzzing and
  source-boundary lifecycle review with forked reviewer `Hypatia the 2nd`.
  Local 32-node steady fuzz, 28-node sanitized churn, 24-node malformed churn,
  `peer_stopped`, and dropped-service-requester tests passed; reviewer
  cross-checks covered node-count config, the six fuzz entries, 12- to 16-node
  deterministic fuzz seeds, requester, service, and peer-stopped slices. All
  rejected candidates mapped to fixed ISSUE-209, malformed-input/ownership
  families ISSUE-053, ISSUE-060, ISSUE-091, ISSUE-234, RC-1, RC-6,
  lifecycle/graceful-stop families ISSUE-215 through ISSUE-225, route
  stability ISSUE-003/RC-7, stream/backpressure families ISSUE-156,
  ISSUE-180, ISSUE-217, ISSUE-220, ISSUE-238, RC-3, and stale
  service-requester families ISSUE-028, ISSUE-072, ISSUE-073, ISSUE-076,
  ISSUE-125. Ledger check found 21 score-80+ issues and all are fixed; no
  reproducible fuzz/source-boundary failure supported a distinct score-80+
  issue.
- Fuzz phase no-new cycle 10 reviewed transport/auth/config-resource
  boundaries with forked reviewer `Tesla the 2nd`. Local focused tests passed
  for handshake/inbound identity, QUIC unidirectional-stream admission,
  discovery, router, and a 26-node malformed/churn fuzz seed. Reviewer
  cross-checks passed `secure::tests`, `quic::tests`, `discovery`, `router`,
  `msg::tests`, `security`, and `readme`. Rejected candidates mapped to
  handshake/auth families ISSUE-002, ISSUE-021, ISSUE-146, ISSUE-176,
  ISSUE-189, ISSUE-194, ISSUE-207, QUIC/setup families ISSUE-117,
  ISSUE-172, ISSUE-173, malformed message/service bounds ISSUE-053,
  ISSUE-060, ISSUE-091, ISSUE-234, route stability ISSUE-003, ISSUE-214,
  RC-7, and discovery/lifecycle families ISSUE-211, ISSUE-212, ISSUE-213,
  ISSUE-215 through ISSUE-225, RC-6. Ledger check found 21 score-80+ issues
  and all are fixed; no reproducible transport/auth/config-resource boundary
  failure supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 11 reviewed service-layer resource and lifecycle
  state with forked reviewer `Banach the 2nd`. Local checks passed pubsub,
  replicated KV, metrics, visualization, alias, and a 24-node valid churn fuzz
  seed. Reviewer cross-checks also passed `service`, `requester`, and
  `dropped_` slices. Rejected candidates mapped to pubsub heartbeat and
  lifecycle families ISSUE-228, ISSUE-231, ISSUE-240 through ISSUE-243,
  ISSUE-246, pubsub RPC families ISSUE-020, ISSUE-043, ISSUE-074,
  ISSUE-075, ISSUE-115, ISSUE-116, ISSUE-123 through ISSUE-126, ISSUE-236,
  replicated-KV families ISSUE-023, ISSUE-025, ISSUE-034, ISSUE-037,
  ISSUE-038, ISSUE-047, ISSUE-059, ISSUE-081 through ISSUE-089, ISSUE-110,
  ISSUE-111, ISSUE-131, ISSUE-143, ISSUE-171, ISSUE-233, ISSUE-237,
  ISSUE-245, telemetry families ISSUE-064, ISSUE-068, ISSUE-165, ISSUE-226,
  ISSUE-232, alias families ISSUE-028, ISSUE-090, ISSUE-125, ISSUE-208,
  ISSUE-235, ISSUE-239, and base service/requester families ISSUE-052,
  ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-234. Ledger check found
  21 score-80+ issues and all are fixed; no reproducible service-layer
  resource/lifecycle failure supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 12 reviewed integration/backpressure boundaries with
  forked reviewer `Epicurus the 2nd`. Local checks passed `security`,
  `stream`, `cross_nodes`, `requester`, `peer_stopped`, and a 30-node steady
  fuzz seed. Reviewer cross-checks passed `security`, `stream`, `cross_nodes`,
  and deterministic steady/churn fuzz seeds. Rejected candidates mapped to
  stream and relay families ISSUE-156, ISSUE-217, ISSUE-220, ISSUE-238,
  unicast/ack families ISSUE-119, ISSUE-224, ISSUE-225, ISSUE-229,
  ISSUE-230, graceful-stop/lifecycle families ISSUE-215 through ISSUE-225,
  stale-event/duplicate-connect families ISSUE-153, ISSUE-189, ISSUE-194,
  ISSUE-221 through ISSUE-223, forged source-binding families ISSUE-001,
  ISSUE-014, ISSUE-018, ISSUE-053, ISSUE-091, queue/backpressure families
  ISSUE-117, ISSUE-172, ISSUE-173, ISSUE-218, ISSUE-219, ISSUE-227,
  ISSUE-235, and panic/service/requester families ISSUE-060, ISSUE-072,
  ISSUE-073, ISSUE-076, ISSUE-191, ISSUE-234. Ledger check found 21
  score-80+ issues and all are fixed; no reproducible integration/backpressure
  failure supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 13 reviewed the manifest, README, examples, readme
  compile gate, and broad public API surface with forked reviewer `Anscombe
  the 2nd`. Local checks passed `cargo check --examples`, full `cargo test
  --lib` with 428 tests, and a 22-node malformed/random fuzz seed. Reviewer
  cross-checks passed `readme`, `security`, `cross_nodes`, `cargo check
  --examples`, and `cargo test --all-targets`. Rejected candidates mapped to
  open-cluster demo/static-binding families ISSUE-014, ISSUE-015, ISSUE-018,
  ISSUE-189, ISSUE-194, public service/requester/stream families ISSUE-011,
  ISSUE-012, ISSUE-052, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076,
  ISSUE-091, ISSUE-156, ISSUE-217, ISSUE-220, ISSUE-238, transport/resource
  families ISSUE-024, ISSUE-097, ISSUE-098, ISSUE-117, ISSUE-172,
  ISSUE-173, ISSUE-174, and route/discovery/lifecycle families ISSUE-003,
  ISSUE-004, ISSUE-010, ISSUE-103, ISSUE-211 through ISSUE-225. Ledger check
  found 21 score-80+ issues and all are fixed; no reproducible public API,
  manifest, example, or full-suite failure supported a distinct score-80+
  issue.
- Fuzz phase no-new cycle 14 reviewed tooling and code-quality/security-scan
  boundaries with forked reviewer `Hegel the 2nd`. `cargo fmt -- --check`
  failed on formatting/import ordering only, and strict clippy failed on
  unused imports, dead code, style lints, and test-code idioms only. Unsafe
  scans found no `unsafe` usage. `cargo audit` and `cargo deny check` were not
  installed, while `cargo metadata`, `cargo check --all-targets`, and broad
  tests passed; local `cargo test --all-targets` passed 428 library tests plus
  all example test targets. Rejected candidates mapped to fixed lifecycle and
  resource families RC-3 through RC-6, ISSUE-060, ISSUE-156, ISSUE-217,
  ISSUE-220, ISSUE-234, ISSUE-236, ISSUE-238, handshake families ISSUE-002,
  ISSUE-021, ISSUE-146, ISSUE-176, ISSUE-207, ISSUE-244, transport families
  ISSUE-117, ISSUE-172, ISSUE-173, router/path families ISSUE-003, ISSUE-180,
  ISSUE-197, ISSUE-214, and public API identity/binding families ISSUE-014,
  ISSUE-015, ISSUE-018, ISSUE-189, ISSUE-194. Ledger check found 21 score-80+
  issues and all are fixed; no reproducible tooling/code-quality failure
  supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 15 reviewed adversarial configured-node fuzz and
  fuzz-harness blind spots with forked reviewer `Dalton the 2nd`. Local
  high-load steady, valid churn, and malformed churn fuzz seeds passed at 34,
  30, and 28 nodes respectively. The reviewer reran those fuzz commands and
  targeted `peer_stopped`, `forged`, relay-stream, inbound-stream, and fuzz-list
  checks; all passed. Rejected candidates mapped to fixed ISSUE-209 for
  high-load configuration, ISSUE-001/ISSUE-004/ISSUE-170/ISSUE-215 through
  ISSUE-225 and RC-6 for graceful stop/restart churn, ISSUE-014/ISSUE-015/
  ISSUE-018 and RC-1 for forged source identity, ISSUE-052/ISSUE-053/
  ISSUE-060/ISSUE-091/ISSUE-234 for invalid services and malformed messages,
  ISSUE-156/ISSUE-217/ISSUE-220/ISSUE-238 and RC-3/RC-4 for stream setup and
  admission, ISSUE-003/RC-7 for route flapping, and RC-3 plus ISSUE-218 through
  ISSUE-230 for bounded backpressure. Ledger check found 21 score-80+ issues
  and all are fixed; no reproducible fuzz/source-boundary failure supported a
  distinct score-80+ issue.
- Fuzz phase no-new cycle 16 reviewed protocol framing, message-size bounds,
  QUIC stream setup, and public service request boundaries with forked reviewer
  `Dirac the 2nd`. Local peer-frame, `write_object`, out-of-range service-id,
  dropped-service-requester, stream-timeout, and 36-node malformed fuzz checks
  passed. The reviewer passed `stream::tests`, out-of-range, stale-handle,
  inbound-out-of-range, forged-source, peer-stopped, open-stream, and full-sync
  focused slices. Rejected candidates mapped to existing stream codec/object
  bounds, ISSUE-052/ISSUE-060/ISSUE-091/ISSUE-234 and RC-6 for service ids,
  ISSUE-014/ISSUE-015/ISSUE-018 and RC-1 for forged source identity,
  ISSUE-072/ISSUE-073/ISSUE-076/ISSUE-234 and RC-6 for stale/dropped handles,
  RC-3/RC-4 plus ISSUE-156/ISSUE-217/ISSUE-220/ISSUE-224/ISSUE-225/ISSUE-238
  for queue-full and stalled stream setup, ISSUE-215 through ISSUE-225 and RC-6
  for graceful shutdown, and ISSUE-003/ISSUE-180/ISSUE-197/RC-7 for route
  decisions during setup. Ledger check found 21 score-80+ issues and all are
  fixed; no reproducible protocol/service-boundary failure supported a
  distinct score-80+ issue.
- Fuzz phase no-new cycle 17 reviewed pubsub and replicated-KV service
  protocols with forked reviewer `Volta the 2nd`. Local `pubsub`,
  `replicate_kv`, `rpc`, `heartbeat`, `full_sync`, `working_state_must`, and
  32-node valid churn fuzz checks passed. The reviewer passed the broad pubsub
  and KV slices plus stale-handle, heartbeat, expected-responder, full-sync
  rejection, working-state rejection, and 36-node churn fuzz checks. Rejected
  candidates mapped to RC-1/RC-2 and ISSUE-020/ISSUE-043/ISSUE-115/ISSUE-116/
  ISSUE-236 for pubsub RPC correlation, RC-3 and ISSUE-123 through ISSUE-126
  plus ISSUE-246 for fanout/backpressure, ISSUE-228 and ISSUE-240 through
  ISSUE-243 for pubsub bounds and chunking, ISSUE-231/ISSUE-246/RC-2/RC-6 for
  stale pubsub lifecycle, ISSUE-237/ISSUE-245 for replicated-KV full-sync
  pagination and staging, ISSUE-081 through ISSUE-089/ISSUE-110/ISSUE-111/
  ISSUE-143/RC-2 for changed repair correlation, and ISSUE-233/RC-3/RC-6 for
  remote-store caps and graceful-stop cleanup. Ledger check found 21 score-80+
  issues and all are fixed; no reproducible service-protocol failure supported
  a distinct score-80+ issue.
- Fuzz phase no-new cycle 18 reviewed route, discovery, connection lifecycle,
  and graceful shutdown behavior with forked reviewer `Pascal the 2nd`. Local
  router, discovery, peer-stopped, stale-peer, duplicate, shutdown,
  discovery-timeout, active-path, and 34-node churn fuzz checks passed. The
  reviewer passed broader route, discovery, peer-stopped, graceful, connect,
  stale, sync, 38-node valid churn, 36-node malformed churn, and targeted
  route/relay checks. Rejected candidates mapped to ISSUE-003/ISSUE-214/RC-7
  for path stability, ISSUE-167/ISSUE-215 through ISSUE-222/RC-6/RC-7 for
  stale route resurrection, ISSUE-214 for direct-route priority, ISSUE-167 and
  ISSUE-211 through ISSUE-213/RC-7 for seed and non-seed deletion,
  ISSUE-215 through ISSUE-225/RC-6 for `PeerStopped` and graceful shutdown,
  ISSUE-153/ISSUE-189/ISSUE-194/ISSUE-223/RC-1/RC-6 for duplicate and stale
  connect events, and ISSUE-063/ISSUE-103/ISSUE-164/ISSUE-180/ISSUE-197/RC-7
  for route/discovery sync caps and malformed rows. Ledger check found 21
  score-80+ issues and all are fixed; no reproducible lifecycle/route failure
  supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 19 reviewed public API, config, transport, and
  example defaults with forked reviewer `Lagrange the 2nd`. Local handshake,
  inbound-handshake, requester-connect, zero-tick, unidirectional-stream,
  broad connect, unauthenticated-inbound, and example-build checks passed;
  `cargo check --examples` passed with unused/dead-code warnings only. The
  reviewer passed secure, inbound-handshake, zero-tick, own-peer-address,
  duplicate-connection, requester, unidirectional-stream, setup-timeout,
  open-stream-timeout, advertise, seed, example no-run, and 34-node valid
  fuzz checks. Rejected candidates mapped to RC-1 and fixed handshake replay
  families for shared-key freshness/replay/identity, RC-1/ISSUE-244 for
  inbound peer binding defaults, ISSUE-153/ISSUE-189/ISSUE-194/ISSUE-223 for
  self/duplicate connect behavior, ISSUE-211 through ISSUE-213/RC-7 for
  advertise and seed address validation, ISSUE-217/ISSUE-220/ISSUE-238 plus
  RC-3/RC-4 for transport admission and stalled setup, and ISSUE-072/
  ISSUE-073/ISSUE-076/ISSUE-234/RC-6 for stale requesters. README/examples
  remain explicit demo/open-cluster surfaces rather than unsafe library
  defaults. Direct issue-entry ledger check found 19 score-80+ entries and all
  are fixed; no reproducible public API/config/transport failure supported a
  distinct score-80+ issue.
- Fuzz phase no-new cycle 20 reviewed channel, task, and resource-boundary
  behavior with forked reviewer `Copernicus the 2nd`. Local `pending`,
  `backpressure`, `queue`, `bounded`, and 36-node valid-action fuzz checks
  passed. The reviewer passed bounded, queue-full, must-not-drop, drop,
  shutdown, backpressure, pending-unicast-ack, peer-disconnect retry,
  sparse-heartbeat, and 34-node valid-action fuzz checks. Rejected candidates
  mapped to RC-3 for send admission and queue backpressure, RC-5 for
  peer-controlled collection caps, and RC-6 for drop/shutdown/stale-handle
  cleanup. Direct issue-entry ledger check found 19 score-80+ entries and all
  are fixed; no reproducible channel/resource failure supported a distinct
  score-80+ issue.
- Fuzz phase no-new cycle 21 reviewed serialization, framing, and public API
  panic/DoS boundaries with forked reviewer `Mencius the 2nd`. Local `codec`,
  `service_id`, `panic`, `invalid`, `oversize`, `object`, `handshake`,
  `stream`, and 20-node malformed-action fuzz checks passed; the `malformed`
  filter matched zero tests and was replaced by exact nearby invalid/oversize
  coverage. The reviewer passed stream, service-id, inbound-handshake,
  oversized, handshake, codec, and 34-node valid-action fuzz checks plus static
  sweeps over object framing, bincode, service ids, frame limits, and
  production panic paths. Rejected candidates mapped to RC-5 for capped bincode
  frames, RC-4/RC-5 for object length-prefix and serialization failures, RC-6
  for service-id parsing, RC-1 for malformed peer-message identity, and fixed
  pubsub heartbeat enum/chunking issues for compatibility concerns. Direct
  issue-entry ledger check found 19 score-80+ entries and all are fixed; no
  reproducible serialization/framing failure supported a distinct score-80+
  issue.
- Fuzz phase no-new cycle 22 reviewed time, ordering, and lifecycle behavior
  with forked reviewer `Fermat the 2nd`. Local `stale`, `stopped`,
  `tombstone`, `timeout`, `route`, `discovery`, `metrics`, `visualization`,
  and 34-node valid churn fuzz checks passed. The reviewer passed discovery,
  router, peer-stopped, stale-peer, metrics, visualization, shutdown,
  reconnect, active-path, timestamp, tombstone, and 34-node valid-action fuzz
  checks; `delayed` matched zero tests and was not counted. Rejected
  candidates mapped to fixed timestamp/timeout coverage, RC-6 stale-event
  ordering, the stale pending outgoing reconnect fix, ISSUE-004/ISSUE-167/
  ISSUE-211 through ISSUE-213 for seed/non-seed lifecycle, ISSUE-215 through
  ISSUE-225 and ISSUE-231 for tombstone freshness, ISSUE-232 for metrics and
  liveness cleanup, graceful-shutdown and `PeerStopped` backpressure tests for
  shutdown ordering, and ISSUE-003/RC-7 for active-path stability. Direct
  issue-entry ledger check found 19 score-80+ entries and all are fixed; no
  reproducible time/order/lifecycle failure supported a distinct score-80+
  issue.
- Fuzz phase no-new cycle 23 reviewed package, API defaults, and example
  surfaces with forked reviewer `Kuhn the 2nd`. Local example build, README,
  zero-tick, advertise, seed, inbound-handshake, requester, duplicate-service,
  out-of-range-service, own-peer-address, package metadata/dependency feature,
  and 24-node valid fuzz checks passed. The reviewer passed README,
  handshake, inbound-handshake, requester, advertise, seed, zero-tick,
  service-id, stream, security, package metadata/listing, example build, and
  24-node valid fuzz checks. `cargo audit` was not installed for advisory
  checks. Rejected candidates mapped to RC-1/ISSUE-244 for inbound
  binding/open-cluster opt-in, ISSUE-211 through ISSUE-213/RC-7 for
  advertise/seed validation, RC-6 and ISSUE-072/073/076/234 for stale
  requester/service behavior, ISSUE-052/060/091/234 for service-id bounds,
  ISSUE-117/156/217/220/238 and RC-3/RC-4/RC-5 for QUIC, stream, framing, and
  admission concerns, and existing route/lifecycle fixes for self/duplicate
  connect and stale events. README/examples remain explicit demo surfaces
  rather than unsafe library defaults; direct issue-entry ledger check found 19
  score-80+ entries and all are fixed; no reproducible package/API/default
  failure supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 24 reviewed shutdown, drop, and task-cancellation
  behavior with forked reviewer `Kant the 2nd`. Local `shutdown`, `dropped`,
  `drop`, `peer_stopped`, `stream`, `service_drop`, `graceful`,
  `disconnected`, `queue`, `reconnect`, and 28-node valid churn fuzz checks
  passed. The reviewer passed `peer_stopped_`, `requester_`,
  `dropped_service`, `graceful`, `shutdown`, `service_drop`, 24-node valid
  churn fuzz, and 24-node mixed churn fuzz checks. Rejected candidates mapped
  to RC-1/RC-6 and ISSUE-001/004/215/216/221/222 for graceful-stop ordering
  and stopped-peer cleanup, RC-3/RC-6 and ISSUE-218/219/224/225 for
  shutdown/full-queue backpressure, RC-6 and ISSUE-072/073/076/234/235 for
  stale requester/service drop behavior, RC-3/RC-4/RC-6 and
  ISSUE-217/220/238 for QUIC close, stalled setup, stream admission, and task
  concerns, and discovery tombstone tests for seed vs non-seed graceful
  removal. Direct issue-entry ledger check found 19 score-80+ entries and all
  are fixed; no reproducible shutdown/drop/task-cancellation failure supported
  a distinct score-80+ issue.
- Fuzz phase no-new cycle 25 reviewed observability/admin surfaces. Local
  source inspection and focused `metrics`, `visualization`, `scan`,
  stale-peer-stats, `pending`, `disconnect`, `dropped_service`, `bounded`, and
  24-node valid fuzz checks passed. Rejected candidates mapped to ISSUE-078,
  ISSUE-079, ISSUE-226, and RC-1/RC-2 for scan authorization; ISSUE-061,
  ISSUE-062, ISSUE-232, RC-2, and RC-6 for unsolicited or stale forged
  `Info`; ISSUE-200 through ISSUE-204 and RC-3 for scan broadcast/response
  backpressure and duplicate response tasks; ISSUE-102, ISSUE-104, ISSUE-105,
  and RC-5 for resource caps; ISSUE-064, ISSUE-068, ISSUE-128, ISSUE-129,
  ISSUE-165, ISSUE-232, and RC-6 for disconnect, stale metrics, graceful
  leaves, and base-service closure behavior; and ISSUE-072/073/076/234/235
  for shutdown/drop stale service/requester behavior. Direct issue-entry
  ledger check found 19 score-80+ entries and all are fixed; no reproducible
  observability/admin failure supported a distinct score-80+ issue.
- Fuzz phase no-new cycle 26 reviewed serialization and public API boundaries.
  Local source inspection and focused `stream::tests`, `codec`, `object`,
  `service_id`, `invalid`, `oversize`, `panic`, `inbound_out_of_range`,
  `stream`, `security`, `service`, two 24-node fuzz seeds, and test-list
  coverage checks passed. The `malformed` and `raw` filters matched zero tests
  and were not counted as evidence. Reviewer `Ptolemy the 2nd` passed
  `object`, `service_id`, `invalid`, `peer_message_codec`, `stream`,
  `pubsub_`, and 24-node valid fuzz checks; reviewer `malformed` also matched
  zero tests. Rejected candidates mapped to ISSUE-024/RC-5 for peer-message
  frame caps, ISSUE-097/098/174 for object serialization and u16 length
  prefix bounds, ISSUE-052/053/060/091/234 and RC-6 for service-id bounds,
  ISSUE-117/156/217/220/238 and RC-3/RC-4 for stream setup/admission,
  ISSUE-094/RC-2/RC-5 for pubsub object serialization and malformed service
  payload behavior, and ISSUE-061/062/200 through ISSUE-204/226/RC-5 for
  metrics/visualization bincode `Info`/`Scan` trust and caps. Direct
  issue-entry ledger check found 19 score-80+ entries and all are fixed; no
  reproducible serialization/API-boundary failure supported a distinct
  score-80+ issue.
- Fuzz phase no-new cycle 27 reviewed secure handshake and QUIC setup
  behavior. Local source inspection and focused `handshake`, `secure::tests`,
  `inbound_handshake`, `setup`, `quic`, `unauthenticated`, `identity`,
  remote-peer-id mismatch, `stream`, `security`, 24-node fuzz, and test-list
  coverage checks passed. Reviewer `Euler the 2nd` passed handshake,
  inbound-handshake, unidirectional-stream admission, setup/open-stream
  timeout, unauthenticated and idle inbound admission, future/overflowing
  timestamp, replay-cache, peer-id mismatch, inbound out-of-range stream
  service id, inbound response-write permit, and 24-node valid fuzz checks.
  Rejected candidates mapped to ISSUE-002/021/146/176/207 and RC-1 for
  handshake freshness/replay/cache pressure, ISSUE-189/194/223/244 and RC-1
  for role/peer-id/self/third-party/static-binding behavior,
  ISSUE-117/172/173/220/223 and RC-3/RC-4 for QUIC/admission caps,
  ISSUE-217/220/238 and RC-4 for malformed or stalled setup cleanup, and
  ISSUE-014/018/RC-1 for post-auth source forgery. Direct issue-entry ledger
  check found 19 score-80+ entries and all are fixed; no reproducible
  secure-handshake/QUIC setup failure supported a distinct score-80+ issue.
- Cycle after ISSUE-231 no-new cycle 1 reviewed routing/discovery/path
  stability and stream/pipe lifecycle integration with forked reviewer
  `Carver`. Focused route, discovery, stream-relay, peer-stopped, and pubsub
  graceful-stop tests passed. Rejected candidates mapped to ISSUE-003/RC-7,
  ISSUE-004, ISSUE-051, ISSUE-063, ISSUE-117, ISSUE-149, ISSUE-156,
  ISSUE-164, ISSUE-167, ISSUE-170, ISSUE-215 through ISSUE-222, ISSUE-229,
  and ISSUE-230; no new root-cause summary change was needed.
- Cycle after ISSUE-234 no-new cycle 1 reviewed routing/path stability,
  stream relay setup, and graceful-stop/non-seed lifecycle cleanup with forked
  reviewer `Planck`. Focused route hysteresis/direct-route, stream-relay,
  peer-stopped, non-seed expiry, seed retention, and stopped-tombstone tests
  passed. Rejected candidates mapped to ISSUE-003/RC-7, ISSUE-004, ISSUE-051,
  ISSUE-063, ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-167, ISSUE-170,
  ISSUE-215 through ISSUE-225, ISSUE-229, and ISSUE-230; no new root-cause
  summary change was needed.
- Cycle after ISSUE-235 no-new cycle 1 reviewed shared-key handshake,
  inbound peer binding, QUIC stream admission, and peer setup timeout behavior
  with forked reviewer `Halley`. Focused handshake, inbound-handshake,
  unidirectional-stream, outbound ConnectReq stall, inbound ConnectRes stall,
  and main-control-open timeout tests passed. Rejected candidates mapped to
  ISSUE-002, ISSUE-021, ISSUE-117, ISSUE-146, ISSUE-172, ISSUE-173,
  ISSUE-176, ISSUE-189, ISSUE-194, ISSUE-207, and existing QUIC uni-stream cap
  coverage; no new root-cause summary change was needed.
- Cycle after ISSUE-235 no-new cycle 2 reviewed requester/connect admission,
  public service-handle/requester boundaries, duplicate-service rejection,
  graceful shutdown, and seed/non-seed lifecycle cleanup with forked reviewer
  `Wegener`. Focused connect, graceful, and non-seed expiry tests passed.
  Rejected candidates mapped to ISSUE-028, ISSUE-030, ISSUE-052, ISSUE-053,
  ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-091, ISSUE-125,
  ISSUE-167, ISSUE-215 through ISSUE-225, ISSUE-234, RC-3, and RC-6; no new
  root-cause summary change was needed.

### RC-3: Backpressure is inconsistent across async boundaries

- Representative issues: ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-123,
  ISSUE-124, ISSUE-125, ISSUE-126,
  ISSUE-127, ISSUE-136, ISSUE-153,
  ISSUE-178, ISSUE-182, ISSUE-184, ISSUE-198, ISSUE-199,
  ISSUE-200, ISSUE-201, ISSUE-202, ISSUE-203, ISSUE-204, ISSUE-209,
  ISSUE-223, ISSUE-224, ISSUE-225, ISSUE-227, ISSUE-229, ISSUE-230,
  ISSUE-235, ISSUE-246.
- ISSUE-235, score 60: fixed by `5b0fc47`. `AliasServiceRequester::register`
  now returns `Result<AliasGuard>` and creates the ownership guard only after
  bounded alias-control admission succeeds. Verification:
  `cargo test alias_register_when_control_queue_full_must_not_return_live_guard -- --nocapture`,
  `cargo test alias -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/alias_service.rs src/tests/alias.rs`,
  and `git diff --check`.
- ISSUE-209: fixed high-load fuzz coverage issue. The fuzz harness silently
  capped `P2P_FUZZ_NODES` values above 8, so intended 12-15 node fuzz cycles
  executed with only 8 nodes. The fix keeps the lower bound at two nodes while
  removing the hidden upper cap. Verification:
  `cargo test fuzz_node_count_must_honor_high_load_configuration -- --nocapture`
  and
  `P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=1 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`.
- ISSUE-223, score 68: fixed by commit `770063f`. Authenticated inbound peers
  whose `PeerConnected` events were backpressured still consumed the
  unauthenticated inbound admission cap because neighbour accounting was only
  updated when the main loop drained `PeerConnected`. Fix: exclude neighbours
  with an already-registered `SharedCtx` alias from the unauthenticated
  admission counter.
- ISSUE-224, score 70: fixed by commit `c893230`. Local fire-and-forget `Broadcast`/`Unicast`
  delivery awaits bounded service queue capacity inside the peer read loop; a
  full service queue can therefore delay a later `PeerStopped` frame on the
  same connection until `LOCAL_SERVICE_DELIVERY_TIMEOUT`. Fix: enqueue
  fire-and-forget local delivery into a bounded per-connection worker, closing
  the connection on worker queue overflow; keep `UnicastWithAck` awaited so
  sender-visible acknowledgements still represent service admission.
- ISSUE-225, score 69: fixed by commit `d1359e9`. Local `UnicastWithAck`
  still awaits bounded destination service admission inside the peer read loop;
  with a full service queue this delays later `PeerStopped` frames on the same
  connection. Fix: enqueue acked local delivery into the bounded
  per-connection local delivery worker with ack metadata, then send the only
  ack from that worker after service admission succeeds or fails.
- ISSUE-227, score 64: fixed by `c0d7616`. `SharedCtx::send_broadcast`
  bounded each peer-alias send but awaited those bounds sequentially, so
  high-load fanout still took `N * BROADCAST_ADMISSION_TIMEOUT` when many peer
  queues were congested. The fix polls all bounded admissions concurrently
  while preserving zero-accepted error semantics.
- ISSUE-229, score 68: fixed by `f87c6dc`. Public awaited `send_unicast` now
  uses acked delivery for relayed destinations too; relay nodes forward
  `UnicastWithAck` downstream and propagate the downstream result to the
  original upstream ack id, so final destination service failures no longer
  look like first-hop success.
- ISSUE-230, score 68: fixed by `2358c31`. `SendUnicastWithAck` now expires
  stale pending ack entries, rejects overflow at `MAX_PENDING_UNICAST_ACKS = 16`
  before writing another frame, and reports `pending unicast ack queue full` to
  the caller instead of letting the pending ack map scale with write throughput.
- Cycle after ISSUE-230 no-new cycle 1: reviewed the issue ledger and summary,
  then audited public requester/control admission, direct and relayed
  `UnicastWithAck`, unicast ingress-loop/source handling, stream relay/setup,
  and recent shutdown/discovery/routing hints. Forked reviewer `Aquinas`
  rejected new issue acceptance: requester/control maps to existing bounded
  admission coverage, direct/relayed acked unicast maps to ISSUE-119,
  ISSUE-224, ISSUE-225, ISSUE-229, and ISSUE-230, unicast relay loops/source
  binding map to ISSUE-014 and ISSUE-197, and stream relay/setup maps to
  ISSUE-011, ISSUE-012, ISSUE-013, ISSUE-018, ISSUE-056, ISSUE-117, ISSUE-149,
  ISSUE-156, ISSUE-169, ISSUE-180, ISSUE-217, and ISSUE-220. Verification:
  `cargo test requester_connect -- --nocapture`,
  `cargo test relayed_open_stream_must_not_succeed_before_downstream_accepts -- --nocapture`,
  `cargo test unicast_relay_must_not_forward_back_to_ingress_peer -- --nocapture`,
  and
  `P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=200 P2P_FUZZ_SEED=23001 cargo test fuzz_random_steady_valid_node_actions_must_not_panic_connection_tasks -- --nocapture`
  passed. No new root cause beyond RC-3, RC-4, RC-6, and RC-7.
- Cycle after ISSUE-230 no-new cycle 2: reviewed shared-key handshake and
  inbound binding setup, base `P2pService`/`SharedCtx` service-id, liveness,
  broadcast/unicast/backpressure, disconnect behavior, and replicated-KV
  snapshot/repair/lifecycle/resource logic. Forked reviewer `Bacon` rejected
  new issue acceptance: auth maps to ISSUE-002, ISSUE-146, ISSUE-176,
  ISSUE-189, ISSUE-194, and ISSUE-223; service context behavior maps to
  ISSUE-052, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-119,
  ISSUE-120, ISSUE-198, ISSUE-199, ISSUE-224, ISSUE-225, ISSUE-227,
  ISSUE-229, and ISSUE-230; disconnect/stopped lifecycle maps to ISSUE-136,
  ISSUE-144, ISSUE-162, and ISSUE-215 through ISSUE-222; replicated-KV maps
  to ISSUE-023, ISSUE-027, ISSUE-045, ISSUE-059, ISSUE-081 through ISSUE-089,
  ISSUE-095, ISSUE-099, ISSUE-110, ISSUE-111, ISSUE-131, ISSUE-138,
  ISSUE-140, ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-162, ISSUE-171,
  ISSUE-175, ISSUE-184, ISSUE-186, and ISSUE-196. Verification:
  `cargo test handshake -- --nocapture`,
  `cargo test inbound_handshake -- --nocapture`,
  `cargo test service_requester -- --nocapture`,
  `cargo test get_service_must_reject_out_of_range_id_without_panicking -- --nocapture`,
  `cargo test duplicate_service_creation_must_not_panic -- --nocapture`,
  and `cargo test replicate_kv -- --nocapture` passed. No new root cause beyond
  RC-1, RC-2, RC-3, RC-5, RC-6, and RC-7.
- Cycle after ISSUE-225 no-new cycle 1: reviewed the issue ledger, ack-worker
  backpressure, peer-control admission, stream setup, graceful-stop cleanup,
  and route/path-jumping surfaces. A 12-node, 200-step steady valid fuzz run
  passed. Duplicate-connection, connection-lost, and
  `local service delivery ack channel ended` logs were reviewed as shutdown or
  bounded-backpressure fallout; a focused healthy acked-unicast burst probe
  passed after avoiding intentional peer-control queue overflow. No new root
  cause beyond RC-3, RC-4, RC-6, and RC-7.
- Cycle after ISSUE-225 no-new cycle 2: forked reviewer `Maxwell` reviewed
  ack-worker/control-frame paths, stream accept/relay/open behavior, route
  stability, graceful shutdown, broadcast/unicast backpressure, and the issue
  ledger. A 12-node, 400-step steady valid fuzz run with seed 22501 passed.
  Duplicate-connection closes, connection-lost logs, and endpoint teardown
  messages mapped to existing churn, lifecycle-cleanup, or bounded
  backpressure fallout. No new root cause beyond RC-3, RC-4, RC-6, and RC-7.
- Cycle after ISSUE-225 no-new cycle 3: forked reviewer `Cicero` reviewed
  post-225 ack/local-service worker behavior, stream setup, route hysteresis
  and direct-route preference, discovery seed/non-seed lifecycle, main-loop
  lifecycle validation, and fuzz harness coverage. A 12-node, 600-step steady
  fuzz pass and a 12-node, 250-step sanitized-churn fuzz pass with seed 22503
  passed. Shutdown/refused-connection and teardown logs mapped to existing
  lifecycle or bounded-backpressure fallout. No new root cause beyond RC-3,
  RC-4, RC-6, and RC-7.
- Cycle after ISSUE-225 no-new cycle 4: forked reviewer `Lagrange` reviewed
  seed/non-seed discovery lifecycle, stopped tombstones, graceful shutdown,
  `PeerConnected` and `PeerStopped` ordering/backpressure, stream/pipe setup,
  and route/path stability. Focused `PeerConnected`/admission tests and a
  12-node, 800-step steady fuzz run with seed 22504 passed. A possible
  delayed-`PeerConnected` versus fast-`PeerStopped` ordering candidate was
  rejected as scheduler-sensitive and overlapping existing issues. No new root
  cause beyond RC-3, RC-4, RC-6, and RC-7.
- Cycle after ISSUE-225 no-new cycle 5: forked reviewer `Carson` reviewed
  post-225 acked local unicast worker behavior, `PeerStopped` lifecycle under
  backpressure, route/path selection, seed/non-seed discovery lifecycle,
  stream/pipe setup, and fuzz coverage. Steady, sanitized-churn, valid-churn,
  ISSUE-225, closed-service-ack, and `peer_stopped_` checks passed. Duplicate
  mappings stayed in RC-3, RC-4, RC-6, and RC-7. This satisfies the requested
  five consecutive no-new cycles after ISSUE-225.
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
  ISSUE-040, ISSUE-042, ISSUE-093, ISSUE-117, ISSUE-121, ISSUE-149,
  ISSUE-169, ISSUE-172, ISSUE-173, ISSUE-176, ISSUE-207, ISSUE-220,
  ISSUE-236.
- Pattern: timeouts wrap only one await point, rely on unchecked timestamp
  arithmetic, use coarse global sweeps, or complete one side of setup before
  proving the end-to-end setup is still alive. Public timer durations can also
  reach Tokio interval construction without non-zero validation. Handshake
  tokens also lack nonce/challenge binding or replay caches.
- Minimal fix proposal: use checked/saturating deadline math, wrap every
  protocol phase in an end-to-end timeout, and tie relay downstream setup to
  upstream cancellation. Bind handshake responses to fresh request nonces and
  reject recently accepted tokens until expiry.
- ISSUE-220, score 66: fixed by commit `881c087`. Accept-side
  `StreamConnectRes` writes could
  await while holding one of the 16 inbound stream setup permits. A peer that
  sends valid setup requests and then stops reading responses can exhaust that
  connection's accept permits and prevent later stream setup on the same
  connection. Fix: use a bounded timeout for every accept-side setup response
  write and return an error on timeout so the permit is released.
- ISSUE-221, score 72: fixed by commit `4758786`. After accepting a direct
  peer's `PeerStopped`, the main loop removed route/neighbour state but the
  connection task could keep processing frames from the still-open connection.
  Authenticated direct broadcasts could therefore reach local services after
  graceful stop was accepted. Fix: close and exit the connection task after an
  admitted or duplicate already-delivered stop while preserving the full-queue
  retry path.
- ISSUE-036: fixed by routing alias find hint and scan timeout checks through
  checked deadline arithmetic. Deadlines that overflow `u64` remain pending
  instead of panicking or wrapping into early expiry. Verification:
  `cargo test find_timeout_at_max_timestamp_must_not_overflow -- --nocapture`.
- ISSUE-042: fixed by using checked visualization peer-timeout deadline
  arithmetic. Overflowing or unrepresentable deadlines remain not expired
  instead of panicking or wrapping into early expiry. Verification:
  `cargo test visualization_peer_timeout_deadline_must_not_overflow -- --nocapture`.
- ISSUE-236, score 63: fixed by `13f3a67`. Pubsub publish/feedback RPC
  deadline calculation now uses `Instant::checked_add`; unrepresentable huge
  caller-supplied timeouts are excluded from the scheduler instead of panicking
  the service loop. Verification:
  `cargo test pubsub_rpc_huge_timeout_must_not_panic_deadline_calculation -- --nocapture`,
  `cargo test pubsub_publish_rpc_must_respect_short_timeout -- --nocapture`,
  `cargo test pending_ -- --nocapture`,
  `cargo test pubsub_outbound_heartbeat_batches_must_respect_inbound_cap -- --nocapture`,
  `cargo test pubsub -- --nocapture`,
  `rustfmt --edition 2021 --check src/service/pubsub_service.rs`, and
  `git diff --check`.
- ISSUE-040: fixed by normalizing `Some(Duration::ZERO)` to each service's
  default collection interval before constructing Tokio timers. Visualization
  stores the normalized option so `None` still disables collection. Verification:
  `cargo test metrics_service_zero_collect_interval_must_not_panic -- --nocapture`
  and
  `cargo test visualization_service_zero_collect_interval_must_not_panic -- --nocapture`.

### RC-5: Application-level resource limits are missing

- Representative issues: ISSUE-010, ISSUE-024, ISSUE-027, ISSUE-035,
  ISSUE-041, ISSUE-043, ISSUE-045, ISSUE-046, ISSUE-100 through ISSUE-108,
  ISSUE-122, ISSUE-131, ISSUE-174, ISSUE-196, ISSUE-228.
- Pattern: decoded service-level collections, pending maps, cache sets,
  tombstones, remote stores, retained channel state, and outbound event queues
  often have no item-count or lifetime cap.
- Minimal fix proposal: add per-structure caps with deterministic
  eviction/rejection: max rows per message, max peers per alias/channel, max
  pending RPCs/finds, max tombstones/remotes, max queued outbound events, and
  prune empty channel state on teardown. Mutation APIs that enqueue work should
  return backpressure errors or coalesce superseded work.
- ISSUE-035: fixed by capping duplicate alias find waiters per alias at
  `MAX_WAITERS_PER_ALIAS = 1024`. Overflow duplicate `Find` callers complete
  immediately with `None` and do not add another waiter or scan/check fanout.
  Verification:
  `cargo test duplicate_find_waiters_for_same_alias_must_be_bounded -- --nocapture`.
- ISSUE-041: fixed by capping distinct pending alias finds at
  `MAX_PENDING_FIND_REQUESTS = 1024`. The cap applies only to new remote lookup
  work after duplicate coalescing and local hits; overflow callers receive
  `None` without scan/check fanout or metrics increments. Verification:
  `cargo test distinct_pending_find_requests_must_be_bounded -- --nocapture`.
- ISSUE-228, score 62: fixed by `f9fd337`. Pubsub heartbeat senders now split
  outbound heartbeat rows into batches no larger than
  `MAX_HEARTBEAT_CHANNELS_PER_BATCH`, so high-channel-count nodes do not emit
  heartbeats that fixed receivers drop under the inbound cap.

### RC-6: Lifecycle cleanup and stale handles are inconsistent

- Representative issues: ISSUE-028, ISSUE-029, ISSUE-051, ISSUE-057,
  ISSUE-060, ISSUE-064, ISSUE-065, ISSUE-069 through ISSUE-076, ISSUE-108,
  ISSUE-128 through ISSUE-132, ISSUE-135, ISSUE-139, ISSUE-142, ISSUE-144,
  ISSUE-148, ISSUE-150, ISSUE-151, ISSUE-161, ISSUE-165,
  ISSUE-167, ISSUE-168, ISSUE-170, ISSUE-179, ISSUE-183, ISSUE-185,
  ISSUE-187, ISSUE-188, ISSUE-193, ISSUE-195, ISSUE-208, ISSUE-222,
  ISSUE-234, ISSUE-235, ISSUE-246.
- Pattern: requesters, services, peer aliases, channel state, and cached hints
  can outlive the owner they represent; shutdown paths can panic, leak, emit
  false public events, keep stale routes/cache entries, announce shutdown while
  local authority remains active, or drop remote membership that arrives before
  local channel ownership exists. Peer lifecycle events also do not consistently
  reach service-owned per-peer membership or public network-event consumers.
- ISSUE-234, score 66: fixed by `54f1118`. Rejected duplicate service creation
  now marks the returned `P2pService` as unregistered, and both direct service
  sends and cloned requesters fail before using the shared context. Verification:
  `cargo test duplicate_service_creation_must_not_return_live_unregistered_sender -- --nocapture`,
  `cargo test duplicate_service_creation_must_not_panic -- --nocapture`,
  `cargo test dropped_service_id_must_be_reusable -- --nocapture`,
  `cargo test service_requester -- --nocapture`, and
  `cargo test duplicate_service -- --nocapture`.
- ISSUE-235, score 60: fixed by `5b0fc47`. Alias registration now reports
  admission failure instead of returning a live-looking guard when the bounded
  alias control queue is full or closed. This is the alias-registration version
  of the stale/false-success handle class. Verification:
  `cargo test alias_register_when_control_queue_full_must_not_return_live_guard -- --nocapture`
  and `cargo test alias -- --nocapture`.
- ISSUE-246, score 54: fixed pubsub registration overflow false success.
  Root cause: publisher/subscriber construction logged bounded queue-full
  registration failures but returned requesters with live control senders, so
  later actions could enqueue and return success even though service state had
  never registered the handle. Smallest fix: store a registration-admitted bit
  in returned requesters, fail later actions for never-registered handles, and
  skip teardown controls for handles that never entered service state.
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
  monotonic counters during teardown. Local guard/reference counts should use a
  counter type that can represent all admitted handles and must not saturate
  silently.
- ISSUE-222, score 64: fixed by commit `648f769`. Accepted
  `PeerStopped` handling removed route and neighbour state but left the
  connection alias registered in `SharedCtx` until peer-task teardown. Local
  fanout paths that iterate `ctx.conns()` could therefore target a stopped peer
  after it had already been reported disconnected. Fix: unregister the
  connection alias in the accepted `PeerStopped` branch after route ownership
  validation and before service disconnect notification.

### RC-7: Routing/discovery accepts unstable topology

- Representative issues: ISSUE-003, ISSUE-005, ISSUE-006, ISSUE-007,
  ISSUE-008, ISSUE-033, ISSUE-044, ISSUE-055, ISSUE-092, ISSUE-103,
  ISSUE-112 through ISSUE-114, ISSUE-164, ISSUE-167,
  ISSUE-177, ISSUE-180, ISSUE-181, ISSUE-190, ISSUE-192, ISSUE-197,
  ISSUE-210, ISSUE-211, ISSUE-212.
- Pattern: route/discovery inputs can include local ids, self seeds, stale
  addresses, overflowed metrics, over-hop routes, duplicate connection races,
  explicit connect addresses that are ignored by peer-id-only fast paths, or
  tiny RTT jitter that changes active paths too aggressively. Malformed route
  or discovery syncs can also contain duplicate destination rows whose last
  value silently wins before validation. Stream relay setup and unicast
  forwarding can also forward back to the ingress connection when route state
  forms a loop. Local advertise config and remote discovery syncs can also
  gossip non-dialable addresses. Configured seed addresses can be available for
  local dialing but omitted from outbound discovery syncs, preventing
  downstream bootstrap peers from converging to direct neighbours. Local
  advertise rows can also be starved by learned remotes under the outbound sync
  cap, hiding the sender's own dial address in large discovery tables.
- ISSUE-210: fixed remote discovery input validation issue. Unlike local
  advertise config, `PeerDiscovery::apply_sync` accepts `0.0.0.0:0` and
  port-zero remote rows as dial candidates. The fix rejects non-dialable
  remote discovery rows with the existing dialability predicate before
  tombstone mutation or insertion. Verification:
  `cargo test apply_sync_must_reject_non_dialable_remote_addresses -- --nocapture`
  and `cargo test discovery::test:: --lib -- --nocapture`.
- ISSUE-211: accepted configured-seed outbound gossip issue. A node includes
  configured seeds in local dial candidates but omits them from
  `PeerDiscovery::create_sync_for`, so a downstream peer can learn only a
  relayed route and never the stable seed dial address. Evidence:
  `cargo test tests::discovery::discovery_remain_node -- --nocapture`
  fails with node3 having one neighbour instead of two. Minimal fix proposal:
  include configured seeds in outbound discovery sync using the current sync
  timestamp, filtered for destination peer, local peer id, and dialable address,
  while preserving receiver-side seed-authority checks in `apply_sync`. Mesh
  convergence also requires route-aware broadcast relay handling so stale
  relayed copies are dropped once a direct route to the original source exists.
- ISSUE-212: accepted local-advertise cap starvation issue. After ISSUE-211,
  `PeerDiscovery::create_sync_for` prioritizes configured seeds but still emits
  learned remotes before the local advertise row, so a large learned-remote
  table can consume `MAX_SYNC_ENTRIES` before the sender's own dial address is
  included. Evidence:
  `cargo test create_sync_for_must_prioritize_local_advertise_under_cap --lib -- --nocapture`
  fails with the local row absent. Minimal fix proposal: emit local advertise
  before learned remotes while preserving seed priority, destination filtering,
  local-peer seed duplicate filtering, dialability checks, and the cap.
- ISSUE-213: fixed duplicate-seed cap starvation issue. A config with
  repeated seed entries can fill `PeerDiscovery::create_sync_for`'s capped
  outbound sync before the local advertise row is emitted. Evidence:
  `cargo test create_sync_for_must_deduplicate_configured_seeds_before_cap --lib -- --nocapture`
  fails with the local row absent. Minimal fix proposal: deduplicate configured
  seed rows by peer id before applying `MAX_SYNC_ENTRIES`, while preserving
  seed priority, destination/local-peer/dialability filters, and the existing
  local-before-remotes ordering. Implemented by per-sync seed peer-id dedup
  after the existing filters and before the cap.
- ISSUE-214: fixed direct-route cap starvation issue. `RouterTable::create_sync`
  previously emitted eligible routes in peer-id order, so a large learned-route
  table could consume `MAX_SYNC_ENTRIES` before high-peer-id direct neighbours
  were advertised. Evidence:
  `cargo test create_sync_must_prioritize_direct_routes_under_cap --lib -- --nocapture`
  fails with the direct route absent. Minimal fix proposal: keep existing route
  filters and cap, but emit direct `relay_hops == 0` routes before learned
  relay paths.
- ISSUE-215: fixed PeerStopped retry suppression issue. A legitimate
  graceful-stop message can be marked as deduplicated before
  `MainEvent::PeerStopped` is admitted to the bounded main queue; if
  `try_send` fails under backpressure, later retries are dropped. Evidence:
  `cargo test peer_stopped_dedup_must_not_suppress_retry_after_main_queue_backpressure --lib -- --nocapture`
  fails because the retry never reaches the main event queue. Minimal fix
  proposal: mark a stopped peer as delivered only after local `PeerStopped`
  admission succeeds, preserving forged-stop rejection and mesh forwarding
  dedup behavior. Implemented by replacing the mutating check with a
  mark-after-success helper and forwarding only after local admission succeeds.
- ISSUE-216: fixed PeerStopped lifecycle dedup issue. After a successful
  graceful-stop admission, `SharedCtx` keeps the peer id in its stopped-message
  dedup cache until LRU eviction. If the same peer id reconnects and later
  stops again, the new lifecycle's stop can be suppressed. Evidence:
  `cargo test peer_stopped_admission_must_not_suppress_new_peer_lifecycle --lib -- --nocapture`
  fails after registering a new alias for the same peer id. Minimal fix
  proposal: clear the peer-stopped dedup entry when a new connection lifecycle
  is registered for that peer id, preserving same-lifecycle duplicate
  suppression. Implemented in `SharedCtxInternal::register_conn`.
- ISSUE-217: fixed relayed stream setup correctness issue. In the relayed
  stream branch, `accept_bi` writes upstream `Ok(())` before
  `alias.open_stream(...)` completes, so callers can receive
  `Ok(P2pQuicStream)` while the downstream peer withholds `StreamConnectRes`.
  Evidence:
  `cargo test relayed_open_stream_must_not_succeed_before_downstream_accepts --lib -- --nocapture`
  fails because the caller observes success before downstream acceptance.
  Minimal fix proposal: reject an already-stopped upstream setup response side
  before opening downstream, open downstream first, write upstream `Err` on
  downstream failure, and write upstream `Ok` only after downstream success.
  Implemented in `accept_bi` with a `P2pQuicStream::write_stopped` guard.
- ISSUE-218: fixed inbound sync head-of-line blocking issue. ISSUE-147
  preserved valid route/discovery syncs under main-queue backpressure by using
  `self.main_tx.send(...).await`, but that awaited bounded send runs inside the
  peer connection read loop. Evidence:
  `cargo test sync_must_not_block_connection_task_on_full_main_queue --lib -- --nocapture`
  fails with both a later unicast send and destination receive timing out after
  a `Sync` frame hits a full main queue. Minimal fix proposal: preserve sync by
  moving bounded waiting out of `on_msg`, using a per-connection pending or
  coalesced async sync-delivery task so later frames can still be read.
  Implemented with one coalescing inbound-sync worker per connection and a
  single latest-sync pending slot.
- ISSUE-219: fixed live outbound main-control write head-of-line blocking
  issue. After authentication, `PeerConnectionInternal::on_control` awaits
  `self.framed.send(...)` for queued control messages inside the same peer
  task that reads inbound frames. Evidence:
  `cargo test outbound_control_send_must_not_block_peer_read_loop --lib -- --nocapture`
  fails after a raw peer stops reading the main control stream and then sends
  `PeerStopped`; the inbound stop is not processed promptly. Minimal fix
  proposal: move live outbound main-control writes out of the read/control
  loop into a bounded writer path, or wrap live `framed.send(...)` with a
  bounded timeout and close/report failure on timeout. Implemented with a
  bounded `send_control_frame` helper for post-auth live main-control sends.
- Minimal fix proposal: sanitize before insertion: reject local/self candidates
  and over-hop routes, pin authenticated direct paths for their peer ids, use
  checked metric math, ignore stale discovery timestamps, reject duplicate
  destination rows in one route or discovery sync, coalesce duplicate connects,
  validate already-connected peer addresses, add hysteresis before switching
  active paths, and reject relay stream or unicast hops that point back to the
  ingress connection. Validate configured local advertise addresses and remote
  discovery row addresses before gossiping or storing them.

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

- ISSUE-060: fixed by allowing `SharedCtxInternal::set_service` to replace a
  registered service sender only after the previous receiver is closed.
  Verification:
  `cargo test dropped_service_id_must_be_reusable -- --nocapture`.
- ISSUE-030: fixed by rejecting duplicate live service registrations without
  asserting inside `SharedCtxInternal::set_service`. Verification:
  `cargo test duplicate_service_creation_must_not_panic -- --nocapture`.
- ISSUE-052: fixed by validating `P2pServiceId::as_service_index()` before
  indexing the service table and rejecting out-of-range registrations without
  unwinding. Verification:
  `cargo test out_of_range_service_id_must_not_panic -- --nocapture`.
- ISSUE-076: fixed by checking the `P2pServiceRequester` liveness token before
  requester broadcast sends can reach `SharedCtx`. Verification:
  `cargo test dropped_service_requester_must_not_continue_sending_broadcast -- --nocapture`.
- ISSUE-072: fixed by checking the `P2pServiceRequester` liveness token before
  requester unicast sends can reach `SharedCtx`. Verification:
  `cargo test dropped_service_requester_must_not_continue_sending_unicast -- --nocapture`.
- ISSUE-073: fixed by giving cloned `P2pServiceRequester`s a liveness token
  tied to the owning `P2pService` receiver and rejecting stale `open_stream`
  attempts before they can reach routing. Verification:
  `cargo test dropped_service_requester_must_not_continue_opening_streams -- --nocapture`.
- ISSUE-062: fixed by correlating collector-side metrics `Info` frames with
  an outstanding scan window. On each collector scan tick, the service records
  current direct peers as expected responders; unicast `Info` is accepted only
  once from a pending responder and unsolicited forged `Info` is ignored. This
  keeps the wire protocol unchanged and does not claim nonce protection against
  an in-window race. Verification:
  `cargo test metrics_info_must_not_be_accepted_without_scan_request -- --nocapture`,
  `cargo test metrics_scan_must_not_disclose_metrics_to_non_collector -- --nocapture`,
  and `cargo test metric_collect -- --nocapture`.
- ISSUE-061: fixed by applying the same scan-responder correlation to
  visualization topology `Info`. Collectors seed expected responders from live
  connections on scan, ignore broadcast `Info`, and accept unicast `Info` only
  once from pending responders. Completed scan-response tasks are drained so
  repeated scans keep normal topology updates flowing. Verification:
  `cargo test visualization_info_must_not_be_accepted_without_scan_request -- --nocapture`,
  `cargo test tests::visualization::discovery_new_node -- --nocapture`, and
  `cargo test visualization_scan_must_not_disclose_topology_to_non_collector -- --nocapture`.
- ISSUE-045: fixed by adding a 1,024-entry admission cap for replicated-KV
  remote stores. New remote identities first trigger the existing timeout
  cleanup path; if the cap is still full, their event is rejected before
  `RemoteStore::new` queues full-sync work. Existing remotes continue to be
  processed, and active remotes are not arbitrarily evicted. Verification:
  `cargo test remote_store_creation_must_be_bounded -- --nocapture`.
- ISSUE-023: fixed by checking replicated-KV `FetchChanged` range arithmetic
  at the untrusted RPC boundary. Overflowing `from + count` or `last + 1`
  bounds now return `FetchChangedError::MissingData` instead of panicking in
  debug builds or wrapping in release builds. This leaves local version
  exhaustion policy to ISSUE-031. Verification:
  `cargo test fetch_changed_with_overflowing_from_version_must_not_panic -- --nocapture`.
- ISSUE-031: fixed by allocating local replicated-KV successor versions with
  checked arithmetic. Local `set`/`del` at `Version(u64::MAX)` now no-op before
  visible state changes or event emission; delete preserves an existing slot
  when no successor version exists. Verification:
  `cargo test local_set_at_max_version_must_not_overflow -- --nocapture` and
  `cargo test local_del_at_max_version_must_not_overflow_or_remove_slot -- --nocapture`.
- ISSUE-032: fixed by normalizing `LocalStore::new(..., compose_max_pkts)` to a
  minimum page size of one. Zero public compose budgets now make one-item
  progress for snapshots and `FetchChanged { count: 1 }`, while requested count
  zero remains rejected. Verification:
  `cargo test snapshot_with_zero_compose_budget_must_make_progress -- --nocapture`
  and `cargo test service::replicate_kv_service::local_storage::tests -- --nocapture`.
- ISSUE-117: fixed by bounding inbound stream-connect setup at both transport
  and application layers. QUIC now admits at most one main control stream plus
  16 application bidirectional streams per peer direction, while
  `PeerConnectionInternal` also keeps only 16 pending inbound stream-connect
  setup tasks and times out peers that never send the initial
  `StreamConnectReq`. Root cause was a 10,000-stream transport cap combined
  with one unbounded spawned task per idle accepted stream. Verification:
  `cargo test idle_inbound_stream_connects_must_be_admission_bounded -- --nocapture`
  and `cargo test test_open_stream -- --nocapture`.
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
- ISSUE-050: fixed by making relayed `send_unicast` use nonblocking
  peer-control admission via `PeerConnectionAlias::try_send`, while keeping the
  direct-route unicast ack path for destination service admission. Congested
  relay control queues now return promptly instead of parking the caller.
  Verified with
  `cargo test send_unicast_must_not_block_on_full_peer_control_queue -- --nocapture`
  and
  `cargo test send_unicast_to_relay_must_not_block_on_full_peer_control_queue -- --nocapture`.
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
- ISSUE-022: fixed by routing remote `AliasMessage::Shutdown` through the
  peer-scoped disconnect cleanup path. A stopped peer now removes only its own
  alias lifecycle/cache hints, shared aliases retain other peers, and unrelated
  cached hints survive. Verification:
  `cargo test shutdown_from_one_peer_must_not_clear_aliases_from_other_peers -- --nocapture`
  and
  `cargo test shutdown_from_cached_hint_must_unblock_pending_find -- --nocapture`.
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
- ISSUE-177: fixed by the same ISSUE-153 `process_connect` path.
  `NetworkNeighbours::has_peer_connection_attempt` is checked before
  `endpoint.connect` and matches existing connected peers or pending outbound
  attempts for the requested peer id. Awaited connects to the same peer id at a
  different socket now return `Err(_)` instead of synthetic success, while
  best-effort duplicates coalesce. Verification:
  `cargo test connect_to_same_peer_id_at_different_address_must_not_report_success -- --nocapture`,
  `cargo test awaited_connect_must_error_while_same_peer_connect_is_pending -- --nocapture`,
  and
  `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`.
- ISSUE-154: fixed by `55b79e5` (`fix: continue partial kv repair
  responses`). `WorkingState::on_rpc_res` now accepts `FetchChanged` success
  only for an active pending `FetchChanged { from, count }`, validates returned
  versions against the pending range, rejects duplicates and zero-count pending
  repairs, and sends a follow-up request for the remaining range after a valid
  partial response. This also closes ISSUE-086 because unsolicited
  `FetchChanged` success responses are rejected before slot/version mutation or
  event emission, and closes ISSUE-087 because unsolicited `FetchChanged` error
  responses are rejected before full-sync fallback or local delete events.
  Verification:
  `cargo test working_state_must_reject_unsolicited_fetch_changed_success -- --nocapture`,
  `cargo test working_state_must_reject_unsolicited_fetch_changed_error -- --nocapture`,
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
- ISSUE-156 regression evidence, score 66: fixed by `fedfa0e`. The previous
  ordering fix still allowed flow-control-stalled upstream setup acknowledgements
  to orphan downstream relayed streams. Relayed opens now set
  `StreamConnectReq.defer_delivery`; final local delivery waits for a relay
  commit, and each relay forwards upstream commit/error to the next hop before
  starting `copy_bidirectional`. Verification:
  `RUST_LOG=error cargo test relay_must_not_deliver_downstream_stream_when_upstream_setup_ack_stalls --lib -- --nocapture`,
  `RUST_LOG=error cargo test multihop_relay_must_not_deliver_downstream_stream_when_upstream_setup_ack_stalls --lib -- --nocapture`,
  `RUST_LOG=error cargo test relayed_open_stream_must_not_succeed_before_downstream_accepts --lib -- --nocapture`,
  `RUST_LOG=error cargo test relay_must_not_deliver_downstream_stream_after_upstream_setup_closes --lib -- --nocapture`,
  `RUST_LOG=error cargo test stream --lib -- --nocapture`,
  `rustfmt --edition 2021 --check src/msg.rs src/peer.rs src/peer/peer_alias.rs src/peer/peer_internal.rs src/ctx.rs src/tests/stream.rs`,
  and `git diff --check`.
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
- ISSUE-111: fixed by the same pending-request validation and remaining-range
  continuation path. Empty successful `FetchChanged` responses leave
  `self.version` below the requested upper bound, so the worker immediately
  emits a follow-up `FetchChanged` instead of canceling repair. Verification:
  `cargo test working_state_must_not_cancel_repair_after_empty_fetch_changed_success -- --nocapture`,
  `cargo test working_state_must_continue_repair_after_partial_fetch_changed_success -- --nocapture`,
  and
  `cargo test working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair -- --nocapture`.
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
- ISSUE-038: fixed by rejecting empty snapshot pages that still carry
  `next_key`, so full sync cannot loop forever on non-progressing
  continuations. Verification:
  `cargo test full_sync_must_reject_empty_snapshot_page_with_next_key -- --nocapture`.
- ISSUE-025: fixed by validating resolved local snapshot bounds before
  `BTreeMap::range`; reversed untrusted `FetchSnapshot` bounds now return
  `None` instead of panicking. Verification:
  `cargo test fetch_snapshot_with_reversed_bounds_must_not_panic -- --nocapture`.
- ISSUE-059: fixed by rejecting `FetchSnapshot(None, version)` terminal
  responses while a full-sync continuation request is pending. The pending
  request stays intact for timeout retry, so a partial snapshot cannot be
  silently completed with missing data. Verification:
  `cargo test full_sync_must_reject_none_continuation_after_partial_snapshot -- --nocapture`.
- ISSUE-110: fixed by rejecting bounded producer snapshot pages when any
  current slot in the requested range is newer than the requested
  `max_version`. Without historical values, the producer now returns `None`
  instead of a terminal empty or partial page, and the hardened continuation
  consumer refuses that response rather than completing with missing data.
  Verification:
  `cargo test snapshot_must_not_return_terminal_empty_page_for_newer_updated_keys -- --nocapture`.
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
- ISSUE-046: fixed by applying the same pending-change admission helper to
  `FetchChanged(Ok(_))` response items. Oversized response batches now hit the
  1,024-entry pending cap and fall back to full sync after request-correlation
  and version-range validation. Verification:
  `cargo test working_state_must_cap_pending_fetch_changed_response -- --nocapture`.
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
  `cargo test peer_message_codec_must_reject_oversized_service_payloads -- --nocapture`.
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
- ISSUE-016: fixed by completing awaited `connect()` calls only after the
  outbound connection reports authenticated `PeerConnected`, or after
  `PeerConnectError` for setup/authentication failure. Best-effort
  `try_connect()` still only queues the attempt, and duplicate pending awaited
  connects return an immediate error. Verified with
  `cargo test connect_must_fail_when_remote_peer_id_does_not_match_address -- --nocapture`,
  `cargo test awaited_connect_must_error_while_same_peer_connect_is_pending -- --nocapture`,
  and `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`.
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
- ISSUE-074: fixed by validating `InternalMsg::PublishRpc` against the live
  local publisher handle map before fanout or pending-RPC insertion. Stale
  cloned publisher requesters now receive `NoDestination` and cannot invoke
  subscriber RPC handlers after their owning `Publisher` has been dropped.
  Verified with
  `cargo test dropped_publisher_requester_must_not_continue_publish_rpc -- --nocapture`.
- ISSUE-069: fixed by validating `InternalMsg::Publish` against the live local
  publisher handle map before publish fanout. Stale cloned publisher requesters
  can no longer deliver ordinary publishes after their owning `Publisher` has
  been dropped. Verified with
  `cargo test dropped_publisher_requester_must_not_continue_publishing -- --nocapture`.
- ISSUE-115: fixed by carrying `SubscriberHandleId` on local
  `PublishRpcAnswer` control messages and recording which local subscriber
  handles actually received each publish RPC. Stale or unrelated subscriber
  requesters now leave the pending request open for the legitimate responder or
  timeout. Verified with
  `cargo test dropped_subscriber_requester_must_not_answer_publish_rpc -- --nocapture`.
- ISSUE-075: fixed by validating `InternalMsg::FeedbackRpc` against the live
  local subscriber handle map before fanout or pending-RPC insertion. Stale
  cloned subscriber requesters now receive `NoDestination` and cannot invoke
  publisher RPC handlers after their owning `Subscriber` has been dropped.
  Verified with
  `cargo test dropped_subscriber_requester_must_not_continue_feedback_rpc -- --nocapture`.
- ISSUE-070: fixed by validating `InternalMsg::Feedback` against the live
  local subscriber handle map before feedback fanout. Stale cloned subscriber
  requesters can no longer deliver ordinary feedback after their owning
  `Subscriber` has been dropped. Verified with
  `cargo test dropped_subscriber_requester_must_not_continue_feedback -- --nocapture`.
- ISSUE-116: fixed by carrying `PublisherHandleId` on local
  `FeedbackRpcAnswer` control messages and recording which local publisher
  handles actually received each feedback RPC. Stale or unrelated publisher
  requesters now leave the pending request open for the legitimate responder or
  timeout. Verified with
  `cargo test dropped_publisher_requester_must_not_answer_feedback_rpc -- --nocapture`.
- ISSUE-121: fixed by replacing the fixed one-second pubsub RPC sweep with a
  deadline-driven service timer. `PubsubService::run_loop` computes the nearest
  pending publish/feedback RPC deadline from the request maps and arms a
  one-shot sleep only while RPCs are pending, so short caller-supplied timeouts
  expire near their own deadline and cleanup remains service-owned. Verified
  with
  `cargo test pubsub_publish_rpc_must_respect_short_timeout -- --nocapture`,
  `cargo test pubsub_publish_rpc_local -- --nocapture`,
  `cargo test pubsub_feedback_rpc_local -- --nocapture`,
  `cargo test pending_publish_rpc_requests_must_be_bounded -- --nocapture`,
  and
  `cargo test pubsub_rpc_must_return_no_destination_when_all_remote_sends_fail -- --nocapture`.
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
- ISSUE-058: fixed by keeping local event-sender guards inside returned
  `Publisher` and `Subscriber` handles. Registration failures can still produce
  unregistered handles under the current direct-handle API, but those handles
  no longer expose an immediately closed event receiver. Verified with
  `cargo test pubsub_publisher_after_service_drop_must_not_be_dead_on_arrival -- --nocapture`.
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
- ISSUE-029: fixed by the same `try_send_alias_control` fail-closed requester
  path. Stale alias requesters and guards no longer panic after `AliasService`
  is dropped; `find` returns `None`, and non-`Result` operations log/drop failed
  control admission. Verification:
  `cargo test alias_find_after_service_drop_returns_none_not_panic -- --nocapture`.
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
- ISSUE-078: fixed by treating metrics `Scan` as a broadcast-only collection
  message. `MetricsService` now ignores unicast `Scan` frames, closing the
  direct injected-scan disclosure path while preserving broadcast scan handling
  for collectors. Verified with
  `cargo test metrics_scan_must_not_disclose_metrics_to_non_collector -- --nocapture`
  and
  `cargo test metrics_info_must_not_be_accepted_without_scan_request -- --nocapture`.
- ISSUE-079: fixed by treating visualization `Scan` as a broadcast-only
  collection message. `VisualizationService` now ignores unicast `Scan` frames,
  closing the direct injected-scan topology disclosure path while preserving
  broadcast scan handling for collectors. Verified with
  `cargo test visualization_scan_must_not_disclose_topology_to_non_collector -- --nocapture`
  and `cargo fmt -- --check`. `cargo test discovery_new_node -- --nocapture`
  still fails with the pre-existing `PeerLeaved` vs `PeerUpdated` behavior, so
  that regression remains separate from ISSUE-079.
- ISSUE-080: fixed by adding heartbeat-specific pubsub role reconciliation.
  Heartbeats now repair equal-generation active/inactive changes and clear
  active remote publisher/subscriber roles for channels omitted from the
  sender's heartbeat snapshot, while explicit join/leave messages keep the
  stricter stale-generation guard. Verified with
  `cargo test pubsub_heartbeat_must_remove_stale_remote_publisher -- --nocapture`,
  `cargo test pubsub_empty_heartbeat_must_remove_omitted_stale_remote_publisher -- --nocapture`,
  `cargo test pubsub_heartbeat_must_remove_stale_remote_subscriber -- --nocapture`,
  stale leave/reset/disconnect pubsub tests, and `cargo fmt -- --check`.
- ISSUE-026: fixed by the same heartbeat-specific pubsub role reconciliation.
  Heartbeats now clear stale remote subscriber roles when a peer reports
  `subscribe=false` or omits the channel from its heartbeat snapshot. Verified
  with
  `cargo test pubsub_heartbeat_must_remove_stale_remote_subscriber -- --nocapture`.
- ISSUE-081: fixed by `86160e9` (`fix: validate full sync snapshot
  responses`), which rejects unbounded initial empty snapshot pages with
  nonzero versions before full sync can transition to `WorkingState`. Verified
  with
  `cargo test full_sync_must_reject_initial_empty_snapshot_with_nonzero_version -- --nocapture`.
- ISSUE-082: fixed by `86160e9` (`fix: validate full sync snapshot
  responses`), which rejects snapshot slot keys greater than
  `SnapshotData.biggest_key` before any invalid snapshot data is emitted or
  stored. Verified with
  `cargo test full_sync_must_reject_snapshot_slot_past_biggest_key -- --nocapture`.
- ISSUE-083: fixed by `86160e9` (`fix: validate full sync snapshot
  responses`), which rejects continuation snapshot slot keys lower than the
  pending `FetchSnapshot { from, .. }` bound before emitting or storing data.
  Verified with
  `cargo test full_sync_must_reject_continuation_slot_before_requested_key -- --nocapture`.
- ISSUE-084: fixed by `86160e9` (`fix: validate full sync snapshot
  responses`), which requires snapshot slot keys to be strictly increasing
  before any slot is emitted, stored, or used to complete full sync. Verified
  with
  `cargo test full_sync_must_reject_unsorted_snapshot_slots -- --nocapture`.
- ISSUE-085: fixed by `86160e9` (`fix: validate full sync snapshot
  responses`), whose strict snapshot key ordering rejects duplicate keys before
  duplicate `KvEvent::Set` events or last-write-wins slot overwrites can occur.
  Verified with
  `cargo test full_sync_must_reject_duplicate_snapshot_keys -- --nocapture`.
- ISSUE-088: fixed by `55b79e5` (`fix: continue partial kv repair
  responses`), which validates the whole `FetchChanged(Ok(_))` batch against
  the active pending request and rejects duplicate versions before mutating
  slots, advancing versions, or emitting local events. Verified with
  `cargo test working_state_must_reject_duplicate_fetch_changed_versions -- --nocapture`.
- ISSUE-089: fixed by `55b79e5` (`fix: continue partial kv repair
  responses`), which validates `FetchChanged(Ok(_))` versions against the
  active pending request's inclusive `[from, from + count - 1]` range before
  applying any changes. Verified with
  `cargo test working_state_must_reject_fetch_changed_versions_beyond_requested_count -- --nocapture`.
- ISSUE-090: fixed by correlating alias `Found` replies with the active pending
  lookup before cache mutation or waiter completion. Cached-hint lookups now
  accept `Found` only from peers that received `Check`, while scan lookups still
  accept scan responses. Verified with
  `cargo test cached_hint_find_must_ignore_found_from_unchecked_peer -- --nocapture`.
- ISSUE-092: fixed by discovery timestamp validation in `apply_sync`, which
  ignores stale advertisements when an existing remote record has a newer or
  equal `last_updated` timestamp. Verified with
  `cargo test apply_sync_must_not_overwrite_newer_discovery_with_stale_advertisement -- --nocapture`.
- ISSUE-093: fixed by discovery tombstone freshness validation in `apply_sync`,
  which ignores only advertisements at or before a live stop tombstone and
  allows fresher restart advertisements to clear the tombstone. Verified with
  `cargo test graceful_stop_tombstone_must_allow_fresh_restart_advertise -- --nocapture`.
- ISSUE-094: fixed by returning user serialization errors from public pubsub
  object helpers instead of panicking on caller-provided `Serialize` values.
  Guest, publisher, and subscriber object helpers now use `?` around
  `bincode::serialize(...)`; internal wire-message serialization is unchanged.
  Verified with
  `cargo test pubsub_guest_object_publish_must_return_error_on_serialize_failure -- --nocapture`.
- ISSUE-095: fixed by rejecting duplicate pending replicated-KV
  `Changed.version` broadcasts before insertion into `WorkingState::pendings`.
  Future broadcasts for the same version no longer overwrite the first pending
  value that will be applied after the missing gap is filled. Verified with
  `cargo test working_state_must_reject_duplicate_pending_changed_broadcast_versions -- --nocapture`.
- ISSUE-096: fixed by replacing replicated-KV outbound wire-event
  serialization panics in `ReplicatedKvService::recv` with explicit log-and-skip
  handling. Serialization failures for caller-provided key/value payloads no
  longer unwind the `recv()` event loop or require a public API change.
  Verified with
  `cargo test replicated_kv_recv_must_not_panic_on_value_serialize_failure -- --nocapture`.
- ISSUE-099: fixed by rejecting zero effective replicated-KV `FetchChanged`
  windows in `LocalStore::changeds_from_to` before building the response range.
  Remote `count = 0` and effective zero windows now return
  `FetchChangedError::MissingData` instead of successful empty repair batches.
  Verified with
  `cargo test fetch_changed_with_zero_count_must_not_return_empty_success -- --nocapture`.
- ISSUE-100: fixed by enforcing `MAX_REMOTE_MEMBERS_PER_CHANNEL = 1024` on
  each pubsub channel's remote publisher and subscriber role maps. Existing
  peers can still update generation/active state at capacity, while inactive
  entries are pruned before admitting a new unknown remote member. Verified
  with
  `cargo test remote_publisher_memberships_must_be_bounded -- --nocapture`.
- ISSUE-101: fixed by capping each alias cache hint set at
  `MAX_ALIAS_HINT_PEERS = 1024` in the shared `insert_cache_hint` path used by
  `NotifySet` and accepted `Found` responses. Existing peers can refresh at
  capacity, while new unknown hints are not retained once the per-alias set is
  full. Verified with
  `cargo test cached_alias_peer_hints_must_be_bounded -- --nocapture`.
- ISSUE-102: fixed by capping retained visualization remote peers at
  `MAX_VISUALIZATION_REMOTE_PEERS = 1024` through a shared broadcast/unicast
  `Info` admission helper. Existing peers still update at capacity, while new
  unknown senders are ignored without emitting inconsistent visualization
  events. Verified with
  `cargo test visualization_remote_peers_must_be_bounded -- --nocapture`.
- ISSUE-104: fixed by enforcing `MAX_METRICS_PER_INFO = 1024` after metrics
  `Info` pending-responder correlation and before emitting
  `OnPeerConnectionMetric`. Oversized correlated responses are dropped instead
  of forwarded as partial or oversized batches. Verified with
  `cargo test metrics_info_batches_must_be_bounded -- --nocapture`.
- ISSUE-105: fixed by enforcing `MAX_TOPOLOGY_ROWS_PER_INFO = 1024` in
  `VisualizationService::on_info` before mutating retained peer state or
  emitting visualization topology events. Oversized `Info` batches are dropped
  instead of forwarded as partial or oversized `PeerJoined` / `PeerUpdated`
  payloads. Verified with
  `cargo test visualization_info_batches_must_be_bounded -- --nocapture` and
  `cargo test visualization_info_batch_at_cap_must_be_accepted -- --nocapture`.
- ISSUE-106: fixed by enforcing `MAX_HEARTBEAT_CHANNELS_PER_BATCH = 1024` at
  the start of inbound pubsub `Heartbeat` handling, before `seen_channels`,
  remote role mutation, or local event fanout. Oversized heartbeat batches are
  dropped instead of truncated so omitted-channel cleanup cannot convert
  truncated rows into false leaves. Verified with
  `cargo test pubsub_heartbeat_channel_batches_must_be_bounded -- --nocapture`
  and
  `cargo test pubsub_heartbeat_channel_batch_at_cap_must_be_accepted -- --nocapture`.
- ISSUE-107: fixed by enforcing `MAX_RPC_METHOD_LEN = 1024` for inbound
  pubsub `GuestPublishRpc`, `PublishRpc`, `GuestFeedbackRpc`, and `FeedbackRpc`
  before channel lookup, membership authorization, or local event fanout.
  Oversized RPC method names are dropped, not truncated, so invalid method
  names cannot be delivered to application handlers or rerouted as shorter
  names. Verified with `cargo test pubsub_rpc_methods_must_be_bounded --
  --nocapture`, `cargo test pubsub_rpc_method_at_cap_must_be_accepted --
  --nocapture`, and
  `cargo test pubsub_other_inbound_rpc_methods_must_be_bounded -- --nocapture`.
- ISSUE-108: fixed by pruning `PubsubService::channels` after a valid final
  local publisher/subscriber handle removal only when local publishers, local
  subscribers, remote publishers, and remote subscribers are all empty. Local
  publisher/subscriber generation allocation moved to service-level monotonic
  counters, so recreating a pruned channel emits a join generation newer than
  the prior leave generation without retaining an unbounded per-channel
  generation map. Verified with
  `cargo test empty_pubsub_channels_must_be_removed_after_last_local_handle_drops -- --nocapture`,
  `cargo test empty_pubsub_publisher_channels_must_be_removed_after_last_local_handle_drops -- --nocapture`,
  `cargo test pubsub_prune_must_preserve_channels_with_remote_state -- --nocapture`,
  `cargo test pubsub_recreate_after_prune_must_use_newer_publisher_generation -- --nocapture`,
  `cargo test pubsub_recreate_after_prune_must_use_newer_subscriber_generation -- --nocapture`,
  and
  `cargo test stale_pubsub_destroy_must_not_create_phantom_channel -- --nocapture`.
- ISSUE-109: fixed by correlating inbound alias `Found` responses with an
  active find request before inserting cache hints or completing waiters.
  Unsolicited `Found` messages with no pending lookup are ignored, unchecked
  `Found` responses during cached-hint validation are ignored, and active scan
  responses still accept the responding peer by design. Verified with
  `cargo test unsolicited_found_must_not_create_alias_cache_hint -- --nocapture`,
  `cargo test cached_hint_find_must_ignore_found_from_unchecked_peer -- --nocapture`,
  `cargo test test_find_cached_alias_found -- --nocapture`, and
  `cargo test found_response_must_not_exceed_alias_hint_cap -- --nocapture`.
- ISSUE-112: fixed by rejecting `addr.peer_id() == self.local_id` at the start
  of `P2pNetwork::process_connect`, before duplicate suppression or QUIC
  dialing. Awaited self-connects return `Err(_)` through the oneshot, while
  best-effort self-connects no-op without inserting neighbours or starting a
  dial. Verified with
  `cargo test connect_to_own_peer_address_must_fail -- --nocapture`,
  `cargo test best_effort_connect_to_own_peer_address_must_not_create_neighbour -- --nocapture`,
  `cargo test awaited_connect_to_own_peer_address_must_error_without_neighbour -- --nocapture`,
  `cargo test awaited_connect_must_error_while_same_peer_connect_is_pending -- --nocapture`,
  and
  `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`.
  `cargo test connect_must_fail_when_remote_peer_id_does_not_match_address -- --nocapture`
  still fails in current source and remains separate from this self-connect
  guard.
- ISSUE-113: fixed by the same pending-attempt coalescing path documented for
  ISSUE-153. `P2pNetwork::process_connect` now checks
  `NetworkNeighbours::has_peer_connection_attempt` before `endpoint.connect`;
  best-effort duplicate connects to a peer with a live connected or pending
  outbound attempt no-op, awaited duplicates return `Err(_)`, and stale failed
  attempts remain retryable after `PeerConnectError` cleanup. Verified with
  `cargo test concurrent_connects_to_same_peer_must_be_coalesced -- --nocapture`,
  `cargo test awaited_connect_must_error_while_same_peer_connect_is_pending -- --nocapture`,
  and
  `cargo test stale_pending_outgoing_peer_does_not_suppress_reconnect -- --nocapture`.
- ISSUE-114: fixed by validating `MainEvent::PeerConnected` before installing
  direct route state. The main loop ignores unknown connection ids and peer ids
  that do not match the registered alias, rejects already-connected duplicate
  peers, sends a nonblocking close control to rejected duplicates, deletes any
  direct route for the rejected connection, removes the neighbour entry, and
  unregisters the alias. The evidence test now uses independent nodes with the
  same `PeerId` to bypass outbound coalescing and verifies both alias and route
  counts stay bounded. Verified with
  `cargo test inbound_duplicate_connections_from_same_peer_must_be_coalesced -- --nocapture`,
  `cargo test stale_peer_connected_event_must_not_install_unusable_route -- --nocapture`,
  and
  `cargo test peer_connected_must_not_rebind_existing_connection_to_different_peer -- --nocapture`.
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
  unacked response retries, so duplicate metrics scans from one requester
  coalesce while a response is still backpressured. Verified with
  `cargo test metrics_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-202 remains separate for immediate dropped
  responses, ISSUE-203 remains separate for visualization response
  accumulation, ISSUE-200/201 remain separate for periodic scan-broadcast
  coalescing, and ISSUE-078/related issues remain separate for unauthorized
  metrics disclosure.
- ISSUE-203: fixed by `VisualizationService::pending_scan_responses` plus
  bounded unacked response retries, so duplicate visualization scans from one
  peer coalesce while a response is still backpressured. Verified with
  `cargo test visualization_scan_responses_must_not_accumulate_behind_full_peer_control_queue -- --nocapture`
  and `cargo fmt -- --check`. ISSUE-201 remains separate for periodic
  visualization scan-broadcast coalescing, ISSUE-204 remains separate for
  metrics response accumulation, and ISSUE-079/related issues remain separate
  for unauthorized topology disclosure.
- ISSUE-202: fixed by retrying metrics scan replies through a bounded unacked
  response path until `SCAN_RESPONSE_SEND_TIMEOUT`, while tracking
  `pending_scan_responses`, so a metrics scan response waits through transient
  peer-control backpressure instead of being dropped immediately. Verified with
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
- ISSUE-049: fixed by wrapping each awaited peer-alias broadcast admission in
  `BROADCAST_ADMISSION_TIMEOUT`, allowing fanout to continue past congested
  peer control queues. Verified with
  `cargo test send_broadcast_must_not_block_on_full_peer_control_queue -- --nocapture`.
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
  and `cargo fmt -- --check`. Historical note: ISSUE-194 was still open at
  this point, but it is now fixed by `InboundPeerBindings`.
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
- ISSUE-244, score 72: fixed handshake replay after exact-cache eviction.
  Root cause: the bounded replay cache evicted the oldest live token marker
  under pressure, so the same signed token could be accepted again before its
  timestamp window expired. Smallest fix: keep the existing exact cache
  bounded for availability, but also insert every accepted token hash into a
  fixed-size rotating replay window and reject tokens seen in that live
  window even after exact-cache eviction.
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
  Root cause: recovery full-sync reused the first-sync init path, which drained
  visible slots before a replacement snapshot completed. Fix: `WorkingState`
  fallback full-syncs stage snapshot pages, keep old slots visible until the
  terminal page, then commit deletes and value-changing sets. Verification:
  `cargo test solicited_full_resync -- --nocapture` and
  `cargo test service::replicate_kv_service::remote_storage::tests -- --nocapture`.
- ISSUE-172, score 68: outbound peer setup hangs while writing `ConnectReq` to
  a stalled peer. Reviewer: James the 3rd.
- ISSUE-173, score 68: inbound peer setup hangs while writing `ConnectRes` to
  a stalled peer. Reviewer: Peirce the 3rd.
- ISSUE-174, score 46: QUIC object writer can bypass `MAX_SIZE` with
  non-deterministic serialization. Reviewer: Hypatia the 3rd.
- ISSUE-175, score 42: replicated KV emits delete changes for keys that were
  never present. Reviewer: Volta the 3rd.
  Root cause: local and remote delete paths emitted visible delete events before
  checking whether the key existed in current state. Fix: make local absent
  deletes no-op before version/changelog mutation, and make ordered remote
  deletes advance protocol version while emitting `KvEvent::Del` only after an
  actual slot removal. Verification:
  `cargo test deleting_absent_key_must_not_emit_delete_event -- --nocapture`
  and
  `cargo test remote_delete_for_absent_key_must_not_emit_delete_event -- --nocapture`.
- ISSUE-176, score 66: shared-key handshake response tokens are replayable.
  Reviewer: Harvey the 3rd.
- ISSUE-177, score 38: `connect()` reports success for a different address
  when the peer id is already connected. Reviewer: Helmholtz the 3rd.
- ISSUE-178, score 57: pubsub RPC treats closed local event channels as live
  destinations. Reviewer: Russell the 3rd.
  Root cause: RPC destination accounting used channel membership as a proxy for
  deliverability, so closed local event channels or failed remote sends could
  leave callers waiting on pending RPC state. Fix: count only successful local
  event delivery and remote fanout, return `NoDestination` when delivered count
  is zero, and skip pending request insertion. Verification:
  `cargo test pubsub_rpc_must_return_no_destination_when_all_local_sends_fail -- --nocapture`,
  `cargo test pubsub_rpc_must_return_no_destination_when_all_remote_sends_fail -- --nocapture`,
  `cargo test pubsub_rpc_must_return_no_destination_when_all_local_subscriber_queues_are_full -- --nocapture`,
  `cargo test feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full -- --nocapture`,
  and
  `cargo test guest_feedback_rpc_must_return_no_destination_when_all_local_publisher_queues_are_full -- --nocapture`.
- ISSUE-179, score 49: local alias shutdown leaves pending find waiters alive.
  Reviewer: Socrates the 3rd.
  Root cause: local alias shutdown only broadcast `AliasMessage::Shutdown` and
  left in-flight local `find_reqs` waiters parked until scan timeout. Fix: drain
  pending finds on shutdown, complete waiters with `None`, and decrement the live
  find gauge once per drained request. Verification:
  `cargo test local_shutdown_must_fail_pending_alias_finds -- --nocapture`.
- ISSUE-180, score 64: relay stream setup can forward back to the ingress peer.
  Reviewer: Carver the 3rd.
  Root cause: accepted stream relay tasks lacked the ingress `ConnectionId`, so
  they could not reject `RouteAction::Next(next)` when `next` was the connection
  that delivered the stream request. Fix: pass the ingress id into `accept_bi`
  and return a prompt `route loop` stream-connect error for ingress-loop relay
  decisions. Verification:
  `cargo test relay_stream_must_not_forward_back_to_ingress_peer -- --nocapture`.
- ISSUE-181, score 45: local advertise config can gossip unroutable wildcard
  addresses. Reviewer: Nash the 3rd.
- ISSUE-182, score 52: QUIC admits unused unidirectional streams. Reviewer:
  Pascal the 3rd.
  Root cause: production QUIC server/client config allowed 10,000 concurrent
  unidirectional streams, but the P2P protocol never calls `accept_uni`.
  Fix: set production server and client `max_concurrent_uni_streams` to zero,
  leaving test-only raw endpoint configs unchanged. Verification:
  `cargo test unused_unidirectional_streams_must_not_be_admitted -- --nocapture`.
- ISSUE-183, score 53: local alias shutdown keeps serving local aliases.
  Reviewer: Newton the 3rd.
  Root cause: local shutdown broadcast stopped authority to peers but kept local
  alias ownership live and accepted later registrations. Fix: latch shutdown,
  clear local aliases, reject later `Register`, answer later `Find` with `None`,
  and suppress duplicate shutdown broadcasts. Verification:
  `cargo test local_shutdown_must_stop_serving_local_aliases -- --nocapture`.
- ISSUE-184, score 57: replicated KV duplicates in-flight FetchChanged repairs
  for the same gap. Reviewer: Poincare the 3rd.
  Root cause: `WorkingState::apply_pendings` rescheduled the same missing range
  for each later future `Changed` while the first repair was still in flight.
  Fix: `WorkingState` now suppresses already-covered in-flight `FetchChanged`
  ranges, permits wider replacement requests, and clears repairs satisfied by
  broadcasts. Verification:
  `cargo test working_state_must_not_duplicate_inflight_fetch_changed_for_same_gap -- --nocapture`,
  `cargo test working_state_must_cancel_fetch_changed_when_broadcast_fills_gap -- --nocapture`,
  `cargo test test_working_state_resend_timeout_fetch_changed -- --nocapture`,
  `cargo test working_state_must_continue_repair_after_partial_fetch_changed_success -- --nocapture`,
  `cargo test working_state_must_not_let_stale_fetch_changed_response_cancel_newer_repair -- --nocapture`,
  and `cargo fmt -- --check`.
- ISSUE-185, score 56: pubsub keeps remote subscriber membership after graceful
  peer stop. Reviewer: Popper the 3rd.
  Root cause: graceful peer-stop cleanup reached discovery/router state but did
  not fan peer lifecycle into service-owned pubsub membership. Fix: accepted
  `PeerStopped` and direct disconnect paths notify services with
  `PeerDisconnected`, and pubsub removes active remote publisher/subscriber
  roles while emitting `PeerLeaved`. Verification:
  `cargo test pubsub_must_remove_remote_subscriber_on_graceful_peer_stop -- --nocapture`.
- ISSUE-186, score 54: ignored replicated-KV broadcasts refresh stale remote
  activity. Reviewer: Nietzsche the 3rd.
  Root cause: `RemoteStore::on_broadcast` refreshed liveness before knowing
  whether the broadcast was accepted. Fix: the accepted-event liveness refresh
  from ISSUE-140 covers the broadcast path, so stale/equal version broadcasts no
  longer keep inactive remotes alive. Verification:
  `cargo test ignored_broadcast_must_not_refresh_remote_activity -- --nocapture`.
- ISSUE-187, score 49: graceful PeerStopped is hidden from public network
  events. Reviewer: Mendel the 3rd.
  Root cause: accepted graceful stops cleaned internal route state but returned
  `Continue` to public consumers. Fix: validated `PeerStopped` handling now
  performs cleanup, notifies services, and returns
  `P2pNetworkEvent::PeerDisconnected(conn, peer)`. Verification:
  `cargo test peer_stopped_must_emit_public_disconnect_event -- --nocapture`.
- ISSUE-188, score 51: pubsub drops early remote publisher joins before local
  channel creation. Reviewer: Noether the 3rd.
  Root cause: `PublisherJoined`/`SubscriberJoined` only updated existing channel
  state, so joins that arrived before local handle creation were discarded.
  Fix: active remote joins now create bounded channel state and reuse the local
  handle replay path; leave/heartbeat messages still do not create missing
  channels, empty remote-only channels are pruned after remote cleanup, and
  reclaimed inactive roles keep generation tombstones to reject stale joins. If
  tombstone capacity would be exceeded, inactive channel state is retained
  instead.
  Verification: `cargo test early_remote -- --nocapture`,
  `cargo test remote_created_channel_cap_must_recover -- --nocapture`,
  `cargo test reclaimed_remote -- --nocapture`,
  `cargo test tombstone_must_survive_newer_join_dropped_by_channel_cap -- --nocapture`,
  `cargo test inactive_channel_must_not_be_reclaimed_when_tombstone_cap_would_drop_generations -- --nocapture`,
  `cargo test remote_publisher_memberships_must_be_bounded -- --nocapture`,
  `cargo test remote_subscriber_memberships_must_be_bounded -- --nocapture`,
  and `cargo fmt -- --check`.
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
- ISSUE-208, score 61: saturated alias guard refcounts can unregister an alias
  while live guards still remain after more than 255 registrations. Reviewer:
  Sartre the 12th. Fixed by widening local alias guard refcounts to `usize`
  and using checked increments, so aliases are removed only when the exact
  tracked guard count reaches zero.

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
  `accept_uni` path. Fixed by setting production server and client
  `max_concurrent_uni_streams` to zero; the focused test now passes.
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
  loop handling is fixed at this point; ISSUE-180 is now fixed by passing the
  ingress `ConnectionId` into `accept_bi` and rejecting same-connection relay.
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
  closed-receiver success reporting was still open at this point. It is now
  covered by the fixed direct and relayed closed-receiver unicast tests.
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

- Cycle after ISSUE-208 no-new cycle 5 ran the harder stop-condition fuzz pass:
  12-node steady valid actions with
  `P2P_FUZZ_SEED=208401 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=3200` and 12-node
  sanitized churn with
  `P2P_FUZZ_SEED=208402 P2P_FUZZ_NODES=12 P2P_FUZZ_STEPS=2600`. Both passed.
  Reviewer `Popper the 12th` confirmed no-new and approved stopping the
  no-new audit loop for now. Route-loop/path noise maps to ISSUE-003,
  ISSUE-180, ISSUE-197, and RC-7; duplicate/connect churn maps to ISSUE-113,
  ISSUE-114, ISSUE-153, ISSUE-177, and RC-7; graceful-stop cleanup churn maps
  to ISSUE-139, ISSUE-144, ISSUE-170, ISSUE-193, and RC-6; backpressure and
  control-close noise maps to ISSUE-118, ISSUE-123 through ISSUE-127,
  ISSUE-153, ISSUE-198 through ISSUE-204, and RC-3. No new root cause or fix
  proposal was recorded.
- Cycle after ISSUE-208 no-new cycle 4 reviewed metrics, visualization, alias
  lifecycle/control, and 9-node steady fuzz. Metrics/visualization info guards,
  alias stale requester handling, local shutdown, and peer-scoped shutdown tests
  passed. `saturated_alias_refcount_must_not_unregister_while_guards_remain`
  still failed at `src/service/alias_service.rs:799:9`, but reviewer
  `Hume the 12th` confirmed it is duplicate ISSUE-208 evidence, not a new
  issue. Steady fuzz with
  `P2P_FUZZ_SEED=208301 P2P_FUZZ_NODES=9 P2P_FUZZ_STEPS=2200` passed.
  Metrics/visualization candidates map to ISSUE-061, ISSUE-062, ISSUE-078,
  ISSUE-079, ISSUE-104, ISSUE-105, ISSUE-120, ISSUE-162, ISSUE-165,
  ISSUE-200 through ISSUE-204. Alias candidates map to ISSUE-029, ISSUE-035,
  ISSUE-036, ISSUE-041, ISSUE-090, ISSUE-101, ISSUE-109, ISSUE-127,
  ISSUE-130, ISSUE-132, ISSUE-137, ISSUE-158, ISSUE-179, ISSUE-183,
  ISSUE-185, ISSUE-206, and ISSUE-208. No new root cause or fix proposal was
  recorded.
- Cycle after ISSUE-208 no-new cycle 3 reviewed route/discovery graceful-stop
  and duplicate-connect boundaries plus pubsub graceful-stop, heartbeat, and
  stale-event boundaries. Focused discovery/pubsub tests passed, and sanitized
  churn fuzz with
  `P2P_FUZZ_SEED=208201 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2000` passed with only
  expected refused-connect, shutdown, deadline, duplicate-connection, and
  connection-lost churn logs. Reviewer `Banach the 12th` confirmed no-new.
  Route/discovery candidates map to ISSUE-003, ISSUE-004, ISSUE-055,
  ISSUE-092, ISSUE-103, ISSUE-112, ISSUE-113, ISSUE-114, ISSUE-133,
  ISSUE-139, ISSUE-153, ISSUE-164, ISSUE-167, ISSUE-170, ISSUE-177,
  ISSUE-181, ISSUE-190, ISSUE-192, and ISSUE-197. Pubsub candidates map to
  ISSUE-020, ISSUE-039, ISSUE-078 through ISSUE-080, ISSUE-094, ISSUE-115,
  ISSUE-121, ISSUE-126, ISSUE-128 through ISSUE-132, ISSUE-142, ISSUE-150,
  ISSUE-155, ISSUE-157, ISSUE-158, ISSUE-188, ISSUE-193, and ISSUE-198.
  No new root cause or fix proposal was recorded.
- Cycle after ISSUE-208 no-new cycle 2 reviewed replicated-KV state boundaries
  and stream setup/lifecycle boundaries, then ran focused stream probes,
  `cargo test replicate -- --nocapture`, and steady valid-node fuzz with
  `P2P_FUZZ_SEED=208101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800`. All passed.
  Reviewer `Poincare the 12th` confirmed no-new. Replicated-KV candidates map
  to ISSUE-034, ISSUE-047, ISSUE-081 through ISSUE-089, ISSUE-110, ISSUE-138,
  ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-171, ISSUE-175, ISSUE-184, and
  ISSUE-186; stream candidates map to ISSUE-011, ISSUE-012, ISSUE-013,
  ISSUE-018, ISSUE-053/091, ISSUE-117, ISSUE-149, ISSUE-156, ISSUE-169,
  ISSUE-180, and ISSUE-182. No new root cause or fix proposal was recorded.
- Cycle after ISSUE-208 no-new cycle 1 reviewed `src/lib.rs`,
  `src/requester.rs`, and `src/tests/security.rs`, then ran the focused
  `PeerStats` tests and sanitized churn fuzz. The two `PeerStats` tests failed
  exactly as existing ISSUE-064 and ISSUE-068 evidence:
  `stale_peer_stats_event_must_not_publish_metrics_for_unknown_connection`
  failed at `src/tests/security.rs:747:5`, and
  `peer_stats_must_validate_peer_matches_connection` failed at
  `src/tests/security.rs:770:5`. The fuzz pass
  `P2P_FUZZ_SEED=208001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800` passed with
  expected shutdown/refused/duplicate/connection-lost churn noise only.
  Reviewer `Franklin the 12th` confirmed no-new; requester/control candidates
  map to ISSUE-016, ISSUE-028, ISSUE-112, ISSUE-113, ISSUE-114, ISSUE-125,
  ISSUE-153, and ISSUE-177.
- Cycle after ISSUE-204 no-new cycle 342 reviewed route/service delivery,
  pubsub lifecycle, and replicated-KV state machines, then ran two fresh fuzz
  passes: `P2P_FUZZ_SEED=342001 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=2200` steady
  valid actions and `P2P_FUZZ_SEED=342101 P2P_FUZZ_NODES=8 P2P_FUZZ_STEPS=1800`
  sanitized churn actions. Both passed with no panic, failed assertion, or new
  invariant. Candidates mapped to ISSUE-058, ISSUE-069, ISSUE-070, ISSUE-074,
  ISSUE-075, ISSUE-115, ISSUE-116, ISSUE-142, ISSUE-117, ISSUE-149,
  ISSUE-156, ISSUE-180, and replicated-KV ISSUE-034, ISSUE-047, ISSUE-081
  through ISSUE-089, ISSUE-110, ISSUE-138, ISSUE-141, ISSUE-143, ISSUE-154,
  ISSUE-171, ISSUE-175, ISSUE-184, and ISSUE-186. No new root cause was
  recorded.
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
- Cycle after ISSUE-209 no-new cycle 1 ran real 12-node steady and sanitized
  churn fuzz passes after removing the hidden eight-node cap. Both runs passed.
  Observed duplicate connection churn, high-load control-queue pressure, and
  stopped-peer reconnect attempts map to existing RC-3, RC-6, and RC-7
  families unless a later run produces a focused failing invariant.
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
