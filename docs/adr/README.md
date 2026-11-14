# Architecture decision records

One file per decision. A decision is anything a future you might reasonably want to argue with: a data structure, a modelling approach, a trade-off. How the code works internally belongs in code comments; what's next belongs in `docs/roadmap.md`.

Records 0001–0004 were written on 13 Nov 2026 from the author's build notes (`tree-sim-claude.md`, kept outside the repo) and the git history. They record decisions already made, so their dates are the dates the decisions landed, where the history shows them.

## Index

| ADR | Title | Status | Built |
|---|---|---|---|
| [0001](0001-index-based-arena.md) | Index-based arena for the tree graph | Accepted | Yes |
| [0002](0002-internode-subdivision-freeze.md) | Internode subdivision with freeze on completion | Accepted | Yes (`51eda3c`, 31 Jul 2026) |
| [0003](0003-sympodial-apical-dominance.md) | Sympodial apical dominance at forks | Accepted | Yes (`3ab7b03`, 3 Aug 2026) |
| [0004](0004-resource-driven-graph.md) | Resource-driven graph instead of a pre-allocated L-system | Accepted | No |
| [0005](0005-section-is-an-internode.md) | One section is one botanical internode | Accepted | Yes (plan 0003, 14 Nov 2026) |

## Rules

- Number sequentially; never reuse a number.
- Status is `Proposed`, `Accepted`, `Rejected` or `Superseded by ADR-NNNN`.
- Don't rewrite an accepted record. Clarify with a dated line under **Amendments**; change the decision with a new record that supersedes it.
- Name the source of every fact (commit, file, notes, date).

## Template

```markdown
# ADR-NNNN: <title>

- **Date:** YYYY-MM-DD
- **Status:** Proposed | Accepted | Rejected (reason below) | Superseded by ADR-NNNN
- **Built:** no | yes (<commit>, <date>)
- **Sources:** where each fact comes from (commit, file, notes, date)
- **Related:** other ADRs this depends on or constrains

## Context
What is true today, why it isn't enough, and what can't change.

## Decision
One bold lead sentence per decision, then its reasons.

## Alternatives considered
- **<alternative>.** Rejected: <reason>.

## Consequences
What the decision costs or forces elsewhere. Facts, not hopes.

## Open items
Questions still to settle. Delete the heading when there are none.

## Amendments
Dated clarifications. Delete the heading until the first one.
```
