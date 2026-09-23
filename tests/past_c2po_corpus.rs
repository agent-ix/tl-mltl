use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, map_past_to_c2po, ClockBinding, MappingSourceIdentity, MappingSourceState,
    PastEvaluationLimits, PastEvaluationRelationInput, PastMappingError, PositionHistoryDocument,
    PositionObservation, TargetOriginContract, ToolIdentity,
};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PastOperatorKind,
    PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain,
    SignalId,
};

const MANIFEST: &[u8] = include_bytes!("../corpus/past-c2po-v1/manifest.json");
const SHA256SUMS: &str = include_str!("../corpus/past-c2po-v1/SHA256SUMS");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    profile: String,
    clock: String,
    source_oracle: String,
    target_observation: TargetObservation,
    target_limitation: String,
    unsafe_since_counterexample: UnsafeSinceCounterexample,
    trace: Vec<Row>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UnsafeSinceCounterexample {
    formula: String,
    position: usize,
    source: bool,
    target: bool,
    source_revision: String,
    compiler_version: String,
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
struct TargetObservation {
    recorded_at: String,
    source_revision: String,
    source_ref: String,
    compiler_version: String,
    compiler_entry_sha256: String,
    python_version: String,
    monitor_build: String,
    compiler_command: String,
    monitor_command: String,
    monitor_executable_sha256: String,
    files: Vec<TargetFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetFile {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    target_formula_id: usize,
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

// Trace: TC-160, TC-161, TC-164, TC-165, TC-174; FR-038-AC-1, FR-038-AC-3, FR-039-AC-1, FR-042-AC-1
#[test]
fn pinned_past_corpus_compares_each_source_step_with_one_retained_target_run() {
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
    let target = &manifest.target_observation;
    assert_eq!(target.recorded_at, "2026-09-22");
    assert_eq!(
        target.source_revision,
        "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a"
    );
    assert_eq!(target.source_ref, "R2U2 4.2-release");
    assert_eq!(target.compiler_version, "C2PO v4.1.0");
    assert_eq!(target.compiler_entry_sha256.len(), 64);
    assert_eq!(target.monitor_executable_sha256.len(), 64);
    assert_eq!(target.python_version, "3.14.7");
    assert!(target.monitor_build.contains("make -C monitors/c"));
    assert!(target.compiler_command.contains("compiler/c2po.py"));
    assert!(target.monitor_command.contains("monitors/c/build/r2u2"));
    assert!(manifest
        .target_limitation
        .contains("One retained six-formula C2PO and R2U2 4.2 execution"));
    assert_eq!(target.files.len(), 9);
    let mut pinned = BTreeSet::new();
    for file in &target.files {
        assert!(pinned.insert(file.path.as_str()));
        let bytes = std::fs::read(format!("corpus/past-c2po-v1/{}", file.path)).unwrap();
        let actual = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(actual, file.sha256, "{}", file.path);
    }
    let counterexample = &manifest.unsafe_since_counterexample;
    assert_eq!(counterexample.formula, "p S[0,2] q");
    assert_eq!(counterexample.position, 2);
    assert!(!counterexample.source);
    assert!(counterexample.target);
    assert_eq!(counterexample.source_revision, target.source_revision);
    assert_eq!(counterexample.compiler_version, target.compiler_version);
    let counterexample_output =
        std::fs::read_to_string("corpus/past-c2po-v1/target-4.2/unsafe-since.stdout").unwrap();
    assert!(counterexample_output.lines().any(|line| line == "0:2,T"));
    let counterexample_trace =
        std::fs::read_to_string("corpus/past-c2po-v1/target-4.2/unsafe-since.csv").unwrap();
    assert_eq!(counterexample_trace, "# p,q\n0,1\n0,0\n1,0\n");
    let unsafe_case = manifest
        .cases
        .iter()
        .find(|case| case.id == "since-zero-two")
        .unwrap();
    let unsafe_nodes = formula(unsafe_case);
    let unsafe_graph = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(unsafe_nodes.len() - 1).unwrap()),
        &unsafe_nodes,
    )
    .unwrap();
    let unsafe_history = PositionHistoryDocument::new(
        "unsafe-since",
        1,
        0,
        2,
        Some(ClockBinding::EventPosition),
        vec![
            PositionObservation::new(0, vec![PropositionId(1)], None),
            PositionObservation::new(1, vec![], None),
            PositionObservation::new(2, vec![PropositionId(0)], None),
        ],
    )
    .unwrap();
    let source_verdict = evaluate_past(
        unsafe_graph,
        "unsafe-since",
        &unsafe_history,
        2,
        "map",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(source_verdict.verdict, counterexample.source);
    assert_ne!(source_verdict.verdict, counterexample.target);
    let output = std::fs::read_to_string("corpus/past-c2po-v1/target-4.2/r2u2.stdout").unwrap();
    let source = std::fs::read_to_string("corpus/past-c2po-v1/target-4.2/past.c2po").unwrap();
    let expressions: Vec<_> = source
        .split("PTSPEC")
        .nth(1)
        .unwrap()
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    assert_eq!(expressions.len(), 6);
    let source_sha = target
        .files
        .iter()
        .find(|file| file.path.ends_with("past.c2po"))
        .unwrap();
    let output_sha = target
        .files
        .iter()
        .find(|file| file.path.ends_with("r2u2.stdout"))
        .unwrap();
    let origin = TargetOriginContract {
        target: ToolIdentity {
            name: "C2PO".to_owned(),
            version: target.compiler_version.clone(),
            executable_sha256: target.compiler_entry_sha256.clone(),
            configuration_sha256: source_sha.sha256.clone(),
        },
        evidence_sha256: output_sha.sha256.clone(),
        admitted_operators: [
            PastOperatorKind::Once,
            PastOperatorKind::Historically,
            PastOperatorKind::StrongPrevious,
            PastOperatorKind::Since,
            PastOperatorKind::Triggered,
        ]
        .into_iter()
        .collect(),
    };
    let catalog = SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap();
    let mut verdicts = BTreeMap::new();
    for line in output.lines() {
        let (identity, verdict) = line.split_once(',').unwrap();
        let (formula, position) = identity.split_once(':').unwrap();
        let formula: usize = formula.parse().unwrap();
        let position: usize = position.parse().unwrap();
        let truth = match verdict {
            "T" => true,
            "F" => false,
            other => panic!("unexpected target verdict {other}"),
        };
        assert!(verdicts.insert((formula, position), truth).is_none());
    }
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
    let mut formula_ids = BTreeSet::new();
    for case in &manifest.cases {
        assert!(formula_ids.insert(case.target_formula_id));
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
        let mapping = map_past_to_c2po(
            graph,
            &case.id,
            case.formula.as_bytes(),
            MappingSourceIdentity {
                revision: "corpus-replay".to_owned(),
                state: MappingSourceState::Clean,
            },
            &catalog,
            &origin,
            100,
        );
        if matches!(case.id.as_str(), "since-zero-two" | "triggered-zero-two") {
            let operator = if case.id == "since-zero-two" {
                PastOperatorKind::Since
            } else {
                PastOperatorKind::Triggered
            };
            assert_eq!(
                mapping,
                Err(PastMappingError::TargetOriginIntervalMismatch {
                    operator,
                    interval: Interval::new(0, 2).unwrap(),
                })
            );
        } else {
            let mapping = mapping.unwrap();
            assert_eq!(
                mapping.expression, expressions[case.target_formula_id],
                "{}",
                case.id
            );
            assert_eq!(mapping.target, origin.target);
            assert_eq!(
                mapping.target_origin_evidence_sha256,
                origin.evidence_sha256
            );
        }
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
            assert_eq!(
                verdicts.get(&(case.target_formula_id, position)),
                Some(expected),
                "retained R2U2 4.2 target mismatch for {} at {position}",
                case.id
            );
        }
    }
    assert_eq!(formula_ids, BTreeSet::from([0, 1, 2, 3, 4, 5]));
}

// Trace: TC-191; FR-052-AC-1
#[test]
fn exact_pin_live_target_replays_bounded_past_and_unsafe_origin() {
    let Ok(source) = std::env::var("TL_MLTL_C2PO_SOURCE") else {
        return;
    };
    let source = Path::new(&source);
    let target: Manifest = serde_json::from_slice(MANIFEST).unwrap();
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(source)
        .output()
        .unwrap();
    assert!(revision.status.success());
    assert_eq!(
        String::from_utf8_lossy(&revision.stdout).trim(),
        target.target_observation.source_revision
    );
    let compiler = source.join("compiler/c2po.py");
    let monitor = source.join("monitors/c/build/r2u2");
    let digest = |path: &Path| {
        Sha256::digest(std::fs::read(path).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    assert_eq!(
        digest(&compiler),
        target.target_observation.compiler_entry_sha256
    );
    assert_eq!(
        digest(&monitor),
        target.target_observation.monitor_executable_sha256
    );
    let directory = tempfile::tempdir().unwrap();
    for (name, spec, trace, map, expected_binary, expected_output) in [
        (
            "bounded",
            "corpus/r2u2-v4.2/formulas.c2po",
            "corpus/r2u2-v4.2/trace.csv",
            Some("corpus/r2u2-v4.2/signals.map"),
            Some("corpus/r2u2-v4.2/spec.bin"),
            "corpus/r2u2-v4.2/r2u2.stdout",
        ),
        (
            "past",
            "corpus/past-c2po-v1/target-4.2/past.c2po",
            "corpus/past-c2po-v1/target-4.2/trace.csv",
            None,
            Some("corpus/past-c2po-v1/target-4.2/spec.bin"),
            "corpus/past-c2po-v1/target-4.2/r2u2.stdout",
        ),
        (
            "unsafe-since",
            "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
            "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
            None,
            Some("corpus/past-c2po-v1/target-4.2/unsafe-since.bin"),
            "corpus/past-c2po-v1/target-4.2/unsafe-since.stdout",
        ),
    ] {
        let binary = directory.path().join(format!("{name}.bin"));
        let mut command = Command::new("python3");
        command.arg(&compiler).arg("--spec").arg(spec);
        if let Some(map) = map {
            command.arg("--map").arg(map);
        } else {
            command.arg("--trace").arg(trace);
        }
        let compiled = command.arg("--output").arg(&binary).output().unwrap();
        assert!(
            compiled.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        if let Some(expected) = expected_binary {
            assert_eq!(
                std::fs::read(&binary).unwrap(),
                std::fs::read(expected).unwrap(),
                "{name} binary"
            );
        }
        let execution = Command::new(&monitor)
            .arg(&binary)
            .arg(trace)
            .output()
            .unwrap();
        assert!(
            execution.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&execution.stderr)
        );
        assert_eq!(
            execution.stdout,
            std::fs::read(expected_output).unwrap(),
            "{name} per-step target output"
        );
    }
}
