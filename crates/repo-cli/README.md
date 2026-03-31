# repo

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.

## Overview

A general-purpose repository maintenance CLI with a plugin system.

`repo` gives you a single entry point for browsing documentation, checking
environment health, managing agent skills, running custom validations, and
extending your workflow with plugins, in any codebase.

## Quick start

```bash
# Install from this codebase
cargo install --path crates/repo-cli

# Or build locally
cargo build --manifest-path crates/repo-cli/Cargo.toml

# Then run inside the repository you want to manage
repo --help
repo docs
```

If you only built locally, invoke the binary from there:

```bash
/path/to/repo-source/target/debug/repo --help
```

## Commands

### `repo`

With no arguments, shows a repository overview.

```
formal-schema

_docs/ overview

  plans          2 doc(s)  2 complete
  designs        1 doc(s)  1 proposal
  adrs           0 doc(s)  (empty)
  references     2 doc(s)  1 draft, 1 —

  plugins 2 built-in, 0 external
  config  .repo/config.toml
```

### `repo docs`

Browse documents organized under `_docs/`.

```bash
repo docs                         # Summary of all document kinds
repo docs plans                   # List plans (with phase progress tracking)
repo docs designs                 # List architecture/design documents
repo docs adrs                    # List ADRs
repo docs references              # List reference documents (alias: refs)
repo docs plans --status draft    # Filter by status
```

Documents are markdown files with YAML frontmatter:

```yaml
---
title: "My Plan"
version: "0.1.0"
status: "proposal"
date: "2026-03-18"
---
```

Plans with `## Phase N` headings and `- [x]`/`- [ ]` checkboxes get
automatic progress tracking.

### `repo health`

Check the development environment against `.repo/health.toml`.

```bash
repo health                   # Validate tools, versions, config, custom checks
repo health --verbose         # Also show tools that are not installed
repo health --check-updates   # Query registries for newer versions
repo health init              # Create a blank .repo/health.toml template
repo health export            # Snapshot current environment into .repo/health.toml
```

**What it checks:**

- **Tools** — git, rustc, cargo, clippy, rustfmt, node, npm, pnpm, bun,
  skills, python, pip, uv, docker, make, cmake, go, java, zsh, bash
- **Version constraints** — `min_version` (numeric semver comparison) and
  `exact_version` from config
- **Repository** — git branch, `.repo/config.toml`, `.repo/health.toml`,
  `_docs/` structure, Python virtualenv, Rust toolchain
- **Environment** — runtime cage detection (host, docker, podman, lxc, wsl,
  kubernetes), privilege escalation (sudo/doas/pkexec), available shells
- **Custom checks** — arbitrary shell commands as assertions
- **Update detection** — with `-u`, queries npm registry, `rustup check`,
  etc. for newer versions

When `.repo/health.toml` exists, validates against it. Without it,
performs a best-effort scan.

### `repo skills`

Manage required agent skills (the [skills](https://www.npmjs.com/package/skills)
ecosystem).

```bash
repo skills                   # Check installed vs. declared in .repo/skills.toml
repo skills init              # Create .repo/skills.toml + copy built-in skills, references, schemas
repo skills export            # Snapshot installed skills into .repo/skills.toml
repo skills sync              # Merge installed skills into config (keeps your edits)
repo skills install           # Install missing skills via npx skills add
```

**`init` copies built-in assets** into `.repo/`:

| Directory | Contents |
|-----------|----------|
| `.repo/skills/` | Built-in skill definitions (12 skills: tsdoc-voice, mental-model, adv-planning, purpose-md, testing-standards, incremental-validation, review-plan, prompt-builder, behavioral-layer, doc-hygiene, cli-ux-patterns, codebase-requirements) |
| `.repo/references/` | Reference documents (tsdoc-spec, mental-model-schema, plan-schema-fields) |
| `.repo/schemas/` | Formal schemas (plan-schema.ts) |

Existing files are never overwritten — run `init` again safely after updates.

**`sync` vs `export`:**

- `export` overwrites `.repo/skills.toml` with what's on disk (fresh start)
- `sync` merges — preserves your `source`, `agents`, `scope` edits, adds
  newly installed skills, removes skills no longer on disk

### `repo plugins`

List all discovered plugins (built-in and external).

```bash
repo plugins                  # or: repo plugins list
```

## Directory structure

```text
<repo-root>/
  .repo/
    config.toml               # Repo configuration
    health.toml               # Tool versions, constraints, custom checks
    skills.toml               # Required agent skills
    skills/                   # Built-in skill definitions (from repo skills init)
    references/               # Reference documents (from repo skills init)
    schemas/                  # Formal schemas (from repo skills init)
    prompts/                  # Prompt snippets (from repo prompt init)
    plugins/                  # External plugins
      my-plugin/
        plugin.toml
        check
  _docs/
    plans/                    # Plans and proposals
    designs/                  # Architecture and design documents
    adrs/                     # Architecture Decision Records
    references/               # Specs, reviews, comparisons
```

## Configuration

### `.repo/config.toml`

```toml
[repo]
name = "my-project"

[plugins]
extra_paths = []
disabled = []

[hooks]
enabled = true
timeout = 30

[check]
fail_on = "error"
```

### `.repo/health.toml`

```toml
[environment]
privilege = "sudo"                # "auto", "sudo", "doas", "pkexec", "none"
allowed_runtimes = ["host"]       # "host", "docker", "podman", "lxc", "wsl", "kubernetes"
required_shell = "zsh"

[tools.node]
required = true
min_version = "18.0.0"
url = "https://nodejs.org/en/download"
install = "curl -fsSL https://fnm.vercel.app/install | bash && fnm install --lts"
latest_cmd = "npm"
latest_args = ["view", "node", "version"]

[tools.skills]
required = true
min_version = "1.4.0"
command = "npx"
version_args = ["skills", "--version"]
url = "https://www.npmjs.com/package/skills"
install = "npm install -g skills"

[checks.claude-md]
command = "test -f CLAUDE.md"
description = "CLAUDE.md exists at repo root"
severity = "error"
hint = "Create a CLAUDE.md with project instructions for Claude Code"
```

Run `repo health init` for a fully commented template, or `repo health export`
to snapshot your current environment.

### `.repo/skills.toml`

```toml
[[skills]]
name = "executing-plans"
source = "https://github.com/obra/superpowers"
skill = "executing-plans"
agents = []                       # empty = all detected agents
scope = "project"                 # "project" or "global"
description = "Structured plan execution"
```

Run `repo skills init` for a template, `repo skills sync` to auto-populate
from installed skills.

## Default templates

Templates and built-in assets for `health init`, `skills init`, and
`prompt init` live in [defaults/](defaults/) and are embedded at compile
time via `include_str!`. Edit those files to change what gets generated.

```text
defaults/
  health.toml                 # Health check template
  skills.toml                 # Skills config template
  prompts/                    # Built-in prompt snippets
  skills/                     # Built-in skill definitions
  references/                 # Built-in reference documents
  schemas/                    # Built-in formal schemas
```

## Writing a plugin

Create a directory under `.repo/plugins/<name>/` with a `plugin.toml` manifest:

```toml
[plugin]
name = "my-check"
version = "0.1.0"
description = "Validates something useful"
provides = ["command", "validation"]

[command]
name = "my-check"
help = "Run my custom check"

[validation]
name = "my-check"
severity = "error"
```
