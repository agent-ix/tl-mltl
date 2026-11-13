//! TL-371: behavior tests for the infinite-trace provider's refusals, which the first full
//! cargo-mutants run showed were not asserted one condition at a time. Each test violates exactly
//! one operand of a refusal and expects that refusal; a mutant that joins the operands with `&&`
//! would then accept the input.

#![cfg(feature = "infinite-trace")]

#[path = "support/lasso_kit.rs"]
mod kit;

use kit::*;
use tl_mltl::infinite::{
    evaluate_lasso, evaluate_model, evaluate_prefix_safety, Disposition, EvaluationLimit,
    EvidenceClosure, InfiniteError, LassoRequest, PrefixRequest, ResultReason,
};
use tl_syntax::PropositionId;
use tl_syntax::{PartialValuation, PartialValue, TraceObservation, ValuationEntry};

fn safety() -> Built {
    build(&T::G(Iv::Open(0), bx(T::P(0))), &[])
}

#[test]
fn infinite_errors_render_their_variant() {
    assert_eq!(
        InfiniteError::InvalidLasso.to_string(),
        "infinite evaluation refused: InvalidLasso"
    );
    assert_eq!(
        InfiniteError::ResourceIncomplete.to_string(),
        "infinite evaluation refused: ResourceIncomplete"
    );
}

