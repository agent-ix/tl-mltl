//! Ultimately periodic Boolean words used by the trace-scoped infinite evaluator.
//!
//! Each node has its own first periodic position. Past operators can delay that
//! position, so folding every node at the lasso's original loop entry is unsound.
//!
//! A closed past interval whose offset reaches before position 0 reads the
//! pre-origin extension of the trace: every atom is false there, constants keep
//! their values, and every operator evaluates there from those values (FSpec
//! FR-092 "Positions before the origin"). A pre-origin offset is never skipped.
//! Each word therefore also carries its values at the pre-origin positions `-1`
//! through `-depth`, and one constant value for every position below `-depth`,
//! where every read the node makes stays inside the extension. Unbounded past
//! operators range over the positions `0..=i-a` only and never read the
//! extension.

use std::collections::BTreeMap;

use tl_syntax::{InfiniteFormula, InfiniteNodeKind, NodeId, PropositionId, TemporalInterval};

use super::{EvaluationLimit, InfiniteError};

#[derive(Clone)]
struct Word {
    values: Vec<bool>,
    entry: usize,
    period: usize,
    /// `pre[k]` is the value at the pre-origin position `-(k + 1)`.
    pre: Vec<bool>,
    /// The value at every pre-origin position below `-pre.len()`.
    far: bool,
}

impl Word {
    /// The value at the position `position` at or after the origin.
    fn at_origin_or_after(&self, position: usize) -> bool {
        let index = if position < self.values.len() {
            position
        } else {
            self.entry + (position - self.entry) % self.period
        };
        self.values[index]
    }

    fn at(&self, position: i64) -> bool {
        match usize::try_from(position) {
            Ok(position) => self.at_origin_or_after(position),
            Err(_) => usize::try_from(position.unsigned_abs() - 1)
                .ok()
                .and_then(|back| self.pre.get(back).copied())
                .unwrap_or(self.far),
        }
    }

    fn depth(&self) -> usize {
        self.pre.len()
    }

    fn negated(&self) -> Self {
        Self {
            values: self.values.iter().map(|v| !v).collect(),
            entry: self.entry,
            period: self.period,
            pre: self.pre.iter().map(|v| !v).collect(),
            far: !self.far,
        }
    }
}

struct Budget {
    steps: u64,
    states: usize,
    limit: EvaluationLimit,
}

