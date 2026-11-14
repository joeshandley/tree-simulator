//! Arena invariants from ADR-0001 and ADR-0002, checked on seeded runs.

use tree_simulator::{GrowthSectionState, TreeSimulation, Vector2D};

fn grow(seed: u64, steps: usize) -> TreeSimulation {
    let mut sim = TreeSimulation::new(Vector2D::new(0.0, 0.0), 20.0, seed);
    for _ in 0..steps {
        sim.step();
    }
    sim
}

/// Every link is in range and agrees in both directions, parents come before
/// children (append-only arena), and each child starts at its parent's end.
fn assert_links_valid(sim: &TreeSimulation, seed: u64) {
    let sections = &sim.growth_sections;
    for (i, section) in sections.iter().enumerate() {
        if let Some(p) = section.parent_index {
            assert!(p < i, "seed {seed}: section {i} has parent {p} after it");
            match &sections[p].state {
                GrowthSectionState::FullyGrown { child_indices } => assert!(
                    child_indices.contains(&i),
                    "seed {seed}: parent {p} doesn't list child {i}"
                ),
                GrowthSectionState::Growing => panic!("seed {seed}: parent {p} of {i} still growing"),
            }
            assert_eq!(
                section.start_point,
                sections[p].end_point(),
                "seed {seed}: section {i} doesn't start at its parent's end"
            );
        } else {
            assert_eq!(i, 0, "seed {seed}: only the trunk has no parent");
        }
        if let GrowthSectionState::FullyGrown { child_indices } = &section.state {
            for &c in child_indices {
                assert!(c > i && c < sections.len(), "seed {seed}: child {c} of {i} out of range");
                assert_eq!(sections[c].parent_index, Some(i), "seed {seed}: child {c} doesn't point back to {i}");
            }
        }
    }
}

/// Geometry of every frozen section, for checking it never changes.
fn frozen_geometry(sim: &TreeSimulation) -> Vec<(usize, String)> {
    sim.growth_sections
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s.state, GrowthSectionState::FullyGrown { .. }))
        .map(|(i, s)| {
            (i, format!("{:?} {:?} {} {} {}", s.start_point, s.end_point(), s.angle, s.thickness, s.max_length))
        })
        .collect()
}

fn assert_frozen_sections_never_change(seed: u64, first: usize, more: usize) {
    let mut sim = grow(seed, first);
    let before = frozen_geometry(&sim);
    for _ in 0..more {
        sim.step();
    }
    let after = frozen_geometry(&sim);
    for (i, geometry) in &before {
        let now = &after.iter().find(|(j, _)| j == i).expect("frozen section vanished").1;
        assert_eq!(now, geometry, "seed {seed}: frozen section {i} changed");
    }
}

#[test]
fn arena_invariants_hold_mid_growth() {
    for seed in 1..=4 {
        assert_links_valid(&grow(seed, 600), seed);
        assert_frozen_sections_never_change(seed, 300, 300);
    }
}

/// Full run to the end; slow in debug, so run with
/// `cargo test --release -- --ignored`.
#[test]
#[ignore]
fn arena_invariants_hold_to_finish() {
    for seed in 1..=3 {
        let mut sim = grow(seed, 0);
        let mut steps = 0;
        while !sim.is_finished() {
            sim.step();
            steps += 1;
            assert!(steps <= 20_000, "seed {seed}: unfinished after 20,000 steps");
        }
        assert_links_valid(&sim, seed);
    }
}
