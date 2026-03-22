#!/usr/bin/env bash
set -euo pipefail

# When piping output (e.g., `... | head`), avoid exiting on SIGPIPE.
trap '' PIPE

cd "$(dirname "$0")/.."

# Ensure rustup env is loaded for non-interactive shells
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Optional: metrics (only if installed)
if command -v km >/dev/null 2>&1; then
  km cycom --include-tests --per-function --top 20
fi

# Optional: coverage (tarpaulin uses llvm engine here)
if command -v cargo-tarpaulin >/dev/null 2>&1; then
  cargo tarpaulin --engine llvm --workspace --out Stdout >/dev/null
fi

echo "OK"
