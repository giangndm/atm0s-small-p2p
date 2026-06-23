# Found Issues

Compact RED-team ledger for `atm0s-small-p2p`.

Acceptance rule: an issue is accepted only when it has failing-test evidence
and reviewer confirmation. Issue score: `100` is critical and must resolve;
`0` is low priority.

The old verbose issue-by-issue narrative was intentionally compacted because it
was hard to read. Use git history before the cleanup commit when the full
historic prose for an older issue is needed.

## Current Status

| Item | Value |
| --- | --- |
| Accepted issues | 248 |
| Missing issue scores | 0 |
| Current consecutive no-new cycles | 2 |
| Final condition | Stress/fuzz tests succeeded without errors |
| Completion state | RED-team goal complete |

## Final Verification

| Area | Command / Evidence | Result |
| --- | --- | --- |
| Service/requester boundary | `CARGO_BUILD_JOBS=8 RUST_LOG=error cargo test --lib service -- --nocapture` | 199 passed |
| Network requester lifecycle | `CARGO_BUILD_JOBS=8 RUST_LOG=error cargo test --lib requester -- --nocapture` | 13 passed |
| Cross-node delivery | `CARGO_BUILD_JOBS=8 RUST_LOG=error cargo test --lib cross_nodes -- --nocapture` | 9 passed |
| Security regression suite | `CARGO_BUILD_JOBS=8 RUST_LOG=error cargo test --lib security -- --nocapture` | 55 passed |
| Local stress fuzz | `CARGO_BUILD_JOBS=8 P2P_FUZZ_NODES=44 P2P_FUZZ_STEPS=3600 P2P_FUZZ_SEED=249050 RUST_LOG=error cargo test --lib fuzz_random_adversarial_node_actions_must_not_panic_connection_tasks -- --nocapture` | 1 passed in 27.60s |
| Reviewer stress fuzz | `CARGO_BUILD_JOBS=8 P2P_FUZZ_NODES=48 P2P_FUZZ_STEPS=2400 P2P_FUZZ_SEED=248250 RUST_LOG=error cargo test --lib fuzz_random_adversarial_node_actions_must_not_panic_connection_tasks -- --nocapture` | 1 passed |

## Recent Accepted Issues

| Issue | Score | Category | Status | Evidence / Fix |
| --- | ---: | --- | --- | --- |
| ISSUE-248 | 82 | Stability evidence, benchmark/report tooling | Fixed | Failing report tests showed long-run benchmark rows could stay `pass` while RSS grew sharply. Report now labels repeated short-cluster iterations and emits `resource-warning`. Reviewer: `Copernicus`. |
| ISSUE-247 | 88 | Stability, resource exhaustion, replicated-KV full sync | Fixed | Failing test showed cumulative staged snapshot slots were unbounded across valid continuation pages. Added aggregate staged-slot cap. Reviewer: `Beauvoir the 2nd`. |
| ISSUE-246 | 54 | API stability, pubsub registration backpressure | Fixed | Failing test showed pubsub registration overflow could return a silent unregistered handle. Rejected handles now reject later requester calls. |
| ISSUE-245 | 58 | Replicated-KV full-sync correctness | Fixed | Initial full sync could expose partial snapshot pages before completion. Snapshot pages now stage until terminal commit. |
| ISSUE-244 | 72 | Authentication replay protection | Fixed | Replay cache pressure could make handshake tokens reusable. Added compact rotating replay window. |

## Final No-New Cycles

| Cycle | Scope | Reviewer | Verification | Result |
| --- | --- | --- | --- | --- |
| 2 after ISSUE-248 | Service/requester/control queues, local delivery, dropped handles, duplicate service registration, churn | `Ptolemy` | service, requester, cross_nodes, security, 44-node fuzz; reviewer also ran requester, service, full, closed, duplicate_service, out_of_range, pending, peer-stopped-full, 48-node fuzz | No distinct score-80+ issue |
| 1 after ISSUE-248 | Discovery, neighbours, connection lifecycle, stopped peers, duplicate connects, churn | `James` | discovery, security, cross_nodes, stopped, 48-node fuzz; reviewer also ran discovery, stopped, duplicate, 36-node churn fuzz | No distinct score-80+ issue |

## Issue Family Index

