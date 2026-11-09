//! TL-233: NFR-001 Limit Register. Every public limit is accepted exactly at
//! the limit and refused one past it with the typed error that names it.
//!
//! Limits with no public input (the recursion ceiling, the past size
//! constants) are tested in their defining modules. Under Miri the heavy
//! owner-wire limits run at a caller-lowered value; natively every owner-wire
//! limit runs at its production value.

use serde_json::Value;
use tl_mltl::wire::common::{produce, read_expected};
use tl_mltl::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
use tl_mltl::{
    evaluate_closed_at_with_stats, evaluate_past_with_stats, map_to_c2po, ClockBinding,
    EvaluationError, EvaluationLimits, MappingError, MappingSourceIdentity, MappingSourceState,
    PastEvaluationError, PastEvaluationLimits, PastEvaluationRelationInput,
    PositionHistoryDocument, PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn resource(result: Result<(), OwnerReadError>, field: &str) {
    let error = result.expect_err("one past the limit is refused");
    assert_eq!(error.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(error.field(), field);
}

fn pick(production: usize, lowered: usize) -> usize {
    if cfg!(miri) {
        lowered
    } else {
        production
    }
}

// ---------------------------------------------------------------------------
// OwnerLimits: semantic usage fields, driven through the public `produce`.
// ---------------------------------------------------------------------------

fn semantic(limits: OwnerLimits, usage: OwnerUsage) -> Result<(), OwnerReadError> {
    produce(0_u8, usage, limits).map(|_| ())
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn every_semantic_owner_limit_accepts_the_limit_and_refuses_one_past() {
    let max = OwnerLimits::owner_max();
    type Case = (
        &'static str,
        fn(OwnerLimits, u128) -> OwnerLimits,
        fn(OwnerUsage, u128) -> OwnerUsage,
        u128,
    );
    #[allow(clippy::cast_possible_truncation)]
    let cases: [Case; 8] = [
        (
            "formulaNodes",
            |l, v| OwnerLimits {
                max_formula_nodes: v as usize,
                ..l
            },
            |u, v| OwnerUsage {
                formula_nodes: v as usize,
                ..u
            },
            max.max_formula_nodes as u128,
        ),
        (
            "formulaDepth",
            |l, v| OwnerLimits {
                max_formula_depth: v as usize,
                ..l
            },
            |u, v| OwnerUsage {
                formula_depth: v as usize,
                ..u
            },
            max.max_formula_depth as u128,
        ),
        (
            "positions",
            |l, v| OwnerLimits {
                max_positions: v as usize,
                ..l
            },
            |u, v| OwnerUsage {
                positions: v as usize,
                ..u
            },
            max.max_positions as u128,
        ),
        (
            "propositions",
            |l, v| OwnerLimits {
                max_propositions: v as usize,
                ..l
            },
            |u, v| OwnerUsage {
                propositions: v as usize,
                ..u
            },
            max.max_propositions as u128,
        ),
        (
            "support",
            |l, v| OwnerLimits {
                max_support: v as usize,
                ..l
            },
            |u, v| OwnerUsage {
                support: v as usize,
                ..u
            },
            max.max_support as u128,
        ),
        (
            "historySpan",
            |l, v| OwnerLimits {
                max_history_span: v as u64,
                ..l
            },
            |u, v| OwnerUsage {
                history_span: v as u64,
                ..u
            },
            u128::from(max.max_history_span),
        ),
        (
            "evaluationSteps",
            |l, v| OwnerLimits {
                max_evaluation_steps: v as u64,
                ..l
            },
            |u, v| OwnerUsage {
                evaluation_steps: v as u64,
                ..u
            },
            u128::from(max.max_evaluation_steps),
        ),
        (
            "recursionDepth",
            |l, v| OwnerLimits {
                max_recursion_depth: v as u32,
                ..l
            },
            |u, v| OwnerUsage {
                recursion_depth: v as u32,
                ..u
            },
            u128::from(max.max_recursion_depth),
        ),
    ];
    for (field, set_limit, set_usage, production) in cases {
        // Production value: the owner maximum itself.
        assert_eq!(
            semantic(max, set_usage(OwnerUsage::default(), production)),
            Ok(()),
            "{field} at the owner maximum"
        );
        resource(
            semantic(max, set_usage(OwnerUsage::default(), production + 1)),
            field,
        );
        // OWNER_MAX clamp: a caller ceiling above the owner maximum is
        // lowered to it, never raised past it.
        let unbounded = set_limit(max, production * 2);
        assert_eq!(
            semantic(unbounded, set_usage(OwnerUsage::default(), production)),
            Ok(()),
            "{field} clamped, at the maximum"
        );
        resource(
            semantic(unbounded, set_usage(OwnerUsage::default(), production + 1)),
            field,
        );
        // A lowered caller ceiling is honored exactly.
        let lowered = set_limit(max, 3);
        assert_eq!(
            semantic(lowered, set_usage(OwnerUsage::default(), 3)),
            Ok(()),
            "{field} lowered"
        );
        resource(
            semantic(lowered, set_usage(OwnerUsage::default(), 4)),
            field,
        );
    }
}

// ---------------------------------------------------------------------------
// OwnerLimits: lexical fields, driven through `produce` / `read_expected`.
// ---------------------------------------------------------------------------

fn produced(value: &Value, limits: OwnerLimits) -> Result<(), OwnerReadError> {
    produce(value.clone(), OwnerUsage::default(), limits).map(|_| ())
}

/// Nests `depth` arrays: `[[...[]...]]`.
fn nested(depth: usize) -> Value {
    (1..depth).fold(Value::Array(vec![]), |inner, _| Value::Array(vec![inner]))
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn owner_depth_limit_accepts_the_limit_and_refuses_one_past() {
    let limit = OwnerLimits::owner_max().max_depth;
    assert_eq!(limit, 64);
    assert_eq!(produced(&nested(limit), OwnerLimits::owner_max()), Ok(()));
    resource(
        produced(&nested(limit + 1), OwnerLimits::owner_max()),
        "depth",
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn owner_string_limit_accepts_the_limit_and_refuses_one_past() {
    let limit = pick(OwnerLimits::owner_max().max_string_bytes, 4_096);
    let limits = OwnerLimits {
        max_string_bytes: limit,
        ..OwnerLimits::owner_max()
    };
    let string = |len: usize| Value::Array(vec![Value::String("a".repeat(len))]);
    assert_eq!(produced(&string(limit), limits), Ok(()));
    resource(produced(&string(limit + 1), limits), "stringBytes");
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn owner_visited_field_limit_accepts_the_limit_and_refuses_one_past() {
    let limit = pick(OwnerLimits::owner_max().max_visited_fields, 4_096);
    let limits = OwnerLimits {
        max_visited_fields: limit,
        ..OwnerLimits::owner_max()
    };
    let array = |len: usize| Value::Array(vec![Value::from(0); len]);
    assert_eq!(produced(&array(limit), limits), Ok(()));
    resource(produced(&array(limit + 1), limits), "visitedFields");
}

/// Canonical array of `u64` whose serialized length is exactly `len` bytes.
fn numbers_of_exact_length(len: usize) -> Vec<u64> {
    // `[` + digits + `,` separators + `]`: len = 1 + digits + (count - 1) + 1,
    // so digits = len - count - 1, with 1..=20 digits per number.
    let count = (len - 1).div_ceil(21);
    let mut extra = len - count - 1 - count;
    let mut numbers = Vec::with_capacity(count);
    for _ in 0..count {
        let more = extra.min(19);
        extra -= more;
        numbers.push(10_u64.pow(u32::try_from(more).unwrap()));
    }
    assert_eq!(extra, 0);
    assert_eq!(serde_json::to_vec(&numbers).unwrap().len(), len);
    numbers
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn owner_output_byte_limit_accepts_the_limit_and_refuses_one_past() {
    let limit = pick(OwnerLimits::owner_max().max_output_bytes, 4_096);
    let limits = OwnerLimits {
        max_output_bytes: limit,
        max_visited_fields: usize::MAX,
        ..OwnerLimits::owner_max()
    };
    let at = serde_json::to_value(numbers_of_exact_length(limit)).unwrap();
    assert_eq!(produced(&at, limits), Ok(()));
    let past = serde_json::to_value(numbers_of_exact_length(limit + 1)).unwrap();
    resource(produced(&past, limits), "outputBytes");
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn owner_input_byte_limit_accepts_the_limit_and_refuses_one_past() {
    let limit = pick(OwnerLimits::owner_max().max_input_bytes, 4_096);
    let limits = OwnerLimits {
        max_input_bytes: limit,
        max_output_bytes: usize::MAX,
        ..OwnerLimits::owner_max()
    };
    let read = |len: usize| {
        let numbers = numbers_of_exact_length(len);
        let bytes = serde_json::to_vec(&numbers).unwrap();
        read_expected(&bytes, &numbers, limits, |_, _| Ok(OwnerUsage::default())).map(|_| ())
    };
    assert_eq!(read(limit), Ok(()));
    resource(read(limit + 1), "inputBytes");
}

// ---------------------------------------------------------------------------
// EvaluationLimits (future).
// ---------------------------------------------------------------------------

fn chain(length: u32, kind: impl Fn(NodeId) -> NodeKind) -> Vec<Node> {
    let mut nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    for index in 0..length {
        nodes.push(Node::new(kind(NodeId(index))));
    }
    nodes
}

fn closed(nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn future_eval(
    nodes: &[Node],
    limits: EvaluationLimits,
) -> (
    Result<tl_mltl::EvaluationReport, EvaluationError>,
    tl_mltl::EvaluationStats,
) {
    let trace = vec![vec![PropositionId(0)]; 8];
    evaluate_closed_at_with_stats(closed(nodes), "f", &trace, "t", 0, limits)
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn future_work_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = chain(3, |operand| NodeKind::Not { operand });
    let (ok, stats) = future_eval(&nodes, EvaluationLimits::default());
    ok.unwrap();
    let work = stats.node_evaluations;
    let at = EvaluationLimits {
        max_node_evaluations: work,
        ..EvaluationLimits::default()
    };
    future_eval(&nodes, at).0.unwrap();
    let past = EvaluationLimits {
        max_node_evaluations: work - 1,
        ..EvaluationLimits::default()
    };
    assert_eq!(
        future_eval(&nodes, past).0,
        Err(EvaluationError::WorkLimitExceeded { limit: work - 1 })
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn future_span_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = chain(1, |operand| NodeKind::Globally {
        interval: Interval::new(0, 4).unwrap(),
        operand,
    });
    let at = EvaluationLimits {
        max_temporal_span: 5,
        ..EvaluationLimits::default()
    };
    future_eval(&nodes, at).0.unwrap();
    let past = EvaluationLimits {
        max_temporal_span: 4,
        ..EvaluationLimits::default()
    };
    assert_eq!(
        future_eval(&nodes, past).0,
        Err(EvaluationError::TemporalSpanExceeded {
            requested: 5,
            limit: 4
        })
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn future_recursion_limit_accepts_the_limit_and_refuses_one_past() {
    // Four nested negations over a proposition reach depth 4.
    let nodes = chain(4, |operand| NodeKind::Not { operand });
    let at = EvaluationLimits {
        max_recursion_depth: 4,
        ..EvaluationLimits::default()
    };
    future_eval(&nodes, at).0.unwrap();
    let past = EvaluationLimits {
        max_recursion_depth: 3,
        ..EvaluationLimits::default()
    };
    assert_eq!(
        future_eval(&nodes, past).0,
        Err(EvaluationError::RecursionDepthExceeded { limit: 3 })
    );
}

// ---------------------------------------------------------------------------
// PastEvaluationLimits.
// ---------------------------------------------------------------------------

fn history(positions: u64) -> PositionHistoryDocument {
    let observations = (0..positions)
        .map(|position| PositionObservation::new(position, vec![PropositionId(0)], None))
        .collect();
    PositionHistoryDocument::new(
        "h",
        1,
        0,
        positions - 1,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap()
}

fn past_nodes(length: u32, kind: impl Fn(NodeId) -> NodeKind) -> Vec<Node> {
    chain(length, kind)
}

fn past_eval(
    nodes: &[Node],
    history: &PositionHistoryDocument,
    limits: PastEvaluationLimits,
) -> (
    Result<tl_mltl::PastEvaluationReport, PastEvaluationError>,
    tl_mltl::PastEvaluationStats,
) {
    let formula = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap();
    let anchor = history.through_position();
    evaluate_past_with_stats(
        formula,
        "f",
        history,
        anchor,
        "m",
        1,
        PastEvaluationRelationInput::Original,
        limits,
    )
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn past_step_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = past_nodes(1, |operand| NodeKind::Once {
        interval: Interval::new(0, 3).unwrap(),
        operand,
    });
    let history = history(8);
    let (ok, stats) = past_eval(&nodes, &history, PastEvaluationLimits::default());
    ok.unwrap();
    let steps = stats.steps;
    let at = PastEvaluationLimits {
        max_steps: steps,
        ..PastEvaluationLimits::default()
    };
    past_eval(&nodes, &history, at).0.unwrap();
    let past = PastEvaluationLimits {
        max_steps: steps - 1,
        ..PastEvaluationLimits::default()
    };
    assert_eq!(
        past_eval(&nodes, &history, past).0,
        Err(PastEvaluationError::StepLimitExceeded { limit: steps - 1 })
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn past_span_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = past_nodes(1, |operand| NodeKind::Once {
        interval: Interval::new(0, 4).unwrap(),
        operand,
    });
    let history = history(8);
    let at = PastEvaluationLimits {
        max_temporal_span: 5,
        ..PastEvaluationLimits::default()
    };
    past_eval(&nodes, &history, at).0.unwrap();
    let past = PastEvaluationLimits {
        max_temporal_span: 4,
        ..PastEvaluationLimits::default()
    };
    assert_eq!(
        past_eval(&nodes, &history, past).0,
        Err(PastEvaluationError::TemporalSpanExceeded {
            requested: 5,
            limit: 4
        })
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn past_recursion_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = past_nodes(4, |operand| NodeKind::Not { operand });
    let history = history(2);
    let at = PastEvaluationLimits {
        max_recursion_depth: 4,
        ..PastEvaluationLimits::default()
    };
    past_eval(&nodes, &history, at).0.unwrap();
    let past = PastEvaluationLimits {
        max_recursion_depth: 3,
        ..PastEvaluationLimits::default()
    };
    assert_eq!(
        past_eval(&nodes, &history, past).0,
        Err(PastEvaluationError::RecursionDepthExceeded { limit: 3 })
    );
}

// Trace: TC-175, NFR-001-AC-3
#[test]
fn past_input_position_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = past_nodes(1, |operand| NodeKind::Not { operand });
    let history = history(8);
    let at = PastEvaluationLimits {
        max_input_positions: 8,
        ..PastEvaluationLimits::default()
    };
    past_eval(&nodes, &history, at).0.unwrap();
    let past = PastEvaluationLimits {
        max_input_positions: 7,
        ..PastEvaluationLimits::default()
    };
    assert_eq!(
        past_eval(&nodes, &history, past).0,
        Err(PastEvaluationError::InputPositionLimitExceeded {
            requested: 8,
            limit: 7
        })
    );
}

// ---------------------------------------------------------------------------
// Mapping render work limit.
// ---------------------------------------------------------------------------

// Trace: TC-175, NFR-001-AC-3
#[test]
fn mapping_work_limit_accepts_the_limit_and_refuses_one_past() {
    let nodes = chain(2, |operand| NodeKind::Not { operand });
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(2), &nodes).unwrap();
    let map = |limit: u64| {
        map_to_c2po(
            formula,
            "f",
            b"{}",
            MappingSourceIdentity {
                revision: "r".to_owned(),
                state: MappingSourceState::Clean,
            },
            None,
            limit,
        )
    };
    // Three nodes are visited, one work unit each.
    map(3).unwrap();
    assert_eq!(
        map(2).unwrap_err(),
        MappingError::WorkLimitExceeded { limit: 2 }
    );
}
