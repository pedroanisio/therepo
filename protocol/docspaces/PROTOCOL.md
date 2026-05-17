---
disclaimer:
  notice: >-
    This document specifies the universal DOCSPACES protocol. The protocol
    itself is the contract — repositories implementing it can be checked
    against this specification using any conforming validator. The
    reference validator lives at protocol/docspaces/validator/. Any claim
    here about behavior is normative; any example is illustrative.
  generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
  date: "2026-05-17"
title: "DOCSPACES Protocol — Universal Specification"
docspace: docspaces
version: "1.0.0"
status: stable
---

# DOCSPACES — A Universal Protocol for Versioned Documentation Workspaces

**Protocol version:** `1.0.0`
**Status:** Stable
**Audience:** Repository maintainers, documentation engineers, AI agents
operating on documentation, tool authors

---

## 1. Motivation

Documentation drifts. A specification ships at `v2.0.0`. A satellite guide,
a changelog, a tutorial, a roadmap each carry their own `version:`
frontmatter. Six months later, half of them claim a version that no longer
reflects what the spec says, and nobody is sure which doc is authoritative.

**DOCSPACES** is a versioning discipline that treats a coherent family of
documents as a single unit. One version is declared centrally; members
inherit. The discipline is enforced mechanically by a validator that runs
on commit or in CI.

The convention is analogous to a Cargo workspace, an npm workspace, or a
Bazel BUILD package: a top-level manifest pins member versions; individual
members opt in by declaration rather than by literal restatement.

> **Naming.** "DOCSPACE" (singular) refers to one versioned family of
> documents. "DOCSPACES" (plural) refers to this protocol and to the
> collection of docspaces declared in a manifest.

---

## 2. Scope of this document

This document specifies:

- The **manifest format** (`DOCSPACES.toml`): the universal core schema.
- The **frontmatter contract**: what conforming documents must declare.
- The **universal checks**: the mandatory verifications a conforming
  validator must perform.
- The **extensions API**: how repos add their own checks without
  contaminating the universal core.
- The **conformance levels** for validators, repositories, and documents.

This document does **not** specify:

- A particular validator implementation. The reference implementation in
  this repository is conforming, but any tool that implements §6 is also
  conforming.
- Specific extensions. Extensions are documented separately under
  `protocol/docspaces/extensions/`.
- Repo-specific policy. Anything that depends on `crates/`, `proofs/`,
  Lean toolchains, or particular markdown table formats is an
  extension — not part of the protocol.

---

## 3. Definitions

A **docspace** is a named family of documents sharing one version string,
declared in a `DOCSPACES.toml` manifest.

A **primary doc** is the single source of truth for a docspace's version.
Its frontmatter `version:` must match the manifest declaration.

A **member** is a document listed in a docspace's `members` array. Every
member must declare `docspace: <name>` in its frontmatter and (unless it
is the primary or is opted out) must inherit the docspace version via
`version: docspace`.

An **opt-out member** is a document that lives inside a docspace's
directory but carries its own independent version lineage. Listed under
`opt_out`, not `members`.

A **bind** is a cross-docspace compatibility declaration: docspace A
declares that, at version vA, it is compatible with docspace B at version
vB. Optionally, the bind names a specific git commit, pinning the
compatibility contract to a content fingerprint.

An **extension** is a repo-specific, named check registered against a
declared `[extensions.<id>]` table in the manifest. Extensions live
outside the universal core and may be enabled, configured, or absent on
a per-repo basis.

A **conforming validator** is any tool that implements the universal
checks defined in §6 and may, optionally, implement any subset of the
registered extensions.

A **conforming repository** is one whose `DOCSPACES.toml` and member
documents satisfy all universal checks against at least one conforming
validator.

---

## 4. The manifest: `DOCSPACES.toml`

### 4.1 Location

A conforming validator MUST locate the manifest by:

1. Honoring an explicit path passed via `--config` (or equivalent).
2. Otherwise, walking upward from the working directory and selecting
   the first `DOCSPACES.toml` found.
3. If none is found by walking up, probing `docs/`, `docs/open/`,
   `documentation/`, and `site/` under each ancestor.

The **docs root** is the directory containing the manifest (or any
override supplied by the caller). All member paths are resolved
relative to a docspace's `directory` field, which is itself relative to
the docs root.

### 4.2 Universal core schema

The following tables and keys constitute the universal core. A
conforming repository MUST use these structures unchanged.

