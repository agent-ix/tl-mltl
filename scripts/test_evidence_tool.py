#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
from pathlib import Path

import build_evidence_envelope as builder
import finalize_collection as finalizer


ROOT = Path(__file__).resolve().parent.parent
KNOWN_REVISION = "54a96ae78a7c427dfb92de8c0a4e543864d88bdd"
KNOWN_RUST_TESTS = 21
KNOWN_PARAMETERS_DIGEST = "fa46a390408d1bc3842bbfb33d3f31d12c6c5727687623ab52c9db9a9cdaee96"


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def healthy_output(evidence_dir: Path, name: str) -> str:
    if name == "candidate-gates":
        return (
            f"test result: ok. {KNOWN_RUST_TESTS} passed; 0 failed; 0 ignored\n"
            "all 12 mandatory local/candidate-CI targets propagate failures\n"
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
    verifier = (ROOT / "scripts" / "verify_evidence.sh").read_text(encoding="utf-8")
    expect("active=0" in verifier and "if [[ $active -eq 0 ]]" in verifier, (
        "evidence verifier does not require a non-retracted active record"
    ))
    expect(
        finalizer.evidence_profile.revision_reachable(KNOWN_REVISION),
        "reviewed source revision is not reachable from a retained ref",
    )
    with tempfile.TemporaryDirectory(prefix="tl-mltl-reachability-") as directory:
        repository = Path(directory)
        subprocess.run(["/usr/bin/git", "init", "-q", "-b", "main"], cwd=repository, check=True)
        subprocess.run(
            ["/usr/bin/git", "config", "user.name", "Evidence Test"], cwd=repository, check=True
        )
        subprocess.run(
            ["/usr/bin/git", "config", "user.email", "evidence@example.invalid"],
            cwd=repository,
            check=True,
        )
        marker = repository / "marker"
        marker.write_text("reachable\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "add", "marker"], cwd=repository, check=True)
        subprocess.run(["/usr/bin/git", "commit", "-qm", "reachable"], cwd=repository, check=True)
        subprocess.run(["/usr/bin/git", "checkout", "-qb", "discarded"], cwd=repository, check=True)
        marker.write_text("dangling\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "commit", "-qam", "dangling"], cwd=repository, check=True)
        dangling = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        subprocess.run(["/usr/bin/git", "checkout", "-q", "main"], cwd=repository, check=True)
        subprocess.run(["/usr/bin/git", "branch", "-D", "discarded"], cwd=repository, check=True)
        expect(
            not finalizer.evidence_profile.revision_reachable(dangling, repository),
            "unreferenced squash-source commit was accepted as reachable",
        )
    with tempfile.TemporaryDirectory(prefix="tl-mltl-retractions-") as directory:
        root = Path(directory)
        evidence_root = root / "evidence"
        evidence_root.mkdir()
        name = "legacy-record"
        record = evidence_root / name
        record.mkdir()
        revision = "1" * 40
        (record / "source-revision.txt").write_text(revision + "\n", encoding="utf-8")
        (record / "collection-input.json").write_text("{}\n", encoding="utf-8")
        outer = evidence_root / f"{name}.sha256"
        outer.write_text("sealed record manifest\n", encoding="utf-8")
        registry = evidence_root / "RETRACTIONS.json"
        registry_value = {
            "schemaVersion": "tl-mltl.evidence-retractions/v2",
            "records": {
                name: {
                    "disposition": "legacy-unqualified",
                    "outerManifestSha256": hashlib.sha256(outer.read_bytes()).hexdigest(),
                    "reason": "legacy collector lacked qualification controls",
                    "sourceRevision": revision,
                }
            },
        }
        registry.write_text(json.dumps(registry_value) + "\n", encoding="utf-8")
        expect(
            finalizer.evidence_profile.retracted_records(registry, evidence_root) == {name},
            "bound legacy retraction was not accepted",
        )
        outer.write_text("tampered\n", encoding="utf-8")
        try:
            finalizer.evidence_profile.retracted_records(registry, evidence_root)
        except ValueError:
            pass
        else:
            raise RuntimeError("retraction accepted a changed outer manifest")
        outer.write_text("sealed record manifest\n", encoding="utf-8")
        (record / "collection-input.json").write_text(
            json.dumps({"qualificationProfile": finalizer.evidence_profile.PROFILE}) + "\n",
            encoding="utf-8",
        )
        try:
            finalizer.evidence_profile.retracted_records(registry, evidence_root)
        except ValueError:
            pass
        else:
            raise RuntimeError("legacy disposition retracted qualification-v2 evidence")

    with tempfile.TemporaryDirectory() as directory:
        evidence_dir = Path(directory)
        (evidence_dir / "candidate-gates.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "candidate-gates.stdout").write_text("passed\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.status.txt").write_text("125\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.stdout").write_text("ordinary-output\n", encoding="utf-8")
        (evidence_dir / "pgm01-validator.status.txt").write_text("3\n", encoding="utf-8")
        outcomes = {item["name"]: item for item in builder.command_outcomes(evidence_dir)}
        expect(
            outcomes["candidate-gates"]
            == {"name": "candidate-gates", "status": "passed", "exitCode": 0},
            "zero candidate-gates status was not classified as passed",
        )
        expect(
            outcomes["pgm01-schema"]
            == {"name": "pgm01-schema", "status": "failed", "exitCode": 125},
            "status 125 without the skip marker was not failed",
        )
        (evidence_dir / "pgm01-schema.stdout").write_text(
            "skipped-unavailable\n", encoding="utf-8"
        )
        (evidence_dir / "pgm01-schema.stderr").write_text("", encoding="utf-8")
        explicitly_skipped = {
            item["name"]: item for item in builder.command_outcomes(evidence_dir)
        }
        expect(
            explicitly_skipped["pgm01-schema"]["status"] == "skipped-unavailable",
            "paired status-125 skip marker was not preserved",
        )
        expect(
            outcomes["pgm01-validator"]
            == {"name": "pgm01-validator", "status": "failed", "exitCode": 3},
            "failed validator was not classified as failed",
        )
        expect(outcomes["make-spec"]["status"] == "inconclusive", "missing command became conclusive")
        expect(builder.classify_result("final", [outcomes["candidate-gates"]])[0] == "inconclusive", "final envelope self-attested")
        expect(builder.classify_result("sealed-failed", [outcomes["candidate-gates"]])[0] == "error", "sealed failure was hidden")

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
        revision = KNOWN_REVISION
        (evidence_dir / "source-revision.txt").write_text(revision + "\n", encoding="utf-8")
        expected_tools = finalizer.tool_identity.validate_lock(
            json.loads(finalizer.git_text(revision, "tools.lock")),
            required=finalizer.tool_identity.LEGACY_REQUIRED,
            expected_target=None,
        )
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(
                {
                    "qualificationProfile": finalizer.evidence_profile.PROFILE,
                    "tools": {"identities": expected_tools},
                }
            )
            + "\n",
            encoding="utf-8",
        )
        parameters = finalizer.historical_parameters_digest(revision)
        expect(
            finalizer.expected_rust_tests(evidence_dir) == KNOWN_RUST_TESTS,
            "source-derived Rust test census disagrees with the known-answer revision",
        )
        expect(
            parameters == KNOWN_PARAMETERS_DIGEST,
            "source-derived parameter digest disagrees with the known-answer revision",
        )
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
        expect(
            not finalizer.validate_tool_identity(evidence_dir),
            "independently retained tool identities did not match the source lock",
        )
        forged_collection = json.loads(
            (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
        )
        forged_collection["tools"]["identities"]["cargo"]["sha256"] = "0" * 64
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(forged_collection) + "\n", encoding="utf-8"
        )
        expect(
            bool(finalizer.validate_tool_identity(evidence_dir)),
            "forged observed tool identity was accepted",
        )
        forged_collection["tools"]["identities"] = expected_tools
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(forged_collection) + "\n", encoding="utf-8"
        )
        make_ci = (evidence_dir / "candidate-gates.stdout").read_text(encoding="utf-8")
        for signature in finalizer.MAKE_CI_SIGNATURES:
            (evidence_dir / "candidate-gates.stdout").write_text(
                make_ci.replace(signature, "missing-signature", 1), encoding="utf-8"
            )
            expect(
                finalizer.summary(evidence_dir)["overallStatus"] == "failed",
                f"missing candidate-gates signature was accepted: {signature}",
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
        (evidence_dir / "candidate-gates.stdout").write_text(
            make_ci.replace(f"ok. {count} passed", f"ok. {count + 1} passed", 1),
            encoding="utf-8",
        )
        expect(
            finalizer.summary(evidence_dir)["overallStatus"] == "failed",
            "Rust test census drift was accepted",
        )
        (evidence_dir / "candidate-gates.stdout").write_text(make_ci, encoding="utf-8")
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

        # The retained summary is the qualification verdict: passed, failed,
        # and inconclusive must produce three distinct process exit classes.
        envelope["parametersDigest"]["value"] = parameters
        envelope["result"]["status"] = "inconclusive"
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps(envelope) + "\n", encoding="utf-8"
        )
        for name in finalizer.CHECKS:
            status = evidence_dir / f"{name}.status.txt"
            if not status.exists():
                status.write_text("0\n", encoding="utf-8")
        (evidence_dir / "candidate-gates.stdout").write_text(make_ci, encoding="utf-8")
        passed = finalizer.summary(evidence_dir)
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(passed, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        result = subprocess.run(
            ["/usr/bin/python3", "scripts/finalize_collection.py", "--check", str(evidence_dir)],
            cwd=ROOT,
            check=False,
            capture_output=True,
        )
        expect(result.returncode == 0, "passing retained summary did not exit zero")

        (evidence_dir / "rustdoc.status.txt").write_text("1\n", encoding="utf-8")
        envelope["result"]["status"] = "error"
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps(envelope) + "\n", encoding="utf-8"
        )
        failed = finalizer.summary(evidence_dir)
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(failed, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        result = subprocess.run(
            ["/usr/bin/python3", "scripts/finalize_collection.py", "--check", str(evidence_dir)],
            cwd=ROOT,
            check=False,
            capture_output=True,
        )
        expect(result.returncode == 1, "failed retained summary did not exit with failure")

        (evidence_dir / "rustdoc.status.txt").unlink()
        envelope["result"]["status"] = "inconclusive"
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps(envelope) + "\n", encoding="utf-8"
        )
        inconclusive = finalizer.summary(evidence_dir)
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(inconclusive, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        result = subprocess.run(
            ["/usr/bin/python3", "scripts/finalize_collection.py", "--check", str(evidence_dir)],
            cwd=ROOT,
            check=False,
            capture_output=True,
        )
        expect(result.returncode == 3, "inconclusive summary lacked its distinct exit class")

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
