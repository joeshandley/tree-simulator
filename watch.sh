#!/bin/sh
# Rebuild and relaunch the simulator whenever src/ or Cargo.toml changes.
# Usage: ./watch.sh   (from this checkout or any worktree of it)
cd "$(dirname "$0")"
. "$(git rev-parse --path-format=absolute --git-common-dir)/../env.sh"
exec cargo watch -w src -w Cargo.toml -x 'run --release'
