# Project-local Rust toolchain. Usage: `source env.sh` from the project root.
# Nothing here touches ~/.cargo, ~/.rustup or your shell profile.
_root="$(cd "$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")" && pwd)"
export RUSTUP_HOME="$_root/.toolchain/rustup"
export CARGO_HOME="$_root/.toolchain/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
unset _root
