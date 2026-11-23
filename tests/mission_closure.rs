//! TL-431: mission-bounding report (FR-090) and mission-bound closure refusal,
//! known-case table and closure report (FR-094).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! TC-333 is withdrawn: decision 4 (2026-11-23) builds no closure conversion.

use tl_mltl::interop::c2po::{import_c2po, import_c2po_requirements, C2poImportError};
use tl_mltl::interop::fret::{import_fret_mltl, FretProfile};
use tl_mltl::interop::mission::{
    known_case_table, mission_bounding_report, mission_closure_report, request_mission_closure,
    ClosureError, ClosureOperator, KnownCaseClass, KnownCaseReference, MissionClass,
    MissionTimeStatus,
};
use tl_mltl::interop::{Attribution, LossKind, ReaderLimits};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

// ------------------------------------------------------ closed-trace trees

#[derive(Clone)]
enum B {
    P(u32),
    F(u32, u32, Box<B>),
    G(u32, u32, Box<B>),
    U(u32, u32, Box<B>, Box<B>),
    R(u32, u32, Box<B>, Box<B>),
}

fn p() -> Box<B> {
    Box::new(B::P(0))
}

fn q() -> Box<B> {
    Box::new(B::P(1))
}

fn lower(f: &B, nodes: &mut Vec<Node>) -> NodeId {
    let iv = |a: u32, b: u32| Interval::new(a, b).unwrap();
    let kind = match f {
        B::P(n) => NodeKind::Proposition {
            proposition: PropositionId(*n),
        },
        B::F(a, b, x) => NodeKind::Future {
            interval: iv(*a, *b),
            operand: lower(x, nodes),
        },
        B::G(a, b, x) => NodeKind::Globally {
            interval: iv(*a, *b),
            operand: lower(x, nodes),
        },
        B::U(a, b, x, y) => {
            let left = lower(x, nodes);
            let right = lower(y, nodes);
            NodeKind::Until {
                interval: iv(*a, *b),
                left,
                right,
            }
        }
        B::R(a, b, x, y) => {
            let left = lower(x, nodes);
            let right = lower(y, nodes);
            NodeKind::Release {
                interval: iv(*a, *b),
                left,
                right,
            }
        }
    };
    nodes.push(Node::new(kind));
    NodeId((nodes.len() - 1) as u32)
}

/// The 11-formula population at M = 5 and its hand-recorded manifest.
fn population() -> Vec<(&'static str, B, u64, Vec<MissionClass>)> {
    use MissionClass::{BoundEqualsMission as Bem, HorizonExceedsMission as Hem, WithinMission as Win};
    vec![
        ("bem-1", B::G(0, 5, p()), 5, vec![Bem]),
        ("bem-2", B::F(2, 5, p()), 5, vec![Bem]),
        ("bem-3", B::U(0, 5, p(), q()), 5, vec![Bem]),
        ("both", B::G(0, 5, Box::new(B::F(0, 1, p()))), 6, vec![Bem, Hem]),
        ("hem-1", B::G(0, 6, p()), 6, vec![Hem]),
        ("hem-2", B::F(0, 3, Box::new(B::G(0, 3, p()))), 6, vec![Hem]),
        ("hem-3", B::R(1, 7, p(), q()), 7, vec![Hem]),
        ("win-1", B::G(0, 4, p()), 4, vec![Win]),
        ("win-2", B::F(0, 2, p()), 2, vec![Win]),
        ("win-3", B::U(0, 1, p(), q()), 1, vec![Win]),
        ("win-4", B::P(0), 0, vec![Win]),
    ]
}

fn lowered() -> Vec<(&'static str, Vec<Node>, NodeId)> {
    population()
        .into_iter()
        .map(|(id, tree, _, _)| {
            let mut nodes = Vec::new();
            let root = lower(&tree, &mut nodes);
            (id, nodes, root)
        })
        .collect()
}

