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
- Fixed timestep and seed (step 1, plan 0001, 14 Nov 2026)
- Measurement harness: metrics, benchmark, baseline, arena invariant test (step 2, plan 0002, 14 Nov 2026)
- Real units and parameters: metres, hours, one section per internode, `GrowthParams` (step 3, plan 0003, ADR-0005, 14 Nov 2026)
- Cheap collision wins, done before steps 4–5 (step 6, plan 0004, 14 Nov 2026)

## Priority list

Today's growth rules are phenomenological (tuned to look right). Items that only tuned those rules were folded into the resource model (step 9), which replaces them.

1. ~~Fixed timestep and seed~~ (done, see above).
2. ~~Measurement harness~~ (done). Baseline in `docs/metrics/baseline.md`.
3. ~~Real units and parameters~~ (done, plan 0003). Was: metres and hours instead of pixels and "simulated seconds"; growth rates and internode lengths from botanical sources; all parameters in one struct, each citing its source. Guide species: elm (single trunk, distichous, sympodial, UK native), with hazel then hornbeam and lime as fallbacks where elm data is thin (author, 14 Nov 2026).
4. **Leaves as light occluders.**
5. **Sky light model.** Several rays across the sky dome, weighted for sun path and diffuse light.
6. ~~Cheap collision wins~~ (done early, plan 0004): ancestor chains cached, geometry checked before relatedness; whole runs ~7.8× faster with identical trees. Squared distance skipped: it could flip borderline collisions for a negligible gain.
7. **Spatial grid** for proximity and ray queries, if needed. Step 2 measured the largest seeded tree (2,083 sections) growing start to finish in 186 ms, slowest step 659 µs (release, 14 Nov 2026), so not needed yet; re-measure after step 5.
8. **Buds as entities.** Each node carries axillary buds on alternating sides (2D phyllotaxis), each with a fate: growing, dormant or dead.
9. **Resource-driven growth** (ADR-0004). Light becomes carbon, which flows through the graph; bud fate follows local resource. Replaces fork cadence, dominance parameters (ADR-0003), stochastic branching outcomes, dormancy and self-pruning.
10. **Pipe-model thickness and secondary growth.** Diameter from the children's cross-sections, thickening each year.
11. **Biomechanics.** Branches sag under their own and their leaves' weight; stiffness from diameter; breakage.
12. **Seasons.** Yearly cycle: leaf flush and fall, growth in season, annual increments.
13. **Roots.** A 2D root system supplying water and nutrients.
14. **Calibrate elm.** Replace the estimates in `GrowthParams` with measured values, and compare metrics against real-tree reference ranges (D10 in plan 0002). Features first, numbers after (author, 14 Nov 2026).
15. **Species parameter sets.** Measured parameters per species beyond elm, with per-tree genetic variation within a species.
16. **Hormonal and vascular simulation.** Auxin, cytokinin, xylem and phloem.
17. **Coppicing and woodland management.** Cutting a stem and regrowth from dormant buds on the stool, following British coppice practice (hazel first). Needs buds (8), resource flow (9), seasons (12) and root reserves (13). Kept low; new items may push it further down (author, 14 Nov 2026).

### Validation sources to collect (for step 2)

Allometric relationships (height–diameter, crown–height), Leonardo da Vinci's rule (area conservation at forks), branch-angle distributions, Strahler/Horton branching ratios; measured data per species for step 14.

## Known bugs and issues

From the 13 Nov 2026 code audit (5 bugs, 7 risks, 7 design issues, 11 nits; the most important are listed here).

- ~~Stall wrecks the tree; speed capped at one internode per tip per frame; shape depends on speed and frame rate; no seed control.~~ Fixed by step 1 (14 Nov 2026).
- **Vertical pan** moves 1.33× faster than the cursor (`src/main.rs`: pan sensitivity uses width for both axes).
- **Thin branches can pass through each other** (3 of 48 test trees; `src/simulation.rs` clearance check).
- **Shade ignores close relatives.** The relative-skip used for collision also hides a branch's own nearby wood from the shade ray.
- **Shared slider state.** `ui::draw_slider` uses a `static mut`.
- **Hard-coded re-centre.** `C` resets to `(400, 300)` rather than `default_camera_target`.
- **Fixed UI layout** for an 800 × 600 window.
- ~~Untested update loop.~~ Arena invariant test added in step 2. It found and fixed a child/parent end-point mismatch in release builds (14 Nov 2026).
- **Clippy:** 6 minor style warnings.
