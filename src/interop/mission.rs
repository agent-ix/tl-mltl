//! Mission-bounding report (FR-090), mission-bound closure refusal, closure
//! report and the computed known-case table (FR-094, ADR-045).
//!
//! Decision 4 (2026-11-23): refusal and report only. No closure conversion is
//! built, `ClosurePolicy` is not introduced, and no code path returns a bounded
//! formula derived from an unbounded one. `export_safety_monitor` is untouched.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

use crate::horizon::lookahead;
use crate::ltlf::{evaluate_ltlf, LtlfLimits};
use crate::{evaluate_closed_at, EvaluationLimits, HorizonError, TruthValue};

/// The operators of the known-case table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClosureOperator {
    /// `F` closed to `F[0,M]`.
    Future,
    /// `G` closed to `G[0,M]`.
    Globally,
    /// `U` closed to `U[0,M]`.
    Until,
    /// `R` closed to `R[0,M]`.
    Release,
    /// `Xs` closed to `F[1,1]`.
    StrongNext,
    /// `Xw` closed to `G[1,1]`.
    WeakNext,
}

impl ClosureOperator {
    /// The closure the table assumes.
    pub fn assumed_closure(self, mission_time: u32) -> String {
        match self {
            Self::Future => format!("F[0,{mission_time}]"),
            Self::Globally => format!("G[0,{mission_time}]"),
            Self::Until => format!("U[0,{mission_time}]"),
            Self::Release => format!("R[0,{mission_time}]"),
            Self::StrongNext => "F[1,1]".to_owned(),
            Self::WeakNext => "G[1,1]".to_owned(),
        }
    }

    const fn depends_on_mission_time(self) -> bool {
        !matches!(self, Self::StrongNext | Self::WeakNext)
    }
}

/// How often a closure agreed with the LTLf verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnownCaseClass {
    /// Agreed on every compared trace.
    AgreesOnAll,
    /// Agreed on some traces: the closure changes meaning there.
    AgreesOnSome,
    /// Agreed on no trace.
    AgreesOnNone,
}

/// One row of the computed known-case table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnownCaseRow {
    /// Operator.
    pub operator: ClosureOperator,
    /// Mission time, `None` for the next operators.
    pub mission_time: Option<u32>,
    /// Comparisons made.
    pub comparisons: u64,
    /// Comparisons that disagreed.
    pub disagreements: u64,
    /// Classification.
    pub class: KnownCaseClass,
}

#[derive(Clone, Copy)]
enum Operand {
    Atom(u32),
    True,
    False,
}

const OPERANDS: [Operand; 4] = [Operand::Atom(0), Operand::Atom(1), Operand::True, Operand::False];

fn operand_node(operand: Operand) -> NodeKind {
    match operand {
        Operand::Atom(n) => NodeKind::Proposition {
            proposition: PropositionId(n),
        },
        Operand::True => NodeKind::True,
        Operand::False => NodeKind::False,
    }
}

/// (LTLf graph, closed-trace closure graph) for one depth-one formula.
fn pair(
    operator: ClosureOperator,
    left: Operand,
    right: Operand,
    mission_time: u32,
) -> (Vec<Node>, Vec<Node>) {
    let mut ltlf = vec![Node::new(operand_node(left)), Node::new(operand_node(right))];
    let mut closed = ltlf.clone();
    let (l, r) = (NodeId(0), NodeId(1));
    let window = Interval::new(0, mission_time).expect("0 <= M");
    let one = Interval::new(1, 1).expect("[1,1]");
    let (ltlf_kind, closed_kind) = match operator {
        ClosureOperator::Future => (
            NodeKind::LtlfFuture { operand: r },
            NodeKind::Future {
                interval: window,
                operand: r,
            },
        ),
        ClosureOperator::Globally => (
            NodeKind::LtlfGlobally { operand: r },
            NodeKind::Globally {
                interval: window,
                operand: r,
            },
        ),
        ClosureOperator::Until => (
            NodeKind::LtlfUntil { left: l, right: r },
            NodeKind::Until {
                interval: window,
                left: l,
                right: r,
            },
        ),
        ClosureOperator::Release => (
            NodeKind::LtlfRelease { left: l, right: r },
            NodeKind::Release {
                interval: window,
                left: l,
                right: r,
            },
        ),
        ClosureOperator::StrongNext => (
            NodeKind::StrongNext { operand: r },
            NodeKind::Future {
                interval: one,
                operand: r,
            },
        ),
        ClosureOperator::WeakNext => (
            NodeKind::WeakNext { operand: r },
            NodeKind::Globally {
                interval: one,
                operand: r,
            },
        ),
    };
    ltlf.push(Node::new(ltlf_kind));
    closed.push(Node::new(closed_kind));
    (ltlf, closed)
}

