"""Extension discovery and registration for the DOCSPACES validator.

The loader walks one or more extension directories, imports each
`*.py` module (skipping `_*.py`), and calls its `register(api)` entry
point. The API surface is intentionally tiny:

    api.register_extension(Extension(id=..., description=..., run=...))

Extensions self-identify by `id`. The manifest opts in by including
`[extensions.<id>]`; absence of either the table or the implementation
deactivates the extension silently.
"""
from __future__ import annotations

import importlib.util
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from .core import CheckContext, Extension


@dataclass
class ExtensionRegistry:
    extensions: dict[str, Extension] = field(default_factory=dict)

    def register_extension(self, ext: Extension) -> None:
        if ext.id in self.extensions:
            raise ValueError(f"duplicate extension id '{ext.id}'")
        self.extensions[ext.id] = ext

    def get(self, ext_id: str) -> Optional[Extension]:
        return self.extensions.get(ext_id)

    def ids(self) -> list[str]:
        return list(self.extensions.keys())


_DEFAULT_EXTENSIONS_DIR = Path(__file__).resolve().parent / "extensions"


def _load_builtin_extension(path: Path) -> object:
    """Load an extension that lives inside this package — uses normal import
    machinery so relative imports (`from ..core import …`) work."""
    import importlib

    return importlib.import_module(f".extensions.{path.stem}", package=__package__)


def _load_external_module(path: Path) -> object:
    """Load an extension from a user-supplied directory. Such modules
    cannot use intra-package relative imports; they must import the
    validator API via absolute imports (`from docspaces_validator import …`
    or `from <parent>.validator import …`)."""
    name = f"_docspaces_ext_{path.stem}"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ImportError(f"failed to load extension module: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def load_extensions(dirs: list[Path]) -> ExtensionRegistry:
    """Import every extension module from each dir, collect Extensions."""
    registry = ExtensionRegistry()
    for d in dirs:
        if not d.is_dir():
            continue
        for py in sorted(d.glob("*.py")):
            if py.name == "__init__.py" or py.name.startswith("_"):
                continue
            try:
                if d.resolve() == _DEFAULT_EXTENSIONS_DIR.resolve():
                    module = _load_builtin_extension(py)
                else:
                    module = _load_external_module(py)
            except ImportError as e:
                # External extension that can't be loaded — surface but don't crash
                # the universal core run.
                print(f"WARN: failed to load extension {py}: {e}")
                continue
            register = getattr(module, "register", None)
            if register is None:
                continue
            register(registry)
    return registry


# Legacy → canonical extension id mapping.
# Pre-1.0 manifests placed extension config under top-level tables
# (e.g. [disclaimer], [link_check]). The protocol now reserves [extensions.<id>]
# as the canonical location; the table below lets the loader honor legacy
# manifests without forcing an immediate migration. Producers of new
# manifests SHOULD use the canonical form.
LEGACY_EXTENSION_TABLES: dict[str, str] = {
    "disclaimer":       "disclaimer",
    "link_check":       "markdown-links",
    "line_refs":        "line-refs",
    "prose_claims":     "prose-claims",
    "readme_status":    "readme-status",
    "cr_registration":  "cr-registration",
}


def _resolve_extension_cfg(config: dict, ext_id: str) -> Optional[dict]:
    """Return the extension's config table, checking canonical then legacy locations."""
    ext_cfg = config.get("extensions", {}).get(ext_id)
    if isinstance(ext_cfg, dict):
        return ext_cfg
    for legacy_key, canonical in LEGACY_EXTENSION_TABLES.items():
        if canonical != ext_id:
            continue
        legacy_cfg = config.get(legacy_key)
        if isinstance(legacy_cfg, dict):
            return legacy_cfg
    return None


def is_extension_enabled(config: dict, ext_id: str) -> bool:
    """True iff [extensions.<id>] (or its legacy alias) is present and enabled.

    Activation rule:
      - Canonical: `[extensions.<id>]` present with `enabled` absent or true.
      - Legacy: top-level table (per LEGACY_EXTENSION_TABLES) present, with
        `enabled = true` OR `required = true`. (Pre-1.0 manifests used
        `required` instead of `enabled`.)
    Explicit `enabled = false` always disables.
    """
    ext_cfg = _resolve_extension_cfg(config, ext_id)
    if ext_cfg is None:
        return False
    enabled = ext_cfg.get("enabled")
    required = ext_cfg.get("required")
    if enabled is False:
        return False
    # Canonical form: presence + non-false `enabled` activates.
    if "enabled" in ext_cfg:
        return enabled is True
    # Legacy form: needs an explicit `required = true` to activate.
    if "required" in ext_cfg:
        return required is True
    # Bare `[extensions.<id>]` with no flags: treat as enabled.
    return True


def extension_config(config: dict, ext_id: str) -> dict:
    """Convenience: the `[extensions.<id>]` sub-table (or its legacy alias), or {}."""
    cfg = _resolve_extension_cfg(config, ext_id)
    return cfg if isinstance(cfg, dict) else {}


def run_extension(ext: Extension, ctx: CheckContext) -> None:
    """Run an extension, honoring `requires_git`."""
    if ext.requires_git and ctx.repo_root is None:
        return
    ext.run(ctx)