// Trace: TC-318, FR-090-AC-1
#[test]
fn mission_report_population_matches_its_manifest() {
    let population = population();
    let graphs = lowered();
    let requirements: Vec<(&str, Formula<'_>)> = graphs
        .iter()
        .map(|(id, nodes, root)| {
            (*id, Formula::new(SemanticProfile::ClosedTraceV1, *root, nodes).unwrap())
        })
        .collect();
    let report = mission_bounding_report(&requirements, Some(5)).unwrap();
    assert_eq!(report.mission_time, MissionTimeStatus::Present(5));
    assert_eq!(report.rows.len(), 11);
    let mut assignments = 0;
    for ((_, _, horizon, classes), row) in population.iter().zip(&report.rows) {
        assert_eq!(row.horizon, *horizon, "{}", row.requirement_id);
        assert_eq!(&row.classes, classes, "{}", row.requirement_id);
        assignments += row.classes.len();
        if row.classes.contains(&MissionClass::WithinMission) {
            assert_eq!(row.wording, "within mission");
        } else {
            assert_eq!(row.wording, "possibly changed upstream");
            assert!(!row.wording.contains("changed upstream") || row.wording.starts_with("possibly"));
        }
    }
    assert_eq!(assignments, 12);
    for class in [
        MissionClass::BoundEqualsMission,
        MissionClass::HorizonExceedsMission,
        MissionClass::WithinMission,
    ] {
        assert_eq!(
            report.rows.iter().filter(|row| row.classes.contains(&class)).count(),
            4
        );
    }
}

// Trace: TC-319, FR-090-AC-2
#[test]
fn absent_mission_time_makes_no_divergence_claim() {
    let graphs = lowered();
    let requirements: Vec<(&str, Formula<'_>)> = graphs
        .iter()
        .map(|(id, nodes, root)| {
            (*id, Formula::new(SemanticProfile::ClosedTraceV1, *root, nodes).unwrap())
        })
        .collect();
    let report = mission_bounding_report(&requirements, None).unwrap();
    assert_eq!(report.mission_time, MissionTimeStatus::Absent);
    for row in &report.rows {
        assert!(row.classes.is_empty());
        assert!(row.citations.is_empty());
        assert!(row.wording.starts_with("mission-time-absent"));
    }
    // SPEC-GAP: an overflowing horizon needs about 2^32 nested u32-bounded
    // nodes and cannot be built in a test; the refusal arm maps
    // `HorizonError::ArithmeticOverflow` to `MissionReportError::HorizonOverflow`
    // (inspection only).
}

// Trace: TC-327, FR-094-AC-1
#[test]
fn unbounded_operators_are_refused_by_every_importer_and_the_closure_api() {
    let attribution = Attribution {
        repository: Some("https://github.com/NASA-SW-VnV/fret".to_owned()),
        path: Some("examples".to_owned()),
        revision: Some("0123456789abcdef".to_owned()),
        licence: Some("Apache-2.0".to_owned()),
    };
    let fret = import_fret_mltl(
        b"G (p -> F q)\n",
        FretProfile::ClosedTrace,
        &attribution,
        ReaderLimits::default(),
    )
    .unwrap();
    assert!(fret.imported.is_empty());
    assert_eq!(fret.unimported[0].kind, LossKind::MissionClosureRequired);

    let c2po = "INPUT\n  a: bool;\nFTSPEC\n  G a;\n";
    assert!(matches!(
        import_c2po(c2po.as_bytes(), Some(SemanticProfile::OnlinePrefixV1), ReaderLimits::default()),
        Err(C2poImportError::MissionClosureRequired { .. })
    ));
    let per_requirement = import_c2po_requirements(
        c2po.as_bytes(),
        Some(SemanticProfile::OnlinePrefixV1),
        ReaderLimits::default(),
    )
    .unwrap();
    assert_eq!(per_requirement.unimported[0].kind, LossKind::MissionClosureRequired);
    assert!(per_requirement.imported.is_empty());

    let nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::LtlfGlobally { operand: NodeId(0) }),
    ];
    let ltlf = Formula::new(SemanticProfile::LtlfFiniteTraceV1, NodeId(1), &nodes).unwrap();
    assert_eq!(
        request_mission_closure(ltlf, 5),
        Err(ClosureError::MissionClosureRequired)
    );
    // `export_safety_monitor` is unchanged by this ticket: `src/infinite/export.rs`
    // is not modified and its behavior stays covered by `tests/infinite_export.rs`.
    // The WEST importer's refusal is in `tests/west_interop.rs` (TL-433).
}

