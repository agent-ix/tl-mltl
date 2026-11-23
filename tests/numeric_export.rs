//! TL-423 (refusal path only) and TL-425 export half: numeric C2PO export is
//! refused until a target numeric profile is pinned (decision 5, 2026-11-23),
//! with a typed loss record; contracts export by decomposition or are refused
//! whole (FR-056, FR-057-AC-2, FR-060).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! Not written (no pinned or synthetic profile is built): TC-1581, TC-1585,
//! TC-1586, TC-1588 and the synthetic-profile half of TC-1596. Blocked on a
//! human R2U2 run: TC-1589, TC-1590, TC-1592, TC-1598, TC-1601, TC-1602.

use tl_mltl::contract::{ContractRoots, ContractSet};
use tl_mltl::interop::c2po::import_c2po;
use tl_mltl::interop::ReaderLimits;
use tl_mltl::numeric::{
    comparison_class, export_contracts_c2po, map_numeric_to_c2po, numeric_node_class, Comparison,
    ExprNode, NumericExportOutcome, NumericLossCause, NumericLossRecord, NumericLossSubject,
    PredicateDefinition, PredicateTable,
};
use tl_mltl::{map_to_c2po, map_to_c2po_with_context, MappingSourceIdentity, MappingSourceState};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

fn atom(n: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(n),
    })
}

/// Signals: 1 `a`, 2 `b` Boolean (propositions 0, 1); 3 `x` Integer [-2, 1];
/// 4 `d` FixedDecimal scale 1 [0, 3].
fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "a".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "b".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(
                SignalId(3),
                "x".to_owned(),
                SignalDomain::Integer(tl_syntax::IntegerSignalDomain::new(-2, 1).unwrap()),
            ),
            OwnedSignalDeclaration::new(
                SignalId(4),
                "d".to_owned(),
                SignalDomain::FixedDecimal(
                    tl_syntax::FixedDecimalSignalDomain::new(0, 3, 1).unwrap(),
                ),
            ),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap()
}

/// The FR-060 contract grid predicates: 10/11/12 are `x > -1`, `x > 0`,
/// `x > 1`; 20 is `d >= 0.2`; 21 is `d >= 0.1`.
fn grid_table() -> PredicateTable {
    let mut expressions = vec![ExprNode::Signal { signal: SignalId(3) }];
    let mut predicates = Vec::new();
    for (proposition, constant) in [(10u32, -1i64), (11, 0), (12, 1)] {
        expressions.push(ExprNode::Integer { value: constant });
        predicates.push(PredicateDefinition {
            proposition: PropositionId(proposition),
            comparison: Comparison::Gt,
            left: 0,
            right: (expressions.len() - 1) as u32,
        });
    }
    expressions.push(ExprNode::Signal { signal: SignalId(4) });
    let d = (expressions.len() - 1) as u32;
    for (proposition, coefficient) in [(20u32, 2i64), (21, 1)] {
        expressions.push(ExprNode::Decimal {
            coefficient,
            scale: 1,
        });
        predicates.push(PredicateDefinition {
            proposition: PropositionId(proposition),
            comparison: Comparison::Ge,
            left: d,
            right: (expressions.len() - 1) as u32,
        });
    }
    PredicateTable {
        expressions,
        predicates,
    }
}

fn single(nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        SemanticProfile::OnlinePrefixV1,
        NodeId((nodes.len() - 1) as u32),
        nodes,
    )
    .unwrap()
}

// Trace: TC-1583, FR-056-AC-2
#[test]
fn reachable_loss_causes_serialize_strictly_and_carry_no_expression() {
    let table = grid_table();
    let catalog = catalog();
    let numeric = vec![atom(11)];
    let unpinned = map_numeric_to_c2po(single(&numeric), &table, &catalog, 1_000).unwrap_err();
    assert_eq!(unpinned.cause, NumericLossCause::TargetNumericProfileUnpinned);
    assert_eq!(unpinned.subject, NumericLossSubject::Proposition(11));

    let undefined = vec![atom(7)];
    let missing = map_numeric_to_c2po(single(&undefined), &table, &catalog, 1_000).unwrap_err();
    assert_eq!(missing.cause, NumericLossCause::PredicateWithoutDefinition);

    let boolean = vec![atom(0)];
    let exhausted = map_numeric_to_c2po(single(&boolean), &table, &catalog, 0).unwrap_err();
    assert_eq!(exhausted.cause, NumericLossCause::ResourceExhausted);

    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &boolean).unwrap();
    let foreign = map_numeric_to_c2po(closed, &table, &catalog, 1_000).unwrap_err();
    assert_eq!(foreign.cause, NumericLossCause::ForeignIdentity);

    for record in [unpinned, missing, exhausted, foreign] {
        let bytes = record.to_json_bytes();
        assert_eq!(NumericLossRecord::from_json_bytes(&bytes).unwrap(), record);
        let mut spaced = bytes.clone();
        spaced.insert(1, b' ');
        assert!(NumericLossRecord::from_json_bytes(&spaced).is_err());
    }
    // TargetIntegerRangeExceeded, FloatMatchingUnsafe, RescaleBeyondTarget and
    // UnsupportedNumericOperator need a pinned profile and are unreachable in
    // this build (decision 5); they exist in the closed cause set only.
}

