"""Selection controls for the live four-crate mutation gate."""

import json
import tempfile
import unittest
from pathlib import Path

import v5_gate


class V5SelectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.graph = {name: {"path": str(self.root / name), "revision": str(index) * 40}
                      for index, name in enumerate(v5_gate.CRATES, 1)}
        self.runs = [{"crate": name, "source_path": entry["path"],
                      "source_revision": entry["revision"],
                      "source_file": "src/semantic.rs", "selection_regex": "operator",
                      "critical_scope": "critical temporal operator",
                      "test_tail": ["--lib"], "survivor_reviews": []}
                     for name, entry in self.graph.items()]
        self.path = self.root / "selection.json"

    def read(self, rows):
        self.path.write_text(json.dumps({"schema": "tl-mltl.v5-mutation-selection/v1",
                                         "runs": rows}))
        return v5_gate.selection(self.path, self.graph)

    def test_exact_four_source_pins_are_accepted_in_any_fixed_order(self):
        self.assertEqual(len(self.read(list(reversed(self.runs)))["runs"]), 4)

    def test_missing_owner_and_source_escape_refuse(self):
        with self.assertRaisesRegex(ValueError, "exactly four"):
            self.read(self.runs[:3])
        changed = json.loads(json.dumps(self.runs))
        changed[0]["source_file"] = "../tests/semantic.rs"
        with self.assertRaisesRegex(ValueError, "not a production Rust file"):
            self.read(changed)
        changed = json.loads(json.dumps(self.runs))
        changed[0]["source_revision"] = "f" * 40
        with self.assertRaisesRegex(ValueError, "source graph changed"):
            self.read(changed)


if __name__ == "__main__":
    unittest.main()
