---
id: TM-001
title: tl-mltl v0.1 test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# tl-mltl v0.1 Test Matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1 through FR-001-AC-3 | TC-001 through TC-004 | ✅ covered |
| FR-002 | FR-002-AC-1 through FR-002-AC-3 | TC-005 through TC-008 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-008 through TC-010, TC-032 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-011 through TC-013 | ✅ covered |
| FR-005 | FR-005-AC-1 through FR-005-AC-3 | TC-014 through TC-016 | ✅ covered |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7 | TC-018, TC-019, TC-020, TC-022, TC-023, TC-024 | ✅ covered |
| FR-007 | FR-007-AC-1 through FR-007-AC-9 | TC-025 through TC-031, TC-034, TC-035 | ✅ covered |
| FR-016 | FR-016-AC-1 through FR-016-AC-6 | TC-076 through TC-080 | ✅ covered |
| FR-017 | FR-017-AC-1 through FR-017-AC-3 | TC-081 through TC-083 | ✅ covered |
| FR-018 | FR-018-AC-1 through FR-018-AC-3 | TC-090 | 🚧 planned |
| FR-019 | FR-019-AC-1 through FR-019-AC-3 | TC-085 | ⛔ retired — superseded by quire-mltl FR-002 (TL-180); TL-179 removed TC-085's backing test with `wire::observation` |
| FR-027 | FR-027-AC-1 through FR-027-AC-3 | TC-086, TC-138 | 🚧 planned |
| FR-028 | FR-028-AC-1 through FR-028-AC-3 | TC-087, TC-138, TC-139 | 🚧 planned |
| FR-029 | FR-029-AC-1 through FR-029-AC-3 | TC-088, TC-089, TC-140 | 🚧 planned |

