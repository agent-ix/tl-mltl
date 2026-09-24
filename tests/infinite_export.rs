#![cfg(feature = "infinite-trace")]

use std::{collections::BTreeSet, fs, path::Path, process::Command};

use tl_mltl::{
    infinite::{
        export_safety_monitor, replay_target_step, EvaluationLimit, PrefixRequest,
        SafetyExportError, SafetyReplayDisposition, TargetStepObservation,
    },
    PastMappingError, TargetOriginContract,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, NodeId, OwnedSignalDeclaration, PartialValuation,
    PartialValue, PastOperatorKind, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId, TemporalInterval, TraceObservation,
    UnboundedInterval, ValuationEntry,
};

fn graph(nodes: Vec<InfiniteNode>) -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn node(kind: K) -> InfiniteNode {
    InfiniteNode::new(kind)
}
fn open() -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(0))
}
fn closed() -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(0, 1).unwrap())
}

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(1),
            "p".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(1))],
    )
    .unwrap()
}

fn contract() -> TargetOriginContract {
    let mut origin = TargetOriginContract::reviewed_r2u2_4_2();
    origin.admitted_operators = [PastOperatorKind::Once]
        .into_iter()
        .collect::<BTreeSet<_>>();
    origin
}

fn rows(value: PartialValue) -> Vec<TraceObservation> {
    vec![TraceObservation {
        position: 0,
        valuation: PartialValuation::new(
            "map".to_owned(),
            &[PropositionId(0)],
            vec![ValuationEntry {
                proposition: PropositionId(0),
                value,
            }],
        )
        .unwrap(),
    }]
}

fn export(
    graph: &InfiniteFormulaDocument,
    observations: &[TraceObservation],
) -> Result<tl_mltl::infinite::SafetyMappingManifest, SafetyExportError> {
    export_safety_monitor(
        &PrefixRequest {
            formula: graph,
            graph_id: &graph.content_identity().unwrap(),
            proposition_map_id: "map",
            propositions: &[PropositionId(0)],
            observations,
            limit: EvaluationLimit::default(),
        },
        None,
        &catalog(),
        &contract(),
        100,
    )
}

