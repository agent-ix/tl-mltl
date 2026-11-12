//! TL-371: behavior tests for exact arithmetic, clock validation, history ordering, accessors,
//! error rendering and deserializer messages in the past profile, which the first full
//! cargo-mutants run showed were not asserted.

use tl_mltl::{
    analyze_required_history, fixed_sample_instant, ClockBinding, ClockError, ClockSample,
    ExactNumber, ExactNumberError, HistoryError, HistoryRequirementError, PositionHistoryDocument,
    PositionObservation, UnsupportedClockKind,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn exact(numerator: i64, denominator: u64) -> ExactNumber {
    ExactNumber::new(numerator, denominator).unwrap()
}

/// Runs `work` on a thread and fails if it does not finish: a mutated arithmetic loop would spin
/// forever, and this turns that hang into an immediate failure.
fn within_seconds<T: Send + 'static>(seconds: u64, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(seconds))
        .expect("the computation must terminate")
}

// Trace: TC-176
#[test]
fn exact_numbers_expose_their_parts_and_normalize_sums() {
    within_seconds(20, exact_number_checks);
}

fn exact_number_checks() {
    let value = exact(-7, 3);
    assert_eq!((value.numerator(), value.denominator()), (-7, 3));
    // 1/4 + 1 * 1/4 = 1/2: the sum 2/4 is reduced by its common divisor.
    let half = fixed_sample_instant(exact(1, 4), exact(1, 4), 1).unwrap();
    assert_eq!((half.numerator(), half.denominator()), (1, 2));
    // 1/6 + 3 * 1/4 = 11/12, with unequal denominators scaled to a common one.
    let sum = fixed_sample_instant(exact(1, 6), exact(1, 4), 3).unwrap();
    assert_eq!((sum.numerator(), sum.denominator()), (11, 12));
    // A negative epoch reduces too: -1/2 + 2 * 3/2 = 5/2.
    let shifted = fixed_sample_instant(exact(-1, 2), exact(3, 2), 2).unwrap();
    assert_eq!((shifted.numerator(), shifted.denominator()), (5, 2));
}

// Trace: TC-176
#[test]
fn error_values_render_a_nonempty_message() {
    let messages = [
        ExactNumberError::ZeroDenominator.to_string(),
        ExactNumberError::ArithmeticOverflow.to_string(),
        ClockError::NonPositivePeriod.to_string(),
        ClockError::SampleUnitMismatch { position: 4 }.to_string(),
        HistoryError::EmptyHistory.to_string(),
        HistoryError::RevisionZero.to_string(),
        HistoryRequirementError::InvalidFormulaIdentity.to_string(),
    ];
    for message in messages {
        assert!(!message.trim().is_empty());
    }
    assert!(ClockError::SampleUnitMismatch { position: 4 }
        .to_string()
        .contains('4'));
}

fn fixed(period: ExactNumber, unit: &str) -> ClockBinding {
    ClockBinding::FixedSample {
        epoch: exact(0, 1),
        period,
        unit: unit.to_owned(),
    }
}

fn sampled(position: u64, instant: ExactNumber, unit: &str) -> PositionObservation {
    PositionObservation::new(
        position,
        vec![PropositionId(0)],
        Some(ClockSample {
            instant,
            unit: unit.to_owned(),
        }),
    )
}

fn document(
    clock: ClockBinding,
    observations: Vec<PositionObservation>,
) -> Result<PositionHistoryDocument, HistoryError> {
    let through = u64::try_from(observations.len() - 1).unwrap();
    PositionHistoryDocument::new("h", 1, 0, through, Some(clock), observations)
}

// Trace: TC-176
#[test]
fn clock_bindings_and_samples_are_validated() {
    let good = || vec![sampled(0, exact(0, 1), "s"), sampled(1, exact(1, 1), "s")];
    assert!(document(fixed(exact(1, 1), "s"), good()).is_ok());
    // A period of exactly one unit is positive; zero is not.
    assert_eq!(
        document(fixed(exact(0, 1), "s"), good()).unwrap_err(),
        HistoryError::Clock(ClockError::NonPositivePeriod)
    );
    assert!(matches!(
        document(
            ClockBinding::Unsupported {
                clock_kind: UnsupportedClockKind::Dense
            },
            good()
        )
        .unwrap_err(),
        HistoryError::Clock(ClockError::Unsupported { .. })
    ));
    // The sample must carry the bound unit and the exact instant.
    assert!(matches!(
        document(
            fixed(exact(1, 1), "s"),
            vec![sampled(0, exact(0, 1), "ms"), sampled(1, exact(1, 1), "ms")]
        )
        .unwrap_err(),
        HistoryError::Clock(ClockError::SampleUnitMismatch { position: 0 })
    ));
    assert!(matches!(
        document(
            fixed(exact(1, 1), "s"),
            vec![sampled(0, exact(0, 1), "s"), sampled(1, exact(2, 1), "s")]
        )
        .unwrap_err(),
        HistoryError::Clock(ClockError::SampleInstantMismatch { position: 1, .. })
    ));
    // An event-position clock admits no sample.
    assert!(matches!(
        document(
            ClockBinding::EventPosition,
            vec![sampled(0, exact(0, 1), "s")]
        )
        .unwrap_err(),
        HistoryError::Clock(ClockError::UnexpectedSample { position: 0 })
    ));
}

