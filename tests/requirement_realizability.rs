//! TL-416 / FR-043: requirement realizability reference play-out (TM-007).
//!
//! The declared population is generated here from the rule of tl-solve TM-100
//! (section "Declared small population"), restated in tl-mltl TM-007; nothing
//! is imported from tl-solve (FR-113). The independent side is a direct
//! closed-trace evaluator over a test-only formula tree plus explicit
//! quantifier elimination; it shares no code with `src/`.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! The full-population tests are `#[ignore]`d because they judge 440,951,040
//! plays; the default run checks a deterministic stride sample of the same
//! population. Both are listed in the PLAN deferred-verification table.

use tl_mltl::requirement_realizability::{
    judge_strategy, play_out, PlayOutError, PlayOutLimits, PlayOutVerdict, RequirementFormula,
    StrategyJudgement,
};
use tl_mltl::EvaluationLimits;
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile, SignalPartition,
};

// ------------------------------------------------------------ test-only tree

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Bop {
    And,
    Or,
    Implies,
    Equivalent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum P {
    Atom(u32),
    Not(Box<P>),
    Bin(Bop, Box<P>, Box<P>),
    F(u32, u32, Box<P>),
    G(u32, u32, Box<P>),
    U(u32, u32, Box<P>, Box<P>),
}

const J: [(u32, u32); 5] = [(0, 0), (0, 1), (1, 1), (0, 2), (1, 2)];
const J2: [(u32, u32); 2] = [(0, 1), (1, 1)];

fn atom(index: u32) -> P {
    P::Atom(index)
}

/// Independent lookahead: sums interval ends along nesting.
fn horizon(formula: &P) -> u64 {
    match formula {
        P::Atom(_) => 0,
        P::Not(operand) => horizon(operand),
        P::Bin(_, left, right) => horizon(left).max(horizon(right)),
        P::F(_, end, operand) | P::G(_, end, operand) => u64::from(*end) + horizon(operand),
        P::U(_, end, left, right) => u64::from(*end) + horizon(left).max(horizon(right)),
    }
}

/// Direct closed-trace semantics: positions past the end are all-false.
/// `trace[t]` is a 4-bit valuation, bit `k` = `p_k`.
fn holds(formula: &P, trace: &[u8], time: usize) -> bool {
    match formula {
        P::Atom(index) => trace.get(time).is_some_and(|letter| letter >> index & 1 == 1),
        P::Not(operand) => !holds(operand, trace, time),
        P::Bin(op, left, right) => {
            let (l, r) = (holds(left, trace, time), holds(right, trace, time));
            match op {
                Bop::And => l && r,
                Bop::Or => l || r,
                Bop::Implies => !l || r,
                Bop::Equivalent => l == r,
            }
        }
        P::F(start, end, operand) => {
            (*start..=*end).any(|offset| holds(operand, trace, time + offset as usize))
        }
        P::G(start, end, operand) => {
            (*start..=*end).all(|offset| holds(operand, trace, time + offset as usize))
        }
        P::U(start, end, left, right) => (*start..=*end).any(|witness| {
            holds(right, trace, time + witness as usize)
                && (*start..witness).all(|offset| holds(left, trace, time + offset as usize))
        }),
    }
}

/// Lowers a tree to a topologically ordered node arena.
fn lower(formula: &P, nodes: &mut Vec<Node>) -> NodeId {
    let kind = match formula {
        P::Atom(index) => NodeKind::Proposition {
            proposition: PropositionId(*index),
        },
        P::Not(operand) => NodeKind::Not {
            operand: lower(operand, nodes),
        },
        P::Bin(op, left, right) => {
            let left = lower(left, nodes);
            let right = lower(right, nodes);
            match op {
                Bop::And => NodeKind::And { left, right },
                Bop::Or => NodeKind::Or { left, right },
                Bop::Implies => NodeKind::Implies { left, right },
                Bop::Equivalent => NodeKind::Equivalent { left, right },
            }
        }
        P::F(start, end, operand) => NodeKind::Future {
            interval: Interval::new(*start, *end).unwrap(),
            operand: lower(operand, nodes),
        },
        P::G(start, end, operand) => NodeKind::Globally {
            interval: Interval::new(*start, *end).unwrap(),
            operand: lower(operand, nodes),
        },
        P::U(start, end, left, right) => {
            let left = lower(left, nodes);
            let right = lower(right, nodes);
            NodeKind::Until {
                interval: Interval::new(*start, *end).unwrap(),
                left,
                right,
            }
        }
    };
    nodes.push(Node::new(kind));
    NodeId((nodes.len() - 1) as u32)
}

struct Lowered {
    nodes: Vec<Node>,
    root: NodeId,
}

impl Lowered {
    fn new(formula: &P) -> Self {
        let mut nodes = Vec::new();
        let root = lower(formula, &mut nodes);
        Self { nodes, root }
    }

    fn formula(&self, profile: SemanticProfile) -> Formula<'_> {
        Formula::new(profile, self.root, &self.nodes).unwrap()
    }
}

