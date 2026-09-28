//! FR-053/TC-193–194: exact production versus independent V11 lasso partition.
//!
//! The declared axes come from the dev-only oracle crate. The expected verdict
//! comes from its independent evaluator, while an independent Boolean count
//! checks fair-completion admission. This is a completed small partition;
//! `full_target_complete` remains false for the unbounded V11 objective.

#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, LassoRequest, ResultReason,
};
use tl_oracle::population::{CaseResult, PopulationLedger, PopulationSummary};
use tl_oracle::v11::{
    Fairness, V11Population, ADMITTED_CASES, ANCHORS, COMPLETE_WORD_COUNT, DECLARED_CASES,
    EMPTY_FAIR_CASES, FAIRNESS_COUNT, FORMULA_COUNT, MIXED_WORD_COUNT, SINGLE_UNKNOWN_WORD_COUNT,
    WORD_COUNT,
};
use tl_oracle::{evaluate_documents, Limits, OracleError, Verdict};
use tl_syntax::{FairnessPremisesDocument, InfiniteClock};

const SCOPE: &str = "formulas30_words372_fair3_anchors4";

fn expected_disposition(verdict: Verdict) -> Disposition {
    match verdict {
        Verdict::Proved => Disposition::Proved,
        Verdict::Refuted => Disposition::Refuted,
        Verdict::Inconclusive => Disposition::Inconclusive,
    }
}

// Trace: TC-193, TC-194; FR-053-AC-1, FR-053-AC-2
#[test]
fn tc_193_complete_v11_partition_matches_production_and_emits_native_census() {
    let population = V11Population::new();
    assert_eq!(population.formulas().len(), FORMULA_COUNT);
    assert_eq!(population.words().len(), WORD_COUNT);
    assert_eq!(DECLARED_CASES, 133_920);
    let mut ledger = PopulationLedger::new(DECLARED_CASES).unwrap();
    let mut expected_admitted = 0_u64;
    let mut expected_refused = 0_u64;
    for (formula_index, formula) in population.formulas().iter().enumerate() {
        let graph_id = formula.graph.content_identity().unwrap();
        let fairness_documents = Fairness::ALL.map(|mode| match mode {
            Fairness::None => None,
            Fairness::P | Fairness::Q => Some(
                FairnessPremisesDocument::new(
                    &formula.graph,
                    graph_id.clone(),
                    InfiniteClock::EventPosition,
                    mode.roots().to_vec(),
                )
                .unwrap(),
            ),
        });
        for (word_index, word) in population.words().iter().enumerate() {
            let trace_id = word.trace.content_identity().unwrap();
            for (fairness_index, fairness) in Fairness::ALL.into_iter().enumerate() {
                let fairness_document = fairness_documents[fairness_index].as_ref();
                let independent_admitted = word.expected_fair_completions(fairness);
                for (anchor_index, anchor) in ANCHORS.into_iter().enumerate() {
                    let id = population
                        .case_id(formula_index, word_index, fairness_index, anchor_index)
                        .unwrap();
                    let oracle = evaluate_documents(
                        &formula.graph,
                        &word.trace,
                        formula.graph.formula().root(),
                        fairness.roots(),
                        usize::try_from(anchor).unwrap(),
                        Limits::default(),
                    );
                    let provider = evaluate_lasso(&LassoRequest {
                        formula: &formula.graph,
                        trace: &word.trace,
                        fairness: fairness_document,
                        evidence_closure: EvidenceClosure::Closed,
                        graph_id: &graph_id,
                        trace_id: &trace_id,
                        selected_position: anchor,
                        limit: EvaluationLimit::default(),
                    })
                    .unwrap_or_else(|error| {
                        panic!(
                            "V11 provider incomplete: case={id}, formula={}, word={word_index}, \
                             fairness={fairness:?}, anchor={anchor}: {error:?}",
                            formula.name
                        )
                    });
                    let (expected, actual) = match oracle {
                        Ok(outcome) => {
                            assert_ne!(independent_admitted, 0, "case={id}");
                            assert_eq!(
                                u64::try_from(outcome.fair_completions).unwrap(),
                                independent_admitted,
                                "case={id}"
                            );
                            expected_admitted += 1;
                            (
                                CaseResult::Admitted((
                                    expected_disposition(outcome.verdict),
                                    independent_admitted,
                                )),
                                CaseResult::Admitted((
                                    provider.disposition,
                                    provider.admitted_completions,
                                )),
                            )
                        }
                        Err(OracleError::EmptyFairAdmission) => {
                            assert_eq!(independent_admitted, 0, "case={id}");
                            assert_eq!(
                                provider.disposition,
                                Disposition::Inconclusive,
                                "case={id}"
                            );
                            assert_eq!(
                                provider.reason,
                                Some(ResultReason::EmptyFairAdmission),
                                "case={id}"
                            );
                            assert_eq!(provider.admitted_completions, 0, "case={id}");
                            expected_refused += 1;
                            (CaseResult::Refused, CaseResult::Refused)
                        }
                        Err(error) => panic!("V11 oracle incomplete: case={id}: {error:?}"),
                    };
                    ledger.record(id, expected, actual).unwrap_or_else(|error| {
                        panic!(
                            "V11 disagreement: {error:?}; case={id}, formula={}, \
                             word={word_index}, fairness={fairness:?}, anchor={anchor}",
                            formula.name
                        )
                    });
                }
            }
        }
    }
    assert_eq!(expected_admitted, ADMITTED_CASES);
    assert_eq!(expected_refused, EMPTY_FAIR_CASES);
    let summary = ledger.finish().unwrap();
    assert_eq!(
        summary,
        PopulationSummary {
            declared: DECLARED_CASES,
            admitted: ADMITTED_CASES,
            refused: EMPTY_FAIR_CASES,
        }
    );
    println!(
        "TL_CAMPAIGN_V11_POPULATION {}",
        serde_json::json!({
            "schema": "tl-mltl.v11-lasso-partition/v1",
            "scope": SCOPE,
            "formula_count": FORMULA_COUNT,
            "complete_words": COMPLETE_WORD_COUNT,
            "single_unknown_words": SINGLE_UNKNOWN_WORD_COUNT,
            "mixed_words": MIXED_WORD_COUNT,
            "word_count": WORD_COUNT,
            "fairness_modes": FAIRNESS_COUNT,
            "anchors": ANCHORS,
            "declared": summary.declared,
            "visited": summary.admitted,
            "refused": summary.refused,
            "failed": 0,
            "max_materialized_lasso_len": 3,
            "full_target_complete": false,
        })
    );
}
