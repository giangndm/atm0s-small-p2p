# Stream Limit Benchmark Report

Generated at unix timestamp: `1782211094`

This benchmark opens streams from one source node to random peer nodes. Each opened stream writes at a random kbps target from the configured range, and that target changes once per second to approximate voice/video user traffic. Random latency is applied before each stream-open attempt to model different global-network paths.

CPU and memory samples are process-level because the benchmark hosts all nodes inside one OS process. Per-node tables report stream and byte counters collected inside the benchmark.

| Iteration | Profile | Nodes | Attempts | Opened | Failed | Inbound streams | Sent bytes | Received bytes | Max latency | Elapsed | Result |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 1 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 2 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.06s | pass |
| 3 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 4 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9203211 | 9203211 | 499ms | 31.06s | pass |
| 5 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 6 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 7 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9202185 | 9202185 | 499ms | 31.07s | pass |
| 8 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9209110 | 9209110 | 499ms | 31.04s | pass |
| 9 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 10 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 11 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 12 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.06s | pass |
| 13 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 14 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 15 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 16 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 17 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 18 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 19 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 20 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 21 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.05s | pass |
| 22 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 23 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 24 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 25 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 26 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 27 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 28 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 29 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 30 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 31 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.05s | pass |
| 32 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.05s | pass |
| 33 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 34 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 35 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.60s | pass |
| 36 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.03s | pass |
| 37 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 38 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 39 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.03s | pass |
| 40 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 41 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.03s | pass |
| 42 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 43 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 44 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 45 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 46 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 47 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 31.04s | pass |
| 48 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 49 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 50 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 51 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 52 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 53 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 54 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 55 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 56 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.03s | pass |
| 57 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 58 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 59 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.04s | pass |
| 60 | stream-limit-10-nodes | 10 | 90 | 90 | 0 | 90 | 9210460 | 9210460 | 499ms | 30.03s | pass |

## Resource Charts