#[test]
fn a_model_needs_both_a_model_identity_and_a_map_identity() {
    let built = safety();
    let graph = built.doc.content_identity().unwrap();
    let ok = evaluate_model(&built.doc, &graph, "model", "map").unwrap();
    assert_eq!(ok.disposition, Disposition::Unsupported);
    assert_eq!(
        evaluate_model(&built.doc, &graph, "", "map").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
    assert_eq!(
        evaluate_model(&built.doc, &graph, "model", "").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
    assert_eq!(
        evaluate_model(&built.doc, "other", "model", "map").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
}

fn lasso_result(built: &Built, limit: EvaluationLimit) -> bool {
    let word = lasso(&[[Tr, Tr]], &[[Tr, Tr]]);
    let result = provider_with(built, &word, 0, EvidenceClosure::Closed, limit);
    result.disposition == Disposition::Failed
        && result.reason == Some(ResultReason::ResourceIncomplete)
}

#[test]
fn each_lasso_work_limit_refuses_on_its_own() {
    let built = safety();
    let nodes = built.doc.nodes().len();
    let base = EvaluationLimit::default();
    assert!(!lasso_result(&built, base));
    assert!(lasso_result(
        &built,
        EvaluationLimit {
            deadline: Some(std::time::Instant::now()),
            ..base
        }
    ));
    assert!(lasso_result(
        &built,
        EvaluationLimit {
            max_nodes: nodes - 1,
            ..base
        }
    ));
    assert!(lasso_result(
        &built,
        EvaluationLimit {
            max_positions: 1,
            ..base
        }
    ));
    assert!(lasso_result(
        &built,
        EvaluationLimit {
            max_valuation_cells: 3,
            ..base
        }
    ));
}

#[test]
fn a_lasso_request_with_one_wrong_identity_is_refused() {
    let built = safety();
    let word = lasso(&[[Tr, Tr]], &[[Tr, Tr]]);
    let graph = built.doc.content_identity().unwrap();
    let trace = word.content_identity().unwrap();
    let call = |graph_id: &str, trace_id: &str| {
        evaluate_lasso(&LassoRequest {
            formula: &built.doc,
            trace: &word,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id,
            trace_id,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        })
    };
    assert!(call(&graph, &trace).is_ok());
    assert_eq!(
        call("x", &trace).unwrap_err(),
        InfiniteError::IdentityMismatch
    );
    assert_eq!(
        call(&graph, "x").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
}

fn observation(position: u32, map: &str, props: &[PropositionId]) -> TraceObservation {
    TraceObservation {
        position,
        valuation: PartialValuation::new(
            map.into(),
            props,
            props
                .iter()
                .map(|p| ValuationEntry {
                    proposition: *p,
                    value: Tr,
                })
                .collect(),
        )
        .unwrap(),
    }
}

fn shape(
    built: &Built,
    graph_id: &str,
    map: &str,
    propositions: &[PropositionId],
    observations: &[TraceObservation],
) -> Result<(), InfiniteError> {
    PrefixRequest {
        formula: &built.doc,
        graph_id,
        proposition_map_id: map,
        propositions,
        observations,
        limit: EvaluationLimit::default(),
    }
    .validate_shape()
}

#[test]
fn prefix_shape_refuses_each_fault_on_its_own() {
    let built = safety();
    let graph = built.doc.content_identity().unwrap();
    let p = [PropositionId(0), PropositionId(1)];
    let good = [observation(0, "map", &p)];
    assert_eq!(shape(&built, &graph, "map", &p, &good), Ok(()));
    let mismatch = Err(InfiniteError::IdentityMismatch);
    assert_eq!(shape(&built, "x", "map", &p, &good), mismatch);
    assert_eq!(shape(&built, &graph, "", &p, &good), mismatch);
    assert_eq!(
        shape(&built, &graph, "map", &p, &[observation(1, "map", &p)]),
        mismatch
    );
    assert_eq!(shape(&built, &graph, "other", &p, &good), mismatch);
    assert_eq!(shape(&built, &graph, "map", &p[..1], &good), mismatch);
    let swapped = [PropositionId(1), PropositionId(0)];
    assert_eq!(shape(&built, &graph, "map", &swapped, &good), mismatch);
}

#[test]
fn a_prefix_identity_is_a_digest_of_the_map_and_rows() {
    let built = safety();
    let graph = built.doc.content_identity().unwrap();
    let p = [PropositionId(0), PropositionId(1)];
    let rows = [observation(0, "map", &p)];
    let identity = |map: &str, rows: &[TraceObservation]| {
        PrefixRequest {
            formula: &built.doc,
            graph_id: &graph,
            proposition_map_id: map,
            propositions: &p,
            observations: rows,
            limit: EvaluationLimit::default(),
        }
        .content_identity()
        .unwrap()
    };
    let a = identity("map", &rows);
    assert_eq!(a.len(), 64);
    assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(a, identity("map", &rows));
    assert_ne!(a, identity("other", &rows));
    assert_ne!(a, identity("map", &[]));
    let _ = evaluate_prefix_safety;
}

use tl_mltl::infinite::{
    DetailedSettlementError, InfiniteProvider, ProviderRegistry, ProviderRequest,
};
use tl_syntax::{LivenessSubject, LivenessSubjectKind};

fn settle(
    request: ProviderRequest<'_>,
    formula: &tl_syntax::InfiniteFormulaDocument,
    kind: LivenessSubjectKind,
    identity: &str,
) -> Result<Disposition, DetailedSettlementError> {
    let provider = InfiniteProvider { request };
    let mut registry = ProviderRegistry::default();
    registry.register(&provider).unwrap();
    registry
        .settle_detailed(formula, LivenessSubject { kind, identity })
        .map(|result| result.disposition)
}

#[test]
fn a_provider_serves_only_its_own_formula_and_subject() {
    let built = safety();
    let other = build(&T::F(Iv::Open(0), bx(T::P(0))), &[]);
    let word = lasso(&[[Tr, Tr]], &[[Tr, Tr]]);
    let graph = built.doc.content_identity().unwrap();
    let trace = word.content_identity().unwrap();
    let lasso_request = || {
        ProviderRequest::Lasso(LassoRequest {
            formula: &built.doc,
            trace: &word,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph,
            trace_id: &trace,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        })
    };
    let refused = Err(DetailedSettlementError::InvalidRequest(
        InfiniteError::IdentityMismatch,
    ));
    let lasso = LivenessSubjectKind::LassoTrace;
    assert!(settle(lasso_request(), &built.doc, lasso, &trace).is_ok());
    assert_eq!(settle(lasso_request(), &built.doc, lasso, "other"), refused);
    assert_eq!(settle(lasso_request(), &other.doc, lasso, &trace), refused);
    let model = LivenessSubjectKind::Model;
    assert_eq!(
        settle(lasso_request(), &built.doc, model, "m"),
        Ok(Disposition::Unsupported)
    );
    assert_eq!(settle(lasso_request(), &other.doc, model, "m"), refused);
    // A lasso provider does not serve a prefix subject.
    let prefix = LivenessSubjectKind::FinitePrefix;
    assert_eq!(settle(lasso_request(), &built.doc, prefix, &trace), refused);

    let p = [PropositionId(0), PropositionId(1)];
    let rows = [observation(0, "map", &p)];
    let prefix_request = || {
        ProviderRequest::FinitePrefix(PrefixRequest {
            formula: &built.doc,
            graph_id: &graph,
            proposition_map_id: "map",
            propositions: &p,
            observations: &rows,
            limit: EvaluationLimit::default(),
        })
    };
    let prefix_id = PrefixRequest {
        formula: &built.doc,
        graph_id: &graph,
        proposition_map_id: "map",
        propositions: &p,
        observations: &rows,
        limit: EvaluationLimit::default(),
    }
    .content_identity()
    .unwrap();
    assert!(settle(prefix_request(), &built.doc, prefix, &prefix_id).is_ok());
    assert_eq!(
        settle(prefix_request(), &built.doc, prefix, "other"),
        refused
    );
    assert_eq!(
        settle(prefix_request(), &other.doc, prefix, &prefix_id),
        refused
    );
    assert_eq!(
        settle(prefix_request(), &built.doc, model, "m"),
        Ok(Disposition::Unsupported)
    );
    assert_eq!(settle(prefix_request(), &other.doc, model, "m"), refused);
    assert_eq!(settle(prefix_request(), &built.doc, lasso, &trace), refused);
}

fn prefix_steps(built: &Built, row: [PartialValue; 2]) -> (Disposition, u64, u64) {
    let word = lasso(&[row], &[[F, F]]);
    let graph = built.doc.content_identity().unwrap();
    let result = evaluate_prefix_safety(&PrefixRequest {
        formula: &built.doc,
        graph_id: &graph,
        proposition_map_id: "map",
        propositions: word.propositions(),
        observations: word.prefix(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    (
        result.disposition,
        result.admitted_completions,
        result.evaluation_steps,
    )
}

/// Completions are tried from all-false upward and the search stops at the first one that
/// satisfies the body, so descending order would stop after the first (all-true) completion.
#[test]
fn prefix_completions_are_tried_in_ascending_order() {
    let body = T::And(bx(T::P(0)), bx(T::P(1)));
    let built = build(&T::G(Iv::Open(0), bx(body)), &[]);
    let (known, _, one) = prefix_steps(&built, [Tr, Tr]);
    assert_eq!(known, Disposition::Inconclusive);
    assert!(one > 0);
    // Only the last of the four completions (both true) satisfies the body.
    let (unknown, completions, steps) = prefix_steps(&built, [M, M]);
    assert_eq!(unknown, Disposition::Inconclusive);
    assert_eq!(completions, 4);
    assert!(
        steps > 2 * one,
        "all four completions are tried before the satisfying one"
    );
}
