---
disclaimer:
  notice: >-
    No information within this document should be taken for granted.
    Any statement or premise not backed by a real logical definition
    or verifiable reference may be invalid, erroneous, or a hallucination.
  generated_by: "Claude Opus 4.6 via Claude Code"
  date: "2026-03-31"
---

# AGENTS.md — repo CLI Reference for AI Agents

This document enables any AI agent (Claude Code, Codex, Copilot, or custom)
to discover, bootstrap, and operate the `repo` CLI in any repository.

**Priority:** Read this file before running any `repo` command. It is the
canonical reference for programmatic CLI usage.

---

## 1. What `repo` Is

`repo` is a Rust CLI that standardizes repository maintenance metadata and
workflows. It operates on the current working directory — no server, no
cloud, no accounts. One binary manages any number of repositories.

**Binary name:** `repo`
**Global flags (available on every command):**

| Flag | Effect |
|------|--------|
| `--json` | Emit machine-readable JSON instead of human-readable output |
| `--plain` | Disable ANSI styling (useful when capturing output) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

**Always use `--json` when parsing output programmatically.**

---

## 2. Bootstrap a New Repository

Run these commands in order inside the target repository:

```bash
# 1. Initialize environment health requirements
repo health init

# 2. Initialize skills metadata and copy built-in assets
repo skills init

# 3. Deploy skills to agent directories (~/.agents/skills/)
repo skills deploy

# 4. Initialize prompt snippets
repo prompt init

# 5. Verify everything
repo --json
```

This creates:

```
.repo/
  config.toml           # Repository configuration (auto-created)
  health.toml           # Tool versions and custom checks
  skills.toml           # Declared agent skills
  skills/               # Built-in skill definitions (12 skills)
  references/           # Reference documents (8 files)
  schemas/              # Formal schemas (2 files)
  prompts/              # Reusable prompt snippets (6 prompts)
  storage/              # Internal storage (auto-created)
  plugins/              # External plugins directory
```

---

## 3. Command Reference

### 3.1 `repo` (no arguments) — Repository Overview

Shows a summary of the repository state: docs counts, plugin counts, config status.

```bash
repo              # human-readable
repo --json       # machine-readable
```

**JSON output shape:**
```json
{
  "name": "project-name",
  "docs": { "plans": 2, "designs": 1, "adrs": 0, "references": 2 },
  "plugins": { "builtin": 7, "external": 0 },
  "config": ".repo/config.toml"
}
```

---

### 3.2 `repo docs` — Browse Documentation

Navigates documents in `_docs/` (designs, ADRs, references) and `.repo/storage/` (plans).

```bash
repo docs                                  # Overview of all doc kinds
repo docs plans                            # List plans with progress tracking
repo docs plans --json                     # Machine-readable plan listing
repo docs plans <query>                    # Show details for a specific plan
repo docs designs                          # List design documents
repo docs designs --status accepted        # Filter by status
repo docs adrs                             # List Architecture Decision Records
repo docs references                       # List reference documents (alias: refs)
```

**Subcommands:** `plans`, `designs`, `adrs`, `references` (alias: `refs`)

**Flags on listing subcommands:**

| Flag | Effect |
|------|--------|
| `<query>` | Filter by filename, stem, or title prefix |
| `--status <STATUS>` | Filter by frontmatter status |
| `--sort <date\|status\|title\|progress>` | Sort results |
| `--limit <N>` | Limit number of results |
| `--details <none\|incomplete\|all>` | Expand phase details |
| `--interactive` | Interactively choose a document |
| `--json` | Emit JSON |

**Document format:** Markdown files with YAML frontmatter:
```yaml
---
title: "My Document"
version: "0.1.0"
status: "proposal"     # draft | proposal | accepted | active | deprecated
date: "2026-03-18"
---
```

Plans with `## Phase N` headings and `- [x]`/`- [ ]` checkboxes get
automatic progress tracking.

---

### 3.3 `repo health` — Environment & Repository Health

Validates tools, versions, configuration, and custom checks against `.repo/health.toml`.

```bash
repo health                    # Run all checks
repo health --verbose          # Also show tools not present
repo health --check-updates    # Query registries for newer versions
repo health --json             # Machine-readable health report
repo health init               # Create blank .repo/health.toml template
repo health export             # Snapshot current environment into .repo/health.toml
```

**Flags:**

| Flag | Effect |
|------|--------|
| `-v, --verbose` | Show optional tools that are not installed |
| `-u, --check-updates` | Query npm/rustup/etc. for newer versions |
| `--json` | Emit JSON report |

