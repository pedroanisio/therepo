# Releasing

This document defines the release process for `therepo` and the SemVer rules for the installed `repo` binary.

## Package And Binary Names

- crates.io package: `therepo`
- installed binary: `repo`

The package name is for distribution. The binary name is the user-facing command. Users install it once and run it inside any repository they want to manage.

## Versioning Policy

This project follows Semantic Versioning.

For this CLI, the public interface includes:

- command names and flags
- machine-consumable output formats
- `.repo/` file formats
- `_docs/` scanning conventions
- plugin manifest expectations

### Patch Releases

Use `x.y.Z` for backward-compatible fixes:

- bug fixes
- wording and help-text corrections
- safer error handling
- non-breaking internal refactors

### Minor Releases

Use `x.Y.0` for backward-compatible additions:

- new subcommands
- new optional flags
- new default assets that do not break existing usage
- additive config fields

### Major Releases

Use `X.0.0` for breaking changes:

- removing commands or flags
- changing file formats incompatibly
- changing JSON output structures relied on by users or scripts
- changing plugin conventions incompatibly

## Pre-Publish Checklist

1. Update `CHANGELOG.md`.
2. Ensure the version in `crates/repo-cli/Cargo.toml` is correct.
3. Run `cargo check --manifest-path crates/repo-cli/Cargo.toml`.
4. Run `cargo package --manifest-path crates/repo-cli/Cargo.toml --list`.
5. Run `cargo publish --manifest-path crates/repo-cli/Cargo.toml --dry-run`.
6. Run `./scripts/check-coverage.sh`.
7. Verify `scripts/install.sh` still matches your release asset naming.

The same coverage command is enforced in CI and by the repository-managed `pre-push` hook.

## Publishing To crates.io

```bash
cargo login
cargo publish --manifest-path crates/repo-cli/Cargo.toml
```

## Release Artifacts

Recommended release assets:

- source crate on crates.io
- Linux tarballs
- macOS tarballs
- Windows zip archive
- shell installer
- PowerShell installer

## Installer Script

The repository includes an installer template at [`scripts/install.sh`](../scripts/install.sh).

It expects GitHub release assets named like:

- `therepo-x86_64-unknown-linux-gnu.tar.gz`
- `therepo-aarch64-apple-darwin.tar.gz`

The GitHub release workflow renders and uploads a release-ready installer as `therepo-installer.sh` with the correct GitHub repository slug baked in.

For local testing before that, set the repository slug yourself:

```bash
export THEREPO_REPO=pedroanisio/therepo
./scripts/install.sh
```

## Recommended Next Step

The repository also includes [`dist-workspace.toml`](../dist-workspace.toml) with a starting `cargo-dist` configuration for:

- GitHub CI integration
- shell installers
- PowerShell installers
- common desktop targets

The repository and homepage metadata now point to:

- `https://github.com/pedroanisio/therepo`

The next step is to run `dist generate` and refine the generated automation if you want to fully hand off release packaging to `cargo-dist`.