// --------------------------------------------------------------- population

fn p1() -> Vec<P> {
    let mut set = Vec::new();
    for a in 0..4 {
        set.push(atom(a));
    }
    for a in 0..4 {
        set.push(P::Not(Box::new(atom(a))));
    }
    for op in [Bop::And, Bop::Or, Bop::Implies] {
        for a in 0..4 {
            for b in 0..4 {
                set.push(P::Bin(op, Box::new(atom(a)), Box::new(atom(b))));
            }
        }
    }
    for temporal in 0..2 {
        for &(s, e) in &J {
            for a in 0..4 {
                set.push(if temporal == 0 {
                    P::F(s, e, Box::new(atom(a)))
                } else {
                    P::G(s, e, Box::new(atom(a)))
                });
            }
        }
    }
    for &(s, e) in &J {
        for a in 0..4 {
            for b in 0..4 {
                set.push(P::U(s, e, Box::new(atom(a)), Box::new(atom(b))));
            }
        }
    }
    set
}

fn p2() -> Vec<P> {
    let mut terms = Vec::new();
    for temporal in 0..2 {
        for &(s, e) in &J {
            for b in 0..4 {
                terms.push(if temporal == 0 {
                    P::F(s, e, Box::new(atom(b)))
                } else {
                    P::G(s, e, Box::new(atom(b)))
                });
            }
        }
    }
    let mut set = Vec::new();
    for op in [Bop::And, Bop::Or, Bop::Implies, Bop::Equivalent] {
        for a in 0..4 {
            for term in &terms {
                set.push(P::Bin(op, Box::new(atom(a)), Box::new(term.clone())));
                set.push(P::Bin(op, Box::new(term.clone()), Box::new(atom(a))));
            }
        }
    }
    set
}

fn p3(p1: &[P]) -> Vec<P> {
    let mut set = Vec::new();
    for temporal in 0..2 {
        for &(s, e) in &J2 {
            for f in p1 {
                set.push(if temporal == 0 {
                    P::F(s, e, Box::new(f.clone()))
                } else {
                    P::G(s, e, Box::new(f.clone()))
                });
            }
        }
    }
    set
}

/// One case: a requirement set and an input mask over `p0..p3`.
struct Case {
    formulas: Vec<P>,
    input_mask: u8,
}

fn population() -> Vec<Case> {
    let p1 = p1();
    let mut singles = p1.clone();
    singles.extend(p2());
    singles.extend(p3(&p1));
    let mut cases = Vec::new();
    for formula in &singles {
        for input_mask in 0..16u8 {
            cases.push(Case {
                formulas: vec![formula.clone()],
                input_mask,
            });
        }
    }
    let two_input_splits: Vec<u8> = (0..16u8).filter(|mask| mask.count_ones() == 2).collect();
    for first in 0..p1.len() {
        for second in (first + 1)..p1.len() {
            for &input_mask in &two_input_splits {
                cases.push(Case {
                    formulas: vec![p1[first].clone(), p1[second].clone()],
                    input_mask,
                });
            }
        }
    }
    cases
}

// ------------------------------------------------- independent game solver

struct Independent {
    length: usize,
    input_mask: u8,
    table: Vec<bool>,
}

fn subsets(mask: u8) -> impl Iterator<Item = u8> {
    (0..16u8).filter(move |value| value & !mask == 0)
}

