---
disclaimer:
  notice: >-
    This document describes this repo's instantiation of the DOCSPACES
    protocol — the docspace this repo declares, the local workflow,
    and per-docspace policy choices. The universal protocol
    specification lives at protocol/docspaces/PROTOCOL.md; this
    document is repo-specific commentary on it. Trust DOCSPACES.toml
    as the live source of truth; re-run
    ``scripts/doc_docspace_check.py --list`` for the current state.
  generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
  date: "2026-05-17"
title: "docs/open/ — DOCSPACES instantiation"
---

# `docs/open/` — DOCSPACES instantiation

This directory uses the **DOCSPACES** protocol (v1.0) — a portable
discipline for versioning documentation. A docspace is a coherent
family of documents that shares one version number; member docs opt
in and inherit.

- **Universal specification:** [`/protocol/docspaces/PROTOCOL.md`](../../protocol/docspaces/PROTOCOL.md)
- **Reference validator:** [`/protocol/docspaces/validator/`](../../protocol/docspaces/validator/) — invoked here via [`scripts/doc_docspace_check.py`](../../scripts/doc_docspace_check.py)
- **This repo's manifest:** [`DOCSPACES.toml`](DOCSPACES.toml)

The protocol is repo-agnostic; everything in this document is
repo-specific commentary on how the protocol is instantiated here.

---

## Why docspaces

Before: every satellite doc carried an independent `version:` in its
frontmatter. Bumping the primary spec's version required chasing stated
versions across every companion — and the doc-hygiene audit found 7 of 11
drifted docs drifted for exactly this reason.

After: one version per docspace, declared in [`DOCSPACES.toml`](DOCSPACES.toml).
Members inherit via `version: docspace`. Cross-docspace references name a
docspace (resolved at check-time), not a file-plus-version string.

## The docspace in this repo

Run `scripts/doc_docspace_check.py --list` for the live view. This repo
declares one docspace — the DOCSPACES protocol specification itself,
which lives under [`/protocol/docspaces/`](../../protocol/docspaces/):

| Docspace    | Version | Primary                                                       | Members |
|-------------|---------|---------------------------------------------------------------|---------|
| `docspaces` | 1.0.0   | [`PROTOCOL.md`](../../protocol/docspaces/PROTOCOL.md)        | 4       |

The four members (`PROTOCOL.md`, `README.md`, `EXTENSIONS.md`,
`MIGRATION.md`) all carry `docspace: docspaces` in their frontmatter;
the three non-primary members inherit via `version: docspace`. The
`directory` field in [`DOCSPACES.toml`](DOCSPACES.toml) (`../../protocol/docspaces`)
points the validator out of `docs/open/` and at the real location.

This repo is the DOCSPACES protocol's own first conforming consumer —
the protocol versions itself.

---

## Local workflow

### Running the validator

```bash
# Full run (errors only):
scripts/doc_docspace_check.py

# With pass lines:
scripts/doc_docspace_check.py --verbose

# List docspaces:
scripts/doc_docspace_check.py --list

# Subset of checks:
scripts/doc_docspace_check.py --check primary-versions,members
scripts/doc_docspace_check.py --check 1,2,8

# From outside the repo (explicit config path):
scripts/doc_docspace_check.py --config path/to/DOCSPACES.toml
```

Exit codes: `0` on success, `1` on any failure or config error.

### Adding a new doc to an existing docspace

1. Create the `.md` file under the docspace's `directory` (e.g.
   `protocol/docspaces/` for the `docspaces` docspace).
2. Add the filename to the docspace's `members = [...]` list in
   [`DOCSPACES.toml`](DOCSPACES.toml).
3. In the new doc's YAML frontmatter, add:
   ```yaml
   ---
   title: "…"
   docspace: <name>          # same as the docspace key in DOCSPACES.toml
   version: docspace         # inherit; resolves via DOCSPACES.toml
   disclaimer:
     notice: "…"             # required per CLAUDE.md §5 unless on opt-out list
   ---
   ```
4. Run `scripts/doc_docspace_check.py` — should pass.

### Adding a doc that needs an independent version (opt-out)

Some docs live in a docspace's directory but carry their own
independent version lineage. Opt them out so the validator does not
require `version: docspace` inheritance.

1. Place the file in the docspace's directory.
2. Add the filename to the docspace's `opt_out = [...]` list in
   [`DOCSPACES.toml`](DOCSPACES.toml) (NOT `members`).
3. In frontmatter, set `version:` to a literal string (don't use
   `version: docspace`).

The validator treats opt-outs as informational — they live in the
docspace's directory but aren't version-checked against the docspace.

### Bumping a docspace version

1. Edit the primary doc: bump its `version:` frontmatter.
2. Edit [`DOCSPACES.toml`](DOCSPACES.toml): update `[docspace.NAME] version`
   to match.
3. (If the docspace has a changelog) add an entry to the changelog file.
4. Run `scripts/doc_docspace_check.py` — must still pass.

Members with `version: docspace` need no edits; they inherit the new
version automatically.

### Adding a cross-docspace bind

Any docspace can bind other docspaces. The binding expresses *"this
docspace, at this version, is compatible against those docspaces at those
versions."*

Two forms:

**Short (version only, no content fingerprint):**
```toml
[docspace.my-docs.binds]
other-docspace = "2.3.4"
```

