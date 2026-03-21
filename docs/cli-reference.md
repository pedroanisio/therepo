# CLI Reference

This document is a command reference for the installed `repo` CLI. Run the binary from inside the repository you want to inspect or manage.

## Top-Level Command

```text
repo [COMMAND] [OPTIONS]
```

### Commands

- `docs`
- `health`
- `skills`
- `prompt`
- `ulid`
- `plugins`

## `repo docs`

Browse repository documentation and stored plans.

```text
repo docs [COMMAND] [OPTIONS]
```

### Subcommands

- `plans`
- `designs`
- `adrs`
- `references`

### Notes

- `plans` reads from `.repo/storage/`
- the other kinds read from `_docs/<kind>/`
- `references` also supports the alias `refs`
- `--status <STATUS>` filters results
- `--json` emits machine-readable output

## `repo health`

Validate the local development environment.

```text
repo health [COMMAND] [OPTIONS]
```

### Subcommands

- `init`
- `export`

### Options

- `-u`, `--check-updates`
- `-v`, `--verbose`

### Notes

- When `.repo/health.toml` exists, it validates against declared requirements.
- Without that file, the command performs a best-effort scan.
- Update checks may query external registries and tooling.

## `repo skills`

Manage declared agent skills and related built-in assets.

```text
repo skills [COMMAND]
```

### Subcommands

- `init`
- `export`
- `sync`
- `install`
- `fix`

### Notes

- declarations live in `.repo/skills.toml`
- built-in assets are copied into `.repo/skills/`, `.repo/references/`, and `.repo/schemas/`
- installation delegates to `npx skills add`

## `repo prompt`

List, show, and materialize reusable prompt snippets.

```text
repo prompt [COMMAND] [OPTIONS]
```

### Subcommands

- `list`
- `init`
- `<name>`

### Options

- `--tag <TAG>`

### Notes

- built-in prompts are embedded at compile time
- prompts in `.repo/prompts/` override built-ins by name

## `repo ulid`

Generate one or more ULIDs.

```text
repo ulid [-n <N>]
```

### Options

- `-n <N>`

## `repo plugins`

List discovered plugins.

```text
repo plugins [list|info]
```

### Notes

- built-in plugins are reported by the binary
- external plugins are discovered from `.repo/plugins/`
- external plugin execution is not implemented yet
- `plugins info` is currently a placeholder

## Exit Behavior

Commands typically exit with a non-zero status on:

- unknown subcommands
- invalid required arguments
- file parsing failures that block a command

Some initialization paths are intentionally non-destructive and skip files that already exist.
