#!/usr/bin/env python3
"""Resolve active and explicitly retracted evidence qualification profiles."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROFILE = "tl-mltl.evidence-qualification/v2"
RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"


def retracted_records(
    registry: Path = RETRACTIONS, evidence_root: Path = ROOT / "evidence"
) -> set[str]:
    value = json.loads(registry.read_text(encoding="utf-8"))
    if value.get("schemaVersion") != "tl-mltl.evidence-retractions/v2":
        raise ValueError("evidence retraction registry has an unknown schema")
    records = value.get("records")
    if not isinstance(records, dict) or not all(
        isinstance(name, str)
        and isinstance(item, dict)
        and set(item)
        == {"disposition", "outerManifestSha256", "reason", "sourceRevision"}
        and item.get("disposition") == "legacy-unqualified"
        and isinstance(item.get("outerManifestSha256"), str)
        and len(item["outerManifestSha256"]) == 64
        and isinstance(item.get("reason"), str)
        and bool(item["reason"])
        and isinstance(item.get("sourceRevision"), str)
        and len(item["sourceRevision"]) == 40
        for name, item in records.items()
    ):
        raise ValueError("evidence retraction registry has a malformed record map")
    for name, item in records.items():
        record = evidence_root / name
        outer = evidence_root / f"{name}.sha256"
        if not record.is_dir() or not outer.is_file():
            raise ValueError(f"retraction names an unavailable record: {name}")
        observed_revision = (record / "source-revision.txt").read_text(encoding="utf-8").strip()
        if observed_revision != item["sourceRevision"]:
            raise ValueError(f"retraction source revision disagrees with record: {name}")
        observed_outer = hashlib.sha256(outer.read_bytes()).hexdigest()
        if observed_outer != item["outerManifestSha256"]:
            raise ValueError(f"retraction outer manifest digest disagrees with record: {name}")
        collection_input = json.loads(
            (record / "collection-input.json").read_text(encoding="utf-8")
        )
        if collection_input.get("qualificationProfile") == PROFILE:
            raise ValueError(f"legacy disposition cannot retract qualification-v2: {name}")
    return set(records)


def resolve_profile(evidence_dir: Path) -> str:
    if evidence_dir.name in retracted_records():
        return "retracted"
    value = json.loads((evidence_dir / "collection-input.json").read_text(encoding="utf-8"))
    if value.get("qualificationProfile") != PROFILE:
        return "inconclusive"
    revision = (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()
    result = subprocess.run(
        ["/usr/bin/git", "cat-file", "-e", f"{revision}:tools.lock"],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    return "v2" if result.returncode == 0 else "inconclusive"
