//! TL-223: surviving-mutant kills for the finite-prefix safety route. The
//! route must apply each work limit inclusively and independently, and it must
//! enumerate every completion of the unknown prefix cells, because a refutation
//! is sound only if no completion satisfies the safety body.

#![cfg(feature = "infinite-trace")]

#[path = "support/lasso_kit.rs"]
mod kit;

use kit::*;
use tl_mltl::infinite::{
    evaluate_prefix_safety, Disposition, EvaluationLimit, InfiniteError, InfiniteResult,
    PrefixRequest, ResultReason,
};
use tl_syntax::{PartialValuation, PartialValue, TraceObservation, ValuationEntry};

fn route(
    built: &Built,
    prefix: &[Cell],
    limit: EvaluationLimit,
) -> Result<InfiniteResult, InfiniteError> {
    let word = lasso(prefix, &[[F, F]]);
    let graph_id = built.doc.content_identity().unwrap();
    evaluate_prefix_safety(&PrefixRequest {
        formula: &built.doc,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: word.propositions(),
        observations: word.prefix(),
        limit,
    })
}

fn safety() -> Built {
    // G[0,) p0: two nodes, one proposition.
    build(&T::G(Iv::Open(0), bx(T::P(0))), &[])
}

fn failed(result: Result<InfiniteResult, InfiniteError>) -> bool {
    let r = result.unwrap();
    r.disposition == Disposition::Failed && r.reason == Some(ResultReason::ResourceIncomplete)
}

// Trace: TC-159, FR-034-AC-2
#[test]
fn every_prefix_work_limit_is_inclusive_and_independent() {
    let built = safety();
    let prefix = [[Tr, Tr], [Tr, Tr]];
    let nodes = built.doc.nodes().len();
    let observations = prefix.len();
    let cells = observations * 2;
    let base = EvaluationLimit::default();
    assert!(!failed(route(&built, &prefix, base)));

    // At the limit: admitted. One below: refused.
    let at_nodes = EvaluationLimit {
        max_nodes: nodes,
        ..base
    };
    assert!(!failed(route(&built, &prefix, at_nodes)));
    let below_nodes = EvaluationLimit {
        max_nodes: nodes - 1,
        ..base
    };
    assert!(failed(route(&built, &prefix, below_nodes)));

    // The route appends one padding row, so the materialized lasso has
    // `observations + 1` positions and that is the inclusive bound.
    let at_positions = EvaluationLimit {
        max_positions: observations + 1,
        ..base
    };
    assert!(!failed(route(&built, &prefix, at_positions)));
    let below_positions = EvaluationLimit {
        max_positions: observations,
        ..base
    };
    assert!(failed(route(&built, &prefix, below_positions)));

    let at_cells = EvaluationLimit {
        max_valuation_cells: cells,
        ..base
    };
    assert!(!failed(route(&built, &prefix, at_cells)));
    let below_cells = EvaluationLimit {
        max_valuation_cells: cells - 1,
        ..base
    };
    assert!(failed(route(&built, &prefix, below_cells)));

    // An already-expired deadline refuses on its own.
    let expired = EvaluationLimit {
        deadline: Some(std::time::Instant::now()),
        ..base
    };
    assert!(failed(route(&built, &prefix, expired)));
}

// Trace: TC-157, FR-033-AC-2
#[test]
fn a_prefix_is_refuted_only_if_no_completion_satisfies_the_body() {
    // Body: neither proposition holds. Both cells of the single prefix row are
    // missing, so the completion (false, false) satisfies the body and the
    // claim is not refuted; refuting would drop a completion.
    let body = T::And(bx(not(T::P(0))), bx(not(T::P(1))));
    let built = build(&T::G(Iv::Open(0), bx(body)), &[]);
    let result = route(&built, &[[M, M]], EvaluationLimit::default()).unwrap();
    assert_eq!(result.disposition, Disposition::Inconclusive);
    assert_eq!(result.reason, Some(ResultReason::FinitePrefixUnsettled));

    // Control: with the body unsatisfiable by every completion the same prefix
    // is refuted, and the refutation counts all four completions.
    let never = build(
        &T::G(Iv::Open(0), bx(T::And(bx(T::P(0)), bx(not(T::P(0)))))),
        &[],
    );
    let refuted = route(&never, &[[M, M]], EvaluationLimit::default()).unwrap();
    assert_eq!(refuted.disposition, Disposition::Refuted);
    assert_eq!(refuted.admitted_completions, 4);
}

// Trace: TC-159, FR-034-AC-1
#[test]
fn each_prefix_identity_axis_refuses_on_its_own() {
    let built = safety();
    let word = lasso(&[[Tr, Tr], [Tr, Tr]], &[[F, F]]);
    let graph_id = built.doc.content_identity().unwrap();
    let request = |observations: &[TraceObservation], map: &str| {
        evaluate_prefix_safety(&PrefixRequest {
            formula: &built.doc,
            graph_id: &graph_id,
            proposition_map_id: map,
            propositions: word.propositions(),
            observations,
            limit: EvaluationLimit::default(),
        })
    };
    assert!(request(word.prefix(), "map").is_ok());
    // Wrong map identity for every row.
    assert_eq!(
        request(word.prefix(), "other").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
    // Wrong position on one row.
    let mut shifted = word.prefix().to_vec();
    shifted[1].position = 7;
    assert_eq!(
        request(&shifted, "map").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
    // Wrong entry count on one row.
    let mut short = word.prefix().to_vec();
    short[1] = TraceObservation {
        position: 1,
        valuation: PartialValuation::new(
            "map".into(),
            &word.propositions()[..1],
            vec![ValuationEntry {
                proposition: word.propositions()[0],
                value: PartialValue::True,
            }],
        )
        .unwrap(),
    };
    assert_eq!(
        request(&short, "map").unwrap_err(),
        InfiniteError::IdentityMismatch
    );
}
