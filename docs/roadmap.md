# Roadmap

Last updated 13 Nov 2026. Sources: the author's build notes (§6–§8, imported 13 Nov 2026) and a code read on the same date.

**When resuming this project, restate the priority list back to the author so they can reassess whether any item is redundant or superseded.**

## Done

- Vector maths with tests (`ee4a003`, 25 Jul 2026)
- Collision avoidance, canopy shading, leaves, time controls (25 Jul 2026)
- Internode subdivision with freeze (ADR-0002, `51eda3c`, 31 Jul 2026)
- Sympodial apical dominance (ADR-0003, `3ab7b03`, 3 Aug 2026)
- Project-local toolchain and docs (13 Nov 2026)

## Priority list: 2D realism

Ordered by the author. Spatial partitioning moved to the top after real lag was seen at ~2000 growth sections.

1. **Spatial partitioning for collision and shade.** The double loop in `TreeSimulation::update` is O(growing tips × all sections) per frame. Add a grid or bounding-box broad phase so a tip only checks nearby sections. In the same pass:
   - cache each section's ancestor chain once (at creation or lazily) instead of rebuilding ~2000 `Vec<usize>`s every frame; frozen chains never change;
   - compare squared distance against `min_clearance²` in the collision check and skip the `sqrt` in `distance_to_segment` on that path.
2. **True pipe-model thickness.** Recompute thickness bottom-up so a parent's cross-section equals the sum of its children's, rather than a fixed `thickness_decay` per fork. Needs a decision on thickness after freeze (ADR-0002 open item).
3. **Stochastic branching outcomes.** Per tip: die, stay dormant, continue, fork into 2, rarely fork into 3.
4. **Self-pruning.** Track how long a section has been blocked or shaded; shed it past a threshold. Needs a removal representation (ADR-0001 open item).
5. **Leaves as light occluders.** The shade raycast only tests branches; most real self-shading is foliage.
6. **Multi-ray light sampling.** A small fan of rays across the sky instead of one ray per tip, for smoother shade.
7. **Broader per-section variation.** Noise on branch angle, growth rate and fork cadence, not just length.
8. **Parameter genome.** A small genotype that nudges the existing tuning parameters per tree, so trees from one model read as individuals. A lighter precursor to ADR-0004's genetic ruleset.

## Smaller issues

From the 13 Nov 2026 code read. Not prioritised against the list above.

- **No seed control.** `rand::srand` is never called and growth steps on variable frame time (`get_frame_time()`), so a given tree can't be reproduced on demand.
- **Shared slider state.** `ui::draw_slider` uses a `static mut SLIDER_ACTIVE`; a second slider would share it.
- **Hard-coded re-centre.** `C` resets the camera to `(400, 300)` rather than `default_camera_target` (`src/main.rs`).
- **Fixed UI layout.** Absolute pixel positions for an 800 × 600 window.
- **Tuning only in code.** No UI for growth parameters.
- **Untested update loop.** `TreeSimulation::update` has no tests; an arena invariant test (valid indices, frozen sections unchanged, run finishes) would cover it.
- **Clippy.** 6 minor style warnings.

## Next phase: resource-driven growth

ADR-0004. Starts after the 2D realism list, per the author's notes; reconfirm when resuming.

## Long-term: hormonal and vascular simulation

Not near-term, and nothing decided. Once resource flow works, go a level deeper than abstract "vigour": auxin (apical dominance; ADR-0003 is today's stand-in), cytokinin (from roots, promotes lateral buds), and the xylem and phloem that carry them. A refinement on top of ADR-0004, not a prerequisite for it.
