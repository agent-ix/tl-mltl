//! Ultimately periodic Boolean words used by the trace-scoped infinite evaluator.
//!
//! Each node has its own first periodic position. Past operators can delay that
//! position, so folding every node at the lasso's original loop entry is unsound.

use std::collections::BTreeMap;

use tl_syntax::{InfiniteFormula, InfiniteNodeKind, NodeId, PropositionId, TemporalInterval};

use super::{EvaluationLimit, InfiniteError};

#[derive(Clone)]
struct Word {
    values: Vec<bool>,
    entry: usize,
    period: usize,
}

impl Word {
    fn at(&self, position: usize) -> bool {
        let index = if position < self.values.len() {
            position
        } else {
            self.entry + (position - self.entry) % self.period
        };
        self.values[index]
    }
}

struct Budget {
    steps: u64,
    states: usize,
    limit: EvaluationLimit,
}

impl Budget {
    fn step(&mut self) -> Result<(), InfiniteError> {
        self.steps = self
            .steps
            .checked_add(1)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if self.steps > self.limit.max_steps {
            Err(InfiniteError::ResourceIncomplete)
        } else {
            Ok(())
        }
    }

    fn word(
        &mut self,
        entry: usize,
        period: usize,
        mut value: impl FnMut(usize, &mut Self) -> Result<bool, InfiniteError>,
    ) -> Result<Word, InfiniteError> {
        let length = entry
            .checked_add(period)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if length > self.limit.max_positions {
            return Err(InfiniteError::ResourceIncomplete);
        }
        self.states = self
            .states
            .checked_add(length)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if self.states > self.limit.max_states {
            return Err(InfiniteError::ResourceIncomplete);
        }
        let mut values = Vec::with_capacity(length);
        for position in 0..length {
            self.step()?;
            values.push(value(position, self)?);
        }
        Ok(Word {
            values,
            entry,
            period,
        })
    }
}

fn child(words: &[Word], node: NodeId) -> Result<&Word, InfiniteError> {
    usize::try_from(node.0)
        .ok()
        .and_then(|index| words.get(index))
        .ok_or(InfiniteError::InvalidFormula)
}

fn aligned(left: &Word, right: &Word) -> usize {
    left.entry.max(right.entry)
}

fn past_entry(
    entry: usize,
    interval: TemporalInterval,
    period: usize,
) -> Result<usize, InfiniteError> {
    let delay = match interval {
        TemporalInterval::Closed(closed) => {
            usize::try_from(closed.end()).map_err(|_| InfiniteError::ResourceIncomplete)?
        }
        TemporalInterval::Unbounded(open) => usize::try_from(open.start())
            .map_err(|_| InfiniteError::ResourceIncomplete)?
            .checked_add(period)
            .ok_or(InfiniteError::ResourceIncomplete)?,
    };
    entry
        .checked_add(delay)
        .ok_or(InfiniteError::ResourceIncomplete)
}

fn upper(interval: TemporalInterval, position: usize) -> Result<usize, InfiniteError> {
    match interval {
        TemporalInterval::Closed(closed) => {
            usize::try_from(closed.end()).map_err(|_| InfiniteError::ResourceIncomplete)
        }
        TemporalInterval::Unbounded(_) => Ok(position),
    }
}

fn future_end(entry: usize, period: usize, start: usize) -> Result<usize, InfiniteError> {
    entry
        .max(start)
        .checked_add(period)
        .ok_or(InfiniteError::ResourceIncomplete)
}

fn future_unary(
    word: &Word,
    position: usize,
    interval: TemporalInterval,
    existential: bool,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let start = position
        .checked_add(
            usize::try_from(interval.start()).map_err(|_| InfiniteError::ResourceIncomplete)?,
        )
        .ok_or(InfiniteError::ResourceIncomplete)?;
    let end = match interval {
        TemporalInterval::Closed(closed) => position.checked_add(
            usize::try_from(closed.end()).map_err(|_| InfiniteError::ResourceIncomplete)?,
        ),
        TemporalInterval::Unbounded(_) => {
            future_end(word.entry, word.period, start)?.checked_sub(1)
        }
    }
    .ok_or(InfiniteError::ResourceIncomplete)?;
    for at in start..=end {
        budget.step()?;
        if word.at(at) == existential {
            return Ok(existential);
        }
    }
    Ok(!existential)
}

