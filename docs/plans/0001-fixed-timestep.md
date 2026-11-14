# Plan 0001: Fixed timestep and seed

- **Roadmap step:** 1
- **Date:** 14 Nov 2026
- **Status:** built (14 Nov 2026); D4–D6 accepted by the author 14 Nov 2026. Quick inline review pass, no blockers.
- **Sources:** `docs/roadmap.md`; code audit of 13 Nov 2026 (`~/claude-workspace/tree-simulator/code-audit-2026-11-13.md`); decisions with the author on 14 Nov 2026

## Problem

`main.rs` passes `frame_time × speed × SPEED_SCALE` straight into `TreeSimulation::update`. So:
- tree shape depends on frame rate and speed (audit: dt 1/60 s gave 1,347 sections, 1/59 s gave 814, same start state);
- a stall produces one huge step (audit: 2 s stall bent tips ~190°);
- each tip finishes at most one internode per frame, capping useful speed (audit: 20× ran only 7.6× faster);
- randomness comes from the global quad-rand generator, so a tree can't be replayed on demand.

## Decisions

| ID | Decision | Easy/hard to undo | Source |
|---|---|---|---|
| D1 | **Speed means simulated seconds per real second.** Real frame time feeds an accumulator, drained in fixed steps. Same speed on any display; higher refresh only looks smoother. | Easy | Author, 14 Nov 2026 |
| D2 | **When the per-frame step cap is hit, the simulation slows down** (leftover time is dropped), rather than catching up later. | Easy | Author, 14 Nov 2026 |
| D3 | **`TreeSimulation` owns its random generator** (`quad_rand::RandGenerator`, already a dependency via macroquad), seeded at creation and reset. Keeps parallel tests deterministic. | Easy | Author, 14 Nov 2026 |
| D4 | **Fixed step `SIM_DT` = 1/60 simulated second.** At current speeds that is ~14 steps per internode. Changes every tree's shape once; real units (step 5) will re-tune anyway. | Easy until step 5 | Proposed |
| D5 | **Real frame time is clamped to 0.1 s before it enters the accumulator**, and at most `MAX_STEPS_PER_FRAME` = 64 steps run per frame. | Easy | Proposed |
| D6 | **Seed UI:** seed shown on screen; `R` grows a new tree with a fresh seed (from system time); `Shift+R` regrows the same seed. | Easy | Proposed |

## Planned changes

1. **Owned RNG** (`src/simulation.rs`): add `rng: RandGenerator` and `seed: u64` to `TreeSimulation`; `new(start, thickness, seed)` and `reset(start, thickness, seed)`; replace the three `rand::gen_range` calls (`jittered_length`, fork roll, leader-side roll) with `self.rng`. `jittered_length` takes the generator as an argument. ~40 lines.
2. **Fixed-step driver** (`src/simulation.rs`): `SIM_DT` constant; `TreeSimulation::step()` runs one `update(SIM_DT)`. Keep `update(dt)` private or test-only so callers can't reintroduce variable steps. ~20 lines.
3. **Accumulator in the game loop** (`src/main.rs`): clamp frame time (D5), add `frame_time × sim_speed × SPEED_SCALE` to the accumulator, run `step()` while accumulator ≥ `SIM_DT` and under the cap, then drop any leftover above one step (D2). Leaf timers advance by the simulated time actually stepped, so pause and slow-down apply to them too. ~30 lines.
4. **Seed UI** (`src/main.rs`): D6. ~15 lines.
5. **Tests** (`src/simulation.rs`), written first:
   - same seed, same number of steps → identical sections (positions, angles, count);
   - different seeds → different trees (smoke check);
   - result is independent of how steps are grouped into frames (e.g. 600 steps as 600×1 vs 10×60) — tests the accumulator logic once it's extracted to a pure function;
   - accumulator: clamp and cap behaviour (table-driven: normal frame, stall, cap hit, paused).
   ~80 lines.
6. **Docs:** `docs/simulation-model.md` (timestep section), roadmap (move step 1 to Done; mark the three timestep bugs fixed), `CLAUDE.md` invariants (fixed step; RNG on the simulation).

Rough size: ~185 lines across 4–5 files. One branch, one commit per numbered change.

## Checklist

- **Invariants:** arena and freeze untouched. New invariant: the simulation only advances in `SIM_DT` steps, and all randomness comes from `TreeSimulation::rng`. Add both to `CLAUDE.md`.
- **Performance:** at the top speed (5 × 4 = 20 sim-s per real second, 60 Hz) that's 20 steps per frame. At ~2000 sections (~1.6 ms per update in release, per the audit) that's ~32 ms per frame: large trees will slow down at top speed, by design (D2). Step 2 measures this properly. In a debug build it will be far slower; run with `--release`.
- **Determinism:** the point of this step. Covered by tests 5.1 and 5.3.
- **Tests:** RNG plumbing and the accumulator are pure and testable; the macroquad loop is not, so the accumulator logic moves into a small pure function.
- **Visual check:** same seed regrows the same tree (`Shift+R`); speed slider gives the same wall-clock growth at 60 Hz and 120 Hz (if a 120 Hz display is available); dragging the window no longer kinks branches.
- **Tuning:** shapes change once due to D4. `SPEED_SCALE` stays. No other parameters move.
- **Roadmap knock-on:** unblocks step 2 (needs seeded, fixed-step runs). Fixes audit bugs 1–3 and the seed issue.

