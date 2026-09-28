"""Run a disposable one-member Campaign with an exit-zero, false native result.

This diagnostic never edits the published Campaign, measured checkout, or
content-addressed receipts. It makes and commits a local scratch source tree,
then lets Quoin produce and independently verify fresh evidence in that tree.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import subprocess
from pathlib import Path


MEMBER = "V1.independent_oracle"
PROCEDURE = "campaign/procedures/v1-independent-oracle.json"
FAILED_SUMMARY = ("running 1 test\n"
                  "test independent_oracle_matches_small_finite_and_origin_complete_words ... FAILED\n"
                  "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; "
                  "0 filtered out; finished in 0.01s\n")
PRODUCER_CODE = f"import sys;sys.stdout.write({FAILED_SUMMARY!r})"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(*args: str, cwd: Path | None = None, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(args, cwd=cwd, env=env, text=True).strip()


def transform(definition: dict, config: dict, revision: str, tree_digest: str,
              checkout: Path, python: Path, python_version: str,
              checker: Path) -> tuple[dict, dict]:
    """Select V1 and replace its producer binding for the scratch source."""
    if config.get("schema") != "quoin.campaign-run-config/v1":
        raise ValueError("expected quoin.campaign-run-config/v1")
    original = next((item for item in definition["members"] if item["name"] == MEMBER), None)
    if original is None or MEMBER not in config["members"]:
        raise ValueError("V1.independent_oracle is absent from definition or config")
    prepared_definition = copy.deepcopy(definition)
    prepared_definition["id"] = "tl-v1-false-pass-diagnostic"
    prepared_definition["members"] = [copy.deepcopy(original)]
    own = next((item for item in prepared_definition["sourceGraph"]
                if item["repository"] == "tl-mltl"), None)
    if own is None:
        raise ValueError("tl-mltl source alias is absent")
    own.update(revision=revision, digest=tree_digest)
    prepared_config = copy.deepcopy(config)
    prepared_config["sources"]["tl-mltl"] = str(checkout)
    prepared_config["members"] = {MEMBER: copy.deepcopy(config["members"][MEMBER])}
    binding = prepared_config["members"][MEMBER]
    binding["producer"]["producer"].update(
        name="python3", version=python_version, sourceRevision=revision,
        executable=str(python), executableDigest=sha256(python))
    binding["checker"]["producer"].update(
        sourceRevision=revision, executable=str(checker), executableDigest=sha256(checker))
    selector = binding.get("environmentSources", {}).get("TL_MLTL_SOURCE_REVISION")
    if selector != {"kind": "source_revision", "repository": "tl-mltl"}:
        raise ValueError("V1 producer lacks the tl-mltl source-revision environment selector")
    environment = binding["producer"].get("environment", {})
    if "TL_MLTL_SOURCE_REVISION" not in environment:
        raise ValueError("V1 producer lacks its bound source-revision environment")
    environment["TL_MLTL_SOURCE_REVISION"] = revision
    binding["sourceRemotes"]["tl-mltl"] = f"file://{checkout}"
    return prepared_definition, prepared_config


def parse_receipt(process: subprocess.CompletedProcess[str], action: str) -> dict:
    try:
        receipt = json.loads(process.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"{action} returned no JSON receipt: {process.stderr}") from error
    if "decision" not in receipt:
        raise RuntimeError(f"{action} returned no Campaign decision: {process.stdout[:400]}")
    return receipt


def assert_false_pass(repo: Path, run_id: str, run_receipt: dict,
                      verify_receipt: dict) -> dict:
    store = repo / "spec/evidence/campaigns"
    retained = json.loads((store / "runs" / f"{run_id}.json").read_text())
    if len(retained["attempts"]) != 1:
        raise AssertionError("diagnostic did not retain exactly one attempt")
    attempt = retained["attempts"][0]
    if attempt["member"] != MEMBER or attempt["status"] != "completed":
        raise AssertionError("false producer did not complete the selected attempt")
    result = json.loads((store / "results" / f'{attempt["resultDigest"]}.json').read_text())
    process = result["process"]
    raw = bytes(process["stdout"]["bytes"])
    if (process["terminalStatus"] != {"kind": "exit_code", "value": 0}
            or raw != FAILED_SUMMARY.encode()
            or process["stdout"]["digest"] != hashlib.sha256(raw).hexdigest()):
        raise AssertionError("producer false pass is not an exact exit-zero raw capture")
    domain = json.loads((store / "domain-verdicts" /
                         f'{attempt["domainVerdictDigest"]}.json').read_text())
    if domain["verdict"] != "reject" or "native_result_unproved" not in domain["reasons"]:
        raise AssertionError("TL checker did not reject the failing native result")
    if (run_receipt["decision"]["verdict"] != "reject"
            or verify_receipt["decision"]["verdict"] != "reject"
            or run_receipt != verify_receipt):
        raise AssertionError("Quoin run and independent replay did not agree on rejection")
    return {"schema": "tl-mltl.false-pass-diagnostic/v1", "runId": run_id,
            "definitionDigest": run_receipt["definitionDigest"],
            "runDigest": run_receipt["runDigest"], "resultDigest": attempt["resultDigest"],
            "domainVerdictDigest": attempt["domainVerdictDigest"],
            "verdict": "reject", "reason": "native_result_unproved"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--definition", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True,
                        help="current-source Quoin config with V1.independent_oracle binding")
    parser.add_argument("--quoin", type=Path, required=True)
    parser.add_argument("--python", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--run-id", default="tl-v1-false-pass-diagnostic-01")
    args = parser.parse_args()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=False)
    original = json.loads(args.definition.read_text())
    config = json.loads(args.config.read_text())
    measured = Path(config["sources"]["tl-mltl"]).resolve()
    expected = next(item["revision"] for item in original["sourceGraph"]
                    if item["repository"] == "tl-mltl")
    if run("git", "rev-parse", "HEAD", cwd=measured) != expected:
        parser.error("config's measured tl-mltl checkout is not definition's exact revision")
    scratch = output / "source"
    subprocess.run(["git", "clone", "--shared", "--quiet", "--no-checkout",
                    str(measured), str(scratch)], check=True)
    subprocess.run(["git", "checkout", "--quiet", "--detach", expected],
                   cwd=scratch, check=True)
    procedure_path = scratch / PROCEDURE
    procedure = json.loads(procedure_path.read_text())
    python_version = run(str(args.python), "--version").split()[-1]
    procedure["producerName"] = "python3"
    procedure["producerVersion"] = python_version
    procedure["arguments"] = [{"kind": "literal", "value": "-c"},
                              {"kind": "literal", "value": PRODUCER_CODE}]
    procedure_path.write_text(json.dumps(procedure, indent=2) + "\n")
    subprocess.run(["git", "add", PROCEDURE], cwd=scratch, check=True)
    subprocess.run(["git", "-c", "user.name=TL diagnostic",
                    "-c", "user.email=tl-diagnostic@example.invalid", "commit", "-qm",
                    "TL diagnostic: false native producer pass"], cwd=scratch, check=True)
    revision = run("git", "rev-parse", "HEAD", cwd=scratch)
    tree_digest = hashlib.sha256(subprocess.check_output(
        ["git", "ls-tree", "-r", "-z", "--full-tree", revision], cwd=scratch)).hexdigest()
    target = output / "target"
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(target)
    subprocess.run(["cargo", "build", "--locked", "--offline", "--features",
                    "campaign-check", "--bin", "tl_campaign_check"], cwd=scratch,
                   env=environment, check=True)
    checker = target / "debug/tl_campaign_check"
    definition, selected = transform(original, config, revision, tree_digest, scratch,
                                     args.python.resolve(), python_version, checker)
    definition_path, config_path, sources_path = (output / name for name in
                                                  ("definition.json", "config.json", "sources.json"))
    definition_path.write_text(json.dumps(definition, indent=2) + "\n")
    config_path.write_text(json.dumps(selected, indent=2) + "\n")
    sources_path.write_text(json.dumps({"schema": "quoin.campaign-sources/v1",
                                        "sources": selected["sources"]}, indent=2) + "\n")
    executed = subprocess.run([str(args.quoin), "measurement", "campaign", "run",
                               "--repo", str(scratch), "--definition", str(definition_path),
                               "--run-id", args.run_id, "--config", str(config_path)],
                              text=True, capture_output=True, check=False)
    run_receipt = parse_receipt(executed, "run")
    verified = subprocess.run([str(args.quoin), "measurement", "campaign", "verify",
                               "--repo", str(scratch), "--definition-digest",
                               run_receipt["definitionDigest"], "--run-id", args.run_id,
                               "--sources", str(sources_path)],
                              text=True, capture_output=True, check=False)
    verify_receipt = parse_receipt(verified, "verify")
    report = assert_false_pass(scratch, args.run_id, run_receipt, verify_receipt)
    report["sourceRevision"] = revision
    (output / "diagnostic.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