// Trace: TC-168, TC-169, TC-171; FR-040-AC-1 and FR-040-AC-3
#[test]
fn exact_outer_safety_guard_exports_only_its_finite_horizon_body() {
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let evidence = export(&past, &rows(PartialValue::True)).unwrap();
    assert_eq!(evidence.section, "PTSPEC");
    assert_eq!(evidence.expression, "O[0,1](p)");
    assert_eq!(evidence.decision_horizon, 0);
    assert!(evidence.refutation_only);
    assert_eq!(evidence.profile, "mltl.infinite-trace/v1");
    let future = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let evidence = export(&future, &rows(PartialValue::True)).unwrap();
    assert_eq!(evidence.section, "FTSPEC");
    assert_eq!(evidence.expression, "F[0,1](p)");
    assert_eq!(evidence.decision_horizon, 1);
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn unsupported_shape_partial_observation_and_mixed_target_context_refuse() {
    let unbounded = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&unbounded, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&past, &rows(PartialValue::Missing)),
        Err(SafetyExportError::PartialValuation)
    );
    let mixed = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(1),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(2),
        }),
    ]);
    assert_eq!(
        export(&mixed, &rows(PartialValue::True)),
        Err(SafetyExportError::MixedTargetContext)
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn each_unbounded_future_family_has_a_distinct_typed_refusal() {
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let future = graph(vec![
        atom,
        node(K::Future {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&future, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    for temporal in [
        K::Until {
            interval: open(),
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Release {
            interval: open(),
            left: NodeId(0),
            right: NodeId(1),
        },
    ] {
        let formula = graph(vec![
            atom,
            node(K::True),
            node(temporal),
            node(K::Globally {
                interval: open(),
                operand: NodeId(2),
            }),
        ]);
        assert_eq!(
            export(&formula, &rows(PartialValue::True)),
            Err(SafetyExportError::UnboundedUntilRelease)
        );
    }
    let wrong_outer = graph(vec![
        atom,
        node(K::Globally {
            interval: TemporalInterval::Unbounded(UnboundedInterval::new(1)),
            operand: NodeId(0),
        }),
    ]);
    assert_eq!(
        export(&wrong_outer, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    let bounded_outer = graph(vec![
        atom,
        node(K::Globally {
            interval: closed(),
            operand: NodeId(0),
        }),
    ]);
    assert_eq!(
        export(&bounded_outer, &rows(PartialValue::True)),
        Err(SafetyExportError::UnsupportedShape)
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn nonempty_fairness_refuses_before_target_output_and_empty_fairness_is_neutral() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let graph_id = safety.content_identity().unwrap();
    let observations = rows(PartialValue::True);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    let premises = FairnessPremisesDocument::new(
        &safety,
        graph_id.clone(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, Some(&premises), &catalog(), &contract(), 100),
        Err(SafetyExportError::FairnessPremise)
    );
    let empty = FairnessPremisesDocument::new(
        &safety,
        graph_id.clone(),
        InfiniteClock::EventPosition,
        vec![],
    )
    .unwrap();
    assert!(export_safety_monitor(&request, Some(&empty), &catalog(), &contract(), 100).is_ok());
    let foreign_graph = graph(vec![node(K::True)]);
    let foreign = FairnessPremisesDocument::new(
        &foreign_graph,
        foreign_graph.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, Some(&foreign), &catalog(), &contract(), 100),
        Err(SafetyExportError::Identity)
    );
}

// Trace: TC-170; FR-040-AC-2
#[test]
fn export_refuses_a_noncanonical_prefix_and_missing_formula_binding() {
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let mut shifted = rows(PartialValue::True);
    shifted[0].position = 1;
    assert_eq!(export(&past, &shifted), Err(SafetyExportError::Identity));

    let request = PrefixRequest {
        formula: &past,
        graph_id: &past.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[],
        observations: &[],
        limit: EvaluationLimit::default(),
    };
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &contract(), 100),
        Err(SafetyExportError::Identity)
    );
}

// Trace: TC-166, TC-173; FR-039-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_target_origin_mismatch_before_artifact() {
    let bounds = Interval::new(2, 2).unwrap();
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: TemporalInterval::Closed(bounds),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&safety, &rows(PartialValue::True)),
        Err(SafetyExportError::TargetOriginIntervalMismatch {
            operator: PastOperatorKind::Once,
            interval: bounds,
        })
    );
}

// Trace: TC-166, TC-173; FR-039-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_exhausted_work_and_unreviewed_past_operator() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let observations = rows(PartialValue::True);
    let graph_id = safety.content_identity().unwrap();
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &contract(), 0),
        Err(SafetyExportError::ResourceIncomplete)
    );
    let mut unreviewed = contract();
    unreviewed.admitted_operators.clear();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &unreviewed, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginUnverified(PastOperatorKind::Once)
        ))
    );
    let mut missing_evidence = contract();
    missing_evidence.evidence_sha256.clear();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &missing_evidence, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::MissingOriginEvidence
        ))
    );
    let mut foreign_target = contract();
    foreign_target.target.version = "C2PO v4.2.0".to_owned();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &foreign_target, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginMismatch
        ))
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_unbounded_past_inside_the_finite_body() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&safety, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedPast)
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1, FR-041-AC-2
#[test]
fn reviewed_zero_window_historically_and_triggered_export_to_past_section() {
    let zero = TemporalInterval::Closed(Interval::new(0, 0).unwrap());
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let historically = graph(vec![
        atom,
        node(K::Historically {
            interval: zero,
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let triggered = graph(vec![
        atom,
        node(K::Triggered {
            interval: zero,
            left: NodeId(0),
            right: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let origin = TargetOriginContract::reviewed_r2u2_4_2();
    let observations = rows(PartialValue::True);
    for (formula, expected) in [
        (&historically, "H[0,0](p)"),
        (&triggered, "(!((!p) S[0,0] (!p)))"),
    ] {
        let graph_id = formula.content_identity().unwrap();
        let request = PrefixRequest {
            formula,
            graph_id: &graph_id,
            proposition_map_id: "map",
            propositions: &[PropositionId(0)],
            observations: &observations,
            limit: EvaluationLimit::default(),
        };
        let manifest = export_safety_monitor(&request, None, &catalog(), &origin, 100).unwrap();
        assert_eq!(manifest.section, "PTSPEC");
        assert_eq!(manifest.expression, expected);
        assert!(manifest.refutation_only);
    }
}

// Trace: TC-172, TC-173; FR-041-AC-2
#[test]
fn safety_export_refuses_catalog_name_without_exact_c2po_identifier() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let graph_id = safety.content_identity().unwrap();
    let observations = rows(PartialValue::True);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    let catalog = SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(1),
            "sensor-name".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(1))],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog, &contract(), 100),
        Err(SafetyExportError::Signal)
    );
}

// Trace: TC-169, TC-170; FR-040-AC-1 and FR-040-AC-2
#[test]
fn target_violation_replays_at_its_exact_position_and_pass_remains_inconclusive() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let false_rows = rows(PartialValue::False);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &safety.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &false_rows,
        limit: EvaluationLimit::default(),
    };
    let manifest = export_safety_monitor(&request, None, &catalog(), &contract(), 100).unwrap();
    let step = |position, verdict| TargetStepObservation {
        target: &manifest.target,
        expression_sha256: &manifest.output_sha256,
        position,
        verdict,
    };
    assert_eq!(
        replay_target_step(&manifest, &request, step(0, false)).unwrap(),
        SafetyReplayDisposition::Refuted
    );
    assert_eq!(
        replay_target_step(&manifest, &request, step(0, true)).unwrap(),
        SafetyReplayDisposition::Mismatch
    );
    assert_eq!(
        replay_target_step(&manifest, &request, step(1, false)).unwrap(),
        SafetyReplayDisposition::Mismatch
    );
    let mut stale = manifest.clone();
    stale.refutation_only = false;
    assert_eq!(
        replay_target_step(&stale, &request, step(0, false)),
        Err(SafetyExportError::TargetMismatch)
    );
    for field in [
        "profile",
        "providerRevision",
        "graphId",
        "inputSha256",
        "clock",
        "target",
        "expression",
    ] {
        let mut changed = manifest.clone();
        match field {
            "profile" => changed.profile = "mltl.closed-trace/v1",
            "providerRevision" => changed.provider_revision = "foreign-revision",
            "graphId" => changed.graph_id.push_str("-foreign"),
            "inputSha256" => changed.input_sha256 = "0".repeat(64),
            "clock" => changed.clock = "fixed-sample",
            "target" => changed.target.version.push_str("-foreign"),
            "expression" => changed.expression.push_str(" && false"),
            _ => unreachable!(),
        }
        assert_eq!(
            replay_target_step(&changed, &request, step(0, false)),
            Err(SafetyExportError::TargetMismatch),
            "tampered manifest field {field} was admitted"
        );
    }
    let wrong_digest = TargetStepObservation {
        target: &manifest.target,
        expression_sha256: "0",
        position: 0,
        verdict: false,
    };
    assert_eq!(
        replay_target_step(&manifest, &request, wrong_digest),
        Err(SafetyExportError::TargetMismatch)
    );

    let true_rows = rows(PartialValue::True);
    let passing_request = PrefixRequest {
        formula: &safety,
        graph_id: &safety.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &true_rows,
        limit: EvaluationLimit::default(),
    };
    let passing =
        export_safety_monitor(&passing_request, None, &catalog(), &contract(), 100).unwrap();
    let passing_step = TargetStepObservation {
        target: &passing.target,
        expression_sha256: &passing.output_sha256,
        position: 0,
        verdict: true,
    };
    assert_eq!(
        replay_target_step(&passing, &passing_request, passing_step).unwrap(),
        SafetyReplayDisposition::Inconclusive
    );
}

// Set TL_MLTL_C2PO_SOURCE to the exact retained R2U2 4.2 source.
// Trace: TC-173; FR-041-AC-2
#[test]
fn c2po_4_2_type_checker_rejects_mixed_time_in_both_sections() {
    let Ok(source) = std::env::var("TL_MLTL_C2PO_SOURCE") else {
        return;
    };
    let source = Path::new(&source);
    let revision = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(source)
        .output()
        .unwrap();
    assert!(revision.status.success());
    assert_eq!(
        String::from_utf8_lossy(&revision.stdout).trim(),
        "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a"
    );
    let directory = tempfile::tempdir().unwrap();
    for (section, expected) in [
        (
            "FTSPEC",
            "mixed-time formulas unsupported, found PT formula in FTSPEC",
        ),
        (
            "PTSPEC",
            "mixed-time formulas unsupported, found FT formula in PTSPEC",
        ),
    ] {
        // This is the renderer's exact nested expression for O[0,1](F[0,1] p).
        let specification = directory.path().join(format!("mixed-{section}.c2po"));
        fs::write(
            &specification,
            format!("INPUT\n  p: bool;\n{section}\n  O[0,1](F[0,1](p));\n"),
        )
        .unwrap();
        let output = Command::new("python3")
            .arg(source.join("compiler/c2po.py"))
            .args(["--spec", specification.to_str().unwrap(), "--type-check"])
            .current_dir(source)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{section} unexpectedly accepted mixed time"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{section}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for (name, section, expression) in [
        ("once", "PTSPEC", "O[0,1](p)"),
        ("historically", "PTSPEC", "H[0,1](p)"),
        ("previous", "PTSPEC", "O[1,1](p)"),
        ("since", "PTSPEC", "(p S[0,1] q)"),
        ("triggered", "PTSPEC", "(!((!p) S[0,1] (!q)))"),
        ("future", "FTSPEC", "F[0,1](p)"),
        ("globally", "FTSPEC", "G[0,1](p)"),
        ("until", "FTSPEC", "(p U[0,1] q)"),
        ("release", "FTSPEC", "(p R[0,1] q)"),
    ] {
        let specification = directory.path().join(format!("admitted-{name}.c2po"));
        fs::write(
            &specification,
            format!("INPUT\n  p,q: bool;\n{section}\n  {expression};\n"),
        )
        .unwrap();
        let output = Command::new("python3")
            .arg(source.join("compiler/c2po.py"))
            .args(["--spec", specification.to_str().unwrap(), "--type-check"])
            .current_dir(source)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
