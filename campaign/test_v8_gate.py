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
            "schema": "tl-mltl.v8-coverage/v1",
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


if __name__ == "__main__":
    unittest.main()
