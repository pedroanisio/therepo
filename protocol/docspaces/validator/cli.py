"""CLI entry point and programmatic `run` for the DOCSPACES validator.

Usage:

    python -m docspaces_validator                     # auto-discover
    python -m docspaces_validator --verbose           # show PASS lines
    python -m docspaces_validator --config PATH       # explicit manifest
    python -m docspaces_validator --root PATH         # override docs root
    python -m docspaces_validator --list              # list docspaces
    python -m docspaces_validator --check 1,2,3       # subset of checks
    python -m docspaces_validator --extensions-dir P  # extra extensions dir

Exit codes: 0 if all selected checks pass, 1 otherwise.
"""
from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path
from typing import Optional

from .checks import UNIVERSAL_CHECK_NUMBERS, UNIVERSAL_CHECKS
from .core import (
    CONFIG_FILENAME,
    CheckContext,
    Reporter,
    RunResult,
    discover_config,
    docspace_entries,
    git_repo_root,
    load_config,
)
from .loader import (
    ExtensionRegistry,
    extension_config,
    is_extension_enabled,
    load_extensions,
    run_extension,
)


def default_extensions_dir() -> Path:
    """The directory that ships with the reference validator."""
    return Path(__file__).resolve().parent / "extensions"


def select_checks(arg: Optional[str], registry: ExtensionRegistry) -> list[str]:
    """Resolve a user `--check` arg to an ordered list of check identifiers.

    The universal core checks have stable numeric aliases (see
    UNIVERSAL_CHECK_NUMBERS). Extensions can be selected by their string id.
    """
    available_string = set(UNIVERSAL_CHECKS) | set(registry.ids())
    if not arg:
        return list(UNIVERSAL_CHECKS) + registry.ids()

    requested: list[str] = []
    for token in (t.strip() for t in arg.split(",")):
        if not token:
            continue
        if token in available_string:
            requested.append(token)
        elif token in UNIVERSAL_CHECK_NUMBERS:
            requested.append(UNIVERSAL_CHECK_NUMBERS[token])
        else:
            known = ", ".join(sorted(available_string))
            sys.exit(
                f"ERROR: unknown check '{token}'. "
                f"Known: {known} or numeric aliases {sorted(UNIVERSAL_CHECK_NUMBERS)}"
            )
    seen: set[str] = set()
    ordered: list[str] = []
    for c in requested:
        if c not in seen:
            ordered.append(c)
            seen.add(c)
    return ordered


def run(
    config_path: Path,
    docs_root: Optional[Path] = None,
    *,
    selected: Optional[list[str]] = None,
    verbose: bool = False,
    extensions_dirs: Optional[list[Path]] = None,
) -> RunResult:
    """Programmatic entry point. Returns a RunResult (use `.ok` for pass/fail)."""
    try:
        config = load_config(config_path)
    except tomllib.TOMLDecodeError as e:
        raise SystemExit(f"ERROR: failed to parse {config_path}: {e}")

    resolved_root = (docs_root or config_path.parent).resolve()
    if not resolved_root.is_dir():
        raise SystemExit(f"ERROR: docs root is not a directory: {resolved_root}")

    ext_dirs = list(extensions_dirs or [])
    ext_dirs.insert(0, default_extensions_dir())
    registry = load_extensions(ext_dirs)

    selected_ids = selected if selected is not None else (
        list(UNIVERSAL_CHECKS) + registry.ids()
    )

    reporter = Reporter(verbose=verbose)
    ctx = CheckContext(
        config=config,
        docs_root=resolved_root,
        reporter=reporter,
        repo_root=git_repo_root(resolved_root),
    )

    for check_id in selected_ids:
        if check_id in UNIVERSAL_CHECKS:
            UNIVERSAL_CHECKS[check_id](ctx)
            continue
        ext = registry.get(check_id)
        if ext is None:
            continue
        if not is_extension_enabled(config, ext.id):
            continue
        run_extension(ext, ctx)

    return RunResult(
        passes=reporter.passes,
        failures=reporter.errors,
        selected_checks=selected_ids,
    )


