"""Core data structures and helpers for the DOCSPACES validator.

This module is policy-free. It exposes:

- Frontmatter parsing (shallow, top-level scalars + scalar lists).
- The Reporter (PASS / FAIL accounting).
- CheckContext, the value passed to every check.
- Extension, the registration record for plugins.
- Helpers for resolving member paths, walking git, and locating
  documents under the docs root.

Anything specific to a particular check lives in `checks.py` (universal
core) or `extensions/*.py` (extensions).
"""
from __future__ import annotations

import re
import subprocess
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Optional

# -----------------------------------------------------------------------------
# Frontmatter
# -----------------------------------------------------------------------------

_FM_KEY_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*)\s*:\s*(.*)$")


@dataclass
class Frontmatter:
    """Shallow view of YAML frontmatter.

    Only top-level scalar keys and top-level scalar lists are surfaced.
    Nested mappings are reported via `has(parent_key)` only; their
    contents are not parsed.
    """

    fields: dict[str, str] = field(default_factory=dict)
    counts: dict[str, int] = field(default_factory=dict)
    lists: dict[str, list[str]] = field(default_factory=dict)

    def get(self, key: str) -> str:
        return self.fields.get(key, "")

    def has(self, key: str) -> bool:
        return key in self.fields

    def count(self, key: str) -> int:
        return self.counts.get(key, 0)

    def get_list(self, key: str) -> list[str]:
        return self.lists.get(key, [])


def _strip_inline_comment(raw: str) -> str:
    return re.sub(r"\s+#.*$", "", raw).strip()


def _clean_scalar(raw: str) -> str:
    value = _strip_inline_comment(raw)
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
        value = value[1:-1]
    return value


def _parse_inline_list(raw: str) -> list[str]:
    value = _strip_inline_comment(raw)
    if not (value.startswith("[") and value.endswith("]")):
        return []
    inner = value[1:-1].strip()
    if not inner:
        return []
    parts = [part.strip() for part in inner.split(",")]
    return [_clean_scalar(part) for part in parts if _clean_scalar(part)]


def _frontmatter_bounds(lines: list[str]) -> tuple[int, int] | None:
    if not lines or lines[0].rstrip() != "---":
        return None
    for i, line in enumerate(lines[1:], start=1):
        if line.rstrip() == "---":
            return 1, i
    return None


def _parse_frontmatter_lines(lines: list[str]) -> Frontmatter:
    bounds = _frontmatter_bounds(lines)
    if bounds is None:
        return Frontmatter()
    start, end = bounds
    fm = Frontmatter()
    i = start
    while i < end:
        line = lines[i]
        if line.startswith((" ", "\t")):
            i += 1
            continue
        m = _FM_KEY_RE.match(line)
        if not m:
            i += 1
            continue
        key, raw_value = m.group(1), m.group(2)
        fm.counts[key] = fm.counts.get(key, 0) + 1
        value = _clean_scalar(raw_value)
        if key not in fm.fields:
            fm.fields[key] = value
        inline_items = _parse_inline_list(raw_value)
        if inline_items and key not in fm.lists:
            fm.lists[key] = inline_items
        j = i + 1
        block_items: list[str] = []
        while j < end and lines[j].startswith((" ", "\t")):
            stripped = lines[j].lstrip()
            if stripped.startswith("- "):
                item = _clean_scalar(stripped[2:])
                if item:
                    block_items.append(item)
            j += 1
        if block_items and key not in fm.lists:
            fm.lists[key] = block_items
        i = j
    return fm


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return ""


def parse_frontmatter(path: Path) -> Frontmatter:
    """Parse YAML frontmatter from a file on disk."""
    return _parse_frontmatter_lines(_read_text(path).splitlines())


def parse_frontmatter_from_text(text: str) -> Frontmatter:
    """Parse YAML frontmatter from a string (e.g. `git show` output)."""
    return _parse_frontmatter_lines(text.splitlines())


def body_without_frontmatter(path: Path) -> str:
    """Return the markdown body with frontmatter stripped."""
    text = _read_text(path)
    lines = text.splitlines()
    bounds = _frontmatter_bounds(lines)
    if bounds is None:
        return text
    _, end = bounds
    return "\n".join(lines[end + 1 :])


