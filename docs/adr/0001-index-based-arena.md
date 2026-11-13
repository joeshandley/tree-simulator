# ADR-0001: Index-based arena for the tree graph

- **Date:** 2026-07-31 (in place by `51eda3c`; may predate it)
- **Status:** Accepted
- **Built:** yes
- **Sources:** author's build notes §4 (13 Nov 2026 import); `src/simulation.rs`, `src/growth_section.rs`
- **Related:** ADR-0002 (relies on stable indices), ADR-0004 (resource flow traverses this graph)

## Context

The tree is a graph: every section has one parent and zero or more children. The simulation needs to walk it both ways (ancestor chains for collision today; resource flow up and down every frame under ADR-0004). In Rust, holding `&mut` references to a node and its parent or children at once fights the borrow checker, and `Rc<RefCell<…>>` trees add runtime cost and panics on double borrow.

## Decision

**All sections live in one flat `Vec<GrowthSection>`; parent and child links are `usize` indices.** `parent_index: Option<usize>` on each section; `child_indices: Vec<usize>` inside `GrowthSectionState::FullyGrown`.

**Entries are never removed or reordered.** Indices are the identity of a section, so they must stay stable for the life of a tree. Reset replaces the whole vector.

**New sections are appended after the update pass.** `update` loops over a snapshot length, collects children in a side vector, computes each child's index as `num_sections + new_sections.len()`, and appends at the end. This keeps the loop's indices valid while it mutates.

## Alternatives considered

- **`Rc<RefCell<Node>>` with `Weak` parent links.** Rejected: runtime borrow panics on simultaneous parent/child access, more allocation, harder to snapshot.
- **Owned child trees (`Vec<Node>` inside each node).** Rejected: no cheap upward traversal; parent access needs unsafe or a separate index.

## Consequences

- Removing wood (pruning, storm damage — roadmap items) can't delete entries. It needs a tombstone state or a "dead" flag, and every loop must skip it.
- Memory grows monotonically with the tree; fine at the ~2000-section scale seen so far.
- A bug that writes a wrong index fails silently or panics on out-of-range access; there's no type-level protection. An invariant test over the arena is on the roadmap.

## Open items

- How to represent removed sections once pruning lands (tombstone variant vs. flag).
