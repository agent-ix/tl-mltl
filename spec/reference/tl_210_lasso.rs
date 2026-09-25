//! Test-only finite-lasso oracle for TL-210. It contains no production imports.
//! It covers future fixed points and past truth on the first materialized lap;
//! mixed future/past over later laps remains a planned TL-221 oracle obligation.
//! Run with `rustc --test spec/reference/tl_210_lasso.rs -o /private/tmp/tl_210_lasso && /private/tmp/tl_210_lasso`.

#[derive(Clone)]
enum Formula {
    Atom,
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    F(Box<Formula>),
    G(Box<Formula>),
    U(Box<Formula>, Box<Formula>),
    R(Box<Formula>, Box<Formula>),
    O(Box<Formula>),
    H(Box<Formula>),
    S(Box<Formula>, Box<Formula>),
    T(Box<Formula>, Box<Formula>),
    Y(Box<Formula>),
}

struct Lasso {
    atoms: Vec<bool>,
    loop_entry: usize,
}

impl Lasso {
    fn new(prefix: &[bool], loop_values: &[bool]) -> Self {
        assert!(!loop_values.is_empty());
        let mut atoms = prefix.to_vec();
        atoms.extend_from_slice(loop_values);
        Self {
            atoms,
            loop_entry: prefix.len(),
        }
    }

    fn next(&self, i: usize) -> usize {
        if i + 1 == self.atoms.len() {
            self.loop_entry
        } else {
            i + 1
        }
    }

    // Finite monotone iteration is independent of any provider encoding.
    fn fixed_point(&self, least: bool, step: impl Fn(usize, &[bool]) -> bool) -> Vec<bool> {
        let mut values = vec![!least; self.atoms.len()];
        loop {
            let next: Vec<_> = (0..values.len()).map(|i| step(i, &values)).collect();
            if next == values {
                return values;
            }
            values = next;
        }
    }

    fn eval(&self, formula: &Formula) -> Vec<bool> {
        use Formula::*;
        match formula {
            Atom => self.atoms.clone(),
            Not(p) => self.eval(p).into_iter().map(|v| !v).collect(),
            And(p, q) => self
                .eval(p)
                .into_iter()
                .zip(self.eval(q))
                .map(|(a, b)| a && b)
                .collect(),
            F(p) => {
                let p = self.eval(p);
                self.fixed_point(true, |i, old| p[i] || old[self.next(i)])
            }
            G(p) => {
                let p = self.eval(p);
                self.fixed_point(false, |i, old| p[i] && old[self.next(i)])
            }
            U(p, q) => {
                let (p, q) = (self.eval(p), self.eval(q));
                self.fixed_point(true, |i, old| q[i] || (p[i] && old[self.next(i)]))
            }
            R(p, q) => {
                let (p, q) = (self.eval(p), self.eval(q));
                self.fixed_point(false, |i, old| q[i] && (p[i] || old[self.next(i)]))
            }
            O(p) => {
                let p = self.eval(p);
                let mut out = Vec::with_capacity(p.len());
                for value in p {
                    out.push(value || out.last().copied().unwrap_or(false));
                }
                out
            }
            H(p) => {
                let p = self.eval(p);
                let mut out = Vec::with_capacity(p.len());
                for value in p {
                    out.push(value && out.last().copied().unwrap_or(true));
                }
                out
            }
            S(p, q) => {
                let (p, q) = (self.eval(p), self.eval(q));
                let mut out = Vec::with_capacity(p.len());
                for (a, b) in p.into_iter().zip(q) {
                    out.push(b || (a && out.last().copied().unwrap_or(false)));
                }
                out
            }
            T(p, q) => {
                let (p, q) = (self.eval(p), self.eval(q));
                let mut out = Vec::with_capacity(p.len());
                for (a, b) in p.into_iter().zip(q) {
                    out.push(b && (a || out.last().copied().unwrap_or(true)));
                }
                out
            }
            Y(p) => {
                let p = self.eval(p);
                std::iter::once(false)
                    .chain(p.into_iter().take(self.atoms.len() - 1))
                    .collect()
            }
        }
    }

    fn fair(&self, premises: &[Formula]) -> bool {
        premises
            .iter()
            .all(|p| self.eval(p)[self.loop_entry..].iter().any(|v| *v))
    }
}

#[cfg(test)]
mod tests {
    use super::{Formula as F, Lasso};

    // Planned criterion: TC-145, TC-146, TC-150, TC-151.
    #[test]
    fn lasso_future_fixed_points() {
        let word = Lasso::new(&[false], &[false, true]);
        assert_eq!(word.eval(&F::F(Box::new(F::Atom))), [true, true, true]);
        assert_eq!(word.eval(&F::G(Box::new(F::Atom))), [false, false, false]);
        assert_eq!(
            word.eval(&F::U(
                Box::new(F::Not(Box::new(F::Atom))),
                Box::new(F::Atom)
            )),
            [true, true, true]
        );
        assert_eq!(
            word.eval(&F::R(Box::new(F::Atom), Box::new(F::Atom))),
            [false, false, true]
        );
    }

    // Planned criterion: TC-147, TC-148.
    #[test]
    fn past_stops_at_origin() {
        let word = Lasso::new(&[false, true], &[false]);
        assert_eq!(word.eval(&F::Y(Box::new(F::Atom))), [false, false, true]);
        assert_eq!(word.eval(&F::O(Box::new(F::Atom))), [false, true, true]);
        assert_eq!(word.eval(&F::H(Box::new(F::Atom))), [false, false, false]);
        assert_eq!(
            word.eval(&F::S(Box::new(F::Atom), Box::new(F::Atom))),
            [false, true, false]
        );
        assert_eq!(
            word.eval(&F::T(Box::new(F::Atom), Box::new(F::Atom))),
            [false, true, false]
        );
    }

    // Planned criterion: TC-149, TC-152.
    #[test]
    fn duality_and_fairness() {
        let word = Lasso::new(&[], &[false, true]);
        let not_atom = F::Not(Box::new(F::Atom));
        let neg_f = F::Not(Box::new(F::F(Box::new(F::Atom))));
        assert_eq!(
            word.eval(&neg_f),
            word.eval(&F::G(Box::new(not_atom.clone())))
        );
        let neg_u = F::Not(Box::new(F::U(
            Box::new(F::Atom),
            Box::new(not_atom.clone()),
        )));
        let dual_r = F::R(Box::new(not_atom), Box::new(F::Atom));
        assert_eq!(word.eval(&neg_u), word.eval(&dual_r));
        let both = F::And(Box::new(F::Atom), Box::new(F::G(Box::new(F::Atom))));
        assert_eq!(word.eval(&both), [false, false]);
        assert!(word.fair(&[F::Atom]));
        assert!(!Lasso::new(&[], &[false]).fair(&[F::Atom]));
    }

    // Planned criterion: TC-150.
    #[test]
    fn empty_prefix_and_loop_unrolling_agree() {
        let first = Lasso::new(&[], &[false, true]);
        let unrolled = Lasso::new(&[false, true], &[false, true]);
        let formula = F::F(Box::new(F::Atom));
        assert_eq!(first.eval(&formula)[0], unrolled.eval(&formula)[0]);
    }
}
