# Plan 0002: Measurement harness

- **Roadmap step:** 2
- **Date:** 14 Nov 2026
- **Status:** built (14 Nov 2026)
- **Sources:** `docs/roadmap.md`; code audit of 13 Nov 2026; decisions with the author on 14 Nov 2026

## Problem

There is no way to measure a change. Speed is judged by eye, and the audit's timings (1.6 ms per update at ~2000 sections in release, 69 ms in debug) came from a throwaway crate. Every later roadmap step needs a before and after, and the update loop has no test of the arena's invariants.

## Decisions

| ID | Decision | Easy/hard to undo | Source |
|---|---|---|---|
| D7 | **Timing uses criterion benchmarks** (`cargo bench`). | Easy | Author, 14 Nov 2026 |
| D8 | **Tree metrics live in the library** (`src/metrics.rs`), and a small example (`cargo run --release --example metrics`) grows seeded trees and prints them. Criterion can't report non-timing numbers, so it needs this second piece. | Easy | Assumption, 14 Nov 2026 (flagged to the author) |
| D9 | **The baseline is committed** to `docs/metrics/baseline.md`, written by the example with `--write`, so numbers travel between machines. Can be deleted later. | Easy | Author, 14 Nov 2026 |
| D10 | **No comparison against real trees yet.** Metrics are tracked between steps only; reference ranges come after real units (step 5). | Easy | Author, 14 Nov 2026 |
| D11 | **Metric definitions** (below). When a definition changes, older baselines stop being comparable; that is accepted, the earliest useful data point just moves forward. A change gets a dated note in the baseline file. | Easy | Author, 14 Nov 2026 |

## Metrics (D11)

Per tree, grown from a seed until `is_finished()` or 20,000 steps:
- **steps to finish** (or "unfinished" at the cap)
- **sections**, and **tips** (fully grown sections with no children)
- **height**: base to highest end point; **crown width**: horizontal span of all end points
- **fork angle**: mean angle between the two children at forks
- **Leonardo ratio**: mean of parent cross-section ÷ children's combined cross-section at forks (thickness², so 1.0 means area is conserved)
- **time per step**: mean wall-clock time per `step()` over the run (release build only; rough, criterion is the precise figure)

The example runs seeds 1–20 and prints mean, min and max per metric.

## Benchmarks (D7)

`benches/step.rs`: time to grow seeds 1–3 from start to finish. (Changed while building: per-step timing at a given size mostly measured nearly finished trees, since a tree reaching a size is usually almost done. The metrics example reports mean and slowest step per run instead.)

## Planned changes

1. **Metrics module** (`src/metrics.rs`), test-first: table tests on hand-built trees (a straight trunk, a single fork with known angle and thicknesses). ~120 lines.
2. **Arena invariant test** (`tests/invariants.rs`): for several seeds, check every `parent_index` and `child_indices` entry is in range and consistent, children start at the parent's end point, and frozen sections don't change over further steps. Short runs in the normal suite; a run-to-finish version marked `#[ignore]`, run with `cargo test --release -- --ignored`. ~80 lines.
3. **Example** (`examples/metrics.rs`) with `--seeds N` and `--write`. ~80 lines.
4. **Benchmark** (`benches/step.rs`) and criterion as a dev-dependency (a version at least two weeks old; downloads go to the project-local toolchain cache). ~40 lines.
5. **First baseline** written and committed; docs: `CLAUDE.md` commands, roadmap.

Rough size: ~330 lines across 7–8 files. One commit per numbered change.

## Checklist

- **Invariants:** no simulation behaviour changes. The new test codifies the arena invariants from ADR-0001 and ADR-0002.
- **Performance:** the harness measures it; the harness itself doesn't run in the app.
- **Determinism:** relies on step 1's seed and fixed step. Same seed → same metrics except time per step.
- **Tests:** metrics are pure functions of `growth_sections`, so table tests on hand-built trees.
- **Visual check:** none; this step has no on-screen change.
- **Tuning:** none.
- **Roadmap knock-on:** steps 3 and 4 use the benchmark to decide whether the spatial grid is still needed.

## Quick review (14 Nov 2026)

- Run-to-finish in a debug build may be very slow (audit: 69 ms per update at ~2000 sections), hence `#[ignore]` for the full run.
- `TreeSimulation` can't be cloned (quad-rand's generator hides its state), so benchmarks re-grow trees instead of cloning one.
- Time per step in the example depends on the machine, so the baseline file records the machine and commit.