fn future_until(
    left: &Word,
    right: &Word,
    position: usize,
    interval: TemporalInterval,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let min_witness = position
        .checked_add(
            usize::try_from(interval.start()).map_err(|_| InfiniteError::ResourceIncomplete)?,
        )
        .ok_or(InfiniteError::ResourceIncomplete)?;
    let end = match interval {
        TemporalInterval::Closed(closed) => position.checked_add(
            usize::try_from(closed.end()).map_err(|_| InfiniteError::ResourceIncomplete)?,
        ),
        TemporalInterval::Unbounded(_) => {
            future_end(aligned(left, right), left.period, min_witness)?.checked_sub(1)
        }
    }
    .ok_or(InfiniteError::ResourceIncomplete)?;
    for witness in position..=end {
        budget.step()?;
        if witness >= min_witness && right.at(witness) {
            return Ok(true);
        }
        if !left.at(witness) {
            return Ok(false);
        }
    }
    Ok(false)
}

fn past_unary(
    word: &Word,
    position: usize,
    interval: TemporalInterval,
    existential: bool,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let start = usize::try_from(interval.start()).map_err(|_| InfiniteError::ResourceIncomplete)?;
    let end = upper(interval, position)?;
    for offset in start..=end {
        budget.step()?;
        let Some(at) = position.checked_sub(offset) else {
            continue;
        };
        let value = word.at(at);
        if value == existential {
            return Ok(existential);
        }
    }
    Ok(!existential)
}

fn past_since(
    left: &Word,
    right: &Word,
    position: usize,
    interval: TemporalInterval,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let min_witness =
        usize::try_from(interval.start()).map_err(|_| InfiniteError::ResourceIncomplete)?;
    let end = upper(interval, position)?.min(position);
    for offset in 0..=end {
        budget.step()?;
        if offset >= min_witness && position.checked_sub(offset).is_some_and(|at| right.at(at)) {
            return Ok(true);
        }
        if !position.checked_sub(offset).is_some_and(|at| left.at(at)) {
            return Ok(false);
        }
    }
    Ok(false)
}