impl Independent {
    fn new(case: &Case) -> Self {
        let length = case.formulas.iter().map(horizon).max().unwrap() as usize + 1;
        let plays = 16usize.pow(length as u32);
        let mut table = Vec::with_capacity(plays);
        let mut trace = vec![0u8; length];
        for play in 0..plays {
            let mut rest = play;
            for step in (0..length).rev() {
                trace[step] = (rest % 16) as u8;
                rest /= 16;
            }
            table.push(case.formulas.iter().all(|f| holds(f, &trace, 0)));
        }
        Self {
            length,
            input_mask: case.input_mask,
            table,
        }
    }

    fn winnable(&self, prefix: usize, step: usize) -> bool {
        if step == self.length {
            return self.table[prefix];
        }
        subsets(self.input_mask).all(|input| {
            subsets(!self.input_mask & 0x0f)
                .any(|output| self.winnable(prefix * 16 + usize::from(input | output), step + 1))
        })
    }

    fn realizable(&self) -> bool {
        self.winnable(0, 0)
    }

    fn forall_exists(&self) -> bool {
        let input_letters: Vec<u8> = subsets(self.input_mask).collect();
        let output_letters: Vec<u8> = subsets(!self.input_mask & 0x0f).collect();
        let sequences = |letters: &[u8]| -> Vec<Vec<u8>> {
            let mut all = vec![Vec::new()];
            for _ in 0..self.length {
                all = all
                    .into_iter()
                    .flat_map(|prefix| {
                        letters.iter().map(move |letter| {
                            let mut next = prefix.clone();
                            next.push(*letter);
                            next
                        })
                    })
                    .collect();
            }
            all
        };
        let outputs = sequences(&output_letters);
        sequences(&input_letters).iter().all(|inputs| {
            outputs.iter().any(|outs| {
                let play = inputs
                    .iter()
                    .zip(outs)
                    .fold(0usize, |acc, (i, o)| acc * 16 + usize::from(i | o));
                self.table[play]
            })
        })
    }

    /// Lowest winning output at `step` after `prefix`, for input `input`.
    fn lowest_response(&self, prefix: usize, step: usize, input: u8) -> u8 {
        subsets(!self.input_mask & 0x0f)
            .find(|output| self.winnable(prefix * 16 + usize::from(input | output), step + 1))
            .unwrap_or(0)
    }
}

// ----------------------------------------------------------------- helpers

fn ids(mask: u8) -> Vec<PropositionId> {
    (0..4u32)
        .filter(|bit| mask >> bit & 1 == 1)
        .map(PropositionId)
        .collect()
}

fn mask_of(list: &[PropositionId]) -> u8 {
    list.iter().fold(0u8, |acc, id| acc | (1 << id.0))
}

// UNBUILT-PIN: `SignalPartition` is tl-syntax FR-034. The constructor name and
// shape are a guess; once tl-syntax lands, build it through
// `SignalCatalog::partition_all` over a v2 catalog if no direct constructor exists.
fn partition(input_mask: u8, all_mask: u8) -> SignalPartition {
    SignalPartition::new(ids(input_mask), ids(all_mask & !input_mask))
}

fn limits() -> PlayOutLimits {
    PlayOutLimits {
        max_trace_length: 4,
        max_bits_per_step: 4,
        max_plays: 1 << 16,
        evaluation: EvaluationLimits::default(),
    }
}

fn run(case: &Case) -> Result<tl_mltl::requirement_realizability::PlayOutReport, PlayOutError> {
    let lowered: Vec<Lowered> = case.formulas.iter().map(Lowered::new).collect();
    let requirements: Vec<RequirementFormula<'_>> = lowered
        .iter()
        .enumerate()
        .map(|(index, lowered)| RequirementFormula {
            id: if index == 0 { "r0" } else { "r1" },
            formula: lowered.formula(SemanticProfile::ClosedTraceV1),
        })
        .collect();
    play_out(&requirements, &partition(case.input_mask, 0x0f), limits())
}

fn check_case(case: &Case) -> (bool, bool) {
    let independent = Independent::new(case);
    let report = run(case).expect("population case is admitted");
    let expected_length = case.formulas.iter().map(horizon).max().unwrap() + 1;
    assert_eq!(report.trace_length, expected_length);
    assert_eq!(report.max_lookahead + 1, report.trace_length);
    assert_eq!(report.plays_evaluated, 16u64.pow(expected_length as u32));
    let realizable = independent.realizable();
    let forall_exists = independent.forall_exists();
    assert_eq!(report.verdict == PlayOutVerdict::Realizable, realizable);
    assert_eq!(report.every_input_sequence_has_response, forall_exists);
    assert!(!(realizable && !forall_exists), "Realizable without relaxation");
    (realizable, forall_exists)
}

