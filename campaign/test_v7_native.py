"""Fault controls for the native V7 process-result parser."""

import unittest

from v7_native import PROBES, classify


class NativeV7ResultTests(unittest.TestCase):
    def test_exact_selected_miri_test_and_build_are_required(self):
        miri = next(probe for probe in PROBES if probe.kind == "miri")
        passed = (f"running 1 test\ntest {miri.test_name} ... ok\n"
                  "test result: ok. 1 passed; 0 failed; 0 ignored; "
                  "0 measured; 7 filtered out")
        self.assertEqual(classify(miri, 0, passed), "passed")
        self.assertEqual(classify(miri, 1, passed), "failed")
        self.assertEqual(classify(miri, 0, passed.replace("running 1 test", "running 0 tests")),
                         "unavailable")
        self.assertEqual(classify(miri, 0, passed.replace(miri.test_name, "other_test")),
                         "unavailable")
        self.assertEqual(classify(miri, 0, passed.replace("1 passed", "0 passed")),
                         "unavailable")
        self.assertEqual(classify(miri, 0, passed + "\n" + passed), "unavailable")

        build = next(probe for probe in PROBES if probe.kind == "build")
        self.assertEqual(classify(build, 0, "Finished `dev` profile"), "passed")
        self.assertEqual(classify(build, 0, "empty"), "unavailable")
        self.assertEqual(classify(build, 101, "Finished `dev` profile"), "failed")


if __name__ == "__main__":
    unittest.main()
