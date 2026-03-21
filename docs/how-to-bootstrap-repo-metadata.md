# How To Bootstrap Repository Metadata

This guide is for maintainers who have already installed the `repo` binary and want to initialize the metadata it expects inside a target project.

## Goal

Create the common `.repo/` files and directories used by the CLI.

## Steps

### 1. Initialize health configuration

```bash
repo health init
```

This creates `.repo/health.toml` if it does not already exist.

### 2. Initialize skills metadata and built-in assets

```bash
repo skills init
```

This creates:

- `.repo/skills.toml`
- `.repo/skills/`
- `.repo/references/`
- `.repo/schemas/`

Existing files are not overwritten.

### 3. Initialize prompt snippets

```bash
repo prompt init
```

This writes built-in prompt snippets into `.repo/prompts/` without replacing files that are already present.

### 4. Review the generated files

At minimum, inspect:

- `.repo/health.toml`
- `.repo/skills.toml`
- `.repo/prompts/`

Adjust them to match the actual repository standards and workflow.

## Notes

- `.repo/storage/` is created automatically when the CLI runs.
- `_docs/` is not initialized by a dedicated command yet; create it manually if you want to organize plans, designs, ADRs, and references there.
- External plugin discovery looks in `.repo/plugins/`, but external plugin execution is not complete yet.
