"""Universal core checks for the DOCSPACES protocol.

These are the checks every conforming validator MUST implement. They
are policy-free: each operates on the manifest and frontmatter alone.

Repo-specific checks live under `extensions/*.py`.
"""
from __future__ import annotations

from .core import (
    CheckContext,
    bind_value,
    docspace_entries,
    docspace_names,
    git_commit_exists,
    git_show_content,
    parse_frontmatter,
    parse_frontmatter_from_text,
    resolve_member_path,
)


# -----------------------------------------------------------------------------
# Check 6: protocol-version
# -----------------------------------------------------------------------------


SUPPORTED_PROTOCOL_MAJOR = 1


def check_protocol_version(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking [protocol].docspaces version")
    protocol = ctx.config.get("protocol", {})
    declared = str(protocol.get("docspaces", "")).strip()
    if not declared:
        ctx.reporter.fail(
            "[protocol].docspaces is missing — manifest must declare a protocol "
            f"version (this validator supports major {SUPPORTED_PROTOCOL_MAJOR}.x)"
        )
        return
    major_str = declared.split(".", 1)[0]
    try:
        major = int(major_str)
    except ValueError:
        ctx.reporter.fail(
            f"[protocol].docspaces='{declared}' — major version must be an integer"
        )
        return
    if major != SUPPORTED_PROTOCOL_MAJOR:
        ctx.reporter.fail(
            f"[protocol].docspaces='{declared}' major={major} — validator supports "
            f"only major {SUPPORTED_PROTOCOL_MAJOR}.x"
        )
        return
    ctx.reporter.ok(f"protocol version '{declared}' compatible")


# -----------------------------------------------------------------------------
# Check 1: primary-versions
# -----------------------------------------------------------------------------


def check_primary_versions(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking primary doc versions match docspace declarations")
    for name, entry in docspace_entries(ctx.config):
        primary = entry.get("primary")
        declared = entry.get("version", "")
        if not primary:
            ctx.reporter.ok(f"[{name}] no primary doc declared — skipped")
            continue
        if not declared:
            ctx.reporter.fail(f"[{name}] declares primary '{primary}' but no version")
            continue
        path = resolve_member_path(ctx.docs_root, entry, primary)
        if not path.is_file():
            ctx.reporter.fail(f"docspace [{name}] primary doc missing: {path}")
            continue
        actual = parse_frontmatter(path).get("version")
        if actual == declared:
            ctx.reporter.ok(f"[{name}] primary '{primary}' version='{declared}'")
        else:
            ctx.reporter.fail(
                f"[{name}] primary '{primary}' declares version='{actual}', "
                f"DOCSPACES.toml says '{declared}'"
            )


# -----------------------------------------------------------------------------
# Check 2: members
# -----------------------------------------------------------------------------


def check_members(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking docspace members")
    for name, entry in docspace_entries(ctx.config):
        primary = entry.get("primary")
        opt_out = set(entry.get("opt_out", []))
        for member in entry.get("members", []):
            path = resolve_member_path(ctx.docs_root, entry, member)
            if not path.is_file():
                ctx.reporter.fail(f"[{name}] member '{member}' file not found: {path}")
                continue
            fm = parse_frontmatter(path)
            declared_ds = fm.get("docspace")
            if declared_ds == name:
                ctx.reporter.ok(f"[{name}] member '{member}' frontmatter OK")
            else:
                ctx.reporter.fail(
                    f"[{name}] member '{member}' frontmatter declares "
                    f"'docspace: {declared_ds}' (expected '{name}')"
                )
            if fm.count("version") > 1:
                ctx.reporter.fail(
                    f"[{name}] member '{member}' has multiple version: fields — ambiguous"
                )
            actual_version = fm.get("version")
            if member != primary and member not in opt_out:
                if actual_version in ("", "docspace"):
                    ctx.reporter.ok(
                        f"[{name}] member '{member}' inherits docspace version"
                    )
                else:
                    ctx.reporter.fail(
                        f"[{name}] member '{member}' pins version='{actual_version}' "
                        "instead of inheriting via 'version: docspace'"
                    )


# -----------------------------------------------------------------------------
# Check 3: no-orphan
# -----------------------------------------------------------------------------


def check_no_orphan_docspace(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking no orphan docspace declarations")
    known = set(docspace_names(ctx.config))
    for md_path in sorted(ctx.docs_root.rglob("*.md")):
        declared = parse_frontmatter(md_path).get("docspace")
        if not declared:
            continue
        if declared not in known:
            try:
                rel = md_path.relative_to(ctx.docs_root)
            except ValueError:
                rel = md_path
            ctx.reporter.fail(
                f"File '{rel}' declares docspace '{declared}' — not in DOCSPACES.toml"
            )


# -----------------------------------------------------------------------------
# Check 4: binds-keys
# -----------------------------------------------------------------------------


def check_binds_keys(ctx: CheckContext) -> None:
    ctx.reporter.section("Checking cross-docspace bind keys")
    known = set(docspace_names(ctx.config))
    for name, entry in docspace_entries(ctx.config):
        binds = entry.get("binds")
        if not isinstance(binds, dict) or not binds:
            continue
        for key, raw in binds.items():
            try:
                bind_value(raw)
            except ValueError as e:
                ctx.reporter.fail(f"[{name}.binds] {key}: malformed bind value — {e}")
                continue
            if key in known:
                ctx.reporter.ok(f"[{name}.binds] {key} → declared docspace")
            else:
                ctx.reporter.fail(
                    f"[{name}.binds] '{key}' references undeclared docspace"
                )


# -----------------------------------------------------------------------------
# Check 5: commit-pins
# -----------------------------------------------------------------------------


def check_commit_pins(ctx: CheckContext) -> None:
    """Verify commit-pinned binds. Silently skipped outside a git tree."""
    if ctx.repo_root is None:
        return

    pinned: list[tuple[str, str, str, str]] = []
    for name, entry in docspace_entries(ctx.config):
        binds = entry.get("binds")
        if not isinstance(binds, dict):
            continue
        for key, raw in binds.items():
            try:
                version, commit = bind_value(raw)
            except ValueError:
                continue
            if commit:
                pinned.append((name, key, version, commit))

    if not pinned:
        return

    ctx.reporter.section("Checking bind commit-pin integrity")
    for source_ds, target_ds, declared_version, commit in pinned:
        if not git_commit_exists(ctx.repo_root, commit):
            ctx.reporter.fail(
                f"[{source_ds}.binds] {target_ds} pinned commit '{commit}' "
                f"not found in local git tree"
            )
            continue
        target_entry = ctx.config.get("docspace", {}).get(target_ds, {})
        primary = target_entry.get("primary")
        if not primary:
            ctx.reporter.ok(
                f"[{source_ds}.binds] {target_ds} @ {commit[:7]} commit exists "
                f"(no primary to cross-check)"
            )
            continue
        directory = target_entry.get("directory", ".")
        primary_path_abs = (
            ctx.docs_root / primary if directory == "." else ctx.docs_root / directory / primary
        )
        try:
            path_in_repo = primary_path_abs.resolve().relative_to(ctx.repo_root)
        except ValueError:
            ctx.reporter.fail(
                f"[{source_ds}.binds] {target_ds}: primary doc path "
                f"{primary_path_abs} is not inside repo root {ctx.repo_root}"
            )
            continue
        content = git_show_content(ctx.repo_root, commit, path_in_repo)
        if content is None:
            ctx.reporter.fail(
                f"[{source_ds}.binds] {target_ds} @ {commit[:7]}: primary doc "
                f"'{path_in_repo}' did not exist at that commit"
            )
            continue
        actual_version = parse_frontmatter_from_text(content).get("version")
        if actual_version == declared_version:
            ctx.reporter.ok(
                f"[{source_ds}.binds] {target_ds} @ {commit[:7]} "
                f"version='{declared_version}' matches primary at commit"
            )
        else:
            ctx.reporter.fail(
                f"[{source_ds}.binds] {target_ds} @ {commit[:7]}: declared "
                f"version='{declared_version}' but primary's version at "
                f"that commit was '{actual_version}'"
            )


# -----------------------------------------------------------------------------
# Registration
# -----------------------------------------------------------------------------

UNIVERSAL_CHECKS: dict[str, "callable[[CheckContext], None]"] = {  # type: ignore[valid-type]
    "protocol-version": check_protocol_version,
    "primary-versions": check_primary_versions,
    "members":          check_members,
    "no-orphan":        check_no_orphan_docspace,
    "binds-keys":       check_binds_keys,
    "commit-pins":      check_commit_pins,
}

# Numeric aliases keep `--check 1,2,3` compatible with the universal core.
# Aliases 7+ are reserved for extensions and registered dynamically by
# the loader.
UNIVERSAL_CHECK_NUMBERS: dict[str, str] = {
    "1": "primary-versions",
    "2": "members",
    "3": "members",   # member-existence / docspace-tag / version-inheritance bundled
    "4": "no-orphan",
    "5": "binds-keys",
    "6": "commit-pins",
    "0": "protocol-version",
}
