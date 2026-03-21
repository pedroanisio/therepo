# Quick Start

This guide is for developers who want to install the `repo` CLI from this codebase and use it inside another repository.

## Prerequisites

- Rust toolchain with `cargo`

## Install The Binary

From this codebase:

```bash
cargo install --path crates/repo-cli
```

If you prefer not to install globally, build it locally instead:

```bash
cargo build --manifest-path crates/repo-cli/Cargo.toml
```

## Move To Your Repository

```bash
cd /path/to/your-repository
```

## Run The Help Command

If installed:

```bash
repo --help
```

If built locally only:

```bash
/path/to/repo-source/target/debug/repo --help
```

You should see the built-in command list:

- `docs`
- `health`
- `skills`
- `prompt`
- `ulid`
- `plugins`

## Try The Main Flows

### Show the repository overview

```bash
repo
```

### Inspect documentation state

```bash
repo docs
```

### Check environment health

```bash
repo health
```

### Initialize prompts

```bash
repo prompt init
```

### Initialize skills metadata

```bash
repo skills init
```

## What Gets Created

Some commands create or inspect metadata under your repository's `.repo/` directory:

- `.repo/config.toml`
- `.repo/health.toml`
- `.repo/skills.toml`
- `.repo/prompts/`
- `.repo/plugins/`
- `.repo/storage/`

The `docs` command also reads `_docs/` when present.

## When Something Looks Wrong

- Run `repo --help` to confirm the installed binary is available.
- If you built locally, run `/path/to/repo-source/target/debug/repo --help` to confirm the binary is runnable.
- Run subcommand help directly, for example `repo health --help`.
- If `health --check-updates` is used, network access is required for registry checks.
