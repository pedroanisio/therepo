---
disclaimer:
  notice: >-
    No information within this document should be taken for granted.
    Any statement or premise not backed by a real logical definition
    or verifiable reference may be invalid, erroneous, or a hallucination.
  generated_by: "Claude Opus 4.6 via Claude Code"
  date: "2026-03-31"
---

# repo

## Why We Built This

Every repository accumulates operational knowledge — environment requirements, agent skills, prompt patterns, design decisions — but that knowledge lives scattered across dotfiles, wikis, tribal memory, and ad-hoc scripts. When a new contributor arrives, or when an AI agent needs to understand a project, there is no canonical place to look. The result is wasted time, repeated mistakes, and tooling that works on one machine but breaks on another.

We believe repository maintenance should be explicit, inspectable, and portable. A project's operational metadata — what tools it needs, what skills its agents require, what decisions shaped its architecture — deserves the same rigor as its source code: versioned, validated, and accessible from the command line.

## How We Approach This

- **Convention over configuration** — A single `.repo/` directory with well-known files (`config.toml`, `health.toml`, `skills.toml`) replaces ad-hoc arrangements. If you know one `repo`-managed project, you know them all.

- **Validation over trust** — `repo health` checks actual tool versions against declared requirements. `repo skills sync` compares declared skills against what is installed. The CLI reports what is real, not what someone assumed.

- **Local-first, no services** — Everything operates on the working tree. No server, no cloud dependency, no accounts. The binary and the filesystem are sufficient.

- **Extensibility through plugins, not forks** — Built-in commands handle common workflows. Repository-local plugins extend them. The plugin system is the growth path, not source modification.

- **Human and machine output** — Every command supports both human-readable terminal output and `--json` for programmatic consumption. Neither audience is an afterthought.

## What It Does

### Core Capabilities

- **Environment health checks** — Validate installed tools, versions, and project prerequisites against a declared specification.
- **Agent skill management** — Declare, install, synchronize, and deploy skills that AI agents need to operate on the repository.
- **Document browsing** — Navigate plans, ADRs, designs, and references from the terminal with filtering, sorting, and progress tracking.
- **Prompt management** — Maintain reusable prompt snippets for AI-assisted workflows.
- **Plugin discovery** — List built-in and repository-local plugins that extend the CLI.

### What This Is Not

This project does **not**:

- **Replace CI/CD** — `repo health` validates local state; it does not run builds, deploy artifacts, or gate merges.
- **Manage remote infrastructure** — It operates on the local working tree only.
- **Enforce workflow policy** — It reports status and provides tools; it does not block commits or reject pull requests on its own.
- **Provide a GUI** — It is a terminal-first tool. JSON output enables integration with other interfaces, but `repo` itself is a CLI.

## Who This Is For

- **Developers joining a project** — Run `repo health` to know immediately what is missing from their environment, instead of discovering it through build failures.
- **AI agents operating on repositories** — Use `repo skills` and `repo prompt` to discover what capabilities and patterns a project expects.
- **Maintainers standardizing workflows** — Define operational metadata once in `.repo/` and let contributors and agents discover it through the CLI.
