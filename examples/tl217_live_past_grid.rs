//! Opt-in TL-217 past-operator grid against pinned C2PO/R2U2.
//!
//! The target's output time axis is not assumed to equal the trace length:
//! missing input positions and extra target positions remain visible in the
//! report. This example never runs from CI or an ordinary library test.

use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, map_past_to_c2po, ClockBinding, MappingSourceIdentity, MappingSourceState,
    PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation, TargetOriginContract, ToolIdentity,
};
use tl_oracle::{
    evaluate_origin_complete, Formula as OracleFormula, Interval as OracleInterval,
    Limits as OracleLimits,
};
use tl_syntax::{
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration,
    PastOperatorKind, PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId,
};

const TARGET_REVISION: &str = "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a";
const COMPILER_SHA256: &str = "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12";
const MONITOR_SHA256: &str = "5743987dddb47cc01829a633e15623095c9c2aff2f8bb24e30d7f0e0f488f85f";
const STEPS: usize = 6;
const INTERVALS: [(&str, u32, u32); 6] = [
    ("zero-singleton", 0, 0),
    ("zero-unit", 0, 1),
    ("zero-upper", 0, 2),
    ("nonzero-singleton-one", 1, 1),
    ("nonzero-range", 1, 2),
    ("nonzero-singleton", 2, 2),
];

#[derive(Serialize)]
struct TargetRun {
    compiler_exit: Option<i32>,
    monitor_exit: Option<i32>,
    formula_count: usize,
    trace_positions: usize,
}

