use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, ClockBinding, PastEvaluationLimits, PastEvaluationRelationInput,
    PositionHistoryDocument, PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const MANIFEST: &[u8] = include_bytes!("../corpus/past-c2po-v1/manifest.json");
const SHA256SUMS: &str = include_str!("../corpus/past-c2po-v1/SHA256SUMS");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    profile: String,
    clock: String,
    source_oracle: String,
    target_observation: Option<serde_json::Value>,
    target_limitation: String,
    trace: Vec<Row>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    position: u64,
    p: bool,
    q: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    formula: String,
    expected_source: Vec<bool>,
    origin_hazard: String,
}

fn formula(case: &Case) -> Vec<Node> {
    let p = Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    });
    let q = Node::new(NodeKind::Proposition {
        proposition: PropositionId(1),
    });
    let pair = [p, q];
    let range = |a, b| Interval::new(a, b).unwrap();
    match (case.id.as_str(), case.formula.as_str()) {
        ("once-zero-one", "O[0,1] p") => vec![
            p,
            Node::new(NodeKind::Once {
                interval: range(0, 1),
                operand: NodeId(0),
            }),
        ],
        ("historically-zero-one", "H[0,1] p") => vec![
            p,
            Node::new(NodeKind::Historically {
                interval: range(0, 1),
                operand: NodeId(0),
            }),
        ],
        ("previous", "Y p") => vec![
            p,
            Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
        ],
        ("since-zero-two", "p S[0,2] q") => pair
            .into_iter()
            .chain([Node::new(NodeKind::Since {
                interval: range(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            })])
            .collect(),
        ("triggered-zero-two", "p T[0,2] q") => pair
            .into_iter()
            .chain([Node::new(NodeKind::Triggered {
                interval: range(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            })])
            .collect(),
        ("once-one-one", "O[1,1] p") => vec![
            p,
            Node::new(NodeKind::Once {
                interval: range(1, 1),
                operand: NodeId(0),
            }),
        ],
        _ => panic!(
            "unreviewed past C2PO corpus formula {}: {}",
            case.id, case.formula
        ),
    }
}

// Trace: TC-164, TC-174; FR-038-AC-3, FR-042-AC-1
#[test]
fn pinned_past_corpus_replays_each_source_step_without_claiming_a_target_run() {
    let digest = Sha256::digest(MANIFEST)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(SHA256SUMS, format!("{digest}  manifest.json\n"));
    let manifest: Manifest = serde_json::from_slice(MANIFEST).unwrap();
    assert_eq!(manifest.schema_version, "tl-mltl.past-c2po-corpus/v1");
    assert_eq!(manifest.profile, "mltl.origin-complete-history/v1");
    assert_eq!(manifest.clock, "event_position");
    assert!(manifest.source_oracle.contains("tl-mltl"));
    assert!(manifest.target_observation.is_none());
    assert!(manifest
        .target_limitation
        .contains("No new C2PO/R2U2 execution"));
    let observations = manifest
        .trace
        .iter()
        .map(|row| {
            let propositions = [(row.p, PropositionId(0)), (row.q, PropositionId(1))]
                .into_iter()
                .filter_map(|(value, id)| value.then_some(id))
                .collect();
            PositionObservation::new(row.position, propositions, None)
        })
        .collect();
    let through = manifest.trace.last().unwrap().position;
    let history = PositionHistoryDocument::new(
        "past-c2po-v1",
        1,
        0,
        through,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap();
    assert_eq!(manifest.cases.len(), 6);
    for case in &manifest.cases {
        assert_eq!(
            case.expected_source.len(),
            manifest.trace.len(),
            "{}",
            case.id
        );
        assert!(!case.origin_hazard.is_empty());
        let nodes = formula(case);
        let graph = Formula::new(
            SemanticProfile::OriginCompleteHistoryV1,
            NodeId(u32::try_from(nodes.len() - 1).unwrap()),
            &nodes,
        )
        .unwrap();
        for (position, expected) in case.expected_source.iter().enumerate() {
            let actual = evaluate_past(
                graph,
                &case.id,
                &history,
                u64::try_from(position).unwrap(),
                "map",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            )
            .unwrap();
            assert_eq!(actual.verdict, *expected, "{} at {position}", case.id);
        }
    }
}