/// Every `STRIDE`-th population case; the full run is the ignored test below.
const STRIDE: usize = 97;

// ------------------------------------------------------------------- tests

// Trace: TC-188, FR-043-AC-1, FR-043-AC-2, StR-004-VC-1
#[test]
fn population_counts_follow_the_declared_rule() {
    let p1 = p1();
    assert_eq!(p1.len(), 176);
    assert_eq!(p2().len(), 1280);
    assert_eq!(p3(&p1).len(), 704);
    let cases = population();
    assert_eq!(cases.len(), 126_960);
    let singles: Vec<&Case> = cases.iter().filter(|case| case.formulas.len() == 1).collect();
    let mut by_horizon = [0usize; 4];
    for case in singles.iter().step_by(16) {
        by_horizon[horizon(&case.formulas[0]) as usize] += 1;
    }
    assert_eq!(by_horizon, [336, 880, 752, 192]);
}

// Trace: TC-188, FR-043-AC-1, StR-004-VC-1
#[test]
fn trace_length_is_one_plus_max_lookahead_on_the_sampled_population() {
    for case in population().iter().step_by(STRIDE) {
        let report = run(case).unwrap();
        let expected = case.formulas.iter().map(horizon).max().unwrap() + 1;
        assert_eq!(report.trace_length, expected);
    }
}

// Trace: TC-188, FR-043-AC-1
#[test]
fn non_closed_trace_profiles_are_refused() {
    // `PlayOutLimits` has no length member and `play_out` no length parameter:
    // the API accepts no caller length (checked by inspection of the types).
    let prefix = Lowered::new(&P::F(0, 1, Box::new(atom(0))));
    let requirement = [RequirementFormula {
        id: "prefix",
        formula: prefix.formula(SemanticProfile::OnlinePrefixV1),
    }];
    assert!(matches!(
        play_out(&requirement, &partition(0b0001, 0b0001), limits()),
        Err(PlayOutError::UnsupportedProfile { .. })
    ));

    let past_nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let past = Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(1), &past_nodes)
        .unwrap();
    let requirement = [RequirementFormula {
        id: "past",
        formula: past,
    }];
    assert!(matches!(
        play_out(&requirement, &partition(0b0001, 0b0001), limits()),
        Err(PlayOutError::UnsupportedProfile { .. })
    ));
}

// Trace: TC-188, TC-189, FR-043-AC-1, FR-043-AC-2, FR-043-AC-3, StR-004-VC-1
#[test]
#[ignore = "full declared population (440,951,040 plays); run with --ignored"]
fn full_population_matches_independent_enumeration() {
    let (mut realizable, mut unrealizable, mut passes_relaxation) = (0u64, 0u64, 0u64);
    for case in &population() {
        let (r, fe) = check_case(case);
        if r {
            realizable += 1;
        } else {
            unrealizable += 1;
            if fe {
                passes_relaxation += 1;
            }
        }
    }
    assert_eq!(realizable + unrealizable, 126_960);
    assert_eq!(realizable, 44_566);
    assert_eq!(unrealizable, 82_394);
    assert_eq!(passes_relaxation, 768);
}

// Trace: TC-189, FR-043-AC-2, FR-043-AC-3, StR-004-VC-1
#[test]
fn sampled_population_matches_independent_enumeration() {
    for case in population().iter().step_by(STRIDE) {
        check_case(case);
    }
}