### stream-limit-10-nodes iteration 1

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,79.3 67.4,72.4 89.0,48.7 110.7,34.8 132.4,59.6 154.1,24.0 175.7,59.0 197.4,52.4 219.1,127.8 240.8,126.5 262.5,126.5 284.1,125.8 305.8,127.1 327.5,127.8 349.2,127.8 370.8,127.8 392.5,127.8 414.2,129.1 435.9,87.1 457.5,32.3 479.2,58.3 500.9,43.2 522.6,37.3 544.3,32.7 565.9,107.5 587.6,116.1 609.3,130.4 631.0,131.7 652.6,130.4 674.3,132.4 696.0,152.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 12.57, last 0.31</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.9 67.4,25.2 89.0,25.1 110.7,25.1 132.4,25.1 154.1,24.9 175.7,24.8 197.4,24.6 219.1,24.5 240.8,24.5 262.5,24.5 284.1,24.4 305.8,24.4 327.5,24.4 349.2,24.4 370.8,24.4 392.5,24.4 414.2,24.3 435.9,24.2 457.5,24.2 479.2,24.2 500.9,24.1 522.6,24.0 544.3,24.0 565.9,43.8 587.6,43.3 609.3,43.3 631.0,43.2 652.6,43.2 674.3,43.2 696.0,47.3" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 40640.00, last 33476.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 2

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,100.3 67.4,130.9 89.0,130.9 110.7,130.4 132.4,54.3 154.1,26.4 175.7,37.9 197.4,124.8 219.1,126.6 240.8,61.3 262.5,24.0 284.1,102.9 305.8,53.9 327.5,83.2 349.2,128.5 370.8,129.1 392.5,87.5 414.2,126.0 435.9,124.8 457.5,126.1 479.2,116.4 500.9,86.9 522.6,91.1 544.3,61.9 565.9,125.4 587.6,128.5 609.3,129.1 631.0,129.7 652.6,131.5 674.3,130.3 696.0,151.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 13.47, last 0.44</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.7 67.4,25.4 89.0,25.4 110.7,25.4 132.4,25.0 154.1,24.8 175.7,24.7 197.4,24.6 219.1,24.6 240.8,24.5 262.5,24.4 284.1,24.2 305.8,24.2 327.5,24.2 349.2,24.2 370.8,24.2 392.5,24.2 414.2,24.3 435.9,24.2 457.5,24.2 479.2,24.2 500.9,24.1 522.6,24.1 544.3,24.1 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 54704.00, last 54704.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 3

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,91.6 91.2,90.0 113.6,91.5 136.0,93.1 158.4,87.0 180.8,82.4 203.2,83.9 225.6,83.9 248.0,77.7 270.4,76.2 292.8,80.8 315.2,82.4 337.6,76.4 360.0,83.9 382.4,83.9 404.8,83.9 427.2,85.4 449.6,83.9 472.0,83.9 494.4,67.0 516.8,60.9 539.2,63.9 561.6,56.2 584.0,56.3 606.4,71.6 628.8,68.5 651.2,70.1 673.6,76.3 696.0,70.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 5.37, last 3.49</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,25.0 68.8,24.8 91.2,24.7 113.6,24.6 136.0,24.5 158.4,24.4 180.8,24.4 203.2,24.5 225.6,24.4 248.0,24.4 270.4,24.2 292.8,24.0 315.2,24.3 337.6,24.3 360.0,24.2 382.4,24.2 404.8,24.2 427.2,24.2 449.6,24.2 472.0,24.1 494.4,24.1 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,58.9 628.8,58.9 651.2,59.0 673.6,58.9 696.0,63.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 70312.00, last 49132.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 4

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,82.2 89.0,78.3 110.7,73.2 132.4,74.5 154.1,71.9 175.7,68.0 197.4,60.3 219.1,61.5 240.8,74.5 262.5,59.0 284.1,65.4 305.8,65.5 327.5,74.5 349.2,71.8 370.8,68.1 392.5,73.2 414.2,71.9 435.9,68.0 457.5,68.1 479.2,78.3 500.9,65.4 522.6,62.8 544.3,56.4 565.9,75.8 587.6,69.5 609.3,73.2 631.0,78.5 652.6,82.3 674.3,75.8 696.0,140.5" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 6.37, last 0.75</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.6 67.4,24.9 89.0,24.9 110.7,24.8 132.4,24.7 154.1,24.6 175.7,24.2 197.4,24.2 219.1,24.0 240.8,33.3 262.5,32.0 284.1,32.0 305.8,32.7 327.5,32.9 349.2,33.0 370.8,33.1 392.5,33.1 414.2,33.1 435.9,33.1 457.5,33.1 479.2,41.4 500.9,41.3 522.6,41.1 544.3,53.4 565.9,58.2 587.6,57.5 609.3,57.5 631.0,57.4 652.6,57.5 674.3,57.3 696.0,57.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 64860.00, last 48616.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 5

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,80.6 89.0,87.2 110.7,91.7 132.4,85.0 154.1,87.3 175.7,81.7 197.4,71.7 219.1,79.5 240.8,69.5 262.5,64.0 284.1,75.1 305.8,64.0 327.5,74.0 349.2,83.9 370.8,77.3 392.5,81.7 414.2,84.0 435.9,68.4 457.5,76.2 479.2,73.9 500.9,69.5 522.6,72.8 544.3,74.0 565.9,77.3 587.6,83.9 609.3,85.0 631.0,87.2 652.6,90.6 674.3,83.9 696.0,132.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.43, last 1.31</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.8 67.4,25.5 89.0,25.3 110.7,25.4 132.4,25.4 154.1,25.3 175.7,25.1 197.4,25.0 219.1,24.9 240.8,24.8 262.5,24.7 284.1,24.7 305.8,24.6 327.5,24.6 349.2,24.5 370.8,24.4 392.5,24.4 414.2,24.3 435.9,24.3 457.5,24.3 479.2,24.2 500.9,24.2 522.6,24.0 544.3,24.0 565.9,24.0 587.6,25.1 609.3,25.1 631.0,25.0 652.6,25.0 674.3,25.0 696.0,24.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 70040.00, last 69544.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 6

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,83.5 89.0,86.5 110.7,85.5 132.4,78.4 154.1,83.5 175.7,78.4 197.4,77.5 219.1,68.4 240.8,78.4 262.5,72.4 284.1,76.5 305.8,76.5 327.5,77.5 349.2,78.5 370.8,76.4 392.5,72.4 414.2,64.4 435.9,62.4 457.5,68.6 479.2,63.1 500.9,64.3 522.6,63.3 544.3,72.4 565.9,67.4 587.6,68.4 609.3,72.4 631.0,77.5 652.6,75.4 674.3,78.4 696.0,127.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.18, last 1.75</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.7 67.4,24.4 89.0,24.4 110.7,24.6 132.4,24.5 154.1,24.5 175.7,24.4 197.4,24.3 219.1,24.1 240.8,24.2 262.5,24.2 284.1,24.2 305.8,24.2 327.5,24.1 349.2,24.1 370.8,24.0 392.5,24.0 414.2,27.1 435.9,27.1 457.5,27.1 479.2,27.1 500.9,27.1 522.6,27.1 544.3,27.2 565.9,27.1 587.6,27.1 609.3,27.1 631.0,27.1 652.6,27.1 674.3,27.2 696.0,26.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 85672.00, last 83768.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 7

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,84.4 89.0,80.6 110.7,83.4 132.4,80.5 154.1,82.0 175.7,78.8 197.4,72.1 219.1,75.0 240.8,63.7 262.5,62.8 284.1,60.1 305.8,66.4 327.5,72.6 349.2,62.7 370.8,65.5 392.5,69.2 414.2,66.9 435.9,68.3 457.5,67.3 479.2,76.8 500.9,84.4 522.6,75.0 544.3,69.3 565.9,76.8 587.6,82.5 609.3,84.4 631.0,83.4 652.6,81.5 674.3,80.6 696.0,116.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.74, last 2.62</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.6 67.4,24.4 89.0,24.4 110.7,24.4 132.4,24.3 154.1,24.2 175.7,24.2 197.4,24.2 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.0 305.8,24.0 327.5,24.0 349.2,26.6 370.8,26.6 392.5,26.5 414.2,26.5 435.9,26.5 457.5,26.4 479.2,26.4 500.9,26.3 522.6,26.4 544.3,26.3 565.9,26.4 587.6,26.7 609.3,26.7 631.0,26.6 652.6,26.6 674.3,26.6 696.0,26.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 105364.00, last 103428.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 8

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,90.0 89.0,82.7 110.7,92.7 132.4,97.3 154.1,101.0 175.7,97.3 197.4,98.2 219.1,97.3 240.8,94.6 262.5,95.5 284.1,97.3 305.8,97.3 327.5,101.0 349.2,102.8 370.8,102.8 392.5,95.4 414.2,100.1 435.9,101.0 457.5,96.4 479.2,94.5 500.9,87.4 522.6,92.7 544.3,98.2 565.9,99.1 587.6,93.6 609.3,95.5 631.0,94.6 652.6,98.3 674.3,103.7 696.0,140.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.98, last 1.06</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.6 67.4,24.4 89.0,24.4 110.7,24.4 132.4,24.3 154.1,24.2 175.7,24.2 197.4,24.2 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.0 327.5,24.0 349.2,24.0 370.8,24.0 392.5,25.7 414.2,25.7 435.9,25.7 457.5,25.6 479.2,25.6 500.9,25.9 522.6,25.9 544.3,26.2 565.9,49.2 587.6,47.0 609.3,46.6 631.0,46.5 652.6,46.4 674.3,46.4 696.0,47.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 121160.00, last 99336.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 9

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,77.8 89.0,85.5 110.7,85.6 132.4,85.5 154.1,81.7 175.7,74.1 197.4,63.8 219.1,66.4 240.8,62.5 262.5,66.3 284.1,71.4 305.8,71.5 327.5,77.9 349.2,75.3 370.8,80.4 392.5,74.0 414.2,77.8 435.9,83.0 457.5,81.6 479.2,57.4 500.9,76.5 522.6,72.8 544.3,75.2 565.9,74.0 587.6,76.6 609.3,79.1 631.0,77.8 652.6,86.8 674.3,84.3 696.0,125.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 6.43, last 1.50</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.0 67.4,24.7 89.0,24.7 110.7,24.6 132.4,24.5 154.1,24.5 175.7,24.3 197.4,24.2 219.1,24.2 240.8,24.1 262.5,24.1 284.1,24.2 305.8,24.1 327.5,24.2 349.2,24.1 370.8,24.1 392.5,24.0 414.2,24.0 435.9,24.1 457.5,24.1 479.2,24.0 500.9,24.1 522.6,25.8 544.3,26.0 565.9,26.0 587.6,26.2 609.3,36.9 631.0,37.0 652.6,37.0 674.3,36.9 696.0,36.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 118992.00, last 107476.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 10

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,36.1 67.4,78.1 89.0,84.0 110.7,86.4 132.4,86.3 154.1,82.8 175.7,76.8 197.4,79.2 219.1,72.0 240.8,76.8 262.5,66.0 284.1,26.4 305.8,40.8 327.5,54.0 349.2,69.6 370.8,48.0 392.5,27.7 414.2,24.1 435.9,72.0 457.5,28.9 479.2,82.8 500.9,45.6 522.6,49.2 544.3,67.3 565.9,24.0 587.6,72.0 609.3,69.5 631.0,40.8 652.6,76.8 674.3,76.9 696.0,121.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 6.87, last 1.81</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.6 67.4,24.4 89.0,24.4 110.7,24.3 132.4,24.3 154.1,24.2 175.7,24.1 197.4,24.0 219.1,35.6 240.8,48.0 262.5,54.3 284.1,54.2 305.8,54.1 327.5,54.1 349.2,54.1 370.8,54.0 392.5,53.9 414.2,53.9 435.9,53.8 457.5,53.7 479.2,53.7 500.9,53.7 522.6,53.7 544.3,53.7 565.9,53.6 587.6,53.6 609.3,53.6 631.0,53.5 652.6,53.5 674.3,53.5 696.0,55.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 124688.00, last 95024.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 11

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,32.5 67.4,85.8 89.0,91.1 110.7,86.9 132.4,83.7 154.1,90.0 175.7,84.8 197.4,49.2 219.1,72.2 240.8,81.6 262.5,71.2 284.1,81.6 305.8,81.7 327.5,90.0 349.2,63.8 370.8,51.3 392.5,67.0 414.2,83.7 435.9,84.8 457.5,92.1 479.2,84.8 500.9,91.1 522.6,87.9 544.3,87.9 565.9,91.0 587.6,87.9 609.3,84.8 631.0,24.0 652.6,35.0 674.3,52.4 696.0,111.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.87, last 2.68</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.9 67.4,24.7 89.0,24.7 110.7,24.7 132.4,24.6 154.1,24.6 175.7,24.5 197.4,24.5 219.1,24.5 240.8,24.4 262.5,24.4 284.1,24.4 305.8,24.4 327.5,24.3 349.2,24.3 370.8,24.2 392.5,24.2 414.2,24.2 435.9,24.2 457.5,24.1 479.2,24.1 500.9,24.1 522.6,24.1 544.3,24.1 565.9,24.1 587.6,24.1 609.3,24.1 631.0,24.1 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 114324.00, last 114324.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 12

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,27.4 67.4,92.3 89.0,98.5 110.7,78.6 132.4,97.3 154.1,102.0 175.7,110.8 197.4,102.6 219.1,107.4 240.8,102.6 262.5,108.8 284.1,107.4 305.8,109.5 327.5,107.4 349.2,106.7 370.8,86.2 392.5,24.0 414.2,100.0 435.9,109.5 457.5,65.0 479.2,102.0 500.9,109.4 522.6,93.7 544.3,102.6 565.9,103.3 587.6,97.1 609.3,98.5 631.0,98.6 652.6,104.7 674.3,104.7 696.0,132.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 12.04, last 2.12</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.1 67.4,24.8 89.0,24.8 110.7,24.5 132.4,24.5 154.1,24.3 175.7,24.3 197.4,24.3 219.1,24.2 240.8,24.2 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.0 392.5,24.0 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,25.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 128804.00, last 127200.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 13

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,114.9 91.2,117.5 113.6,118.7 136.0,113.0 158.4,100.4 180.8,103.5 203.2,110.5 225.6,112.4 248.0,109.2 270.4,110.5 292.8,109.8 315.2,109.9 337.6,115.6 360.0,114.3 382.4,114.3 404.8,112.4 427.2,110.5 449.6,109.2 472.0,114.3 494.4,111.7 516.8,108.1 539.2,113.0 561.6,108.0 584.0,114.3 606.4,114.9 628.8,116.2 651.2,118.7 673.6,117.4 696.0,118.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 13.04, last 3.75</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.7 68.8,24.6 91.2,24.6 113.6,24.6 136.0,24.6 158.4,24.5 180.8,24.5 203.2,24.5 225.6,24.4 248.0,24.4 270.4,24.4 292.8,24.3 315.2,24.3 337.6,24.2 360.0,24.2 382.4,24.2 404.8,24.2 427.2,24.2 449.6,24.2 472.0,24.1 494.4,24.1 516.8,24.1 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 146092.00, last 146092.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 14

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,90.0 91.2,87.9 113.6,87.8 136.0,86.8 158.4,84.7 180.8,81.4 203.2,83.5 225.6,81.4 248.0,83.5 270.4,84.6 292.8,80.3 315.2,83.5 337.6,71.7 360.0,86.7 382.4,85.6 404.8,85.7 427.2,83.6 449.6,80.3 472.0,83.5 494.4,84.7 516.8,83.5 539.2,81.3 561.6,84.6 584.0,85.7 606.4,84.6 628.8,84.6 651.2,80.2 673.6,83.6 696.0,90.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.62, last 3.81</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.7 68.8,24.5 91.2,24.5 113.6,24.5 136.0,24.5 158.4,24.4 180.8,24.3 203.2,24.3 225.6,24.3 248.0,24.2 270.4,24.2 292.8,24.2 315.2,24.2 337.6,24.2 360.0,24.2 382.4,24.2 404.8,24.2 427.2,24.1 449.6,24.1 472.0,24.1 494.4,24.1 516.8,24.1 539.2,24.1 561.6,24.1 584.0,24.1 606.4,24.1 628.8,24.1 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 161804.00, last 161804.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 15

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,111.3 67.4,110.1 89.0,58.4 110.7,86.9 132.4,96.0 154.1,81.2 175.7,86.5 197.4,79.4 219.1,96.0 240.8,45.6 262.5,78.3 284.1,84.8 305.8,62.6 327.5,84.9 349.2,71.6 370.8,54.4 392.5,74.6 414.2,103.0 435.9,75.3 457.5,62.1 479.2,56.7 500.9,89.4 522.6,51.4 544.3,71.6 565.9,53.0 587.6,24.0 609.3,77.4 631.0,43.2 652.6,64.2 674.3,98.5 696.0,102.3" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 19.92, last 8.10</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.7 67.4,24.6 89.0,24.6 110.7,24.5 132.4,24.5 154.1,24.4 175.7,24.4 197.4,24.3 219.1,24.3 240.8,24.3 262.5,24.3 284.1,24.2 305.8,24.2 327.5,24.2 349.2,24.2 370.8,24.2 392.5,24.2 414.2,24.1 435.9,24.1 457.5,24.1 479.2,24.1 500.9,24.1 522.6,24.1 544.3,24.0 565.9,24.0 587.6,24.1 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 176176.00, last 176172.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 16

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,56.7 67.4,33.7 89.0,94.5 110.7,88.3 132.4,88.7 154.1,103.0 175.7,110.6 197.4,86.9 219.1,42.4 240.8,95.6 262.5,99.8 284.1,66.4 305.8,106.0 327.5,92.5 349.2,49.4 370.8,42.9 392.5,69.1 414.2,77.5 435.9,99.8 457.5,24.0 479.2,46.1 500.9,46.8 522.6,73.8 544.3,92.9 565.9,67.1 587.6,94.5 609.3,86.8 631.0,103.6 652.6,108.7 674.3,65.9 696.0,99.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 21.42, last 9.23</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.5 67.4,24.4 89.0,24.4 110.7,24.3 132.4,24.3 154.1,24.3 175.7,24.3 197.4,24.3 219.1,24.2 240.8,24.2 262.5,24.2 284.1,24.2 305.8,24.2 327.5,24.1 349.2,24.1 370.8,24.1 392.5,24.1 414.2,24.1 435.9,24.1 457.5,24.1 479.2,24.1 500.9,24.1 522.6,24.1 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 194756.00, last 194756.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 17

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,57.3 67.4,32.3 89.0,74.2 110.7,61.4 132.4,45.1 154.1,61.0 175.7,51.0 197.4,64.8 219.1,76.6 240.8,68.6 262.5,24.0 284.1,41.3 305.8,65.2 327.5,78.0 349.2,57.8 370.8,119.8 392.5,120.7 414.2,121.2 435.9,121.7 457.5,120.3 479.2,122.1 500.9,86.9 522.6,101.5 544.3,61.0 565.9,101.9 587.6,93.9 609.3,102.9 631.0,100.1 652.6,118.9 674.3,105.7 696.0,126.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 17.52, last 3.93</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.5 67.4,24.3 89.0,24.3 110.7,24.3 132.4,24.2 154.1,24.2 175.7,24.1 197.4,24.1 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.0 392.5,24.0 414.2,24.0 435.9,24.0 457.5,27.4 479.2,27.4 500.9,27.4 522.6,27.4 544.3,27.4 565.9,27.4 587.6,27.4 609.3,27.4 631.0,27.4 652.6,27.4 674.3,27.4 696.0,28.3" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 208424.00, last 201608.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 18

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,71.2 67.4,117.4 89.0,105.2 110.7,53.9 132.4,70.2 154.1,85.5 175.7,75.9 197.4,60.0 219.1,36.7 240.8,63.5 262.5,62.6 284.1,24.0 305.8,78.9 327.5,113.9 349.2,102.2 370.8,77.3 392.5,83.9 414.2,110.8 435.9,104.7 457.5,118.9 479.2,117.9 500.9,120.0 522.6,117.4 544.3,118.9 565.9,119.5 587.6,120.4 609.3,122.0 631.0,122.0 652.6,122.5 674.3,121.0 696.0,137.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 16.23, last 2.31</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.3 67.4,24.2 89.0,24.2 110.7,24.2 132.4,24.2 154.1,24.2 175.7,24.2 197.4,24.2 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.1 392.5,24.0 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 222436.00, last 222436.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 19

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,75.6 91.2,82.7 113.6,79.7 136.0,80.8 158.4,76.6 180.8,77.6 203.2,71.4 225.6,73.5 248.0,63.2 270.4,68.3 292.8,72.4 315.2,76.6 337.6,76.7 360.0,77.7 382.4,79.7 404.8,72.5 427.2,76.6 449.6,76.6 472.0,76.6 494.4,78.6 516.8,75.5 539.2,75.6 561.6,75.6 584.0,75.5 606.4,77.6 628.8,77.6 651.2,80.7 673.6,79.7 696.0,84.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.99, last 4.31</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 236548.00, last 235364.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 20

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,64.6 89.0,64.5 110.7,60.8 132.4,62.0 154.1,57.0 175.7,56.9 197.4,54.5 219.1,53.2 240.8,48.1 262.5,37.8 284.1,45.5 305.8,54.4 327.5,49.4 349.2,53.1 370.8,48.1 392.5,36.6 414.2,48.1 435.9,49.2 457.5,50.7 479.2,54.5 500.9,49.4 522.6,49.4 544.3,46.8 565.9,50.6 587.6,53.2 609.3,58.2 631.0,57.0 652.6,58.2 674.3,53.1 696.0,102.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 6.49, last 2.62</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.7 67.4,24.5 89.0,24.5 110.7,24.5 132.4,24.4 154.1,24.3 175.7,24.2 197.4,24.2 219.1,24.2 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.1 392.5,24.1 414.2,24.1 435.9,24.1 457.5,24.1 479.2,24.1 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 248464.00, last 248432.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 21

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,89.2 91.2,91.6 113.6,90.0 136.0,90.0 158.4,79.0 180.8,73.1 203.2,69.6 225.6,55.3 248.0,61.2 270.4,75.7 292.8,87.4 315.2,84.9 337.6,91.0 360.0,89.2 382.4,90.0 404.8,89.2 427.2,89.1 449.6,90.0 472.0,88.3 494.4,91.7 516.8,89.2 539.2,89.2 561.6,88.3 584.0,90.0 606.4,90.8 628.8,91.6 651.2,92.5 673.6,93.3 696.0,92.5" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.74, last 4.68</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.0 91.2,24.0 113.6,24.0 136.0,24.0 158.4,24.0 180.8,33.4 203.2,32.3 225.6,55.3 248.0,54.8 270.4,54.7 292.8,54.6 315.2,54.6 337.6,54.6 360.0,54.5 382.4,54.5 404.8,54.4 427.2,54.4 449.6,54.3 472.0,54.3 494.4,54.2 516.8,54.2 539.2,54.2 561.6,54.1 584.0,54.1 606.4,54.0 628.8,54.0 651.2,54.0 673.6,53.9 696.0,54.5" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 270456.00, last 208032.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 22

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,84.9 89.0,83.9 110.7,83.0 132.4,80.3 154.1,81.2 175.7,82.1 197.4,74.7 219.1,76.6 240.8,78.4 262.5,79.3 284.1,74.7 305.8,69.2 327.5,72.0 349.2,78.4 370.8,78.5 392.5,72.0 414.2,67.4 435.9,68.3 457.5,82.1 479.2,80.3 500.9,79.3 522.6,77.6 544.3,79.4 565.9,78.4 587.6,79.4 609.3,80.5 631.0,84.0 652.6,84.9 674.3,84.0 696.0,117.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.92, last 2.62</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,25.2 67.4,25.1 89.0,25.1 110.7,25.0 132.4,25.0 154.1,24.9 175.7,24.9 197.4,24.8 219.1,24.7 240.8,24.7 262.5,24.6 284.1,24.6 305.8,24.5 327.5,24.5 349.2,24.4 370.8,24.4 392.5,24.3 414.2,24.3 435.9,24.2 457.5,24.2 479.2,24.1 500.9,24.1 522.6,24.1 544.3,24.1 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 224904.00, last 224904.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 23

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,71.2 91.2,73.2 113.6,74.2 136.0,71.1 158.4,68.1 180.8,68.0 203.2,67.0 225.6,65.9 248.0,69.0 270.4,62.8 292.8,63.9 315.2,65.9 337.6,69.0 360.0,70.0 382.4,69.0 404.8,67.0 427.2,66.9 449.6,68.0 472.0,72.1 494.4,70.0 516.8,67.0 539.2,70.1 561.6,69.1 584.0,68.0 606.4,71.1 628.8,71.1 651.2,74.2 673.6,72.1 696.0,73.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.96, last 5.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.4 68.8,24.3 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.2 203.2,24.2 225.6,24.2 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 238832.00, last 237668.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 24

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,90.5 91.2,88.8 113.6,90.4 136.0,89.5 158.4,88.7 180.8,87.0 203.2,84.6 225.6,85.4 248.0,85.3 270.4,85.3 292.8,81.1 315.2,86.2 337.6,86.2 360.0,85.4 382.4,87.9 404.8,87.9 427.2,86.2 449.6,83.7 472.0,87.0 494.4,88.8 516.8,85.4 539.2,84.5 561.6,84.6 584.0,84.5 606.4,87.0 628.8,89.6 651.2,92.1 673.6,88.7 696.0,93.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.80, last 4.62</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 257996.00, last 257996.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 25

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,62.4 91.2,63.6 113.6,63.4 136.0,62.5 158.4,60.3 180.8,55.9 203.2,52.6 225.6,59.2 248.0,55.9 270.4,53.6 292.8,54.7 315.2,54.8 337.6,55.8 360.0,60.2 382.4,57.0 404.8,61.3 427.2,58.0 449.6,55.9 472.0,60.2 494.4,59.1 516.8,55.9 539.2,54.8 561.6,55.9 584.0,56.9 606.4,60.3 628.8,60.3 651.2,61.3 673.6,62.5 696.0,64.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 7.49, last 5.18</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.6 68.8,24.5 91.2,24.5 113.6,24.5 136.0,24.3 158.4,24.3 180.8,24.2 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.0 494.4,24.1 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.1 673.6,24.1 696.0,24.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 267972.00, last 267848.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 26

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,69.7 91.2,68.8 113.6,73.7 136.0,70.7 158.4,72.7 180.8,69.7 203.2,66.8 225.6,66.8 248.0,66.8 270.4,66.7 292.8,60.8 315.2,67.7 337.6,65.8 360.0,67.8 382.4,67.7 404.8,63.7 427.2,65.8 449.6,65.7 472.0,66.7 494.4,66.8 516.8,64.8 539.2,67.8 561.6,59.1 584.0,49.1 606.4,48.8 628.8,71.7 651.2,73.7 673.6,74.8 696.0,74.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.31, last 5.11</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.2 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.1 494.4,24.1 516.8,24.1 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 285084.00, last 285072.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 27

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,71.9 91.2,72.0 113.6,72.0 136.0,72.0 158.4,70.0 180.8,67.0 203.2,62.2 225.6,64.1 248.0,63.1 270.4,65.1 292.8,61.2 315.2,67.0 337.6,64.1 360.0,67.1 382.4,66.1 404.8,65.0 427.2,66.1 449.6,67.0 472.0,66.1 494.4,66.1 516.8,65.1 539.2,65.1 561.6,64.2 584.0,66.2 606.4,69.0 628.8,71.0 651.2,70.0 673.6,69.1 696.0,71.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.43, last 5.43</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 300244.00, last 300244.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 28

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,76.6 89.0,70.4 110.7,67.7 132.4,66.8 154.1,59.6 175.7,58.8 197.4,54.3 219.1,73.1 240.8,73.0 262.5,74.0 284.1,70.4 305.8,73.1 327.5,76.6 349.2,74.8 370.8,73.1 392.5,76.5 414.2,74.8 435.9,75.7 457.5,74.8 479.2,74.8 500.9,74.0 522.6,74.8 544.3,70.4 565.9,72.1 587.6,73.1 609.3,76.6 631.0,79.3 652.6,76.6 674.3,80.2 696.0,107.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.24, last 3.37</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.2 67.4,24.2 89.0,24.2 110.7,24.2 132.4,24.2 154.1,24.1 175.7,24.1 197.4,24.1 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.0 370.8,24.0 392.5,24.0 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 318708.00, last 318708.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 29

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,108.1 67.4,101.9 89.0,80.8 110.7,57.7 132.4,73.9 154.1,78.6 175.7,24.0 197.4,54.1 219.1,105.2 240.8,74.5 262.5,70.6 284.1,37.3 305.8,56.6 327.5,72.3 349.2,74.9 370.8,120.6 392.5,121.3 414.2,120.6 435.9,120.9 457.5,121.3 479.2,119.1 500.9,35.8 522.6,56.3 544.3,58.8 565.9,110.3 587.6,53.7 609.3,121.3 631.0,100.9 652.6,101.9 674.3,123.9 696.0,134.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 22.56, last 3.74</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.5 67.4,24.4 89.0,24.4 110.7,24.4 132.4,24.4 154.1,24.3 175.7,24.3 197.4,24.2 219.1,24.2 240.8,24.2 262.5,24.2 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.0 392.5,24.0 414.2,24.1 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 330788.00, last 330740.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 30

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,45.8 67.4,64.8 89.0,31.7 110.7,48.9 132.4,31.9 154.1,72.7 175.7,100.6 197.4,32.5 219.1,52.1 240.8,61.0 262.5,79.4 284.1,85.4 305.8,28.3 327.5,74.7 349.2,82.0 370.8,87.2 392.5,64.2 414.2,58.9 435.9,122.4 457.5,121.4 479.2,123.1 500.9,123.2 522.6,122.4 544.3,121.4 565.9,117.9 587.6,56.8 609.3,24.0 631.0,70.6 652.6,55.6 674.3,41.9 696.0,103.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 23.33, last 9.23</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.2 67.4,24.2 89.0,24.1 110.7,24.1 132.4,24.1 154.1,24.1 175.7,24.1 197.4,24.1 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.1 392.5,24.1 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 350420.00, last 350420.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 31

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,29.5 67.4,124.6 89.0,123.6 110.7,124.3 132.4,123.6 154.1,123.6 175.7,106.2 197.4,61.5 219.1,61.5 240.8,33.3 262.5,70.3 284.1,73.1 305.8,34.7 327.5,24.0 349.2,41.4 370.8,61.6 392.5,44.4 414.2,73.1 435.9,114.2 457.5,91.9 479.2,75.5 500.9,82.8 522.6,85.3 544.3,57.0 565.9,59.2 587.6,81.8 609.3,78.1 631.0,93.1 652.6,93.3 674.3,99.9 696.0,133.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 23.64, last 4.12</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.2 67.4,24.2 89.0,24.2 110.7,24.2 132.4,24.2 154.1,24.2 175.7,24.2 197.4,24.1 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.0 370.8,24.0 392.5,24.0 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 366984.00, last 366984.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 32

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,73.8 91.2,81.4 113.6,80.7 136.0,77.6 158.4,76.0 180.8,76.0 203.2,75.3 225.6,74.5 248.0,79.2 270.4,79.1 292.8,77.6 315.2,76.8 337.6,83.8 360.0,80.7 382.4,79.8 404.8,76.8 427.2,78.4 449.6,80.8 472.0,81.5 494.4,75.2 516.8,81.5 539.2,79.1 561.6,78.6 584.0,79.9 606.4,82.3 628.8,79.1 651.2,81.8 673.6,73.7 696.0,80.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.61, last 6.06</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 381568.00, last 381556.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 33

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,74.1 91.2,80.1 113.6,81.6 136.0,74.0 158.4,74.8 180.8,77.1 203.2,77.1 225.6,77.8 248.0,73.3 270.4,76.3 292.8,77.1 315.2,77.9 337.6,80.1 360.0,73.3 382.4,74.8 404.8,79.4 427.2,76.3 449.6,66.4 472.0,45.3 494.4,55.2 516.8,71.0 539.2,71.8 561.6,68.0 584.0,65.7 606.4,71.1 628.8,73.3 651.2,75.6 673.6,76.3 696.0,78.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.86, last 6.36</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 398900.00, last 398872.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 34

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,51.8 89.0,50.0 110.7,59.0 132.4,56.4 154.1,52.7 175.7,60.9 197.4,57.2 219.1,59.9 240.8,59.9 262.5,53.7 284.1,57.3 305.8,63.6 327.5,55.4 349.2,59.9 370.8,62.5 392.5,55.4 414.2,59.8 435.9,53.6 457.5,58.0 479.2,61.7 500.9,56.4 522.6,59.0 544.3,59.8 565.9,60.0 587.6,59.1 609.3,58.1 631.0,61.7 652.6,62.5 674.3,59.9 696.0,95.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.17, last 4.18</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.3 67.4,24.2 89.0,24.2 110.7,24.2 132.4,24.2 154.1,24.2 175.7,24.2 197.4,24.2 219.1,24.2 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.1 370.8,24.1 392.5,24.1 414.2,24.1 435.9,24.1 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 411124.00, last 411104.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 35

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,63.5 67.4,86.6 89.0,85.3 110.7,75.8 132.4,24.0 154.1,74.8 175.7,75.4 197.4,74.8 219.1,87.8 240.8,89.0 262.5,90.3 284.1,94.0 305.8,95.2 327.5,93.4 349.2,92.7 370.8,94.6 392.5,93.4 414.2,81.5 435.9,86.0 457.5,93.9 479.2,95.3 500.9,94.6 522.6,94.6 544.3,92.8 565.9,90.3 587.6,93.4 609.3,90.9 631.0,92.8 652.6,95.9 674.3,93.4 696.0,105.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 13.29, last 5.06</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.1 67.4,24.0 89.0,24.0 110.7,69.4 132.4,62.7 154.1,62.5 175.7,62.4 197.4,62.3 219.1,62.2 240.8,62.1 262.5,62.1 284.1,62.0 305.8,61.9 327.5,61.9 349.2,61.8 370.8,61.7 392.5,61.7 414.2,61.6 435.9,61.6 457.5,61.5 479.2,61.4 500.9,61.4 522.6,61.3 544.3,61.3 565.9,61.2 587.6,61.1 609.3,61.1 631.0,61.0 652.6,61.0 674.3,60.9 696.0,60.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 424104.00, last 305656.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 36

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,64.7 91.2,67.4 113.6,54.9 136.0,53.2 158.4,60.3 180.8,59.3 203.2,57.7 225.6,58.5 248.0,57.6 270.4,59.3 292.8,55.9 315.2,52.3 337.6,58.6 360.0,52.3 382.4,62.9 404.8,56.7 427.2,56.7 449.6,55.9 472.0,63.8 494.4,64.7 516.8,61.2 539.2,58.5 561.6,55.8 584.0,62.1 606.4,62.0 628.8,63.8 651.2,64.7 673.6,65.6 696.0,62.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.30, last 6.56</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,25.4 68.8,25.2 91.2,25.2 113.6,25.1 136.0,25.1 158.4,25.0 180.8,24.9 203.2,24.8 225.6,24.7 248.0,24.7 270.4,24.6 292.8,24.5 315.2,24.4 337.6,24.4 360.0,24.3 382.4,24.2 404.8,24.2 427.2,24.2 449.6,24.1 472.0,24.1 494.4,24.1 516.8,24.1 539.2,24.1 561.6,24.1 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 326924.00, last 326924.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 37

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,84.2 91.2,83.5 113.6,79.9 136.0,76.2 158.4,79.1 180.8,78.5 203.2,77.7 225.6,77.0 248.0,74.1 270.4,74.1 292.8,78.4 315.2,77.7 337.6,80.6 360.0,80.6 382.4,81.3 404.8,77.0 427.2,66.1 449.6,74.8 472.0,81.3 494.4,80.6 516.8,79.9 539.2,80.6 561.6,81.0 584.0,82.1 606.4,84.2 628.8,82.8 651.2,82.1 673.6,86.5 696.0,82.8" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.37, last 6.31</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 342592.00, last 342592.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 38

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,61.6 91.2,60.7 113.6,62.6 136.0,60.8 158.4,56.3 180.8,58.9 203.2,53.5 225.6,57.0 248.0,52.5 270.4,50.6 292.8,51.6 315.2,51.5 337.6,55.3 360.0,57.1 382.4,54.3 404.8,54.3 427.2,56.2 449.6,53.3 472.0,55.2 494.4,56.1 516.8,52.5 539.2,54.3 561.6,52.5 584.0,54.3 606.4,57.0 628.8,55.2 651.2,58.9 673.6,52.5 696.0,61.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.99, last 6.43</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.3 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 355000.00, last 354940.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 39

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,60.7 91.2,57.0 113.6,52.3 136.0,56.0 158.4,55.1 180.8,51.4 203.2,52.3 225.6,46.7 248.0,46.6 270.4,44.7 292.8,44.6 315.2,39.1 337.6,52.3 360.0,53.2 382.4,51.3 404.8,53.2 427.2,50.4 449.6,45.6 472.0,51.4 494.4,54.3 516.8,36.2 539.2,45.7 561.6,48.5 584.0,48.5 606.4,53.1 628.8,53.2 651.2,50.4 673.6,57.0 696.0,54.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.74, last 6.74</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.2 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 369920.00, last 369920.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 40

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,58.9 91.2,53.6 113.6,60.8 136.0,56.3 158.4,52.8 180.8,55.4 203.2,56.3 225.6,55.4 248.0,54.5 270.4,54.5 292.8,53.6 315.2,57.2 337.6,54.5 360.0,52.7 382.4,54.5 404.8,57.4 427.2,54.6 449.6,53.7 472.0,57.2 494.4,56.4 516.8,57.2 539.2,54.5 561.6,53.7 584.0,52.7 606.4,52.7 628.8,57.2 651.2,57.2 673.6,57.3 696.0,59.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.18, last 6.68</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.2 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 384512.00, last 384456.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 41

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,60.7 91.2,59.8 113.6,60.7 136.0,60.6 158.4,51.9 180.8,53.7 203.2,54.5 225.6,54.5 248.0,52.8 270.4,53.7 292.8,53.7 315.2,55.5 337.6,57.1 360.0,58.1 382.4,60.7 404.8,59.8 427.2,58.9 449.6,57.1 472.0,60.7 494.4,60.6 516.8,58.9 539.2,57.2 561.6,54.6 584.0,53.8 606.4,57.1 628.8,60.7 651.2,59.9 673.6,58.1 696.0,64.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.42, last 6.55</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 399904.00, last 399904.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 42

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,48.6 91.2,53.2 113.6,52.3 136.0,49.8 158.4,42.0 180.8,47.6 203.2,45.7 225.6,42.8 248.0,43.9 270.4,42.9 292.8,43.9 315.2,43.8 337.6,48.5 360.0,47.6 382.4,42.8 404.8,46.7 427.2,41.0 449.6,46.6 472.0,45.7 494.4,45.7 516.8,46.7 539.2,47.7 561.6,48.4 584.0,52.4 606.4,45.7 628.8,55.1 651.2,48.6 673.6,56.0 696.0,53.3" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 8.74, last 6.80</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.3 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.1 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 414548.00, last 414524.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 43

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,68.3 91.2,69.0 113.6,69.0 136.0,70.5 158.4,65.9 180.8,64.4 203.2,64.4 225.6,63.6 248.0,65.2 270.4,60.1 292.8,60.4 315.2,64.4 337.6,64.3 360.0,65.9 382.4,63.8 404.8,63.6 427.2,67.5 449.6,63.6 472.0,63.6 494.4,62.8 516.8,62.9 539.2,61.2 561.6,62.1 584.0,61.3 606.4,62.7 628.8,66.7 651.2,68.3 673.6,66.0 696.0,68.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.61, last 7.06</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 430616.00, last 430616.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 44

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,67.8 91.2,64.6 113.6,64.7 136.0,63.8 158.4,63.8 180.8,63.0 203.2,60.7 225.6,59.8 248.0,61.5 270.4,59.1 292.8,59.0 315.2,60.7 337.6,59.8 360.0,60.7 382.4,60.6 404.8,60.7 427.2,63.8 449.6,59.9 472.0,59.1 494.4,59.9 516.8,59.0 539.2,60.6 561.6,57.4 584.0,55.9 606.4,59.1 628.8,60.6 651.2,60.7 673.6,61.5 696.0,61.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.37, last 7.43</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.2 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.1 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 446944.00, last 446944.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 45

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,77.3 91.2,73.8 113.6,77.3 136.0,78.6 158.4,74.5 180.8,71.0 203.2,72.4 225.6,75.1 248.0,73.8 270.4,74.4 292.8,71.7 315.2,74.5 337.6,77.3 360.0,75.9 382.4,74.4 404.8,72.4 427.2,75.9 449.6,73.7 472.0,73.1 494.4,76.5 516.8,73.8 539.2,73.8 561.6,72.4 584.0,72.4 606.4,69.7 628.8,75.9 651.2,74.5 673.6,77.3 696.0,76.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.93, last 7.18</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 465676.00, last 465668.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 46

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,81.2 91.2,78.7 113.6,81.2 136.0,81.9 158.4,77.5 180.8,76.2 203.2,76.2 225.6,78.7 248.0,74.3 270.4,77.5 292.8,78.1 315.2,78.8 337.6,78.1 360.0,81.2 382.4,81.3 404.8,76.1 427.2,81.9 449.6,78.1 472.0,83.2 494.4,79.4 516.8,80.0 539.2,79.0 561.6,75.6 584.0,78.7 606.4,80.6 628.8,76.8 651.2,81.8 673.6,83.7 696.0,80.6" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 13.11, last 7.49</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 481368.00, last 481368.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 47

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.0 67.4,70.7 89.0,74.1 110.7,76.1 132.4,70.6 154.1,70.6 175.7,70.7 197.4,68.6 219.1,67.9 240.8,67.2 262.5,70.6 284.1,72.7 305.8,68.5 327.5,71.4 349.2,71.3 370.8,70.0 392.5,74.8 414.2,67.8 435.9,67.2 457.5,59.0 479.2,71.4 500.9,66.5 522.6,74.1 544.3,70.6 565.9,68.6 587.6,69.9 609.3,74.1 631.0,81.6 652.6,75.5 674.3,72.7 696.0,98.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.97, last 5.25</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="45.7,24.1 67.4,24.1 89.0,24.1 110.7,24.1 132.4,24.1 154.1,24.1 175.7,24.1 197.4,24.1 219.1,24.1 240.8,24.1 262.5,24.1 284.1,24.1 305.8,24.1 327.5,24.1 349.2,24.0 370.8,24.0 392.5,24.0 414.2,24.0 435.9,24.0 457.5,24.0 479.2,24.0 500.9,24.0 522.6,24.0 544.3,24.0 565.9,24.0 587.6,24.0 609.3,24.0 631.0,24.0 652.6,24.0 674.3,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 496204.00, last 496204.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">31s</text></svg>

