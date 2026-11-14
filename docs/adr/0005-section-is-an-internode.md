# ADR-0005: One section is one botanical internode

- **Date:** 2026-11-14
- **Status:** Accepted
- **Built:** yes (plan 0003)
- **Sources:** decision with the author, 14 Nov 2026 (plan 0003, D13)
- **Related:** ADR-0001 (arena), ADR-0002 (internodes freeze; this fixes what an internode is), ADR-0004 (resource flow runs over these sections)

## Context

Moving to real units (roadmap step 3) forced a choice about what a growth section represents. Real elm internodes are a few centimetres long, so a realistic section count for a grown tree is tens to hundreds of thousands, against about 1,100 at the time. Every per-step check scales with section count, and every later model (buds at nodes, resource flow, pipe thickness, bending) works on whatever a section is.

## Decision

**A section is one botanical internode: node to node, with its node, and later its buds and leaf, at the section's end.** Its target length is `GrowthParams::internode_length`, a real length (3 cm, estimate).

**Simulation time stays affordable by simulating young trees** (a few years, a few thousand sections) until the performance steps (roadmap 6–7) are needed.

## Alternatives considered

- **A section is a simulation segment of tunable length, separate from nodes.** Rejected by the author: cheaper now, but harder to bring back to real internodes later, and buds would sit part-way along sections.
- **A section is one year's shoot.** Rejected: too coarse for bending and branching within a year.

## Consequences

- Section counts grow with realism; collision and shade checks need the performance steps sooner than they otherwise would.
- `ANCESTOR_SKIP_DEPTH` must cover a full run between forks in internodes (raised to 20 for 12 ± 3), which makes each step dearer: mean step time rose from ~66 µs to ~220 µs (plan 0003 measurements).
- Buds (roadmap 8) attach to section ends, one node per section.
