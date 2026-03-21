# Contributing

## Scope

This guide covers contributions to the `repo` CLI in this repository. It is aimed at Rust developers extending the CLI, its built-in commands, or its embedded default assets.

## Before You Change Code

1. Read the project overview in [`README.md`](./README.md).
2. Review the implementation layout in [`docs/architecture.md`](./docs/architecture.md).
3. Check the existing crate-level usage notes in [`crates/repo-cli/README.md`](./crates/repo-cli/README.md).

## Local Workflow

### Build

```bash
cargo build --manifest-path crates/repo-cli/Cargo.toml
```

### Run the CLI

```bash
cargo run --manifest-path crates/repo-cli/Cargo.toml -- --help
```

### Run a focused command

```bash
cargo run --manifest-path crates/repo-cli/Cargo.toml -- docs
cargo run --manifest-path crates/repo-cli/Cargo.toml -- health
cargo run --manifest-path crates/repo-cli/Cargo.toml -- skills
```

### Enable pre-commit hooks

```bash
./scripts/setup-git-hooks.sh
```

This configures Git to use the repository-managed hooks in `.githooks/`.

The repository currently enforces:

- `pre-commit` for Clippy and test failures
- `pre-push` for the coverage threshold

### Check coverage

```bash
cargo llvm-cov --version || cargo install cargo-llvm-cov --locked
./scripts/check-coverage.sh
```

The current repository policy is line coverage above 90%, enforced as a minimum of 91%.

## Contribution Expectations

### Code

- Keep command behavior explicit and unsurprising.
- Prefer small parsing helpers over deeply nested command handlers.
- Avoid introducing panics on user-controlled inputs.
- Preserve existing plain-text CLI output style unless there is a strong reason to change it.

### Documentation

- Update user-facing docs when adding or changing commands.
- Keep examples runnable against the current CLI surface.
- Put long-form explanation or reference content in `docs/` rather than crowding the root README.

### Embedded Defaults

Defaults under `crates/repo-cli/defaults/` are part of the product surface.

When changing them:

- keep generated output stable and intentional
- avoid overwriting user-managed files by default
- verify that any `include_str!` path still resolves correctly

## Suggested Change Process

1. Identify the command or module you are changing.
2. Update the implementation.
3. Build the crate with `cargo build` or `cargo check`.
4. Run the relevant command help and at least one realistic command path.
5. Ensure the pre-commit gates pass:

```bash
cargo clippy --manifest-path crates/repo-cli/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path crates/repo-cli/Cargo.toml --all-features
```

6. Ensure the coverage threshold passes:

```bash
./scripts/check-coverage.sh
```

7. Update documentation and examples in the same change.

## Areas That Need Extra Care

- `crates/repo-cli/src/plugin/builtin/docs.rs`
- `crates/repo-cli/src/plugin/builtin/health.rs`
- `crates/repo-cli/src/plugin/builtin/skills.rs`

These files currently combine CLI parsing, filesystem access, rendering, and business logic. Changes there should stay disciplined to avoid making them harder to split later.

## Testing Status

This repository has crate-level automated tests and an enforced coverage threshold.

Manual command-path validation is still expected for CLI changes because passing unit tests do not fully cover user-facing command behavior.

When a change affects CLI output, filesystem interactions, or embedded defaults:

- run the affected command paths manually
- prefer incremental changes with clear behavior boundaries
- add or extend automated tests where practical

## Pull Request Checklist

- The crate builds successfully.
- Help text remains accurate.
- Examples in docs still match the code.
- New user-visible behavior is documented.
- New files and defaults use ASCII unless there is a strong reason not to.

## What To Document In PRs

Include:

- what changed
- why it changed
- how you validated it
- any known limitations or follow-up work