### stream-limit-10-nodes iteration 48

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,58.1 91.2,60.2 113.6,55.7 136.0,64.0 158.4,59.4 180.8,59.4 203.2,55.0 225.6,52.6 248.0,57.2 270.4,57.2 292.8,58.7 315.2,52.7 337.6,60.3 360.0,53.5 382.4,55.0 404.8,57.3 427.2,55.7 449.6,40.6 472.0,62.5 494.4,57.3 516.8,62.5 539.2,54.2 561.6,61.7 584.0,55.7 606.4,64.7 628.8,62.5 651.2,61.0 673.6,64.0 696.0,66.3" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.93, last 7.42</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 509340.00, last 509308.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 49

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,56.9 91.2,56.1 113.6,58.6 136.0,53.6 158.4,51.2 180.8,44.6 203.2,45.4 225.6,37.3 248.0,35.5 270.4,35.5 292.8,29.7 315.2,49.5 337.6,41.3 360.0,48.6 382.4,52.0 404.8,51.2 427.2,44.5 449.6,52.0 472.0,50.3 494.4,48.8 516.8,42.1 539.2,48.8 561.6,44.4 584.0,49.5 606.4,51.1 628.8,44.5 651.2,52.8 673.6,55.3 696.0,56.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 9.98, last 7.49</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.2 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 522100.00, last 522084.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 50

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,69.0 91.2,67.0 113.6,68.2 136.0,66.2 158.4,68.2 180.8,64.8 203.2,65.4 225.6,69.0 248.0,64.7 270.4,67.6 292.8,66.2 315.2,64.1 337.6,68.3 360.0,65.5 382.4,69.7 404.8,66.1 427.2,69.0 449.6,66.9 472.0,67.6 494.4,68.2 516.8,65.5 539.2,64.1 561.6,66.2 584.0,64.1 606.4,69.0 628.8,68.3 651.2,67.6 673.6,65.5 696.0,71.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.74, last 7.55</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.0 225.6,24.0 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 542144.00, last 542144.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 51

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,61.9 91.2,64.8 113.6,62.7 136.0,61.9 158.4,58.2 180.8,62.7 203.2,56.0 225.6,55.9 248.0,55.9 270.4,57.4 292.8,57.4 315.2,57.4 337.6,57.4 360.0,59.0 382.4,61.2 404.8,61.9 427.2,58.2 449.6,62.0 472.0,61.1 494.4,62.7 516.8,62.6 539.2,58.3 561.6,56.7 584.0,58.2 606.4,61.1 628.8,62.7 651.2,62.6 673.6,61.9 696.0,63.4" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.12, last 7.80</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 555348.00, last 555348.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 52

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,71.4 91.2,72.8 113.6,70.8 136.0,70.0 158.4,69.4 180.8,65.2 203.2,67.3 225.6,68.0 248.0,65.9 270.4,64.6 292.8,65.3 315.2,66.0 337.6,70.1 360.0,68.0 382.4,67.3 404.8,68.0 427.2,66.0 449.6,68.8 472.0,66.6 494.4,68.7 516.8,64.6 539.2,67.4 561.6,67.4 584.0,68.7 606.4,72.1 628.8,72.1 651.2,72.1 673.6,72.8 696.0,73.5" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.99, last 7.49</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.0 225.6,24.0 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 573076.00, last 573076.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 53

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,61.4 91.2,60.6 113.6,62.1 136.0,60.7 158.4,62.1 180.8,59.8 203.2,55.4 225.6,51.7 248.0,57.0 270.4,58.5 292.8,57.0 315.2,57.7 337.6,59.2 360.0,56.2 382.4,58.4 404.8,56.9 427.2,56.1 449.6,57.7 472.0,57.7 494.4,58.4 516.8,57.7 539.2,59.9 561.6,54.7 584.0,54.6 606.4,59.2 628.8,57.6 651.2,59.1 673.6,57.6 696.0,62.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.06, last 7.86</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.1 360.0,24.1 382.4,24.1 404.8,24.1 427.2,24.1 449.6,24.1 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 586032.00, last 586028.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 54

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,55.8 91.2,56.0 113.6,55.1 136.0,53.5 158.4,54.3 180.8,49.4 203.2,48.7 225.6,49.5 248.0,48.0 270.4,48.6 292.8,47.1 315.2,50.2 337.6,48.0 360.0,47.9 382.4,49.4 404.8,51.8 427.2,49.5 449.6,49.5 472.0,51.9 494.4,50.4 516.8,48.7 539.2,46.4 561.6,47.1 584.0,44.7 606.4,53.5 628.8,51.1 651.2,51.0 673.6,48.7 696.0,55.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 10.37, last 7.93</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.3 68.8,24.2 91.2,24.2 113.6,24.2 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 599452.00, last 599412.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 55

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,64.5 91.2,62.5 113.6,63.1 136.0,63.8 158.4,63.1 180.8,63.1 203.2,61.0 225.6,62.4 248.0,63.2 270.4,61.7 292.8,65.2 315.2,64.5 337.6,64.5 360.0,61.0 382.4,62.4 404.8,61.8 427.2,63.9 449.6,62.4 472.0,58.3 494.4,63.8 516.8,61.7 539.2,61.0 561.6,61.8 584.0,63.1 606.4,61.8 628.8,64.6 651.2,63.8 673.6,65.3 696.0,65.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.80, last 8.11</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 617280.00, last 617264.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 56

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,59.9 91.2,55.5 113.6,60.6 136.0,55.5 158.4,57.7 180.8,59.2 203.2,52.6 225.6,58.5 248.0,56.2 270.4,51.1 292.8,53.3 315.2,53.4 337.6,55.5 360.0,54.1 382.4,54.0 404.8,58.5 427.2,58.4 449.6,55.5 472.0,60.7 494.4,57.0 516.8,57.7 539.2,54.0 561.6,55.5 584.0,54.0 606.4,57.8 628.8,57.7 651.2,60.0 673.6,59.1 696.0,60.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.24, last 8.11</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 631796.00, last 631728.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 57

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,72.5 91.2,73.2 113.6,73.1 136.0,73.2 158.4,70.1 180.8,72.5 203.2,68.9 225.6,70.8 248.0,69.4 270.4,67.6 292.8,71.3 315.2,65.8 337.6,71.9 360.0,67.6 382.4,71.3 404.8,73.9 427.2,71.9 449.6,70.7 472.0,70.7 494.4,70.7 516.8,70.1 539.2,70.6 561.6,70.7 584.0,71.9 606.4,71.9 628.8,73.2 651.2,76.3 673.6,74.4 696.0,75.7" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 13.23, last 8.05</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.0 203.2,24.0 225.6,24.0 248.0,24.0 270.4,24.0 292.8,24.0 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 650228.00, last 650228.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 58

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,54.9 91.2,54.1 113.6,57.1 136.0,55.0 158.4,54.1 180.8,54.2 203.2,48.2 225.6,52.0 248.0,52.7 270.4,52.6 292.8,51.9 315.2,51.2 337.6,55.7 360.0,51.3 382.4,53.4 404.8,54.2 427.2,53.3 449.6,53.5 472.0,53.3 494.4,56.4 516.8,51.9 539.2,52.7 561.6,49.0 584.0,51.3 606.4,53.4 628.8,56.3 651.2,53.5 673.6,52.7 696.0,57.9" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.25, last 8.36</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.1 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.1 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 662724.00, last 662724.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 59

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,57.2 91.2,60.0 113.6,62.8 136.0,61.3 158.4,57.8 180.8,58.6 203.2,53.6 225.6,54.9 248.0,56.0 270.4,55.7 292.8,52.8 315.2,55.7 337.6,58.5 360.0,56.4 382.4,54.3 404.8,55.0 427.2,55.7 449.6,53.5 472.0,55.0 494.4,54.2 516.8,59.3 539.2,54.3 561.6,53.5 584.0,54.3 606.4,54.2 628.8,58.5 651.2,57.1 673.6,57.9 696.0,59.2" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.67, last 8.55</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.2 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 676820.00, last 676776.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

