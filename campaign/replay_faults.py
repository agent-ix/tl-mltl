"""Fault replay of a retained EA/Quoin Campaign in disposable source checkouts.

This is a diagnostic over a supplied run, not a Campaign producer or receipt.
The Quoin executable must implement ``measurement campaign verify``.
"""

from __future__ import annotations

import argparse
import copy
import json
import os
import subprocess
import tempfile
from pathlib import Path


def load(path: Path) -> dict:
    return json.loads(path.read_text())


def save(path: Path, value: dict) -> None:
    replacement = path.with_name(path.name + ".replacement")
    replacement.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")))
    os.replace(replacement, path)


def copied_store(source: Path, destination: Path) -> None:
    # Content addressed files are shared only until a fault is injected. All
    # writes below replace or unlink a copied directory entry, never a link.
    for current, _, files in os.walk(source):
        relative = Path(current).relative_to(source)
        output = destination / relative
        output.mkdir(parents=True, exist_ok=True)
        for name in files:
            original, copied = Path(current) / name, output / name
            if copied.exists():
                if copied.read_bytes() != original.read_bytes():
                    raise ValueError(f"checkout conflicts with retained evidence: {copied}")
            else:
                os.link(original, copied)


def replace_bytes(path: Path) -> None:
    original = path.read_bytes()
    if not original:
        raise ValueError(f"cannot alter empty retained artifact: {path}")
    replacement = path.with_name(path.name + ".replacement")
    replacement.write_bytes(bytes([original[0] ^ 1]) + original[1:])
    os.replace(replacement, path)


def mutate(case: str, repo: Path, run_id: str, target: dict) -> None:
    store = repo / "spec/evidence/campaigns"
    run_path = store / "runs" / f"{run_id}.json"
    run = load(run_path)
    attempts = run["attempts"]
    selected = next(row for row in attempts if row["member"] == target["member"]
                    and row["index"] == target["index"])
    if case == "omission":
        attempts.remove(selected)
        run["verdict"] = "inconclusive"
    elif case == "repeated_attempt":
        attempts.append(copy.deepcopy(selected))
        run["verdict"] = "rejected"
    elif case == "absent_checker":
        (store / "domain-verdicts" / f'{selected["domainVerdictDigest"]}.json').unlink()
        run["verdict"] = "inconclusive"
    elif case == "altered_raw":
        artifact = selected["rawArtifacts"][0]
        replace_bytes(store / "raw" / f'{artifact["digest"]}.bin')
        run["verdict"] = "rejected"
    elif case == "stale_source":
        run["sourceGraphDigest"] = "0" * 64
        run["verdict"] = "rejected"
    else:
        raise ValueError(f"unknown fault: {case}")
    save(run_path, run)


def receipt(command: list[str]) -> tuple[dict | None, str]:
    finished = subprocess.run(command, text=True, capture_output=True, check=False)
    try:
        parsed = json.loads(finished.stdout)
    except json.JSONDecodeError:
        return None, f"exit={finished.returncode} stderr={finished.stderr.strip()}"
    # Quoin emits the receipt itself, not an envelope, for partial decisions.
    if "decision" in parsed:
        return parsed, f"exit={finished.returncode}"
    return None, f"exit={finished.returncode} output={finished.stdout[:300]}"


def outcomes(value: dict) -> dict[str, dict]:
    return {member["name"]: member for member in value["decision"]["members"]}


