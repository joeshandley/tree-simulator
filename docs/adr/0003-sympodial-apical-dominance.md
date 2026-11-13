# ADR-0003: Sympodial apical dominance at forks

- **Date:** 2026-08-03
- **Status:** Accepted
- **Built:** yes (`3ab7b03`, 3 Aug 2026)
- **Sources:** author's build notes §6 and §8; commit `3ab7b03` "Adjust branch forking to have a primary and secondary branch"; doc comments on `dominance_ratio`, `length_dominance_blend`, `LEADER_CORRECTION_STRENGTH` in `src/simulation.rs`
- **Related:** ADR-0002 (forks happen at internode completion), ADR-0004 (resource flow is meant to replace this approximation)

## Context

Forks produced two identical children. The result was a symmetric, fan-like tree with no trunk. In real sympodial species (elm, for example) one bud at each fork wins the race for resources and becomes the new leader; the other becomes a subordinate lateral. The biology behind this (auxin-driven correlative inhibition) is out of scope until the resource model exists (ADR-0004), so a procedural stand-in is needed now.

## Decision

**Each fork produces a leader and a lateral that split the fork's budget asymmetrically.** `dominance_ratio` (0.6) is the leader's share of thickness; the lateral gets the rest. The leader's angle offset is `branch_angle_spread × (1 − dominance_ratio)`; the lateral takes the full spread on the other side.

**Length uses a softened asymmetry.** `length_dominance_blend` (0.5) scales how much of the dominance asymmetry applies to length. Length compounds across generations, so full asymmetry starves laterals at the length floor after one or two forks.

**The leader's side is biased towards vertical.** The coin flip for which side the leader takes is weighted by `LEADER_CORRECTION_STRENGTH` (0.35, probability clamped 10–90%) towards correcting the axis back to upright. A thick leader barely bends under gravitropism before it freezes, so an unbiased flip is a random walk in heading and the trunk snakes.

**This is an approximation, to be replaced by emergent dominance once resource flow exists.**

## Alternatives considered

- **Symmetric forks (previous behaviour).** Rejected: no trunk, unrealistic silhouette.
- **Full asymmetry on length as well.** Rejected: laterals terminate too early; sparse "handful of branches" tree (per `length_dominance_blend` doc comment).
- **Unbiased leader side.** Rejected: snaking trunk.

## Consequences

- Three more tuning parameters interact with `length_decay` and `MIN_VIABLE_INTERNODE_LENGTH`. `length_decay` was raised from 0.7 to 0.75 to compensate (see its comment).
- When ADR-0004 lands, these parameters should be removed or demoted to initial conditions, not left fighting the resource model.
