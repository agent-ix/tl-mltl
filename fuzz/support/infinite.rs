//! Byte decoding shared by the TL-223 infinite-trace fuzz targets.
//!
//! The decoder maps any byte string to a formula, a partial lasso and a
//! selected position, with bounded size so that both the provider and the
//! reference procedures finish. An exhausted input reads as zero bytes, so
//! decoding is total and deterministic.

#![allow(dead_code)]

use crate::kit::{self, bx, not, Cell, Iv, Tr, C, F, M, T};
use tl_mltl::infinite::{Disposition, ResultReason};

pub struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub fn byte(&mut self) -> u8 {
        let b = self.data.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        b
    }

    fn interval(&mut self) -> Iv {
        let a = u32::from(self.byte() % 4);
        if self.byte().is_multiple_of(3) {
            Iv::Open(a)
        } else {
            Iv::Closed(a, a + u32::from(self.byte() % 3))
        }
    }

    pub fn formula(&mut self, depth: u32) -> T {
        let tag = self.byte() % if depth == 0 { 4 } else { 19 };
        match tag {
            0 => T::P(0),
            1 => T::P(1),
            2 => T::True,
            3 => not(T::P(0)),
            4 => not(self.formula(depth - 1)),
            5 => T::And(bx(self.formula(depth - 1)), bx(self.formula(depth - 1))),
            6 => T::Or(bx(self.formula(depth - 1)), bx(self.formula(depth - 1))),
            7 => T::Implies(bx(self.formula(depth - 1)), bx(self.formula(depth - 1))),
            8 => T::Equiv(bx(self.formula(depth - 1)), bx(self.formula(depth - 1))),
            9 => T::Y(bx(self.formula(depth - 1))),
            10 => {
                let i = self.interval();
                T::F(i, bx(self.formula(depth - 1)))
            }
            11 => {
                let i = self.interval();
                T::G(i, bx(self.formula(depth - 1)))
            }
            12 => {
                let i = self.interval();
                T::Once(i, bx(self.formula(depth - 1)))
            }
            13 => {
                let i = self.interval();
                T::Hist(i, bx(self.formula(depth - 1)))
            }
            14 => {
                let i = self.interval();
                T::U(i, bx(self.formula(depth - 1)), bx(self.formula(depth - 1)))
            }
            15 => {
                let i = self.interval();
                T::R(i, bx(self.formula(depth - 1)), bx(self.formula(depth - 1)))
            }
            16 => {
                let i = self.interval();
                T::Since(i, bx(self.formula(depth - 1)), bx(self.formula(depth - 1)))
            }
            17 => {
                let i = self.interval();
                T::Trig(i, bx(self.formula(depth - 1)), bx(self.formula(depth - 1)))
            }
            _ => T::False,
        }
    }

    /// Up to two fairness premises, each a formula of depth at most two.
    pub fn fairness(&mut self) -> Vec<T> {
        (0..self.byte() % 3).map(|_| self.formula(2)).collect()
    }

    /// A lasso of at most three prefix and three loop cells with at most
    /// `unknown_budget` missing or conflicting values.
    pub fn lasso(&mut self, mut unknown_budget: usize) -> (Vec<Cell>, Vec<Cell>) {
        let prefix = usize::from(self.byte() % 4);
        let tail = 1 + usize::from(self.byte() % 3);
        let mut cell = |r: &mut Self| -> Cell {
            let mut out = [F, F];
            for slot in &mut out {
                let b = r.byte();
                *slot = match b % 4 {
                    0 => F,
                    1 => Tr,
                    2 if unknown_budget > 0 => {
                        unknown_budget -= 1;
                        M
                    }
                    3 if unknown_budget > 0 => {
                        unknown_budget -= 1;
                        C
                    }
                    _ => Tr,
                };
            }
            out
        };
        let p: Vec<Cell> = (0..prefix).map(|_| cell(self)).collect();
        let t: Vec<Cell> = (0..tail).map(|_| cell(self)).collect();
        (p, t)
    }
}

/// The `lasso_differential` target body: provider against both references.
pub fn lasso_differential(data: &[u8]) {
    let mut reader = Reader::new(data);
    let claim = reader.formula(3);
    let fairness = reader.fairness();
    let (prefix, tail) = reader.lasso(8);
    let position = u64::from(reader.byte() % 8);
    let built = kit::build(&claim, &fairness);
    let word = kit::lasso(&prefix, &tail);
    let actual = kit::provider(&built, &word, position);
    let fixpoint = kit::fixpoint(&built, &word, position);
    let scanning = kit::scanning(&built, &word, position);
    assert_eq!(
        fixpoint, scanning,
        "references disagree: {claim:?} {fairness:?} {prefix:?}.{tail:?} at {position}"
    );
    if let Some(why) = kit::disagreement(&actual, &fixpoint) {
        panic!("{why}: {claim:?} {fairness:?} {prefix:?}.{tail:?} at {position}");
    }
}

fn refine(cells: &[Cell], reader: &mut Reader<'_>) -> Vec<Cell> {
    cells
        .iter()
        .map(|cell| {
            let mut out = *cell;
            for value in &mut out {
                if *value == M {
                    *value = if reader.byte() & 1 == 1 { Tr } else { F };
                }
            }
            out
        })
        .collect()
}

/// The `partial_valuation_monotone` target body: a conclusive verdict survives
/// refining missing values, and the refined result matches the reference.
pub fn partial_valuation_monotone(data: &[u8]) {
    let mut reader = Reader::new(data);
    let claim = reader.formula(3);
    let fairness = reader.fairness();
    let (prefix, tail) = reader.lasso(8);
    let position = u64::from(reader.byte() % 8);
    let (fine_prefix, fine_tail) = (refine(&prefix, &mut reader), refine(&tail, &mut reader));

    let built = kit::build(&claim, &fairness);
    let coarse = kit::provider(&built, &kit::lasso(&prefix, &tail), position);
    let fine_word = kit::lasso(&fine_prefix, &fine_tail);
    let fine = kit::provider(&built, &fine_word, position);

    if let Some(why) = kit::disagreement(&fine, &kit::fixpoint(&built, &fine_word, position)) {
        panic!(
            "refined lasso: {why}: {claim:?} {fairness:?} {fine_prefix:?}.{fine_tail:?} at {position}"
        );
    }
    if matches!(
        coarse.disposition,
        Disposition::Proved | Disposition::Refuted
    ) {
        if fine.admitted_completions > 0 {
            assert_eq!(
                coarse.disposition, fine.disposition,
                "conclusive verdict changed under refinement: {claim:?} {fairness:?} \
                 {prefix:?}.{tail:?} -> {fine_prefix:?}.{fine_tail:?} at {position}"
            );
        } else {
            assert_eq!(fine.reason, Some(ResultReason::EmptyFairAdmission));
            assert!(!fairness.is_empty());
        }
    }
}