| Family | Issue IDs | Summary |
| --- | --- | --- |
| Authentication and identity binding | 001, 002, 014-018, 146, 176, 189, 194, 207, 244 | Forged peer identities, source spoofing, handshake freshness, replay pressure |
| Routing and discovery | 003-010, 033, 044, 092, 093, 103, 160, 167, 181, 190, 192, 210-214 | Route flapping, stale routes, sync caps, local/self routes, malformed or non-dialable discovery |
| Streams and delivery | 011-013, 049, 050, 056, 091, 117-120, 149, 156, 169, 180, 197-199, 217, 220, 229, 238 | False success, local queue backpressure, relay loops, stalled handshakes, stream admission |
| Service/requester lifecycle | 028-030, 052-054, 060, 072, 073, 076, 108, 125, 139, 234, 246 | Dropped handles, stale requesters, duplicate registration, service-id bounds, control queue caps |
| Pubsub | 020, 026, 039, 043, 048, 058, 069, 070, 074, 075, 080, 094, 100, 106, 107, 115, 116, 121, 123, 124, 126, 142, 150, 155, 163, 168, 178, 185, 188, 205, 228, 231, 236, 240-243 | RPC binding, member authorization, heartbeat chunks, remote membership caps, local handle lifecycle |
| Replicated KV | 023, 025, 027, 031, 032, 034, 037, 038, 045-047, 059, 071, 077, 081-089, 095, 096, 099, 110, 111, 131, 138, 140, 141, 143, 154, 171, 175, 184, 186, 196, 233, 237, 245, 247 | Snapshot validation, repair state, staging, changed-response validation, resource caps |
| Alias service | 019, 022, 029, 035, 036, 041, 090, 101, 109, 127, 130, 132, 137, 148, 152, 158, 179, 183, 206, 208, 235, 239 | Alias cache poisoning, lifecycle generation, pending waiter caps, shutdown cleanup |
| Metrics and visualization | 040, 061, 062, 064, 068, 078, 079, 102, 104, 105, 128, 129, 165, 193, 195, 200-204, 226, 232 | Collector authorization, stale scan/Info handling, topology/metrics disclosure, row caps |
| Churn, stop, and cleanup | 004, 051, 063, 065-067, 118, 122, 133-136, 144, 151, 157, 161, 162, 164, 170, 187, 215-225 | Graceful stop propagation, stale events, main-queue pressure, alias cleanup, post-stop traffic |
| Transport and framing | 024, 097, 098, 172-174, 182 | Frame/object limits, serialization failures, QUIC stream limits, stalled setup writes |
| Tooling and evidence | 191, 209, 248 | README/example compile checks, fuzz profile validity, benchmark report correctness |

## Latest Issue Detail

### ISSUE-248: Stream-limit benchmark can report repeated short clusters as a long-run pass while RSS grows

| Field | Detail |
| --- | --- |
| Score | 82 |
| Status | Fixed |
| Reviewer | `Copernicus` |
| Root cause | `--min-run-seconds` repeated fresh short `run_profile` executions, and `render_report` marked clean stream counters as `pass` without considering repeated-run semantics or RSS growth. |
| Impact | A 30-minute stability report could look successful while actually representing many fresh short clusters and large process RSS growth. |
| Fix | The report now states when rows are repeated short-cluster iterations, detects sharp RSS growth, and marks affected rows `resource-warning`. |
| Evidence | `cargo test --example stream_limit_benchmark -- --nocapture` originally failed the new report-contract tests, then passed after the fix. |

### ISSUE-247: Replicated-KV full sync can stage unbounded snapshot slots across pages

| Field | Detail |
| --- | --- |
| Score | 88 |
| Status | Fixed |
| Reviewer | `Beauvoir the 2nd` |
| Root cause | Full sync capped each page but not the aggregate staged snapshot state held until terminal completion. |
| Impact | A malicious authenticated peer could grow receiver memory by sending many valid continuation pages. |
| Fix | Added `MAX_STAGED_SNAPSHOT_SLOTS` and rejected pages before mutation when accepting them would exceed the total cap. |
| Evidence | `full_sync_staged_snapshot_slots_must_be_bounded_across_pages` failed before the fix and passed after it. |

## Reading Notes

- This file is intentionally concise. It preserves current status, final
  verification, recent high-signal issues, and the issue-family index.
- The old detailed ledger can be recovered from git history if exact prose for
  an older issue is needed.
- `docs/stream_limit_benchmark_report.md` remains separate because it contains
  the benchmark report and SVG resource charts.
