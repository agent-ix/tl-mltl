"""Fault controls for the V8 raw-export reconciliation gate."""

import json
import platform
import tempfile
import unittest
from pathlib import Path

import v8_gate
from v8_coverage import CRATES, CRITICAL_PREFIXES, EXPECTED_CRITICAL, classify_export


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
                relative = f"{prefix}mod.rs" if prefix.endswith("/") else prefix
                file = repo / relative
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text("fn check() {}\n")
                files.append({"filename": str(file),
                              "summary": {"lines": {"count": 1, "covered": 1},
                                          "branches": {"count": 2, "covered": 2}},
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
            critical_files = {file: details["branches"]
                              for file, details in coverage["files"].items()
                              if any(file.startswith(prefix) for prefix in
                                     CRITICAL_PREFIXES[name])}
            missing = [prefix for prefix in EXPECTED_CRITICAL[name][feature]
                       if not any(file.startswith(prefix) for file in critical_files)]
            rows.append({"id": key, "repo": name, "feature": feature,
                         "source_revision": self.graph[f"tl-{name}"]["revision"],
                         "argv": argv, "exit_code": 0, "raw": files,
                         "coverage": coverage, "critical_uncovered": [],
                         "critical_branch_census": {
                             "files": critical_files, "missing_files": missing,
                             "count": sum(item["count"] for item in critical_files.values()),
                             "covered": sum(item["covered"] for item in critical_files.values())},
                         "status": "passed"})
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

    def test_complete_raw_export_population_passes(self):
        status, population, artifacts = self.verify()
        self.assertEqual(status, "passed")
        total = sum(row["coverage"]["totals"]["branches"]["count"]
                    for row in self.report["runs"])
        self.assertEqual(population["production_branches"],
                         {"count": total, "covered": total})
        self.assertEqual(len(artifacts), 25)

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


if __name__ == "__main__":
    unittest.main()