FR-027 through FR-029 retain TC-086 through TC-089 as planned rows. TM-004
adds the feature-boundary and infinite-semantics checks, including their
shared rows. The TL-native profile is `mltl.infinite-trace/v1`.

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-001, FR-005, FR-016 | TC-001, TC-002, TC-015, TC-076 | ✅ covered |
| StR-002 | FR-002, FR-003, FR-004, FR-006, FR-017 | TC-006, TC-009, TC-011, TC-023, TC-081 | ✅ covered |
| StR-003 | FR-007 | TC-025 through TC-028, TC-031, TC-034 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic, digest, and resource-limit tests | TC-003, TC-004, TC-006, TC-013, TC-025, TC-028, TC-029, TC-034 | ✅ covered |
| NFR-002 | schema-negative tests, contextual identity checks, compiled-test census, and tracked review-identity census | TC-012, TC-014, TC-016, TC-017, TC-025, TC-028, TC-031, TC-033, TC-034, TC-035, TC-082 | ✅ covered |
| NFR-003 | producer-boundary, shared-input serialization, state-vocabulary, mutation-probe, and hosted-tool identity tests | TC-018, TC-019, TC-022, TC-024, TC-036 | ✅ covered |
| NFR-006 | static Makefile/environment inspection, per-gate completion records, and declared/executed reconciliation | TC-130 through TC-137 | ✅ implemented |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Evaluate Boolean and unary temporal primitives | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-002 | Evaluate Until, Release, nesting, and closed boundaries | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-003 | Preserve result identities and deterministic outcomes | Unit | P0 | FR-001-AC-2, NFR-001-AC-1 | ✅ implemented |
| TC-004 | Reject profile mismatch and work-limit exhaustion | Unit | P0 | FR-001-AC-3, NFR-001-AC-2 | ✅ implemented |
| TC-005 | Match shared-corpus horizon oracles | Integration | P0 | FR-002-AC-1 | ✅ implemented |
| TC-006 | Detect checked arithmetic/resource overflow | Unit | P0 | FR-002-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-007 | Retain horizon identities and units | Unit | P1 | FR-002-AC-3 | ✅ implemented |
| TC-008 | Match prefix decision deadlines | Unit | P0 | FR-002-AC-1, FR-003-AC-1 | ✅ implemented |
| TC-009 | Preserve pending and early decisive verdicts | Unit | P0 | FR-003-AC-1, FR-003-AC-3 | ✅ implemented |
| TC-010 | Make closed prefixes equal closed evaluation | Unit | P0 | FR-003-AC-2 | ✅ implemented |
| TC-011 | Emit stable supported monitor mapping | Unit | P0 | FR-004-AC-1 | ✅ implemented |
| TC-012 | Reject unsupported adapter inputs | Unit | P0 | FR-004-AC-2, NFR-002-AC-1 | ✅ implemented |
| TC-013 | Verify mapping identities and digests | Unit | P0 | FR-004-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-014 | Exercise deterministic CLI schemas | Integration | P0 | FR-005-AC-1, NFR-001-AC-1, NFR-002-AC-1 | ✅ implemented |
| TC-015 | Compare supported and non-conclusive differential cases | Integration | P0 | FR-005-AC-2, StR-001-VC-2 | ✅ implemented |
| TC-016 | Verify retained differential inputs and non-conclusive cases are complete | Integration | P0 | FR-005-AC-3, NFR-002-AC-2 | ✅ implemented |
| TC-017 | Bind every requirement-tagged Rust test to a test Cargo compiles and runs, with none ignored or configured out | Integration | P0 | NFR-002-AC-3 | ✅ implemented |
| TC-018 | Classify every shared pin through the packaged compatibility matrix and refuse a mirror reference, with shared-input access serialized through the required guard | Integration | P0 | FR-006-AC-1, NFR-003-AC-1 | ✅ implemented |
| TC-019 | Reach Quoin without Quoin, Quire, or any gate executing a producer or the external monitor; require shared-input access through a serialized guard, and pair the injected-child refusal with an unmodified-driver run whose owned Quoin store is outside the repository store | Integration | P0 | FR-006-AC-2, NFR-003-AC-1, NFR-003-AC-2 | ✅ implemented |
| TC-020 | Bind the sealed record's impact snapshot to a Quire export that names every requirement | Integration | P0 | FR-006-AC-3 | ✅ implemented |
| TC-022 | Demonstrate twelve verification outcomes, each paired with an accepted positive control, and refuse a control naming a scenario that does not exist | Integration | P0 | FR-006-AC-5, NFR-003-AC-3 | ✅ implemented |
| TC-023 | Keep the R2U2 differential a comparison: three classifications, four external states, counts from the corpus manifests, and survival into the retained bytes | Integration | P0 | FR-006-AC-6, StR-002-VC-1, StR-002-VC-2 | ✅ implemented |
| TC-024 | Leave no local evidence framework or retained legacy evidence; enumerate every tracked and untracked-not-ignored repository path through Git, compare exact denial and deleted-reference sets with the change declaration, constrain exact tracked per-area cardinalities and declaration-exemption sets, reject renamed legacy-compatibility paths and obligations, scan raw bytes, and retain negative controls for hostile Git templates, ignored preferred makefiles, the plain compatibility-target name, non-UTF-8 content, unreadable paths, and enumeration failure | Integration | P0 | FR-006-AC-7, NFR-003-AC-1 | ✅ implemented |
| TC-025 | Bind shared signal/context identity and exact tl-mltl/tl-syntax revisions into contextual closed/prefix evaluation and horizon records | Integration | P0 | FR-007-AC-1, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-4 | ✅ implemented |
| TC-026 | Render exact valid non-reserved Boolean signal names in C2PO expressions and refuse every lexical, reserved, or unresolved name case without output | Property | P0 | FR-007-AC-2, StR-003-VC-2 | ✅ implemented |
| TC-027 | Compare contextual external verdicts without executing a monitor and keep context/version mismatch, semantic mismatch, and non-conclusive states distinct | Integration | P0 | FR-007-AC-3, StR-003-VC-2 | ✅ implemented |
| TC-028 | Detect every independent catalog, context, semantic input, revision, expression, external identity, and outcome mutation through native digests or typed refusal | Property | P0 | FR-007-AC-4, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-4 | ✅ implemented |
| TC-029 | Preserve existing context-free API/CLI behavior and exact v1 bytes while strictly round-tripping all contextual v2 native record families | Snapshot | P0 | FR-007-AC-5, NFR-001-AC-1 | ✅ implemented |
| TC-030 | Carry contextual native records through the existing producer-owned Quoin intake without new generic machinery or external execution | Integration | P0 | FR-007-AC-6 | ✅ implemented |
| TC-031 | Exercise the bounded overlay-response example with one shared named-signal catalog and requirement context across every contextual operation | Integration | P0 | FR-007-AC-7, StR-003-VC-1, NFR-002-AC-4 | ✅ implemented |
| TC-032 | Generate 512 bounded closed/prefix formula and trace inputs; closing the prefix agrees with closed evaluation | Property | P0 | FR-003-AC-2 | ✅ implemented |
| TC-033 | Refuse an empty tracked SpecReview set and duplicate semantic YAML identities while naming each colliding path | Integration | P0 | NFR-002-AC-5 | ✅ implemented |
| TC-034 | With all other inputs fixed, keep every contextual request and result identity invariant across absent, empty, and distinct valid diagnostic formula spans; retain those structural span differences and keep exact mapping formula bytes bound | Integration | P0 | FR-007-AC-8, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-4 | ✅ implemented |
| TC-035 | Refuse a structurally valid borrowed formula above the canonical tl-syntax document node limit with a typed contextual identity failure before operation work | Integration | P0 | FR-007-AC-9, NFR-002-AC-1 | ✅ implemented |
| TC-036 | Select semantic job-step run scalars before enforcing one executable scoped ix-flow 0.2.3 package across command-position bare/path-qualified npm after assignments, shell groups, the complete documented npm-install alias family, and literal nested shells; fail closed on unquoted redirection and executable expansion scripts; ignore inert command arguments and non-`-c` shell invocations without suppressing later commands, distinguish true comments from started-word hashes, reject alternates, and observe the semantic manual-trigger set and exact runtime | Integration | P0 | NFR-003-AC-5 | ✅ implemented |
| TC-076 | For W and M over every window within [0,3], every two-proposition trace up to length 5, and verdict times 0 through 2, closed-trace verdicts of the tl-syntax-lowered graph equal an independent first-occurrence direct reference for plain, constant, repeated, nested, negated, and correlated operands | Integration | P0 | FR-016-AC-1 | ✅ implemented |
| TC-077 | Open-prefix verdicts of lowered W/M equal the exact continuation verdict and exercise pending, closed prefixes equal the closed direct reference, and progress along longer traces never retracts and decides at the horizon | Integration | P0 | FR-016-AC-2 | ✅ implemented |
| TC-078 | Lowered and directly constructed W/M horizon reports are equal under both profiles and match the direct lookahead at [0,0], [0,u32::MAX], [u32::MAX,u32::MAX], and nested maximum windows without wrapping | Integration | P0 | FR-016-AC-3 | ✅ implemented |
| TC-079 | Lowered W/M nodes, spans, and reports equal direct construction, share exact work, recursion, temporal-span, and time-overflow outcomes under both profiles, and respect the formula-v1 node budget boundary | Integration | P0 | FR-016-AC-4 | ✅ implemented |
| TC-080 | Eight wrong W/M expansions each disagree with the direct reference while the real lowering disagrees nowhere, and the evaluator source and lowered graphs contain no derived future vocabulary | Integration | P0 | FR-016-AC-5, FR-016-AC-6 | ✅ implemented |
| TC-081 | Every digest-verified W/M corpus case lowers to its pinned canonical document, and each online-prefix lowered graph yields a C2PO manifest identical to its direct canonical pair with no W or M token | Integration | P0 | FR-017-AC-1 | ✅ implemented |
| TC-082 | Every closed-trace W/M corpus case, lowered or direct, is refused for C2PO with no manifest, and every refused lowering yields its declared refusal code and no graph | Integration | P0 | FR-017-AC-2, NFR-002-AC-1 | ✅ implemented |
| TC-083 | The recorded tl-parse cross-check is pinned and not a dependency, the corpus is consumed at the compiled tl-syntax revision, exported manifests name no external tool and carry the non-qualification limitation, and no source names FRETish | Integration | P0 | FR-017-AC-3 | ✅ implemented |
| TC-085 | Bind QObs C00 and preserve temporal dispatch while refusing repair and query semantics | Integration | P0 | FR-019-AC-1, FR-019-AC-2, FR-019-AC-3 | ⛔ retired — FR-019 is superseded by quire-mltl FR-002 (TL-180); its test was removed with `tests/tc_084_temporal_owner_wire.rs` (TL-179) |
| TC-086 | Compare existing bounded API and golden bytes in default and feature-enabled builds | Integration | P0 | FR-027-AC-1, FR-027-AC-3 | 🚧 planned |
| TC-087 | Check feature-off capability absence and bounded result/CLI byte stability | Integration | P0 | FR-028-AC-1, FR-028-AC-3 | 🚧 planned |
| TC-088 | Inspect Linear dependency order and TL-215 acceptance gate before implementation or qualification | Inspection | P1 | FR-029-AC-1, FR-029-AC-3 | 🚧 planned |
| TC-089 | Bind infinite evidence to the exact provider, TL profile, graph and clock identities | Integration | P0 | FR-029-AC-2 | ✅ implemented |
| TC-090 | Confirm every retained TL-owned wire contract (`trace/v1`, `command/v1`, `position-history/v1`, `history-requirement/v1`, `past-evaluation/v1`) publishes immutable schema bytes/digest, strictly rejects malformed/noncanonical input under enforced `OwnerLimits`, and that removing the QObs-coupled request/result/mapping layer (TL-179) leaves existing future/past/CLI/legacy-mapping behavior unchanged | Inspection | P1 | FR-018-AC-1, FR-018-AC-2, FR-018-AC-3 | 🚧 planned |
| TC-130 | Static inspector rejects each of `SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, `.SILENT:`, a `-`-prefixed recipe line, `\|\| true`/`\|\| :`, a stderr-to-`/dev/null` redirect, `$(eval`, a bare `;` command separator (including a quoted-value/trailing-comment MAKEFLAGS variation and a check-and-record chain), and a surface inside a recursively scanned `include`d file; a clean control and `for`/`while`/`if`/`case` control-flow semicolons are accepted without a false positive | Unit | P0 | NFR-006-AC-1 | ✅ implemented |
| TC-131 | Entry point invoked with `MAKEFLAGS` set to an `-i`/`-k`/`-S`-equivalent value, including a bundled short-flag token in both `ik` and `-ik` form, in the environment is refused before Make starts; a clean environment is not; `\|\| true` is detected wherever it appears on a recipe line, including immediately before a trailing `;` and further commands | Unit | P0 | NFR-006-AC-2 | ✅ implemented |
| TC-132 | Running each of the 15 `ci` prerequisites individually against its real recipe writes exactly one completion record naming that gate and its exit status; a recipe forced to fail writes no record for that gate | Integration | P0 | NFR-006-AC-3 | ✅ implemented |
| TC-133 | Given a completion-record set missing one declared gate and carrying one record for an undeclared gate, the entry point reports both mismatches by name; given an exact match, it reports none | Integration | P0 | NFR-006-AC-4 | ✅ implemented |
| TC-134 | A completion record deleted after being written, and one rewritten with a run-id preceding the evaluated run, are each treated as absent by reconciliation | Integration | P0 | NFR-006-AC-5 | ✅ implemented |
| TC-135 | The entry point run against a `.IGNORE:`-prepended copy of the real Makefile, a skeleton Makefile with every recipe replaced by a failing stub, and a recipe line joining a failing check and the completion-record call with a bare `;`, each exits non-zero with a named violation — the direct reproduction of the tracked measurement in TL-65/`agent-ix/tl-mltl#14` | Integration | P0 | NFR-006-AC-6 | ✅ implemented |
| TC-136 | The entry point run against an unmodified fixture Makefile with a clean environment and every gate passing, including ordinary shell control-flow semicolons, exits zero with no violation reported | Integration | P0 | NFR-006-AC-7 | ✅ implemented |
| TC-137 | Inspect README, CLAUDE.md, and `.github/workflows/*.yml` for every reference to running the full local gate set; each names the entry point (`make guarded-ci`), not a bare `make ci` | Inspection | P1 | NFR-006-AC-8 | ✅ implemented |
