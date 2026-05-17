"""Extension: `binds-frontmatter` — primary doc's `binds_docspaces:` list
must equal the set of `[docspace.X.binds]` keys for its docspace.

Manifest:

    [extensions.binds-frontmatter]
    enabled = true
"""
from __future__ import annotations

from ..core import (
    CheckContext,
    Extension,
    docspace_entries,
    parse_frontmatter,
    resolve_member_path,
)

EXT_ID = "binds-frontmatter"


def run(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking binds_docspaces frontmatter mirrors TOML bind keys")
    for name, entry in docspace_entries(ctx.config):
        binds = entry.get("binds")
        if not isinstance(binds, dict):
            continue
        primary = entry.get("primary")
        if not primary:
            continue
        path = resolve_member_path(ctx.docs_root, entry, primary)
        fm = parse_frontmatter(path)
        actual = set(fm.get_list("binds_docspaces"))
        expected = set(binds.keys())
        if actual == expected:
            ctx.reporter.ok(f"[{name}] binds_docspaces matches TOML bind keys")
        else:
            ctx.reporter.fail(
                f"[{name}] binds_docspaces={sorted(actual)} but "
                f"TOML binds={sorted(expected)}"
            )


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="primary doc's binds_docspaces frontmatter mirrors TOML bind keys",
        run=run,
    ))
