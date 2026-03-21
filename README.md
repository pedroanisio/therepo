# repo

`repo` is a Rust CLI for repository maintenance workflows.

The source code for the binary lives in [`crates/repo-cli/`](./crates/repo-cli/), but the intended usage model is simple:

1. install or build the `repo` binary here
2. run that binary inside the repository you want to manage

## What It Does

`repo` standardizes repository-local operational metadata and maintenance workflows:

- `.repo/config.toml` for repository configuration
- `.repo/health.toml` for environment requirements
- `.repo/skills.toml` for declared agent skills
- `.repo/prompts/` for reusable prompt snippets
- `.repo/plugins/` for repository-local plugins
- `_docs/` for plans, designs, ADRs, and references

It also provides built-in commands for working with those directories directly.

## Current Status

> [!IMPORTANT]
> The built-in commands are usable, but external plugin execution is not fully implemented yet. External plugins can be discovered, but dispatch is still incomplete.

## Quick Start

### Prerequisites

- Rust toolchain with `cargo`

### Install

```bash
cargo install --path crates/repo-cli
```

### Or Build Locally

```bash
cargo build --manifest-path crates/repo-cli/Cargo.toml
```

### Use It In Your Repository

After installation, change into the repository you want to manage and run:

```bash
repo --help
repo
repo docs
repo health
repo prompt init
repo skills init
```

If you only built it locally, invoke the compiled binary from inside that repository:

```bash
/path/to/repo-source/target/debug/repo --help
```

`repo` operates on the current working tree.

## Command Surface

The current built-in commands are:

- `docs` for listing plans, designs, ADRs, and references
- `health` for validating environment state and tool versions
- `skills` for declaring and syncing agent skills
- `prompt` for listing and materializing prompt snippets
- `ulid` for generating valid ULIDs
- `plugins` for listing built-in and external plugins

See the full command reference in [`docs/cli-reference.md`](./docs/cli-reference.md).

## Documentation Map

This repository now keeps its project documentation split by purpose:

- [`docs/quickstart.md`](./docs/quickstart.md): install and first-run workflow
- [`docs/how-to-bootstrap-repo-metadata.md`](./docs/how-to-bootstrap-repo-metadata.md): repository setup recipe
- [`docs/cli-reference.md`](./docs/cli-reference.md): command reference
- [`docs/architecture.md`](./docs/architecture.md): code structure and design notes
- [`docs/releasing.md`](./docs/releasing.md): SemVer, publishing, and installer policy
- [`CONTRIBUTING.md`](./CONTRIBUTING.md): contributor workflow for this codebase
- [`CHANGELOG.md`](./CHANGELOG.md): notable project changes

The crate-specific README remains at [`crates/repo-cli/README.md`](./crates/repo-cli/README.md).

## Repository Layout

```text
.
├── crates/
│   └── repo-cli/
│       ├── src/
│       └── defaults/
├── docs/
├── CONTRIBUTING.md
├── CHANGELOG.md
└── README.md
```

## Development Notes

- The project is a single binary crate today.
- Built-in command implementations are concentrated in `crates/repo-cli/src/plugin/builtin/`.
- The `docs`, `health`, and `skills` modules currently contain most of the project logic.
- There is no automated test suite yet.

## Next Areas To Improve

- add automated tests around parsing and command behavior
- complete external plugin dispatch
- split large built-in modules into smaller units
- add a workspace-level Cargo manifest if the project grows beyond one crate
