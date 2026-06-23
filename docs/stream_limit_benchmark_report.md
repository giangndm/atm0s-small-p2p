# Stream Limit Benchmark Report

Generated at unix timestamp: `1782208845`

This benchmark opens streams from one source node to random peer nodes. Each opened stream writes at a random kbps target from the configured range, and that target changes once per second to approximate voice/video user traffic. Random latency is applied before each stream-open attempt to model different global-network paths.

| Profile | Nodes | Attempts | Opened | Failed | Inbound streams | Sent bytes | Received bytes | Max latency | Elapsed | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| stream-limit-3-nodes | 3 | 24 | 24 | 0 | 24 | 644990 | 644990 | 117ms | 8.00s | pass |
| stream-limit-5-nodes | 5 | 48 | 48 | 0 | 48 | 2010010 | 2010010 | 233ms | 12.80s | pass |
| stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.20s | pass |

## Profiles

- `stream-limit-3-nodes`: source peer `1`, nodes `3`, latency `10..=120ms`, target stream rate `32..=160 kbps`, stream live range `1..=3s`, open timeout `2000ms`, seed `31001`.
- `stream-limit-5-nodes`: source peer `1`, nodes `5`, latency `25..=250ms`, target stream rate `32..=256 kbps`, stream live range `1..=4s`, open timeout `2000ms`, seed `51001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.

## Failures

No stream open, write, or read failures were observed in this run.

## Reproduce

```bash
RUST_LOG=error CARGO_BUILD_JOBS=8 cargo run --example stream_limit_benchmark -- --profiles benchmarks/stream_limit_profiles.yaml --report docs/stream_limit_benchmark_report.md
```
