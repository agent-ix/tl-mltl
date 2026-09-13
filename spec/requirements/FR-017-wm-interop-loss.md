---
id: FR-017
title: Export W/M only as canonical graphs and refuse unpreservable targets
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-016
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-010
    type: depends_on
---

# FR-017: Export W/M only as canonical graphs and refuse unpreservable targets

## Description

Where a bounded weak-until (`W[a,b]`) or strong-release (`M[a,b]`) formula is
exported to a monitor target, tl-mltl shall export only the canonical primitive
graph returned by the tl-syntax `tl-syntax.future-operators/v1` lowering, shall
refuse with no manifest a target that cannot preserve the formula's semantic
profile, and shall not treat foreign parser or monitor acceptance as
qualification evidence.

## Inputs

- The retained byte-identical copy of tl-syntax `corpus/future-operators` at the
  compiled tl-syntax revision `5b1c13440e54d5a851df2d33cc88944135574bc6`,
  corpus `tl-syntax.future-operator-corpus/v1` revision 1, whose
  `manifest.json` SHA-256 is
  `e38ef2a7bfc49631932c9c8527b9d08ba1087825e8ae3bccff5f326e74605172` and pins
  every case and expected document.
- Its `derived`, `direct`, and `refused` cases, replayed through
  `tl_syntax::FutureLoweringRequest::lower` and `tl_syntax::Formula::new`.
- The R2U2/C2PO mapping target of FR-004, identified by the tl-mltl source
  revision and state the build records.
- The corpus manifest's recorded tl-parse cross-check revision
  `9ca856b4c040fc2c3329b6defd26a1c9b57de748`.

## Behavior

- R2U2/C2PO is the only monitor target. A lowered W/M graph under
  `mltl.online-prefix/v1` maps to exactly the manifest its directly constructed
  canonical pair maps to; the expression never re-sugars `W` or `M`.
- Target loss is explicit. Under `mltl.closed-trace/v1` the C2PO target cannot
  preserve the profile, so lowered and direct graphs alike are refused with
  `UnsupportedProfile` and no manifest. A refused lowering produces no graph,
  so nothing reaches a target.
- FRETish stays output-only outside this crate: tl-mltl has no FRETish emitter,
  importer, or target.
- tl-parse is a recorded upstream cross-check of the corpus sources. tl-mltl
  neither depends on nor runs it. No gate runs R2U2 or C2PO. Exported manifests
  name no external tool and keep the FR-004 non-qualification limitation.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Every retained corpus file matches the pinned manifest digest. Every derived case lowers to its pinned expected document, ignoring spans. Each of the seven online-prefix derived cases yields a C2PO manifest equal to its direct pair's, bound to the compiled tl-syntax revision, with no standalone `W` or `M` token. | Test (TC-081) |
| FR-017-AC-2 | All eight closed-trace derived cases and their direct pairs are refused for C2PO with `UnsupportedProfile` and no manifest. All sixteen refused cases produce their declared refusal code and no graph. | Test (TC-082) |
| FR-017-AC-3 | The corpus manifest's tl-parse cross-check equals the pinned parser revision, and `Cargo.toml` names no tl-parse dependency. `corpus/README.md` records the corpus at the compiled tl-syntax revision. Every exported manifest has no external tool and the non-qualification limitation. No file under `src/` names FRETish. | Test (TC-083) |

## Dependencies

Depends on FR-004 mapping and FR-016 lowered-graph parity. Implements the
tl-mltl lowered-graph export and target-profile loss/refusal cases of tl-syntax
FR-010 routed by
[agent-ix/tl-mltl#48](https://github.com/agent-ix/tl-mltl/issues/48). The
corpus comes from the tl-syntax#43 squash merge. The evaluator revision is the
tl-mltl source revision each manifest records.