/// The 340 traces of length 1 to 4 over two atoms.
pub fn two_atom_traces() -> Vec<Vec<Vec<PropositionId>>> {
    let mut all = Vec::with_capacity(340);
    for length in 1..=4u32 {
        for code in 0..4usize.pow(length) {
            let mut rest = code;
            let mut trace = Vec::with_capacity(length as usize);
            for _ in 0..length {
                let letter = rest % 4;
                rest /= 4;
                trace.push(
                    (0..2u32)
                        .filter(|bit| letter >> bit & 1 == 1)
                        .map(PropositionId)
                        .collect(),
                );
            }
            all.push(trace);
        }
    }
    all
}

/// Computes the known-case table: 40 depth-one temporal formulas x
/// `M in {1, 2}` x 340 traces (27,200) plus `Xs`/`Xw` on 4 operands x 340
/// traces (2,720); 29,920 comparisons of the LTLf verdict with the
/// closed-trace verdict of the closure.
///
/// Implements: FR-094 (known-case table)
pub fn known_case_table() -> Vec<KnownCaseRow> {
    let traces = two_atom_traces();
    let mut rows = Vec::new();
    let cases: [(ClosureOperator, Option<u32>); 10] = [
        (ClosureOperator::Future, Some(1)),
        (ClosureOperator::Future, Some(2)),
        (ClosureOperator::Globally, Some(1)),
        (ClosureOperator::Globally, Some(2)),
        (ClosureOperator::Until, Some(1)),
        (ClosureOperator::Until, Some(2)),
        (ClosureOperator::Release, Some(1)),
        (ClosureOperator::Release, Some(2)),
        (ClosureOperator::StrongNext, None),
        (ClosureOperator::WeakNext, None),
    ];
    for (operator, mission_time) in cases {
        let binary = matches!(operator, ClosureOperator::Until | ClosureOperator::Release);
        let lefts: &[Operand] = if binary { &OPERANDS } else { &OPERANDS[..1] };
        let mut comparisons = 0u64;
        let mut disagreements = 0u64;
        for left in lefts {
            for right in OPERANDS {
                let (ltlf_nodes, closed_nodes) =
                    pair(operator, *left, right, mission_time.unwrap_or(0));
                let root = NodeId(2);
                let ltlf = Formula::new(SemanticProfile::LtlfFiniteTraceV1, root, &ltlf_nodes)
                    .expect("depth-one LTLf formula");
                let closed = Formula::new(SemanticProfile::ClosedTraceV1, root, &closed_nodes)
                    .expect("depth-one closed formula");
                for trace in &traces {
                    let finite = evaluate_ltlf(ltlf, trace, LtlfLimits::default())
                        .expect("nonempty bounded trace");
                    let bounded = evaluate_closed_at(
                        closed,
                        "closure",
                        trace,
                        "known-case",
                        0,
                        EvaluationLimits::default(),
                    )
                    .expect("small closed evaluation")
                    .verdict
                        == TruthValue::True;
                    comparisons += 1;
                    if finite != bounded {
                        disagreements += 1;
                    }
                }
            }
        }
        let class = if disagreements == 0 {
            KnownCaseClass::AgreesOnAll
        } else if disagreements == comparisons {
            KnownCaseClass::AgreesOnNone
        } else {
            KnownCaseClass::AgreesOnSome
        };
        rows.push(KnownCaseRow {
            operator,
            mission_time,
            comparisons,
            disagreements,
            class,
        });
    }
    rows
}

