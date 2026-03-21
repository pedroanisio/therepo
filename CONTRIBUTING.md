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
5. Update documentation and examples in the same change.

## Areas That Need Extra Care

- `crates/repo-cli/src/plugin/builtin/docs.rs`
- `crates/repo-cli/src/plugin/builtin/health.rs`
- `crates/repo-cli/src/plugin/builtin/skills.rs`

These files currently combine CLI parsing, filesystem access, rendering, and business logic. Changes there should stay disciplined to avoid making them harder to split later.

## Testing Status

This repository currently has no automated test suite. Until tests exist:

- verify changes by building the crate
- run the affected command paths manually
- prefer incremental changes with clear behavior boundaries

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
