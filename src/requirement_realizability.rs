//! Requirement realizability reference play-out (FR-043, ADR-004).
//!
//! An exponential, bounded reference oracle: it enumerates every play of the
//! fixed-horizon, inputs-first game of ADR-004 and judges each complete play
//! with [`crate::evaluate_closed_at`]. It is the independent source of truth
//! that tl-solve's game search and re-check are compared against. It generates
//! no strategy and no code, shares no code with tl-solve (FR-113), and is not a
//! production decision procedure.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;
use std::collections::BTreeSet;

use tl_syntax::{Formula, NodeKind, PropositionId, SemanticProfile, SignalPartition};

use crate::{
    analyze_horizon, evaluate_closed_at, EvaluationError, EvaluationLimits, HorizonError,
    TruthValue,
};

/// One requirement formula with its caller-supplied identity.
#[derive(Clone, Copy, Debug)]
pub struct RequirementFormula<'a> {
    /// Caller-supplied formula identity, carried into the report.
    pub id: &'a str,
    /// Validated `mltl.closed-trace/v1` formula.
    pub formula: Formula<'a>,
}

/// Declared ceilings checked before any play is evaluated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayOutLimits {
    /// Maximum trace length `L` in steps.
    pub max_trace_length: u64,
    /// Maximum input-plus-output propositions per step.
    pub max_bits_per_step: u32,
    /// Maximum number of complete plays evaluated.
    pub max_plays: u64,
    /// Limits of every closed-trace evaluation of a play.
    pub evaluation: EvaluationLimits,
}

impl Default for PlayOutLimits {
    fn default() -> Self {
        Self {
            max_trace_length: 8,
            max_bits_per_step: 8,
            max_plays: 1 << 20,
            evaluation: EvaluationLimits::default(),
        }
    }
}

/// Game verdict of the fixed-horizon, inputs-first game.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayOutVerdict {
    /// `forall i_0 exists o_0 ... forall i_H exists o_H` every formula holds.
    Realizable,
    /// The alternating game value is false.
    Unrealizable,
}

/// The reference answer with the counts that show how it was reached.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayOutReport {
    /// Game verdict.
    pub verdict: PlayOutVerdict,
    /// Played trace length `L = H + 1`.
    pub trace_length: u64,
    /// The maximum `lookahead` `H` that fixed the length.
    pub max_lookahead: u64,
    /// Identity of the first formula (request order) whose horizon report
    /// carried `max_lookahead`, so a reviewer can see where `L` came from.
    pub horizon_formula_id: String,
    /// Input propositions of the partition, strictly increasing.
    pub inputs: Vec<PropositionId>,
    /// Output propositions of the partition, strictly increasing.
    pub outputs: Vec<PropositionId>,
    /// Number of complete plays judged, `2^((|I|+|O|)*L)`.
    pub plays_evaluated: u64,
    /// Truth of `forall i_0..i_H exists o_0..o_H` (the necessary relaxation).
    /// `Realizable` implies it; the reverse does not hold.
    pub every_input_sequence_has_response: bool,
}

/// Result of judging one caller strategy (`judge_strategy`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrategyJudgement {
    /// Every play produced by the callback satisfies every formula.
    Wins {
        /// Number of input sequences played.
        input_sequences_played: u64,
    },
    /// Some play violates a formula.
    Loses {
        /// Lexicographically first losing input sequence (see [`judge_strategy`]).
        first_losing_input_sequence: Vec<Vec<PropositionId>>,
        /// Number of input sequences played up to and including the losing one.
        input_sequences_played: u64,
    },
}

