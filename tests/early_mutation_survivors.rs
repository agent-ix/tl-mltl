//! TL-371: behavior tests for the surfaces the first full cargo-mutants run showed were not
//! asserted: error rendering, per-limit and per-field refusals, the comparison rule and the
//! campaign inventory checks. Each test pins an exact value or isolates one operand of a
//! condition. The file name sorts first so a mutant that hangs a later suite is still killed here.

use tl_mltl::{
    compare_external, ComparisonStatus, ContextualBindingError, ContextualComparisonError,
    ContextualExternalVerdict, EvaluationLimits, ExternalStatus, ExternalVerdict, ToolIdentity,
    TruthValue,
};
use tl_syntax::{Formula, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

// Trace: FR-005
#[test]
fn contextual_errors_render_their_identities() {
    assert_eq!(
        ContextualBindingError::MissingProposition {
            proposition: PropositionId(7)
        }
        .to_string(),
        "missing signal binding for proposition 7"
    );
    assert_eq!(
        ContextualBindingError::Refused("bad".into()).to_string(),
        "signal binding refused: bad"
    );
    assert_eq!(
        ContextualComparisonError("boom".into()).to_string(),
        "contextual comparison serialization failed: boom"
    );
}

#[test]
fn a_context_that_is_neither_object_nor_null_names_what_it_expects() {
    let error = serde_json::from_str::<ContextualExternalVerdict>(r#"{"requirementContext": 5}"#)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("an explicit requirement context object or null"),
        "{error}"
    );
}

fn reference(verdict_time: u64) -> tl_mltl::EvaluationReport {
    let nodes = [Node {
        kind: NodeKind::Proposition {
            proposition: PropositionId(0),
        },
        span: None,
    }];
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    tl_mltl::evaluate_prefix_at(
        formula,
        "f",
        &[vec![PropositionId(0)], vec![PropositionId(0)]],
        "t",
        true,
        verdict_time,
        EvaluationLimits::default(),
    )
    .unwrap()
}

fn external(value: bool, time: u64) -> ExternalVerdict {
    ExternalVerdict {
        schema_version: "tl-mltl.external-verdict/v1".to_owned(),
        tool: ToolIdentity {
            name: "x".into(),
            version: "1".into(),
        },
        formula_id: "f".into(),
        trace_id: "t".into(),
        status: ExternalStatus::Conclusive,
        value: Some(value),
        verdict_time: Some(time),
        detail: None,
    }
}

// Trace: FR-005
#[test]
fn agreement_needs_both_the_truth_value_and_the_verdict_time() {
    let reference = reference(0);
    assert_eq!(reference.verdict, TruthValue::True);
    let status = |value, time| compare_external(&reference, external(value, time)).status;
    assert_eq!(status(true, 0), ComparisonStatus::Agreement);
    assert_eq!(status(true, 1), ComparisonStatus::Mismatch);
    assert_eq!(status(false, 0), ComparisonStatus::Mismatch);
    assert_eq!(status(false, 1), ComparisonStatus::Mismatch);
}
