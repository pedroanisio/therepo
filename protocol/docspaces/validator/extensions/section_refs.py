"""Extension: `section-refs` — cross-document section citations must resolve.

Patterns checked:
  - `parent §N.M`  /  `prior §N.M`   (against the doc's `companion_to:`)
  - `<primary-stem> §N.M`            (against the named primary's headings)
  - `<primary-filename> §N.M`        (same, by filename)

Prefixes that mark a reference as historical
(`formerly `, `corrected from `, `previously `, `historical `) cause it
to be skipped — those are intentional callouts to past section numbers.

Manifest:

    [extensions.section-refs]
    enabled = true
"""
from __future__ import annotations

import re
from pathlib import Path

from ..core import (
    CheckContext,
    Extension,
    body_without_frontmatter,
    docspace_entries,
    parse_frontmatter,
    read_text,
    resolve_member_path,
)

EXT_ID = "section-refs"
_HEADING_SECTION_RE = re.compile(r"^\s{0,3}#{1,6}\s+(?:§\s*)?([0-9]+(?:\.[0-9]+)*)\b")
_BLOCK_SECTION_RE = re.compile(r"^\s*\*\*[A-Za-z][^*]*?([0-9]+(?:\.[0-9]+)*)\b")
_ORDERED_ITEM_RE = re.compile(r"^\s{0,3}([0-9]+)\.\s+")
_PATH_TOKEN_RE = re.compile(r"((?:\./|\.\./)?[A-Za-z0-9_./-]+\.md)\b")
_HISTORICAL_PREFIXES = ("formerly ", "corrected from ", "previously ", "historical ")


def _section_index(path: Path) -> set[str]:
    refs: set[str] = set()
    current_heading: str | None = None
    for line in read_text(path).splitlines():
        m = _HEADING_SECTION_RE.match(line)
        if m:
            current_heading = m.group(1)
            refs.add(current_heading)
            continue
        m = _BLOCK_SECTION_RE.match(line)
        if m:
            refs.add(m.group(1))
            continue
        if current_heading:
            m = _ORDERED_ITEM_RE.match(line)
            if m:
                refs.add(f"{current_heading}.{m.group(1)}")
    return refs


def _extract_path_token(value: str) -> str:
    m = _PATH_TOKEN_RE.match(value)
    return m.group(1) if m else ""


def _is_historical(text: str, start: int) -> bool:
    prefix = text[max(0, start - 40) : start].lower()
    return any(marker in prefix for marker in _HISTORICAL_PREFIXES)


def run(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking primary-doc section references")

    primary_paths: dict[str, Path] = {}
    primary_sections: dict[str, set[str]] = {}
    primary_aliases: dict[str, str] = {}
    for name, entry in docspace_entries(ctx.config):
        primary = entry.get("primary")
        if not primary:
            continue
        path = resolve_member_path(ctx.docs_root, entry, primary)
        primary_paths[name] = path
        primary_sections[name] = _section_index(path)
        primary_aliases[Path(primary).stem] = name
        primary_aliases[primary] = name

    explicit_ref_re = (
        re.compile(
            r"\b("
            + "|".join(
                re.escape(alias)
                for alias in sorted(primary_aliases, key=len, reverse=True)
            )
            + r")\s+§([0-9]+(?:\.[0-9]+)*)"
        )
        if primary_aliases
        else None
    )

    for md_path in sorted(ctx.docs_root.rglob("*.md")):
        fm = parse_frontmatter(md_path)
        try:
            rel = md_path.relative_to(ctx.docs_root).as_posix()
        except ValueError:
            rel = str(md_path)
        body = body_without_frontmatter(md_path)

        companion = fm.get("companion_to")
        if companion:
            token = _extract_path_token(companion)
            target = (md_path.parent / token).resolve() if token else md_path
            target_sections = _section_index(target) if target.is_file() else set()
            for match in re.finditer(r"\b(?:parent|prior)\s+§([0-9]+(?:\.[0-9]+)*)", body):
                num = match.group(1)
                if num in target_sections:
                    ctx.reporter.ok(f"[{rel}] parent ref §{num} resolves via companion_to")
                else:
                    ctx.reporter.fail(
                        f"[{rel}] parent ref §{num} does not resolve in companion_to "
                        f"'{token or companion}'"
                    )

        if explicit_ref_re is None:
            continue
        for match in explicit_ref_re.finditer(body):
            if _is_historical(body, match.start()):
                continue
            alias, num = match.group(1), match.group(2)
            docspace_name = primary_aliases[alias]
            if num in primary_sections.get(docspace_name, set()):
                ctx.reporter.ok(f"[{rel}] {alias} §{num} resolves")
            else:
                ctx.reporter.fail(
                    f"[{rel}] {alias} §{num} missing from primary "
                    f"'{primary_paths[docspace_name].name}'"
                )


def register(api) -> None:
    api.register_extension(Extension(
        id=EXT_ID,
        description="cross-document section refs (parent §N.M, <primary> §N.M) resolve",
        run=run,
    ))
