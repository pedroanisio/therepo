"""DOCSPACES reference validator (protocol 1.0).

Public API:

    from docspaces_validator import main, run

`main()` is the CLI entry point.

`run(config_path, docs_root, ...)` returns a `RunResult` for programmatic
use (e.g. from tests).
"""
from .core import (
    CheckContext,
    Extension,
    Frontmatter,
    Reporter,
    RunResult,
    parse_frontmatter,
)
from .cli import main, run

__all__ = [
    "CheckContext",
    "Extension",
    "Frontmatter",
    "Reporter",
    "RunResult",
    "main",
    "parse_frontmatter",
    "run",
]

# Protocol version this validator implements (major.minor).
PROTOCOL_VERSION = "1.0"