```toml
# Required: declare protocol version. Validators MUST verify
# this is a version they support.
[protocol]
docspaces = "1.0"        # major.minor of the DOCSPACES protocol

# Zero or more docspaces. Each name is a free-form identifier
# (recommended: kebab-case, ASCII).
[docspace.<name>]
version   = "<string>"               # required; arbitrary version string
primary   = "<file>"                 # required; relative to `directory`
members   = ["<file>", "<file>"]     # required; primary MUST be included
directory = "."                      # optional; default "."
changelog = "<file>"                 # optional; documentation-only
opt_out   = ["<file>"]               # optional; default []

# Optional: cross-docspace bindings.
[docspace.<name>.binds]
other-docspace = "<version-string>"               # short form
another        = { version = "<v>", commit = "<sha>" }  # inline table form

# Or, equivalently, as a sub-table:
[docspace.<name>.binds.another]
version = "<v>"
commit  = "<sha>"   # full or abbreviated git SHA; must exist locally

# Optional: documents that live under the docs root but are not
# members of any docspace. Informational only — not verified by the
# universal core.
[unaffiliated]
docs          = ["<file>"]
binary_assets = ["<file>"]
```

**Versioning rule.** The `[protocol].docspaces` field declares which
version of *this specification* the manifest claims to conform to.
Validators MUST refuse to validate a manifest whose major version does
not match the validator's. Minor-version mismatches MUST be tolerated
(forward-compatible additions only land in minor versions).

### 4.3 Frontmatter contract

Every member document MUST carry YAML frontmatter at the top of the file
with at least:

```yaml
---
docspace: <name>         # MUST equal the docspace key in the manifest
version: docspace        # MUST be the literal string "docspace"
                         # …except in the primary, where it MUST be
                         # the same string as the manifest declares,
                         # and except in opt-out members, where it
                         # MUST be any non-"docspace" string.
---
```

A member MUST NOT carry more than one top-level `version:` field.
Documents may carry additional frontmatter fields freely — the universal
core does not constrain them.

### 4.4 Extension tables

Any TOML table whose top-level key is `extensions` is reserved for the
extensions API (see §7). The universal core MUST NOT inspect any
`[extensions.*]` table beyond verifying that the table is well-formed
TOML.

For backward compatibility with pre-1.0 deployments, validators MAY
accept top-level tables other than the ones listed in §4.2 and treat
them as "legacy extension tables." Producers of new manifests SHOULD
place all extension config under `[extensions.<id>]`.

---

## 5. The frontmatter contract

### 5.1 Required keys (per role)

| Role              | `docspace:` | `version:`              | Notes                                                  |
|-------------------|-------------|-------------------------|--------------------------------------------------------|
| Primary           | `<name>`    | manifest version string | Source of truth for the docspace version.              |
| Non-primary member| `<name>`    | `docspace` (literal)    | Inherits version from the manifest at check time.      |
| Opt-out member    | `<name>`    | any non-`docspace`      | Lives in directory but versions independently.         |
| Unaffiliated doc  | (omitted)   | (free)                  | Validator does not constrain.                          |

### 5.2 Parser obligations

A conforming validator MUST parse YAML frontmatter as follows:

- Frontmatter begins at line 1 with a literal `---` line and ends at
  the next `---` line at column 0.
- Top-level scalar keys match `^[A-Za-z_][A-Za-z0-9_-]*\s*:\s*(.*)$`.
- Quoted values (`"..."` or `'...'`) MUST be unquoted before comparison.
- Trailing ` # ...` inline comments MUST be stripped from scalar values.
- The validator's view of frontmatter is intentionally shallow: only
  top-level scalar keys and top-level scalar lists are observed.
  Nested mappings (e.g. `disclaimer.notice`) are visible as their
  flattened key (`disclaimer`) being present, with no value inspection.

This deliberately avoids requiring a full YAML parser. A repo that
needs deeper structural validation of frontmatter can register an
extension.

---

## 6. Universal checks

A conforming validator MUST implement every check in this section. Each
check has a stable string identifier (`primary-versions`, `members`,
etc.) and a stable numeric alias.

| #  | Identifier          | Mandatory? | Verifies                                                                                                              |
|----|---------------------|------------|-----------------------------------------------------------------------------------------------------------------------|
| 1  | `primary-versions`  | yes        | Each docspace's primary doc exists and its frontmatter `version:` equals the manifest's declared version.             |
| 2  | `members`           | yes        | Every member file exists; declares the correct `docspace:`; inherits via `version: docspace` unless primary/opt-out. |
| 3  | `no-orphan`         | yes        | No document under the docs root declares a `docspace:` not present in the manifest.                                  |
| 4  | `binds-keys`        | yes        | Every key under `[docspace.X.binds]` names a declared docspace.                                                       |
| 5  | `commit-pins`       | conditional| For binds with a `commit` field, the commit exists locally AND the pinned docspace's primary at that commit has a frontmatter `version:` equal to the bind's `version`. Skipped iff the docs root is not in a git working tree. |
| 6  | `protocol-version`  | yes        | `[protocol].docspaces` is present and its major part matches the validator's supported major version.                |

