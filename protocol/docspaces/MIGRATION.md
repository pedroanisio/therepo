---
disclaimer:
  notice: >-
    This document covers two flavors of migration: (a) adopting
    DOCSPACES in a new repository, (b) upgrading a pre-1.0 manifest to
    DOCSPACES protocol 1.0. The worked example uses this repository's
    own migration as the case study.
  generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
  date: "2026-05-17"
title: "DOCSPACES — Adoption & migration guide"
docspace: docspaces
version: docspace
---

# DOCSPACES — Adoption & migration guide

This guide covers three scenarios:

1. **Greenfield adoption** — adding DOCSPACES to a repo that has none.
2. **Upgrading a pre-1.0 manifest** — bringing an existing `DOCSPACES.toml`
   into protocol-1.0 conformance.
3. **Porting the validator into another repository** — running the
   reference implementation outside this one.

---

## 1. Greenfield adoption

### 1.1 Prerequisites

- Python ≥ 3.11 (the validator uses stdlib `tomllib`).
- A documentation directory you can write to. Conventionally `docs/`,
  `docs/open/`, `documentation/`, or `site/`. The validator
  auto-discovers any of these.

### 1.2 Steps

**Step 1 — install the validator.** Copy
`protocol/docspaces/validator/` into your repo (commit it as
`tools/docspaces/`, `vendor/docspaces/`, or wherever). Once installed,
`python3 -m validator` from that directory is your entry point.

**Step 2 — create the manifest.** Copy
`protocol/docspaces/DOCSPACES.template.toml` to your docs root as
`DOCSPACES.toml`. Edit:

```toml
[protocol]
docspaces = "1.0"

[docspace.my-spec]
version = "0.1.0"
primary = "spec.md"
members = ["spec.md", "guide.md", "CHANGELOG.md"]
```

**Step 3 — add frontmatter to each member.** Every member document
needs at minimum:

```yaml
---
docspace: my-spec
version: docspace        # …except the primary, which uses the literal version
---
```

The primary doc's frontmatter:

```yaml
---
docspace: my-spec
version: "0.1.0"
---
```

**Step 4 — run the validator.**

```bash
python3 -m validator
# …if your CWD is not inside the docs root:
python3 -m validator --config docs/DOCSPACES.toml
```

Iterate until you see `Summary: N passed, 0 failed`.

**Step 5 — wire CI.** Add the validator to your CI pipeline or to a
pre-commit hook. A minimal pre-commit hook:

```bash
#!/usr/bin/env sh
set -eu
# Only run when docs change
if git diff --cached --name-only | grep -qE '\.md$|DOCSPACES\.toml$'; then
  python3 -m validator
fi
```

### 1.3 Adding extensions

Each extension is opt-in via `[extensions.<id>]`. Start without
any — the universal core gives you version coherence on its own. Add
extensions only when their specific drift class has bitten you.

Recommended starter set for most repos:

```toml
[extensions.disclaimer]            # if you have a project-wide disclaimer rule
enabled = true

[extensions.frontmatter-paths]      # if docs reference each other in frontmatter
enabled = true

[extensions.readme-table]           # if your docs README has a docspace summary
enabled = true
```

---

## 2. Upgrading a pre-1.0 manifest to protocol 1.0

### 2.1 What changed

The 1.0 protocol introduces three breaking-ish changes vs the pre-1.0
informal convention:

- **`[protocol].docspaces` is mandatory.** Validators now refuse a
  manifest without it.
- **Extension config moves to `[extensions.<id>]`.** Legacy top-level
  tables (`[disclaimer]`, `[link_check]`, etc.) still work via the
  loader's legacy aliasing, but new manifests should use the canonical
  form.
- **Extensions that were "always on" are now opt-in.** Pre-1.0
  validators ran `binds-frontmatter`, `readme-table`, `frontmatter-paths`,
  and `section-refs` unconditionally. They now require an explicit
  `[extensions.<id>] enabled = true`.

### 2.2 Steps

**Step 1 — declare protocol version.** At the top of your manifest:

```toml
[protocol]
docspaces = "1.0"
```

**Step 2 — canonicalize extension config.** For each legacy table you
have, choose: keep the legacy form (validator still understands it) or
migrate to the canonical form. Migration table:

