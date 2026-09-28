"""Pure preparation and retained-evidence checks for the false-pass diagnostic."""

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import false_pass_diagnostic as diagnostic


class FalsePassDiagnosticTests(unittest.TestCase):
    def test_transform_selects_one_member_without_changing_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            python, checker = root / "python3", root / "checker"
            python.write_bytes(b"python")
            checker.write_bytes(b"checker")
            definition = {"id": "original", "members": [
                {"name": diagnostic.MEMBER, "planId": "MP-008"},
                {"name": "V2.other", "planId": "MP-015"}],
                "sourceGraph": [{"repository": "tl-mltl", "revision": "a" * 40,
                                 "digest": "b" * 64}]}
            config = {"schema": "quoin.campaign-run-config/v1",
                      "sources": {"tl-mltl": "/old"},
                      "members": {diagnostic.MEMBER: {
                          "producer": {"producer": {"name": "cargo"},
                                       "environment": {"TL_MLTL_SOURCE_REVISION": "a" * 40,
                                                       "PATH": "/usr/bin:/bin"}},
                          "checker": {"producer": {"name": "tl_campaign_check"}},
                          "environmentSources": {
                              "TL_MLTL_SOURCE_REVISION": {
                                  "kind": "source_revision", "repository": "tl-mltl"}},
                          "sourceRemotes": {"tl-mltl": "https://example.invalid"}},
                          "V2.other": {}}}
            changed_definition, changed_config = diagnostic.transform(
                definition, config, "c" * 40, "d" * 64, root, python, "3.13.11", checker)
            self.assertEqual(len(changed_definition["members"]), 1)
            self.assertEqual(list(changed_config["members"]), [diagnostic.MEMBER])
            self.assertEqual(changed_definition["sourceGraph"][0]["revision"], "c" * 40)
            self.assertEqual(changed_config["members"][diagnostic.MEMBER]
                             ["producer"]["producer"]["executableDigest"],
                             hashlib.sha256(b"python").hexdigest())
            self.assertEqual(changed_config["members"][diagnostic.MEMBER]
                             ["producer"]["environment"]["TL_MLTL_SOURCE_REVISION"],
                             "c" * 40)
            self.assertEqual(changed_config["members"][diagnostic.MEMBER]
                             ["producer"]["environment"]["PATH"], "/usr/bin:/bin")
            self.assertEqual(definition["id"], "original")
            self.assertEqual(config["sources"]["tl-mltl"], "/old")
            self.assertEqual(config["members"][diagnostic.MEMBER]
                             ["producer"]["environment"]["TL_MLTL_SOURCE_REVISION"],
                             "a" * 40)

    def test_retained_exit_zero_failure_needs_checker_and_quoin_rejection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            store = repo / "spec/evidence/campaigns"
            for kind in ("runs", "results", "domain-verdicts"):
                (store / kind).mkdir(parents=True)
            (store / "runs/diagnostic.json").write_text(json.dumps({"attempts": [{
                "member": diagnostic.MEMBER, "status": "completed", "resultDigest": "a" * 64,
                "domainVerdictDigest": "b" * 64}]}))
            raw = diagnostic.FAILED_SUMMARY.encode()
            (store / "results" / ("a" * 64 + ".json")).write_text(json.dumps({
                "process": {"terminalStatus": {"kind": "exit_code", "value": 0},
                            "stdout": {"bytes": list(raw),
                                       "digest": hashlib.sha256(raw).hexdigest()}}}))
            (store / "domain-verdicts" / ("b" * 64 + ".json")).write_text(json.dumps({
                "verdict": "reject", "reasons": ["native_result_unproved"]}))
            receipt = {"definitionDigest": "c" * 64, "runDigest": "d" * 64,
                       "decision": {"verdict": "reject"}}
            report = diagnostic.assert_false_pass(repo, "diagnostic", receipt, receipt)
            self.assertEqual(report["reason"], "native_result_unproved")
            (store / "domain-verdicts" / ("b" * 64 + ".json")).write_text(json.dumps({
                "verdict": "accept", "reasons": []}))
            with self.assertRaisesRegex(AssertionError, "TL checker"):
                diagnostic.assert_false_pass(repo, "diagnostic", receipt, receipt)


if __name__ == "__main__":
    unittest.main()