/// Independent enumerator: LTLf and closed-trace on bitmask traces.
fn independent_row(operator: ClosureOperator, mission: u32) -> (u64, u64) {
    let atoms = [Some(0u32), Some(1), None, None];
    let constant = [false, false, true, false];
    let value = |operand: usize, letter: u8| match atoms[operand] {
        Some(bit) => letter >> bit & 1 == 1,
        None => constant[operand],
    };
    let mut comparisons = 0;
    let mut disagreements = 0;
    let binary = matches!(operator, ClosureOperator::Until | ClosureOperator::Release);
    let lefts = if binary { 4 } else { 1 };
    for left in 0..lefts {
        for right in 0..4 {
            for length in 1..=4u32 {
                for code in 0..4usize.pow(length) {
                    let trace: Vec<u8> = (0..length)
                        .map(|k| ((code / 4usize.pow(k)) % 4) as u8)
                        .collect();
                    let n = trace.len();
                    let at = |op: usize, i: usize| i < n && value(op, trace[i]);
                    let ltlf = match operator {
                        ClosureOperator::Future => (0..n).any(|j| at(right, j)),
                        ClosureOperator::Globally => (0..n).all(|j| at(right, j)),
                        ClosureOperator::Until => {
                            (0..n).any(|j| at(right, j) && (0..j).all(|k| at(left, k)))
                        }
                        ClosureOperator::Release => {
                            (0..n).all(|j| at(right, j) || (0..j).any(|k| at(left, k)))
                        }
                        ClosureOperator::StrongNext => n > 1 && at(right, 1),
                        ClosureOperator::WeakNext => n == 1 || at(right, 1),
                    };
                    // Closed trace: positions >= n are all-false; constants keep value.
                    let padded = |op: usize, i: usize| {
                        if i < n {
                            value(op, trace[i])
                        } else {
                            atoms[op].is_none() && constant[op]
                        }
                    };
                    let m = mission as usize;
                    let closed = match operator {
                        ClosureOperator::Future => (0..=m).any(|j| padded(right, j)),
                        ClosureOperator::Globally => (0..=m).all(|j| padded(right, j)),
                        ClosureOperator::Until => {
                            (0..=m).any(|j| padded(right, j) && (0..j).all(|k| padded(left, k)))
                        }
                        ClosureOperator::Release => {
                            (0..=m).all(|j| padded(right, j) || (0..j).any(|k| padded(left, k)))
                        }
                        ClosureOperator::StrongNext => padded(right, 1),
                        ClosureOperator::WeakNext => padded(right, 1),
                    };
                    comparisons += 1;
                    if ltlf != closed {
                        disagreements += 1;
                    }
                }
            }
        }
    }
    (comparisons, disagreements)
}

// Trace: TC-328, FR-094-AC-2
#[test]
fn known_case_table_matches_an_independent_enumerator() {
    let table = known_case_table();
    let total: u64 = table.iter().map(|row| row.comparisons).sum();
    assert_eq!(total, 29_920);
    for row in &table {
        let (comparisons, disagreements) =
            independent_row(row.operator, row.mission_time.unwrap_or(0));
        assert_eq!((row.comparisons, row.disagreements), (comparisons, disagreements), "{row:?}");
    }
    let class = |operator, mission| {
        table
            .iter()
            .find(|row| row.operator == operator && row.mission_time == mission)
            .unwrap()
            .class
    };
    // G on a trace shorter than M + 1 changes meaning.
    assert_ne!(class(ClosureOperator::Globally, Some(1)), KnownCaseClass::AgreesOnAll);
    assert_ne!(class(ClosureOperator::StrongNext, None), KnownCaseClass::AgreesOnAll);
    assert_ne!(class(ClosureOperator::WeakNext, None), KnownCaseClass::AgreesOnAll);
}

