# Roadmap

Last updated 14 Nov 2026. Sources: the author's build notes (imported 13 Nov 2026), a roadmap reassessment agreed with the author on 14 Nov 2026, and a code audit on 13 Nov 2026.

## Goal

**The most realistic tree simulator: realistic in mechanism, not just in appearance.** Growth should emerge from biology (light, resources, buds, wood mechanics), not from hand-tuned rules that imitate what trees look like. Everything works well in 2D first; 3D is out of scope for now.

**When resuming this project, restate the priority list back to the author so they can reassess whether any item is redundant or superseded.**

## Done

- Vector maths with tests (`ee4a003`, 25 Jul 2026)
- Collision avoidance, canopy shading, leaves, time controls (25 Jul 2026)
- Internode subdivision with freeze (ADR-0002, `51eda3c`, 31 Jul 2026)
- Sympodial apical dominance (ADR-0003, `3ab7b03`, 3 Aug 2026)
- Project-local toolchain and docs (13 Nov 2026)
- Simulation runs 4× faster at every speed setting (`742443a`, 13 Nov 2026)

## Priority list

Today's growth rules are phenomenological (tuned to look right). Items that only tuned those rules were folded into the resource model (step 9), which replaces them.

1. **Fixed timestep and seed.** Step the simulation at a fixed dt, with N sub-steps per frame from the speed slider, and cap real frame time. Seedable RNG shown on screen. Fixes the stall, speed-cap and frame-rate bugs below.
2. **Measurement harness.** Headless run of a seeded tree reporting time per update (release build) and tree metrics: height, crown width, section count, branch angles, diameter ratios at forks. Plus an arena invariant test. Metrics are how each later step shows it made the tree more real.
3. **Cheap collision wins.** Cache ancestor chains; compare squared distance instead of taking `sqrt`.
4. **Spatial grid** for proximity and ray queries. Reprioritise on step 2's numbers: the audit measured 1.6 ms per update at ~2000 sections in release and 69 ms in debug, so the lag seen may have been a debug build.
5. **Real units and parameters.** Metres and days instead of "units" and seconds; growth rates and internode lengths from botanical sources; all parameters in one struct, each citing its source.
6. **Leaves as light occluders.**
7. **Sky light model.** Several rays across the sky dome, weighted for sun path and diffuse light.
8. **Buds as entities.** Each node carries axillary buds on alternating sides (2D phyllotaxis), each with a fate: growing, dormant or dead.
9. **Resource-driven growth** (ADR-0004). Light becomes carbon, which flows through the graph; bud fate follows local resource. Replaces fork cadence, dominance parameters (ADR-0003), stochastic branching outcomes, dormancy and self-pruning.
10. **Pipe-model thickness and secondary growth.** Diameter from the children's cross-sections, thickening each year.
11. **Biomechanics.** Branches sag under their own and their leaves' weight; stiffness from diameter; breakage.
12. **Seasons.** Yearly cycle: leaf flush and fall, growth in season, annual increments.
13. **Roots.** A 2D root system supplying water and nutrients.
14. **Species parameter sets.** Measured parameters per species, with per-tree genetic variation within a species.
15. **Hormonal and vascular simulation.** Auxin, cytokinin, xylem and phloem.

### Validation sources to collect (for step 2)

Allometric relationships (height–diameter, crown–height), Leonardo da Vinci's rule (area conservation at forks), branch-angle distributions, Strahler/Horton branching ratios; measured data per species for step 14.

## Known bugs and issues

From the 13 Nov 2026 code audit (5 bugs, 7 risks, 7 design issues, 11 nits; the most important are listed here).

- **Stall wrecks the tree.** Frame time is uncapped (`src/main.rs`), so a 2 s stall turned growing tips ~190° in one frame, frozen as permanent kinks. Step 1.
- **Speed is capped at one internode per tip per frame.** Growth past `max_length` is discarded (`src/simulation.rs`), so 20× effective speed finished only 7.6× faster. Step 1.
- **Shape depends on speed and frame rate.** Same seed: dt 1/60 s gave 1,347 sections, 1/59 s gave 814. Step 1.
- **No real seed control.** quad-rand starts from the same state every launch; trees differ only through frame timing. Step 1.
- **Vertical pan** moves 1.33× faster than the cursor (`src/main.rs`: pan sensitivity uses width for both axes).
- **Thin branches can pass through each other** (3 of 48 test trees; `src/simulation.rs` clearance check).
- **Shade ignores close relatives.** The relative-skip used for collision also hides a branch's own nearby wood from the shade ray.
- **Shared slider state.** `ui::draw_slider` uses a `static mut`.
- **Hard-coded re-centre.** `C` resets to `(400, 300)` rather than `default_camera_target`.
- **Fixed UI layout** for an 800 × 600 window.
- **Untested update loop.** Step 2 adds the invariant test.
- **Clippy:** 6 minor style warnings.
