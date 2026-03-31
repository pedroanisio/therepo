---
title: "Test Coverage Threshold Above 90 Percent"
version: "1.0.0"
status: "accepted"
date: "2026-03-21"
---

# Test Coverage Threshold Above 90 Percent

## Status

Accepted

## Context

The repository now enforces pre-commit quality gates for Clippy and tests. That prevents obviously broken code from being committed, but it does not say anything about how much of the codebase is actually exercised.

Without a coverage target, it is easy for the test suite to remain green while meaningful portions of the code are left unverified.

## Decision

This repository adopts a minimum automated test coverage target above 90%.

The policy is:

1. overall coverage must remain greater than 90%
2. changes that reduce coverage below that threshold are not acceptable
3. new code should be accompanied by tests unless there is a documented reason it cannot be covered directly

## Rationale

- A high coverage floor creates pressure to test new paths as they are introduced.
- It reduces the chance that refactors silently bypass important logic.
- It complements the pre-commit gate by improving confidence in the tests that are required to pass.
- A threshold above 90% is high enough to matter without demanding unrealistic 100% coverage.

## Consequences

### Positive

- test additions become a normal part of feature work
- regressions are more likely to be caught before review or release
- contributors have a clearer definition of “done”

### Negative

- some changes will take longer because test scaffolding must be added
- coverage tooling must be maintained in CI or local workflows
- coverage percentage can become a vanity metric if used without judgment

## Implementation Notes

The repository measures coverage with `cargo llvm-cov` using a repo-managed script:

```bash
./scripts/check-coverage.sh
```

That script enforces line coverage via an environment variable with a default of 91:

```bash
MIN_LINE_COVERAGE="${MIN_LINE_COVERAGE:-91}"

cargo llvm-cov \
  --manifest-path crates/repo-cli/Cargo.toml \
  --all-features \
  --workspace \
  --fail-under-lines "$MIN_LINE_COVERAGE" \
  --summary-only
```

The script is enforced in CI and by the repository-managed `pre-push` hook.
To temporarily lower the bar locally: `MIN_LINE_COVERAGE=80 ./scripts/check-coverage.sh`.

Coverage policy should focus on meaningful exercised behavior, not just line-count inflation.
