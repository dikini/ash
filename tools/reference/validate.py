#!/usr/bin/env python3
"""Compatibility entrypoint for reference corpus validation.

Phase 144 task recipes use ``tools/reference/validate.py``. The underlying
validator remains ``check_frontmatter.py``; this wrapper preserves the task
contract without duplicating validation logic.
"""
from __future__ import annotations

import runpy
import sys
from pathlib import Path


if __name__ == "__main__":
    script = Path(__file__).with_name("check_frontmatter.py")
    sys.argv[0] = str(script)
    runpy.run_path(str(script), run_name="__main__")
