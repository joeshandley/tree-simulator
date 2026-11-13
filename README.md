# Tree Growth Simulator

A small Rust program that grows a 2D tree in real time. Branches lengthen, curve upwards and towards the light, fork into a leading stem and a side branch, stop when they bump into other wood, slow down in shade, and sprout leaves at their tips.

Built with [macroquad](https://macroquad.rs).

## Running it

Everything (including the Rust toolchain) is kept inside this folder. See `CLAUDE.md` for first-time setup.

```sh
source env.sh
cargo run --release
```

## Controls

| Input | Action |
|---|---|
| Scroll, or `+` / `-` | Zoom |
| Click and drag | Pan |
| `Space` or the `>>`/`||` button | Pause / play |
| Speed slider | Simulation speed (0.1×–5×) |
| `R` | Grow a new tree |
| `C` | Re-centre the view |
| *Pan Drag* button | Invert pan direction |
| *Show Leaves* button | Toggle leaves |

## How it works

See `docs/simulation-model.md`.