def read_text(path: Path) -> str:
    """Public reader (UTF-8 with safe fallback). Useful for extensions."""
    return _read_text(path)


# -----------------------------------------------------------------------------
# Manifest helpers
# -----------------------------------------------------------------------------

CONFIG_FILENAME = "DOCSPACES.toml"


def discover_config(start: Path) -> Optional[Path]:
    """Walk upward from *start* for DOCSPACES.toml; then probe common docs dirs."""
    current = start.resolve()
    for parent in [current, *current.parents]:
        candidate = parent / CONFIG_FILENAME
        if candidate.is_file():
            return candidate
    for parent in [current, *current.parents]:
        for rel in ("docs/open", "docs", "documentation", "site"):
            candidate = parent / rel / CONFIG_FILENAME
            if candidate.is_file():
                return candidate
    return None


def load_config(path: Path) -> dict:
    with path.open("rb") as fh:
        return tomllib.load(fh)


def docspace_names(config: dict) -> list[str]:
    """Top-level docspace identifiers, in config order."""
    return list(config.get("docspace", {}).keys())


def docspace_entries(config: dict) -> Iterable[tuple[str, dict]]:
    """Yield (name, entry) for top-level docspaces that declare members."""
    for name, entry in config.get("docspace", {}).items():
        if isinstance(entry, dict) and "members" in entry:
            yield name, entry


def resolve_member_path(docs_root: Path, entry: dict, member: str) -> Path:
    directory = entry.get("directory", ".")
    if directory == ".":
        return docs_root / member
    return docs_root / directory / member


def bind_value(raw: object) -> tuple[str, Optional[str]]:
    """Normalize a bind value to (version, commit-or-None)."""
    if isinstance(raw, str):
        return raw, None
    if isinstance(raw, dict):
        v = raw.get("version", "")
        c = raw.get("commit")
        if not isinstance(v, str):
            raise ValueError(f"bind.version must be a string, got {type(v).__name__}")
        if c is not None and not isinstance(c, str):
            raise ValueError(f"bind.commit must be a string, got {type(c).__name__}")
        return v, c
    raise ValueError(f"bind value must be string or table, got {type(raw).__name__}")


# -----------------------------------------------------------------------------
# Git helpers
# -----------------------------------------------------------------------------


def git_repo_root(start: Path) -> Optional[Path]:
    try:
        result = subprocess.run(
            ["git", "-C", str(start), "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return None
    if result.returncode != 0:
        return None
    return Path(result.stdout.strip()).resolve()


def git_commit_exists(repo_root: Path, commit: str) -> bool:
    result = subprocess.run(
        ["git", "-C", str(repo_root), "cat-file", "-e", f"{commit}^{{commit}}"],
        capture_output=True,
        check=False,
    )
    return result.returncode == 0


def git_show_content(repo_root: Path, commit: str, path_in_repo: Path) -> Optional[str]:
    spec = f"{commit}:{path_in_repo.as_posix()}"
    result = subprocess.run(
        ["git", "-C", str(repo_root), "show", spec],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    return result.stdout


# -----------------------------------------------------------------------------
# Reporter
# -----------------------------------------------------------------------------


@dataclass
class Reporter:
    errors: int = 0
    passes: int = 0
    verbose: bool = False

    def ok(self, msg: str) -> None:
        self.passes += 1
        if self.verbose:
            print(f"PASS: {msg}")

    def fail(self, msg: str) -> None:
        self.errors += 1
        print(f"FAIL: {msg}")

    def section(self, title: str) -> None:
        print(f"== {title} ==")


@dataclass
class RunResult:
    """Result of running the validator programmatically."""

    passes: int
    failures: int
    selected_checks: list[str]

    @property
    def ok(self) -> bool:
        return self.failures == 0


# -----------------------------------------------------------------------------
# Check / Extension protocols
# -----------------------------------------------------------------------------


@dataclass
class CheckContext:
    """The value passed to every check (core or extension)."""

    config: dict
    docs_root: Path
    reporter: Reporter
    repo_root: Optional[Path]


@dataclass
class Extension:
    """A registered extension check.

    `requires_git=True` causes the extension to be silently skipped if
    `ctx.repo_root is None` (i.e. the docs root is not in a git tree).
    """

    id: str
    description: str
    run: "callable[[CheckContext], None]"  # type: ignore[valid-type]
    requires_git: bool = False