**Long (commit-pinned, structurally stronger — recommended after draft
content stabilizes):**
```toml
[docspace.my-docs.binds.other-docspace]
version = "2.3.4"
commit  = "abc1234"              # full or abbreviated git SHA
```

With a commit pin, the validator checks:
- the commit exists locally,
- `other-docspace`'s primary doc at that commit has `version: 2.3.4`.

If either fails, the bind is flagged as drifted.

### Frontmatter disclaimer requirement

CLAUDE.md §5 requires every AI-generated `.md` file to carry a frontmatter
`disclaimer:` block. The validator enforces this when [`DOCSPACES.toml`](DOCSPACES.toml)
has a `[disclaimer]` section with `required = true`. Opt-out entries go in
`[disclaimer].opt_out`.

Remove the `[disclaimer]` section in repos that don't need the check.

---

## The checks

The validator runs the **universal core** plus any **extensions** this
repo opts into via `[extensions.<id>]` in DOCSPACES.toml.

### Universal core (always run — PROTOCOL.md §6)

| # | Identifier         | What it verifies                                                                                            |
|---|--------------------|-------------------------------------------------------------------------------------------------------------|
| 1 | `primary-versions` | Each docspace's primary doc has a `version:` frontmatter matching the declared version in DOCSPACES.toml.   |
| 2 | `members`          | Every member file exists; declares the correct `docspace:`; inherits via `version: docspace` unless opt-out.|
| 3 | `no-orphan`        | No doc under the docs root declares a docspace not in DOCSPACES.toml.                                       |
| 4 | `binds-keys`       | Every `[docspace.X.binds]` key names a declared docspace.                                                   |
| 5 | `commit-pins`      | Commit-pinned binds: commit exists locally AND target's primary at that commit has the declared version.    |
| 6 | `protocol-version` | The manifest's `[protocol].docspaces` major matches the validator's supported major.                        |

### Extensions enabled in this repo (DOCSPACES.toml `[extensions.*]`)

| Identifier            | What it verifies                                                                                                         |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------|
| `disclaimer`          | Every `.md` under `docs/open/` carries a `disclaimer:` frontmatter field, with the listed opt-outs.                       |
| `binds-frontmatter`   | The primary doc's `binds_docspaces:` frontmatter list equals the keys under `[docspace.X.binds]`.                         |
| `readme-table`        | A summary table in `docs/open/README.md` mirrors per-docspace version / primary / member-count.                           |
| `frontmatter-paths`   | Frontmatter `companion_to` / `depends_on` / `extends` references resolve to real files.                                   |
| `section-refs`        | Cross-document section citations (`parent §N.M`, `<primary> §N.M`) resolve against the target's heading index.            |

### Extensions reserved by ID but not yet shipped

The legacy tables `[line_refs]`, `[prose_claims]`, `[readme_status]`,
`[cr_registration]`, and the link/anchor checks formerly under
`[link_check]` correspond to extension IDs `line-refs`, `prose-claims`,
`readme-status`, `cr-registration`, and `markdown-links` respectively
(see [`/protocol/docspaces/EXTENSIONS.md`](../../protocol/docspaces/EXTENSIONS.md)
§1 for the catalogue). The legacy tables remain in this repo's manifest
and will activate automatically when reference implementations land.

---

## Pre-commit hook wiring

The repo's [`.githooks/pre-commit`](../../.githooks/pre-commit) runs the
docspace validator automatically whenever a commit touches files under
`docs/open/`. To install the hook:

```bash
git config core.hooksPath .githooks
```

(One-time setup. Alternative: copy `.githooks/pre-commit` to
`.git/hooks/pre-commit` manually.)

With the hook active, commits that would leave the corpus in a drifted
state are rejected before they land. The check is fast (<1s) and only
runs on commits touching `docs/open/`.

---

## Portability

DOCSPACES is now a portable protocol. To adopt it in another repo, see
the universal adoption guide:
[`/protocol/docspaces/MIGRATION.md`](../../protocol/docspaces/MIGRATION.md).

Dependencies: Python ≥ 3.11 (stdlib `tomllib`). No third-party libraries.

---

## Related files

**In this repo:**

- [`DOCSPACES.toml`](DOCSPACES.toml) — this repo's manifest (authoritative for which docspaces / members exist)
- [`../../scripts/doc_docspace_check.py`](../../scripts/doc_docspace_check.py) — legacy entry-point shim; delegates to the reference validator
- [`../../.githooks/pre-commit`](../../.githooks/pre-commit) — wires the validator into git commits
- [`../../CLAUDE.md`](../../CLAUDE.md) — project-wide rules (§5 disclaimer requirement)
- [`../../DISCLAIMER.md`](../../DISCLAIMER.md) — methodological commitments

**The portable protocol:**

- [`../../protocol/docspaces/PROTOCOL.md`](../../protocol/docspaces/PROTOCOL.md) — universal specification
- [`../../protocol/docspaces/EXTENSIONS.md`](../../protocol/docspaces/EXTENSIONS.md) — extensions API and authoring guide
- [`../../protocol/docspaces/MIGRATION.md`](../../protocol/docspaces/MIGRATION.md) — adopting DOCSPACES elsewhere
- [`../../protocol/docspaces/validator/`](../../protocol/docspaces/validator/) — reference validator implementation

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.
