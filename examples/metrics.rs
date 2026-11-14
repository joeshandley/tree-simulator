//! Grows seeded trees without a window and reports their metrics (plan 0002).
//!
//!   cargo run --release --example metrics                 seeds 1-20, print
//!   cargo run --release --example metrics -- --seeds 5    seeds 1-5
//!   cargo run --release --example metrics -- --write      also write docs/metrics/baseline.md

use std::process::Command;
use std::time::Instant;
use tree_simulator::metrics::{TreeMetrics, measure};
use tree_simulator::{TreeSimulation, Vector2D};

/// Simulated hours before a run is cut off (about 25 years).
const MAX_STEPS: u32 = 220_000;

struct Run {
    metrics: TreeMetrics,
    steps: u32,
    finished: bool,
    micros_per_step: f64,
    slowest_step_micros: f64,
}

fn grow(seed: u64) -> Run {
    let mut sim = TreeSimulation::new(Vector2D::new(0.0, 0.0), seed);
    let started = Instant::now();
    let (mut steps, mut slowest) = (0, 0.0_f64);
    while !sim.is_finished() && steps < MAX_STEPS {
        let step_started = Instant::now();
        sim.step();
        slowest = slowest.max(step_started.elapsed().as_secs_f64() * 1e6);
        steps += 1;
    }
    let micros_per_step = started.elapsed().as_secs_f64() * 1e6 / steps.max(1) as f64;
    Run {
        metrics: measure(&sim.growth_sections),
        steps,
        finished: sim.is_finished(),
        micros_per_step,
        slowest_step_micros: slowest,
    }
}

fn row(name: &str, values: &[f64], decimals: usize) -> String {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    format!("| {name} | {mean:.decimals$} | {min:.decimals$} | {max:.decimals$} |")
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args
        .iter()
        .position(|a| a == "--seeds")
        .and_then(|i| args.get(i + 1))
        .map(|n| n.parse().expect("--seeds takes a number"))
        .unwrap_or(20);
    let write = args.iter().any(|a| a == "--write");

    let runs: Vec<Run> = (1..=seeds).map(grow).collect();
    let col = |f: &dyn Fn(&Run) -> Option<f64>| -> Vec<f64> { runs.iter().filter_map(f).collect() };

    let unfinished = runs.iter().filter(|r| !r.finished).count();
    let mut table = vec![
        "| Metric | Mean | Min | Max |".to_string(),
        "|---|---|---|---|".to_string(),
        row("Years to finish", &col(&|r| Some(r.steps as f64 / 8_760.0)), 1),
        row("Sections", &col(&|r| Some(r.metrics.sections as f64)), 0),
        row("Tips", &col(&|r| Some(r.metrics.tips as f64)), 0),
        row("Height (m)", &col(&|r| Some(r.metrics.height as f64)), 2),
        row("Crown width (m)", &col(&|r| Some(r.metrics.crown_width as f64)), 2),
        row("Fork angle (°)", &col(&|r| r.metrics.mean_fork_angle_deg.map(f64::from)), 1),
        row("Leonardo ratio", &col(&|r| r.metrics.mean_leonardo_ratio.map(f64::from)), 3),
        row("Time per step (µs)", &col(&|r| Some(r.micros_per_step)), 0),
        row("Slowest step (µs)", &col(&|r| Some(r.slowest_step_micros)), 0),
    ];
    if unfinished > 0 {
        table.push(format!("\n{unfinished} of {seeds} trees hit the {MAX_STEPS}-step cap."));
    }
    let table = table.join("\n");
    println!("{seeds} seeded trees (seeds 1-{seeds}), release build\n\n{table}");

    if write {
        let commit = command_output("git", &["rev-parse", "--short", "HEAD"]);
        let date = command_output("date", &["+%Y-%m-%d"]);
        let cpu = command_output("sysctl", &["-n", "machdep.cpu.brand_string"]);
        let doc = format!(
            "# Metrics baseline\n\nWritten by `cargo run --release --example metrics -- --write`. \
             Definitions: plan 0002 (D11). When a definition changes, add a dated note here; \
             older numbers stop being comparable.\n\n- **Date:** {date}\n- **Commit:** `{commit}`\n\
             - **Machine:** {cpu} ({} {})\n- **Seeds:** 1-{seeds}\n\n{table}\n",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        std::fs::write("docs/metrics/baseline.md", doc).expect("write docs/metrics/baseline.md");
        println!("\nWrote docs/metrics/baseline.md");
    }
}