def check_case(case: str, baseline: dict, replay: dict | None,
               diagnostic: str, target: str, dependents: set[str]) -> dict:
    if replay is None:
        # Structural EA refusal is valid for a duplicate or stale run record.
        if case not in {"repeated_attempt", "stale_source"} or diagnostic.startswith("exit=0"):
            raise AssertionError(f"{case}: no independent receipt: {diagnostic}")
        return {"case": case, "result": "structural_refusal", "diagnostic": diagnostic}
    verdict = replay["decision"]["verdict"]
    if verdict == "accept":
        raise AssertionError(f"{case}: independent replay accepted fault")
    before, after = outcomes(baseline), outcomes(replay)
    if case not in {"repeated_attempt", "stale_source"}:
        if after[target]["verdict"] == "accept":
            raise AssertionError(f"{case}: target remained accepted")
    for name in before.keys() - {target} - dependents:
        if before[name] != after[name]:
            raise AssertionError(f"{case}: unrelated sibling {name} changed")
    return {"case": case, "result": verdict,
            "target": after.get(target), "diagnostic": diagnostic}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quoin", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True,
                        help="clean measured source checkout containing retained evidence")
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--sources", type=Path, required=True,
                        help="quoin.campaign-sources/v1 selection JSON")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source_selection = load(args.sources)
    if source_selection.get("schema") != "quoin.campaign-sources/v1":
        parser.error("--sources has the wrong schema")
    repo = args.repo.resolve()
    own_alias = [name for name, path in source_selection["sources"].items()
                 if Path(path).resolve() == repo]
    if len(own_alias) != 1:
        parser.error("--repo must appear exactly once in --sources")
    run = load(repo / "spec/evidence/campaigns/runs" / f"{args.run_id}.json")
    digest = run["definitionDigest"]
    definition = load(repo / "spec/evidence/campaigns/definitions" / f"{digest}.json")
    members = {row["name"]: row for row in definition["members"]}
    baseline_source = args.sources.resolve()
    baseline, detail = receipt([
        str(args.quoin), "measurement", "campaign", "verify", "--repo", str(repo),
        "--definition-digest", digest, "--run-id", args.run_id,
        "--sources", str(baseline_source)])
    if baseline is None:
        raise RuntimeError(f"unmodified run cannot be independently replayed: {detail}")
    accepted = outcomes(baseline)
    targets = [row for row in run["attempts"] if row["status"] == "completed"
               and row.get("domainVerdictDigest") and members[row["member"]]["required"]
               and accepted[row["member"]]["verdict"] == "accept"]
    if not targets:
        parser.error("run has no completed required member with a checker receipt")
    target = next((row for row in targets if row["member"] == "V1.independent_oracle"),
                  targets[0])
    artifact_use = {}
    for row in run["attempts"]:
        for artifact in row.get("rawArtifacts", []):
            artifact_use[artifact["digest"]] = artifact_use.get(artifact["digest"], 0) + 1
    raw_target = next((row for row in targets if row.get("rawArtifacts")
                       and artifact_use[row["rawArtifacts"][0]["digest"]] == 1), None)
    if raw_target is None:
        parser.error("run has no accepted required member with a unique raw artifact")
    with tempfile.TemporaryDirectory(prefix="tl-campaign-replay-") as temporary:
        scratch = Path(temporary)
        selected = scratch / "sources.json"
        selected.write_text(json.dumps(source_selection))
        def command(test_repo: Path) -> list[str]:
            return [str(args.quoin), "measurement", "campaign", "verify",
                    "--repo", str(test_repo), "--definition-digest", digest,
                    "--run-id", args.run_id, "--sources", str(selected)]
        if target["member"] not in outcomes(baseline):
            raise RuntimeError("selected member absent from baseline receipt")
        records = []
        for case in ("omission", "repeated_attempt", "absent_checker", "altered_raw",
                     "stale_source"):
            chosen = raw_target if case == "altered_raw" else target
            test_repo = scratch / case / "repo"
            test_repo.parent.mkdir()
            subprocess.run(["git", "clone", "--shared", "--quiet", "--no-checkout",
                            str(repo), str(test_repo)], check=True)
            revision = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"],
                                               text=True).strip()
            subprocess.run(["git", "-C", str(test_repo), "checkout", "--quiet", "--detach",
                            revision], check=True)
            copied_store(repo / "spec/evidence", test_repo / "spec/evidence")
            source_selection["sources"][own_alias[0]] = str(test_repo)
            save(selected, source_selection)
            mutate(case, test_repo, args.run_id, chosen)
            replay, diagnostic = receipt(command(test_repo))
            dependents = set()
            while True:
                extended = {name for name, member in members.items()
                            if set(member.get("dependsOn", [])) &
                            ({chosen["member"]} | dependents)}
                if extended <= dependents:
                    break
                dependents |= extended
            records.append(check_case(case, baseline, replay, diagnostic,
                                      chosen["member"], dependents))
        args.output.write_text(json.dumps({
            "schema": "tl-mltl.campaign-fault-replay/v1", "runId": args.run_id,
            "definitionDigest": digest, "baselineVerdict": baseline["decision"]["verdict"],
            "cases": records,
            "uncovered": ["producer_false_pass_semantics: requires a fresh, self-consistent "
                          "producer/collection/checker fixture; changing an existing result "
                          "only tests content-addressed seals"],
        }, indent=2) + "\n")


if __name__ == "__main__":
    main()