These six checks are the **universal core**. They make sense in any
documentation repository and require no policy decisions beyond the
manifest.

Identifiers `7..n` are **registered extension** identifiers, defined in
`protocol/docspaces/extensions/<id>.md`. A validator MAY implement any
subset; absence of an extension implementation MUST NOT cause a
universal check to fail.

### 6.1 Output conformance

A conforming validator MUST:

- Print each check's result with a stable tag: `PASS:` or `FAIL:`
  (machine-greppable).
- Exit with code `0` iff zero failures across all selected checks;
  exit `1` otherwise.
- Support a check-selection argument (`--check` or equivalent) that
  accepts identifiers, numeric aliases, or a comma-separated list of
  either.
- Support `--list` to dump the declared docspaces without running
  checks.

A conforming validator MAY produce additional output (a section
divider, a summary line, verbose `PASS:` traces) provided the
machine-greppable contract is preserved.

---

## 7. Extensions API

Extensions are the protocol's escape hatch: every check that is
genuinely useful but not universal lives here. The current reference
implementation ships with the extensions documented in
`protocol/docspaces/extensions/`.

### 7.1 Registration

An extension is registered by:

1. A documentation file at
   `protocol/docspaces/extensions/<id>.md` describing what it checks,
   its TOML schema under `[extensions.<id>]`, and its activation rule.
2. A loadable check module — for the reference validator, a Python
   file at `protocol/docspaces/validator/extensions/<id>.py` exposing
   a `register(api)` function.

The `<id>` is a kebab-case identifier. Globally registered identifiers
(those that ship with the reference validator) live in this repository.
Repos may define their own private extensions under their own ids; if
two repos use the same id for incompatible semantics, the id has been
abused.

### 7.2 Manifest shape

All extension config lives under `[extensions.<id>]`:

```toml
[extensions.<id>]
enabled = true                # optional; default true if the table is present
# ...extension-specific keys...
```

Activation: an extension is **active** for a given run iff
(a) `[extensions.<id>]` is present in the manifest, (b) its `enabled`
field is `true` or absent, and (c) the validator has the extension
implementation available.

A validator MUST NOT fail an extension check when the extension is not
configured or its implementation is missing. It MAY emit an
informational note.

### 7.3 The Python API (reference validator)

```python
class CheckContext:
    config: dict          # parsed TOML manifest
    docs_root: Path       # absolute path to docs root
    reporter: Reporter    # `ok(msg)` and `fail(msg)`
    repo_root: Path | None  # git toplevel, or None if not in git

class Extension:
    id: str               # kebab-case id, matches [extensions.<id>]
    description: str      # one-line summary
    requires_git: bool    # if True, silently skipped outside a git tree
    def run(self, ctx: CheckContext) -> None: ...

def register(api):
    api.register_extension(Extension(...))
```

The validator discovers extensions by importing every `*.py` under
`protocol/docspaces/validator/extensions/` (excluding `_*.py`) and
calling its `register(api)` function. Repos that need additional
extensions can add modules to that directory or point the validator at
an additional extensions directory via `--extensions-dir`.

### 7.4 Stability

The Python API in §7.3 is **stable within a major protocol version**.
The reference validator MUST NOT make breaking changes to the
`CheckContext`, `Extension`, or `register` shapes between `1.x`
releases.

### 7.5 Currently registered extensions

The following extension identifiers are reserved by this version of the
protocol and documented under `protocol/docspaces/extensions/`. A
validator that implements any of them MUST follow the identifier
exactly.

