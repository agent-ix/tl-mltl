---
id: SR-1518
title: "Evidence analysis of P-TL-151: ungrounded external claims"
type: SpecReview
analysis: evidence
scope: "external-format claims"
review_set: subset
---

## Summary

Retained bytes searched: tl-mltl corpus contains C2PO (12 formulas, FTSPEC/PTSPEC, `INPUT s0,s1: bool;`, `F[1,2] s0;`) and R2U2 outputs, and nothing for FRET, BLACK, nuXmv or LTLf (grep for fret, west, black, nuxmv, ltlf over corpus: no hits). WEST grammar and 20 lines are grounded in tl-rewrite corpus/west-v1 (formulas_d1.txt, 10 selectedCases, MIT licence). C2PO operators `&&`, `||`, `->`, `<->` appear in exporter source (src/infinite/export.rs:355-358, src/mapping/past.rs:166-169), not in retained corpus bytes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15181 | high | UNVERIFIED, no retained bytes: FRET MLTL export layout, operator tokens, C2PO bundle, variable map, trace header, "published FRET examples" (FR-087, FR-088, TC-240, TC-242). The author states this; TL-431 ACs cannot be evaluated until retrieval. | FR-087, FR-088 |
| FND-15182 | medium | UNVERIFIED: C2PO unsupported-construct list of 10 (FR-086); only partly grounded by exporter and corpus. | FR-086 |
| FND-15183 | medium | UNVERIFIED: BLACK input syntax, finite-trace mode, weak-next spelling (FR-307); nuXmv end-marker reduction as in De Giacomo, De Masellis, Montali AAAI 2014 and that paper's exact translation (FR-308, ADR-302); Johannsen et al. FMCAD 2025 encoding (ADR-301, not read). | FR-307, FR-308, ADR-301 |
| FND-15184 | medium | UNVERIFIED: LTLf2DFA delegating to MONA and BLACK as secondary reference (tl-mltl ADR-044); Aurandt et al. NFM 2026 and R2U2 documentation defining bare next differently (ADR-045 tl-syntax); literature's named finite-versus-infinite counterexample. | ADR-044 |
| FND-15185 | medium | Evidence methods: FR-093 ACs are Inspection (quire: mints no source symbol, validate errors), TL-428 AC3 and TL-431 AC1 are blocked or DEFERRED; Demonstration rows TC-321..325 are not automatable in CI. | FR-093, TM-300 |
| FND-15186 | low | Grounded and checked: WEST 20 lines, 10 selectedCases, MIT licence, upstream URL and path; C2PO 6+6 retained formulas. | tl-rewrite corpus |
