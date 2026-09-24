#!/usr/bin/env python3
"""Exercise the public feature boundary as a Cargo consumer."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


REPO = Path(__file__).resolve().parent.parent
FIXTURES = REPO / "tests/fixtures/cli-requests"
MANIFEST = tomllib.loads((REPO / "Cargo.toml").read_text())
assert MANIFEST["features"]["default"] == [], "the provider must not be a default feature"


def run(*args, input_bytes=None, expect_success=True):
    process = subprocess.run(
        args,
        cwd=REPO,
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if expect_success and process.returncode:
        raise AssertionError(
            f"{' '.join(map(str, args))} failed ({process.returncode}):\n"
            f"{process.stderr.decode(errors='replace')}"
        )
    return process


def cargo(*args, expect_success=True):
    return run("cargo", *args, expect_success=expect_success)


def consumer_manifest(features):
    syntax = MANIFEST["dependencies"]["tl-syntax"]
    syntax_features = ", ".join(f'"{feature}"' for feature in syntax["features"])
    mltl_features = ", ".join(f'"{feature}"' for feature in features)
    return (
        '[package]\nname = "mltl-feature-boundary-consumer"\n'
        'version = "0.0.0"\nedition = "2021"\n\n[dependencies]\n'
        f'tl-mltl = {{ path = "{REPO}", default-features = false, '
        f'features = [{mltl_features}] }}\n'
        f'tl-syntax = {{ version = "{syntax["version"]}", '
        f'git = "{syntax["git"]}", rev = "{syntax["rev"]}", '
        f'default-features = false, features = [{syntax_features}] }}\n'
    )


BOUNDED_AND_ABSENT = r"""
use tl_mltl::{EvaluationReport, TruthValue};
use tl_syntax::{
    settle_liveness, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind, LivenessDisposition, LivenessSubject, LivenessSubjectKind,
    NodeId, SemanticProfile, LIVENESS_CAPABILITY_V1,
};

fn main() {
    let _: Option<EvaluationReport> = None;
    let _: Option<TruthValue> = None;
    let formula = InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(0),
        vec![InfiniteNode::new(InfiniteNodeKind::True)],
    ).unwrap();
    let subject = LivenessSubject {
        kind: LivenessSubjectKind::LassoTrace,
        identity: "fixture-lasso",
    };
    let absent = settle_liveness(&formula, subject, None);
    assert_eq!(absent.formula, &formula);
    assert_eq!(absent.subject, subject);
    assert_eq!(absent.disposition, LivenessDisposition::Unsupported);
    assert_eq!(absent.warning, Some(LIVENESS_CAPABILITY_V1));
}
"""


def check_consumer(directory, enabled):
    source = directory / "src/main.rs"
    source.write_text(
        BOUNDED_AND_ABSENT
        + ('\nfn provider_identity() {\n'
           '    assert_eq!(tl_mltl::infinite::FEATURE, "infinite-trace");\n'
           '    assert_eq!(tl_mltl::infinite::PROFILE, "mltl.infinite-trace/v1");\n'
           '    assert_eq!(tl_mltl::infinite::CAPABILITY, tl_syntax::LIVENESS_CAPABILITY_V1);\n'
           '}\n' if enabled else "")
        .replace("fn main() {", "fn main() {\n    provider_identity();" if enabled else "fn main() {", 1)
    )
    cargo("check", "--manifest-path", str(directory / "Cargo.toml"))
    tree = cargo(
        "tree", "--manifest-path", str(directory / "Cargo.toml"),
        "--edges", "features", "--prefix", "none",
    ).stdout.decode()
    selected = 'tl-mltl feature "infinite-trace"' in tree
    assert selected == enabled, f"wrong resolved feature tree (enabled={enabled}):\n{tree}"
    packages = cargo(
        "tree", "--manifest-path", str(directory / "Cargo.toml"),
        "--edges", "normal", "--prefix", "none",
    ).stdout.decode()
    assert "tl-oracle" not in packages, "the dev-only oracle entered the production graph"
    if not enabled:
        source.write_text(BOUNDED_AND_ABSENT + "\nuse tl_mltl::infinite::CAPABILITY;\n")
        refused = cargo(
            "check", "--manifest-path", str(directory / "Cargo.toml"),
            expect_success=False,
        )
        stderr = refused.stderr.decode(errors="replace")
        assert (
            refused.returncode != 0
            and "could not find `infinite` in `tl_mltl`" in stderr
            and "gated behind the `infinite-trace` feature" in stderr
        ), stderr
        source.write_text(BOUNDED_AND_ABSENT)
    cargo("run", "--quiet", "--manifest-path", str(directory / "Cargo.toml"))


def bounded_cli_bytes(enabled, directory):
    flags = ["--features", "infinite-trace"] if enabled else ["--no-default-features"]
    cargo("build", "--bin", "tl-mltl", *flags)
    target = Path(os.environ.get("CARGO_TARGET_DIR", REPO / "target"))
    binary = target / "debug/tl-mltl"
    snapshot = directory / ("tl-mltl-on" if enabled else "tl-mltl-off")
    shutil.copy2(binary, snapshot)
    cases = json.loads((FIXTURES / "manifest.json").read_text())["cases"]
    results = {}
    for case in cases:
        output = run(snapshot, "-", input_bytes=(FIXTURES / case["request"]).read_bytes(), expect_success=False)
        assert output.returncode == case["expectedExit"], case["id"]
        results[case["id"]] = (output.returncode, output.stdout, output.stderr)
    return results


def main():
    with tempfile.TemporaryDirectory(prefix="mltl-feature-boundary-") as temp:
        directory = Path(temp)
        (directory / "src").mkdir()
        manifest = directory / "Cargo.toml"
        manifest.write_text(consumer_manifest([]))
        check_consumer(directory, enabled=False)
        off = bounded_cli_bytes(False, directory)
        manifest.write_text(consumer_manifest(["infinite-trace"]))
        check_consumer(directory, enabled=True)
        on = bounded_cli_bytes(True, directory)
        for case_id in off:
            assert off[case_id] == on[case_id], f"bounded CLI bytes changed: {case_id}"
        print(f"TC-086/TC-087: feature boundary and {len(off)} bounded CLI byte pairs pass")


if __name__ == "__main__":
    main()
