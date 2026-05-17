#!/usr/bin/env python3
# =============================================================================
# doc_docspace_check.py — Thin shim for the DOCSPACES reference validator.
#
# The real implementation now lives at protocol/docspaces/validator/. This
# shim preserves the legacy invocation path (`scripts/doc_docspace_check.py`
# from this repo's root) so existing pre-commit hooks and CI keep working.
#
# For programmatic use or extension authoring, import from the validator
# package directly rather than from this shim.
#
#     python3 -m validator                 # from inside protocol/docspaces/
#     # or, in tools:
#     from validator import main; main()
#
# The DOCSPACES protocol itself: protocol/docspaces/PROTOCOL.md
# =============================================================================
"""Legacy entry point — delegates to protocol/docspaces/validator/."""
from __future__ import annotations

import sys
from pathlib import Path

_THIS = Path(__file__).resolve()
_REPO_ROOT = _THIS.parent.parent
_VALIDATOR_PARENT = _REPO_ROOT / "protocol" / "docspaces"

if not (_VALIDATOR_PARENT / "validator" / "__init__.py").is_file():
    sys.exit(
        f"ERROR: DOCSPACES validator not found at {_VALIDATOR_PARENT}/validator/. "
        "Has the protocol/ directory been deleted?"
    )

sys.path.insert(0, str(_VALIDATOR_PARENT))

from validator import main  # noqa: E402

if __name__ == "__main__":
    sys.exit(main())
