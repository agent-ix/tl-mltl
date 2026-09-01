#!/usr/bin/env python3
"""Run every checked-in evidence-policy behavior test by census."""

from __future__ import annotations

import hashlib
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REQUIRED = {
    "test_evidence_tool.py": "02085c338d6042643746b1facfab4a9d05e291b52c7c025d439e5ec41b81f8aa",
    "test_failure_propagation.py": "705dfd9fc890e0094168b17dfe8bfb37bf8401087fc82a67d0af223cb47b39a2",
    "test_tool_identity.py": "7c9f66bcb7cafd8dbd529852ef9560cd2cfa802d76cd7b96c1767d19a0414704",
}


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy checks", file=sys.stderr)
        return 2
    tests = sorted((ROOT / "scripts").glob("test_*.py"))
    observed = {test.name for test in tests}
    if observed != set(REQUIRED):
        print(
            f"policy test census drift: missing={sorted(set(REQUIRED) - observed)}, "
            f"extra={sorted(observed - set(REQUIRED))}",
            file=sys.stderr,
        )
        return 1
    child_env = dict(os.environ)
    child_env.pop("PYTHONOPTIMIZE", None)
    for test in tests:
        actual = hashlib.sha256(test.read_bytes()).hexdigest()
        if actual != REQUIRED[test.name]:
            print(
                f"policy test content drift for {test.name}: "
                f"expected {REQUIRED[test.name]}, got {actual}",
                file=sys.stderr,
            )
            return 1
        result = subprocess.run([sys.executable, str(test)], cwd=ROOT, check=False, env=child_env)
        if result.returncode != 0:
            return result.returncode
    print(f"all {len(tests)} evidence-policy behavior tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
