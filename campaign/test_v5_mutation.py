"""FR-047/TC-183–184: native V5 population and fault controls."""

from __future__ import annotations

import copy
import subprocess
import tempfile
import unittest
from pathlib import Path

import v5_mutation as v5
import v5_run

TAIL = ["--lib", "--test", "infinite_trace"]
ARGV = ["cargo", "test", "--package=tl-mltl@0.3.0", "--all-features", *TAIL]


def fixture() -> tuple[list[dict], list[dict], dict, list[dict]]:
    discovered = [{"name": f"mutant-{index}", "file": "src/infinite/mod.rs"}
                  for index in range(12)]
    selected = discovered[:10]
    outcomes = [{"scenario": "Baseline", "summary": "Success", "phase_results": [
        {"phase": "Build", "process_status": "Success", "argv": ARGV[:-3]},
        {"phase": "Test", "process_status": "Success", "argv": ARGV},
    ]}]
    for index, mutant in enumerate(selected):
        outcomes.append({
            "scenario": {"Mutant": mutant},
            "summary": "MissedMutant" if index == 9 else "CaughtMutant",
            "phase_results": [{"phase": "Test", "process_status": "Success",
                               "argv": ARGV}],
        })
    native = {"outcomes": outcomes, "total_mutants": 10, "caught": 9,
              "missed": 1, "timeout": 0, "unviable": 0,
              "cargo_mutants_version": "fixture"}
    reviews = [{"name": "mutant-9", "disposition": "proof_candidate",
                "detail": "The admitted owner shape makes this predicate unreachable."}]
    return discovered, selected, native, reviews


class V5MutationTests(unittest.TestCase):
    # TC-183: selected, viable, caught, missed, and not-run reconcile exactly.
    def test_completed_population_passes_only_at_declared_threshold(self) -> None:
        discovered, selected, native, reviews = fixture()
        result = v5.classify(discovered, selected, native, TAIL, reviews)
        self.assertEqual(result["status"], "passed")
        self.assertEqual((result["discovered"], result["selected"], result["not_run"]),
                         (12, 10, 2))
        self.assertEqual(result["kill_rate"], {"caught": 9, "viable": 10})
        v5.exact_selection(discovered, selected, "src/infinite/mod.rs", r"mutant-[0-9]$")
        with self.assertRaisesRegex(ValueError, "fixed discovery filter"):
            v5.exact_selection(discovered, selected, "src/infinite/mod.rs", r"mutant-[0-9]+$")
        native["outcomes"][8]["summary"] = "MissedMutant"
        native["caught"] = 8
        native["missed"] = 2
        reviews.append({"name": "mutant-7", "disposition": "reviewed_limitation",
                        "detail": "Independent fixture retains this survivor."})
        self.assertEqual(v5.classify(discovered, selected, native, TAIL, reviews)["status"],
                         "failed")

    # TC-184: a survivor, timeout, altered test command or omitted case cannot
    # be silently removed from the denominator or credited as a completed pass.
    def test_seeded_survivor_timeout_selection_and_restoration_faults(self) -> None:
        discovered, selected, native, reviews = fixture()
        with self.assertRaisesRegex(ValueError, "survivor review"):
            v5.classify(discovered, selected, native, TAIL, [])
        timeout = copy.deepcopy(native)
        timeout["outcomes"][-1]["summary"] = "Timeout"
        timeout["missed"] = 0
        timeout["timeout"] = 1
        self.assertEqual(v5.classify(discovered, selected, timeout, TAIL, reviews)["status"],
                         "incomplete")
        altered = copy.deepcopy(native)
        altered["outcomes"][1]["phase_results"][0]["argv"] = ["cargo", "test", "--lib"]
        with self.assertRaisesRegex(ValueError, "test selection changed"):
            v5.classify(discovered, selected, altered, TAIL, reviews)
        with self.assertRaisesRegex(ValueError, "exactly cover"):
            v5.classify(discovered, selected, native | {"outcomes": native["outcomes"][:-1]},
                        TAIL, reviews)

        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            subprocess.run(["git", "init", "-q", str(source)], check=True)
            (source / "source.rs").write_text("fn stable() {}\n")
            subprocess.run(["git", "add", "source.rs"], cwd=source, check=True)
            subprocess.run(["git", "-c", "user.name=Test",
                            "-c", "user.email=test@example.invalid", "commit", "-qm", "fixed"],
                           cwd=source, check=True)
            revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=source, check=True,
                                      capture_output=True, text=True).stdout.strip()
            v5.exact_source(source, revision)
            (source / "source.rs").write_text("fn changed() {}\n")
            with self.assertRaisesRegex(ValueError, "restoration is dirty"):
                v5.exact_source(source, revision)
            with self.assertRaisesRegex(ValueError, "source revision changed"):
                v5.exact_source(source, "0" * 40)

    def test_producer_uses_a_copy_and_fixed_test_tail(self) -> None:
        argv = v5_run.command("src/infinite/mod.rs", "evaluate_lasso", Path("raw"), TAIL)
        self.assertEqual(argv[-len(TAIL):], TAIL)
        self.assertNotIn("--in-place", argv)
        self.assertEqual(argv[argv.index("--file") + 1], "src/infinite/mod.rs")
        self.assertEqual(argv[argv.index("--re") + 1], "evaluate_lasso")
        with self.assertRaisesRegex(ValueError, "must be fixed"):
            v5_run.command("src/infinite/mod.rs", "", Path("raw"), TAIL)
        restored_argv = v5_run.restored_control_command(TAIL)
        self.assertEqual(restored_argv, ["cargo", "test", "--locked", "--all-features", *TAIL])
        restored = {"source_revision": "a" * 40, "argv": restored_argv, "exit_code": 0}
        v5.verify_restored_control(restored, "a" * 40, TAIL)
        with self.assertRaisesRegex(ValueError, "source revision differs"):
            v5.verify_restored_control(restored, "b" * 40, TAIL)
        with self.assertRaisesRegex(ValueError, "test selection differs"):
            v5.verify_restored_control(restored | {"argv": restored_argv[:-1]},
                                       "a" * 40, TAIL)
        with self.assertRaisesRegex(ValueError, "green control failed"):
            v5.verify_restored_control(restored | {"exit_code": 1}, "a" * 40, TAIL)


if __name__ == "__main__":
    unittest.main()
