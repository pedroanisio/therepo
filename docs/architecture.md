# Architecture

This document explains how the `repo` CLI is structured today.

## High-Level Shape

The project is currently a single Rust binary crate:

- `crates/repo-cli`

The crate compiles one executable:

- `repo`

## Entry Point

The binary entry point is `crates/repo-cli/src/main.rs`.

It is responsible for:

- parsing top-level arguments
- locating the repository root
- ensuring `.repo/storage/` exists
- dispatching built-in subcommands
- attempting to match external plugin commands

## Core Support Modules

### `config.rs`

Loads `.repo/config.toml` into a small `RepoConfig` model and provides repository root discovery.

### `output.rs`

Provides lightweight ANSI formatting helpers and honors `NO_COLOR`.

### `plugin/`

Contains plugin discovery and plugin manifest parsing.

## Built-In Command Modules

Built-in command behavior lives under `crates/repo-cli/src/plugin/builtin/`.

### `docs.rs`

Handles:

- scanning `_docs/`
- scanning `.repo/storage/`
- parsing frontmatter
- deriving plan progress
- table and JSON output

### `health.rs`

Handles:

- environment probing
- tool version checks
- update checks
- runtime and shell detection
- custom configured checks

### `health_config.rs`

Contains the config model and snapshot/template helpers used by `health`.

### `skills.rs`

Handles:

- `.repo/skills.toml`
- built-in asset initialization
- installed-skill scanning
- export, sync, install, and fix flows

### `prompt.rs`

Handles:

- embedded prompt defaults
- prompt parsing
- prompt listing and filtering
- prompt initialization into `.repo/prompts/`

### `ulid.rs`

Handles ULID generation.

## Embedded Defaults

The crate embeds default files with `include_str!` from:

- `crates/repo-cli/defaults/prompts/`
- `crates/repo-cli/defaults/skills/`
- `crates/repo-cli/defaults/references/`
- `crates/repo-cli/defaults/schemas/`
- `crates/repo-cli/defaults/traits/`
- `crates/repo-cli/defaults/templates/`

These defaults are copied into `.repo/` by the relevant commands when needed.

## Current Constraints

The implementation is usable, but there are clear structural limits:

- command parsing and command logic are tightly coupled
- large built-in modules mix parsing, filesystem I/O, rendering, and policy
- plugin discovery exists, but external plugin execution is incomplete
- automated tests and a coverage threshold now exist, but command-path coverage is still shallow

## Recommended Refactoring Direction

If the codebase grows, the next clean split would be:

1. move pure parsing and domain logic into library-style modules
2. keep CLI argument handling thin
3. separate rendering from scanning/parsing logic
4. add tests around parsing and command behavior before larger feature work