/// The known-case row for `operator` at `mission_time`, if the table has one.
pub fn known_case_row(
    table: &[KnownCaseRow],
    operator: ClosureOperator,
    mission_time: u32,
) -> Option<KnownCaseRow> {
    table.iter().copied().find(|row| {
        row.operator == operator
            && (!operator.depends_on_mission_time() || row.mission_time == Some(mission_time))
    })
}

// ------------------------------------------------------------ closure API

/// Closure refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosureError {
    /// Turning an unbounded formula into a bounded one needs an explicit
    /// policy, which this build does not provide (decision 4).
    MissionClosureRequired,
    /// The mission time does not fit `u32`.
    MissionTimeOverflow,
    /// The formula is not LTLf (or infinite-trace).
    UnsupportedProfile {
        /// Profile.
        actual: &'static str,
    },
}

impl fmt::Display for ClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissionClosureRequired => write!(
                formatter,
                "mission-bound closure changes meaning and is not performed implicitly"
            ),
            Self::MissionTimeOverflow => write!(formatter, "mission time exceeds u32"),
            Self::UnsupportedProfile { actual } => {
                write!(formatter, "closure report does not apply to {actual}")
            }
        }
    }
}

impl std::error::Error for ClosureError {}

/// Where a closure row's known-case reference comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnownCaseReference {
    /// The computed table row.
    Row(KnownCaseRow),
    /// The table compares LTLf with closed-trace only, at `M in {1, 2}`.
    NotComputed,
}

/// One unbounded temporal node of the source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureRow {
    /// Node identity.
    pub node: NodeId,
    /// Operator.
    pub operator: ClosureOperator,
    /// Closure the table assumes.
    pub assumed_closure: String,
    /// Known-case reference.
    pub known_case: KnownCaseReference,
    /// Always the same statement: the meaning may change.
    pub statement: &'static str,
}

/// Closure report. It converts nothing and carries no bounded formula.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureReport {
    /// Source profile.
    pub source_profile: &'static str,
    /// Mission time.
    pub mission_time: u32,
    /// One row per unbounded temporal node, in node order.
    pub rows: Vec<ClosureRow>,
}

const MAY_CHANGE: &str = "closing this operator at the mission time may change its meaning";

/// Reports, per unbounded node of an LTLf formula, the closure the known-case
/// table assumes and its row. Converts nothing.
///
/// Implements: FR-094 (closure report)
pub fn mission_closure_report(
    formula: Formula<'_>,
    mission_time: u64,
) -> Result<ClosureReport, ClosureError> {
    let mission_time = u32::try_from(mission_time).map_err(|_| ClosureError::MissionTimeOverflow)?;
    if formula.profile() != SemanticProfile::LtlfFiniteTraceV1 {
        return Err(ClosureError::UnsupportedProfile {
            actual: formula.profile().as_str(),
        });
    }
    let table = known_case_table();
    let mut rows = Vec::new();
    for (index, node) in formula.nodes().iter().enumerate() {
        let operator = match node.kind {
            NodeKind::LtlfFuture { .. } => ClosureOperator::Future,
            NodeKind::LtlfGlobally { .. } => ClosureOperator::Globally,
            NodeKind::LtlfUntil { .. } => ClosureOperator::Until,
            NodeKind::LtlfRelease { .. } => ClosureOperator::Release,
            NodeKind::StrongNext { .. } => ClosureOperator::StrongNext,
            NodeKind::WeakNext { .. } => ClosureOperator::WeakNext,
            NodeKind::False
            | NodeKind::True
            | NodeKind::Proposition { .. }
            | NodeKind::Not { .. }
            | NodeKind::And { .. }
            | NodeKind::Or { .. }
            | NodeKind::Implies { .. }
            | NodeKind::Equivalent { .. }
            | NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. }
            | NodeKind::Once { .. }
            | NodeKind::Historically { .. }
            | NodeKind::StrongPrevious { .. }
            | NodeKind::Since { .. }
            | NodeKind::Triggered { .. } => continue,
        };
        rows.push(ClosureRow {
            node: NodeId(index as u32),
            operator,
            assumed_closure: operator.assumed_closure(mission_time),
            known_case: known_case_row(&table, operator, mission_time)
                .map_or(KnownCaseReference::NotComputed, KnownCaseReference::Row),
            statement: MAY_CHANGE,
        });
    }
    Ok(ClosureReport {
        source_profile: formula.profile().as_str(),
        mission_time,
        rows,
    })
}

