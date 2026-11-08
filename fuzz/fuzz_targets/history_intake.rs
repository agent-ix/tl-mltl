#![no_main]
#![forbid(unsafe_code)]

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use tl_mltl::past::history::{derive, read};
use tl_mltl::wire::OwnerLimits;
use tl_mltl::{
    evaluate_past, ClockBinding, PastEvaluationLimits, PastEvaluationRelationInput,
    PositionHistoryDocument, PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const MAX_INPUT: usize = 16 * 1024;

fn formula_nodes() -> Vec<Node> {
    let p = |n| {
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(n),
        })
    };
    let span = Interval::new(0, 3).unwrap();
    vec![
        p(0),
        p(1),
        Node::new(NodeKind::Once {
            interval: span,
            operand: NodeId(0),
        }),
        Node::new(NodeKind::Since {
            interval: span,
            left: NodeId(0),
            right: NodeId(1),
        }),
        Node::new(NodeKind::StrongPrevious { operand: NodeId(3) }),
        Node::new(NodeKind::Or {
            left: NodeId(2),
            right: NodeId(4),
        }),
    ]
}

// A history that was admitted must be usable by the evaluator at any anchor
// without panicking; an out-of-range anchor is a typed error.
fn evaluate(history: &PositionHistoryDocument) {
    let nodes = formula_nodes();
    let formula = Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(5), &nodes)
        .expect("fixed formula is valid");
    for anchor in [0, 1, 7, u64::MAX] {
        let _ = evaluate_past(
            formula,
            "f",
            history,
            anchor,
            "m",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        );
    }
}

fn admit(history: &PositionHistoryDocument) {
    let tight = OwnerLimits {
        max_positions: 4,
        max_propositions: 4,
        max_history_span: 8,
        ..OwnerLimits::owner_max()
    };
    for limits in [OwnerLimits::owner_max(), tight] {
        if let Ok(owner) = derive(history, limits) {
            let validated = read(owner.bytes(), history, limits)
                .expect("derived canonical bytes re-read under the same limits");
            assert_eq!(validated.document(), history);
            assert_eq!(validated.bytes(), owner.bytes());
        }
    }
    evaluate(history);
}

// Structure-aware intake: observations may be out of order, duplicated,
// sparse, huge in position or carry unsorted proposition lists; the
// constructor must answer with a typed error or a value the owner boundary
// and evaluator accept.
fn structured(u: &mut Unstructured<'_>) -> Option<()> {
    let position = |u: &mut Unstructured<'_>| -> Option<u64> {
        Some(match u.int_in_range(0..=5_u8).ok()? {
            0 => u64::MAX - u64::from(u.int_in_range(0..=2_u8).ok()?),
            1 => u64::from(u.arbitrary::<u32>().ok()?),
            _ => u64::from(u.int_in_range(0..=10_u8).ok()?),
        })
    };
    let origin = position(u)?;
    let through = position(u)?;
    let count = u.int_in_range(0..=16_usize).ok()?;
    let mut observations = Vec::with_capacity(count);
    for _ in 0..count {
        let mut present = Vec::new();
        for _ in 0..u.int_in_range(0..=4_usize).ok()? {
            present.push(PropositionId(u.int_in_range(0..=4_u32).ok()?));
        }
        observations.push(PositionObservation::new(position(u)?, present, None));
    }
    let clock = u
        .arbitrary::<bool>()
        .ok()?
        .then_some(ClockBinding::EventPosition);
    let revision = u64::from(u.int_in_range(0..=3_u8).ok()?);
    if let Ok(history) =
        PositionHistoryDocument::new("h", revision, origin, through, clock, observations.clone())
    {
        admit(&history);
        // Appending a correction revision is the same intake path.
        let more = position(u)?;
        if let Ok(next) = history.corrected(revision + 1, more, observations) {
            admit(&next);
        }
    }
    Some(())
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT {
        return;
    }
    if let Ok(history) = serde_json::from_slice::<PositionHistoryDocument>(data) {
        admit(&history);
    }
    let _ = structured(&mut Unstructured::new(data));
});
