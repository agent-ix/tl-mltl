#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import json
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


def healthy_output(evidence_dir: Path, name: str) -> str:
    if name == "make-ci":
        count = finalizer.expected_rust_tests(evidence_dir)
        return (
            f"test result: ok. {count} passed; 0 failed; 0 ignored\n"
            "all 10 mandatory local-CI targets propagate failures\n"
            "all 3 evidence-policy behavior tests passed\n"
            "Coverage: 49/49 rows backed (100%)\n"
            "licenses ok\n"
            "sources ok\n"
            + "\n".join(finalizer.MAKE_CI_SIGNATURES)
            + "\n"
        )
    if name == "make-spec":
        return "Coverage: 49/49 rows backed (100%)\nspec gate passed\n"
    if name == "quire-coverage":
        return "Coverage: 49/49 rows backed (100%)\n"
    if name == "rustdoc":
        return "Generated /tmp/doc/tl_mltl/index.html\n"
    if name == "default-dependencies":
        return "tl-mltl v0.1.0 (/tmp/tl-mltl)\n"
    if name in {
        "input-schema",
        "manifest-schema",
        "pgm01-schema",
        "pgm01-validator",
        "sealed-pgm01-schema",
        "sealed-pgm01-validator",
    }:
        return '{"errors": [], "valid": true}\n'
    return ""


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

    with tempfile.TemporaryDirectory(prefix="tl-mltl-qualified-") as directory:
        evidence_dir = Path(directory)
        revision = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        (evidence_dir / "source-revision.txt").write_text(revision + "\n", encoding="utf-8")
        (evidence_dir / "collection-input.json").write_text(
            json.dumps({"qualificationProfile": finalizer.evidence_profile.PROFILE}) + "\n",
            encoding="utf-8",
        )
        parameters = finalizer.historical_parameters_digest(revision)
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps(
                {
                    "result": {"status": "inconclusive"},
                    "parametersDigest": {"value": parameters},
                }
            )
            + "\n",
            encoding="utf-8",
        )
        for name in finalizer.CHECKS:
            (evidence_dir / f"{name}.status.txt").write_text("0\n", encoding="utf-8")
            (evidence_dir / f"{name}.stdout").write_text(
                healthy_output(evidence_dir, name), encoding="utf-8"
            )
            (evidence_dir / f"{name}.stderr").write_text("", encoding="utf-8")

        value = finalizer.summary(evidence_dir)
        expect(value["overallStatus"] == "passed", "healthy v2 evidence did not pass")
        make_ci = (evidence_dir / "make-ci.stdout").read_text(encoding="utf-8")
        for signature in finalizer.MAKE_CI_SIGNATURES:
            (evidence_dir / "make-ci.stdout").write_text(
                make_ci.replace(signature, "missing-signature", 1), encoding="utf-8"
            )
            expect(
                finalizer.summary(evidence_dir)["overallStatus"] == "failed",
                f"missing make-ci signature was accepted: {signature}",
            )
        (evidence_dir / "diff-integrity.stdout").write_text(
            "unexpected diff diagnostic\n", encoding="utf-8"
        )
        expect(
            finalizer.summary(evidence_dir)["overallStatus"] == "failed",
            "nonempty diff-integrity output was accepted",
        )
        (evidence_dir / "diff-integrity.stdout").write_text("", encoding="utf-8")
        (evidence_dir / "default-dependencies.stdout").write_text(
            "tl-mltl v0.1.0 (/tmp/tl-mltl)\nserde v1.0.0\n", encoding="utf-8"
        )
        expect(
            finalizer.summary(evidence_dir)["overallStatus"] == "failed",
            "nonempty default dependency set was accepted",
        )
        (evidence_dir / "default-dependencies.stdout").write_text(
            healthy_output(evidence_dir, "default-dependencies"), encoding="utf-8"
        )
        count = finalizer.expected_rust_tests(evidence_dir)
        (evidence_dir / "make-ci.stdout").write_text(
            make_ci.replace(f"ok. {count} passed", f"ok. {count + 1} passed", 1),
            encoding="utf-8",
        )
        expect(
            finalizer.summary(evidence_dir)["overallStatus"] == "failed",
            "Rust test census drift was accepted",
        )
        (evidence_dir / "make-ci.stdout").write_text(make_ci, encoding="utf-8")
        (evidence_dir / "rustdoc.status.txt").unlink()
        missing = finalizer.summary(evidence_dir)
        expect(
            next(item for item in missing["outcomes"] if item["name"] == "rustdoc")[
                "status"
            ]
            == "inconclusive",
            "missing mandatory retained lane was not inconclusive",
        )
        (evidence_dir / "rustdoc.status.txt").write_text("0\n", encoding="utf-8")
        expect(
            not finalizer.validate_parameter_identity(evidence_dir),
            "source-derived parameter identity was not accepted",
        )
        envelope = json.loads(
            (evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8")
        )
        envelope["parametersDigest"]["value"] = "0" * 64
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps(envelope) + "\n", encoding="utf-8"
        )
        expect(
            bool(finalizer.validate_parameter_identity(evidence_dir)),
            "forged parametersDigest was accepted",
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
