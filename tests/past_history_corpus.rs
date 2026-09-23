use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    analyze_required_history, evaluate_past, ClockBinding, ClockError, ClockSample, ExactNumber,
    HistoryError, OwnerHistoryState, PastEvaluationError, PastEvaluationLimits,
    PastEvaluationRelationInput, PastEvaluationReport, PastResultRelationKind,
    PositionHistoryDocument, PositionHistorySource, PositionObservation,
};
use tl_syntax::{
    Formula, FormulaDocument, FormulaError, Interval, Node, NodeId, NodeKind, PropositionId,
    SemanticProfile, TemporalFamily,
};

const DIRECTORY: &str = "past-history";
const MANIFEST_SHA256: &str = "0bb497481a08d82ae74db794657eb6e7c57e6d1e5b5a8471b3559f82f405afd1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    corpus: String,
    revision: u64,
    role: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    history_schema: String,
    dialect: String,
    implementation_revisions: Revisions,
    files: Vec<Pin>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revisions {
    tl_syntax: String,
    tl_parse: String,
    tl_mltl: String,
    tl_rewrite: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cases {
    corpus: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    dialect: String,
    formulas: Vec<FormulaCase>,
    histories: Vec<HistoryWire>,
    evaluations: Vec<Evaluation>,
    rewrites: Vec<serde_json::Value>,
    refusals: Vec<serde_json::Value>,
    target_dispositions: Vec<serde_json::Value>,
    mutation_axes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormulaCase {
    id: String,
    source: String,
    required_history: u64,
    document: FormulaDocument,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryWire {
    id: String,
    history_id: String,
    revision: u64,
    origin_position: u64,
    through_position: u64,
    clock: ClockWire,
    observations: Vec<ObservationWire>,
}
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ClockWire {
    EventPosition,
    FixedSample {
        epoch: NumberWire,
        period: NumberWire,
        unit: String,
    },
}
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct NumberWire {
    numerator: i64,
    denominator: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservationWire {
    position: u64,
    true_propositions: Vec<u32>,
    sample: Option<NumberWire>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Evaluation {
    id: String,
    formula: String,
    history: String,
    anchor: u64,
    verdict: bool,
    required_history: u64,
    relation: String,
    predecessor_history: Option<String>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn number(value: NumberWire) -> ExactNumber {
    ExactNumber::new(value.numerator, value.denominator).unwrap()
}

fn history(value: &HistoryWire) -> PositionHistoryDocument {
    let (clock, unit) = match &value.clock {
        ClockWire::EventPosition => (ClockBinding::EventPosition, None),
        ClockWire::FixedSample {
            epoch,
            period,
            unit,
        } => (
            ClockBinding::FixedSample {
                epoch: number(*epoch),
                period: number(*period),
                unit: unit.clone(),
            },
            Some(unit.as_str()),
        ),
    };
    let observations = value
        .observations
        .iter()
        .map(|item| {
            PositionObservation::new(
                item.position,
                item.true_propositions
                    .iter()
                    .copied()
                    .map(PropositionId)
                    .collect(),
                item.sample.map(|sample| ClockSample {
                    instant: number(sample),
                    unit: unit.unwrap_or_default().to_owned(),
                }),
            )
        })
        .collect();
    PositionHistoryDocument::new(
        &value.history_id,
        value.revision,
        value.origin_position,
        value.through_position,
        Some(clock),
        observations,
    )
    .unwrap()
}

fn read_corpus(root: &Path, relative: &str) -> Vec<u8> {
    let path = root.join(relative);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn load() -> (Manifest, Cases) {
    let root = Path::new(tl_syntax::CORPUS_DIR).join(DIRECTORY);
    let manifest_bytes = read_corpus(&root, "manifest.json");
    assert_eq!(digest(&manifest_bytes), MANIFEST_SHA256);
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).unwrap();
    let pins: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|pin| (&pin.path, &pin.sha256))
        .collect();
    assert_eq!(pins.len(), 3);
    for pin in &manifest.files {
        assert_eq!(
            digest(&read_corpus(&root, &pin.path)),
            pin.sha256,
            "{}",
            pin.path
        );
    }
    let cases = serde_json::from_slice(&read_corpus(&root, "cases.json")).unwrap();
    (manifest, cases)
}

// Trace: TC-056, FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-012-AC-2, FR-012-AC-3, FR-012-AC-4, FR-013-AC-3
#[test]
fn exact_shared_corpus_replays_history_analysis_evaluation_and_corrections() {
    let (manifest, cases) = load();
    assert_eq!(manifest.corpus, "tl-syntax.past-history-corpus/v1");
    assert_eq!(manifest.revision, 1);
    assert_eq!(manifest.role, "evidence-input");
    assert_eq!(manifest.formula_schema, "tl-syntax.formula/v2");
    assert_eq!(manifest.operator_profile, "tl-syntax.past-operators/v1");
    assert_eq!(manifest.semantic_profile, "mltl.origin-complete-history/v1");
    assert_eq!(manifest.history_schema, "tl-mltl.position-history/v1");
    assert_eq!(manifest.dialect, "tl-parse.clean-ascii/v3");
    assert_eq!(
        manifest.implementation_revisions.tl_syntax,
        "e70f2379a752117c79603bc399a86c26feed7716"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_parse,
        "f82b0c724675c0f774415aa696c360959da30481"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_mltl,
        "b346cd0902794633e862f644a5575fc9776c34fb"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_rewrite,
        "22b9cadcb1692cec8d3a97768f4f3b38fc654a5e"
    );
    assert_eq!(cases.corpus, manifest.corpus);
    assert_eq!(cases.formula_schema, manifest.formula_schema);
    assert_eq!(cases.operator_profile, manifest.operator_profile);
    assert_eq!(cases.semantic_profile, manifest.semantic_profile);
    assert_eq!(cases.dialect, manifest.dialect);
    assert_eq!(cases.rewrites.len(), 3);
    assert_eq!(cases.refusals.len(), 12);
    assert_eq!(cases.target_dispositions.len(), 4);
    assert_eq!(cases.mutation_axes.len(), 10);

    let formulas: BTreeMap<_, _> = cases
        .formulas
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let histories: BTreeMap<_, _> = cases
        .histories
        .iter()
        .map(|item| (item.id.as_str(), history(item)))
        .collect();
    assert_eq!(formulas.len(), cases.formulas.len());
    assert_eq!(histories.len(), cases.histories.len());
    assert_eq!(
        histories["event-main-r1"].history_id(),
        histories["event-main-r2"].history_id()
    );
    assert_ne!(
        histories["event-main-r1"].history_sha256(),
        histories["event-main-r2"].history_sha256()
    );

    for case in &cases.formulas {
        assert!(!case.source.is_empty());
        let formula = case.document.validate().unwrap();
        assert_eq!(
            analyze_required_history(formula, &case.id)
                .unwrap()
                .required_positions,
            case.required_history,
            "{}",
            case.id
        );
    }

    for case in &cases.evaluations {
        let formula_case = formulas[case.formula.as_str()];
        let formula = formula_case.document.validate().unwrap();
        let current_history = &histories[case.history.as_str()];
        let predecessor: Option<PastEvaluationReport> =
            case.predecessor_history.as_ref().map(|id| {
                evaluate_past(
                    formula,
                    &case.formula,
                    &histories[id.as_str()],
                    case.anchor,
                    "corpus-map",
                    1,
                    PastEvaluationRelationInput::Original,
                    PastEvaluationLimits::default(),
                )
                .unwrap()
            });
        let (result_revision, relation) = match (case.relation.as_str(), predecessor.as_ref()) {
            ("original", None) => (1, PastEvaluationRelationInput::Original),
            ("superseding", Some(prior)) => (2, PastEvaluationRelationInput::Superseding(prior)),
            ("invalidating", Some(prior)) => (2, PastEvaluationRelationInput::Invalidating(prior)),
            _ => panic!("invalid relation shape for {}", case.id),
        };
        let report = evaluate_past(
            formula,
            &case.formula,
            current_history,
            case.anchor,
            "corpus-map",
            result_revision,
            relation,
            PastEvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(report.verdict, case.verdict, "{}", case.id);
        assert_eq!(
            report.required_history, case.required_history,
            "{}",
            case.id
        );
        assert_eq!(
            report.relation.kind,
            match case.relation.as_str() {
                "original" => PastResultRelationKind::Original,
                "superseding" => PastResultRelationKind::Superseding,
                "invalidating" => PastResultRelationKind::Invalidating,
                _ => unreachable!(),
            }
        );
        assert_eq!(
            report.history.history_sha256,
            current_history.history_sha256()
        );
        report
            .validate_with_predecessor(predecessor.as_ref())
            .unwrap();
        let bytes = serde_json::to_vec(&report).unwrap();
        assert_eq!(
            serde_json::from_slice::<PastEvaluationReport>(&bytes).unwrap(),
            report
        );
    }
}

// Trace: TC-056, FR-012-AC-3, FR-013-AC-3
#[test]
fn corpus_digest_and_history_identity_mutations_are_detected() {
    let root = Path::new(tl_syntax::CORPUS_DIR).join(DIRECTORY);
    let mut manifest = read_corpus(&root, "manifest.json");
    manifest[0] ^= 1;
    assert_ne!(digest(&manifest), MANIFEST_SHA256);

    let (_, cases) = load();
    let admitted = history(&cases.histories[0]);
    let mut value = serde_json::to_value(&admitted).unwrap();
    value["historySha256"] = serde_json::Value::String("0".repeat(64));
    assert!(serde_json::from_value::<PositionHistoryDocument>(value).is_err());
}

// Trace: TC-053, TC-056, FR-011-AC-4, FR-012-AC-1, FR-012-AC-5, FR-013-AC-3
#[test]
fn every_shared_refusal_case_exercises_its_native_boundary() {
    let (_, cases) = load();
    let formulas: BTreeMap<_, _> = cases
        .formulas
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let histories: BTreeMap<_, _> = cases
        .histories
        .iter()
        .map(|item| (item.id.as_str(), history(item)))
        .collect();
    let once = formulas["once-endpoints"].document.validate().unwrap();

    for refusal in &cases.refusals {
        let value = refusal.as_object().unwrap();
        let id = value["id"].as_str().unwrap();
        let observed = match id {
            "mixed-future-past" => {
                let nodes = [
                    Node::new(NodeKind::True),
                    Node::new(NodeKind::Future {
                        interval: Interval::new(0, 1).unwrap(),
                        operand: NodeId(0),
                    }),
                ];
                assert_eq!(
                    Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(1), &nodes),
                    Err(FormulaError::ProfileIncompatibleNode {
                        profile: SemanticProfile::OriginCompleteHistoryV1,
                        node: NodeId(1),
                        family: TemporalFamily::Future,
                    })
                );
                "profile_incompatible_node"
            }
            "history-gap" => {
                let error = PositionHistoryDocument::new(
                    "gap",
                    1,
                    0,
                    2,
                    Some(ClockBinding::EventPosition),
                    vec![
                        PositionObservation::new(0, vec![], None),
                        PositionObservation::new(2, vec![], None),
                    ],
                )
                .unwrap_err();
                assert!(matches!(error, HistoryError::Gap { .. }));
                "gap"
            }
            "history-duplicate" => {
                let error = PositionHistoryDocument::new(
                    "duplicate",
                    1,
                    0,
                    0,
                    Some(ClockBinding::EventPosition),
                    vec![
                        PositionObservation::new(0, vec![], None),
                        PositionObservation::new(0, vec![], None),
                    ],
                )
                .unwrap_err();
                assert!(matches!(error, HistoryError::DuplicatePosition { .. }));
                "duplicate_position"
            }
            "history-order" => {
                let error = PositionHistoryDocument::new(
                    "order",
                    1,
                    0,
                    0,
                    Some(ClockBinding::EventPosition),
                    vec![
                        PositionObservation::new(0, vec![], None),
                        PositionObservation::new(1, vec![], None),
                        PositionObservation::new(0, vec![], None),
                    ],
                )
                .unwrap_err();
                assert!(matches!(error, HistoryError::OutOfOrder { .. }));
                "out_of_order"
            }
            "history-stale-digest" => {
                let mut value = serde_json::to_value(&histories["event-main-r1"]).unwrap();
                value["historySha256"] = serde_json::Value::String("0".repeat(64));
                assert!(serde_json::from_value::<PositionHistoryDocument>(value).is_err());
                "stale_digest"
            }
            "anchor-out-of-range" => {
                let error = evaluate_past(
                    once,
                    "anchor",
                    &histories["event-main-r1"],
                    4,
                    "corpus-map",
                    1,
                    PastEvaluationRelationInput::Original,
                    PastEvaluationLimits::default(),
                )
                .unwrap_err();
                assert!(matches!(
                    error,
                    PastEvaluationError::AnchorOutOfRange { .. }
                ));
                "anchor_out_of_range"
            }
            "fixed-sample-rounded" | "fixed-sample-absent" => {
                let fixed = cases
                    .histories
                    .iter()
                    .find(|item| item.id == "fixed-singleton")
                    .unwrap();
                let ClockWire::FixedSample {
                    epoch,
                    period,
                    unit,
                } = &fixed.clock
                else {
                    unreachable!()
                };
                let sample = (id == "fixed-sample-rounded").then_some(ClockSample {
                    instant: ExactNumber::new(101, 1).unwrap(),
                    unit: unit.clone(),
                });
                let error = PositionHistoryDocument::new(
                    "bad-fixed",
                    1,
                    0,
                    0,
                    Some(ClockBinding::FixedSample {
                        epoch: number(*epoch),
                        period: number(*period),
                        unit: unit.clone(),
                    }),
                    vec![PositionObservation::new(0, vec![], sample)],
                )
                .unwrap_err();
                if id == "fixed-sample-rounded" {
                    assert!(matches!(
                        error,
                        HistoryError::Clock(ClockError::SampleInstantMismatch { .. })
                    ));
                    "sample_mismatch"
                } else {
                    assert!(matches!(error, HistoryError::IncompleteSample { .. }));
                    "incomplete_sample"
                }
            }
            "capture-absent" | "predicate-partial" => {
                let state = if id == "capture-absent" {
                    OwnerHistoryState::Incomplete
                } else {
                    OwnerHistoryState::Failed
                };
                let error = evaluate_past(
                    once,
                    "owner-state",
                    PositionHistorySource::NonValue(state),
                    0,
                    "corpus-map",
                    1,
                    PastEvaluationRelationInput::Original,
                    PastEvaluationLimits::default(),
                )
                .unwrap_err();
                assert_eq!(error, PastEvaluationError::OwnerStatePreserved { state });
                "preserved"
            }
            "required-history-limit" | "step-limit" => {
                let limits = if id == "required-history-limit" {
                    PastEvaluationLimits {
                        max_temporal_span: 1,
                        ..PastEvaluationLimits::default()
                    }
                } else {
                    PastEvaluationLimits {
                        max_steps: 1,
                        ..PastEvaluationLimits::default()
                    }
                };
                let error = evaluate_past(
                    once,
                    "resource",
                    &histories["event-main-r1"],
                    3,
                    "corpus-map",
                    1,
                    PastEvaluationRelationInput::Original,
                    limits,
                )
                .unwrap_err();
                if id == "required-history-limit" {
                    assert!(matches!(
                        error,
                        PastEvaluationError::TemporalSpanExceeded { .. }
                    ));
                    "temporal_span_exceeded"
                } else {
                    assert!(matches!(
                        error,
                        PastEvaluationError::StepLimitExceeded { .. }
                    ));
                    "step_limit_exceeded"
                }
            }
            unknown => panic!("unreplayed corpus refusal {unknown}"),
        };
        assert_eq!(observed, value["expected"].as_str().unwrap(), "{id}");
    }
}
