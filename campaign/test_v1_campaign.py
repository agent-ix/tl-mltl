"""FR-054/055: actual runner output, stale evidence, and milestone fault tests."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import make_manifest
import v1_campaign as campaign


class CampaignTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "tl-mltl"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        (self.repo / "source.txt").write_text("candidate source\n")
        (self.repo / "corpus").mkdir()
        (self.repo / "corpus" / "trace.csv").write_text("0,1\n")
        (self.repo / "fuzz" / "corpus").mkdir(parents=True)
        (self.repo / "fuzz" / "corpus" / "seed").write_bytes(b"\x00\x01")
        (self.repo / ".gitignore").write_text("target/\n")
        (self.repo / "Cargo.toml").write_text(
            '[package]\nname="tl-mltl"\nversion="0.1.0"\nedition="2021"\n'
            '[lib]\nname="campaign_fixture"\n'
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
        census = json.dumps({
            "schema": "tl-mltl.finite-partition/v1",
            "scope": "depth1_atom1_closed0_2_words1_3",
            "atom_basis": ["p0"], "max_depth": 1, "interval_max": 2,
            "trace_max_len": 3, "full_target_complete": False,
            "formulas": 375, "word_positions": 34,
            "declared": 12750, "visited": 12750, "refused": 0, "failed": 0,
        }, separators=(",", ":"))
        full_census = json.dumps({
            "schema": "tl-mltl.full-domain-census/v1",
            "scope": "depth3_atom1_closed0_4_words1_6_with_depth1_partition",
            "atom_basis": ["p0"], "symmetry_reductions": [],
            "grammar": "ordered_trees_all_boolean_and_applicable_temporal_roots",
            "max_depth": 3, "interval_max": 4, "trace_max_len": 6,
            "closed_formulas": 1_031_120_211_193_068,
            "past_formulas": 1_062_364_497_622_965,
            "word_positions": 642, "declared": 1_344_017_183_059_893_186,
            "visited": 518_094, "unvisited": 1_344_017_183_059_375_092,
            "refused": 0, "failed": 0,
            "completed_partition": {"max_depth": 1, "formulas": 807,
                                    "word_positions": 642, "declared": 518_094,
                                    "visited": 518_094},
            "full_target_complete": False,
        }, separators=(",", ":"))
        (self.repo / "tests" / "v1_finite_partition.rs").write_text(
            f'#[test] fn census() {{ println!("TL_CAMPAIGN_POPULATION {{}}", r#"{census}"#); }}\n'
            f'#[test] fn full_census() {{ println!("TL_CAMPAIGN_FULL_DOMAIN {{}}", r#"{full_census}"#); }}\n'
            "#[test] fn fault_control() { assert!(campaign_fixture::ready()); }\n"
        )
        v11_census = json.dumps({
            "schema": "tl-mltl.v11-lasso-partition/v1",
            "scope": "formulas30_words372_fair3_anchors4",
            "formula_count": 30,
            "complete_words": 228, "single_unknown_words": 136, "mixed_words": 8,
            "word_count": 372, "fairness_modes": 3, "anchors": [0, 1, 3, 6],
            "declared": 133920, "visited": 108720, "refused": 25200, "failed": 0,
            "max_materialized_lasso_len": 3, "full_target_complete": False,
        }, separators=(",", ":"))
        (self.repo / "tests" / "v11_lasso_partition.rs").write_text(
            f'#[test] fn census() {{ println!("TL_CAMPAIGN_V11_POPULATION {{}}", r#"{v11_census}"#); }}\n'
        )
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=self.repo,
                       capture_output=True, check=True)
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
             "commit", "-qm", "source"], cwd=self.repo, check=True
        )
        self.revision = campaign.git_revision(self.repo)
        self.repos = {"tl-mltl": self.repo}
        self.revisions = {"tl-mltl": self.revision}
        for name in campaign.SOURCE_NAMES:
            if name == "tl-mltl":
                continue
            repo = self.root / name
            shutil.copytree(self.repo, repo, ignore=shutil.ignore_patterns(".git", "target"))
            cargo = (repo / "Cargo.toml")
            cargo.write_text(cargo.read_text().replace('name="tl-mltl"', f'name="{name}"', 1))
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=repo,
                           capture_output=True, check=True)
            subprocess.run(["git", "add", "."], cwd=repo, check=True)
            subprocess.run(
                ["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "-qm", "source"], cwd=repo, check=True
            )
            self.repos[name] = repo
            self.revisions[name] = campaign.git_revision(repo)
        self.input = self.root / "input.txt"
        self.input.write_text("p0 U[1,2] q0\n")
        self.manifest = {
            "schema": "tl-mltl.v1-campaign-manifest/v1",
            "sources": {name: {"path": str(self.repos[name]),
                               "revision": self.revisions[name]}
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
                         hashlib.sha256((self.repos["tl-syntax"] / "Cargo.toml").read_bytes()).hexdigest())
        self.assertIn("cargo", report["semantic_payload"]["tool_versions"])
        self.assertEqual(report["semantic_payload"]["measurements"]
                         ["mutation_populations"]["status"], "not_run")
        self.manifest["sources"]["tl-syntax"]["revision"] = "0" * 40
        with self.assertRaisesRegex(ValueError, "stale source revision"):
            self.report()
        self.manifest["sources"]["tl-syntax"]["revision"] = self.revisions["tl-syntax"]
        self.input.write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "stale input digest"):
            self.report()

    def test_manifest_corpus_discovery_uses_tracked_bytes(self) -> None:
        selected = {path.relative_to(self.repo).as_posix()
                    for path in make_manifest.corpus_paths(self.repo)}
        self.assertEqual(selected, {"corpus/trace.csv", "fuzz/corpus/seed"})

    def test_source_labels_require_distinct_matching_cargo_packages(self) -> None:
        self.manifest["sources"]["tl-syntax"] = self.manifest["sources"]["tl-mltl"].copy()
        with self.assertRaisesRegex(ValueError, "distinct paths"):
            self.report()
        self.manifest["sources"]["tl-syntax"] = {
            "path": str(self.repos["tl-parse"]),
            "revision": self.revisions["tl-parse"],
        }
        self.manifest["sources"]["tl-parse"] = {
            "path": str(self.repos["tl-syntax"]),
            "revision": self.revisions["tl-syntax"],
        }
        with self.assertRaisesRegex(ValueError, "package name does not match"):
            self.report()

    def test_required_v2_partition_passes_and_full_depth_three_remains_disclosed(self) -> None:
        repo, parser, argv = campaign.COMMAND_CONTRACTS["finite_small_partition"]
        full_repo, full_parser, full_argv = campaign.COMMAND_CONTRACTS["full_domain_census"]
        self.manifest["lanes"] = [{
            "id": "finite_small_partition", "milestone": "V2", "mode": "command",
            "repo": repo, "parser": parser, "argv": argv,
            "seed": {"kind": "none", "reason": "finite_exhaustive"},
        }, {
            "id": "full_domain_census", "milestone": "V2", "mode": "command",
            "repo": full_repo, "parser": full_parser, "argv": full_argv,
            "seed": {"kind": "none", "reason": "finite_exhaustive"},
        }]
        report = self.report()
        semantic = report["semantic_payload"]
        lane = semantic["lanes"]["finite_small_partition"]
        self.assertEqual(lane["status"], "passed")
        self.assertEqual(lane["population"]["declared"], 12750)
        full_lane = semantic["lanes"]["full_domain_census"]
        self.assertEqual(full_lane["status"], "passed")
        self.assertEqual(full_lane["population"]["visited"], 518_094)
        self.assertEqual(full_lane["population"]["unvisited"], 1_344_017_183_059_375_092)
        self.assertFalse(full_lane["population"]["full_target_complete"])
        self.assertEqual(semantic["milestones"]["V2"]["status"], "passed")
        raw = Path(report["raw_artifacts"]["finite_small_partition"]["stdout"]["path"])
        broken = raw.read_bytes().replace(b'"visited":12750', b'"visited":1')
        status, population = campaign.classify(broken, "cargo_population", 0)
        self.assertEqual((status, population["reason"]), ("incomplete", "unvisited_population"))
        full_raw = Path(report["raw_artifacts"]["full_domain_census"]["stdout"]["path"])
        broken_full = full_raw.read_bytes().replace(b'"visited":518094', b'"visited":518095')
        status, population = campaign.classify(broken_full, "cargo_full_domain_census", 0)
        self.assertEqual((status, population["reason"]), ("failed", "malformed_full_domain_census"))

    def test_v8_review_input_is_explicit_pinned_and_checked_after_measurement(self) -> None:
        cargo_home = self.root / "cargo-home"
        cargo_home.mkdir()
        reviews = self.root / "v8-reviews.json"
        reviews.write_text("[]\n")
        ordinary = make_manifest.make_manifest(self.root, v8_cargo_home=cargo_home)
        self.assertNotIn("v8_reviews", ordinary["inputs"])
        ordinary_lane = next(lane for lane in ordinary["lanes"]
                             if lane["id"] == "coverage")
        self.assertNotIn("reviews_path", ordinary_lane)

        manifest = make_manifest.make_manifest(self.root, v8_cargo_home=cargo_home,
                                               v8_reviews=reviews)
        lane = next(lane for lane in manifest["lanes"] if lane["id"] == "coverage")
        self.assertEqual(lane["reviews_path"], str(reviews.resolve()))
        self.assertEqual(manifest["inputs"]["v8_reviews"], {
            "path": str(reviews.resolve()),
            "sha256": campaign.sha256(reviews.read_bytes()),
        })
        graph = campaign.source_graph(manifest)
        inputs = campaign.input_graph(manifest)
        wrong = dict(lane, reviews_path=str((self.root / "other.json").resolve()))
        (self.root / "other.json").write_text("[]\n")
        result, _ = campaign.run_lane(wrong, graph, inputs, self.raw)
        self.assertEqual((result["status"], result["reason"]),
                         ("incomplete", "v8_reviews_not_pinned"))

        def changed_during_measurement(argv, **_kwargs):
            reviews.write_text("[{}]\n")
            return subprocess.CompletedProcess(argv, 0, b"", b"")

        with patch.object(campaign.subprocess, "run", side_effect=changed_during_measurement):
            result, _ = campaign.run_lane(lane, graph, inputs, self.raw)
        self.assertEqual((result["status"], result["population"]["reason"]),
                         ("failed", "malformed_v8_native_evidence"))
        with self.assertRaisesRegex(ValueError, "v8_reviews: stale input digest"):
            campaign.input_graph(manifest)
        reviews.unlink()
        with self.assertRaises(FileNotFoundError):
            campaign.input_graph(manifest)

    def test_live_target_is_explicit_and_native_population_is_fault_checked(self) -> None:
        ordinary = make_manifest.make_manifest(self.root)
        self.assertNotIn("live_r2u2", {lane["id"] for lane in ordinary["lanes"]})
        self.assertNotIn("live_past_grid", {lane["id"] for lane in ordinary["lanes"]})
        opted_in = make_manifest.make_manifest(self.root, self.root / "r2u2-source")
        live = next(lane for lane in opted_in["lanes"] if lane["id"] == "live_r2u2")
        self.assertEqual(live["target_source"], str((self.root / "r2u2-source").resolve()))
        self.assertEqual(live["argv"], campaign.COMMAND_CONTRACTS["live_r2u2"][2])
        grid = next(lane for lane in opted_in["lanes"] if lane["id"] == "live_past_grid")
        self.assertEqual(grid["target_source"], live["target_source"])
        self.assertEqual(grid["argv"], campaign.COMMAND_CONTRACTS["live_past_grid"][2])

        artifacts = {
            f"{case}.{kind}": "a" * 64
            for case in ("bounded", "past", "unsafe-since")
            for kind in ("bin", "compiler.stdout", "compiler.stderr",
                         "r2u2.stdout", "r2u2.stderr")
        }
        artifacts.update({f"safety.{kind}": "a" * 64 for kind in (
            "c2po", "csv", "bin", "compiler.stdout", "compiler.stderr",
            "r2u2.stdout", "r2u2.stderr",
        )})
        bounded_cases = {
            "r2u2-future-witness-v1": 0,
            "r2u2-globally-counterexample-v1": 0,
            "r2u2-future-deadline-v1": 0,
            "r2u2-until-lower-bound-v1": 0,
            "r2u2-release-lower-bound-v1": 0,
            "r2u2-nested-until-v1": 0,
            "r2u2-future-at-one-v1": 1,
            "r2u2-globally-at-one-v1": 1,
        }
        rows = [
            dict(case=case, family="bounded", position=at,
                 classification="agreement",
                 oracle=case != "r2u2-globally-counterexample-v1",
                 target=case != "r2u2-globally-counterexample-v1")
            for case, at in bounded_cases.items()
        ]
        rows += [
            dict(case=case, family="past", position=at,
                 classification="agreement", oracle=False, target=False)
            for case in ("once-zero-one", "historically-zero-one", "previous", "once-one-one")
            for at in range(3)
        ]
        rows += [
            dict(case=name, family="past", position=at,
                 classification="unsupported_mapping", oracle=False, target=False)
            for name in ("since-zero-two", "triggered-zero-two") for at in range(3)
        ]
        rows.append(dict(case="unsafe-since", family="past", position=2,
                         classification="unsupported_mapping", oracle=False, target=True))
        marker = {
            "schema": "tl-mltl.live-r2u2/v1",
            "source_revision": campaign.LIVE_TARGET_REVISION,
            "compiler_sha256": campaign.LIVE_COMPILER_SHA256,
            "monitor_sha256": campaign.LIVE_MONITOR_SHA256,
            "license": "Apache-2.0", "bounded_cells": 8, "past_cells": 18,
            "unsafe_cells": 1, "safety_export_cells": 2,
            "artifacts": artifacts, "classifications": rows,
            "runs": {
                "bounded": {"compiler_exit": 0, "monitor_exit": 0,
                            "spec": "corpus/r2u2-v4.2/formulas.c2po",
                            "trace": "corpus/r2u2-v4.2/trace.csv",
                            "map": "corpus/r2u2-v4.2/signals.map"},
                "past": {"compiler_exit": 0, "monitor_exit": 0,
                         "spec": "corpus/past-c2po-v1/target-4.2/past.c2po",
                         "trace": "corpus/past-c2po-v1/target-4.2/trace.csv",
                         "map": None},
                "unsafe-since": {"compiler_exit": 0, "monitor_exit": 0,
                                 "spec": "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
                                 "trace": "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
                                 "map": None},
                "safety": {"compiler_exit": 0, "monitor_exit": 0,
                           "spec": "/private/tmp/safety.c2po",
                           "trace": "/private/tmp/safety.csv", "map": None},
            },
            "safety_export": {
                "schema": "tl-mltl.infinite-safety-mapping/v1",
                "section": "FTSPEC", "expression": "q",
                "expression_sha256": campaign.sha256(b"q"),
                "input_sha256": "b" * 64, "graph_id": "c" * 64,
                "decision_horizon": 0, "refutation_only": True,
                "false_position": 0, "false_disposition": "refuted",
                "target_false": False, "true_position": 1,
                "true_disposition": "inconclusive", "target_true": True,
            },
            "bad_prefix": {"basis": "bad_prefix", "disposition": "refuted",
                           "oracle": "refuted", "violation_position": 0,
                           "target_case": "r2u2-globally-counterexample-v1",
                           "target_position": 0, "target_verdict": False},
        }
        def raw(value: dict) -> bytes:
            return ("TL_CAMPAIGN_LIVE_TARGET " + json.dumps(value, separators=(",", ":"))
                    + "\n").encode()

        state, population = campaign.classify(raw(marker), "cargo_live_target", 0)
        self.assertEqual(state, "passed")
        self.assertEqual((population["visited"], population["unsupported_mapping"]), (27, 7))
        for corrupt in (
            lambda value: value.update(source_revision="0" * 40),
            lambda value: value["bad_prefix"].update(disposition="proved"),
            lambda value: value["classifications"][0].update(target=False),
            lambda value: value["classifications"][0].update(case="invented-cell"),
            lambda value: value["classifications"].pop(),
            lambda value: value["artifacts"].pop("bounded.r2u2.stdout"),
            lambda value: value["runs"]["past"].update(monitor_exit=1),
            lambda value: value["safety_export"].update(true_disposition="proved"),
            lambda value: value["safety_export"].update(refutation_only=False),
            lambda value: value["runs"]["safety"].update(spec="/tmp/other.c2po"),
        ):
            broken = json.loads(json.dumps(marker))
            corrupt(broken)
            self.assertEqual(campaign.classify(raw(broken), "cargo_live_target", 0)[0], "failed")
        self.assertEqual(campaign.classify(raw(marker), "cargo_live_target", 1)[0], "failed")

    def test_live_target_missing_source_cannot_pass(self) -> None:
        repo, parser, argv = campaign.COMMAND_CONTRACTS["live_r2u2"]
        lane = {"id": "live_r2u2", "milestone": "V10", "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "target_source": str(self.root / "missing-target"),
                "seed": {"kind": "none", "reason": "deterministic_live_run"}}
        self.manifest["lanes"] = [lane]
        report = self.report()["semantic_payload"]
        self.assertEqual(report["lanes"]["live_r2u2"]["status"], "blocked")
        self.assertEqual(report["milestones"]["V10"]["status"], "blocked")

    def test_live_grid_requires_exact_command_and_target(self) -> None:
        repo, parser, argv = campaign.COMMAND_CONTRACTS["live_past_grid"]
        lane = {"id": "live_past_grid", "milestone": "V10", "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "target_source": str(self.root / "missing-target"),
                "seed": {"kind": "none", "reason": "deterministic_live_run"}}
        self.manifest["lanes"] = [lane]
        semantic = self.report()["semantic_payload"]
        self.assertEqual(semantic["lanes"]["live_past_grid"]["status"], "blocked")
        self.assertEqual(semantic["milestones"]["V10"]["status"], "blocked")
        lane["argv"] = ["python3", "-c", "print('all grid cells passed')"]
        semantic = self.report()["semantic_payload"]
        self.assertEqual(semantic["lanes"]["live_past_grid"]["status"], "incomplete")
    # TC-193/194: V11's real command and native census parser accept exactly
    # its declared small partition and reject missing, altered, or excess data.
    def test_native_v11_population_and_seeded_parser_faults(self) -> None:
        repo, parser, argv = campaign.COMMAND_CONTRACTS["lasso_population_census"]
        self.manifest["lanes"] = [{
            "id": "lasso_population_census", "milestone": "V11", "mode": "command",
            "repo": repo, "parser": parser, "argv": argv,
            "seed": {"kind": "none", "reason": "declared_exhaustive_partition"},
        }]
        report = self.report()
        semantic = report["semantic_payload"]
        lane = semantic["lanes"]["lasso_population_census"]
        self.assertEqual(lane["status"], "passed")
        self.assertEqual(lane["population"]["declared"], 133920)
        self.assertEqual(lane["population"]["visited"], 108720)
        self.assertEqual(lane["population"]["refused"], 25200)
        self.assertEqual(semantic["milestones"]["V11"]["contract"]
                         ["lasso_population_census"]["kind"], "exact_command")
        raw = Path(report["raw_artifacts"]["lasso_population_census"]["stdout"]["path"])
        complete = raw.read_bytes()
        self.assertEqual(campaign.classify(complete, parser, 0)[0], "passed")
        missing = complete.replace(b'"visited":108720', b'"visited":1')
        self.assertEqual(campaign.classify(missing, parser, 0)[0], "incomplete")
        wrong_split = complete.replace(b'"visited":108720', b'"visited":108719').replace(
            b'"refused":25200', b'"refused":25201')
        self.assertEqual(campaign.classify(wrong_split, parser, 0)[0], "failed")
        wrong_axis = complete.replace(b'"word_count":372', b'"word_count":371')
        self.assertEqual(campaign.classify(wrong_axis, parser, 0)[0], "failed")
        false_as_number = complete.replace(b'"full_target_complete":false',
                                           b'"full_target_complete":0')
        self.assertEqual(campaign.classify(false_as_number, parser, 0)[0], "failed")
        self.assertEqual(campaign.classify(complete + complete, parser, 0)[0], "failed")
        self.assertEqual(campaign.classify(complete, parser, 1)[0], "failed")
    # TC-179/180, FR-045-AC-1/2, NFR-008-AC-1: the V3 parser must not
    # credit a copied success summary with omitted criteria, a vacuous run,
    # unexecuted examples, changed seed, or malformed wire population.
    def test_native_properties_require_reconciled_criteria_and_fault_controls(self) -> None:
        classes = {criterion: {"kind": "excluded", "evidence": "other lane: TC-190"}
                   for criterion in campaign.v1_criterion_ids()}
        classes["FR-045-AC-1"] = {"kind": "property", "evidence": "duality"}
        classes["FR-045-AC-2"] = {"kind": "property", "evidence": "strict_round_trips"}
        classes["FR-030-AC-1"] = {"kind": "example", "evidence": "cross_compare_small_lassos"}
        marker = {
            "schema": "tl-mltl.semantic-properties/v1",
            "scope": "tl_mltl_v1_semantic_laws_and_owner_wires",
            "seed_hex": "45" * 32,
            "generated": 64, "accepted": 64, "rejected": 0,
            "law_cases": {law: 64 for law in campaign.PROPERTY_LAWS},
            "wire_checks": 24, "rewrite_equivalence_owner": "tl-rewrite",
            "classifications": classes,
        }

        def raw(value: dict, *, inline_marker: bool = False) -> bytes:
            names = (
                "native_semantic_laws_and_strict_round_trips",
                "seeded_law_fault_is_detected",
                "cross_compare_small_lassos",
            )
            marker_line = "TL_CAMPAIGN_PROPERTIES " + json.dumps(
                value, separators=(",", ":"))
            if inline_marker:
                native = f"test v1_campaign::{names[0]} ... {marker_line}\nok"
                tests = [native, *(f"test v1_campaign::{name} ... ok" for name in names[1:])]
                return ("\n".join(tests) + "\n" + self.cargo_summary(3) + "\n").encode()
            return ("\n".join(f"test {name} ... ok" for name in names)
                    + "\n" + marker_line + "\n" + self.cargo_summary(3) + "\n").encode()

        passed, population = campaign.classify(raw(marker), "cargo_properties", 0)
        self.assertEqual(passed, "passed")
        self.assertEqual(population["classified"], 61)
        self.assertEqual(population["accepted"], 64)
        inline = raw(marker, inline_marker=True)
        self.assertEqual(campaign.classify(inline, "cargo_properties", 0)[0], "passed")
        self.assertEqual(campaign.classify(inline.replace(b"\nok\n", b"\nFAILED\n", 1),
                                           "cargo_properties", 0)[0], "failed")
        self.assertEqual(campaign.classify(inline + b"TL_CAMPAIGN_PROPERTIES {}\n",
                                           "cargo_properties", 0)[0], "failed")

        altered = json.loads(json.dumps(marker))
        altered["classifications"].pop("FR-033-AC-2")
        self.assertEqual(campaign.classify(raw(altered), "cargo_properties", 0)[0], "failed")
        for field, value in (("accepted", 0), ("rejected", 1),
                             ("wire_checks", 23), ("seed_hex", "00" * 32)):
            altered = marker | {field: value}
            self.assertEqual(campaign.classify(raw(altered), "cargo_properties", 0)[0],
                             "failed", field)
        altered = json.loads(json.dumps(marker))
        altered["law_cases"]["duality"] = 0
        self.assertEqual(campaign.classify(raw(altered), "cargo_properties", 0)[0], "failed")
        altered = json.loads(json.dumps(marker))
        altered["classifications"]["FR-030-AC-1"]["evidence"] = "never_ran"
        self.assertEqual(campaign.classify(raw(altered), "cargo_properties", 0)[0], "failed")
        duplicate = raw(marker) + b"TL_CAMPAIGN_PROPERTIES {}\n"
        self.assertEqual(campaign.classify(duplicate, "cargo_properties", 0)[0], "failed")
        duplicate_key = raw(marker).replace(b'"accepted":64', b'"accepted":64,"accepted":64')
        self.assertEqual(campaign.classify(duplicate_key, "cargo_properties", 0)[0], "failed")
        self.assertEqual(campaign.classify(raw(marker), "cargo_properties", 1)[0], "failed")

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

    # Trace: TC-197, TC-198, FR-055-AC-1, FR-055-AC-2
    # All eleven declared gates exist; one failing or stale lane
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
                         ["full_domain_census"]["kind"], "exact_command")
        self.assertEqual(report["milestones"]["V11"]["contract"]
                         ["lasso_population_census"]["kind"], "exact_command")
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
            "source_revisions": self.revisions.copy(),
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
        record["source_revisions"]["tl-oracle"] = self.revisions["tl-oracle"]
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