### stream-limit-10-nodes iteration 60

CPU percent:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="CPU % over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.0 68.8,55.2 91.2,56.5 113.6,58.7 136.0,53.7 158.4,54.4 180.8,53.7 203.2,53.7 225.6,53.8 248.0,53.7 270.4,55.9 292.8,55.1 315.2,52.3 337.6,57.2 360.0,55.8 382.4,54.5 404.8,54.5 427.2,55.9 449.6,51.6 472.0,53.7 494.4,57.9 516.8,55.1 539.2,53.0 561.6,53.8 584.0,50.2 606.4,56.6 628.8,58.7 651.2,57.9 673.6,58.7 696.0,60.1" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">CPU %, max 11.68, last 8.49</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>

RSS memory KiB:

<svg width="720" height="180" viewBox="0 0 720 180" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="RSS KiB over time"><rect width="720" height="180" fill="#fff"/><line x1="24" y1="156" x2="696" y2="156" stroke="#bbb"/><line x1="24" y1="24" x2="24" y2="156" stroke="#bbb"/><polyline points="46.4,24.2 68.8,24.1 91.2,24.1 113.6,24.1 136.0,24.1 158.4,24.1 180.8,24.1 203.2,24.1 225.6,24.1 248.0,24.1 270.4,24.1 292.8,24.1 315.2,24.0 337.6,24.0 360.0,24.0 382.4,24.0 404.8,24.0 427.2,24.0 449.6,24.0 472.0,24.0 494.4,24.0 516.8,24.0 539.2,24.0 561.6,24.0 584.0,24.0 606.4,24.0 628.8,24.0 651.2,24.0 673.6,24.0 696.0,24.0" fill="none" stroke="#2563eb" stroke-width="2"/><text x="24" y="16" font-family="monospace" font-size="12">RSS KiB, max 691708.00, last 691708.00</text><text x="24" y="176" font-family="monospace" font-size="11">0s</text><text x="650" y="176" font-family="monospace" font-size="11">30s</text></svg>


