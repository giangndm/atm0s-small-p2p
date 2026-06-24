# Throughput Benchmark Summary

Generated from release-mode runs of the `stream_limit_benchmark` example.
Each profile ran by itself, opened streams from one source node to random peers,
and kept each throughput stream live for 300 seconds.

## Result

| Profile | Nodes | Streams | Failed | Sent bytes | Received bytes | Min kbps | Max kbps | Avg kbps | Sum kbps | Elapsed | Max CPU | Max RSS | Status | Detail |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| throughput-3-nodes | 3 | 6 | 0 | 366629617664 | 366629617664 | 1610946.02 | 1642952.66 | 1629463.62 | 9776781.75 | 304.31s | 28.16% | 28984 KiB | pass | [3-node report](throughput_benchmark_3_nodes.md) |
| throughput-5-nodes | 5 | 12 | 0 | 491260608512 | 491260608512 | 913445.88 | 1315728.87 | 1091689.17 | 13100270.07 | 306.31s | 36.34% | 38484 KiB | pass | [5-node report](throughput_benchmark_5_nodes.md) |
| throughput-10-nodes | 10 | 24 | 0 | 546170077184 | 546170077184 | 480934.20 | 685596.52 | 606854.61 | 14564510.54 | 311.32s | 43.52% | 54740 KiB | pass | [10-node report](throughput_benchmark_10_nodes.md) |

## Notes

- Bandwidth values are per-stream min/max/avg plus sum of all writer streams.
- CPU and RSS are process-level samples because all benchmark nodes run in one
  OS process.
- All three runs reported no stream open, write, or read failures.

## Commands

```bash
CARGO_BUILD_JOBS=8 RUST_LOG=error cargo run --release --example stream_limit_benchmark -- --profile throughput-3-nodes --profiles benchmarks/stream_limit_profiles.yaml --report docs/throughput_benchmark_3_nodes.md
CARGO_BUILD_JOBS=8 RUST_LOG=error cargo run --release --example stream_limit_benchmark -- --profile throughput-5-nodes --profiles benchmarks/stream_limit_profiles.yaml --report docs/throughput_benchmark_5_nodes.md
CARGO_BUILD_JOBS=8 RUST_LOG=error cargo run --release --example stream_limit_benchmark -- --profile throughput-10-nodes --profiles benchmarks/stream_limit_profiles.yaml --report docs/throughput_benchmark_10_nodes.md
```
