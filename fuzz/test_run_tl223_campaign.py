"""Fault checks for the TL-223 native fuzz receipt gate."""

from __future__ import annotations

import copy
import gzip
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

MODULE_PATH = Path(__file__).with_name("run_tl223_campaign.py")
SPEC = importlib.util.spec_from_file_location("tl223_campaign", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = campaign
SPEC.loader.exec_module(campaign)


def native_log(target: str, runs: int, seed: int, files: int, seconds: int = 2) -> bytes:
    return (
        f"     Running `fuzz/target/aarch64-apple-darwin/release/{target} "
        f"-runs={runs} -seed={seed} -max_total_time=100 -max_len=256`\n"
        f"INFO: Seed: {seed}\n"
        f"INFO:        {files} files found in /private/tmp/corpus\n"
        f"INFO: seed corpus: files: {files} min: 1b max: 2b total: 3b\n"
        f"#{files + 1}\tINITED cov: 10 ft: 20\n"
        f"#{files + 2}\tNEW cov: 11 ft: 21\n"
        f"#{runs}\tDONE cov: 11 ft: 21\n"
        f"Done {runs} runs in {seconds} second(s)\n"
    ).encode()


class Tl223CampaignTest(unittest.TestCase):
    def test_only_a_coherent_native_count_passes(self) -> None:
        request = campaign.NativeRequest("lasso_differential", 1_000_000, 100, 223, 5)
        complete = native_log(request.target, request.runs, request.seed,
                              request.corpus_files)
        self.assertEqual(
            campaign.classify(0, complete, request, {}),
            ("complete", 1_000_000, "execution_budget", 2),
        )
        smoke = campaign.NativeRequest(request.target, 999_999, 100, 223, 5)
        self.assertEqual(campaign.classify(
            0, native_log(smoke.target, smoke.runs, smoke.seed, smoke.corpus_files), smoke, {}
        )[0], "smoke_only")
        for malformed in (
            b"#1000000 DONE\n",  # No native startup, progress or terminal footer.
            complete.replace(b"INFO: Seed: 223", b"INFO: Seed: 224"),
            complete.replace(b"#7\tNEW", b"#7\tUNKNOWN"),
            complete.replace(b"Done 1000000 runs", b"Done 999999 runs"),
            complete.replace(b"release/lasso_differential", b"release/other"),
        ):
            self.assertEqual(campaign.classify(0, malformed, request, {})[0],
                             "incomplete")
        self.assertEqual(campaign.classify(2, complete, request, {})[0],
                         "engine_failed")
        self.assertEqual(campaign.classify(0, complete, request, {"crash": "abc"})[0],
                         "crash_requires_replay")

    def test_pair_gate_rejects_short_forged_or_tampered_receipts(self) -> None:
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
                    stderr = gzip.compress(native_log(target, 1_000_000, 223, 1), mtime=0)
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
                        "engine": "libFuzzer",
                        "sanitizer": "address",
                        "tool_versions": {
                            "rustc": "rustc 1.100.0-nightly\nrelease: 1.100.0-nightly",
                            "cargo": "cargo 1.100.0-nightly",
                            "cargo_fuzz": "cargo-fuzz 0.13.1",
                        },
                        "command": [
                            "cargo", "fuzz", "run", "--sanitizer", "address", target,
                            f"/private/tmp/tl223-{target}-abc/corpus", "--", "-runs=1000000",
                            "-seed=223", "-max_total_time=100", "-max_len=256",
                            f"-artifact_prefix={directory / 'artifacts'}/",
                        ],
                        "linear_min_executions": 1_000_000,
                        "budget": {"runs": 1_000_000, "seconds": 100, "seed": 223},
                        "observed": {"executions": 1_000_000, "exit_code": 0,
                                     "elapsed_seconds": 3, "native_seconds": 2,
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
                report_path = receipts[0] / "report.json"
                baseline = json.loads(report_path.read_text())
                for key, bad_value, reason in (
                    ("engine", "not-libFuzzer", "engine or sanitizer"),
                    ("sanitizer", "none", "engine or sanitizer"),
                    ("tool_versions", {}, "tool versions"),
                    ("command", ["true"], "native command"),
                    ("budget", {"runs": 1_000_000, "seconds": 1, "seed": 223},
                     "native command"),
                    ("budget", {"runs": 1_000_000, "seconds": 100, "seed": 224},
                     "native command"),
                    ("budget", {"runs": 999_999, "seconds": 100, "seed": 223},
                     "native command"),
                ):
                    fault = copy.deepcopy(baseline)
                    fault[key] = bad_value
                    report_path.write_text(json.dumps(fault))
                    with self.assertRaisesRegex(ValueError, reason):
                        campaign.verify(receipts)
                report_path.write_text(json.dumps(baseline))

                # Rehash a forged DONE-only stream. The receipt still fails
                # because native startup, progress and footer are absent.
                lone_done = gzip.compress(b"#1000000 DONE\n", mtime=0)
                (receipts[1] / "stderr.log.gz").write_bytes(lone_done)
                forged_path = receipts[1] / "report.json"
                forged = json.loads(forged_path.read_text())
                forged["raw_sha256"]["stderr.log.gz"] = campaign.digest(lone_done)
                forged_path.write_text(json.dumps(forged))
                with self.assertRaisesRegex(ValueError, "did not meet"):
                    campaign.verify(receipts)


if __name__ == "__main__":
    unittest.main()
