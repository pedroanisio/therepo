# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project is intended to follow Semantic Versioning.

## [0.4.0-rc-3] - 2026-04-01

### Added

- `drift-risk-map.skill` — new built-in ZIP skill for documentation drift risk analysis
- built-in skill count raised to 14 (was 13)

### Fixed

- ZIP skill extraction now handles subdirectory layouts (e.g. `<name>/SKILL.md`) via `find_skill_md_in_zip` helper, fixing deploy failures for `codebase-requirements` and `response-dispatch`
- `repo skills deploy` now succeeds with 0 failures; deploy tests updated from expecting partial failure to expecting full success
- removed dead `bundled_skill_name` function

## [0.4.0-rc-2] - 2026-04-01

### Added

- `response-dispatch.skill` — new built-in ZIP skill for response dispatch patterns
- `audit-repo.md` — new built-in prompt for codebase comparative analysis and inspection
- built-in skill count raised to 13 (was 12), prompt count to 7 (was 6)

## [0.4.0-rc-1] - 2026-04-01

### Added

- `repo health fix` subcommand that auto-fixes failed custom checks with a `fix_cmd` field
- `fix_cmd` field on custom health checks, supporting shell commands and `builtin:` handlers
- embedded CLAUDE.md and DISCLAIMER.md seed templates under `defaults/docs/`, copyable via `builtin:copy-doc`
- default health checks for CLAUDE.md, DISCLAIMER.md, and README disclaimer reference in the `health init` template
- `AGENTS.md` — complete `repo` CLI reference for AI agents, wired into `CLAUDE.md` reading order and `README.md` documentation map
- `PURPOSE.md` — project purpose document following the Golden Circle framework
- `repo skills init` now copies all 12 built-in skills (was 6), all 8 references (was 6), and both schemas (was 1)
- ZIP-packaged skill extraction in `repo skills init` for `cli-ux-patterns` and `codebase-requirements`
- 6 new `progress.rs` unit tests covering finish-with-state, drop cleanup, double-finish safety, and is_enabled

### Fixed

- coverage threshold default in `scripts/check-coverage.sh` changed from 75 to 91, matching the ADR policy
- ADR implementation notes now document the actual variable-based script instead of a hardcoded command
- skills count corrected from "10" to "12" in `quickstart.md`, `how-to-bootstrap-repo-metadata.md`, help text, and code comments
- crate README skills table expanded from 5 to all 12 built-in skills
- root README directory tree expanded to reflect actual repository layout
- `docs/architecture.md` now documents the `commands/` dispatch layer, `progress.rs`, `lib.rs`, and `defaults/examples/` and `defaults/scripts/`
- disclaimer references added to root README and crate README per project guidelines

## [0.3.0] - 2026-03-31

### Added

- richer `repo docs` browsing with document queries, sorting, limits, phase-detail modes, and interactive selection
- repo-local prompt templates for corpus assessment, feedback processing, plan formatting, and review workflows under `.repo/prompts/`
- `PURPOSE.md`, a doc hygiene audit report, and a quarantine helper script for repository documentation maintenance
- machine-readable JSON output for `repo completions`, `repo prompt init`, `repo health init`, `repo health export`, `repo ulid`, and `repo skills deploy`

### Changed

- built-in plugin discovery now reports `health` and `skills` consistently alongside the updated docs command surface
- architecture, release, and repository-bootstrap docs now describe the expanded `repo docs` filtering and sorting workflow
- root and crate READMEs now include disclaimer context, expanded repository layout details, and the full built-in skills inventory
- the default coverage gate is now 91% in `scripts/check-coverage.sh`, matching the ADR and contributor documentation

### Fixed

- regression coverage now protects global `--json`, targeted plan inspection, incomplete plan detail rendering, and `skills install` failure handling
- `--plain` now suppresses spinner output instead of only disabling ANSI color
- built-in command handlers now return consistent exit codes instead of terminating from deep library code paths
- `repo skills deploy` now exits non-zero on partial deployment failures and reports per-skill outcomes in JSON mode

## [0.2.0] - 2026-03-22

### Added

- clap-based command-line parsing with explicit subcommands and shared global flags
- JSON output support for `repo skills` and the top-level documentation overview
- progress spinner feedback for longer-running skill operations
- recommendation fields and follow-up guidance in `repo health` reports
- release automation, installer scripts, and starter `cargo-dist` configuration
- root-level project docs for quickstart, bootstrapping repository metadata, architecture, releasing, and contributing
- ADRs covering pre-commit quality gates and the test coverage threshold
- expanded automated coverage for CLI flows, health checks, docs output, prompt and ULID commands, and skills synchronization

### Changed

- plugin manifest handling and built-in asset packaging to support the current plugin discovery model
- repository documentation layout so project docs live at the root while crate-specific usage stays under `crates/repo-cli/`
- README and CLI reference guidance to clarify coverage expectations and the install-once, run-anywhere workflow for `repo`

### Fixed

- `repo skills install` now exits non-zero when required skills cannot be installed

## [0.1.0] - 2026-03-21

### Added

- initial Rust CLI implementation under `crates/repo-cli`
- built-in commands for `docs`, `health`, `skills`, `prompt`, `ulid`, and `plugins`
- embedded default assets for prompts, skills, schemas, references, and traits
- plugin discovery for built-in and repository-local plugins
