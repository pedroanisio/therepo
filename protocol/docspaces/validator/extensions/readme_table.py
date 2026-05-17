"""Extension: `readme-table` — a summary table in the docs-root README
must mirror the manifest's per-docspace `version` / `primary` / `member-count`.

Expected row format:

    | `<docspace-name>` | <free description> | `<primary>` | <member-count> |

Manifest:

    [extensions.readme-table]
    enabled = true
    readme  = "README.md"     # path relative to docs root; default "README.md"
"""
from __future__ import annotations

import re

from ..core import CheckContext, Extension, docspace_entries, read_text
from ..loader import extension_config

EXT_ID = "readme-table"
_ROW_RE = re.compile(
    r"^\|\s*`([^`]+)`\s*\|\s*([^|]+?)\s*\|\s*`([^`]+)`\s*\|\s*([0-9]+)\s*\|$"
)


def run(ctx: CheckContext) -> None:
    cfg = extension_config(ctx.config, EXT_ID)
    readme_rel = cfg.get("readme", "README.md")
    readme = ctx.docs_root / readme_rel
    ctx.reporter.section(f"Checking {readme_rel} summary table against DOCSPACES.toml")
    if not readme.is_file():
        ctx.reporter.fail(f"{readme_rel} missing under docs root")
        return

    rows: dict[str, tuple[str, str, str]] = {}
    for line in read_text(readme).splitlines():
        m = _ROW_RE.match(line.strip())
        if m:
            rows[m.group(1)] = (m.group(2).strip(), m.group(3).strip(), m.group(4).strip())

    for name, entry in docspace_entries(ctx.config):
        directory = entry.get("directory", ".")
        primary = entry.get("primary", "")
        expected = (
            entry.get("version", ""),
            primary if directory == "." else f"{directory}/{primary}",
            str(len(entry.get("members", []))),
        )
        actual = rows.get(name)
        if actual == expected:
            ctx.reporter.ok(f"[README] row for '{name}' matches DOCSPACES.toml")
        else:
            ctx.reporter.fail(
                f"[README] row for '{name}'={actual} but expected {expected}"
            )


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="docs-root README summary table mirrors the manifest",
        run=run,
    ))