// Trace: TC-329, FR-094-AC-3
#[test]
fn closure_report_has_one_row_per_unbounded_node_and_converts_nothing() {
    // 0 p, 1 q, 2 F p, 3 G q, 4 p U q, 5 p R q, 6 Xs p, 7 Xw q, then a conjunction chain.
    let mut nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
        Node::new(NodeKind::LtlfFuture { operand: NodeId(0) }),
        Node::new(NodeKind::LtlfGlobally { operand: NodeId(1) }),
        Node::new(NodeKind::LtlfUntil {
            left: NodeId(0),
            right: NodeId(1),
        }),
        Node::new(NodeKind::LtlfRelease {
            left: NodeId(0),
            right: NodeId(1),
        }),
        Node::new(NodeKind::StrongNext { operand: NodeId(0) }),
        Node::new(NodeKind::WeakNext { operand: NodeId(1) }),
    ];
    let mut acc = NodeId(2);
    for next in 3..8u32 {
        nodes.push(Node::new(NodeKind::And {
            left: acc,
            right: NodeId(next),
        }));
        acc = NodeId((nodes.len() - 1) as u32);
    }
    let formula = Formula::new(SemanticProfile::LtlfFiniteTraceV1, acc, &nodes).unwrap();
    let report = mission_closure_report(formula, 2).unwrap();
    let operators: Vec<ClosureOperator> = report.rows.iter().map(|row| row.operator).collect();
    assert_eq!(
        operators,
        vec![
            ClosureOperator::Future,
            ClosureOperator::Globally,
            ClosureOperator::Until,
            ClosureOperator::Release,
            ClosureOperator::StrongNext,
            ClosureOperator::WeakNext,
        ]
    );
    let closures: Vec<&str> = report.rows.iter().map(|row| row.assumed_closure.as_str()).collect();
    assert_eq!(closures, vec!["F[0,2]", "G[0,2]", "U[0,2]", "R[0,2]", "F[1,1]", "G[1,1]"]);
    for row in &report.rows {
        assert!(matches!(row.known_case, KnownCaseReference::Row(_)));
        assert!(row.statement.contains("may change"));
    }
    // A mission time outside the table cites no row; overflow refuses.
    let far = mission_closure_report(formula, 5).unwrap();
    assert!(matches!(far.rows[0].known_case, KnownCaseReference::NotComputed));
    assert!(matches!(far.rows[4].known_case, KnownCaseReference::Row(_)));
    assert_eq!(
        mission_closure_report(formula, u64::from(u32::MAX) + 1),
        Err(ClosureError::MissionTimeOverflow)
    );

    #[cfg(feature = "infinite-trace")]
    {
        use tl_mltl::interop::mission::mission_closure_report_infinite;
        use tl_syntax::{
            InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as K,
            TemporalInterval, UnboundedInterval,
        };
        let document = InfiniteFormulaDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            NodeId(1),
            vec![
                InfiniteNode::new(K::Proposition {
                    proposition: PropositionId(0),
                }),
                InfiniteNode::new(K::Globally {
                    interval: TemporalInterval::Unbounded(UnboundedInterval::new(0)),
                    operand: NodeId(0),
                }),
            ],
        )
        .unwrap();
        let infinite = mission_closure_report_infinite(document.formula(), 5).unwrap();
        assert_eq!(infinite.rows.len(), 1);
        assert_eq!(infinite.rows[0].known_case, KnownCaseReference::NotComputed);
    }
}