/// Closure report for an infinite-trace formula: one `NotComputed` row per
/// unbounded `F G U R` node (the table compares LTLf with closed-trace only).
///
/// Implements: FR-094 (closure report, infinite-trace source)
#[cfg(feature = "infinite-trace")]
pub fn mission_closure_report_infinite(
    formula: tl_syntax::InfiniteFormula<'_>,
    mission_time: u64,
) -> Result<ClosureReport, ClosureError> {
    use tl_syntax::{InfiniteNodeKind as K, TemporalInterval};
    let mission_time = u32::try_from(mission_time).map_err(|_| ClosureError::MissionTimeOverflow)?;
    let mut rows = Vec::new();
    for (index, node) in formula.nodes().iter().enumerate() {
        let (operator, interval) = match node.kind {
            K::Future { interval, .. } => (ClosureOperator::Future, interval),
            K::Globally { interval, .. } => (ClosureOperator::Globally, interval),
            K::Until { interval, .. } => (ClosureOperator::Until, interval),
            K::Release { interval, .. } => (ClosureOperator::Release, interval),
            _ => continue,
        };
        if let TemporalInterval::Unbounded(_) = interval {
            rows.push(ClosureRow {
                node: NodeId(index as u32),
                operator,
                assumed_closure: operator.assumed_closure(mission_time),
                known_case: KnownCaseReference::NotComputed,
                statement: MAY_CHANGE,
            });
        }
    }
    Ok(ClosureReport {
        source_profile: "mltl.infinite-trace/v1",
        mission_time,
        rows,
    })
}

/// The one entry point that would convert an unbounded formula at a mission
/// time. It always refuses and emits no artifact (decision 4: no conversion
/// is built; ADR-045 option A).
///
/// Implements: FR-094 (refusal)
pub fn request_mission_closure(
    _formula: Formula<'_>,
    _mission_time: u64,
) -> Result<core::convert::Infallible, ClosureError> {
    Err(ClosureError::MissionClosureRequired)
}

// ------------------------------------------------------- mission report

/// Report class of one imported requirement (not exclusive).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MissionClass {
    /// Some temporal node's upper bound equals `M`.
    BoundEqualsMission,
    /// `H > M`: undecided at mission end.
    HorizonExceedsMission,
    /// Neither of the above.
    WithinMission,
}

/// Citation of the known-case table for a flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnownCaseCitation {
    /// Operator of the flagged node (or of the root for horizon flags).
    pub operator: Option<ClosureOperator>,
    /// The cited row.
    pub reference: KnownCaseReference,
}

/// One requirement's report row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissionReportRow {
    /// Requirement identity.
    pub requirement_id: String,
    /// Root node.
    pub root: NodeId,
    /// Horizon `H`.
    pub horizon: u64,
    /// Classes, sorted; empty when the mission time is absent.
    pub classes: Vec<MissionClass>,
    /// Citations for flagged classes.
    pub citations: Vec<KnownCaseCitation>,
    /// Never "changed": a flagged requirement was possibly changed upstream.
    pub wording: &'static str,
}

/// Whether a mission time was available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissionTimeStatus {
    /// Mission time supplied.
    Present(u32),
    /// `mission-time-absent`: no divergence claim is made.
    Absent,
}