#[derive(Serialize)]
struct RunReport {
    target: TargetRun,
    extra_target_positions: usize,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum MappingReport {
    Admitted { expression_sha256: String },
    Refused { reason: String },
}

#[derive(Serialize)]
struct CellReport {
    case: String,
    operator: &'static str,
    interval: Option<(u32, u32)>,
    depth: usize,
    trace: &'static str,
    position: usize,
    origin_hazard: bool,
    tl: bool,
    oracle: bool,
    target: Option<bool>,
    mapping: MappingReport,
    classification: &'static str,
}

#[derive(Serialize)]
struct GridReport {
    schema: &'static str,
    source_revision: String,
    source_state: &'static str,
    cargo_lock_sha256: String,
    target_revision: &'static str,
    compiler_sha256: &'static str,
    monitor_sha256: &'static str,
    license: &'static str,
    intervals: [(&'static str, u32, u32); 6],
    steps: usize,
    formula_trace_cases: usize,
    per_step_cells: usize,
    classifications: BTreeMap<String, usize>,
    unexplained_admitted_cells: usize,
    runs: BTreeMap<String, RunReport>,
    rows: Vec<CellReport>,
    artifacts: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operator {
    Once,
    Historically,
    Since,
    Triggered,
    Previous,
}

impl Operator {
    const INTERVALLED: [Self; 4] = [Self::Once, Self::Historically, Self::Since, Self::Triggered];

    fn name(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Historically => "historically",
            Self::Since => "since",
            Self::Triggered => "triggered",
            Self::Previous => "previous",
        }
    }
}

struct Case {
    id: String,
    operator: Operator,
    interval: Option<(u32, u32)>,
    depth: usize,
    expression: String,
    nodes: Vec<Node>,
    oracle: OracleFormula,
}

fn make_case(operator: Operator, interval: Option<(u32, u32)>, depth: usize) -> Case {
    assert!((1..=3).contains(&depth));
    assert_eq!(operator == Operator::Previous, interval.is_none());
    let mut nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
    ];
    let mut current = NodeId(0);
    let mut expression = "p".to_owned();
    let mut oracle = OracleFormula::Atom(PropositionId(0));
    for _ in 0..depth {
        let left = current;
        let range = interval.map(|(a, b)| Interval::new(a, b).unwrap());
        let oracle_range = interval.map(|(a, b)| OracleInterval::Closed {
            start: a as usize,
            end: b as usize,
        });
        let node = match operator {
            Operator::Once => {
                let (a, b) = interval.unwrap();
                expression = format!("O[{a},{b}]({expression})");
                oracle = OracleFormula::Once(oracle_range.unwrap(), Box::new(oracle));
                NodeKind::Once {
                    interval: range.unwrap(),
                    operand: left,
                }
            }
            Operator::Historically => {
                let (a, b) = interval.unwrap();
                expression = format!("H[{a},{b}]({expression})");
                oracle = OracleFormula::Historically(oracle_range.unwrap(), Box::new(oracle));
                NodeKind::Historically {
                    interval: range.unwrap(),
                    operand: left,
                }
            }
            Operator::Since => {
                let (a, b) = interval.unwrap();
                expression = format!("({expression} S[{a},{b}] q)");
                oracle = OracleFormula::Since(
                    oracle_range.unwrap(),
                    Box::new(oracle),
                    Box::new(OracleFormula::Atom(PropositionId(1))),
                );
                NodeKind::Since {
                    interval: range.unwrap(),
                    left,
                    right: NodeId(1),
                }
            }
            Operator::Triggered => {
                let (a, b) = interval.unwrap();
                expression = format!("(!((!{expression}) S[{a},{b}] (!q)))");
                oracle = OracleFormula::Triggered(
                    oracle_range.unwrap(),
                    Box::new(oracle),
                    Box::new(OracleFormula::Atom(PropositionId(1))),
                );
                NodeKind::Triggered {
                    interval: range.unwrap(),
                    left,
                    right: NodeId(1),
                }
            }
            Operator::Previous => {
                expression = format!("O[1,1]({expression})");
                oracle = OracleFormula::StrongPrevious(Box::new(oracle));
                NodeKind::StrongPrevious { operand: left }
            }
        };
        nodes.push(Node::new(node));
        current = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    }
    let interval_name = interval
        .map(|(a, b)| format!("{a}-{b}"))
        .unwrap_or_else(|| "none".to_owned());
    Case {
        id: format!("{}-{interval_name}-d{depth}", operator.name()),
        operator,
        interval,
        depth,
        expression,
        nodes,
        oracle,
    }
}

fn groups() -> Vec<(String, Vec<Case>, usize)> {
    let mut result = INTERVALS
        .into_iter()
        .map(|(name, a, b)| {
            let cases = Operator::INTERVALLED
                .into_iter()
                .flat_map(|operator| {
                    (1..=3).map(move |depth| make_case(operator, Some((a, b)), depth))
                })
                .collect();
            (name.to_owned(), cases, b as usize)
        })
        .collect::<Vec<_>>();
    result.push((
        "previous".to_owned(),
        (1..=3)
            .map(|depth| make_case(Operator::Previous, None, depth))
            .collect(),
        1,
    ));
    result
}

fn trace(kind: &str, boundary: usize) -> Vec<(bool, bool)> {
    (0..STEPS)
        .map(|position| match kind {
            "all-true" => (true, true),
            "all-false" => (false, false),
            "boundary-toggle" => {
                let p = position >= boundary;
                (p, !p)
            }
            _ => panic!("unknown trace kind {kind}"),
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn stdout(mut command: Command) -> Vec<u8> {
    let output = command.output().expect("command can start");
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn pinned_target(source: &Path) {
    let mut git = Command::new("git");
    git.args(["rev-parse", "HEAD"]).current_dir(source);
    assert_eq!(
        String::from_utf8(stdout(git)).unwrap().trim(),
        TARGET_REVISION
    );
    let mut git = Command::new("git");
    git.args(["status", "--porcelain"]).current_dir(source);
    assert!(stdout(git).is_empty(), "target source must be clean");
    assert_eq!(
        sha256(&fs::read(source.join("compiler/c2po.py")).unwrap()),
        COMPILER_SHA256
    );
    assert_eq!(
        sha256(&fs::read(source.join("monitors/c/build/r2u2")).unwrap()),
        MONITOR_SHA256
    );
}

fn parse_target(raw: &[u8]) -> BTreeMap<(usize, usize), bool> {
    let mut verdicts = BTreeMap::new();
    for line in std::str::from_utf8(raw)
        .expect("target output UTF-8")
        .lines()
    {
        let (id, value) = line.split_once(',').expect("target row comma");
        let (formula, position) = id.split_once(':').expect("target row formula:position");
        let value = match value {
            "T" => true,
            "F" => false,
            _ => panic!("target row has unknown verdict {value}"),
        };
        let key = (formula.parse().unwrap(), position.parse().unwrap());
        assert!(
            verdicts.insert(key, value).is_none(),
            "duplicate target verdict"
        );
    }
    assert!(!verdicts.is_empty(), "target output is empty");
    verdicts
}

fn write_run(
    raw_dir: &Path,
    id: &str,
    cases: &[Case],
    trace: &[(bool, bool)],
    source: &Path,
) -> (BTreeMap<(usize, usize), bool>, TargetRun) {
    let spec = raw_dir.join(format!("{id}.c2po"));
    let trace_path = raw_dir.join(format!("{id}.csv"));
    let binary = raw_dir.join(format!("{id}.bin"));
    let spec_bytes = format!(
        "INPUT\n p,q: bool;\nPTSPEC\n{}\n",
        cases
            .iter()
            .map(|case| format!(" {};", case.expression))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let trace_bytes = format!(
        "# p,q\n{}",
        trace
            .iter()
            .map(|(p, q)| format!("{},{}\n", u8::from(*p), u8::from(*q)))
            .collect::<String>()
    );
    fs::write(&spec, spec_bytes).unwrap();
    fs::write(&trace_path, trace_bytes).unwrap();
    let compiler = Command::new("python3")
        .arg(source.join("compiler/c2po.py"))
        .args(["--spec"])
        .arg(&spec)
        .args(["--trace"])
        .arg(&trace_path)
        .args(["--output"])
        .arg(&binary)
        .output()
        .unwrap();
    fs::write(
        raw_dir.join(format!("{id}.compiler.stdout")),
        &compiler.stdout,
    )
    .unwrap();
    fs::write(
        raw_dir.join(format!("{id}.compiler.stderr")),
        &compiler.stderr,
    )
    .unwrap();
    assert!(
        compiler.status.success(),
        "{id}: C2PO failed: {}",
        String::from_utf8_lossy(&compiler.stderr)
    );
    let monitor = Command::new(source.join("monitors/c/build/r2u2"))
        .arg(&binary)
        .arg(&trace_path)
        .output()
        .unwrap();
    fs::write(
        raw_dir.join(format!("{id}.monitor.stdout")),
        &monitor.stdout,
    )
    .unwrap();
    fs::write(
        raw_dir.join(format!("{id}.monitor.stderr")),
        &monitor.stderr,
    )
    .unwrap();
    assert!(
        monitor.status.success(),
        "{id}: R2U2 failed: {}",
        String::from_utf8_lossy(&monitor.stderr)
    );
    let verdicts = parse_target(&monitor.stdout);
    assert!(
        verdicts.keys().all(|(formula, _)| *formula < cases.len()),
        "{id}: unknown target formula ID"
    );
    (
        verdicts,
        TargetRun {
            compiler_exit: compiler.status.code(),
            monitor_exit: monitor.status.code(),
            formula_count: cases.len(),
            trace_positions: trace.len(),
        },
    )
}

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap()
}

fn origin_contract() -> TargetOriginContract {
    TargetOriginContract {
        target: ToolIdentity {
            name: "C2PO".to_owned(),
            version: "C2PO v4.1.0".to_owned(),
            executable_sha256: COMPILER_SHA256.to_owned(),
            configuration_sha256: sha256(include_bytes!(
                "../corpus/past-c2po-v1/target-4.2/past.c2po"
            )),
        },
        evidence_sha256: sha256(include_bytes!(
            "../corpus/past-c2po-v1/target-4.2/r2u2.stdout"
        )),
        admitted_operators: [
            PastOperatorKind::Once,
            PastOperatorKind::Historically,
            PastOperatorKind::Since,
            PastOperatorKind::Triggered,
            PastOperatorKind::StrongPrevious,
        ]
        .into_iter()
        .collect(),
    }
}

fn source_verdict(
    case: &Case,
    rows: &[(bool, bool)],
    position: usize,
) -> (bool, bool, MappingReport) {
    let observations = rows
        .iter()
        .enumerate()
        .map(|(index, (p, q))| {
            let propositions = [(PropositionId(0), *p), (PropositionId(1), *q)]
                .into_iter()
                .filter_map(|(id, value)| value.then_some(id))
                .collect();
            PositionObservation::new(index as u64, propositions, None)
        })
        .collect();
    let history = PositionHistoryDocument::new(
        "tl217-grid",
        1,
        0,
        (rows.len() - 1) as u64,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap();
    let graph = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(case.nodes.len() - 1).unwrap()),
        &case.nodes,
    )
    .unwrap();
    let tl = evaluate_past(
        graph,
        &case.id,
        &history,
        position as u64,
        "map",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap()
    .verdict;
    let word = rows
        .iter()
        .map(|(p, q)| [(PropositionId(0), *p), (PropositionId(1), *q)].into())
        .collect::<Vec<BTreeMap<_, _>>>();
    let oracle =
        evaluate_origin_complete(&case.oracle, &word, position, OracleLimits::default()).unwrap();
    let document = FormulaDocument::from_formula_v2(graph).unwrap();
    let bytes = document.canonical_json_bytes().unwrap();
    let mapping = map_past_to_c2po(
        graph,
        &case.id,
        &bytes,
        MappingSourceIdentity {
            revision: "tl217-grid".to_owned(),
            state: MappingSourceState::Clean,
        },
        &catalog(),
        &origin_contract(),
        100,
    );
    let mapping = match mapping {
        Ok(mapped) => {
            assert_eq!(
                mapped.expression, case.expression,
                "{} mapping expression",
                case.id
            );
            MappingReport::Admitted {
                expression_sha256: mapped.output_sha256,
            }
        }
        Err(refusal) => MappingReport::Refused {
            reason: format!("{refusal:?}"),
        },
    };
    (tl, oracle, mapping)
}

fn main() {
    let source =
        PathBuf::from(std::env::var("TL_MLTL_C2PO_SOURCE").expect("explicit target source"));
    let raw_dir =
        PathBuf::from(std::env::var("TL_MLTL_LIVE_RAW_DIR").expect("explicit raw directory"));
    assert!(source.is_absolute() && raw_dir.is_absolute());
    assert!(!raw_dir.starts_with(env!("CARGO_MANIFEST_DIR")));
    fs::create_dir_all(&raw_dir).unwrap();
    assert!(
        fs::read_dir(&raw_dir).unwrap().next().is_none(),
        "raw directory must be empty"
    );
    pinned_target(&source);
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut git = Command::new("git");
    git.args(["rev-parse", "HEAD"]).current_dir(repo);
    let source_revision = String::from_utf8(stdout(git)).unwrap().trim().to_owned();
    let mut git = Command::new("git");
    git.args(["status", "--porcelain"]).current_dir(repo);
    let source_state = if stdout(git).is_empty() {
        "clean"
    } else {
        "dirty"
    };
    let mut cases_total = 0_usize;
    let mut cells_total = 0_usize;
    let mut classifications = BTreeMap::<String, usize>::new();
    let mut rows = Vec::new();
    let mut runs = BTreeMap::new();
    let mut unexplained = 0_usize;
    for (group, cases, boundary) in groups() {
        for trace_kind in ["all-true", "all-false", "boundary-toggle"] {
            let id = format!("{group}-{trace_kind}");
            let trace = trace(trace_kind, boundary);
            let (target, run) = write_run(&raw_dir, &id, &cases, &trace, &source);
            let extra = target
                .keys()
                .filter(|(_, position)| *position >= STEPS)
                .count();
            runs.insert(
                id.clone(),
                RunReport {
                    target: run,
                    extra_target_positions: extra,
                },
            );
            for (formula_id, case) in cases.iter().enumerate() {
                cases_total += 1;
                for position in 0..STEPS {
                    cells_total += 1;
                    let (tl, oracle, mapping) = source_verdict(case, &trace, position);
                    assert_eq!(tl, oracle, "{} oracle at {position}", case.id);
                    let observed = target.get(&(formula_id, position)).copied();
                    let admitted = matches!(mapping, MappingReport::Admitted { .. });
                    let classification = match (admitted, observed) {
                        (false, _) => "unsupported_mapping",
                        (true, None) => {
                            unexplained += 1;
                            "nonconclusive_target_missing"
                        }
                        (true, Some(value)) if value == tl => "agreement",
                        (true, Some(_)) => {
                            unexplained += 1;
                            "semantic_mismatch"
                        }
                    };
                    *classifications
                        .entry(classification.to_owned())
                        .or_default() += 1;
                    rows.push(CellReport {
                        case: case.id.clone(),
                        operator: case.operator.name(),
                        interval: case.interval,
                        depth: case.depth,
                        trace: trace_kind,
                        position,
                        origin_hazard: (matches!(
                            case.operator,
                            Operator::Historically | Operator::Triggered
                        ) && case.interval.is_some_and(|(_, b)| {
                            position < (b as usize).saturating_mul(case.depth)
                        })) || (case.operator == Operator::Previous
                            && position < case.depth),
                        tl,
                        oracle,
                        target: observed,
                        mapping,
                        classification,
                    });
                }
            }
        }
    }
    assert_eq!(cases_total, 225);
    assert_eq!(cells_total, 1350);
    let artifacts = fs::read_dir(&raw_dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_string_lossy().to_string(),
                sha256(&fs::read(path).unwrap()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(artifacts.len(), 147);
    let report = GridReport {
        schema: "tl-mltl.r2u2-past-grid/v1",
        source_revision,
        source_state,
        cargo_lock_sha256: sha256(&fs::read(repo.join("Cargo.lock")).unwrap()),
        target_revision: TARGET_REVISION,
        compiler_sha256: COMPILER_SHA256,
        monitor_sha256: MONITOR_SHA256,
        license: "Apache-2.0",
        intervals: INTERVALS,
        steps: STEPS,
        formula_trace_cases: cases_total,
        per_step_cells: cells_total,
        classifications,
        unexplained_admitted_cells: unexplained,
        runs,
        rows,
        artifacts,
    };
    println!(
        "TL217_PAST_GRID {}",
        serde_json::to_string(&report).unwrap()
    );
    assert_eq!(
        unexplained, 0,
        "admitted target mismatches or missing cells remain unexplained"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_axes_are_complete_and_unique() {
        let groups = groups();
        assert_eq!(groups.len(), 7);
        assert_eq!(
            groups
                .iter()
                .map(|(_, cases, _)| cases.len())
                .sum::<usize>(),
            75
        );
        let ids = groups
            .iter()
            .flat_map(|(_, cases, _)| cases.iter().map(|case| case.id.as_str()))
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 75);
        assert_eq!(
            groups
                .iter()
                .map(|(_, cases, _)| cases.len() * 3 * STEPS)
                .sum::<usize>(),
            1350
        );
        assert_eq!(
            trace("boundary-toggle", 2),
            vec![
                (false, true),
                (false, true),
                (true, false),
                (true, false),
                (true, false),
                (true, false)
            ]
        );
    }

    #[test]
    fn parser_refuses_duplicate_or_unknown_target_rows() {
        assert_eq!(parse_target(b"0:0,T\n0:1,F\n").len(), 2);
        assert!(std::panic::catch_unwind(|| parse_target(b"0:0,T\n0:0,F\n")).is_err());
        assert!(std::panic::catch_unwind(|| parse_target(b"0:0,unknown\n")).is_err());
    }
}
