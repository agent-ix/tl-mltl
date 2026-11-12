//! TL-371: behavior tests for the safety C2PO export and its replay, which the first full
//! cargo-mutants run showed were not asserted one operand at a time.
#![cfg(feature = "infinite-trace")]

use tl_mltl::{
    infinite::{
        export_safety_monitor, replay_target_step, EvaluationLimit, PrefixRequest,
        SafetyExportError, SafetyMappingManifest, TargetStepObservation,
    },
    TargetOriginContract,
};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as K, Interval, NodeId,
    OwnedSignalDeclaration, PartialValuation, PartialValue, PropositionBinding, PropositionId,
    SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId, TemporalInterval,
    TraceObservation, UnboundedInterval, ValuationEntry,
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

fn export_with(
    graph: &InfiniteFormulaDocument,
    limit: u64,
) -> Result<SafetyMappingManifest, SafetyExportError> {
    let observations = rows(PartialValue::True);
    export_safety_monitor(
        &PrefixRequest {
            formula: graph,
            graph_id: &graph.content_identity().unwrap(),
            proposition_map_id: "map",
            propositions: &[PropositionId(0)],
            observations: &observations,
            limit: EvaluationLimit::default(),
        },
        None,
        &catalog(),
        &TargetOriginContract::reviewed_r2u2_4_2(),
        limit,
    )
}

fn globally(body: Vec<InfiniteNode>) -> InfiniteFormulaDocument {
    let mut nodes = body;
    let operand = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    nodes.push(node(K::Globally {
        interval: open(),
        operand,
    }));
    graph(nodes)
}

fn p() -> InfiniteNode {
    node(K::Proposition {
        proposition: PropositionId(0),
    })
}

// Trace: TC-168
#[test]
fn the_render_work_limit_is_inclusive_and_binary_operators_render_both_sides() {
    let one = globally(vec![p()]);
    assert_eq!(export_with(&one, 1).unwrap().expression, "p");
    assert_eq!(
        export_with(&one, 0),
        Err(SafetyExportError::ResourceIncomplete)
    );
    let both = globally(vec![
        p(),
        p(),
        node(K::And {
            left: NodeId(0),
            right: NodeId(1),
        }),
    ]);
    assert_eq!(export_with(&both, 100).unwrap().expression, "(p && p)");
}

// Trace: TC-168
#[test]
fn a_since_is_admitted_only_as_the_point_interval_zero() {
    let since = |start, end| {
        globally(vec![
            p(),
            p(),
            node(K::Since {
                interval: TemporalInterval::Closed(Interval::new(start, end).unwrap()),
                left: NodeId(0),
                right: NodeId(1),
            }),
        ])
    };
    // Only the exact point interval [0,0] is admitted by the target origin evidence.
    for (start, end) in [(0, 1), (1, 1)] {
        assert!(matches!(
            export_with(&since(start, end), 100),
            Err(SafetyExportError::TargetOriginIntervalMismatch { .. })
        ));
    }
    // The exact point interval [0,0] reduces to the right operand.
    assert_eq!(export_with(&since(0, 0), 100).unwrap().expression, "p");
}

// Trace: TC-169
#[test]
fn a_replay_is_refused_when_any_manifest_field_alone_differs() {
    let safety = globally(vec![p()]);
    let false_rows = rows(PartialValue::False);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &safety.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &false_rows,
        limit: EvaluationLimit::default(),
    };
    let manifest = export_safety_monitor(
        &request,
        None,
        &catalog(),
        &TargetOriginContract::reviewed_r2u2_4_2(),
        100,
    )
    .unwrap();
    let replay = |m: &SafetyMappingManifest, digest: &str| {
        replay_target_step(
            m,
            &request,
            TargetStepObservation {
                target: &manifest.target,
                expression_sha256: digest,
                position: 0,
                verdict: false,
            },
        )
    };
    assert!(replay(&manifest, &manifest.output_sha256).is_ok());
    let refused = Err(SafetyExportError::TargetMismatch);
    let tampered: Vec<(&str, Box<dyn Fn(&mut SafetyMappingManifest)>)> = vec![
        ("profile", Box::new(|m| m.profile = "other")),
        ("revision", Box::new(|m| m.provider_revision = "other")),
        ("graph", Box::new(|m| m.graph_id = "other".into())),
        ("input", Box::new(|m| m.input_sha256 = "0".repeat(64))),
        ("clock", Box::new(|m| m.clock = "other")),
        ("target", Box::new(|m| m.target.name = "other".into())),
        ("expression", Box::new(|m| m.expression = "q".into())),
    ];
    for (name, tamper) in tampered {
        let mut changed = manifest.clone();
        tamper(&mut changed);
        assert_eq!(replay(&changed, &manifest.output_sha256), refused, "{name}");
    }
    let mut digest_only = manifest.clone();
    digest_only.output_sha256 = "f".repeat(64);
    assert_eq!(replay(&digest_only, &"f".repeat(64)), refused);
}

// Trace: TC-169
#[test]
fn export_errors_render_their_variant() {
    let message = SafetyExportError::TargetMismatch.to_string();
    assert!(message.contains("TargetMismatch"), "{message}");
}
