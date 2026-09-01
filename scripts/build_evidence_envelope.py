#!/usr/bin/env python3
"""Build tl-mltl's PGM-01 collection input, manifest, and envelope."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path
from typing import Any

import parameter_identity
import tool_identity


ROOT = Path(__file__).resolve().parent.parent
PGM01_POLICY_REVISION = "7dac9d8c19952412b56a0347387666e2ca81e01d"
PGM01_ENVELOPE_SCHEMA_DIGEST = (
    "0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256"
)
TOOLS_LOCK = ROOT / "tools.lock"
EVIDENCE_RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"
INPUT_SCHEMA = ROOT / "schemas" / "tl-mltl-evidence-input-v1.schema.json"
MANIFEST_SCHEMA = ROOT / "schemas" / "tl-mltl-evidence-manifest-v1.schema.json"
COLLECTOR = ROOT / "scripts" / "collect_evidence.sh"
BUILDER = Path(__file__).resolve()
SCHEMA_VALIDATOR = ROOT / "scripts" / "validate_json_schema.py"
COLLECTION_FINALIZER = ROOT / "scripts" / "finalize_collection.py"

COMMANDS = (
    "candidate-gates",
    "make-spec",
    "quire-coverage",
    "rustdoc",
    "default-dependencies",
    "diff-integrity",
    "input-schema",
    "manifest-schema",
    "pgm01-schema",
    "pgm01-validator",
)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def digest(value: str) -> dict[str, str]:
    return {"algorithm": "sha256", "value": value}


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_first_line(path: Path) -> str:
    return path.read_text(encoding="utf-8").splitlines()[0]


def command_outcomes(evidence_dir: Path) -> list[dict[str, object]]:
    outcomes: list[dict[str, object]] = []
    for name in COMMANDS:
        status_path = evidence_dir / f"{name}.status.txt"
        if not status_path.exists():
            outcomes.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        exit_code = int(status_path.read_text().strip())
        stdout_path = evidence_dir / f"{name}.stdout"
        stderr_path = evidence_dir / f"{name}.stderr"
        skipped = (
            exit_code == 125
            and stdout_path.exists()
            and stdout_path.read_text(encoding="utf-8", errors="replace").strip()
            == "skipped-unavailable"
            and stderr_path.exists()
            and not stderr_path.read_text(encoding="utf-8", errors="replace").strip()
        )
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "passed" if exit_code == 0 else "failed"
                ),
                "exitCode": exit_code,
            }
        )
    return outcomes


def classify_result(
    phase: str, outcomes: list[dict[str, object]]
) -> tuple[str, str]:
    statuses = {outcome["status"] for outcome in outcomes}
    if phase == "sealed-failed" or "failed" in statuses:
        return "error", "one or more retained tl-mltl checks failed"
    if phase in {"provisional", "final"}:
        return "inconclusive", "exact finalized-envelope validation is external or pending"
    if "inconclusive" in statuses or "skipped-unavailable" in statuses:
        return "inconclusive", "schema or governance validation is unavailable or pending"
    return "conclusive", "all retained tl-mltl checks passed"


def parameter_paths() -> tuple[Path, ...]:
    tree = set(
        subprocess.run(
            ["/usr/bin/git", "ls-files"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    )
    return tuple(ROOT / relative for relative in parameter_identity.parameter_names(tree))


def hash_parameter_files() -> str:
    state = hashlib.sha256()
    for path in parameter_paths():
        state.update(str(path.relative_to(ROOT)).encode("utf-8"))
        state.update(b"\0")
        state.update(path.read_bytes())
        state.update(b"\0")
    return state.hexdigest()


def schema_identity(name: str, path: Path) -> dict[str, object]:
    return {"id": name, "version": "v1", "digest": digest(sha256_file(path))}


def build(evidence_dir: Path, phase: str) -> None:
    evidence_dir = evidence_dir.resolve()
    relative_dir = str(evidence_dir.relative_to(ROOT))
    revision = (evidence_dir / "source-revision.txt").read_text().strip()
    source_state = (evidence_dir / "source-state.txt").read_text().strip()
    metadata = json.loads((evidence_dir / "metadata.json").read_text())
    package = next(item for item in metadata["packages"] if item["name"] == "tl-mltl")
    recorded_at = (
        dt.datetime.now(dt.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z")
    )

    collection_input = {
        "schemaVersion": "tl-mltl.evidence-input/v1",
        "qualificationProfile": "tl-mltl.evidence-qualification/v2",
        "sourceRevision": revision,
        "sourceState": source_state,
        "commands": [
            "make ci-for-evidence (candidate gates; final make ci adds evidence self-binding)",
            "make spec",
            "quire coverage --scope . --strict",
            "RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features",
            "cargo tree --no-default-features --edges normal",
            f"git diff --check origin/main...{revision}",
            "python3 scripts/validate_json_schema.py INPUT_SCHEMA collection-input.json",
            "python3 scripts/validate_json_schema.py MANIFEST_SCHEMA evidence-manifest.json",
            "python3 scripts/validate_json_schema.py PGM01_SCHEMA evidence-envelope.json",
            "PGM01_PYTHON PGM01_VALIDATOR --fixture evidence-envelope.json",
            "python3 scripts/build_evidence_envelope.py EVIDENCE_DIR final",
            "python3 scripts/validate_json_schema.py PGM01_SCHEMA finalized-evidence-envelope.json",
            "PGM01_PYTHON PGM01_VALIDATOR --fixture finalized-evidence-envelope.json",
            "python3 scripts/finalize_collection.py EVIDENCE_DIR",
        ],
        "tools": {
            "cargo": read_first_line(evidence_dir / "cargo-version.txt"),
            "jsonschema": (evidence_dir / "jsonschema-version.txt").read_text().strip(),
            "python": (evidence_dir / "python-version.txt").read_text().strip(),
            "quire": json.loads((evidence_dir / "quire-provenance.json").read_text())["cli"][
                "version"
            ],
            "rustc": read_first_line(evidence_dir / "rustc-version.txt"),
            "identities": {
                name: {
                    "path": (evidence_dir / f"tool-{name}-path.txt").read_text().strip(),
                    "sha256": (evidence_dir / f"tool-{name}-sha256.txt").read_text().strip(),
                }
                for name in tool_identity.REQUIRED
            },
        },
        "pgm01": {
            "policy": "ix://agent-ix/quire-contract-ir/PGM-01",
            "candidateRevision": PGM01_POLICY_REVISION,
            "envelopeSchema": "quire.derivation-evidence/v1",
            "envelopeSchemaDigest": digest(PGM01_ENVELOPE_SCHEMA_DIGEST),
        },
        "dependencies": {
            "tlSyntaxRevision": "740182f13b84858008d6f176f75136737d405c1b",
            "cargoLockDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "corpus": {
            "revision": "tl-mltl-corpus/v1",
            "sharedManifestDigest": digest(
                sha256_file(ROOT / "corpus" / "tl-syntax-v1" / "manifest.json")
            ),
            "formulaSchema": schema_identity(
                "tl-syntax.formula",
                ROOT / "corpus" / "tl-syntax-v1" / "schema" / "formula-v1.schema.json",
            ),
            "propositionMapSchema": schema_identity(
                "tl-syntax.proposition-map",
                ROOT
                / "corpus"
                / "tl-syntax-v1"
                / "schema"
                / "proposition-map-v1.schema.json",
            ),
            "differentialManifestDigest": digest(
                sha256_file(ROOT / "corpus" / "r2u2-v4.2" / "manifest.json")
            ),
            "differentialReportDigest": digest(
                sha256_file(ROOT / "corpus" / "r2u2-v4.2" / "differential-report.json")
            ),
        },
        "externalTools": {
            "c2po": {
                "version": "4.1.0",
                "sourceRevision": "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a",
                "executableDigest": digest(
                    "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12"
                ),
            },
            "r2u2": {
                "version": "4.2-release",
                "sourceRevision": "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a",
                "executableDigest": digest(
                    "6b98ee5cfcad7073eef49a333b00be1e5b512ed9d3bed6b4e07418357a87ab92"
                ),
                "configurationDigest": digest(
                    "234c5f0a1fb827c1ef10cab4ed4ae9ce8ffdb07e6863c6fa9522730e49ca0da8"
                ),
            },
        },
    }
    input_path = evidence_dir / "collection-input.json"
    write_json(input_path, collection_input)

    excluded = {
        "collection-input.json",
        "evidence-envelope.json",
        "evidence-manifest.json",
        "collection-summary.json",
    }
    artifacts = []
    for path in sorted(evidence_dir.iterdir(), key=lambda item: item.name):
        if path.is_file() and path.name not in excluded:
            artifacts.append(
                {"path": path.name, "sha256": sha256_file(path), "size": path.stat().st_size}
            )

    outcomes = command_outcomes(evidence_dir)
    any_failed = any(outcome["status"] == "failed" for outcome in outcomes)
    any_inconclusive = any(
        outcome["status"] in {"inconclusive", "skipped-unavailable"}
        for outcome in outcomes
    )
    limitations = [
        "the merged PGM-01 policy's manual-dispatch CI was not dispatched",
        "R2U2 differential evidence covers eight declared supported formula/time cases and one explicit unsupported profile case",
        "the differential run does not qualify R2U2 or a consuming monitor",
        "independent CODEOWNER approval and the human source-release decision are pending",
        "local deterministic checks were collected on one host target; cross-platform release review remains pending",
    ]
    if any_failed:
        limitations.append("one or more locally collected commands failed")
    if any_inconclusive:
        limitations.append("one or more schema or governance checks were unavailable or pending")
    if phase == "provisional":
        limitations.append("this provisional envelope precedes its own schema and governance checks")
    if phase == "final":
        limitations.append(
            "the exact finalized envelope is validated externally and does not self-attest"
        )
    if phase == "sealed-failed":
        limitations.append("validation of the finalized envelope failed; see sealed validation artifacts")

    manifest = {
        "schemaVersion": "tl-mltl.evidence-manifest/v1",
        "sourceRevision": revision,
        "collectedAt": recorded_at,
        "outcomes": outcomes,
        "artifacts": artifacts,
        "limitations": limitations,
    }
    manifest_path = evidence_dir / "evidence-manifest.json"
    write_json(manifest_path, manifest)

    host = next(
        line.split(": ", 1)[1]
        for line in (evidence_dir / "rustc-version.txt").read_text().splitlines()
        if line.startswith("host: ")
    )
    result_status, result_summary = classify_result(phase, outcomes)
    envelope = {
        "schemaVersion": "quire.derivation-evidence/v1",
        "recordId": evidence_dir.name,
        "recordedAt": recorded_at,
        "producer": {
            "name": "tl-mltl-evidence-collector",
            "version": package["version"],
            "sourceRevision": revision,
            "executableDigest": digest(sha256_file(COLLECTOR)),
            "invocation": ["bash", "scripts/collect_evidence.sh", relative_dir],
        },
        "inputs": [
            {
                "role": "evidence-collection-input",
                "uri": "collection-input.json",
                "mediaType": "application/json",
                "schema": schema_identity("tl-mltl.evidence-input", INPUT_SCHEMA),
                "contentDigest": digest(sha256_file(input_path)),
            }
        ],
        "backend": {
            "kind": "none",
            "reason": "deterministic evidence packaging; invoked tools are identified in the input",
        },
        "outputs": [
            {
                "role": "tl-mltl-evidence-manifest",
                "uri": "evidence-manifest.json",
                "mediaType": "application/json",
                "schema": schema_identity("tl-mltl.evidence-manifest", MANIFEST_SCHEMA),
                "contentDigest": digest(sha256_file(manifest_path)),
            }
        ],
        "parametersDigest": digest(hash_parameter_files()),
        "environment": {
            "targetTriple": host,
            "operatingSystem": platform.platform(),
            "toolchain": collection_input["tools"]["rustc"],
            "dependenciesDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "provenance": {
            "repository": "https://github.com/agent-ix/tl-mltl",
            "sourceRevision": revision,
            "candidateRevision": revision,
            "contributionMethod": "agent-assisted",
            "reviewers": ["@kreneskyp"],
        },
        "result": {
            "status": result_status,
            "summary": result_summary,
            "requirementRefs": ["PGM-01-R08", "PGM-01-R09", "MP-001"],
        },
        "extensions": {
            "dev.agent-ix.tl-mltl": {
                "componentClass": "analysis-evidence-tool",
                "corpusRevision": "tl-mltl-corpus/v1",
                "envelopeSchemaDigest": PGM01_ENVELOPE_SCHEMA_DIGEST,
                "pgm01CandidateRevision": PGM01_POLICY_REVISION,
                "reviewState": "pending",
                "sourceState": source_state,
            }
        },
    }
    write_json(evidence_dir / "evidence-envelope.json", envelope)


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print("usage: build_evidence_envelope.py EVIDENCE_DIR [PHASE]", file=sys.stderr)
        return 2
    phase = sys.argv[2] if len(sys.argv) == 3 else "final"
    if phase not in {"provisional", "final", "sealed-failed"}:
        print(f"unknown evidence build phase: {phase}", file=sys.stderr)
        return 2
    build(Path(sys.argv[1]), phase)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
