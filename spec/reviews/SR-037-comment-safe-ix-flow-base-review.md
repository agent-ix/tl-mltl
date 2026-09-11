---
id: SR-037
title: "Base specification review — comment-safe hosted ix-flow census"
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl#42; NFR-003-AC-5; TC-036; TM-001"
review_set: base
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: reviews
  - target: ix://agent-ix/tl-mltl/TM-001
    type: references
---

## Summary

Reviewed the issue #42 refinement that excludes YAML comment text from the
hosted ix-flow package population while preserving every existing executable,
trigger, and runtime control. The initial ambiguity around quoted hash
characters was corrected before implementation. Independent exact-head review
then exposed the distinct YAML-versus-shell comment boundary; that gap is now
specified and covered by the corrective implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3701 | medium | **FIXED:** merely saying comments are ignored did not define where comment text begins and could incorrectly discard a hash inside a quoted executable package token. NFR-003-AC-5 now removes text only from an unquoted `#` and preserves hashes inside single- or double-quoted tokens. | NFR-003-AC-5, TC-036 |
| FND-3702 | low | The corrected criterion keeps alias-form executable duplicates, every npm install spelling, the sole manual trigger, and the observed runtime version in scope; the matrix truthfully returns TC-036 to planned until its positive and negative comment controls exist. | NFR-003-AC-5, TC-036, TM-001 |
| FND-3703 | low | No language-boundary gap is introduced: this is internal hosted-assurance parsing and does not define a user-authored Quire or tl-syntax language surface. | NFR-003, owner ruling 2026-09-09 |
| FND-3704 | medium | **FIXED:** the existing sealed declaration named NFR-003 but omitted both the hosted workflow bytes and NFR-003-AC-5 from its record projection. `hosted-ci`, `.github`, and the criterion are now explicit source, subject, and definition entries, so the candidate claim cannot float free of the workflow it qualifies. | NFR-003-AC-5, TC-036, hosted-ci |
| FND-3705 | high | **FIXED after independent review of `ab6c4ce`:** globally removing every unquoted `#` treated word-internal hashes inside literal-block shell scripts as comments, hiding an executable alias install. The control now parses YAML before tokenizing each run script and applies shell word-boundary comment rules. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3706 | low | **FIXED after independent review of `ab6c4ce`:** plain-scalar metadata containing an apostrophe could distort global quote state and count inert package text. The YAML parser now selects only scalar `run` values; TC-036 covers apostrophe-bearing metadata and a quoted `run` key. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3707 | high | **FIXED after independent review of `4e8b3e9`:** a literal nested `bash -c` script flattened into one outer token and hid an executable alternate install. TC-036 now recursively classifies statically literal `sh`/`bash -c` scripts. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3708 | medium | **FIXED after independent review of `4e8b3e9`:** recursive selection treated `defaults.run.shell` metadata as an executable script. Selection is now bounded to semantic `jobs.*.steps[*].run` string values, with `defaults.run` and multiline-name controls. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3709 | low | **FIXED after independent review of `4e8b3e9`:** the sole-trigger assertion depended on exact raw layout around `on` and `jobs`. TC-036 now compares the parsed trigger set and accepts intervening top-level metadata. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3710 | high | **FIXED after independent review of `7649173`:** shell option detection treated `--norc` as `-c`, and scanning every argument treated inert command-shaped data as executable. The scanner now accepts only a short-option bundle containing `c` on the resolved command executable and TC-036 covers both controls. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3711 | low | **FIXED after independent review of `7649173`:** the trigger-metadata control edited the tracked workflow into invalid duplicate top-level `permissions` if that key already existed. It now parses an independent valid fixture with intervening metadata. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-mltl#43 review |
| FND-3712 | medium | **FIXED during cross-lane remediation after review of `215745b`:** leading redirections could hide command position and a non-`-c` shell command could suppress later commands. Command discovery now skips attached or separate leading redirections, non-`-c` shells end only their command inspection, and the inert fixture exposes exact npm/bash argv words. | NFR-003-AC-5, TC-036, tests/shared_assurance.rs, tl-parse#28 review, tl-rewrite#34 review |

## Verdict

**PASS** — the refined contract is bounded, measurable, and ready for a
Rust-only implementation. This owner-delegated specification review is not an
independent exact-head code review, hosted-run authorization, or release
decision.
