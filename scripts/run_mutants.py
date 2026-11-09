#!/usr/bin/env python3
"""Run cargo-mutants and judge the result against mutation-thresholds.toml.

Usage: run_mutants.py [--diff-base REF] [--jobs N] [--source-env] <out dir>

Without --diff-base this is the full run. With it, only mutants in the diff
`REF...HEAD` are generated and every one of them must be killed
(`diff_min_kill_percent`). The diff is `git diff REF...HEAD`, so only committed
changes are mutated; uncommitted edits are not. cargo-mutants exits 2 when
mutants survive and 3 when some mutants time out; both are judged by
mutation_gate.py (a timeout counts as not killed), so exit 0, 2 and 3 are all
accepted here and every other status is passed on. `--source-env` exports TL_MLTL_SOURCE_REVISION and
TL_MLTL_SOURCE_STATE for crates whose build.rs reads git, because cargo-mutants
builds in a copy of the tree without `.git`.

Kept out of the Makefile because the Makefile guard (NFR-006) refuses recipes
that join commands with a bare `;` or tolerate a failing status. Python 3.11 or
newer (the gate reads TOML with tomllib), stdlib only. The same script is used
in tl-syntax, tl-parse, tl-rewrite and tl-mltl; only tl-mltl keeps --source-env.
"""
import os
import subprocess
import sys


def main(argv):
    args = argv[1:]
    diff_base = None
    jobs = "4"
    source_env = False
    while args and args[0].startswith("--"):
        flag = args.pop(0)
        if flag == "--diff-base":
            diff_base = args.pop(0)
        elif flag == "--jobs":
            jobs = args.pop(0)
        elif flag == "--source-env":
            source_env = True
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 2
    out_dir = args[0]
    os.makedirs(out_dir, exist_ok=True)
    env = dict(os.environ)
    if source_env:
        revision = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
        dirty = subprocess.check_output(
            ["git", "status", "--porcelain", "--untracked-files=no"], text=True
        ).strip()
        env["TL_MLTL_SOURCE_REVISION"] = revision
        env["TL_MLTL_SOURCE_STATE"] = "modified" if dirty else "clean"
    command = ["cargo", "mutants", "-j", jobs, "--no-shuffle", "--output", out_dir]
    if diff_base:
        diff_path = os.path.join(out_dir, "changes.diff")
        with open(diff_path, "wb") as handle:
            subprocess.run(["git", "diff", f"{diff_base}...HEAD"], stdout=handle, check=True)
        command += ["--in-diff", diff_path]
    status = subprocess.run(command, env=env).returncode
    if status not in (0, 2, 3):
        print(f"run_mutants: cargo mutants failed with status {status}", file=sys.stderr)
        return status
    gate = [sys.executable, os.path.join(os.path.dirname(__file__), "mutation_gate.py")]
    if diff_base:
        gate.append("--diff")
    gate.append(os.path.join(out_dir, "mutants.out"))
    return subprocess.run(gate).returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv))
