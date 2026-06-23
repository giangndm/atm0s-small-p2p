# RED-Team Audit Summary

Compact review copy for the `atm0s-small-p2p` audit. The detailed current
ledger is in `docs/found_issues.md`; the old verbose history remains available
in git history.

## Status

| Item | Value |
| --- | --- |
| Accepted issues | 248 |
| Missing issue scores | 0 |
| Current consecutive no-new cycles | 2 |
| Final result | Complete after stress/fuzz success and reviewer no-new confirmation |
| Latest branch state | `red/audit-failing-evidence` pushed cleanly |

## Final Verification

| Area | Evidence | Result |
| --- | --- | --- |
| Service/requester boundary | `cargo test --lib service` | 199 passed |
| Network requester lifecycle | `cargo test --lib requester` | 13 passed |
| Cross-node delivery | `cargo test --lib cross_nodes` | 9 passed |
| Security regression suite | `cargo test --lib security` | 55 passed |
| Local stress fuzz | `P2P_FUZZ_NODES=44 P2P_FUZZ_STEPS=3600 P2P_FUZZ_SEED=249050 cargo test --lib fuzz_random_adversarial_node_actions_must_not_panic_connection_tasks` | 1 passed |
| Reviewer stress fuzz | `P2P_FUZZ_NODES=48 P2P_FUZZ_STEPS=2400 P2P_FUZZ_SEED=248250 cargo test --lib fuzz_random_adversarial_node_actions_must_not_panic_connection_tasks` | 1 passed |

## Latest Accepted Issues

| Issue | Score | Status | Root Cause | Fix Summary |
| --- | ---: | --- | --- | --- |
| ISSUE-248 | 82 | Fixed | Stream benchmark long-run report treated repeated short clusters as plain pass and ignored sharp RSS growth. | Report now labels repeated short-cluster iterations and marks sharp RSS growth as `resource-warning`. |
| ISSUE-247 | 88 | Fixed | Replicated-KV full sync capped each snapshot page but not cumulative staged snapshot state. | Added aggregate staged-slot cap before accepting continuation pages. |
| ISSUE-246 | 54 | Fixed | Pubsub registration could return a live-looking handle after internal admission failure. | Mark rejected publisher/subscriber handles unregistered and reject later requester calls. |
| ISSUE-245 | 58 | Fixed | Replicated-KV initial full sync exposed partial snapshot pages before terminal completion. | Stage snapshot pages until a validated terminal page commits atomically. |
| ISSUE-244 | 72 | Fixed | Handshake replay tokens became reusable after replay-cache pressure. | Added compact rotating replay window. |

## Final No-New Cycles

| Cycle | Scope | Reviewer | Local Evidence | Result |
| --- | --- | --- | --- | --- |
| 2 after ISSUE-248 | Service/requester/control boundary, local delivery, queue pressure, dropped handles, duplicate service registration, churn | `Ptolemy` | service, requester, cross_nodes, security, 44-node fuzz | No new score-80+ issue |
| 1 after ISSUE-248 | Discovery, neighbours, connection lifecycle, stopped peers, duplicate connects, churn | `James` | discovery, security, cross_nodes, stopped, 48-node fuzz | No new score-80+ issue |

## Issue Families

