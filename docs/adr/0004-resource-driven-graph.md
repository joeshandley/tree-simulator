# ADR-0004: Resource-driven graph instead of a pre-allocated L-system

- **Date:** 2026-08-03 (assumed: date of the latest commit, `3ab7b03`, per the author on 13 Nov 2026)
- **Status:** Accepted
- **Built:** no. The realism roadmap comes first (author's build notes §7).
- **Sources:** author's build notes §2–§5 and §8 (13 Nov 2026 import)
- **Related:** ADR-0001 (graph structure it runs on), ADR-0003 (approximation it replaces)

## Context

The original plan was a deterministic, pre-allocated L-system: generate the whole tree's "DNA" string at t = 0, then draw it. Standard L-systems are context-free, so pre-allocating the tree makes it very hard to respond to the environment while it grows: pruning, a storm breaking a branch, local nutrient starvation, or dormant buds waking up. `lsystem.rs` was deleted.

## Decision

**Growth will be driven by resources flowing through the tree graph,** following functional–structural plant models, specifically the extended Borchert–Honda model (Palubicki et al., 2009).

- **Acropetal flow (upwards):** roots gather water and nutrients and push them up the branches.
- **Basipetal flow (downwards):** leaves and tips gather light (carbon) and push it down.
- **Vigour:** a tip or dormant bud only elongates or spawns children once its local resources pass a threshold.
- **Dynamic response:** if a branch is removed, the resources that went to it redirect to dormant lateral buds lower down, which wake and form a new leader.

**Each frame runs in three phases:**

1. **Flow calculation:** read-only pass to work out resource distribution.
2. **Growth application:** mutate lengths and thickness from accumulated resources.
3. **Topology update:** spawn new sections where vigour thresholds are met.

Separating read from write keeps the arena (ADR-0001) borrow-safe and the passes testable.

**L-system rewriting may return later as a "just-in-time" genetic ruleset,** evaluated only when resources allow, not as a rigid blueprint. Roadmap item 8 (parameter genome) is a lighter precursor.

## Alternatives considered

- **Pre-allocated, context-free L-system.** Rejected: can't respond to damage, shade or resource changes after t = 0.
- **Keep today's time-driven procedural rules.** Not rejected outright; it's the interim model while the realism roadmap is done first.

## Consequences

- Topology moves out of the hard-coded update loop into the vigour threshold. Today's fork cadence (`fork_interval`, `should_fork`) and dominance parameters (ADR-0003) become candidates for removal.
- Every frame needs two full traversals (up and down). The arena's flat indices make this a pair of linear passes if sections are stored parent-before-child, which append-only growth already guarantees.
- Pruning (roadmap item 4) and dormant buds need a representation the arena doesn't have yet (ADR-0001 open item).

## Open items

- Vigour threshold values and how roots are modelled (no root system exists yet).
- Whether to build this before or after the 2D realism roadmap is finished. The notes say after; restate this when resuming.

## Further reading

- Palubicki et al. (2009), *Self-organizing tree models for image synthesis*: combines L-systems with Borchert–Honda resource flow.
- Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants*: the botanical rules to overlay on the resource graph later.

## Amendments

- **14 Nov 2026.** Timing settled: this is step 9 of the roadmap, after the timestep, measurement, performance, units, light and bud steps. It also absorbs stochastic branching outcomes, dormancy and self-pruning, which were separate roadmap items. Reason: the goal is realism in mechanism, so tuning phenomenological rules that this replaces would be wasted work (`docs/roadmap.md`, author, 14 Nov 2026).
