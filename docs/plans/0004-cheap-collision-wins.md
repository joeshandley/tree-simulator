# Plan 0004: Cheap collision wins

- **Roadmap step:** 6, done before steps 4–5 (author, 14 Nov 2026: the crown phase was slow to watch)
- **Date:** 14 Nov 2026
- **Status:** built

## Problem

Each step, every growing tip checked every other section. For each pair it first ran the "recent relative" check (up to ~440 comparisons of ancestor lists), before the cheap distance and ray tests, and every section's ancestor chain was rebuilt from scratch every step.

## Changes

1. Distance and ray tests first; the relative check only for the rare pairs that hit one of them. Same outcome for every pair.
2. Ancestor chains kept per section, built once at creation (a section's ancestry never changes).
3. Squared distance: **skipped.** Rounding near the threshold could flip a borderline collision and change trees, for a negligible gain.

## Proof

- A fingerprint test (`optimisations_leave_the_tree_unchanged`) hashes seed 3's full state after 30,000 steps, recorded before the change at `7f3fd35`; it still matches.
- The 20-seed metrics baseline is identical except timings.

| | Before | After |
|---|---|---|
| Seed 1 grown to finish | 28.5 s | 3.6 s |
| Seed 2 | 4.0 s | 0.49 s |
| Seed 3 | 8.5 s | 1.04 s |
| Mean step (20 seeds) | 183 µs | 23 µs |
| Slowest step | 11.9 ms | 0.56 ms |