/// Evaluates every graph node as an ultimately periodic word for one complete
/// Boolean lasso completion. The caller owns identity and completion admission.
pub(super) fn evaluate(
    formula: InfiniteFormula<'_>,
    claim_root: NodeId,
    valuations: &[Vec<bool>],
    loop_entry: usize,
    proposition_index: &BTreeMap<PropositionId, usize>,
    position: usize,
    fairness: &[NodeId],
    limit: EvaluationLimit,
) -> Result<(bool, Vec<bool>, u64), InfiniteError> {
    let period = valuations
        .len()
        .checked_sub(loop_entry)
        .filter(|period| *period > 0)
        .ok_or(InfiniteError::InvalidLasso)?;
    let mut budget = Budget {
        steps: 0,
        states: 0,
        limit,
    };
    let mut words: Vec<Word> = Vec::with_capacity(formula.nodes().len());
    for node in formula.nodes() {
        let word = match node.kind {
            InfiniteNodeKind::False => budget.word(loop_entry, period, |_, _| Ok(false))?,
            InfiniteNodeKind::True => budget.word(loop_entry, period, |_, _| Ok(true))?,
            InfiniteNodeKind::Proposition { proposition } => {
                let index = *proposition_index
                    .get(&proposition)
                    .ok_or(InfiniteError::InvalidFormula)?;
                budget.word(loop_entry, period, |at, _| Ok(valuations[at][index]))?
            }
            InfiniteNodeKind::Not { operand } => {
                let inner = child(&words, operand)?;
                budget.word(inner.entry, period, |at, _| Ok(!inner.at(at)))?
            }
            InfiniteNodeKind::And { left, right }
            | InfiniteNodeKind::Or { left, right }
            | InfiniteNodeKind::Implies { left, right }
            | InfiniteNodeKind::Equivalent { left, right } => {
                let a = child(&words, left)?;
                let b = child(&words, right)?;
                budget.word(aligned(a, b), period, |at, _| {
                    Ok(match node.kind {
                        InfiniteNodeKind::And { .. } => a.at(at) && b.at(at),
                        InfiniteNodeKind::Or { .. } => a.at(at) || b.at(at),
                        InfiniteNodeKind::Implies { .. } => !a.at(at) || b.at(at),
                        InfiniteNodeKind::Equivalent { .. } => a.at(at) == b.at(at),
                        _ => unreachable!(),
                    })
                })?
            }
            InfiniteNodeKind::Future { interval, operand }
            | InfiniteNodeKind::Globally { interval, operand } => {
                let inner = child(&words, operand)?;
                let existential = matches!(node.kind, InfiniteNodeKind::Future { .. });
                budget.word(inner.entry, period, |at, work| {
                    future_unary(inner, at, interval, existential, work)
                })?
            }
            InfiniteNodeKind::Until {
                interval,
                left,
                right,
            }
            | InfiniteNodeKind::Release {
                interval,
                left,
                right,
            } => {
                let a = child(&words, left)?;
                let b = child(&words, right)?;
                let release = matches!(node.kind, InfiniteNodeKind::Release { .. });
                let not_a = Word {
                    values: a.values.iter().map(|v| !v).collect(),
                    entry: a.entry,
                    period,
                };
                let not_b = Word {
                    values: b.values.iter().map(|v| !v).collect(),
                    entry: b.entry,
                    period,
                };
                budget.word(aligned(a, b), period, |at, work| {
                    let value = if release {
                        future_until(&not_a, &not_b, at, interval, work)?
                    } else {
                        future_until(a, b, at, interval, work)?
                    };
                    Ok(value ^ release)
                })?
            }
            InfiniteNodeKind::Once { interval, operand }
            | InfiniteNodeKind::Historically { interval, operand } => {
                let inner = child(&words, operand)?;
                let existential = matches!(node.kind, InfiniteNodeKind::Once { .. });
                let entry = past_entry(inner.entry, interval, period)?;
                budget.word(entry, period, |at, work| {
                    past_unary(inner, at, interval, existential, work)
                })?
            }
            InfiniteNodeKind::StrongPrevious { operand } => {
                let inner = child(&words, operand)?;
                let entry = inner
                    .entry
                    .checked_add(1)
                    .ok_or(InfiniteError::ResourceIncomplete)?;
                budget.word(entry, period, |at, _| {
                    Ok(at.checked_sub(1).is_some_and(|prior| inner.at(prior)))
                })?
            }
            InfiniteNodeKind::Since {
                interval,
                left,
                right,
            }
            | InfiniteNodeKind::Triggered {
                interval,
                left,
                right,
            } => {
                let a = child(&words, left)?;
                let b = child(&words, right)?;
                let trigger = matches!(node.kind, InfiniteNodeKind::Triggered { .. });
                let not_a = Word {
                    values: a.values.iter().map(|v| !v).collect(),
                    entry: a.entry,
                    period,
                };
                let not_b = Word {
                    values: b.values.iter().map(|v| !v).collect(),
                    entry: b.entry,
                    period,
                };
                let entry = past_entry(aligned(a, b), interval, period)?;
                budget.word(entry, period, |at, work| {
                    let value = if trigger {
                        past_since(&not_a, &not_b, at, interval, work)?
                    } else {
                        past_since(a, b, at, interval, work)?
                    };
                    Ok(value ^ trigger)
                })?
            }
        };
        words.push(word);
    }
    let root = child(&words, claim_root)?.at(position);
    let mut fair = Vec::with_capacity(fairness.len());
    for premise in fairness {
        let word = child(&words, *premise)?;
        let end = word
            .entry
            .checked_add(word.period)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        fair.push((word.entry..end).any(|at| word.at(at)));
    }
    Ok((root, fair, budget.steps))
}
