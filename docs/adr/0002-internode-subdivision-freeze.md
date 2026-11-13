# ADR-0002: Internode subdivision with freeze on completion

- **Date:** 2026-07-31
- **Status:** Accepted
- **Built:** yes (`51eda3c`, 31 Jul 2026)
- **Sources:** author's build notes §6; commit `51eda3c` "Split branches into growth sections that grow independently"; `GrowthSection::freeze` doc comment
- **Related:** ADR-0001 (arena), ADR-0003 (forks happen at internode boundaries)

## Context

Branches were one rigid segment for their whole growth period. Tropism rotates a growing segment's angle every frame, so as a long branch bent, all of its already-grown wood swung with it. Real wood doesn't move once it has lignified.

## Decision

**A branch is a chain of short internodes (target length 12 units), not one segment.** Each internode grows and bends; on reaching its target length it hands over to the next.

**A finished internode is frozen permanently.** `freeze` caches the end point and switches state to `FullyGrown`. After that its angle, length, thickness and end point never change. `end_point()` returns the cached value for frozen sections.

**On completion an internode does one of three things:** spawns one child (continuation), two children (fork), or none (terminal: length floor reached, or physically blocked). Children start at the parent's cached end point.

**A future dormant-bud feature must take its start point from `cached_end_point` at creation time,** not reintroduce a per-frame position-sync pass. (From the `freeze` doc comment.)

## Alternatives considered

- **Keep one segment per branch, and stop tropism once it's "old enough".** Rejected: still a straight rod; can't show curvature accumulated during growth.
- **Recompute all positions from the root every frame.** Rejected: frozen wood would still move if any ancestor changed, which is the bug being fixed; also costs a full traversal per frame.

## Consequences

- Section count is much higher (a tree reached ~2000 sections during tuning, per the notes), which exposed the quadratic collision/shade loop. See roadmap item 1.
- Curvature is now a polyline of straight internodes; smoothness depends on internode length.
- Collision must skip a tip's recent relatives, since short internodes put a tip close to its own wood (`ANCESTOR_SKIP_DEPTH`).
- Thickness is fixed at creation, so true pipe-model thickness (roadmap item 2) will need an exception to "frozen is final" for thickness only, or a separate derived value. Not decided.

## Open items

- Whether thickness may change after freeze once pipe-model thickness lands (roadmap item 2).
