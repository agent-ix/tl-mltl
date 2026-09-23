"""Fault checks for the TL-223 native fuzz receipt gate."""

from __future__ import annotations

import gzip
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

MODULE_PATH = Path(__file__).with_name("run_tl223_campaign.py")
SPEC = importlib.util.spec_from_file_location("tl223_campaign", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(campaign)


class Tl223CampaignTest(unittest.TestCase):
    def test_only_a_complete_native_count_passes(self) -> None:
        complete = b"#1000000 DONE\n"
        self.assertEqual(
            campaign.classify(0, complete, 1_000_000, {}),
            ("complete", 1_000_000, "execution_budget"),
        )
        self.assertEqual(campaign.classify(0, b"#999999 DONE\n", 999_999, {})[0],
                         "smoke_only")
        self.assertEqual(campaign.classify(0, complete, 1_000_001, {})[0],
                         "incomplete")
        self.assertEqual(campaign.classify(2, complete, 1_000_000, {})[0],
                         "engine_failed")
        self.assertEqual(campaign.classify(0, complete, 1_000_000, {"crash": "abc"})[0],
                         "crash_requires_replay")
        self.assertEqual(campaign.classify(0, complete + complete, 1_000_000, {})[0],
                         "incomplete")

    def test_pair_gate_rejects_short_or_tampered_receipts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for target in campaign.TARGETS:
                for path in (*campaign.SOURCE_PATHS,
                             f"fuzz/fuzz_targets/{target}.rs"):
                    member = root / path
                    member.parent.mkdir(parents=True, exist_ok=True)
                    member.write_bytes(path.encode())
                corpus = root / "fuzz" / "corpus" / target
                corpus.mkdir(parents=True)
                (corpus / "seed").write_bytes(target.encode())
                (corpus / "SHA256SUMS").write_text(
                    f"{campaign.digest(target.encode())}  seed\n"
                )
            receipts: list[Path] = []
            with patch.object(campaign, "ROOT", root), patch.object(
                campaign, "command", side_effect=lambda argv: "" if "status" in argv else "sha"
            ):
                for target in campaign.TARGETS:
                    directory = root / f"receipt-{target}"
                    directory.mkdir()
                    (directory / "artifacts").mkdir()
                    stdout = gzip.compress(b"", mtime=0)
                    stderr = gzip.compress(b"#1000000 DONE\n", mtime=0)
                    (directory / "stdout.log.gz").write_bytes(stdout)
                    (directory / "stderr.log.gz").write_bytes(stderr)
                    report = {
                        "schema": "tl-mltl.tl223-fuzz/v1",
                        "target": target,
                        "source_revision": "sha",
                        "source_sha256": campaign.source_digests(target),
                        "corpus_sha256": campaign.corpus_digests(target),
                        "corpus_manifest_sha256": campaign.digest(
                            (root / "fuzz" / "corpus" / target / "SHA256SUMS").read_bytes()
                        ),
                        "linear_min_executions": 1_000_000,
                        "budget": {"runs": 1_000_000},
                        "observed": {"executions": 1_000_000, "exit_code": 0,
                                     "stop_reason": "execution_budget"},
                        "raw_sha256": {"stdout.log.gz": campaign.digest(stdout),
                                       "stderr.log.gz": campaign.digest(stderr)},
                        "artifact_sha256": {},
                        "replay": {"required": False, "confirmed": False},
                        "status": "complete",
                    }
                    (directory / "report.json").write_text(json.dumps(report))
                    receipts.append(directory)
                campaign.verify(receipts)
                short = json.loads((receipts[0] / "report.json").read_text())
                short["budget"]["runs"] = 999_999
                (receipts[0] / "report.json").write_text(json.dumps(short))
                with self.assertRaisesRegex(ValueError, "below Linear"):
                    campaign.verify(receipts)
                short["budget"]["runs"] = 1_000_000
                (receipts[0] / "report.json").write_text(json.dumps(short))
                (receipts[1] / "stderr.log.gz").write_bytes(gzip.compress(b"#2 DONE\n"))
                with self.assertRaisesRegex(ValueError, "raw stream"):
                    campaign.verify(receipts)


if __name__ == "__main__":
    unittest.main()
