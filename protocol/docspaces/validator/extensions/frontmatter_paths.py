"""Extension: `frontmatter-paths` — frontmatter `companion_to`,
`depends_on`, `extends` references must resolve to real files.

Manifest:

    [extensions.frontmatter-paths]
    enabled = true
"""
from __future__ import annotations

import re

from ..core import CheckContext, Extension, parse_frontmatter

EXT_ID = "frontmatter-paths"
_PATH_KEYS = ("companion_to", "depends_on", "extends")
_PATH_TOKEN_RE = re.compile(r"((?:\./|\.\./)?[A-Za-z0-9_./-]+\.md)\b")


def _extract_path_token(value: str) -> str:
    m = _PATH_TOKEN_RE.match(value)
    return m.group(1) if m else ""


def run(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking frontmatter path references resolve")
    for md_path in sorted(ctx.docs_root.rglob("*.md")):
        fm = parse_frontmatter(md_path)
        try:
            rel = md_path.relative_to(ctx.docs_root).as_posix()
        except ValueError:
            rel = str(md_path)
        for key in _PATH_KEYS:
            values: list[str] = []
            if fm.get(key):
                values.append(fm.get(key))
            values.extend(fm.get_list(key))
            for value in values:
                token = _extract_path_token(value)
                if not token:
                    continue
                candidate = (md_path.parent / token).resolve()
                if candidate.is_file():
                    ctx.reporter.ok(f"[{rel}] {key} -> {token}")
                else:
                    ctx.reporter.fail(
                        f"[{rel}] {key} references missing path '{token}'"
                    )


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="frontmatter companion_to/depends_on/extends resolve to real files",
        run=run,
    ))