def _resolve_config(args: argparse.Namespace) -> Path:
    if args.config:
        path = args.config.resolve()
        if not path.is_file():
            sys.exit(f"ERROR: --config path not found: {path}")
        return path
    discovered = discover_config(Path.cwd())
    if discovered is None:
        sys.exit(
            f"ERROR: no {CONFIG_FILENAME} found from {Path.cwd()} upward. "
            f"Pass --config PATH."
        )
    return discovered


def _format_path_for_log(path: Path) -> str:
    try:
        return str(path.relative_to(Path.cwd()))
    except ValueError:
        return str(path)


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate docspace conventions for any DOCSPACES.toml repo.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            f"{CONFIG_FILENAME} is auto-discovered by walking up from CWD; "
            "pass --config to override. Universal checks: "
            + ", ".join(UNIVERSAL_CHECKS.keys())
        ),
    )
    parser.add_argument("--config", type=Path,
                        help=f"explicit path to {CONFIG_FILENAME}")
    parser.add_argument("--root", type=Path,
                        help="override documentation root (default: parent of manifest)")
    parser.add_argument("--check",
                        help="comma-separated checks (ids or numeric aliases); "
                             "default: all universal + active extensions")
    parser.add_argument("--list", action="store_true",
                        help="list declared docspaces and exit")
    parser.add_argument("--list-extensions", action="store_true",
                        help="list discovered extensions and exit")
    parser.add_argument("--verbose", action="store_true",
                        help="print PASS lines alongside failures")
    parser.add_argument("--extensions-dir", type=Path, action="append", default=[],
                        help="additional directory of extension modules "
                             "(may be passed multiple times)")
    args = parser.parse_args(argv)

    config_path = _resolve_config(args)
    try:
        config = load_config(config_path)
    except tomllib.TOMLDecodeError as e:
        sys.exit(f"ERROR: failed to parse {config_path}: {e}")

    docs_root = (args.root.resolve() if args.root else config_path.parent).resolve()
    if not docs_root.is_dir():
        sys.exit(f"ERROR: docs root is not a directory: {docs_root}")

    ext_dirs: list[Path] = [default_extensions_dir(), *args.extensions_dir]
    registry = load_extensions(ext_dirs)

    if args.list:
        print(f"Config: {config_path}")
        print(f"Docs root: {docs_root}")
        print("Docspaces:")
        for name, entry in docspace_entries(config):
            version = entry.get("version", "?")
            primary = entry.get("primary", "—")
            member_count = len(entry.get("members", []))
            print(f"  - {name}  version={version}  primary={primary}  members={member_count}")
        return 0

    if args.list_extensions:
        print("Universal checks (always run):")
        for cid in UNIVERSAL_CHECKS:
            print(f"  - {cid}")
        print()
        print("Discovered extensions:")
        for ext_id in registry.ids():
            ext = registry.get(ext_id)
            assert ext is not None
            active = "active" if is_extension_enabled(config, ext_id) else "inactive"
            cfg_present = bool(extension_config(config, ext_id))
            label = active if cfg_present else "not-configured"
            print(f"  - {ext_id} [{label}]  {ext.description}")
        return 0

    selected = select_checks(args.check, registry)

    print(f"# config: {_format_path_for_log(config_path)}")
    print(f"# docs root: {_format_path_for_log(docs_root)}")

    reporter = Reporter(verbose=args.verbose)
    ctx = CheckContext(
        config=config,
        docs_root=docs_root,
        reporter=reporter,
        repo_root=git_repo_root(docs_root),
    )

    for check_id in selected:
        if check_id in UNIVERSAL_CHECKS:
            UNIVERSAL_CHECKS[check_id](ctx)
            continue
        ext = registry.get(check_id)
        if ext is None:
            continue
        if not is_extension_enabled(config, ext.id):
            continue
        run_extension(ext, ctx)

    print()
    print("=" * 47)
    print(f"Summary: {reporter.passes} passed, {reporter.errors} failed")
    print("=" * 47)
    return 1 if reporter.errors else 0
