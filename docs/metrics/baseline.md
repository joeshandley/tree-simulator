# Metrics baseline

Written by `cargo run --release --example metrics -- --write`. Definitions: plan 0002 (D11). When a definition changes, add a dated note here; older numbers stop being comparable.

- **Date:** 2026-11-14
- **Commit:** `feature/cheap-collision-wins` (plan 0004; tree metrics identical to the previous baseline, timings only changed)
- **Machine:** Apple M5 (macos aarch64)
- **Seeds:** 1-20

| Metric | Mean | Min | Max |
|---|---|---|---|
| Years to finish | 5.0 | 2.6 | 8.4 |
| Sections | 1440 | 370 | 3649 |
| Tips | 56 | 15 | 149 |
| Height (m) | 1.90 | 0.92 | 3.26 |
| Crown width (m) | 1.03 | 0.59 | 1.41 |
| Fork angle (°) | 53.6 | 51.9 | 54.9 |
| Leonardo ratio | 0.962 | 0.962 | 0.962 |
| Time per step (µs) | 23 | 2 | 92 |
| Slowest step (µs) | 126 | 21 | 560 |
