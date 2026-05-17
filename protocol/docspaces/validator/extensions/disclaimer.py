"""Extension: `disclaimer` — every .md must carry a `disclaimer:` frontmatter field.

Manifest:

    [extensions.disclaimer]
    enabled = true
    opt_out = ["CR/one-off.md"]   # paths relative to docs root
"""
from __future__ import annotations

from ..core import CheckContext, Extension, parse_frontmatter
from ..loader import extension_config

EXT_ID = "disclaimer"


def run(ctx: CheckContext) -> None:
    cfg = extension_config(ctx.config, EXT_ID)
    ctx.reporter.section("Checking frontmatter disclaimer: presence")
    opt_out = set(cfg.get("opt_out", []))
    for md_path in sorted(ctx.docs_root.rglob("*.md")):
        try:
            rel = md_path.relative_to(ctx.docs_root).as_posix()
        except ValueError:
            rel = str(md_path)
        if rel in opt_out:
            ctx.reporter.ok(f"{rel} (opt-out)")
            continue
        if parse_frontmatter(md_path).has("disclaimer"):
            ctx.reporter.ok(f"{rel} has disclaimer:")
        else:
            ctx.reporter.fail(f"{rel} missing 'disclaimer:' frontmatter")


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="every .md under docs root carries `disclaimer:` frontmatter",
        run=run,
    ))
