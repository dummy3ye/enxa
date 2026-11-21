#!/usr/bin/env bash
# One command to know whether the repo is healthy.
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
echo "OK"