impl Budget {
    fn step(&mut self) -> Result<(), InfiniteError> {
        if self.limit.expired() {
            return Err(InfiniteError::ResourceIncomplete);
        }
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

    /// Materializes a word: positions `0..entry+period`, the pre-origin
    /// positions `-1..=-depth`, and the constant value below `-depth`.
    fn word(
        &mut self,
        entry: usize,
        period: usize,
        depth: usize,
        mut value: impl FnMut(i64, &mut Self) -> Result<bool, InfiniteError>,
    ) -> Result<Word, InfiniteError> {
        if self.limit.expired() {
            return Err(InfiniteError::ResourceIncomplete);
        }
        let length = entry
            .checked_add(period)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if length > self.limit.max_positions || depth > self.limit.max_positions {
            return Err(InfiniteError::ResourceIncomplete);
        }
        self.states = self
            .states
            .checked_add(length)
            .and_then(|states| states.checked_add(depth))
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if self.states > self.limit.max_states {
            return Err(InfiniteError::ResourceIncomplete);
        }
        let mut values = Vec::with_capacity(length);
        for position in 0..length {
            self.step()?;
            values.push(value(position_of(position)?, self)?);
        }
        let mut pre = Vec::with_capacity(depth);
        for back in 1..=depth {
            self.step()?;
            pre.push(value(-position_of(back)?, self)?);
        }
        self.step()?;
        let below = depth
            .checked_add(1)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        let far = value(-position_of(below)?, self)?;
        Ok(Word {
            values,
            entry,
            period,
            pre,
            far,
        })
    }
}

fn position_of(position: usize) -> Result<i64, InfiniteError> {
    i64::try_from(position).map_err(|_| InfiniteError::ResourceIncomplete)
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

fn deepest(left: &Word, right: &Word) -> usize {
    left.depth().max(right.depth())
}

/// The pre-origin depth of a future node: below it, every offset the node
/// reads first lands below its operands' depth. A closed interval reads at most
/// `b` ahead; an unbounded one starts reading `a` ahead.
fn future_depth(depth: usize, interval: TemporalInterval) -> Result<usize, InfiniteError> {
    let reach = match interval {
        TemporalInterval::Closed(closed) => closed.end(),
        TemporalInterval::Unbounded(open) => open.start(),
    };
    usize::try_from(reach)
        .ok()
        .and_then(|reach| depth.checked_add(reach))
        .ok_or(InfiniteError::ResourceIncomplete)
}

/// The pre-origin depth of a past node: a closed interval only reads farther
/// back, so it keeps its operands' depth; an unbounded one reads nothing
/// before the origin, so it is constant at every pre-origin position.
fn past_depth(depth: usize, interval: TemporalInterval) -> usize {
    match interval {
        TemporalInterval::Closed(_) => depth,
        TemporalInterval::Unbounded(_) => 0,
    }
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

/// The largest past offset read at `position`: `b` for a closed interval, and
/// `position` itself for an unbounded one, which stops at the origin.
fn upper(interval: TemporalInterval, position: i64) -> i64 {
    match interval {
        TemporalInterval::Closed(closed) => i64::from(closed.end()),
        TemporalInterval::Unbounded(_) => position,
    }
}

fn shifted(position: i64, offset: u32) -> Result<i64, InfiniteError> {
    position
        .checked_add(i64::from(offset))
        .ok_or(InfiniteError::ResourceIncomplete)
}

fn future_end(entry: usize, period: usize, start: i64) -> Result<i64, InfiniteError> {
    position_of(entry)?
        .max(start)
        .checked_add(position_of(period)?)
        .ok_or(InfiniteError::ResourceIncomplete)
}

fn future_unary(
    word: &Word,
    position: i64,
    interval: TemporalInterval,
    existential: bool,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let start = shifted(position, interval.start())?;
    let end = match interval {
        TemporalInterval::Closed(closed) => shifted(position, closed.end())?,
        TemporalInterval::Unbounded(_) => future_end(word.entry, word.period, start)? - 1,
    };
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
    position: i64,
    interval: TemporalInterval,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let min_witness = shifted(position, interval.start())?;
    let end = match interval {
        TemporalInterval::Closed(closed) => shifted(position, closed.end())?,
        TemporalInterval::Unbounded(_) => {
            future_end(aligned(left, right), left.period, min_witness)? - 1
        }
    };
    // The left operand covers the offsets `[a, i)` before the witness `i`, so
    // nothing below the lower bound is read (FR-031).
    for witness in min_witness..=end {
        budget.step()?;
        if right.at(witness) {
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
    position: i64,
    interval: TemporalInterval,
    existential: bool,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let start = i64::from(interval.start());
    for offset in start..=upper(interval, position) {
        budget.step()?;
        if word.at(position - offset) == existential {
            return Ok(existential);
        }
    }
    Ok(!existential)
}

fn past_since(
    left: &Word,
    right: &Word,
    position: i64,
    interval: TemporalInterval,
    budget: &mut Budget,
) -> Result<bool, InfiniteError> {
    let min_witness = i64::from(interval.start());
    // The left operand covers the offsets `[a, i)` between the witness and the
    // selected position, so nothing below the lower bound is read (FR-031).
    for offset in min_witness..=upper(interval, position) {
        budget.step()?;
        if right.at(position - offset) {
            return Ok(true);
        }
        if !left.at(position - offset) {
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
    if limit.expired() {
        return Err(InfiniteError::ResourceIncomplete);
    }
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
            InfiniteNodeKind::False => budget.word(loop_entry, period, 0, |_, _| Ok(false))?,
            InfiniteNodeKind::True => budget.word(loop_entry, period, 0, |_, _| Ok(true))?,
            InfiniteNodeKind::Proposition { proposition } => {
                let index = *proposition_index
                    .get(&proposition)
                    .ok_or(InfiniteError::InvalidFormula)?;
                budget.word(loop_entry, period, 0, |at, _| {
                    Ok(usize::try_from(at).is_ok_and(|at| valuations[at][index]))
                })?
            }
            InfiniteNodeKind::Not { operand } => {
                let inner = child(&words, operand)?;
                budget.word(
                    inner.entry,
                    period,
                    inner.depth(),
                    |at, _| Ok(!inner.at(at)),
                )?
            }
            InfiniteNodeKind::And { left, right }
            | InfiniteNodeKind::Or { left, right }
            | InfiniteNodeKind::Implies { left, right }
            | InfiniteNodeKind::Equivalent { left, right } => {
                let a = child(&words, left)?;
                let b = child(&words, right)?;
                budget.word(aligned(a, b), period, deepest(a, b), |at, _| {
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
                let depth = future_depth(inner.depth(), interval)?;
                budget.word(inner.entry, period, depth, |at, work| {
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
                let not_a = a.negated();
                let not_b = b.negated();
                let depth = future_depth(deepest(a, b), interval)?;
                budget.word(aligned(a, b), period, depth, |at, work| {
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
                let depth = past_depth(inner.depth(), interval);
                budget.word(entry, period, depth, |at, work| {
                    past_unary(inner, at, interval, existential, work)
                })?
            }
            InfiniteNodeKind::StrongPrevious { operand } => {
                // `once[1,1]`: at position 0 it reads the pre-origin position.
                let inner = child(&words, operand)?;
                let entry = inner
                    .entry
                    .checked_add(1)
                    .ok_or(InfiniteError::ResourceIncomplete)?;
                budget.word(entry, period, inner.depth(), |at, _| Ok(inner.at(at - 1)))?
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
                let not_a = a.negated();
                let not_b = b.negated();
                let entry = past_entry(aligned(a, b), interval, period)?;
                let depth = past_depth(deepest(a, b), interval);
                budget.word(entry, period, depth, |at, work| {
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
    let root = child(&words, claim_root)?.at_origin_or_after(position);
    let mut fair = Vec::with_capacity(fairness.len());
    for premise in fairness {
        let word = child(&words, *premise)?;
        let end = word
            .entry
            .checked_add(word.period)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        fair.push((word.entry..end).any(|at| word.at_origin_or_after(at)));
    }
    Ok((root, fair, budget.steps))
}
