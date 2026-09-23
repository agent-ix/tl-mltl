"""V6 parser and native-contract faults for bounded proof credit."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import v1_campaign
import v6_kani


SYNTAX = v6_kani.CLAIMS["tl-syntax"][0]


def clean_output(harness: str) -> bytes:
    return (f"Checking harness {harness}...\n"
            "CBMC version 6.11.0 (cbmc-6.11.0) 64-bit arm64 macos\n"
            "Solving with CaDiCaL 3.0.0\n"
            f"Check 1: {harness}.assertion.1\n"
            "  - Status: SUCCESS\n"
            "SUMMARY:\n ** 0 of 1 failed\n"
            "VERIFICATION:- SUCCESSFUL\n"
            "Complete - 1 successfully verified harnesses, 0 failures, 1 total.\n").encode()


def false_output(rows: tuple[list[int], list[int]]) -> bytes:
    start, end = rows
    return (f"Checking harness {v6_kani.FALSE_NAME}...\n"
            "SUMMARY:\n ** 1 of 35 failed\n"
            "Failed Checks: assertion failed: interval.cardinality() == Some(1)\n"
            "VERIFICATION:- FAILED\n"
            "Concrete playback unit test for " + v6_kani.FALSE_NAME + ":\n"
            f"let concrete_vals = vec![vec!{start!r}, vec!{end!r}];\n").encode()


class V6KaniTests(unittest.TestCase):
    def test_clean_parser_requires_complete_checked_assertion(self) -> None:
        good = clean_output(SYNTAX)
        result = v6_kani.parse_clean(good, 0, SYNTAX)
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["checks"], 1)
        self.assertEqual(result["assertion_checks"], 1)
        self.assertEqual(result["solver_identity"], "CaDiCaL 3.0.0")
        for bad in (
            good.replace(b"0 of 1 failed", b"0 of 2 failed"),
            good.replace(b"Status: SUCCESS", b"Status: UNKNOWN"),
            good.replace(b".assertion.1", b".pointer_dereference.1"),
            good.replace(b"VERIFICATION:- SUCCESSFUL", b"VERIFICATION:- FAILED"),
            good.replace(b"Complete - 1 successfully verified harnesses, 0 failures, 1 total.", b""),
        ):
            self.assertEqual(v6_kani.parse_clean(bad, 0, SYNTAX)["status"], "incomplete")
        self.assertEqual(v6_kani.parse_clean(good, 124, SYNTAX)["status"], "incomplete")

    def test_false_assertion_requires_concrete_nontrivial_bytes(self) -> None:
        good = false_output(([0, 0, 0, 128], [0, 0, 0, 192]))
        result = v6_kani.parse_false(good, 1)
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["counterexample_bytes"],
                         [[0, 0, 0, 128], [0, 0, 0, 192]])
        self.assertEqual(result["cardinality"], 1_073_741_825)
        for bad in (
            good.replace(b"1 of 35 failed", b"0 of 35 failed"),
            good.replace(b"Concrete playback unit test for", b"Missing playback for"),
            false_output(([0, 0, 0, 128], [0, 0, 0, 128])),
        ):
            self.assertEqual(v6_kani.parse_false(bad, 1)["status"], "incomplete")
        self.assertEqual(v6_kani.parse_false(good, 0)["status"], "incomplete")
        self.assertIn("u32::from_le_bytes([0, 0, 0, 128])",
                      v6_kani.replay_test(result["counterexample_bytes"]))

    def test_source_harness_rejects_vacuous_assumption(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "src/formula/graph.rs"
            path.parent.mkdir(parents=True)
            path.write_text('''#[kani::proof]
    fn interval_cardinality_matches_wide_arithmetic() {
        let start: u32 = kani::any();
        let end: u32 = kani::any();
        assert!(start <= end);
    }
''')
            source = v6_kani.exact_harness_source(root, "src/formula/graph.rs", SYNTAX)
            self.assertEqual(source["symbolic_any_count"], 2)
            path.write_text(path.read_text().replace("assert!(start <= end);",
                                                     "kani::assume(start <= end);"))
            with self.assertRaisesRegex(ValueError, "vacuous"):
                v6_kani.exact_harness_source(root, "src/formula/graph.rs", SYNTAX)

    def test_arbitrary_v6_command_and_wrong_seed_are_incomplete(self) -> None:
        graph = {"tl-mltl": {"path": "/tmp", "revision": "0" * 40}}
        command = {"id": "bounded_proof", "milestone": "V6", "mode": "command",
                   "repo": "tl-mltl", "parser": "cargo_test",
                   "argv": ["python3", "-c", "print('test result: ok. 1 passed; "
                            "0 failed; 0 ignored; 0 measured; 0 filtered out')"],
                   "seed": {"kind": "none", "reason": "symbolic_no_random_seed"}}
        with tempfile.TemporaryDirectory() as directory:
            result, _ = v1_campaign.run_lane(command, graph, {}, Path(directory))
        self.assertEqual(result["status"], "incomplete")
        wrong = command | {"mode": "native", "seed": {"kind": "fixed", "value": 1}}
        wrong.pop("argv")
        wrong.pop("repo")
        wrong.pop("parser")
        result, _ = v1_campaign.run_lane(wrong, graph, {}, Path("/tmp"))
        self.assertEqual(result["status"], "incomplete")


if __name__ == "__main__":
    unittest.main()