// Trace: TC-189, FR-043-AC-2, FR-043-AC-3
#[test]
fn copy_now_is_realizable_and_copy_next_is_not() {
    // p0 input, p1 output.
    let now = Case {
        formulas: vec![P::G(
            0,
            1,
            Box::new(P::Bin(Bop::Equivalent, Box::new(atom(1)), Box::new(atom(0)))),
        )],
        input_mask: 0b0001,
    };
    let lowered = Lowered::new(&now.formulas[0]);
    let requirement = [RequirementFormula {
        id: "copy-now",
        formula: lowered.formula(SemanticProfile::ClosedTraceV1),
    }];
    let report = play_out(&requirement, &partition(0b0001, 0b0011), limits()).unwrap();
    assert_eq!(report.verdict, PlayOutVerdict::Realizable);
    assert!(report.every_input_sequence_has_response);

    let next = P::Bin(
        Bop::Equivalent,
        Box::new(atom(1)),
        Box::new(P::F(1, 1, Box::new(atom(0)))),
    );
    let lowered = Lowered::new(&next);
    let requirement = [RequirementFormula {
        id: "copy-next",
        formula: lowered.formula(SemanticProfile::ClosedTraceV1),
    }];
    let report = play_out(&requirement, &partition(0b0001, 0b0011), limits()).unwrap();
    assert_eq!(report.verdict, PlayOutVerdict::Unrealizable);
    // The relaxation passes: the response exists once all inputs are known.
    assert!(report.every_input_sequence_has_response);
}

// Trace: TC-190, FR-043-AC-4, StR-004-VC-2
#[test]
fn all_output_is_satisfiability_and_all_input_is_validity() {
    for case in population()
        .iter()
        .filter(|case| case.input_mask == 0 || case.input_mask == 0x0f)
        .step_by(7)
    {
        let independent = Independent::new(case);
        let report = run(case).unwrap();
        let expected = if case.input_mask == 0 {
            independent.table.iter().any(|holds| *holds)
        } else {
            independent.table.iter().all(|holds| *holds)
        };
        assert_eq!(report.verdict == PlayOutVerdict::Realizable, expected);
    }
}

// Trace: TC-191, FR-043-AC-5, StR-004-VC-2
#[test]
fn limits_admit_the_edge_and_refuse_one_past_it() {
    // G[0,1] p0 over p0 (output) and p1 (input): L = 2, width 2, 16 plays.
    let lowered = Lowered::new(&P::G(0, 1, Box::new(atom(0))));
    let requirement = [RequirementFormula {
        id: "g",
        formula: lowered.formula(SemanticProfile::ClosedTraceV1),
    }];
    let split = partition(0b0010, 0b0011);
    let exact = PlayOutLimits {
        max_trace_length: 2,
        max_bits_per_step: 2,
        max_plays: 16,
        evaluation: EvaluationLimits::default(),
    };
    assert!(play_out(&requirement, &split, exact).is_ok());
    assert!(matches!(
        play_out(&requirement, &split, PlayOutLimits { max_trace_length: 1, ..exact }),
        Err(PlayOutError::TraceLengthLimit { requested: 2, limit: 1 })
    ));
    assert!(matches!(
        play_out(&requirement, &split, PlayOutLimits { max_bits_per_step: 1, ..exact }),
        Err(PlayOutError::AlphabetWidthLimit { requested: 2, limit: 1 })
    ));
    assert!(matches!(
        play_out(&requirement, &split, PlayOutLimits { max_plays: 15, ..exact }),
        Err(PlayOutError::PlayCountLimit { requested: 16, limit: 15 })
    ));
    // Refusal happens before any play: a zero evaluation budget would
    // otherwise surface as an evaluation failure.
    let starved = PlayOutLimits {
        max_plays: 15,
        evaluation: EvaluationLimits {
            max_node_evaluations: 0,
            ..EvaluationLimits::default()
        },
        ..exact
    };
    assert!(matches!(
        play_out(&requirement, &split, starved),
        Err(PlayOutError::PlayCountLimit { .. })
    ));
    // p0 occurs but the partition lists only p1.
    assert_eq!(
        play_out(&requirement, &partition(0b0010, 0b0010), exact),
        Err(PlayOutError::PropositionNotPartitioned {
            proposition: PropositionId(0)
        })
    );
    assert_eq!(
        play_out(&[], &split, exact),
        Err(PlayOutError::EmptyRequirementSet)
    );
}

