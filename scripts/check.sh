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

# Use a repo-local target dir to avoid filesystem permission quirks.
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target-node}

# Keep compilation serial in this environment.
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-1}

# Work around occasional permission issues by forcing a clean writable target dir.
rm -rf "$CARGO_TARGET_DIR"
mkdir -p "$CARGO_TARGET_DIR"

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
