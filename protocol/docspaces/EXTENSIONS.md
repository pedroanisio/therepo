---
disclaimer:
  notice: >-
    This document describes the DOCSPACES extensions API and authoring
    guide. It is normative for the reference Python validator. Other
    conforming validators MAY implement equivalent APIs in other
    languages; this document does not constrain them.
  generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
  date: "2026-05-17"
title: "DOCSPACES — Extensions API & authoring guide"
docspace: docspaces
version: docspace
---

# DOCSPACES Extensions

Universal checks (PROTOCOL.md §6) cover the things every documentation
repository cares about: that primary versions match, that members exist,
that binds reference declared docspaces. Everything else — disclaimers,
README mirroring, line-range stability, prose-claim reconciliation — is
**repo policy**, and lives in extensions.

This document covers:

1. The registered extension catalogue (the extensions that ship with
   the reference validator).
2. How to author your own extension.
3. The Python API surface and its stability guarantees.

---

## 1. Registered extensions

These are the protocol-blessed extension identifiers. Each is
documented in detail in `protocol/docspaces/extensions/<id>.md` (when
the per-extension docs are written; until then this document is the
authoritative reference).

### Shipped (implemented in the reference validator)

| ID                     | What it checks                                                                                                                                 |
|------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|
| `disclaimer`           | Every `.md` under the docs root has a `disclaimer:` frontmatter field, respecting an opt-out list.                                              |
| `binds-frontmatter`    | The primary doc's `binds_docspaces:` frontmatter list equals the set of `[docspace.X.binds]` keys.                                              |
| `readme-table`         | A summary table in the docs-root README mirrors per-docspace `version` / `primary` / `member-count`.                                            |
| `frontmatter-paths`    | Frontmatter `companion_to`, `depends_on`, `extends` reference real files (relative to the doc).                                                 |
| `section-refs`         | Cross-document section citations of the forms `parent §N.M`, `prior §N.M`, `<primary-stem> §N.M` resolve against the target's heading index.    |

### Reserved (registered IDs, reference impl pending)

The following IDs are reserved by the protocol for repo-specific checks
that have been useful in practice. Reference implementations are
welcome PRs; until then, repos may ship private implementations under
these IDs.

| ID                | Intended check                                                                                                                                                |
|-------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `markdown-links`  | Relative `[text](path.md)` links resolve; with `verify_anchors`, anchors resolve to GitHub-style heading slugs.                                                |
| `line-refs`       | `[text](path.rs#L<N>-L<M>)` line-range links do not pin past end-of-file.                                                                                     |
| `prose-claims`    | Numeric / version claims in prose are reconciled against live repo state (e.g. "N proven theorems", "zero `sorry`", toolchain version pinning).                |
| `readme-status`   | Implementation-status markers (`stub`, `TBD`, `TODO`, `FIXME`, `XXX:`, `unimplemented`) are either resolved or carry an explicit `status-deferred:` annotation.|
| `cr-registration` | Every file under a CR (change-record) directory is registered in the manifest — either as a docspace member, an opt-out, or in `[unaffiliated]`.               |

If you implement one of these for your repo and the implementation is
generally useful, open a PR to upstream it under
`protocol/docspaces/validator/extensions/<id>.py`.

---

## 2. Activation rules

An extension `<id>` is **active** in a given run iff:

1. The manifest contains `[extensions.<id>]` (canonical form), and either
   `enabled` is absent or `enabled = true`.
2. **Or** the manifest contains a legacy top-level table that aliases
   to `<id>` (see `LEGACY_EXTENSION_TABLES` in `loader.py`) with either
   `enabled = true` or `required = true`.
3. The validator can resolve a registered implementation of `<id>` —
   either bundled in `validator/extensions/`, or in a directory passed
   via `--extensions-dir`.

If any of those conditions fails, the extension is silently inactive.
A conforming validator MUST NOT fail just because an extension is
configured but unimplemented (item 3) — that scenario is reported via
`--list-extensions` instead.

---

## 3. Authoring an extension

An extension is a Python module that exposes a `register(api)` function.
The module is placed in one of two locations:

- `protocol/docspaces/validator/extensions/<id>.py` for protocol-blessed
  extensions shipped with this repo.
- Anywhere on disk, surfaced via `--extensions-dir <dir>`, for repo-
  private extensions.

