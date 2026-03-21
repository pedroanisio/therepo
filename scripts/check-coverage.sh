#!/usr/bin/env sh
set -eu

CRATE_MANIFEST="crates/repo-cli/Cargo.toml"
MIN_LINE_COVERAGE="${MIN_LINE_COVERAGE:-75}"

cargo llvm-cov \
  --manifest-path "$CRATE_MANIFEST" \
  --all-features \
  --workspace \
  --fail-under-lines "$MIN_LINE_COVERAGE" \
  --summary-only