## Per-Node Counters

### stream-limit-10-nodes iteration 1

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 2

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 3

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 4

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9203211 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 840143 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 804628 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 5

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 6

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 7

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9202185 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1050875 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 844080 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 8

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9209110 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 844080 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 9

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 10

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 11

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 12

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 13

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 14

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 15

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 16

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 17

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 18

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 19

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 20

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 21

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 22

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 23

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 24

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 25

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 26

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 27

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 28

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 29

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 30

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 31

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 32

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 33

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 34

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 35

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 36

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 37

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 38

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 39

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 40

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 41

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 42

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 43

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 44

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 45

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 46

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 47

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 48

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 49

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 50

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 51

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 52

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 53

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 54

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 55

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 56

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 57

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 58

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 59

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |

### stream-limit-10-nodes iteration 60

| Peer | Opened streams | Inbound streams | Sent bytes | Received bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 90 | 0 | 9210460 | 0 |
| 2 | 0 | 7 | 0 | 655570 |
| 3 | 0 | 6 | 0 | 537470 |
| 4 | 0 | 11 | 0 | 1057800 |
| 5 | 0 | 13 | 0 | 1072520 |
| 6 | 0 | 8 | 0 | 845430 |
| 7 | 0 | 11 | 0 | 1302430 |
| 8 | 0 | 16 | 0 | 1551000 |
| 9 | 0 | 5 | 0 | 806590 |
| 10 | 0 | 13 | 0 | 1381650 |


## Profiles

- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.
- `stream-limit-10-nodes`: source peer `1`, nodes `10`, latency `50..=500ms`, target stream rate `32..=512 kbps`, stream live range `1..=5s`, open timeout `2500ms`, seed `101001`.

## Failures

No stream open, write, or read failures were observed in this run.

## Reproduce

```bash
RUST_LOG=error CARGO_BUILD_JOBS=8 cargo run --example stream_limit_benchmark -- --profile stream-limit-10-nodes --min-run-seconds 1800 --profiles benchmarks/stream_limit_profiles.yaml --report docs/stream_limit_benchmark_report.md
```