/// Closed set of typed refusals. No variant carries a verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayOutError {
    /// A formula is not under `mltl.closed-trace/v1`.
    UnsupportedProfile {
        /// Offending formula identity.
        formula_id: String,
        /// Its profile identity.
        actual: &'static str,
    },
    /// Horizon analysis failed, or `H + 1` overflowed.
    HorizonFailure {
        /// Offending formula identity.
        formula_id: String,
        /// Horizon failure.
        error: HorizonError,
    },
    /// No formula was supplied.
    EmptyRequirementSet,
    /// A proposition occurs in a formula but in neither partition list, or in both.
    PropositionNotPartitioned {
        /// The proposition.
        proposition: PropositionId,
    },
    /// `L` exceeds `max_trace_length`.
    TraceLengthLimit {
        /// Derived `L`.
        requested: u64,
        /// Configured limit.
        limit: u64,
    },
    /// `|I| + |O|` exceeds `max_bits_per_step`.
    AlphabetWidthLimit {
        /// Requested bits per step.
        requested: u64,
        /// Configured limit.
        limit: u32,
    },
    /// `2^((|I|+|O|)*L)` (or `2^(|I|*L)` for `judge_strategy`) exceeds `max_plays`.
    PlayCountLimit {
        /// Requested play count, saturated at `u128::MAX`.
        requested: u128,
        /// Configured limit.
        limit: u64,
    },
    /// Closed-trace evaluation of a play failed.
    EvaluationFailure {
        /// Formula whose evaluation failed.
        formula_id: String,
        /// Evaluation failure.
        error: EvaluationError,
    },
    /// A `judge_strategy` callback named a proposition outside the output list.
    ResponseOutsideOutputs {
        /// Step at which the callback answered.
        step: u64,
        /// Offending proposition.
        proposition: PropositionId,
    },
    /// A `judge_strategy` callback answer was not strictly increasing.
    ResponseNotStrictlyOrdered {
        /// Step at which the callback answered.
        step: u64,
    },
}

impl fmt::Display for PlayOutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { formula_id, actual } => write!(
                formatter,
                "requirement realizability play-out requires {}, formula {formula_id} is {actual}",
                SemanticProfile::ClosedTraceV1.as_str()
            ),
            Self::HorizonFailure { formula_id, error } => {
                write!(formatter, "horizon of formula {formula_id} failed: {error}")
            }
            Self::EmptyRequirementSet => write!(formatter, "empty requirement set"),
            Self::PropositionNotPartitioned { proposition } => write!(
                formatter,
                "proposition {} is not in exactly one partition list",
                proposition.0
            ),
            Self::TraceLengthLimit { requested, limit } => {
                write!(formatter, "trace length {requested} exceeds limit {limit}")
            }
            Self::AlphabetWidthLimit { requested, limit } => {
                write!(formatter, "{requested} bits per step exceed limit {limit}")
            }
            Self::PlayCountLimit { requested, limit } => {
                write!(formatter, "{requested} plays exceed limit {limit}")
            }
            Self::EvaluationFailure { formula_id, error } => {
                write!(formatter, "evaluation of formula {formula_id} failed: {error}")
            }
            Self::ResponseOutsideOutputs { step, proposition } => write!(
                formatter,
                "strategy response at step {step} names non-output proposition {}",
                proposition.0
            ),
            Self::ResponseNotStrictlyOrdered { step } => write!(
                formatter,
                "strategy response at step {step} is not strictly increasing"
            ),
        }
    }
}

impl std::error::Error for PlayOutError {}

/// Admitted request: derived length, partition lists and checked play count.
struct Game<'a, 'f> {
    formulas: &'a [RequirementFormula<'f>],
    inputs: Vec<PropositionId>,
    outputs: Vec<PropositionId>,
    length: u64,
    max_lookahead: u64,
    horizon_formula_id: String,
    evaluation: EvaluationLimits,
}

// UNBUILT-PIN: `SignalPartition` is tl-syntax FR-034 (TL-149 branch, not on
// tl-syntax main at a80d09e). The accessor names `inputs()`/`outputs()` are a
// guess at the spec'd `inputs` and `outputs` lists; replace with the merged API.
fn partition_lists(partition: &SignalPartition) -> (Vec<PropositionId>, Vec<PropositionId>) {
    (partition.inputs().to_vec(), partition.outputs().to_vec())
}

