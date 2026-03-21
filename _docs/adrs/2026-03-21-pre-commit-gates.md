---
title: "Pre-Commit Quality Gates"
version: "1.0.0"
status: "accepted"
date: "2026-03-21"
---

# Pre-Commit Quality Gates

## Status

Accepted

## Context

This repository needs a minimal quality bar that is enforced before code is committed.

Two failure modes are especially expensive to allow into the history:

- Rust lint regressions that Clippy already detects
- test regressions that are already caught by the existing test suite

Without an explicit gate, contributors can commit code that is obviously unready, pushing basic verification downstream into CI or review.

## Decision

This repository will enforce the following pre-commit gates:

1. no Clippy errors
2. no test failures

In practice, a commit must not be accepted if either of these conditions is true:

- `cargo clippy` reports errors
- `cargo test` reports failing tests

Warnings may still be handled separately by policy, but Clippy errors and test failures are hard blockers.

## Rationale

- Clippy catches a class of correctness and maintainability issues before review.
- Tests are the fastest signal that behavior has regressed.
- These checks are local, deterministic, and already aligned with normal Rust workflow.
- A small number of hard blockers is easier to adopt than a larger, less focused gate set.

## Consequences

### Positive

- fewer obviously broken commits enter the branch history
- less review time is spent on failures that tooling can detect directly
- contributors get immediate local feedback before pushing changes

### Negative

- commit latency increases because contributors must run or wait for checks
- contributors may need local toolchain or fixture setup before they can commit
- incomplete work may need to stay unstaged or be developed on temporary branches until it passes

## Implementation Notes

The intended enforcement mechanism is a pre-commit hook or equivalent local gate that runs at least:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

If the repository later narrows or expands the exact command set, the policy remains the same: commits must not pass with Clippy errors or failing tests.
