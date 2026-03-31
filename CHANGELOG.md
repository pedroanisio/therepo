# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project is intended to follow Semantic Versioning.

## [Unreleased]

### Added

- top-level project documentation set
- contributor workflow guide
- CLI command reference
- architecture overview
- richer `repo docs` browsing with document queries, sorting, limits, phase-detail modes, and interactive selection
- built-in plugin registration for `health` and `skills` in plugin discovery output
- regression coverage for global `--json`, targeted plan inspection, incomplete plan detail rendering, and `skills install` failure handling

### Changed

- repository documentation is now split between root docs and crate-level docs
- global `--json` now propagates through `repo docs`, including the top-level docs overview
- host-tool documentation now makes clear that `repo` is installed once and manages per-repository `.repo/` metadata

### Fixed

- `repo skills install` now exits non-zero when required skills cannot be installed

## [0.1.0]

### Added

- initial Rust CLI implementation under `crates/repo-cli`
- built-in commands for `docs`, `health`, `skills`, `prompt`, `ulid`, and `plugins`
- embedded default assets for prompts, skills, schemas, references, and traits
- plugin discovery for built-in and repository-local plugins
