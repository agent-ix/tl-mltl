//! Bounded production-boundary calls shared by cargo-fuzz and seed smoke tests.

use tl_mltl::past::{
    history, ClockBinding, ClockSample, ExactNumber, HistoryError, PositionHistoryDocument,
    PositionObservation,
};
use tl_mltl::wire::{OwnerLimits, OwnerReadErrorCode, ValidatedCommand, ValidatedTrace};
use tl_mltl::CommandDocument;
use tl_syntax::PropositionId;

const MAX_FUZZ_INPUT: usize = 64 * 1024;

fn limits() -> OwnerLimits {
    OwnerLimits {
        max_input_bytes: MAX_FUZZ_INPUT,
        max_output_bytes: MAX_FUZZ_INPUT,
        max_depth: 32,
        max_string_bytes: 4096,
        max_formula_nodes: 256,
        max_formula_depth: 128,
        max_positions: 64,
        max_propositions: 256,
        max_support: 256,
        max_history_span: 64,
        max_evaluation_steps: 4096,
        max_recursion_depth: 64,
        max_visited_fields: 4096,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireOutcome {
    Oversize,
    DecodeRefused,
    OwnerRefused(OwnerReadErrorCode),
    Admitted,
}

/// Drive the actual legacy CLI decode and the strict owner command reader.
pub fn exercise_wire_cli(data: &[u8]) -> WireOutcome {
    if data.len() > MAX_FUZZ_INPUT {
        return WireOutcome::Oversize;
    }
    let legacy = serde_json::from_slice::<CommandDocument>(data);
    if let Ok(command) = &legacy {
        let _ = command.formula.validate();
        if let Some(trace) = &command.trace {
            let _ = ValidatedTrace::admit(trace, limits());
        }
    }
    match ValidatedCommand::from_json_bytes(data, limits()) {
        Ok(owner) => {
            assert_eq!(legacy.as_ref().ok(), Some(owner.document()));
            assert_eq!(owner.bytes(), data);
            WireOutcome::Admitted
        }
        Err(error) if legacy.is_ok() => WireOutcome::OwnerRefused(error.code()),
        Err(_) => WireOutcome::DecodeRefused,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IntakeOutcome {
    pub oversize: bool,
    pub trace_admitted: bool,
    pub history_admitted: bool,
    pub structured_admitted: bool,
    pub correction_admitted: bool,
    pub resource_refused: bool,
    pub structured_error: Option<HistoryError>,
}

/// Drive trace/history byte intake and bounded append/correction constructors.
pub fn exercise_trace_history(data: &[u8]) -> IntakeOutcome {
    if data.len() > MAX_FUZZ_INPUT {
        return IntakeOutcome {
            oversize: true,
            ..IntakeOutcome::default()
        };
    }
    let mut outcome = IntakeOutcome {
        trace_admitted: ValidatedTrace::from_json_bytes(data, limits()).is_ok(),
        ..IntakeOutcome::default()
    };
    if let Ok(document) = serde_json::from_slice::<PositionHistoryDocument>(data) {
        if let Ok(owner) = history::read(data, &document, limits()) {
            assert_eq!(owner.bytes(), data);
            outcome.history_admitted = true;
        }
    }

    let count = usize::from(data.first().copied().unwrap_or(0)) % 66 + 1;
    let fixed = data.get(1).copied().unwrap_or(0) & 1 != 0;
    let clock = if fixed {
        let Ok(zero) = ExactNumber::new(0, 1) else {
            return outcome;
        };
        let Ok(one) = ExactNumber::new(1, 1) else {
            return outcome;
        };
        ClockBinding::FixedSample {
            epoch: zero,
            period: one,
            unit: "tick".to_owned(),
        }
    } else {
        ClockBinding::EventPosition
    };
    let observations: Vec<_> = (0..count)
        .map(|index| observation(index, data.get(index + 2).copied().unwrap_or(2), fixed))
        .collect();
    match PositionHistoryDocument::new(
        "fuzz-history",
        1,
        0,
        u64::try_from(count - 1).unwrap_or(0),
        Some(clock.clone()),
        observations,
    ) {
        Ok(document) => {
            outcome.structured_admitted = true;
            match history::derive(&document, limits()) {
                Ok(owner) => {
                    assert!(history::read(owner.bytes(), &document, limits()).is_ok());
                }
                Err(error) => {
                    outcome.resource_refused =
                        error.code() == OwnerReadErrorCode::ResourceIncomplete;
                }
            }
        }
        Err(error) => outcome.structured_error = Some(error),
    }

    let first = observation(0, 2, fixed);
    if let Ok(original) =
        PositionHistoryDocument::new("fuzz-append", 1, 0, 0, Some(clock), vec![first.clone()])
    {
        let appended = observation(1, data.get(3).copied().unwrap_or(2), fixed);
        if let Ok(corrected) = original.corrected(2, 1, vec![first, appended]) {
            outcome.correction_admitted = true;
            if let Ok(owner) = history::derive(&corrected, limits()) {
                assert!(history::read(owner.bytes(), &corrected, limits()).is_ok());
            }
        }
    }
    outcome
}

fn observation(index: usize, selector: u8, fixed: bool) -> PositionObservation {
    let position = if index > 0 && selector.is_multiple_of(16) {
        u64::try_from(index - 1).unwrap_or(0)
    } else if index > 1 && selector % 16 == 1 {
        0
    } else if selector % 16 == 3 {
        u64::try_from(index + 1).unwrap_or(0)
    } else {
        u64::try_from(index).unwrap_or(0)
    };
    let propositions = match selector % 8 {
        5 => vec![PropositionId(1), PropositionId(0)],
        6 => vec![PropositionId(0), PropositionId(0)],
        7 => vec![PropositionId(0)],
        _ => Vec::new(),
    };
    let sample = if fixed && selector % 8 != 4 {
        ExactNumber::new(i64::try_from(position).unwrap_or(0), 1)
            .ok()
            .map(|instant| ClockSample {
                instant,
                unit: if selector % 8 == 5 { "other" } else { "tick" }.to_owned(),
            })
    } else {
        None
    };
    PositionObservation::new(position, propositions, sample)
}