### Minimal skeleton

```python
"""Extension: my-check — short one-line description."""
from __future__ import annotations

# For built-in extensions (inside this validator package), use relative imports:
#   from ..core import CheckContext, Extension
#   from ..loader import extension_config
# For repo-private extensions, use absolute imports:
#   from validator import CheckContext, Extension
#   from validator.loader import extension_config

from ..core import CheckContext, Extension
from ..loader import extension_config

EXT_ID = "my-check"


def run(ctx: CheckContext) -> None:
    cfg = extension_config(ctx.config, EXT_ID)
    ctx.reporter.section("Checking my custom thing")
    # Walk ctx.docs_root, read files, call ctx.reporter.ok(...) / .fail(...).
    # Use cfg.get("some_key", default) for configuration.


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="one-line summary of what this checks",
        run=run,
        requires_git=False,   # set True if the check uses ctx.repo_root
    ))
```

### Manifest declaration

```toml
[extensions.my-check]
enabled = true
# any keys your check expects — read them in `run(ctx)`
some_key = "some_value"
```

### What `CheckContext` gives you

```python
@dataclass
class CheckContext:
    config: dict          # parsed TOML manifest
    docs_root: Path       # absolute path to the docs root
    reporter: Reporter    # ok(msg) / fail(msg) / section(title)
    repo_root: Path | None  # git toplevel, or None if not in git
```

You should NOT mutate `ctx.config`. The `reporter` accumulates
PASS / FAIL counts that drive the validator's exit code.

### Helpers available from `validator.core`

- `parse_frontmatter(path) -> Frontmatter` — shallow YAML frontmatter.
- `body_without_frontmatter(path) -> str` — markdown body, no frontmatter.
- `resolve_member_path(docs_root, entry, member)` — turn a docspace
  member name into an absolute path.
- `docspace_entries(config)` — iterate `(name, entry)` over docspaces.
- `read_text(path)` — UTF-8 read with safe fallback.

For git access (when `requires_git=True` is set), `validator.core`
exposes `git_repo_root`, `git_commit_exists`, `git_show_content`.

### Error handling

Anything raised in `run(ctx)` bubbles up and aborts the validator.
**Prefer `ctx.reporter.fail(msg)` over raising.** Use exceptions only
for unrecoverable errors that indicate a bug in the extension itself
(e.g. malformed manifest you cannot work around).

---

## 4. API stability

The interfaces in `validator/core.py` and `validator/loader.py` marked
as `@dataclass` (`CheckContext`, `Extension`, `Reporter`,
`ExtensionRegistry`, `Frontmatter`, `RunResult`) and the
`register(api)` entry-point contract are **stable within DOCSPACES
protocol major version 1**.

We MAY add new fields with safe defaults in minor versions. We will NOT
remove fields, change method signatures, or rename existing fields
within a major.

The internal helpers (`_FM_KEY_RE`, `_parse_inline_list`, etc.) are
NOT stable API. Extension authors should not import them.

---

## 5. Testing your extension

A future minor release of the protocol will ship a conformance test
harness (`protocol/docspaces/conformance/`). Until then, test your
extension by:

1. Creating a fixture directory with a small `DOCSPACES.toml` and a
   handful of `.md` files.
2. Pointing the validator at it: `python3 -m validator --config
   fixture/DOCSPACES.toml --extensions-dir path/to/your/exts`.
3. Asserting on the printed `PASS:` / `FAIL:` lines and the exit code.

For programmatic testing:

```python
from pathlib import Path
from validator import run

result = run(
    config_path=Path("fixture/DOCSPACES.toml"),
    extensions_dirs=[Path("my/extensions")],
)
assert result.ok
```

---

## 6. Publishing a new globally registered extension

The path:

1. Implement your extension. Keep it policy-clean enough that other
   repos could plausibly use it.
2. Place the module at `protocol/docspaces/validator/extensions/<id>.py`.
3. Add an entry to PROTOCOL.md §7.5 and to §1 above.
4. Bump the protocol minor version (`1.X.0`) in PROTOCOL.md.
5. Open a PR.

Aim for extensions that are useful across at least two real
repositories — a good filter against scope creep. Repo-specific checks
that are only ever useful in one tree should stay as private
extensions loaded via `--extensions-dir`.

---

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.
