---
disclaimer:
  notice: >-
    This is the entry point for the DOCSPACES protocol. The authoritative
    specification is PROTOCOL.md; this README is navigational. If any
    sentence here disagrees with PROTOCOL.md, PROTOCOL.md wins.
  generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
  date: "2026-05-17"
title: "DOCSPACES — universal versioned documentation protocol"
docspace: docspaces
version: docspace
---

# DOCSPACES Protocol

**Status:** Stable · v1.0.0 · 2026-05-17

DOCSPACES is a universal, portable protocol for versioning documentation
in a way that survives drift. It treats a coherent family of documents
(a *docspace*) as one versioned unit, declared centrally and inherited
by members. Cross-document version references resolve through the
manifest, not through ad-hoc literal strings scattered across files.

The protocol is intentionally small — six universal checks, one
manifest format, one frontmatter contract — so any repository can adopt
it without bringing in opinions about toolchains, frameworks, or
languages.

## What's in this directory

| File                                                | Purpose                                                            |
|-----------------------------------------------------|--------------------------------------------------------------------|
| [PROTOCOL.md](PROTOCOL.md)                          | The normative specification. Read this first.                      |
| [DOCSPACES.template.toml](DOCSPACES.template.toml)  | Annotated reference manifest — copy and edit for your repo.        |
| [EXTENSIONS.md](EXTENSIONS.md)                      | How extensions work; how to author one; the registered catalogue.  |
| [MIGRATION.md](MIGRATION.md)                        | Adopt DOCSPACES in a new repo; upgrade a pre-1.0 manifest to 1.0.  |
| [validator/](validator/)                            | Reference Python validator (3.11+, stdlib-only).                   |

## 30-second adoption

1. Copy [DOCSPACES.template.toml](DOCSPACES.template.toml) to your
   docs root as `DOCSPACES.toml`. Declare your first `[docspace.<name>]`.
2. Add frontmatter to each member doc:
   ```yaml
   ---
   docspace: <name>
   version: docspace   # primary uses the literal version
   ---
   ```
3. Copy [validator/](validator/) into your repo (or vendor it however
   you prefer).
4. Run: `python3 -m validator`.
5. Iterate until clean. Optionally wire into a pre-commit hook.

Full walkthrough: [MIGRATION.md](MIGRATION.md).

## Why DOCSPACES exists

Before DOCSPACES, satellite docs each carry their own `version:`
frontmatter, and they drift. A spec ships at `2.0.0`; the changelog
still says `1.7.3`; the tutorial says `1.9`; nobody is sure which is
authoritative. The audit that motivated this protocol found 7 of 11
satellite docs were stating versions that no longer matched their
primary.

DOCSPACES borrows from Cargo workspaces: declare member versions once,
let members inherit. Then mechanically check that everyone is in
sync — every commit, in CI, or on demand.

## What DOCSPACES is not

- **Not a framework for *what* to write.** Diátaxis and similar
  frameworks already do that. DOCSPACES is orthogonal — it tracks
  versions of whatever you wrote.
- **Not opinionated about file layout.** Members can live anywhere
  under the docs root; a `directory` field lets you group.
- **Not coupled to any toolchain.** Python is the reference
  implementation, but the protocol is language-agnostic. Any tool
  that implements PROTOCOL.md §6 and the §6.1 output contract is
  conforming.

## Conformance

The protocol defines three conformance levels (PROTOCOL.md §8):

- **Validator conformance** — a tool implements the six universal checks.
- **Repository conformance** — a repo passes all universal checks via at
  least one conforming validator.
- **Document conformance** — a single `.md` file satisfies the
  frontmatter contract.

This repository's `docs/open/DOCSPACES.toml` is itself a conforming
manifest at the file level; pre-existing repo drift (missing files
from a 2026-04-23 monorepo extraction) prevents full conformance at
the repository level. See `MIGRATION.md` §2.3.

## Status & roadmap

| Item                          | Status  |
|-------------------------------|---------|
| Universal core (6 checks)     | Shipped |
| Extensions API                | Shipped |
| 5 registered extensions       | Shipped |
| Legacy-table back-compat      | Shipped |
| 5 reserved extension IDs      | Specified, impl pending |
| Conformance test suite        | Planned |
| pip distribution              | Planned |

## Related work

- **Cargo / npm / pnpm workspaces** — same idea for source code; this
  is the documentation analogue.
- **JSON Schema `$id`** — resolution by id rather than literal path;
  binds work the same way.
- **Diátaxis** — orthogonal: prescribes *what* to write, not how to
  version it.

## License

Same license as the host repository. The protocol specification itself
is intended to be freely implementable; see PROTOCOL.md §10 for the
versioning policy.

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.
