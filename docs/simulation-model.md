# Simulation model

How the tree grows, and why the numbers are what they are. Source of truth is the code (`src/simulation.rs`, `src/growth_section.rs`, `src/params.rs`); this page explains it.

**Units:** metres, hours and radians throughout the simulation. Only drawing converts metres to pixels (`PX_PER_M` = 200 at zoom 1×). Guide species: elm; most values are still estimates (see *Parameters*).

## The unit of growth: the internode

The tree is a list of **growth sections**. Each is one botanical internode, node to node (ADR-0005): a short straight segment each with a start point, angle, length, thickness and a parent. A section is either:

- **Growing**: lengthening each frame and able to bend, or
- **Fully grown**: frozen. Its geometry is locked and it has zero, one or two children.

```
 Growing ──reaches max_length──► fork?  ──yes──► freeze with 2 children (leader + lateral)
    │                              └──no───► freeze with 1 child (continuation)
    ├──max_length ≤ floor──────────────────► freeze with 0 children (tip → leaf)
    └──blocked by other wood───────────────► freeze with 0 children (tip → leaf)
```

## Time

The simulation advances in fixed steps of `SIM_DT` = 1 simulated hour. Each frame, `plan_steps` adds real frame time (clamped to 0.1 s) × speed to an accumulator and runs as many whole steps as it holds, up to 64. Past the cap the simulation slows down instead of building a backlog. Speed is simulated hours per real second (slider value × `SPEED_SCALE` 3,000, so 1× is about a year per 3 s; above ~1.3× the 64-step cap per frame sets the pace), the same on any display. There are no seasons yet (roadmap step 13): a year's growth is spread evenly over all 8,760 hours. All randomness comes from a generator seeded per tree, so a seed reproduces a tree exactly.

## Each step, for every growing section

1. **Sample the environment** at the tip, checking every other section except close relatives (see *Collision* below):
   - **Collision**: if the tip is within `(own thickness + other thickness) × thickness_decay` of another segment, it's blocked.
   - **Shade**: cast a ray from the tip towards the sun; count segments it crosses.
2. **Light intensity** = 0 if blocked, else `0.85 ^ shade_count`.
3. **Grow**: length increases by `growth_rate × light × Δt` (0.40 m/year, so one 3 cm internode takes ~657 h at full light).
4. **Bend** (tropism):
   - gravitropism torque = `sin(UP − angle) × gravitropism_rate`
   - phototropism torque = `sin(light angle − angle) × phototropism_rate × light`
   - flexibility = `clamp(flexibility_reference_thickness / max(thickness, min_flexibility_thickness), 0.1, 2.0)`, so thin tips bend more than thick wood
   - rates are per hour, scaled from the pre-units tuning so each internode bends as much as before
   - `angle += (gravity + photo) × flexibility × Δt`
5. **On reaching full length**, decide what happens next (below).

## Fork, continue or stop

- **Stop** if `max_length ≤ min_viable_internode_length` (0.5 cm). The axis ends; the tip gets a leaf.
- **Fork** when `should_fork` says so: after `fork_interval ± fork_jitter` internodes (12 ± 3, about one yearly shoot) since the last fork. The rule is deterministic given a random roll, so every axis is guaranteed to fork eventually.
- **Continue** otherwise: one child with the same angle and thickness, and a target length with ±8% jitter.

### Asymmetric forks (apical dominance)

A fork produces a **leader** and a **lateral**, modelling sympodial growth (e.g. elm), where one bud wins and carries the axis on.

| | Leader | Lateral |
|---|---|---|
| Thickness | `t × thickness_decay × 2 × dominance_ratio` | `… × (1 − dominance_ratio)` |
| Length | `L × length_decay × 2 × r` | `… × (1 − r)` |
| Angle offset | `spread × (1 − dominance_ratio)` | full `spread` on the other side |

where `r = 0.5 + (dominance_ratio − 0.5) × length_dominance_blend`. Length uses a softened ratio because length compounds across generations: at full asymmetry, laterals hit the length floor after one or two forks and never build a canopy.

**Leader side bias.** Which side the leader kinks to isn't a pure coin flip. It's biased (`LEADER_CORRECTION_STRENGTH` = 0.35, clamped to 10–90%) towards bringing the axis back to vertical. Without it, a thick leader barely bends, so its heading performs a random walk and the trunk snakes.

Either child is dropped if its length would fall below the floor.

## Collision: skipping relatives

With short internodes, a tip is always geometrically close to its own recent wood. The check skips any section that is an ancestor, a descendant, or shares an ancestor within `ANCESTOR_SKIP_DEPTH` (21) hops. Each section's ancestor chain is built once, when the section is created. The cheap distance and ray tests run first; the relative check only runs when one of them hits (plan 0004).

## Parameters

All in `GrowthParams` (`src/params.rs`), whose doc comments carry the full reasoning. **Estimate** = no measured source yet; roadmap step 15 calibrates.

| Parameter | Value | Source |
|---|---|---|
| `internode_length` | 3 cm | Estimate |
| `growth_rate` | 0.40 m/year (per hour) | Estimate |
| `trunk_thickness` | 5 cm | Estimate |
| `min_viable_internode_length` | 0.5 cm | 1/6 of internode, kept from earlier tuning |
| `branch_angle_spread` | 40° | Tuned |
| `length_decay` | 0.75 | Tuned |
| `thickness_decay` | 0.707 | Leonardo's rule (≈ 1/√2) |
| `dominance_ratio` / `length_dominance_blend` | 0.6 / 0.5 | Tuned (ADR-0003) |
| `fork_interval` / `fork_jitter` | 12 / 3 | Ulmus glabra: 10–15 leaves per shoot (medium confidence) |
| `gravitropism_rate` / `phototropism_rate` | per hour, derived | Estimate, scaled from earlier tuning |
| `flexibility_reference_thickness` / `min_flexibility_thickness` | 3.75 mm / 1.25 mm | Estimate, scaled from earlier tuning |
| `leaf_length` × `leaf_width` | 8 × 5 cm | Estimate |
| `sun_azimuth` | straight up | Direction *to* the sun |

Constants in `src/simulation.rs`: `ANCESTOR_SKIP_DEPTH` 20, `LENGTH_JITTER_FRACTION` 0.08, `LEADER_CORRECTION_STRENGTH` 0.35.

## Leaves

Drawn in `main.rs`, not the simulation. Each terminal section's `leaf_timer` counts up to one simulated week; the leaf ellipse scales in with a quadratic ease-out. Toggling leaves off resets every timer.

## Decisions behind this

ADR-0001 (arena), ADR-0002 (internodes and freezing), ADR-0003 (apical dominance), ADR-0005 (one section is one internode). The planned replacement for time-driven growth is ADR-0004 (resource-driven graph). All in `docs/adr/`.

## Further reading

- Palubicki et al. (2010), *Self-organizing tree models for image synthesis*.
- Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants*.
