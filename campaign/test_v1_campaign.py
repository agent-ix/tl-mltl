"""FR-054/055: actual runner output, stale evidence, and milestone fault tests."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import make_manifest
import v1_campaign as campaign


class CampaignTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "source"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        (self.repo / "source.txt").write_text("candidate source\n")
        (self.repo / "corpus").mkdir()
        (self.repo / "corpus" / "trace.csv").write_text("0,1\n")
        (self.repo / "fuzz" / "corpus").mkdir(parents=True)
        (self.repo / "fuzz" / "corpus" / "seed").write_bytes(b"\x00\x01")
        (self.repo / ".gitignore").write_text("target/\n")
        (self.repo / "Cargo.toml").write_text(
            '[package]\nname="campaign-fixture"\nversion="0.1.0"\nedition="2021"\n'
            '[features]\ninfinite-trace=[]\n'
        )
        (self.repo / "src").mkdir()
        (self.repo / "src" / "lib.rs").write_text("pub fn ready() -> bool { true }\n")
        (self.repo / "tests").mkdir()
        (self.repo / "tests" / "oracle_finite_past.rs").write_text(
            "#[test] fn independent_oracle() { assert!(campaign_fixture::ready()); }\n"
        )
        (self.repo / "tests" / "infinite_oracle.rs").write_text(
            "#[test] fn lasso() { assert!(campaign_fixture::ready()); }\n"
        )
        for name in ("reference", "dependency_boundary", "oracle_finite_rewrite",
                     "infinite_rules", "infinite_trace", "semantic_laws"):
            (self.repo / "tests" / f"{name}.rs").write_text(
                "#[test] fn evidence() { assert!(campaign_fixture::ready()); }\n"
            )
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=self.repo,
                       capture_output=True, check=True)
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
             "commit", "-qm", "source"], cwd=self.repo, check=True
        )
        self.revision = campaign.git_revision(self.repo)
        self.input = self.root / "input.txt"
        self.input.write_text("p0 U[1,2] q0\n")
        self.manifest = {
            "schema": "tl-mltl.v1-campaign-manifest/v1",
            "sources": {name: {"path": str(self.repo), "revision": self.revision}
                        for name in campaign.SOURCE_NAMES},
            "inputs": {"formula": {"path": str(self.input),
                                   "sha256": hashlib.sha256(self.input.read_bytes()).hexdigest()}},
            "lanes": [],
        }
        self.raw = self.root / "raw"

    def lane(self, lane_id: str, milestone: str, code: str, parser: str = "cargo_test") -> dict:
        return {
            "id": lane_id, "milestone": milestone, "mode": "command",
            "repo": "tl-mltl", "argv": [sys.executable, "-c", code], "parser": parser,
            "seed": {"kind": "fixed", "value": 42},
        }

    def real_v1_lanes(self) -> list[dict]:
        lanes = []
        for lane_id in campaign.REQUIRED["V1"]:
            repo, parser, argv = campaign.COMMAND_CONTRACTS[lane_id]
            lanes.append({
                "id": lane_id, "milestone": "V1", "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "seed": {"kind": "none", "reason": "deterministic_seeded_fault_test"},
            })
        return lanes

    @staticmethod
    def cargo_summary(passed: int, failed: int = 0) -> str:
        state = "FAILED" if failed else "ok"
        return (f"test result: {state}. {passed} passed; {failed} failed; "
                "0 ignored; 0 measured; 0 filtered out; finished in 0.01s")

    def report(self) -> dict:
        return campaign.build_report(self.manifest, self.raw)

    # TC-195: Execute a real subprocess, retain its exact stdout digest, and
    # reject a substituted source or changed input instead of crediting it.
    def test_actual_command_output_exact_graph_and_digest(self) -> None:
        self.manifest["lanes"] = self.real_v1_lanes()
        report = self.report()
        lane = report["semantic_payload"]["lanes"]["independent_oracle"]
        self.assertEqual(lane["status"], "passed")
        stdout = Path(report["raw_artifacts"]["independent_oracle"]["stdout"]["path"])
        self.assertEqual(report["raw_artifacts"]["independent_oracle"]["stdout"]["sha256"],
                         hashlib.sha256(stdout.read_bytes()).hexdigest())
        self.assertEqual(report["semantic_payload"]["source_pins"]["tl-syntax"]
                         ["cargo_toml_sha256"],
                         hashlib.sha256((self.repo / "Cargo.toml").read_bytes()).hexdigest())
        self.assertIn("cargo", report["semantic_payload"]["tool_versions"])
        self.assertEqual(report["semantic_payload"]["measurements"]
                         ["mutation_populations"]["status"], "not_run")
        self.manifest["sources"]["tl-syntax"]["revision"] = "0" * 40
        with self.assertRaisesRegex(ValueError, "stale source revision"):
            self.report()
        self.manifest["sources"]["tl-syntax"]["revision"] = self.revision
        self.input.write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "stale input digest"):
            self.report()

    def test_manifest_corpus_discovery_uses_tracked_bytes(self) -> None:
        selected = {path.relative_to(self.repo).as_posix()
                    for path in make_manifest.corpus_paths(self.repo)}
        self.assertEqual(selected, {"corpus/trace.csv", "fuzz/corpus/seed"})

    # TC-196: Identical deterministic runs have identical semantic payloads
    # even though raw paths may differ. Preserve refused, missing and failed.
    def test_repeated_semantics_and_population_classes(self) -> None:
        self.manifest["lanes"] = [
            self.lane("partition_probe", "V2",
                      "import json;print(json.dumps(dict(declared=4,visited=2,refused=1,failed=0)))",
                      "population_json"),
            self.lane("property_probe", "V3",
                      "import json;print(json.dumps(dict(declared=4,visited=2,refused=1,failed=1)))",
                      "population_json"),
        ]
        first = self.report()
        self.raw = self.root / "other-raw"
        second = self.report()
        self.assertEqual(first["semantic_sha256"], second["semantic_sha256"])
        lanes = first["semantic_payload"]["lanes"]
        self.assertEqual(lanes["partition_probe"]["status"], "incomplete")
        self.assertEqual(lanes["partition_probe"]["population"]["refused"], 1)
        self.assertEqual(lanes["property_probe"]["status"], "failed")
        self.assertEqual(first["semantic_payload"]["milestones"]["V4"]["status"], "incomplete")
        self.assertEqual(lanes["fuzz_replay"]["status"], "not_run")
        self.assertEqual(first["semantic_payload"]["aggregate_status"], "failed")

    # TC-197/198: All eleven declared gates exist; one failing or stale lane
    # cannot turn the aggregate green or suppress passing sibling evidence.
    def test_gate_mapping_and_failed_stale_missing_siblings(self) -> None:
        self.manifest["lanes"] = [
            *self.real_v1_lanes(),
            self.lane("broken_comparison", "V2", f"print({self.cargo_summary(0, 1)!r})"),
            {"id": "semantic_properties", "milestone": "V3", "mode": "record",
             "parser": "cargo_test", "receipt": str(self.root / "missing.json")},
        ]
        report = self.report()["semantic_payload"]
        self.assertEqual(set(report["milestones"]), set(campaign.REQUIRED))
        self.assertEqual(report["milestones"]["V1"]["status"], "passed")
        self.assertEqual(report["milestones"]["V1"]["contract"]
                         ["independent_oracle"]["kind"], "exact_command")
        self.assertEqual(report["milestones"]["V2"]["contract"]
                         ["exhaustive_partition"]["kind"], "unsupported")
        self.assertEqual(report["milestones"]["V11"]["contract"]
                         ["lasso_population_census"]["kind"], "unsupported")
        self.assertEqual(report["milestones"]["V2"]["status"], "failed")
        self.assertEqual(report["milestones"]["V3"]["status"], "incomplete")
        self.assertEqual(report["lanes"]["semantic_properties"]["reason"],
                         "unusable_record:FileNotFoundError")
        self.assertEqual(report["lanes"]["independent_oracle"]["population"]["passed"], 1)
        self.assertEqual(report["aggregate_status"], "failed")

    # TC-195/196: Imported output is read and reclassified from its bytes;
    # receipt verdicts have no authority and changed raw bytes are refused.
    def test_external_record_uses_real_bytes_and_refuses_stale_substitution(self) -> None:
        out = self.root / "external.stdout"
        err = self.root / "external.stderr"
        out.write_text(self.cargo_summary(3) + "\n")
        err.write_bytes(b"")
        receipt = self.root / "receipt.json"
        record = {
            "source_revisions": {name: self.revision for name in campaign.SOURCE_NAMES},
            "input_sha256": {"formula": self.manifest["inputs"]["formula"]["sha256"]},
            "parser": "cargo_test", "exit_code": 0,
            "stdout": {"path": str(out), "sha256": campaign.sha256(out.read_bytes())},
            "stderr": {"path": str(err), "sha256": campaign.sha256(err.read_bytes())},
            "status": "failed",  # Deliberate false assertion must be ignored.
        }
        receipt.write_text(json.dumps(record))
        self.manifest["lanes"] = [{"id": "independent_oracle", "milestone": "V1",
                                   "mode": "record", "parser": "cargo_test",
                                   "receipt": str(receipt)}]
        first = self.report()["semantic_payload"]["lanes"]["independent_oracle"]
        self.assertEqual(first["status"], "incomplete")
        self.assertEqual(first["observed_status"], "passed")
        out.write_text(self.cargo_summary(4) + "\n")
        self.assertEqual(self.report()["semantic_payload"]["lanes"]["independent_oracle"]["status"],
                         "incomplete")
        out.write_text(self.cargo_summary(3) + "\n")
        record["source_revisions"]["tl-oracle"] = "0" * 40
        receipt.write_text(json.dumps(record))
        self.assertEqual(self.report()["semantic_payload"]["lanes"]["independent_oracle"]["status"],
                         "incomplete")
        record["source_revisions"]["tl-oracle"] = self.revision
        record["nested"] = {"human": {"accepted": True}}
        receipt.write_text(json.dumps(record))
        lane = self.report()["semantic_payload"]["lanes"]["independent_oracle"]
        self.assertEqual(lane["status"], "incomplete")
        self.assertEqual(lane["reason"], "unusable_record:ValueError")

    # TC-199: Empty success, excess visits and forbidden approval claims fail
    # or remain open; the output only states automated evidence.
    def test_no_vacuous_or_human_claims(self) -> None:
        self.manifest["lanes"] = [
            self.lane("empty_probe", "V2",
                      "import json;print(json.dumps(dict(declared=0,visited=0,refused=0,failed=0)))",
                      "population_json"),
            self.lane("duplicate_probe", "V3",
                      "import json;print(json.dumps(dict(declared=2,visited=3,refused=0,failed=0)))",
                      "population_json"),
        ]
        semantic = self.report()["semantic_payload"]
        self.assertEqual(semantic["lanes"]["empty_probe"]["status"], "incomplete")
        self.assertEqual(semantic["lanes"]["duplicate_probe"]["status"], "failed")
        self.assertEqual(semantic["claim_boundary"], "automated_evidence_only")
        self.assertNotIn("accepted", semantic)
        self.manifest["accepted"] = True
        with self.assertRaisesRegex(ValueError, "claims"):
            self.report()
        del self.manifest["accepted"]
        self.manifest["lanes"][0]["metadata"] = {"release": {"certified": True}}
        with self.assertRaisesRegex(ValueError, "claims"):
            self.report()

    def test_required_gate_rejects_forged_cargo_summary_command(self) -> None:
        self.manifest["lanes"] = [self.lane(
            "independent_oracle", "V1", f"print({self.cargo_summary(100)!r})"
        )]
        lane = self.report()["semantic_payload"]["lanes"]["independent_oracle"]
        self.assertEqual(lane["status"], "incomplete")
        self.assertEqual(lane["reason"], "unregistered_required_gate")
        self.assertFalse((self.raw / "independent_oracle.stdout").exists())

    def test_unavailable_tool_is_blocked_without_hiding_other_lanes(self) -> None:
        self.manifest["lanes"] = [
            *self.real_v1_lanes(),
            {"id": "missing_tool_probe", "milestone": "V2", "mode": "command",
             "repo": "tl-mltl", "argv": ["/nonexistent/tl-campaign-tool"],
             "parser": "cargo_test", "seed": {"kind": "none", "reason": "deterministic"}},
        ]
        report = self.report()["semantic_payload"]
        self.assertEqual(report["lanes"]["missing_tool_probe"]["status"], "blocked")
        self.assertEqual(report["lanes"]["independent_oracle"]["status"], "passed")
        self.assertEqual(report["milestones"]["V2"]["status"], "blocked")


if __name__ == "__main__":
    unittest.main()
