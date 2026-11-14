//! Time to grow seeded trees from seed to finish (plan 0002, D7).
//!
//! Whole runs rather than single steps at a given size: a tree reaching a size
//! is usually nearly finished, so single-step timings there mostly measure an
//! idle tree. The metrics example reports mean and slowest step per run.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use tree_simulator::{TreeSimulation, Vector2D};

fn grow_to_finish(seed: u64) -> usize {
    let mut sim = TreeSimulation::new(Vector2D::new(0.0, 0.0), seed);
    while !sim.is_finished() {
        sim.step();
    }
    sim.growth_sections.len()
}

fn full_runs(c: &mut Criterion) {
    let mut group = c.benchmark_group("grow_to_finish");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(5));
    // Seed 1 is the largest of seeds 1-20 (2,083 sections), seed 3 among the smallest.
    for seed in [1, 2, 3] {
        group.bench_function(format!("seed_{seed}"), |b| {
            b.iter(|| grow_to_finish(black_box(seed)))
        });
    }
    group.finish();
}

criterion_group!(benches, full_runs);
criterion_main!(benches);