// Trace: TC-1584, FR-056-AC-3
#[test]
fn every_node_kind_and_comparison_has_exactly_one_class() {
    let kinds = [
        ExprNode::Signal { signal: SignalId(3) },
        ExprNode::Integer { value: 0 },
        ExprNode::Decimal {
            coefficient: 0,
            scale: 1,
        },
        ExprNode::Neg { operand: 0 },
        ExprNode::Abs { operand: 0 },
        ExprNode::Add { left: 0, right: 0 },
        ExprNode::Sub { left: 0, right: 0 },
        ExprNode::Mul { left: 0, right: 0 },
        ExprNode::Rescale {
            operand: 0,
            scale: 2,
        },
    ];
    for kind in kinds {
        assert_eq!(numeric_node_class(kind), NumericLossCause::TargetNumericProfileUnpinned);
    }
    for comparison in Comparison::ALL {
        assert_eq!(comparison_class(comparison), NumericLossCause::TargetNumericProfileUnpinned);
    }
    // SPEC-GAP: the FR-041 interval/profile context axis of the partition is
    // not crossed here; with no pinned profile every cell is the same refusal.
}

// Trace: TC-1587, TC-1582, FR-057-AC-2, FR-056-AC-1
#[test]
fn unpinned_profile_refuses_every_numeric_predicate_and_boolean_export_is_unchanged() {
    // 36 Integer + 36 Decimal predicates (6 operators x 6 constants each).
    let catalog = catalog();
    let mut refused = 0;
    for (signal, scale) in [(3u32, 0u8), (4, 1)] {
        for comparison in Comparison::ALL {
            for constant in -3i64..3 {
                let right = if scale == 0 {
                    ExprNode::Integer { value: constant }
                } else {
                    ExprNode::Decimal {
                        coefficient: constant,
                        scale,
                    }
                };
                let table = PredicateTable {
                    expressions: vec![ExprNode::Signal { signal: SignalId(signal) }, right],
                    predicates: vec![PredicateDefinition {
                        proposition: PropositionId(30),
                        comparison,
                        left: 0,
                        right: 1,
                    }],
                };
                let nodes = vec![
                    atom(30),
                    Node::new(NodeKind::Future {
                        interval: Interval::new(0, 1).unwrap(),
                        operand: NodeId(0),
                    }),
                ];
                let record = map_numeric_to_c2po(single(&nodes), &table, &catalog, 1_000).unwrap_err();
                assert_eq!(record.cause, NumericLossCause::TargetNumericProfileUnpinned);
                refused += 1;
            }
        }
    }
    assert_eq!(refused, 72);

    // Pure Boolean export: same expression as the existing contextual mapping,
    // and the legacy manifest renderer is untouched.
    let nodes = vec![
        atom(0),
        atom(1),
        Node::new(NodeKind::Until {
            interval: Interval::new(1, 2).unwrap(),
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let formula = single(&nodes);
    let NumericExportOutcome::Boolean { expression } =
        map_numeric_to_c2po(formula, &PredicateTable::default(), &catalog, 1_000).unwrap();
    let source = MappingSourceIdentity {
        revision: "0".repeat(40),
        state: MappingSourceState::Clean,
    };
    let contextual =
        map_to_c2po_with_context(formula, "f", b"{}", source.clone(), None, 1_000, &catalog, None)
            .unwrap();
    assert_eq!(expression, contextual.expression);
    assert_eq!(expression, "(a U[1,2] b)");
    let legacy = map_to_c2po(formula, "f", b"{}", source, None, 1_000).unwrap();
    assert_eq!(legacy.expression, "(p0 U[1,2] p1)");
}

// Trace: TC-1591, FR-058-AC-3
#[test]
fn numeric_contract_and_interop_modules_spawn_no_process() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![root.join("contract.rs"), root.join("ltlf.rs")];
    for directory in ["numeric", "interop"] {
        for entry in std::fs::read_dir(root.join(directory)).unwrap() {
            files.push(entry.unwrap().path());
        }
    }
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(!text.contains("process::Command"), "{}", file.display());
        assert!(!text.contains("Command::new"), "{}", file.display());
    }
    // SPEC-GAP: the planned live campaign member (`make r2u2-numeric-live`) is
    // not added: no Make target exists to reserve and none may run R2U2.
}

fn grid_formula_nodes() -> Vec<Node> {
    // 0..2 x>-1, x>0, x>1; 3 d>=0.2; 4 d>=0.1; 5 G[0,1] (d>=0.1); 6 root.
    vec![
        atom(10),
        atom(11),
        atom(12),
        atom(20),
        atom(21),
        Node::new(NodeKind::Globally {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(4),
        }),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(5),
        }),
    ]
}