| Identifier            | Reference module                | Summary                                                                       |
|-----------------------|---------------------------------|-------------------------------------------------------------------------------|
| `disclaimer`          | `disclaimer.py`                 | Every `.md` under the docs root carries a `disclaimer:` frontmatter field.    |
| `binds-frontmatter`   | `binds_frontmatter.py`          | Primary doc's `binds_docspaces:` list matches the manifest's bind keys.       |
| `readme-table`        | `readme_table.py`               | A summary table in `docs/<root>/README.md` mirrors the manifest.              |
| `frontmatter-paths`   | `frontmatter_paths.py`          | Frontmatter `companion_to` / `depends_on` / `extends` resolve to real files.  |
| `section-refs`        | `section_refs.py`               | Cross-document section references like `<primary> §N.M` resolve.              |
| `markdown-links`      | `markdown_links.py`             | Relative `[text](path.md)` links resolve; anchors validate to headings.       |
| `line-refs`           | `line_refs.py`                  | `path.rs#L<N>-L<M>` line-range links don't run past end-of-file.              |
| `prose-claims`        | `prose_claims.py`               | Numeric claims in prose (e.g. "N proven theorems") still match repo state.    |
| `readme-status`       | `readme_status.py`              | Implementation-status markers in code/READMEs are explicitly justified.       |

Extensions beyond this list MAY exist in implementing repositories but
are not protocol-blessed. Pull requests that propose a new globally
registered extension should land its `<id>.md` in
`protocol/docspaces/extensions/` along with a reference module.

---

## 8. Conformance levels

The protocol defines three conformance levels.

### 8.1 Validator conformance

A validator is **DOCSPACES-conforming** iff it implements every
universal check in §6 with the §6.1 output contract and the §4.1
discovery rules.

A validator is **DOCSPACES+** conforming iff it is conforming AND
implements the §7.3 Python extensions API (or an equivalent
language-native API documented as conforming).

### 8.2 Repository conformance

A repository is **DOCSPACES-conforming** iff:

1. Its docs root contains a `DOCSPACES.toml` matching §4.2.
2. Every member document satisfies §5.1.
3. At least one conforming validator reports zero failures across the
   universal checks.

A repository is **DOCSPACES+ strict** iff it is conforming AND a
pre-commit hook (or equivalent CI gate) blocks commits that would
break universal conformance.

### 8.3 Document conformance

An individual `.md` file is **DOCSPACES-conforming** as a docspace
member iff its frontmatter satisfies §5.1 and the file path matches a
member entry in some `[docspace.X]` table.

---

## 9. Repository adoption

To adopt DOCSPACES in a new repository:

1. Create `DOCSPACES.toml` at your documentation root with at least the
   `[protocol]` table and one `[docspace.<name>]` declaration.
2. Add `docspace: <name>` and `version: docspace` frontmatter to each
   member document.
3. Copy `protocol/docspaces/validator/` into your repo (or install via
   a package distribution if available).
4. Run the validator. Iterate until clean.
5. Optionally, wire the validator into a pre-commit hook or CI.

See `protocol/docspaces/MIGRATION.md` for a worked example.

---

## 10. Versioning of the protocol itself

This specification follows semantic versioning:

- **Major** (`X.0.0`): breaking changes — universal check semantics,
  frontmatter contract, manifest shape changes that existing manifests
  cannot satisfy.
- **Minor** (`1.X.0`): backward-compatible additions — new optional
  manifest keys, new registered extension identifiers, new universal
  checks behind a default-off opt-in.
- **Patch** (`1.0.X`): clarifications, typo fixes, examples — no
  semantic change.

A validator MUST refuse manifests whose declared
`[protocol].docspaces` major version it does not support. A validator
SHOULD tolerate higher minor versions and produce a warning if it
encounters unknown optional keys.

---

## 11. Conformance suite (informative)

A future minor release of this protocol will ship a conformance test
suite under `protocol/docspaces/conformance/`. The intent: a fixture
directory of mini-repositories plus expected pass/fail manifests
against which any validator can be checked.

Until that lands, the reference validator's behavior against this
repository's own `DOCSPACES.toml` serves as the de facto conformance
baseline.

---

## 12. Related work

- **Cargo workspaces** (Rust). Pin member crate versions centrally.
  DOCSPACES is the documentation analogue.
- **npm / pnpm workspaces** (JavaScript). Same idea, different
  package manager.
- **JSON Schema `$id` + `$ref`** (web standards). Resolve references
  by id rather than literal path; DOCSPACES does the same for binds.
- **Diátaxis** (Procida). Framework for *what* documentation to write;
  DOCSPACES is orthogonal — it tracks versions of whatever you wrote.

---

## 13. Index of files in this directory

- [PROTOCOL.md](PROTOCOL.md) — this document
- [DOCSPACES.template.toml](DOCSPACES.template.toml) — annotated reference manifest
- [MIGRATION.md](MIGRATION.md) — adopt DOCSPACES in a new or existing repo
- [EXTENSIONS.md](EXTENSIONS.md) — how to author an extension
- [validator/](validator/) — reference validator (Python 3.11+, stdlib-only)
- [extensions/](extensions/) — protocol-registered extension specifications

---

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.
