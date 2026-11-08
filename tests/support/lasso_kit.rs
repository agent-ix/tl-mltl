//! Shared test kit for the infinite-trace verification suites (TL-221, TL-222,
//! TL-238, TL-223): a formula tree that lowers to a syntax graph, lasso
//! builders, the provider call, and the bridges to the two independent
//! reference procedures in `tl-oracle`.
#![allow(dead_code)]

use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, InfiniteResult, LassoRequest,
    ResultReason, SettlementEvidence,
};
use tl_oracle::{Limits, OracleError, Outcome, Uncertainty, Verdict};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, LassoTraceDocument, NodeId, PartialValuation, PartialValue,
    PropositionId, SemanticProfile, TemporalInterval, TraceObservation, UnboundedInterval,
    ValuationEntry,
};

#[allow(unused_imports)]
pub use PartialValue::{Conflicting as C, False as F, Missing as M, True as Tr};

/// A temporal interval: closed `[a,b]` or unbounded `[a,)`.
#[derive(Clone, Copy, Debug)]
pub enum Iv {
    Closed(u32, u32),
    Open(u32),
}

impl Iv {
    fn lower(self) -> TemporalInterval {
        match self {
            Self::Closed(a, b) => TemporalInterval::Closed(Interval::new(a, b).unwrap()),
            Self::Open(a) => TemporalInterval::Unbounded(UnboundedInterval::new(a)),
        }
    }
}

/// A formula tree over propositions 0 and 1.
#[derive(Clone, Debug)]
pub enum T {
    True,
    False,
    P(u32),
    Not(Box<T>),
    And(Box<T>, Box<T>),
    Or(Box<T>, Box<T>),
    Implies(Box<T>, Box<T>),
    Equiv(Box<T>, Box<T>),
    F(Iv, Box<T>),
    G(Iv, Box<T>),
    U(Iv, Box<T>, Box<T>),
    R(Iv, Box<T>, Box<T>),
    Once(Iv, Box<T>),
    Hist(Iv, Box<T>),
    Since(Iv, Box<T>, Box<T>),
    Trig(Iv, Box<T>, Box<T>),
    Y(Box<T>),
}

pub fn bx(t: T) -> Box<T> {
    Box::new(t)
}
pub fn not(t: T) -> T {
    T::Not(bx(t))
}