fn occurring_propositions(formula: Formula<'_>) -> BTreeSet<PropositionId> {
    formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition),
            _ => None,
        })
        .collect()
}

/// `2^exponent` as `u128`, saturating.
fn power_of_two(exponent: u64) -> u128 {
    if exponent >= 128 {
        u128::MAX
    } else {
        1u128 << exponent
    }
}

impl<'a, 'f> Game<'a, 'f> {
    /// Validates the request and checks every limit before any play.
    fn admit(
        formulas: &'a [RequirementFormula<'f>],
        partition: &SignalPartition,
        limits: PlayOutLimits,
        judging: bool,
    ) -> Result<Self, PlayOutError> {
        if formulas.is_empty() {
            return Err(PlayOutError::EmptyRequirementSet);
        }
        let mut max_lookahead = 0u64;
        let mut horizon_formula_id = formulas[0].id.to_owned();
        for requirement in formulas {
            if requirement.formula.profile() != SemanticProfile::ClosedTraceV1 {
                return Err(PlayOutError::UnsupportedProfile {
                    formula_id: requirement.id.to_owned(),
                    actual: requirement.formula.profile().as_str(),
                });
            }
            let report = analyze_horizon(requirement.formula, requirement.id).map_err(|error| {
                PlayOutError::HorizonFailure {
                    formula_id: requirement.id.to_owned(),
                    error,
                }
            })?;
            if report.lookahead > max_lookahead {
                max_lookahead = report.lookahead;
                horizon_formula_id = requirement.id.to_owned();
            }
        }
        let (inputs, outputs) = partition_lists(partition);
        for requirement in formulas {
            for proposition in occurring_propositions(requirement.formula) {
                let in_inputs = inputs.binary_search(&proposition).is_ok();
                let in_outputs = outputs.binary_search(&proposition).is_ok();
                if in_inputs == in_outputs {
                    return Err(PlayOutError::PropositionNotPartitioned { proposition });
                }
            }
        }
        let length = max_lookahead
            .checked_add(1)
            .ok_or_else(|| PlayOutError::HorizonFailure {
                formula_id: horizon_formula_id.clone(),
                error: HorizonError::ArithmeticOverflow {
                    node: formulas[0].formula.root(),
                },
            })?;
        if length > limits.max_trace_length {
            return Err(PlayOutError::TraceLengthLimit {
                requested: length,
                limit: limits.max_trace_length,
            });
        }
        let width = (inputs.len() as u64).saturating_add(outputs.len() as u64);
        if width > u64::from(limits.max_bits_per_step) {
            return Err(PlayOutError::AlphabetWidthLimit {
                requested: width,
                limit: limits.max_bits_per_step,
            });
        }
        let enumerated_bits = if judging {
            inputs.len() as u64
        } else {
            width
        };
        let plays = power_of_two(enumerated_bits.saturating_mul(length));
        if plays > u128::from(limits.max_plays) {
            return Err(PlayOutError::PlayCountLimit {
                requested: plays,
                limit: limits.max_plays,
            });
        }
        Ok(Self {
            formulas,
            inputs,
            outputs,
            length,
            max_lookahead,
            horizon_formula_id,
            evaluation: limits.evaluation,
        })
    }

    fn input_bits(&self) -> u32 {
        // Bounded by `max_bits_per_step: u32`.
        self.inputs.len() as u32
    }

    fn output_bits(&self) -> u32 {
        self.outputs.len() as u32
    }

    fn steps(&self) -> usize {
        // Bounded by `max_plays: u64` via `2^(width * L)`; a positive width or
        // `L <= max_trace_length` keeps this small in every admitted request.
        self.length as usize
    }

    /// True propositions of one step: bit `k` of `input` selects `inputs[k]`,
    /// bit `k` of `output` selects `outputs[k]`. Strictly increasing.
    fn instant(&self, input: u64, output: u64) -> Vec<PropositionId> {
        let mut instant = Vec::new();
        for (bit, proposition) in self.inputs.iter().enumerate() {
            if input >> bit & 1 == 1 {
                instant.push(*proposition);
            }
        }
        for (bit, proposition) in self.outputs.iter().enumerate() {
            if output >> bit & 1 == 1 {
                instant.push(*proposition);
            }
        }
        instant.sort_unstable();
        instant
    }

    fn input_list(&self, input: u64) -> Vec<PropositionId> {
        self.instant(input, 0)
    }

    /// Judges one complete play: every formula `True` at position zero.
    fn judge(&self, trace: &[Vec<PropositionId>]) -> Result<bool, PlayOutError> {
        for requirement in self.formulas {
            let report = evaluate_closed_at(
                requirement.formula,
                requirement.id,
                trace,
                "requirement-realizability-play",
                0,
                self.evaluation,
            )
            .map_err(|error| PlayOutError::EvaluationFailure {
                formula_id: requirement.id.to_owned(),
                error,
            })?;
            if report.verdict != TruthValue::True {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Satisfaction table over every play. Play index: step 0 is most
    /// significant; within a step the input valuation is above the output.
    fn table(&self) -> Result<Vec<bool>, PlayOutError> {
        let step_bits = self.input_bits() + self.output_bits();
        // Admission bounded this by `max_plays: u64`.
        let plays = 1u64 << (u64::from(step_bits) * self.length);
        let output_mask = (1u64 << self.output_bits()) - 1;
        let step_mask = (1u64 << step_bits) - 1;
        let mut table = Vec::with_capacity(plays as usize);
        let mut trace = vec![Vec::new(); self.steps()];
        for play in 0..plays {
            for step in 0..self.steps() {
                let shift = u64::from(step_bits) * (self.length - 1 - step as u64);
                let letter = (play >> shift) & step_mask;
                let input = letter >> self.output_bits();
                let output = letter & output_mask;
                trace[step] = self.instant(input, output);
            }
            table.push(self.judge(&trace)?);
        }
        Ok(table)
    }

    /// `forall i_t exists o_t ... ` from step `step` under play prefix `prefix`.
    fn alternating(&self, table: &[bool], step: u64, prefix: u64) -> bool {
        if step == self.length {
            return table[prefix as usize];
        }
        let step_bits = self.input_bits() + self.output_bits();
        for input in 0..(1u64 << self.input_bits()) {
            let mut answered = false;
            for output in 0..(1u64 << self.output_bits()) {
                let letter = (input << self.output_bits()) | output;
                let next = (prefix << step_bits) | letter;
                if self.alternating(table, step + 1, next) {
                    answered = true;
                    break;
                }
            }
            if !answered {
                return false;
            }
        }
        true
    }

    /// `forall i_0 .. i_H exists o_0 .. o_H`.
    fn every_input_sequence_has_response(&self, table: &[bool]) -> bool {
        let input_sequences = 1u64 << (u64::from(self.input_bits()) * self.length);
        let output_sequences = 1u64 << (u64::from(self.output_bits()) * self.length);
        let input_mask = (1u64 << self.input_bits()) - 1;
        let output_mask = (1u64 << self.output_bits()) - 1;
        let step_bits = self.input_bits() + self.output_bits();
        for inputs in 0..input_sequences {
            let mut answered = false;
            for outputs in 0..output_sequences {
                let mut play = 0u64;
                for step in 0..self.length {
                    let shift = self.length - 1 - step;
                    let input = (inputs >> (u64::from(self.input_bits()) * shift)) & input_mask;
                    let output = (outputs >> (u64::from(self.output_bits()) * shift)) & output_mask;
                    play = (play << step_bits) | (input << self.output_bits()) | output;
                }
                if table[play as usize] {
                    answered = true;
                    break;
                }
            }
            if !answered {
                return false;
            }
        }
        true
    }
}

/// Decides requirement realizability by exhaustive play-out.
///
/// `L = H + 1` where `H` is the maximum `analyze_horizon` lookahead; the caller
/// supplies no length. Every limit is checked before any play is evaluated.
/// Any evaluation failure returns that failure and no verdict.
///
/// Implements: FR-043
pub fn play_out(
    formulas: &[RequirementFormula<'_>],
    partition: &SignalPartition,
    limits: PlayOutLimits,
) -> Result<PlayOutReport, PlayOutError> {
    let game = Game::admit(formulas, partition, limits, false)?;
    let table = game.table()?;
    let realizable = game.alternating(&table, 0, 0);
    let relaxation = game.every_input_sequence_has_response(&table);
    Ok(PlayOutReport {
        verdict: if realizable {
            PlayOutVerdict::Realizable
        } else {
            PlayOutVerdict::Unrealizable
        },
        trace_length: game.length,
        max_lookahead: game.max_lookahead,
        horizon_formula_id: game.horizon_formula_id.clone(),
        inputs: game.inputs.clone(),
        outputs: game.outputs.clone(),
        plays_evaluated: table.len() as u64,
        every_input_sequence_has_response: relaxation,
    })
}

/// Judges a caller strategy against the same game.
///
/// `respond` maps the input history through step `t` (one strictly increasing
/// list of true input propositions per step) to the output valuation of step
/// `t`. Input sequences are enumerated in lexicographic order of
/// `(v_0, .., v_H)` where `v_t` is the integer whose bit `k` is set iff
/// `inputs[k]` is true at step `t`; `Loses` carries the first losing one.
/// The callback type mentions no tl-solve type (FR-113).
///
/// Implements: FR-043 (FR-043-AC-7)
pub fn judge_strategy<F>(
    formulas: &[RequirementFormula<'_>],
    partition: &SignalPartition,
    limits: PlayOutLimits,
    mut respond: F,
) -> Result<StrategyJudgement, PlayOutError>
where
    F: FnMut(&[Vec<PropositionId>]) -> Vec<PropositionId>,
{
    // SPEC-GAP: FR-043 does not order "lexicographically first" input
    // sequences; the order above is the simplest total order on valuations.
    let game = Game::admit(formulas, partition, limits, true)?;
    let input_bits = u64::from(game.input_bits());
    let input_mask = (1u64 << game.input_bits()) - 1;
    let sequences = 1u64 << (input_bits * game.length);
    let mut played = 0u64;
    for sequence in 0..sequences {
        played += 1;
        let mut history: Vec<Vec<PropositionId>> = Vec::with_capacity(game.steps());
        let mut trace: Vec<Vec<PropositionId>> = Vec::with_capacity(game.steps());
        for step in 0..game.length {
            let shift = input_bits * (game.length - 1 - step);
            let input = (sequence >> shift) & input_mask;
            let input_list = game.input_list(input);
            history.push(input_list.clone());
            let response = respond(&history);
            if response.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(PlayOutError::ResponseNotStrictlyOrdered { step });
            }
            if let Some(proposition) = response
                .iter()
                .find(|proposition| game.outputs.binary_search(proposition).is_err())
            {
                return Err(PlayOutError::ResponseOutsideOutputs {
                    step,
                    proposition: *proposition,
                });
            }
            let mut instant = input_list;
            instant.extend(response);
            instant.sort_unstable();
            trace.push(instant);
        }
        if !game.judge(&trace)? {
            return Ok(StrategyJudgement::Loses {
                first_losing_input_sequence: history,
                input_sequences_played: played,
            });
        }
    }
    Ok(StrategyJudgement::Wins {
        input_sequences_played: played,
    })
}