**What it checks:**
- **Tools:** git, rustc, cargo, clippy, rustfmt, node, npm, pnpm, bun, skills, python, pip, uv, docker, make, cmake, go, java, zsh, bash
- **Version constraints:** `min_version` and `exact_version` from config
- **Repository state:** git branch, `.repo/config.toml`, `_docs/` structure, Python virtualenv, Rust toolchain
- **Environment:** runtime cage (host/docker/podman/lxc/wsl/kubernetes), privilege escalation, shells
- **Custom checks:** arbitrary shell commands as assertions (defined in `.repo/health.toml`)

**`.repo/health.toml` structure:**
```toml
[environment]
privilege = "auto"           # "auto", "sudo", "doas", "pkexec", "none"
allowed_runtimes = []        # "host", "docker", "podman", "lxc", "wsl", "kubernetes"
# required_shell = "zsh"

[tools.node]
required = true
min_version = "18.0.0"
url = "https://nodejs.org/en/download"
install = "curl -fsSL https://fnm.vercel.app/install | bash && fnm install --lts"

[checks.claude-md]
command = "test -f CLAUDE.md"
description = "CLAUDE.md exists at repo root"
severity = "error"           # "error" = fail check, "warning" = note
hint = "Create a CLAUDE.md with project instructions"
```

---

### 3.4 `repo skills` — Agent Skill Management

Manages the skills AI agents need to operate on this repository.

```bash
repo skills                    # Check installed vs. declared
repo skills --json             # Machine-readable check
repo skills init               # Create .repo/skills.toml + copy built-in assets
repo skills export             # Snapshot installed skills into .repo/skills.toml
repo skills sync               # Merge installed skills into config (preserves edits)
repo skills sync --json        # Machine-readable sync report
repo skills install            # Install missing skills via `npx skills add`
repo skills install --json     # Machine-readable install report
repo skills fix                # Remove unfixable entries from config
repo skills deploy             # Deploy skills to ~/.agents/skills/
repo skills deploy --force     # Overwrite existing skill files
```

**Subcommands:**

| Command | Effect |
|---------|--------|
| (none) | Check installed vs. declared |
| `init` | Create `.repo/skills.toml` + copy built-in assets to `.repo/` |
| `export` | Overwrite config with what is on disk (fresh start) |
| `sync` | Merge — preserves source/agents/scope edits, adds new, removes missing |
| `install` | Install missing skills via `npx skills add` |
| `fix` | Remove entries with no source or unresolvable |
| `deploy` | Copy built-in skills to `~/.agents/skills/` and symlink for all detected agents |

**`deploy` flag:** `--force` / `-f` — overwrite already-installed skills

**12 built-in skills:**

| Skill | Description |
|-------|-------------|
| tsdoc-voice | Enforce TSDoc voice guide |
| mental-model | Build structured mental models before planning |
| adv-planning | Generate formal PlanSchema JSON execution plans |
| purpose-md | Create/review PURPOSE.md files |
| testing-standards | Enforce testing coverage, TDD, and quality standards |
| incremental-validation | Continuous validation after every change |
| review-plan | Inspect, approve, or improve PlanSchema JSON plans |
| prompt-builder | Design valid PromptDocument artifacts |
| behavioral-layer | Define behavioral traits for agent reasoning |
| doc-hygiene | Discover, classify, and manage documentation |
| cli-ux-patterns | CLI UX patterns in Rust with clap |
| codebase-requirements | Generate comprehensive REQUIREMENTS.md |

**`.repo/skills.toml` structure:**
```toml
[[skills]]
name = "executing-plans"
source = "https://github.com/obra/superpowers"
skill = "executing-plans"
agents = []                    # empty = all detected agents
scope = "project"              # "project" or "global"
description = "Structured plan execution"
```

---

### 3.5 `repo prompt` — Prompt Snippets

Reusable prompt templates for AI-assisted workflows.

```bash
repo prompt                          # List prompts (human-readable)
repo prompt --json                   # Machine-readable listing
repo prompt list                     # List all available prompts
repo prompt list --tag review        # Filter by tag
repo prompt list --tag review --json # Filtered listing as JSON
repo prompt <name>                   # Print prompt body to stdout
repo prompt format-plan              # Example: print the format-plan prompt
repo prompt init                     # Write built-in defaults to .repo/prompts/
```

**Flags:**

