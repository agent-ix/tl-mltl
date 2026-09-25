"""Fault controls for the V8 raw-export reconciliation gate."""

import json
import platform
import tempfile
import unittest
from pathlib import Path

import v8_gate
from v8_coverage import (CRATES, CRITICAL_PREFIXES, classify_export, critical_census,
                         parse_prep_binary, parse_prep_command)


class V8GateTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.raw_dir = self.root / "coverage-native"
        self.raw_dir.mkdir()
        self.graph = {}
        exports = {}
        for name in CRATES:
            repo = self.root / f"tl-{name}"
            files = []
            for prefix in CRITICAL_PREFIXES[name]:
                relatives = (("src/wire/command.rs", "src/wire/common.rs",
                              "src/wire/trace.rs") if prefix == "src/wire/" else (prefix,))
                for relative in relatives:
                    file = repo / relative
                    file.parent.mkdir(parents=True, exist_ok=True)
                    file.write_text("fn check() {}\n")
                    files.append({"filename": str(file),
                                  "summary": {"lines": {"count": 1, "covered": 1},
                                              "branches": {"count": 2, "covered": 2},
                                              "functions": {"count": 1, "covered": 1},
                                              "regions": {"count": 1, "covered": 1}},
                                  "branches": [[1, 2, 1, 12, 7, 8, 0, 0, 4],
                                               [1, 2, 1, 12, 7, 8, 0, 0, 4]]})
            self.graph[f"tl-{name}"] = {
                "revision": str(CRATES.index(name) + 1) * 40,
                "cargo_lock_sha256": str(CRATES.index(name) + 5) * 64,
                "path": str(repo),
            }
            exports[name] = {"type": "llvm.coverage.json.export", "data": [{"files": files}]}
        rows = []
        for key, name, feature, argv in v8_gate.expected_runs(self.raw_dir):
            files = {}
            for kind, suffix in (("stdout", "stdout"), ("stderr", "stderr"),
                                 ("export", "json")):
                path = self.raw_dir / f"{key}.{suffix}"
                data = (json.dumps(exports[name]).encode() if kind == "export" else
                        b"Finished report saved\n" if kind == "stderr" else b"")
                path.write_bytes(data)
                files[kind] = {"path": str(path.resolve()), "sha256": v8_gate.digest(data)}
            coverage = classify_export(exports[name], Path(self.graph[f"tl-{name}"]["path"]))
            critical_files, missing, gaps = critical_census(coverage, name, feature)
            row = {"id": key, "repo": name, "feature": feature,
                         "source_revision": self.graph[f"tl-{name}"]["revision"],
                         "argv": argv, "exit_code": 0, "raw": files,
                         "coverage": coverage, "critical_uncovered": gaps,
                         "critical_branch_census": {
                             "files": critical_files, "missing_files": missing,
                             "count": sum(item["count"] for item in critical_files.values()),
                             "covered": sum(item["covered"] for item in critical_files.values())},
                         "status": "passed"}
            if name == "parse":
                binary = parse_prep_binary(self.raw_dir / f"{key}.target")
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b"pinned parse example binary")
                prep_raw = {}
                for kind in ("stdout", "stderr"):
                    path = self.raw_dir / f"{key}.prep.{kind}"
                    path.write_bytes(f"parse preparation {kind}\n".encode())
                    prep_raw[kind] = {"path": str(path.resolve()),
                                      "sha256": v8_gate.digest(path.read_bytes())}
                row["prep"] = {"argv": parse_prep_command(), "exit_code": 0,
                               "raw": prep_raw,
                               "binary": {"path": str(binary.resolve()),
                                          "sha256": v8_gate.digest(binary.read_bytes())}}
            rows.append(row)
        self.tools = {"rustc": "rustc fixture", "llvm_cov": "llvm-cov fixture",
                      "llvm_profdata": "llvm-profdata fixture",
                      "cargo_llvm_cov": "cargo-llvm-cov fixture"}
        self.report = {
            "schema": "tl-mltl.v8-coverage/v2",
            "source_revisions": {name: self.graph[f"tl-{name}"]["revision"] for name in CRATES},
            "cargo_lock_sha256": {name: self.graph[f"tl-{name}"]["cargo_lock_sha256"]
                                  for name in CRATES},
            "tools": self.tools, "profile": "test", "test_selection": ["lib", "tests"],
            "host": {"system": platform.system(), "machine": platform.machine()},
            "runs": rows, "status": "passed",
        }

    def verify(self, report=None):
        return v8_gate.verify(json.dumps(report or self.report).encode(),
                              self.raw_dir, self.graph, self.tools)

    def restamp_export(self, index, export):
        row = self.report["runs"][index]
        path = self.raw_dir / f"{row['id']}.json"
        path.write_text(json.dumps(export))
        row["raw"]["export"]["sha256"] = v8_gate.digest(path.read_bytes())
        measured = classify_export(export, Path(self.graph[f"tl-{row['repo']}"]["path"]))
        row["coverage"] = measured
        files, missing, gaps = critical_census(measured, row["repo"], row["feature"])
        row["critical_uncovered"] = gaps
        row["critical_branch_census"] = {
            "files": files, "missing_files": missing,
            "count": sum(item["count"] for item in files.values()),
            "covered": sum(item["covered"] for item in files.values()),
        }
        row["status"] = "incomplete" if missing or gaps else "passed"
        if missing:
            row["reason"] = "critical_branches_not_instrumented"
        elif gaps:
            row["reason"] = "critical_branch_target_open"
        else:
            row.pop("reason", None)
        self.report["status"] = ("passed" if all(item["status"] == "passed"
                                               for item in self.report["runs"])
                                 else "incomplete")

    def reviewed_gap(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        past = next(file for file in export["data"][0]["files"]
                    if file["filename"].endswith("/src/past/mod.rs"))
        past["summary"]["branches"]["covered"] = 1
        for branch in past["branches"]:
            branch[5] = 0
        self.restamp_export(index, export)
        row = self.report["runs"][index]
        self.assertEqual(len(row["critical_uncovered"]), 1)
        gap = row["critical_uncovered"][0]
        source = Path(self.graph["tl-mltl"]["path"]) / gap["file"]
        review = {"run": row["id"], **gap,
                  "source_file_sha256": v8_gate.digest(source.read_bytes()),
                  "reason": ("The false side requires a report with an impossible "
                             "predecessor state after public validation."),
                  "reviewer": "Ada Reviewer"}
        summary_review = {"kind": "file_summary", "run": row["id"], "file": gap["file"],
                          "summary_missing_sides": 1,
                          "source_file_sha256": review["source_file_sha256"],
                          "raw_export_sha256": row["raw"]["export"]["sha256"],
                          "reason": ("The file summary's missing side is confined to a "
                                     "validated predecessor state absent at this boundary."),
                          "reviewer": "Ada Reviewer"}
        self.report["reviewed_infeasibility"] = [review, summary_review]
        row["status"] = "passed"
        row.pop("reason", None)
        self.report["status"] = "passed"
        return review, source

    def test_complete_raw_export_population_passes(self):
        status, population, artifacts = self.verify()
        self.assertEqual(status, "passed")
        total = sum(row["coverage"]["totals"]["branches"]["count"]
                    for row in self.report["runs"])
        self.assertEqual(population["production_branches"],
                         {"count": total, "covered": total})
        self.assertEqual(len(artifacts), 28)

    def test_parse_preparation_command_and_binary_are_bound(self):
        report = json.loads(json.dumps(self.report))
        index = next(i for i, row in enumerate(report["runs"])
                     if row["id"] == "parse-default")
        report["runs"][index]["prep"]["argv"].append("--release")
        with self.assertRaisesRegex(ValueError, "preparation identity mismatch"):
            self.verify(report)
        binary = parse_prep_binary(self.raw_dir / "parse-default.target")
        binary.write_bytes(b"substituted example binary")
        with self.assertRaisesRegex(ValueError, "preparation binary changed"):
            self.verify()

    def test_parse_preparation_logs_and_failure_cannot_claim_coverage(self):
        log = self.raw_dir / "parse-default.prep.stderr"
        log.write_bytes(b"changed build log")
        with self.assertRaisesRegex(ValueError, "preparation log changed"):
            self.verify()
        log.write_bytes(b"parse preparation stderr\n")
        report = json.loads(json.dumps(self.report))
        index = next(i for i, row in enumerate(report["runs"])
                     if row["id"] == "parse-default")
        report["runs"][index]["prep"]["exit_code"] = 1
        with self.assertRaisesRegex(ValueError, "failed parse preparation claimed coverage"):
            self.verify(report)

    def test_missing_export_and_restamped_critical_result_fail(self):
        report = json.loads(json.dumps(self.report))
        report["runs"][0]["critical_branch_census"]["covered"] = 1
        with self.assertRaisesRegex(ValueError, "critical branch census tampered"):
            self.verify(report)
        (self.raw_dir / "syntax-core.json").write_text("{}")
        with self.assertRaisesRegex(ValueError, "raw path or digest mismatch"):
            self.verify()

    def test_missing_critical_module_cannot_pass(self):
        report = json.loads(json.dumps(self.report))
        first = report["runs"][0]
        export_path = self.raw_dir / "syntax-core.json"
        export = json.loads(export_path.read_text())
        export["data"][0]["files"] = export["data"][0]["files"][:1]
        export_path.write_text(json.dumps(export))
        first["raw"]["export"]["sha256"] = v8_gate.digest(export_path.read_bytes())
        with self.assertRaisesRegex(ValueError, "production coverage tampered"):
            self.verify(report)

    def test_missing_wire_sibling_cannot_pass_even_when_report_is_restamped(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        export["data"][0]["files"] = [file for file in export["data"][0]["files"]
                                        if not file["filename"].endswith("/src/wire/command.rs")]
        self.restamp_export(index, export)
        status, _, _ = self.verify()
        self.assertEqual(status, "incomplete")
        self.assertIn("src/wire/command.rs", self.report["runs"][index]
                      ["critical_branch_census"]["missing_files"])

    def test_zero_branch_wire_file_cannot_pass(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        command = next(file for file in export["data"][0]["files"]
                       if file["filename"].endswith("/src/wire/command.rs"))
        command["summary"]["branches"] = {"count": 0, "covered": 0}
        command["branches"] = []
        self.restamp_export(index, export)
        status, _, _ = self.verify()
        self.assertEqual(status, "incomplete")
        self.assertIn("src/wire/command.rs", self.report["runs"][index]
                      ["critical_branch_census"]["missing_files"])

    def test_zero_branch_const_policy_requires_executed_line_function_and_region(self):
        for key, path in (("parse-default", "/src/dialect/v4.rs"),
                          ("rewrite-default", "/src/disposition.rs")):
            with self.subTest(key=key):
                index = next(i for i, row in enumerate(self.report["runs"])
                             if row["id"] == key)
                export_path = self.raw_dir / f"{key}.json"
                original = json.loads(export_path.read_text())
                policy = next(file for file in original["data"][0]["files"]
                              if file["filename"].endswith(path))
                policy["summary"]["branches"] = {"count": 0, "covered": 0}
                policy["branches"] = []
                self.restamp_export(index, original)
                self.assertEqual(self.verify()[0], "passed")
                for metric in ("lines", "functions", "regions"):
                    export = json.loads(export_path.read_text())
                    policy = next(file for file in export["data"][0]["files"]
                                  if file["filename"].endswith(path))
                    policy["summary"][metric]["covered"] = 0
                    self.restamp_export(index, export)
                    self.assertEqual(self.verify()[0], "incomplete")
                    self.assertIn(path.removeprefix("/"), self.report["runs"][index]
                                  ["critical_branch_census"]["missing_files"])
                    self.restamp_export(index, original)

    def test_zero_summary_policy_detail_still_requires_location_review(self):
        for key, suffix in (("parse-default", "/src/dialect/v4.rs"),
                            ("rewrite-default", "/src/disposition.rs")):
            with self.subTest(key=key):
                index = next(i for i, row in enumerate(self.report["runs"])
                             if row["id"] == key)
                export = json.loads((self.raw_dir / f"{key}.json").read_text())
                policy = next(file for file in export["data"][0]["files"]
                              if file["filename"].endswith(suffix))
                policy["summary"]["branches"] = {"count": 0, "covered": 0}
                policy["branches"] = [[1, 2, 1, 12, 0, 0, 0, 0, 4]]
                self.restamp_export(index, export)
                status, population, _ = self.verify()
                self.assertEqual(status, "incomplete")
                self.assertIn({"run": key, "file": suffix.removeprefix("/"),
                               "line": 1, "column": 2, "true_count": 0,
                               "false_count": 0}, population["critical_uncovered"])

    def test_past_reexport_cannot_replace_evaluator_coverage(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        past = next(file for file in export["data"][0]["files"]
                    if file["filename"].endswith("/src/past/mod.rs"))
        past["filename"] = past["filename"].replace("/mod.rs", "/evaluate.rs")
        self.restamp_export(index, export)
        self.assertEqual(self.verify()[0], "incomplete")
        self.assertIn("src/past/mod.rs", self.report["runs"][index]
                      ["critical_branch_census"]["missing_files"])

    def test_exact_source_bound_review_keeps_uncovered_location_visible(self):
        review, _ = self.reviewed_gap()
        status, population, _ = self.verify()
        self.assertEqual(status, "passed")
        self.assertEqual(population["reviewed_infeasibility"],
                         self.report["reviewed_infeasibility"])
        self.assertEqual(len(population["critical_uncovered"]), 1)
        self.assertEqual(population["critical_uncovered"][0]["file"], "src/past/mod.rs")
        self.assertLess(population["production_branches"]["covered"],
                        population["production_branches"]["count"])
        self.assertEqual(v8_gate.verify(
            json.dumps(self.report).encode(), self.raw_dir, self.graph, self.tools,
            expected_review_bytes=json.dumps(self.report["reviewed_infeasibility"]).encode(),
        )[0], "passed")
        with self.assertRaisesRegex(ValueError, "reviews differ from declared input"):
            v8_gate.verify(json.dumps(self.report).encode(), self.raw_dir,
                           self.graph, self.tools, expected_review_bytes=b"[]")

    def test_named_gap_cannot_waive_export_bound_file_summary(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        past = next(file for file in export["data"][0]["files"]
                    if file["filename"].endswith("/src/past/mod.rs"))
        past["summary"]["branches"] = {"count": 6, "covered": 4}
        past["branches"] = [[1, 2, 1, 12, 5, 0, 0, 0, 4],
                            [2, 2, 2, 12, 2, 0, 0, 0, 4],
                            [2, 2, 2, 12, 3, 1, 0, 0, 4]]
        self.restamp_export(index, export)
        row = self.report["runs"][index]
        self.assertEqual(self.verify()[0], "incomplete")

        gap = row["critical_uncovered"][0]
        source = Path(self.graph["tl-mltl"]["path"]) / gap["file"]
        source_sha = v8_gate.digest(source.read_bytes())
        location = {"run": row["id"], **gap,
                    "source_file_sha256": source_sha,
                    "reason": ("The absent source branch requires an impossible "
                               "validated predecessor state in this measured file."),
                    "reviewer": "Ada Reviewer"}
        summary = {"kind": "file_summary", "run": row["id"], "file": gap["file"],
                   "summary_missing_sides": 2,
                   "source_file_sha256": source_sha,
                   "raw_export_sha256": row["raw"]["export"]["sha256"],
                   "reason": ("The measured file summary has two missing sides "
                              "whose feasibility was reviewed over this source."),
                   "reviewer": "Ada Reviewer"}
        for reviews in ([location], [summary]):
            self.report["reviewed_infeasibility"] = reviews
            self.assertEqual(self.verify()[0], "incomplete")

        self.report["reviewed_infeasibility"] = [location, summary]
        row["status"] = "passed"
        row.pop("reason", None)
        self.report["status"] = "passed"
        status, population, _ = self.verify()
        self.assertEqual(status, "passed")
        self.assertEqual(population["critical_summary_missing"], [{
            "run": row["id"], "file": gap["file"],
            "summary_missing_sides": 2,
            "raw_export_sha256": row["raw"]["export"]["sha256"],
        }])

        for field, replacement, message in (
            ("summary_missing_sides", 3, "unknown or stale file summary"),
            ("raw_export_sha256", "0" * 64, "unknown or stale file summary"),
            ("source_file_sha256", "0" * 64, "source file digest is stale"),
            ("reason", "unreachable", "substantive infeasibility reason"),
            ("reviewer", "unknown", "named reviewer"),
        ):
            with self.subTest(field=field):
                original = summary[field]
                summary[field] = replacement
                with self.assertRaisesRegex(ValueError, message):
                    self.verify()
                summary[field] = original

        self.report["reviewed_infeasibility"].append(dict(summary))
        with self.assertRaisesRegex(ValueError, "duplicate V8 review"):
            self.verify()
        self.report["reviewed_infeasibility"].pop()
        row["coverage"]["files"][gap["file"]]["summary_missing_sides"] = 0
        with self.assertRaisesRegex(ValueError, "production coverage tampered"):
            self.verify()
        row["coverage"]["files"][gap["file"]]["summary_missing_sides"] = 2

        past["branches"][1][4] += 1
        self.restamp_export(index, export)
        self.report["runs"][index]["status"] = "passed"
        self.report["runs"][index].pop("reason", None)
        self.report["status"] = "passed"
        with self.assertRaisesRegex(ValueError, "unknown or stale file summary"):
            self.verify()

    def test_detail_omitted_from_summary_still_requires_named_review(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export = json.loads((self.raw_dir / "mltl-default.json").read_text())
        past = next(file for file in export["data"][0]["files"]
                    if file["filename"].endswith("/src/past/mod.rs"))
        for true_count, false_count in ((0, 0), (5, 0)):
            with self.subTest(detail=(true_count, false_count)):
                self.report["reviewed_infeasibility"] = []
                past["branches"] = [[1, 2, 1, 12, 7, 8, 0, 0, 4],
                                    [2, 2, 2, 12, true_count, false_count, 0, 0, 4]]
                self.restamp_export(index, export)
                row = self.report["runs"][index]
                self.assertEqual(row["critical_branch_census"]["files"]["src/past/mod.rs"],
                                 {"count": 2, "covered": 2})
                self.assertEqual(len(row["critical_uncovered"]), 1)
                self.assertEqual(self.verify()[0], "incomplete")
                gap = row["critical_uncovered"][0]
                source = Path(self.graph["tl-mltl"]["path"]) / gap["file"]
                review = {"run": row["id"], **gap,
                          "source_file_sha256": v8_gate.digest(source.read_bytes()),
                          "reason": ("The emitted detail belongs to a specialized "
                                     "instance absent from the file summary."),
                          "reviewer": "Ada Reviewer"}
                self.report["reviewed_infeasibility"] = [review]
                row["status"] = "passed"
                row.pop("reason", None)
                self.report["status"] = "passed"
                status, population, _ = self.verify()
                self.assertEqual(status, "passed")
                self.assertFalse(any(item["run"] == row["id"]
                                     for item in population["critical_summary_missing"]))

    def test_omitted_named_detail_cannot_waive_counted_duplicate_gap(self):
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export = json.loads((self.raw_dir / "mltl-default.json").read_text())
        past = next(file for file in export["data"][0]["files"]
                    if file["filename"].endswith("/src/past/mod.rs"))
        past["summary"]["branches"] = {"count": 4, "covered": 3}
        past["branches"] = [[1, 2, 1, 12, 4, 0, 0, 0, 4],
                            [1, 2, 1, 12, 7, 8, 0, 0, 4],
                            [2, 2, 2, 12, 5, 0, 0, 0, 4]]
        self.restamp_export(index, export)
        row = self.report["runs"][index]
        self.assertEqual(self.verify()[0], "incomplete")
        self.assertEqual(row["critical_uncovered"], [
            {"file": "src/past/mod.rs", "line": 2, "column": 2,
             "true_count": 5, "false_count": 0}])
        gap = row["critical_uncovered"][0]
        source = Path(self.graph["tl-mltl"]["path"]) / gap["file"]
        source_sha = v8_gate.digest(source.read_bytes())
        location = {"run": row["id"], **gap, "source_file_sha256": source_sha,
                    "reason": ("The one-sided detail is absent from the summary "
                               "and cannot cover the counted duplicate side."),
                    "reviewer": "Ada Reviewer"}
        summary = {"kind": "file_summary", "run": row["id"], "file": gap["file"],
                   "summary_missing_sides": 1,
                   "source_file_sha256": source_sha,
                   "raw_export_sha256": row["raw"]["export"]["sha256"],
                   "reason": ("The counted duplicate instance has one missing "
                              "side requiring its own file-summary review."),
                   "reviewer": "Ada Reviewer"}
        base_bytes = json.dumps(self.report).encode()
        report_path = self.root / "retained-native.json"
        review_path = self.root / "retained-reviews.json"
        report_path.write_bytes(base_bytes)
        for reviews, expected in (([location], "incomplete"),
                                  ([summary], "incomplete"),
                                  ([location, summary], "passed")):
            review_bytes = json.dumps(reviews).encode()
            review_path.write_bytes(review_bytes)
            status, population, artifacts = v8_gate.verify_retained(
                base_bytes, self.raw_dir, self.graph, self.tools,
                review_bytes, report_path, review_path)
            self.assertEqual(status, expected)
            self.assertEqual(population["retained_report_status"], "incomplete")
            self.assertEqual(artifacts["report"]["path"], str(report_path))
            self.assertEqual(artifacts["review_input"]["sha256"],
                             v8_gate.digest(review_bytes))
        with self.assertRaisesRegex(ValueError, "pinned input path"):
            v8_gate.verify_retained(base_bytes, self.raw_dir, self.graph,
                                    self.tools, json.dumps([location]).encode(),
                                    report_path, None)
        report_path.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "report bytes changed"):
            v8_gate.verify_retained(base_bytes, self.raw_dir, self.graph,
                                    self.tools, b"[]", report_path, None)
        report_path.write_bytes(base_bytes)
        review_path.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "review bytes changed"):
            v8_gate.verify_retained(base_bytes, self.raw_dir, self.graph,
                                    self.tools, b"[]", report_path, review_path)
        for reviews in ([location], [summary]):
            self.report["reviewed_infeasibility"] = reviews
            self.assertEqual(self.verify()[0], "incomplete")
        self.report["reviewed_infeasibility"] = [location, summary]
        row["status"] = "passed"
        row.pop("reason", None)
        self.report["status"] = "passed"
        status, population, _ = self.verify()
        self.assertEqual(status, "passed")
        self.assertEqual([item for item in population["critical_summary_missing"]
                          if item["run"] == row["id"]], [{
                              "run": row["id"], "file": gap["file"],
                              "summary_missing_sides": 1,
                              "raw_export_sha256": row["raw"]["export"]["sha256"],
                          }])
        reviewed_report_bytes = json.dumps(self.report).encode()
        report_path.write_bytes(reviewed_report_bytes)
        with self.assertRaisesRegex(ValueError, "must be unreviewed"):
            v8_gate.verify_retained(reviewed_report_bytes, self.raw_dir,
                                    self.graph, self.tools, b"[]", report_path, None)

    def test_stale_unknown_duplicate_or_weak_review_cannot_pass(self):
        review, source = self.reviewed_gap()
        for field, replacement, message in (
            ("line", review["line"] + 1, "unknown or stale gap"),
            ("false_count", 4, "unknown or stale gap"),
            ("source_file_sha256", "0" * 64, "source file digest is stale"),
            ("reason", "unreachable", "substantive infeasibility reason"),
            ("reviewer", "unknown", "named reviewer"),
        ):
            with self.subTest(field=field):
                original = review[field]
                review[field] = replacement
                with self.assertRaisesRegex(ValueError, message):
                    self.verify()
                review[field] = original
        self.report["reviewed_infeasibility"].append(dict(review))
        with self.assertRaisesRegex(ValueError, "duplicate V8 review"):
            self.verify()
        self.report["reviewed_infeasibility"].pop()
        source.write_text("fn revised_source() {}\n")
        with self.assertRaisesRegex(ValueError, "source file digest is stale"):
            self.verify()

    def test_unreviewed_gap_and_missing_file_cannot_claim_reviewed_pass(self):
        self.reviewed_gap()
        self.report["reviewed_infeasibility"] = []
        with self.assertRaisesRegex(ValueError, "critical target status mismatch"):
            self.verify()
        self.reviewed_gap()
        index = next(i for i, row in enumerate(self.report["runs"])
                     if row["id"] == "mltl-default")
        export_path = self.raw_dir / "mltl-default.json"
        export = json.loads(export_path.read_text())
        export["data"][0]["files"] = [
            file for file in export["data"][0]["files"]
            if not file["filename"].endswith("/src/wire/common.rs")]
        self.restamp_export(index, export)
        self.report["reviewed_infeasibility"] = []
        self.report["runs"][index]["status"] = "passed"
        self.report["runs"][index].pop("reason", None)
        self.report["status"] = "passed"
        with self.assertRaisesRegex(ValueError, "critical target status mismatch"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