fn lower(tree: &T, nodes: &mut Vec<InfiniteNode>) -> NodeId {
    let kind = match tree {
        T::True => K::True,
        T::False => K::False,
        T::P(p) => K::Proposition {
            proposition: PropositionId(*p),
        },
        T::Not(x) => K::Not {
            operand: lower(x, nodes),
        },
        T::And(x, y) => K::And {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Or(x, y) => K::Or {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Implies(x, y) => K::Implies {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Equiv(x, y) => K::Equivalent {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::F(i, x) => K::Future {
            interval: i.lower(),
            operand: lower(x, nodes),
        },
        T::G(i, x) => K::Globally {
            interval: i.lower(),
            operand: lower(x, nodes),
        },
        T::U(i, x, y) => K::Until {
            interval: i.lower(),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::R(i, x, y) => K::Release {
            interval: i.lower(),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Once(i, x) => K::Once {
            interval: i.lower(),
            operand: lower(x, nodes),
        },
        T::Hist(i, x) => K::Historically {
            interval: i.lower(),
            operand: lower(x, nodes),
        },
        T::Since(i, x, y) => K::Since {
            interval: i.lower(),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Trig(i, x, y) => K::Triggered {
            interval: i.lower(),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Y(x) => K::StrongPrevious {
            operand: lower(x, nodes),
        },
    };
    nodes.push(InfiniteNode::new(kind));
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

/// A claim and its fairness premises in one validated graph.
pub struct Built {
    pub doc: InfiniteFormulaDocument,
    pub claim: NodeId,
    pub fairness: Vec<NodeId>,
}

pub fn build(claim: &T, fairness: &[T]) -> Built {
    let mut nodes = Vec::new();
    let root = lower(claim, &mut nodes);
    let roots: Vec<NodeId> = fairness.iter().map(|t| lower(t, &mut nodes)).collect();
    let doc = InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        root,
        nodes,
    )
    .unwrap();
    Built {
        doc,
        claim: root,
        fairness: roots,
    }
}

pub type Cell = [PartialValue; 2];

pub fn lasso(prefix: &[Cell], tail: &[Cell]) -> LassoTraceDocument {
    let props = vec![PropositionId(0), PropositionId(1)];
    let observation = |index: usize, values: Cell| TraceObservation {
        position: u32::try_from(index).unwrap(),
        valuation: PartialValuation::new(
            "map".into(),
            &props,
            props
                .iter()
                .zip(values)
                .map(|(proposition, value)| ValuationEntry {
                    proposition: *proposition,
                    value,
                })
                .collect(),
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".into(),
        props.clone(),
        prefix
            .iter()
            .enumerate()
            .map(|(i, v)| observation(i, *v))
            .collect(),
        tail.iter()
            .enumerate()
            .map(|(i, v)| observation(prefix.len() + i, *v))
            .collect(),
    )
    .unwrap()
}

/// Prefix and loop cells of a trace, in order.
pub fn cells(word: &LassoTraceDocument) -> (Vec<Cell>, Vec<Cell>) {
    let row = |o: &TraceObservation| -> Cell {
        let e = o.valuation.entries();
        [e[0].value, e[1].value]
    };
    (
        word.prefix().iter().map(row).collect(),
        word.loop_observations().iter().map(row).collect(),
    )
}

pub fn provider_with(
    built: &Built,
    word: &LassoTraceDocument,
    position: u64,
    closure: EvidenceClosure,
    limit: EvaluationLimit,
) -> InfiniteResult {
    let graph_id = built.doc.content_identity().unwrap();
    let trace_id = word.content_identity().unwrap();
    let fairness = (!built.fairness.is_empty()).then(|| {
        FairnessPremisesDocument::new(
            &built.doc,
            graph_id.clone(),
            InfiniteClock::EventPosition,
            built.fairness.clone(),
        )
        .unwrap()
    });
    evaluate_lasso(&LassoRequest {
        formula: &built.doc,
        trace: word,
        fairness: fairness.as_ref(),
        evidence_closure: closure,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: position,
        limit,
    })
    .unwrap()
}

pub fn provider(built: &Built, word: &LassoTraceDocument, position: u64) -> InfiniteResult {
    provider_with(
        built,
        word,
        position,
        EvidenceClosure::Closed,
        EvaluationLimit::default(),
    )
}

/// The successor-fixpoint reference (`tl_oracle::fixpoint`).
pub fn fixpoint(
    built: &Built,
    word: &LassoTraceDocument,
    position: u64,
) -> Result<Outcome, OracleError> {
    let formula = serde_json::from_slice(&built.doc.canonical_json_bytes().unwrap())
        .expect("oracle syntax admits formula wire");
    let trace = serde_json::from_slice(&serde_json::to_vec(word).unwrap())
        .expect("oracle syntax admits lasso wire");
    let claim = serde_json::from_value(serde_json::to_value(built.claim).unwrap()).unwrap();
    let fairness: Vec<_> = built
        .fairness
        .iter()
        .map(|n| serde_json::from_value(serde_json::to_value(n).unwrap()).unwrap())
        .collect();
    tl_oracle::fixpoint::decide_documents(
        &formula,
        &trace,
        claim,
        &fairness,
        usize::try_from(position).unwrap(),
        Limits::default(),
    )
}

/// The offset-scanning reference in the crate root of `tl-oracle`.
pub fn scanning(
    built: &Built,
    word: &LassoTraceDocument,
    position: u64,
) -> Result<Outcome, OracleError> {
    let formula = serde_json::from_slice(&built.doc.canonical_json_bytes().unwrap())
        .expect("oracle syntax admits formula wire");
    let trace = serde_json::from_slice(&serde_json::to_vec(word).unwrap())
        .expect("oracle syntax admits lasso wire");
    let claim = serde_json::from_value(serde_json::to_value(built.claim).unwrap()).unwrap();
    let fairness: Vec<_> = built
        .fairness
        .iter()
        .map(|n| serde_json::from_value(serde_json::to_value(n).unwrap()).unwrap())
        .collect();
    tl_oracle::evaluate_documents(
        &formula,
        &trace,
        claim,
        &fairness,
        usize::try_from(position).unwrap(),
        Limits::default(),
    )
}

/// Maps a reference outcome onto what the provider must report, and returns a
/// description of the first difference, if any.
pub fn disagreement(
    actual: &InfiniteResult,
    expected: &Result<Outcome, OracleError>,
) -> Option<String> {
    let Ok(out) = expected else {
        return match expected {
            Err(OracleError::EmptyFairAdmission) => {
                if actual.disposition == Disposition::Inconclusive
                    && actual.admitted_completions == 0
                    && actual.reason == Some(ResultReason::EmptyFairAdmission)
                {
                    None
                } else {
                    Some(format!("empty fair admission, provider said {actual:?}"))
                }
            }
            Err(e) => Some(format!("reference refused: {e:?}")),
            Ok(_) => unreachable!(),
        };
    };
    let disposition = match out.verdict {
        Verdict::Proved => Disposition::Proved,
        Verdict::Refuted => Disposition::Refuted,
        Verdict::Inconclusive => Disposition::Inconclusive,
    };
    if actual.disposition != disposition {
        return Some(format!(
            "disposition {:?} != {disposition:?}",
            actual.disposition
        ));
    }
    if actual.admitted_completions != out.fair_completions as u64 {
        return Some(format!(
            "admitted {} != {}",
            actual.admitted_completions, out.fair_completions
        ));
    }
    let (sat, fal) = match &actual.evidence {
        Some(SettlementEvidence::ExhaustiveTrace(t)) => {
            (t.satisfying.is_some(), t.falsifying.is_some())
        }
        _ => (false, false),
    };
    if sat != out.possible_true || fal != out.possible_false {
        return Some(format!(
            "witness flags ({sat},{fal}) != ({},{})",
            out.possible_true, out.possible_false
        ));
    }
    if disposition == Disposition::Inconclusive {
        let reason = match out.uncertainty {
            Uncertainty::None => None,
            Uncertainty::Missing => Some(ResultReason::MissingObservation),
            Uncertainty::Conflicting => Some(ResultReason::ConflictingObservation),
            Uncertainty::Mixed => Some(ResultReason::MissingAndConflictingObservations),
        };
        if actual.reason != reason {
            return Some(format!("reason {:?} != {reason:?}", actual.reason));
        }
    }
    None
}
