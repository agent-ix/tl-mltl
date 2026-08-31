#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

import build_evidence_envelope as builder
import finalize_collection as finalizer


ROOT = Path(__file__).resolve().parent.parent


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        evidence_dir = Path(directory)
        (evidence_dir / "make-ci.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text("passed\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.status.txt").write_text("125\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.stdout").write_text("ordinary-output\n", encoding="utf-8")
        (evidence_dir / "pgm01-validator.status.txt").write_text("3\n", encoding="utf-8")
        outcomes = {item["name"]: item for item in builder.command_outcomes(evidence_dir)}
        expect(
            outcomes["make-ci"] == {"name": "make-ci", "status": "passed", "exitCode": 0},
            "zero make-ci status was not classified as passed",
        )
        expect(
            outcomes["pgm01-schema"]
            == {"name": "pgm01-schema", "status": "skipped-unavailable", "exitCode": 125},
            "unavailable schema gate was not classified as skipped",
        )
        expect(
            outcomes["pgm01-validator"]
            == {"name": "pgm01-validator", "status": "failed", "exitCode": 3},
            "failed validator was not classified as failed",
        )
        expect(outcomes["make-spec"]["status"] == "inconclusive", "missing command became conclusive")
        expect(builder.classify_result("final", [outcomes["make-ci"]])[0] == "inconclusive", "final envelope self-attested")
        expect(builder.classify_result("sealed-failed", [outcomes["make-ci"]])[0] == "error", "sealed failure was hidden")

        (evidence_dir / "evidence-envelope.json").write_text("{}\n", encoding="utf-8")
        for name in finalizer.CHECKS:
            (evidence_dir / f"{name}.status.txt").write_text("0\n", encoding="utf-8")
        retained = finalizer.derive_outcomes(evidence_dir, require_positive=False)
        (evidence_dir / "rustdoc.status.txt").write_text("1\n", encoding="utf-8")
        rederived = finalizer.derive_outcomes(evidence_dir, require_positive=False)
        expect(retained != rederived, "status mutation did not change rederived outcomes")
        expect(
            next(item for item in rederived if item["name"] == "rustdoc")["status"] == "failed",
            "nonzero retained command was not classified as failed",
        )

    optimized = dict(os.environ)
    optimized["PYTHONOPTIMIZE"] = "1"
    result = subprocess.run(
        ["/usr/bin/python3", "scripts/run_policy_tests.py"],
        cwd=ROOT,
        env=optimized,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    expect(result.returncode != 0, "optimized Python policy execution was accepted")
    print("evidence outcome behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
