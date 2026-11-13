# Simulation model

How the tree grows, and why the numbers are what they are. Source of truth is the code (`src/simulation.rs`, `src/growth_section.rs`); this page explains it.

## The unit of growth: the internode

The tree is a list of **growth sections** (internodes): short straight segments, each with a start point, angle, length, thickness and a parent. A section is either:

- **Growing**: lengthening each frame and able to bend, or
- **Fully grown**: frozen. Its geometry is locked and it has zero, one or two children.

```
 Growing ──reaches max_length──► fork?  ──yes──► freeze with 2 children (leader + lateral)
    │                              └──no───► freeze with 1 child (continuation)
    ├──max_length ≤ floor──────────────────► freeze with 0 children (tip → leaf)
    └──blocked by other wood───────────────► freeze with 0 children (tip → leaf)
```

## Each frame, for every growing section

1. **Sample the environment** at the tip, checking every other section except close relatives (see *Collision* below):
   - **Collision**: if the tip is within `(own thickness + other thickness) × thickness_decay` of another segment, it's blocked.
   - **Shade**: cast a ray from the tip towards the sun; count segments it crosses.
2. **Light intensity** = 0 if blocked, else `0.85 ^ shade_count`.
3. **Grow**: length increases by `growth_rate × light × Δt` (`growth_rate` = 50).
4. **Bend** (tropism):
   - gravitropism torque = `sin(UP − angle) × gravitropism_weight` (1.0)
   - phototropism torque = `sin(light angle − angle) × phototropism_weight × light` (0.5)
   - flexibility = `clamp(1.5 / max(thickness, 0.5), 0.1, 2.0)`, so thin tips bend more than thick wood
   - `angle += (gravity + photo) × flexibility × Δt`
5. **On reaching full length**, decide what happens next (below).

## Fork, continue or stop

- **Stop** if `max_length ≤ MIN_VIABLE_INTERNODE_LENGTH` (2.0). The axis ends; the tip gets a leaf.
- **Fork** when `should_fork` says so: after `fork_interval ± fork_jitter` internodes (9 ± 3) since the last fork. The rule is deterministic given a random roll, so every axis is guaranteed to fork eventually.
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

With short internodes, a tip is always geometrically close to its own recent wood. The check skips any section that is an ancestor, a descendant, or shares an ancestor within `ANCESTOR_SKIP_DEPTH` (10) hops. Ancestor chains are computed once per frame.

## Parameters

| Parameter | Value | Why |
|---|---|---|
| `internode_length` | 12.0 | Trunk's starting internode length. |
| `branch_angle_spread` | 40° | Lateral's angle off the parent. |
| `length_decay` | 0.75 | Combined length budget per fork. Raised from 0.7 so the leader survives more generations; the leader hitting the floor is what caps tree size. |
| `thickness_decay` | 0.707 | ≈ 1/√2 — Leonardo da Vinci's rule (cross-section area is conserved across a split). |
| `dominance_ratio` | 0.6 | Leader's share; 0.5 would be symmetric. |
| `length_dominance_blend` | 0.5 | How much of the dominance asymmetry applies to length (see above). |
| `fork_interval` / `fork_jitter` | 9 / 3 | Longer straight runs add size linearly rather than exponentially. |
| `MIN_VIABLE_INTERNODE_LENGTH` | 2.0 | Low relative to 12 so several fork generations fit before termination. |
| `ANCESTOR_SKIP_DEPTH` | 10 | Covers a full `fork_interval + fork_jitter` run plus margin. |
| `LENGTH_JITTER_FRACTION` | 0.08 | Proportional, so it stays minor at small lengths. |
| `LEADER_CORRECTION_STRENGTH` | 0.35 | Self-corrects the leader while still letting it zigzag visibly. |
| `sun_azimuth` | straight up | Direction *to* the sun. |

## Leaves

Drawn in `main.rs`, not the simulation. Each terminal section's `leaf_timer` counts up to 1 second; the leaf ellipse scales in with a quadratic ease-out. Toggling leaves off resets every timer.

## Decisions behind this

ADR-0001 (arena), ADR-0002 (internodes and freezing), ADR-0003 (apical dominance). The planned replacement for time-driven growth is ADR-0004 (resource-driven graph). All in `docs/adr/`.

## Further reading

- Palubicki et al. (2009), *Self-organizing tree models for image synthesis*.
- Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants*.
