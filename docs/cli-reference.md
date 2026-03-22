# CLI Reference

This document describes the installed `repo` CLI as it exists today. Run the binary inside the repository you want to inspect or manage.

## Top-Level Command

```text
repo [OPTIONS] [COMMAND]
```

### Global Options

- `--plain` disables ANSI styling in human-readable output
- `--json` emits machine-readable output where supported
- `-h`, `--help` prints help
- `-V`, `--version` prints the version

### Commands

- `docs`
- `health`
- `skills`
- `prompt`
- `ulid`
- `plugins`
- `completions`

### Common Examples

```bash
repo
repo --json
repo docs designs --status accepted
repo health --verbose --json
repo prompt list --tag review --json
repo skills sync --json
repo completions zsh
```

## `repo docs`

Browse repository documentation and stored plans.

```text
repo docs [OPTIONS] [COMMAND]
```

### Subcommands

- `plans`
- `designs`
- `adrs`
- `references`
- `refs` as an alias for `references`

### Command-Specific Options

- `--status <STATUS>` filters listed documents by status
- `--json` emits machine-readable output for the selected doc list

### Notes

- `plans` reads from `.repo/storage/`
- the other kinds read from `_docs/<kind>/`
- when no subcommand is given, `repo docs` shows an overview of all doc kinds

### Examples

```bash
repo docs
repo docs plans --json
repo docs designs --status accepted
repo docs refs
```

## `repo health`

Validate the local development environment and repo setup.

```text
repo health [OPTIONS] [COMMAND]
```

### Subcommands

- `init`
- `export`

### Command-Specific Options

- `-u`, `--check-updates`
- `-v`, `--verbose`

### Notes

- when `.repo/health.toml` exists, `repo health` validates against declared requirements
- without that file, the command performs a best-effort scan
- `--json` returns a structured report including sections, check status, details, and recommendations
- update checks may query external registries and tooling

### Examples

```bash
repo health
repo health --verbose
repo health --check-updates --json
repo health init
repo health export
```

## `repo skills`

Manage declared agent skills and related built-in assets.

```text
repo skills [OPTIONS] [COMMAND]
```

### Subcommands

- `init`
- `export`
- `sync`
- `install`
- `fix`
- `deploy`

### Notes

- declarations live in `.repo/skills.toml`
- `init` copies built-in assets into `.repo/skills/`, `.repo/references/`, and `.repo/schemas/`
- `install` delegates to `npx skills add` for skills declared in `.repo/skills.toml`
- `export`, `sync`, `check` via default invocation, `install`, and `fix` support `--json`
- `deploy` installs built-in skills into `~/.agents/skills/` and creates agent-specific symlinks for detected agents
- use `repo skills deploy --force` to overwrite existing installs

### Examples

```bash
repo skills
repo skills --json
repo skills sync --json
repo skills install
repo skills deploy --force
```

## `repo prompt`

List, show, and materialize reusable prompt snippets.

```text
repo prompt [OPTIONS] [COMMAND]
```

### Subcommands

- `list`
- `init`
- `<name>` to print a named prompt

### Command-Specific Options

- `--tag <TAG>` filters prompt listings

### Notes

- built-in prompts are embedded at compile time
- prompts in `.repo/prompts/` override built-ins by name
- `repo prompt list --json` emits machine-readable prompt data

### Examples

```bash
repo prompt
repo prompt list --tag review --json
repo prompt format-plan
repo prompt init
```

## `repo ulid`

Generate one or more ULIDs.

```text
repo ulid [-n <N>]
```

### Options

- `-n`, `--count <N>` sets how many ULIDs to generate

### Examples

```bash
repo ulid
repo ulid -n 3
```

## `repo plugins`

List discovered plugins and inspect individual plugin metadata.

```text
repo plugins [OPTIONS] [COMMAND]
```

### Subcommands

- `list`
- `info <NAME>`

### Notes

- built-in plugins are reported by the binary
- external plugins are discovered from `.repo/plugins/`
- `repo plugins --json` emits a machine-readable plugin list
- `repo plugins info <NAME> --json` emits machine-readable plugin metadata
- external plugin execution is not implemented yet

### Examples

```bash
repo plugins
repo plugins --json
repo plugins info docs
repo plugins info docs --json
```

## `repo completions`

Generate shell completion scripts on demand.

```text
repo completions <bash|elvish|fish|powershell|zsh>
```

### Examples

```bash
repo completions zsh
repo completions bash
```

## Exit Behavior

Commands typically exit with a non-zero status on:

- unknown subcommands
- invalid required arguments
- blocking file parsing failures
- validation failures such as missing required tools or missing declared skills

Some initialization paths are intentionally non-destructive and skip files that already exist.