| Legacy table         | Canonical `[extensions.<id>]`                       |
|----------------------|-----------------------------------------------------|
| `[disclaimer]`        | `[extensions.disclaimer]`                            |
| `[link_check]`        | `[extensions.markdown-links]`                        |
| `[line_refs]`         | `[extensions.line-refs]`                             |
| `[prose_claims]`      | `[extensions.prose-claims]`                          |
| `[readme_status]`     | `[extensions.readme-status]`                         |
| `[cr_registration]`   | `[extensions.cr-registration]`                       |

The legacy `required = true` becomes `enabled = true`. Other keys
(e.g. `opt_out`, `scan`) carry over unchanged.

**Step 3 — opt in to previously-unconditional checks.** If your repo
relied on `binds-frontmatter`, `readme-table`, `frontmatter-paths`, or
`section-refs` running by default, add them explicitly:

```toml
[extensions.binds-frontmatter]
enabled = true

[extensions.readme-table]
enabled = true

[extensions.frontmatter-paths]
enabled = true

[extensions.section-refs]
enabled = true
```

**Step 4 — re-run the validator.** Counts should match what you had
under the pre-1.0 validator (modulo any drift introduced since the
last clean run).

### 2.3 This repository's migration (worked example)

The migration applied to this repo's `docs/open/DOCSPACES.toml`:

```diff
+ [protocol]
+ docspaces = "1.0"
+
  [docspace.usl-ng]
  …

- [disclaimer]
- required = true
- opt_out = [...]
+ [extensions.disclaimer]
+ enabled = true
+ opt_out = [...]
+
+ [extensions.binds-frontmatter]
+ enabled = true
+
+ [extensions.readme-table]
+ enabled = true
+
+ [extensions.frontmatter-paths]
+ enabled = true
+
+ [extensions.section-refs]
+ enabled = true
```

The legacy tables `[readme_status]`, `[cr_registration]`,
`[line_refs]`, `[prose_claims]`, and `[link_check]` are left **as-is**
in this repo's manifest. Their corresponding extension implementations
(`readme-status`, `cr-registration`, `line-refs`, `prose-claims`,
`markdown-links`) are not yet shipped in the reference validator —
they remain reserved IDs (see EXTENSIONS.md §1). When implementations
land, the legacy tables will activate automatically via the loader's
aliasing.

---

## 3. Porting the validator to another repo

There are three viable distribution patterns; pick what fits the
host repo.

### 3.1 Vendored (recommended for now)

Copy `protocol/docspaces/validator/` into your repo at any path you
like — common choices: `tools/docspaces/validator/`,
`vendor/docspaces/`, `.tools/docspaces/`. Add a thin shim like
`scripts/doc_docspace_check.py` that adjusts `sys.path` and calls
`validator.main`.

Advantages: no dependency manager required, validator version is
locked to your repo's git history.

Trade-off: you reapply upstream improvements by re-copying.

### 3.2 Submodule

Add this repo (or a future spin-off) as a git submodule:

```bash
git submodule add https://github.com/example/docspaces .tools/docspaces
```

Then `python3 -m validator --config docs/DOCSPACES.toml` from
`.tools/docspaces/protocol/docspaces/` (or write a one-line shim).

### 3.3 Pip-installable (future)

A `pyproject.toml` distribution under `protocol/docspaces/` is on the
roadmap. Once shipped, adoption simplifies to:

```bash
pipx install docspaces-validator
docspaces-validator --config docs/DOCSPACES.toml
```

Until then, vendoring is the recommended approach.

---

## 4. Common adoption pitfalls

- **Forgetting `version: docspace` on a member.** The validator sees
  empty frontmatter and reports "no version: field." Add the line.
- **A member's frontmatter says `docspace: foo` but the manifest lists
  it under `docspace: bar`.** Check #2 (members) reports the mismatch.
- **A primary doc's `version:` doesn't match the manifest.** Check #1
  reports it. Decide which is right and update the other.
- **A bind references a docspace that isn't declared.** Check #4
  reports it. Either declare the docspace or remove the bind.
- **A commit-pinned bind references a commit not in your local git
  history.** Check #5 reports it. Either fetch the commit or change
  the bind to version-only form.

---

## Disclaimer

This work is subject to the methodological caveats and commitments described in [@DISCLAIMER.md](../../DISCLAIMER.md).
> No statement or premise not backed by a real logical definition or verifiable reference should be taken for granted.