fn event(positions: &[u64]) -> Result<PositionHistoryDocument, HistoryError> {
    PositionHistoryDocument::new(
        "h",
        1,
        0,
        *positions.last().unwrap(),
        Some(ClockBinding::EventPosition),
        positions
            .iter()
            .map(|p| PositionObservation::new(*p, vec![], None))
            .collect(),
    )
}

// Trace: TC-176
#[test]
fn history_positions_must_be_consecutive_and_ascending() {
    assert!(event(&[0, 1, 2]).is_ok());
    assert_eq!(
        event(&[0, 0]).unwrap_err(),
        HistoryError::DuplicatePosition { position: 0 }
    );
    assert_eq!(
        event(&[0, 1, 2, 1]).unwrap_err(),
        HistoryError::OutOfOrder {
            previous: 2,
            current: 1
        }
    );
    assert_eq!(
        event(&[0, 2]).unwrap_err(),
        HistoryError::Gap {
            expected: 1,
            found: 2
        }
    );
}

// Trace: TC-176
#[test]
fn history_accessors_return_their_stored_values() {
    let history = PositionHistoryDocument::new(
        "history-x",
        3,
        0,
        1,
        Some(ClockBinding::EventPosition),
        vec![
            PositionObservation::new(0, vec![PropositionId(0)], None),
            PositionObservation::new(1, vec![], None),
        ],
    )
    .unwrap();
    assert_eq!(history.history_id(), "history-x");
    assert_eq!(history.revision(), 3);
    assert_eq!(history.through_position(), 1);
    assert_eq!(history.history_sha256().len(), 64);
    assert_ne!(history.history_sha256(), "xyzzy");
    assert_eq!(history.clock(), Some(&ClockBinding::EventPosition));
    assert_eq!(history.observations().len(), 2);
}

fn nested(inner: u32, outer: u32) -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, inner).unwrap(),
            operand: NodeId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, outer).unwrap(),
            operand: NodeId(1),
        }),
    ]
}

// Trace: TC-176
#[test]
fn required_history_adds_operand_requirements_and_bounds_the_identity() {
    let nodes = nested(5, 2);
    let formula =
        Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(2), &nodes).unwrap();
    assert_eq!(
        analyze_required_history(formula, "f")
            .unwrap()
            .required_positions,
        7
    );
    let max = "x".repeat(256);
    assert!(analyze_required_history(formula, max.as_str()).is_ok());
    let too_long = "x".repeat(257);
    assert_eq!(
        analyze_required_history(formula, too_long.as_str()).unwrap_err(),
        HistoryRequirementError::InvalidFormulaIdentity
    );
    assert_eq!(
        analyze_required_history(formula, "").unwrap_err(),
        HistoryRequirementError::InvalidFormulaIdentity
    );
    let report = analyze_required_history(formula, "f").unwrap();
    report.validate().unwrap();
    let mut long = report.clone();
    long.formula_id = "x".repeat(257);
    assert!(long.validate().is_err());
    let mut empty = report;
    empty.formula_id.clear();
    assert!(empty.validate().is_err());
}

// Trace: TC-176
#[test]
fn deserializer_errors_name_what_they_expected() {
    let unit = serde_json::from_value::<ClockSample>(
        serde_json::json!({"instant": {"numerator": 0, "denominator": 1}, "unit": 5}),
    )
    .unwrap_err()
    .to_string();
    assert!(unit.contains("clock unit with at most"), "{unit}");
    let ok = serde_json::from_value::<ClockSample>(
        serde_json::json!({"instant": {"numerator": 0, "denominator": 1}, "unit": "ticks"}),
    )
    .unwrap();
    assert_eq!(ok.unit, "ticks");
    let propositions = serde_json::from_value::<PositionObservation>(
        serde_json::json!({"position": 0, "truePropositions": 5}),
    )
    .unwrap_err()
    .to_string();
    assert!(
        propositions.contains("proposition identities"),
        "{propositions}"
    );
    let observations =
        serde_json::from_value::<PositionHistoryDocument>(serde_json::json!({"observations": 5}))
            .unwrap_err()
            .to_string();
    assert!(
        observations.contains("position observations"),
        "{observations}"
    );
}

// Trace: TC-176
#[test]
fn sample_instants_with_unit_denominators_terminate() {
    let instant = within_seconds(20, || {
        fixed_sample_instant(exact(0, 1), exact(1, 1), 2).unwrap()
    });
    assert_eq!((instant.numerator(), instant.denominator()), (2, 1));
}