| Family | Representative Issue Range | Main Risk Covered |
| --- | --- | --- |
| Authentication and identity binding | ISSUE-001, ISSUE-002, ISSUE-014..018, ISSUE-146, ISSUE-176, ISSUE-189, ISSUE-194, ISSUE-207, ISSUE-244 | Forged identity, replay, source spoofing |
| Routing and discovery | ISSUE-003..010, ISSUE-033, ISSUE-044, ISSUE-092, ISSUE-103, ISSUE-160, ISSUE-167, ISSUE-190, ISSUE-210..214 | Route flapping, stale routes, malformed discovery, capped sync behavior |
| Streams and delivery | ISSUE-011..013, ISSUE-049, ISSUE-050, ISSUE-056, ISSUE-091, ISSUE-117..120, ISSUE-149, ISSUE-156, ISSUE-169, ISSUE-180, ISSUE-197, ISSUE-217, ISSUE-220, ISSUE-229, ISSUE-238 | False success, backpressure, relay correctness, stream resource bounds |
| Service/requester lifecycle | ISSUE-028..030, ISSUE-052, ISSUE-053, ISSUE-060, ISSUE-072, ISSUE-073, ISSUE-076, ISSUE-108, ISSUE-125, ISSUE-234, ISSUE-246 | Dropped handles, duplicate registration, stale requester behavior |
| Pubsub | ISSUE-020, ISSUE-026, ISSUE-039, ISSUE-043, ISSUE-048, ISSUE-058, ISSUE-069, ISSUE-070, ISSUE-074, ISSUE-075, ISSUE-080, ISSUE-100, ISSUE-106, ISSUE-107, ISSUE-115, ISSUE-116, ISSUE-121, ISSUE-123, ISSUE-124, ISSUE-126, ISSUE-142, ISSUE-150, ISSUE-155, ISSUE-163, ISSUE-168, ISSUE-178, ISSUE-185, ISSUE-188, ISSUE-205, ISSUE-228, ISSUE-231, ISSUE-236, ISSUE-240..243 | RPC binding, membership integrity, heartbeat/chunk correctness, resource caps |
| Replicated KV | ISSUE-023, ISSUE-025, ISSUE-027, ISSUE-031, ISSUE-032, ISSUE-034, ISSUE-037, ISSUE-038, ISSUE-045..047, ISSUE-059, ISSUE-071, ISSUE-077, ISSUE-081..089, ISSUE-095, ISSUE-096, ISSUE-099, ISSUE-110, ISSUE-111, ISSUE-131, ISSUE-138, ISSUE-140, ISSUE-141, ISSUE-143, ISSUE-154, ISSUE-171, ISSUE-175, ISSUE-184, ISSUE-186, ISSUE-196, ISSUE-233, ISSUE-237, ISSUE-245, ISSUE-247 | Snapshot validation, repair state, resource caps, stale/unsolicited responses |
| Alias service | ISSUE-019, ISSUE-022, ISSUE-029, ISSUE-035, ISSUE-036, ISSUE-041, ISSUE-090, ISSUE-101, ISSUE-109, ISSUE-127, ISSUE-130, ISSUE-132, ISSUE-137, ISSUE-148, ISSUE-152, ISSUE-158, ISSUE-179, ISSUE-183, ISSUE-206, ISSUE-208, ISSUE-235, ISSUE-239 | Alias lifecycle, cache poisoning, waiter/resource caps |
| Metrics and visualization | ISSUE-040, ISSUE-061, ISSUE-062, ISSUE-064, ISSUE-068, ISSUE-078, ISSUE-079, ISSUE-102, ISSUE-104, ISSUE-105, ISSUE-128, ISSUE-129, ISSUE-165, ISSUE-193, ISSUE-195, ISSUE-200..204, ISSUE-226, ISSUE-232 | Disclosure, stale metrics, scan/Info caps, lifecycle cleanup |
| Churn and graceful stop | ISSUE-004, ISSUE-051, ISSUE-063, ISSUE-065..067, ISSUE-118, ISSUE-122, ISSUE-133..136, ISSUE-144, ISSUE-151, ISSUE-157, ISSUE-161, ISSUE-162, ISSUE-164, ISSUE-170, ISSUE-187, ISSUE-211..225 | Stop propagation, cleanup, stale events, queue pressure under churn |
| Tooling and evidence | ISSUE-191, ISSUE-209, ISSUE-248 | Examples, fuzz profile validity, benchmark evidence quality |

## Notes

- Every accepted issue had failing-test evidence and reviewer confirmation at
  the time it was accepted.
- This compact version intentionally removes the old repeated per-cycle prose.
  Use git history before the cleanup commit for the full verbose audit trail.
