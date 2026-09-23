"""TL-211: verify the planned past-C2PO corpus pin without claiming target replay."""

import hashlib
import json
from pathlib import Path


def main() -> None:
    root = Path(__file__).resolve().parents[2] / "corpus" / "past-c2po-v1"
    manifest = root / "manifest.json"
    expected, filename = (root / "SHA256SUMS").read_text().strip().split("  ")
    assert filename == manifest.name
    assert hashlib.sha256(manifest.read_bytes()).hexdigest() == expected
    data = json.loads(manifest.read_text())
    assert data["schemaVersion"] == "tl-mltl.past-c2po-corpus/v1"
    assert data["targetObservation"] is None
    assert len(data["cases"]) == len({case["id"] for case in data["cases"]}) == 6
    assert all(len(case["expectedSource"]) == len(data["trace"]) for case in data["cases"])


if __name__ == "__main__":
    main()
