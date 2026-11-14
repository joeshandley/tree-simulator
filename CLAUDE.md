# Tree Simulator

A 2D tree-growth simulation in Rust, drawn with macroquad. One tree grows from a trunk, bends towards light and against gravity, forks, avoids its own wood and grows leaves at its tips.

**Goal: the most realistic tree simulator, realistic in mechanism, not just in appearance.** Prefer changes that make growth emerge from biology (light, resources, buds, wood mechanics) over hand-tuned rules that imitate it. 2D first; 3D is out of scope. Plan every change against this (`docs/roadmap.md`).

## Starting a session

1. Read `docs/roadmap.md` and **restate its priority list to the author** so they can reprioritise or drop items before work starts.
2. Read the ADRs in `docs/adr/` that touch the area you're working in.

## Where things live

| What | Where |
|---|---|
| Decisions and their reasons | `docs/adr/` (index and template in `docs/adr/README.md`) |
| How the growth model works, parameter values | `docs/simulation-model.md` |
| What's done, what's next, known issues | `docs/roadmap.md` |
| Implementation detail, tuning rationale, bug-fix notes | Code comments in `src/` |
| Running it, controls | `README.md` |

- A new design decision (data structure, modelling approach, a trade-off someone could argue with) gets a new ADR. Don't rewrite an accepted one; supersede it.
- When an item lands, move it to *Done* in the roadmap with its commit and date.
- Date facts and name their source.

## Setup

The Rust toolchain is installed **inside the project** in `.toolchain/` (git-ignored). Nothing lives in `~/.cargo`, `~/.rustup` or the shell profile, and nothing should be installed globally.

```sh
source env.sh            # once per terminal: points cargo at .toolchain/
cargo run --release      # open the simulator window
cargo dev                # same, but rebuilds and relaunches on every change to src/
cargo test               # unit and invariant tests
cargo test --release -- --ignored   # slow run-to-finish invariant test
cargo bench              # criterion: time to grow seeded trees to finish
cargo run --release --example metrics [-- --seeds N] [-- --write]   # tree metrics; --write updates docs/metrics/baseline.md
cargo clippy             # lints
```

If `.toolchain/` is missing, reinstall it locally (Apple Silicon shown):

```sh
mkdir -p .toolchain/installer
curl --proto '=https' --tlsv1.2 -sSf -o .toolchain/installer/rustup-init \
  https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init
chmod +x .toolchain/installer/rustup-init
RUSTUP_HOME="$PWD/.toolchain/rustup" CARGO_HOME="$PWD/.toolchain/cargo" \
  .toolchain/installer/rustup-init -y --no-modify-path --profile minimal \
  --default-toolchain stable -c clippy -c rustfmt
rm -rf .toolchain/installer
```

Requires Xcode Command Line Tools for the linker. The crate uses edition 2024, so Rust 1.85 or later.

## Module map

| File | Role |
|---|---|
| `src/lib.rs` | Library root; re-exports `GrowthSection`, `TreeSimulation`, `Vector2D`. |
| `src/vector.rs` | `Vector2D` maths: angle, dot, point-to-segment distance, ray–segment intersection. `ANGLE_UP` constant. |
| `src/growth_section.rs` | One internode (`GrowthSection`): growth and tropism physics, `freeze`, and the pure `should_fork` rule. |
| `src/params.rs` | `GrowthParams`: every tunable, in metres and hours, each with its source or "estimate". |
| `src/metrics.rs` | Tree metrics for the measurement harness. |
| `src/simulation.rs` | `TreeSimulation`: per-frame update loop, collision and shade sampling, fork/continue/terminate decisions, tuning parameters. |
| `src/main.rs` | Binary: window, camera, drawing branches and leaves, input, leaf timers. |
| `src/ui.rs` | Immediate-mode toggle button and slider. |

## Invariants — don't break these

- **Flat arena.** All sections live in `TreeSimulation::growth_sections: Vec<GrowthSection>`. Tree links are indices (`parent_index`, `child_indices`). Never remove or reorder entries; indices must stay stable.
- **Two-pass update.** `update` iterates over a snapshot length (`num_sections`), collects new children in `new_sections`, then appends them after the loop. Child indices are computed as `num_sections + new_sections.len()` before the push.
- **Frozen is final.** Once `freeze` runs, a section's state is `FullyGrown` and `end_point()` returns `cached_end_point`. Its geometry never changes again. New children take their `start_point` from the parent's end point at creation time.
- **Leaves are terminal sections.** A leaf is drawn on any `FullyGrown` section with empty `child_indices`.
- **Units.** The simulation works in metres, hours and radians; every tunable lives in `GrowthParams` (`src/params.rs`) with its unit and source. Only `main.rs` converts metres to pixels (`PX_PER_M`).
- **Screen coordinates.** +Y points down, so up is `ANGLE_UP = -π/2`.
- **`should_fork` stays pure.** Randomness is passed in (`jitter_roll`) so it can be unit-tested.
- **Fixed timestep.** The simulation only advances through `TreeSimulation::step()`, one `SIM_DT` at a time; `main.rs` turns real time into steps with `plan_steps`. Never pass frame time into the simulation.
- **One random source.** All randomness comes from `TreeSimulation`'s own seeded generator, never the global `rand::` functions. Same seed and step count, same tree.

## Conventions

- Keep randomness out of pure functions; pass rolls in as arguments.
- Tuning constants carry a doc comment explaining *why* the value was chosen. Keep that up when changing them.
- Existing code comments use American spelling; new prose docs use British spelling.
- Add unit tests alongside the module (`#[cfg(test)] mod tests`). The update loop in `simulation.rs` has no tests yet.
