"""Coverage export controls for production scoping and branch gaps."""

import tempfile
import unittest
from pathlib import Path

from v8_coverage import classify_export


class CoverageExportTests(unittest.TestCase):
    def test_production_branch_gap_is_located_and_tests_are_excluded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / "src" / "future.rs"
            test_file = root / "tests" / "future.rs"
            production.parent.mkdir()
            test_file.parent.mkdir()
            production.write_text("fn check() {}\n")
            test_file.write_text("fn test() {}\n")
            def file(path):
                return {"filename": str(path),
                        "summary": {"lines": {"count": 1, "covered": 1},
                                    "branches": {"count": 2, "covered": 1}},
                        "branches": [[1, 2, 1, 12, 7, 0, 0, 0, 4]]}
            export = {"type": "llvm.coverage.json.export", "data": [
                {"files": [file(production), file(test_file)]}]}
            measured = classify_export(export, root)
            self.assertEqual(list(measured["files"]), ["src/future.rs"])
            self.assertEqual(measured["totals"]["branches"], {"count": 2, "covered": 1})
            self.assertEqual(measured["files"]["src/future.rs"]
                             ["uncovered_branch_locations"], [
                                 {"line": 1, "column": 2, "true_count": 7,
                                  "false_count": 0}])

    def test_empty_or_impossible_production_measurement_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ValueError, "no production branches"):
                classify_export({"type": "llvm.coverage.json.export",
                                 "data": [{"files": []}]}, root)
            production = root / "src" / "future.rs"
            production.parent.mkdir()
            production.write_text("fn check() {}\n")
            with self.assertRaisesRegex(ValueError, "impossible coverage counts"):
                classify_export({"type": "llvm.coverage.json.export", "data": [{"files": [
                    {"filename": str(production),
                     "summary": {"lines": {"count": 1, "covered": 2},
                                 "branches": {"count": 1, "covered": 1}},
                     "branches": [[1, 0, 1, 1, 1, 1, 0, 0, 4]]}]}]}, root)


if __name__ == "__main__":
    unittest.main()