// Trace: TC-192, FR-043-AC-6, StR-004-VC-1
#[test]
fn reports_are_deterministic_and_evaluation_failure_has_no_verdict() {
    let case = Case {
        formulas: vec![P::U(1, 2, Box::new(atom(0)), Box::new(atom(1)))],
        input_mask: 0b0101,
    };
    assert_eq!(run(&case), run(&case));

    let lowered = Lowered::new(&case.formulas[0]);
    let requirement = [RequirementFormula {
        id: "u",
        formula: lowered.formula(SemanticProfile::ClosedTraceV1),
    }];
    let starved = PlayOutLimits {
        evaluation: EvaluationLimits {
            max_node_evaluations: 1,
            ..EvaluationLimits::default()
        },
        ..limits()
    };
    assert!(matches!(
        play_out(&requirement, &partition(0b0101, 0x0f), starved),
        Err(PlayOutError::EvaluationFailure { .. })
    ));
}

// Trace: TC-193, FR-043-AC-7, StR-004-VC-1
#[test]
fn judge_strategy_wins_loses_and_refuses() {
    // Copy-now game: p0 input, p1 output.
    let formula = P::G(
        0,
        1,
        Box::new(P::Bin(Bop::Equivalent, Box::new(atom(1)), Box::new(atom(0)))),
    );
    let lowered = Lowered::new(&formula);
    let requirement = [RequirementFormula {
        id: "copy-now",
        formula: lowered.formula(SemanticProfile::ClosedTraceV1),
    }];
    let split = partition(0b0001, 0b0011);
    let copy = |history: &[Vec<PropositionId>]| -> Vec<PropositionId> {
        if history.last().unwrap().contains(&PropositionId(0)) {
            vec![PropositionId(1)]
        } else {
            Vec::new()
        }
    };
    assert_eq!(
        judge_strategy(&requirement, &split, limits(), copy),
        Ok(StrategyJudgement::Wins {
            input_sequences_played: 4
        })
    );

    // Violates only on input sequence (p0, p0) at step 1: order is
    // (0,0) (0,1) (1,0) (1,1), so it is the fourth and first losing one.
    let faulty = |history: &[Vec<PropositionId>]| -> Vec<PropositionId> {
        let all_high = history.len() == 2
            && history.iter().all(|step| step.contains(&PropositionId(0)));
        if history.last().unwrap().contains(&PropositionId(0)) && !all_high {
            vec![PropositionId(1)]
        } else {
            Vec::new()
        }
    };
    assert_eq!(
        judge_strategy(&requirement, &split, limits(), faulty),
        Ok(StrategyJudgement::Loses {
            first_losing_input_sequence: vec![vec![PropositionId(0)], vec![PropositionId(0)]],
            input_sequences_played: 4,
        })
    );

    let outside = |_: &[Vec<PropositionId>]| vec![PropositionId(0)];
    assert!(matches!(
        judge_strategy(&requirement, &split, limits(), outside),
        Err(PlayOutError::ResponseOutsideOutputs {
            step: 0,
            proposition: PropositionId(0)
        })
    ));
}

fn judge_with_lowest_response(case: &Case) {
    let independent = Independent::new(case);
    let lowered: Vec<Lowered> = case.formulas.iter().map(Lowered::new).collect();
    let requirements: Vec<RequirementFormula<'_>> = lowered
        .iter()
        .map(|lowered| RequirementFormula {
            id: "r",
            formula: lowered.formula(SemanticProfile::ClosedTraceV1),
        })
        .collect();
    let respond = |history: &[Vec<PropositionId>]| -> Vec<PropositionId> {
        let mut prefix = 0usize;
        let mut output = 0u8;
        for (step, inputs) in history.iter().enumerate() {
            let input = mask_of(inputs);
            output = independent.lowest_response(prefix, step, input);
            prefix = prefix * 16 + usize::from(input | output);
        }
        ids(output)
    };
    let judgement =
        judge_strategy(&requirements, &partition(case.input_mask, 0x0f), limits(), respond)
            .unwrap();
    assert_eq!(
        matches!(judgement, StrategyJudgement::Wins { .. }),
        independent.realizable()
    );
}

// Trace: TC-193, FR-043-AC-7, StR-004-VC-1
#[test]
fn derived_lowest_response_wins_exactly_when_realizable_sampled() {
    for case in population().iter().step_by(STRIDE) {
        judge_with_lowest_response(case);
    }
}

// Trace: TC-193, FR-043-AC-7, StR-004-VC-1
#[test]
#[ignore = "full declared population; run with --ignored"]
fn derived_lowest_response_wins_exactly_when_realizable_full() {
    for case in &population() {
        judge_with_lowest_response(case);
    }
}