/// FR-090 report.
// SPEC-GAP: FR-090 asks for a SHA-256 of the report's own canonical bytes;
// decision 7 (2026-11-23) forbids digests outside the ADR-019 / tl-solve
// ADR-004 register, so the report carries none.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissionReport {
    /// Mission time status.
    pub mission_time: MissionTimeStatus,
    /// One row per requirement.
    pub rows: Vec<MissionReportRow>,
}

/// Report refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissionReportError {
    /// The horizon overflowed (refused, never wrapped).
    HorizonOverflow {
        /// Requirement index.
        requirement: usize,
    },
    /// A requirement is not a bounded future formula.
    UnsupportedProfile {
        /// Requirement index.
        requirement: usize,
    },
}

fn bounded_operator(kind: NodeKind) -> Option<(ClosureOperator, u32)> {
    match kind {
        NodeKind::Future { interval, .. } => Some((ClosureOperator::Future, interval.end())),
        NodeKind::Globally { interval, .. } => Some((ClosureOperator::Globally, interval.end())),
        NodeKind::Until { interval, .. } => Some((ClosureOperator::Until, interval.end())),
        NodeKind::Release { interval, .. } => Some((ClosureOperator::Release, interval.end())),
        _ => None,
    }
}

/// Classifies imported bounded requirements against a mission time.
///
/// Implements: FR-090
pub fn mission_bounding_report(
    requirements: &[(&str, Formula<'_>)],
    mission_time: Option<u32>,
) -> Result<MissionReport, MissionReportError> {
    let table = mission_time.map(|_| known_case_table());
    let mut rows = Vec::with_capacity(requirements.len());
    for (index, (id, formula)) in requirements.iter().enumerate() {
        if !matches!(
            formula.profile(),
            SemanticProfile::ClosedTraceV1 | SemanticProfile::OnlinePrefixV1
        ) {
            return Err(MissionReportError::UnsupportedProfile { requirement: index });
        }
        let horizon = lookahead(*formula).map_err(|error| match error {
            HorizonError::ArithmeticOverflow { .. } => {
                MissionReportError::HorizonOverflow { requirement: index }
            }
            HorizonError::UnsupportedProfile { .. } | HorizonError::UnboundedByTraceLength => {
                MissionReportError::UnsupportedProfile { requirement: index }
            }
        })?;
        let mut classes = Vec::new();
        let mut citations = Vec::new();
        if let (Some(m), Some(table)) = (mission_time, table.as_deref()) {
            if let Some((operator, _)) = formula
                .nodes()
                .iter()
                .filter_map(|node| bounded_operator(node.kind))
                .find(|(_, end)| *end == m)
            {
                classes.push(MissionClass::BoundEqualsMission);
                citations.push(KnownCaseCitation {
                    operator: Some(operator),
                    reference: known_case_row(table, operator, m)
                        .map_or(KnownCaseReference::NotComputed, KnownCaseReference::Row),
                });
            }
            if horizon > u64::from(m) {
                classes.push(MissionClass::HorizonExceedsMission);
                let root_operator = formula
                    .node(formula.root())
                    .and_then(|node| bounded_operator(node.kind))
                    .map(|(operator, _)| operator);
                citations.push(KnownCaseCitation {
                    operator: root_operator,
                    reference: root_operator
                        .and_then(|operator| known_case_row(table, operator, m))
                        .map_or(KnownCaseReference::NotComputed, KnownCaseReference::Row),
                });
            }
            if classes.is_empty() {
                classes.push(MissionClass::WithinMission);
            }
        }
        let flagged = classes
            .iter()
            .any(|class| *class != MissionClass::WithinMission);
        rows.push(MissionReportRow {
            requirement_id: (*id).to_owned(),
            root: formula.root(),
            horizon,
            classes,
            citations,
            wording: if mission_time.is_none() {
                "mission-time-absent: no divergence claim"
            } else if flagged {
                "possibly changed upstream"
            } else {
                "within mission"
            },
        });
    }
    Ok(MissionReport {
        mission_time: mission_time.map_or(MissionTimeStatus::Absent, MissionTimeStatus::Present),
        rows,
    })
}