| Flag | Effect |
|------|--------|
| `--tag <TAG>` | Filter prompts by tag |
| `--json` | Emit JSON |

**6 built-in prompts:**

| Name | Tags | Purpose |
|------|------|---------|
| assess-corpus | review, assess, corpus | Formal document assessment against reference corpus |
| feedback-processor | feedback, review, process | Evaluate feedback as claims before acting |
| format-plan | plan, format, refactor, markdown | Format into phased markdown plans with progress |
| review-cycle | review, verify, reference | Formal verification against a reference corpus |
| review-internal | review, verify, coherence | Verify internal document coherence |
| validate-plan | plan, validate, review, compliance | Validate plan completeness and compliance |

**Usage pattern for agents:** Retrieve a prompt and use it as system context:
```bash
PROMPT=$(repo prompt format-plan --plain)
# Use $PROMPT as instructions for the current task
```

---

### 3.6 `repo ulid` — Generate ULIDs

```bash
repo ulid               # Generate 1 ULID
repo ulid -n 5          # Generate 5 ULIDs
repo ulid --json        # Machine-readable output
```

**Flag:** `-n, --count <N>` — number of ULIDs (default: 1)

---

### 3.7 `repo plugins` — Plugin Discovery

```bash
repo plugins             # List all discovered plugins
repo plugins list        # Same as above
repo plugins info <name> # Show details about a specific plugin
repo plugins --json      # Machine-readable listing
```

**7 built-in plugins:** docs, health, skills, prompt, ulid, plugins, completions

External plugins are discovered from `.repo/plugins/<name>/plugin.toml`.

---

### 3.8 `repo completions` — Shell Completions

```bash
repo completions bash
repo completions zsh
repo completions fish
repo completions powershell
repo completions elvish
```

Outputs completion script to stdout. Redirect to the appropriate file for your shell.

---

## 4. Agent Workflows

### 4.1 First-Time Setup (run once per repository)

```bash
repo health init
repo skills init
repo skills deploy
repo prompt init
```

### 4.2 Environment Validation (run at start of each session)

```bash
repo health --json
```

Parse the JSON output. If any check has `"status": "fail"`, resolve before proceeding.

### 4.3 Skill Synchronization (after installing or removing skills)

```bash
repo skills sync --json
```

### 4.4 Find a Prompt for the Current Task

```bash
repo prompt list --json
```

Then retrieve the relevant prompt body:
```bash
repo prompt <name> --plain
```

### 4.5 Browse Project Documentation

```bash
# Get structured overview
repo docs --json

# Find a specific plan
repo docs plans <query> --json

# List accepted designs
repo docs designs --status accepted --json
```

### 4.6 Generate IDs for New Documents

```bash
repo ulid
```

Use the output as a filename prefix for new plans, ADRs, or references.

---

## 5. JSON Output Patterns

Every command that supports `--json` emits valid JSON to stdout. Errors go to
stderr. Exit code 0 means success; non-zero means failure.

**Agent rule:** Always use `--json --plain` when parsing output. `--plain`
prevents ANSI escape codes from contaminating the output.

---

## 6. Bundled Assets (deployed by `repo skills init`)

### Skills (12)
Copied to `.repo/skills/` — agent capability definitions.

### References (8)
Copied to `.repo/references/` — spec documents for schemas and patterns.

| Reference | Topic |
|-----------|-------|
| tsdoc-spec | TSDoc specification |
| mental-model-schema | Mental model JSON structure |
| plan-schema-fields | PlanSchema v0.3.0 field reference |
| schema-reference | Prompt schema field constraints |
| trait-spec | Behavioral trait specification |
| detection-patterns | Documentation discovery heuristics |
| report-template | Doc-hygiene report output structure |
| sync-checks | Doc validation checks |

### Schemas (2)
Copied to `.repo/schemas/` — formal TypeScript schema definitions.

| Schema | Defines |
|--------|---------|
| plan-schema.ts | PlanSchema v0.3.0 |
| prompt-schema.ts | PromptDocument v0.3.0 |

### Prompts (6)
Copied to `.repo/prompts/` — reusable prompt snippets (see section 3.5).

---

## 7. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Check failed or command error |
| 2 | CLI usage error (bad arguments) |

---

## 8. Detecting `repo` Availability

Before running any command, verify the binary is available:

```bash
command -v repo >/dev/null 2>&1 && repo --version
```

If not installed, install from source:

```bash
cargo install --path crates/repo-cli
```

Or download a release binary from the GitHub releases page.
