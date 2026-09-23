//! Explicit, exact-pin foreign-target run for TC-191 and TC-192.
//! The example is run explicitly and requires caller-supplied
//! source and raw-output directories. It never downloads or builds a target.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tl_mltl::infinite::{
    evaluate_prefix_safety, Disposition, EvaluationLimit, EvidenceBasis, ObservationValue,
    PrefixRequest, SettlementEvidence,
};
use tl_oracle::{
    evaluate as evaluate_lasso_oracle, evaluate_closed_trace_v1, evaluate_origin_complete,
    Evidence as OracleEvidence, Formula as OracleFormula, Interval as OracleInterval,
    Lasso as OracleLasso, Limits, Verdict as OracleVerdict,
};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind, LassoTraceDocument,
    NodeId, PartialValuation, PropositionId, SemanticProfile, TemporalInterval, TraceObservation,
    UnboundedInterval, ValuationEntry,
};

const TARGET_REVISION: &str = "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a";
const COMPILER_SHA256: &str = "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12";
const MONITOR_SHA256: &str = "5743987dddb47cc01829a633e15623095c9c2aff2f8bb24e30d7f0e0f488f85f";

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn command_stdout(command: &mut Command) -> Vec<u8> {
    let result = command.output().expect("target command can start");
    assert!(
        result.status.success(),
        "target command failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    result.stdout
}

fn target_verdicts(raw: &[u8]) -> BTreeMap<(usize, usize), bool> {
    let text = std::str::from_utf8(raw).expect("target output is UTF-8");
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let (identity, value) = line.split_once(',').expect("target row has comma");
        let (formula, position) = identity.split_once(':').expect("target row has position");
        let value = match value {
            "T" => true,
            "F" => false,
            _ => panic!("unrecognized target verdict {value}"),
        };
        assert!(
            result
                .insert((formula.parse().unwrap(), position.parse().unwrap()), value)
                .is_none(),
            "duplicate target row"
        );
    }
    assert!(!result.is_empty(), "target output cannot be empty");
    result
}

fn range(start: usize, end: usize) -> OracleInterval {
    OracleInterval::Closed { start, end }
}

fn atom(id: u32) -> OracleFormula {
    OracleFormula::Atom(PropositionId(id))
}

fn bounded_formula(index: usize) -> OracleFormula {
    use OracleFormula as O;
    match index {
        0 => O::Future(range(1, 2), Box::new(atom(0))),
        1 => O::Globally(range(0, 2), Box::new(atom(1))),
        2 => O::Future(range(0, 3), Box::new(atom(2))),
        3 => O::Until(range(1, 2), Box::new(atom(0)), Box::new(atom(1))),
        4 => O::Release(range(1, 2), Box::new(atom(3)), Box::new(atom(2))),
        5 => O::Future(
            range(0, 0),
            Box::new(O::Until(range(1, 2), Box::new(atom(0)), Box::new(atom(1)))),
        ),
        _ => panic!("unreviewed bounded target formula {index}"),
    }
}

