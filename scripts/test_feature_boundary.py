"""Executable consumer checks for the default and opt-in mlTL feature sets."""

import unittest

import check_feature_boundary


class FeatureBoundaryTests(unittest.TestCase):
    def test_external_consumer_and_bounded_cli_bytes(self) -> None:
        """Trace: TC-086, TC-087, FR-027-AC-1, FR-027-AC-3, FR-028-AC-1, FR-028-AC-3."""
        check_feature_boundary.main()


if __name__ == "__main__":
    unittest.main()
