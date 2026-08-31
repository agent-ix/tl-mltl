#!/usr/bin/env python3
"""Run every checked-in evidence-policy behavior test by census."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REQUIRED = {"test_evidence_tool.py", "test_failure_propagation.py", "test_tool_identity.py"}


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy checks", file=sys.stderr)
        return 2
    tests = sorted((ROOT / "scripts").glob("test_*.py"))
    observed = {test.name for test in tests}
    if observed != REQUIRED:
        print(
            f"policy test census drift: missing={sorted(REQUIRED - observed)}, "
            f"extra={sorted(observed - REQUIRED)}",
            file=sys.stderr,
        )
        return 1
    child_env = dict(os.environ)
    child_env.pop("PYTHONOPTIMIZE", None)
    for test in tests:
        result = subprocess.run([sys.executable, str(test)], cwd=ROOT, check=False, env=child_env)
        if result.returncode != 0:
            return result.returncode
    print(f"all {len(tests)} evidence-policy behavior tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