fn past_formula(index: usize) -> OracleFormula {
    use OracleFormula as O;
    match index {
        0 => O::Once(range(0, 1), Box::new(atom(0))),
        1 => O::Historically(range(0, 1), Box::new(atom(0))),
        2 => O::StrongPrevious(Box::new(atom(0))),
        3 => O::Since(range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
        4 => O::Triggered(range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
        5 => O::Once(range(1, 1), Box::new(atom(0))),
        _ => panic!("unreviewed past target formula {index}"),
    }
}

fn read_manifest(path: &str) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn run_target(
    source: &Path,
    raw_dir: &Path,
    name: &str,
    spec: &str,
    trace: &str,
    map: Option<&str>,
) -> (BTreeMap<(usize, usize), bool>, Value) {
    let compiler = source.join("compiler/c2po.py");
    let monitor = source.join("monitors/c/build/r2u2");
    let binary = raw_dir.join(format!("{name}.bin"));
    let mut compile = Command::new("python3");
    compile.arg(compiler).arg("--spec").arg(spec);
    if let Some(map) = map {
        compile.arg("--map").arg(map);
    } else {
        compile.arg("--trace").arg(trace);
    }
    let compiled = compile.arg("--output").arg(&binary).output().unwrap();
    fs::write(
        raw_dir.join(format!("{name}.compiler.stdout")),
        &compiled.stdout,
    )
    .unwrap();
    fs::write(
        raw_dir.join(format!("{name}.compiler.stderr")),
        &compiled.stderr,
    )
    .unwrap();
    assert!(
        compiled.status.success(),
        "{name} compiler exited {:?}: {}",
        compiled.status.code(),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = Command::new(monitor)
        .arg(&binary)
        .arg(trace)
        .output()
        .unwrap();
    fs::write(
        raw_dir.join(format!("{name}.r2u2.stdout")),
        &executed.stdout,
    )
    .unwrap();
    fs::write(
        raw_dir.join(format!("{name}.r2u2.stderr")),
        &executed.stderr,
    )
    .unwrap();
    assert!(
        executed.status.success(),
        "{name} monitor exited {:?}: {}",
        executed.status.code(),
        String::from_utf8_lossy(&executed.stderr)
    );
    (
        target_verdicts(&executed.stdout),
        json!({
            "compiler_exit": compiled.status.code(),
            "monitor_exit": executed.status.code(),
            "spec": spec,
            "trace": trace,
            "map": map,
        }),
    )
}

fn word_from_rows(rows: &[Value], atom_count: u32) -> Vec<BTreeMap<PropositionId, bool>> {
    rows.iter()
        .map(|row| {
            (0..atom_count)
                .map(|id| {
                    let present = row
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|value| value.as_u64() == Some(u64::from(id)));
                    (PropositionId(id), present)
                })
                .collect()
        })
        .collect()
}

fn past_word(rows: &[Value]) -> Vec<BTreeMap<PropositionId, bool>> {
    rows.iter()
        .map(|row| {
            ["p", "q"]
                .into_iter()
                .enumerate()
                .map(|(id, name)| {
                    (
                        PropositionId(u32::try_from(id).unwrap()),
                        row[name].as_bool().unwrap(),
                    )
                })
                .collect()
        })
        .collect()
}

fn bad_prefix_witness() -> Value {
    let proposition = PropositionId(1);
    let graph = InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(1),
        vec![
            InfiniteNode::new(InfiniteNodeKind::Proposition { proposition }),
            InfiniteNode::new(InfiniteNodeKind::Globally {
                interval: TemporalInterval::Unbounded(UnboundedInterval::new(0)),
                operand: NodeId(0),
            }),
        ],
    )
    .unwrap();
    let observation = |position, value| TraceObservation {
        position,
        valuation: PartialValuation::new(
            "map".to_owned(),
            &[proposition],
            vec![ValuationEntry { proposition, value }],
        )
        .unwrap(),
    };
    let trace = LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        vec![proposition],
        vec![observation(0, ObservationValue::False)],
        vec![observation(1, ObservationValue::True)],
    )
    .unwrap();
    let graph_id = graph.content_identity().unwrap();
    let result = evaluate_prefix_safety(&PrefixRequest {
        formula: &graph,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: trace.propositions(),
        observations: trace.prefix(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    assert_eq!(result.disposition, Disposition::Refuted);
    assert_eq!(result.basis, EvidenceBasis::BadPrefix);
    let Some(SettlementEvidence::BadPrefix(witness)) = result.evidence else {
        panic!("refutation must carry bad-prefix evidence")
    };
    assert_eq!(witness.violation_position, 0);
    let oracle_lasso = OracleLasso::new(
        vec![[(proposition, OracleEvidence::False)].into()],
        vec![[(proposition, OracleEvidence::True)].into()],
    )
    .unwrap();
    let oracle_result = evaluate_lasso_oracle(
        &OracleFormula::Globally(
            OracleInterval::Unbounded { start: 0 },
            Box::new(OracleFormula::Atom(proposition)),
        ),
        &[],
        &oracle_lasso,
        0,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(oracle_result.verdict, OracleVerdict::Refuted);
    json!({"disposition":"refuted","basis":"bad_prefix","violation_position":witness.violation_position,"oracle":"refuted","target_case":"r2u2-globally-counterexample-v1","target_position":0,"target_verdict":false})
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
    let revision = command_stdout(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&source),
    );
    assert_eq!(
        std::str::from_utf8(&revision).unwrap().trim(),
        TARGET_REVISION
    );
    let dirty = command_stdout(
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&source),
    );
    assert!(dirty.is_empty(), "target source must be clean");
    assert_eq!(
        digest(&fs::read(source.join("compiler/c2po.py")).unwrap()),
        COMPILER_SHA256
    );
    assert_eq!(
        digest(&fs::read(source.join("monitors/c/build/r2u2")).unwrap()),
        MONITOR_SHA256
    );

    let (bounded, bounded_run) = run_target(
        &source,
        &raw_dir,
        "bounded",
        "corpus/r2u2-v4.2/formulas.c2po",
        "corpus/r2u2-v4.2/trace.csv",
        Some("corpus/r2u2-v4.2/signals.map"),
    );
    let (past, past_run) = run_target(
        &source,
        &raw_dir,
        "past",
        "corpus/past-c2po-v1/target-4.2/past.c2po",
        "corpus/past-c2po-v1/target-4.2/trace.csv",
        None,
    );
    let (unsafe_since, unsafe_run) = run_target(
        &source,
        &raw_dir,
        "unsafe-since",
        "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
        "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
        None,
    );
    let bounded_manifest = read_manifest("corpus/r2u2-v4.2/manifest.json");
    let bounded_cells = word_from_rows(bounded_manifest["trace"].as_array().unwrap(), 4);
    let mut rows = Vec::new();
    let mut observed_bounded = BTreeSet::new();
    for case in bounded_manifest["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let index = case["formulaIndex"].as_u64().unwrap() as usize;
        let at = case["expected"]["verdictTime"].as_u64().unwrap() as usize;
        assert!(
            observed_bounded.insert((index, at)),
            "duplicate reviewed bounded cell"
        );
        let reference = evaluate_closed_trace_v1(
            &bounded_formula(index),
            &bounded_cells,
            at,
            Limits::default(),
        )
        .unwrap();
        assert_eq!(
            reference,
            case["expected"]["verdict"].as_bool().unwrap(),
            "{id} oracle fixture"
        );
        let target = *bounded.get(&(index, at)).expect("bounded target cell");
        assert_eq!(target, reference, "{id} target/oracle mismatch");
        rows.push(json!({"case":id,"family":"bounded","position":at,"classification":"agreement","oracle":reference,"target":target}));
    }
    assert_eq!(rows.len(), 8);
    assert_eq!(
        bounded.get(&(1, 0)),
        Some(&false),
        "the target must refute the bounded form of the bad-prefix witness"
    );

    let past_manifest = read_manifest("corpus/past-c2po-v1/manifest.json");
    let past_cells = past_word(past_manifest["trace"].as_array().unwrap());
    let mut past_cells_seen = BTreeSet::new();
    for case in past_manifest["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let index = case["targetFormulaId"].as_u64().unwrap() as usize;
        for at in 0..past_cells.len() {
            assert!(past_cells_seen.insert((index, at)));
            let reference =
                evaluate_origin_complete(&past_formula(index), &past_cells, at, Limits::default())
                    .unwrap();
            assert_eq!(
                reference,
                case["expectedSource"][at].as_bool().unwrap(),
                "{id} oracle fixture"
            );
            let target = *past.get(&(index, at)).expect("past target cell");
            let classification = if matches!(index, 3 | 4) {
                "unsupported_mapping"
            } else {
                "agreement"
            };
            if classification == "agreement" {
                assert_eq!(target, reference, "{id} target/oracle mismatch");
            }
            rows.push(json!({"case":id,"family":"past","position":at,"classification":classification,"oracle":reference,"target":target}));
        }
    }
    assert_eq!(past_cells_seen.len(), 18);

    let unsafe_cells = vec![
        [(PropositionId(0), false), (PropositionId(1), true)].into(),
        [(PropositionId(0), false), (PropositionId(1), false)].into(),
        [(PropositionId(0), true), (PropositionId(1), false)].into(),
    ];
    let unsafe_oracle =
        evaluate_origin_complete(&past_formula(3), &unsafe_cells, 2, Limits::default()).unwrap();
    let unsafe_target = *unsafe_since.get(&(0, 2)).expect("unsafe target cell");
    assert!(
        !unsafe_oracle && unsafe_target,
        "known target-origin mismatch must remain visible"
    );
    rows.push(json!({"case":"unsafe-since","family":"past","position":2,"classification":"unsupported_mapping","oracle":unsafe_oracle,"target":unsafe_target}));

    let witness = bad_prefix_witness();
    let artifacts: BTreeMap<String, String> = fs::read_dir(&raw_dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                digest(&fs::read(path).unwrap()),
            )
        })
        .collect();
    assert_eq!(artifacts.len(), 15);
    println!(
        "TL_CAMPAIGN_LIVE_TARGET {}",
        json!({
            "schema":"tl-mltl.live-r2u2/v1",
            "source_revision":TARGET_REVISION,
            "compiler_sha256":COMPILER_SHA256,
            "monitor_sha256":MONITOR_SHA256,
            "license":"Apache-2.0",
            "bounded_cells":8,
            "past_cells":18,
            "unsafe_cells":1,
            "classifications":rows,
            "bad_prefix":witness,
            "commands":{
                "compiler":"python3 compiler/c2po.py --spec <input> --trace/--map <input> --output <raw>.bin",
                "monitor":"monitors/c/build/r2u2 <raw>.bin <trace>",
            },
            "runs":{"bounded":bounded_run,"past":past_run,"unsafe-since":unsafe_run},
            "artifacts":artifacts,
        })
    );
}