fn contract_set(pairs: &[(u32, u32, u32)]) -> ContractSet {
    ContractSet {
        graph_identity: String::new(),
        contracts: pairs
            .iter()
            .map(|&(id, assumption, guarantee)| ContractRoots {
                id,
                name: format!("c{id}"),
                assumption: NodeId(assumption),
                guarantee: NodeId(guarantee),
            })
            .collect(),
    }
}

// Trace: TC-1596, FR-060-AC-1
#[test]
fn numeric_contract_grid_is_refused_and_boolean_contracts_decompose_to_three_formulas() {
    let nodes = grid_formula_nodes();
    let formula = single(&nodes);
    let grid = contract_set(&[(1, 0, 3), (2, 0, 5), (3, 1, 3), (4, 1, 5), (5, 2, 3), (6, 2, 5)]);
    let export = export_contracts_c2po(formula, &grid, &grid_table(), &catalog(), 10_000);
    assert!(export.exported.is_empty());
    assert_eq!(export.refused.len(), 6);
    for (_, record) in &export.refused {
        assert_eq!(record.cause, NumericLossCause::TargetNumericProfileUnpinned);
    }
    assert!(export.note.contains("DEFERRED"));

    // Boolean contract: A = a, G = F[0,1] b. Read back by tl-mltl's own reader.
    let boolean_nodes = vec![
        atom(0),
        atom(1),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(1),
        }),
    ];
    let boolean = single(&boolean_nodes);
    let export = export_contracts_c2po(
        boolean,
        &contract_set(&[(1, 0, 2)]),
        &PredicateTable::default(),
        &catalog(),
        10_000,
    );
    assert!(export.refused.is_empty());
    let decomposition = &export.exported[0];
    let text = format!(
        "INPUT\n  a, b: bool;\nFTSPEC\n  {};\n  {};\n  {};\n",
        decomposition.assumption, decomposition.verified, decomposition.invalid
    );
    let read = import_c2po(
        text.as_bytes(),
        Some(SemanticProfile::OnlinePrefixV1),
        ReaderLimits::default(),
    )
    .unwrap();
    assert_eq!(read.requirements.len(), 3);
    let future = Node::new(NodeKind::Future {
        interval: Interval::new(0, 1).unwrap(),
        operand: NodeId(1),
    });
    assert_eq!(read.requirements[0].nodes, vec![atom(0)]);
    assert_eq!(
        read.requirements[1].nodes,
        vec![
            atom(0),
            atom(1),
            future,
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(2)
            }),
        ]
    );
    assert_eq!(
        read.requirements[2].nodes,
        vec![
            atom(0),
            atom(1),
            future,
            Node::new(NodeKind::Not { operand: NodeId(2) }),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(3)
            }),
        ]
    );
}

// Trace: TC-1597, FR-060-AC-2
#[test]
fn an_unexportable_clause_refuses_the_whole_contract() {
    // Boolean assumption `a` (node 0 of a graph that also holds x > 0), numeric guarantee.
    let nodes = vec![
        atom(0),
        atom(11),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let export = export_contracts_c2po(
        single(&nodes),
        &contract_set(&[(1, 0, 1), (2, 0, 0)]),
        &grid_table(),
        &catalog(),
        10_000,
    );
    assert_eq!(export.refused.len(), 1);
    assert_eq!(export.refused[0].0, 1);
    assert_eq!(export.refused[0].1.cause, NumericLossCause::TargetNumericProfileUnpinned);
    // No partial output for contract 1; contract 2 (Boolean) exports whole.
    assert_eq!(export.exported.len(), 1);
    assert_eq!(export.exported[0].id, 2);
}
