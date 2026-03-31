# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project is intended to follow Semantic Versioning.

## [Unreleased]

### Added

- No unreleased changes yet.

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
