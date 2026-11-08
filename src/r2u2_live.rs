//! Live R2U2 differential over a generated past-time grid (TL-217).
//!
//! The grid is every past operator, nested to depth 1-3, with four interval
//! kinds and three trace classes. Each case is evaluated by this crate's own
//! origin-complete past evaluator and exported through
//! [`map_past_to_c2po`]. A case the exporter refuses is
//! a typed `unsupported` cell. The admitted cases are written as C2PO specs,
//! compiled and run by a real R2U2 checkout, and compared per step.
//!
//! A disagreement inside the origin-exposed window (positions before the
//! formula's total look-back has been reached) is a documented profile
//! difference; one outside it is unexplained and fails the run. Nothing here
//! invents a target verdict: when R2U2 is not present the report says so and
//! every admitted cell stays `not_run`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

use crate::{
    evaluate_past, map_past_to_c2po, ClockBinding, MappingSourceIdentity, MappingSourceState,
    PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation, TargetOriginContract,
};

/// Schema of the differential report.
pub const REPORT_SCHEMA: &str = "tl-mltl.r2u2-past-differential/v1";
/// Steps in every generated trace.
pub const TRACE_LEN: u64 = 8;

/// A past operator the grid exercises.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Op {
    /// `O` once.
    Once,
    /// `H` historically.
    Historically,
    /// `Y` strong previous.
    Previous,
    /// `S` since.
    Since,
    /// `T` triggered.
    Triggered,
}

const OPS: [Op; 5] = [
    Op::Once,
    Op::Historically,
    Op::Previous,
    Op::Since,
    Op::Triggered,
];

/// Interval kinds: `[0,0]`, `[0,b]`, `[a,b]`, `a=b`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum IntervalKind {
    /// `[0,0]`.
    Zero,
    /// `[0,2]`.
    FromZero,
    /// `[1,3]`.
    Window,
    /// `[2,2]`.
    Point,
}

const KINDS: [IntervalKind; 4] = [
    IntervalKind::Zero,
    IntervalKind::FromZero,
    IntervalKind::Window,
    IntervalKind::Point,
];

impl IntervalKind {
    const fn bounds(self) -> (u32, u32) {
        match self {
            Self::Zero => (0, 0),
            Self::FromZero => (0, 2),
            Self::Window => (1, 3),
            Self::Point => (2, 2),
        }
    }
}

/// Trace classes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TraceClass {
    /// `p` and `q` always true.
    AllTrue,
    /// `p` and `q` always false.
    AllFalse,
    /// `p` true through the interval's lower bound, `q` true only at its upper bound.
    BoundaryToggle,
}

const CLASSES: [TraceClass; 3] = [
    TraceClass::AllTrue,
    TraceClass::AllFalse,
    TraceClass::BoundaryToggle,
];

/// The trace of `(p, q)` rows for a group.
pub fn trace_rows(class: TraceClass, kind: IntervalKind) -> Vec<[bool; 2]> {
    let (a, b) = kind.bounds();
    (0..TRACE_LEN)
        .map(|i| match class {
            TraceClass::AllTrue => [true, true],
            TraceClass::AllFalse => [false, false],
            TraceClass::BoundaryToggle => [i <= u64::from(a), i == u64::from(b)],
        })
        .collect()
}

/// Cases sharing one trace run in one R2U2 invocation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Group {
    /// Trace class.
    pub class: TraceClass,
    /// Interval kind that parameterised the trace.
    pub kind: IntervalKind,
}

impl Group {
    /// Stable directory name.
    pub fn name(self) -> String {
        format!("{:?}-{:?}", self.class, self.kind).to_lowercase()
    }
}

/// One generated formula over one trace group.
#[derive(Clone, Debug)]
pub struct GridCase {
    /// Stable id: operators innermost-first, interval kind, trace class.
    pub id: String,
    /// Rendered formula text.
    pub text: String,
    /// Operators, outermost last.
    pub ops: Vec<Op>,
    /// Trace group.
    pub group: Group,
    nodes: Vec<Node>,
}

impl GridCase {
    /// Positions before this are still inside some operator's look-back.
    pub fn origin_reach(&self) -> u64 {
        let (_, b) = self.group.kind.bounds();
        self.ops
            .iter()
            .map(|op| if *op == Op::Previous { 1 } else { u64::from(b) })
            .sum()
    }
}

fn build(ops: &[Op], kind: IntervalKind) -> (Vec<Node>, String) {
    let (a, b) = kind.bounds();
    let range = Interval::new(a, b).expect("grid intervals are ordered");
    let node_id = |nodes: &Vec<Node>| NodeId(u32::try_from(nodes.len() - 1).expect("small grid"));
    let mut nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    let mut text = "p".to_owned();
    let mut last = NodeId(0);
    for (depth, op) in ops.iter().enumerate() {
        match op {
            Op::Once => {
                nodes.push(Node::new(NodeKind::Once {
                    interval: range,
                    operand: last,
                }));
                text = format!("O[{a},{b}]({text})");
            }
            Op::Historically => {
                nodes.push(Node::new(NodeKind::Historically {
                    interval: range,
                    operand: last,
                }));
                text = format!("H[{a},{b}]({text})");
            }
            Op::Previous => {
                nodes.push(Node::new(NodeKind::StrongPrevious { operand: last }));
                text = format!("Y({text})");
            }
            Op::Since | Op::Triggered => {
                // The right operand is the nest so far; innermost it is `q`.
                let (right, inner) = if depth == 0 {
                    nodes.push(Node::new(NodeKind::Proposition {
                        proposition: PropositionId(1),
                    }));
                    (node_id(&nodes), "q".to_owned())
                } else {
                    (last, text.clone())
                };
                let (node, sym) = if *op == Op::Since {
                    (
                        NodeKind::Since {
                            interval: range,
                            left: NodeId(0),
                            right,
                        },
                        "S",
                    )
                } else {
                    (
                        NodeKind::Triggered {
                            interval: range,
                            left: NodeId(0),
                            right,
                        },
                        "T",
                    )
                };
                nodes.push(Node::new(node));
                text = format!("(p {sym}[{a},{b}] {inner})");
            }
        }
        last = node_id(&nodes);
    }
    (nodes, text)
}

/// Every grid case, in a fixed order.
pub fn generate_grid() -> Vec<GridCase> {
    let mut cases = Vec::new();
    for depth in 1..=3usize {
        let mut shapes: Vec<Vec<Op>> = vec![Vec::new()];
        for _ in 0..depth {
            shapes = shapes
                .into_iter()
                .flat_map(|s| {
                    OPS.iter().map(move |o| {
                        let mut n = s.clone();
                        n.push(*o);
                        n
                    })
                })
                .collect();
        }
        for ops in shapes {
            for kind in KINDS {
                // Y carries no interval: skip its duplicate interval kinds.
                if ops.iter().all(|o| *o == Op::Previous) && kind != IntervalKind::Zero {
                    continue;
                }
                let (nodes, text) = build(&ops, kind);
                for class in CLASSES {
                    let tag: String = ops
                        .iter()
                        .map(|o| format!("{o:?}").to_lowercase())
                        .collect::<Vec<_>>()
                        .join(">");
                    cases.push(GridCase {
                        id: format!("{tag}/{kind:?}/{class:?}").to_lowercase(),
                        text: text.clone(),
                        ops: ops.clone(),
                        group: Group { class, kind },
                        nodes: nodes.clone(),
                    });
                }
            }
        }
    }
    cases
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
    .expect("fixed catalog")
}

/// Source verdict of `case` at every trace position.
pub fn expected_verdicts(case: &GridCase) -> Result<Vec<bool>, String> {
    let root = NodeId(u32::try_from(case.nodes.len() - 1).map_err(|e| e.to_string())?);
    let graph = Formula::new(SemanticProfile::OriginCompleteHistoryV1, root, &case.nodes)
        .map_err(|e| format!("{e:?}"))?;
    let rows = trace_rows(case.group.class, case.group.kind);
    let observations = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let props = [(r[0], PropositionId(0)), (r[1], PropositionId(1))]
                .into_iter()
                .filter_map(|(v, id)| v.then_some(id))
                .collect();
            PositionObservation::new(i as u64, props, None)
        })
        .collect();
    let history = PositionHistoryDocument::new(
        "grid",
        1,
        0,
        TRACE_LEN - 1,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .map_err(|e| format!("{e:?}"))?;
    (0..TRACE_LEN)
        .map(|at| {
            evaluate_past(
                graph,
                case.id.clone(),
                &history,
                at,
                "map",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            )
            .map(|r| r.verdict)
            .map_err(|e| format!("{e:?}"))
        })
        .collect()
}

/// The exporter's C2PO expression for `case`, or its typed refusal name.
pub fn export(case: &GridCase) -> Result<String, String> {
    let root = NodeId(u32::try_from(case.nodes.len() - 1).map_err(|e| e.to_string())?);
    let graph = Formula::new(SemanticProfile::OriginCompleteHistoryV1, root, &case.nodes)
        .map_err(|e| format!("{e:?}"))?;
    map_past_to_c2po(
        graph,
        &case.id,
        case.text.as_bytes(),
        MappingSourceIdentity {
            revision: "grid".to_owned(),
            state: MappingSourceState::Clean,
        },
        &catalog(),
        &TargetOriginContract::reviewed_r2u2_4_2(),
        1000,
    )
    .map(|m| m.expression)
    .map_err(|e| {
        let d = format!("{e:?}");
        d.split(['(', ' ', '{'])
            .next()
            .unwrap_or("Unknown")
            .to_owned()
    })
}

/// Lower-case hex SHA-256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// A group's R2U2 inputs.
pub struct GroupInputs {
    /// `spec.c2po` text.
    pub spec: String,
    /// `trace.csv` text.
    pub trace: String,
    /// Case ids in R2U2 formula-id order.
    pub case_ids: Vec<String>,
}

/// Build the spec and trace for `group` from its admitted cases.
pub fn group_inputs(group: Group, admitted: &[(&GridCase, String)]) -> GroupInputs {
    let mut spec = String::from("INPUT\n  p,q: bool;\nPTSPEC\n");
    for (_, expr) in admitted {
        spec.push_str(&format!("  {expr};\n"));
    }
    let mut trace = String::from("# p,q\n");
    for r in trace_rows(group.class, group.kind) {
        trace.push_str(&format!("{},{}\n", u8::from(r[0]), u8::from(r[1])));
    }
    GroupInputs {
        spec,
        trace,
        case_ids: admitted.iter().map(|(c, _)| c.id.clone()).collect(),
    }
}

/// Parse `formula:position,T|F` lines; other lines are ignored.
pub fn parse_r2u2_stdout(text: &str) -> BTreeMap<(usize, u64), bool> {
    text.lines()
        .filter_map(|l| {
            let (id, v) = l.trim().split_once(',')?;
            let (f, p) = id.split_once(':')?;
            let v = match v {
                "T" => true,
                "F" => false,
                _ => return None,
            };
            Some(((f.parse().ok()?, p.parse().ok()?), v))
        })
        .collect()
}

/// Per-case comparison result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    /// The exporter refused; never compared.
    Unsupported,
    /// Admitted but R2U2 did not run.
    NotRun,
    /// Every step agrees.
    Agree,
    /// Disagrees only inside the origin-exposed window: documented profile difference.
    OriginProfileDifference,
    /// R2U2 produced no verdict for a step.
    MissingTarget,
    /// Disagrees outside the origin-exposed window.
    Unexplained,
}

/// One case row.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRow {
    /// Case id.
    pub id: String,
    /// Formula text.
    pub formula: String,
    /// Trace group name.
    pub group: String,
    /// Outcome.
    pub status: CaseStatus,
    /// Refusal name for an unsupported case.
    pub refusal: Option<String>,
    /// Source verdicts as T/F.
    pub expected: String,
    /// R2U2 verdicts as T/F/? (empty when not run).
    pub target: String,
    /// Steps where they differ or R2U2 is silent.
    pub differing_steps: Vec<u64>,
}

fn tf(b: bool) -> char {
    if b {
        'T'
    } else {
        'F'
    }
}

/// Classify an admitted case against parsed target verdicts.
pub fn classify(
    case: &GridCase,
    expected: &[bool],
    formula_id: usize,
    target: &BTreeMap<(usize, u64), bool>,
) -> CaseRow {
    let mut diff = Vec::new();
    let mut missing = false;
    let mut shown = String::new();
    for (i, want) in expected.iter().enumerate() {
        let at = i as u64;
        match target.get(&(formula_id, at)) {
            None => {
                missing = true;
                shown.push('?');
                diff.push(at);
            }
            Some(got) => {
                shown.push(tf(*got));
                if got != want {
                    diff.push(at);
                }
            }
        }
    }
    let reach = case.origin_reach();
    let status = if missing {
        CaseStatus::MissingTarget
    } else if diff.is_empty() {
        CaseStatus::Agree
    } else if diff.iter().all(|p| *p < reach) {
        CaseStatus::OriginProfileDifference
    } else {
        CaseStatus::Unexplained
    };
    CaseRow {
        id: case.id.clone(),
        formula: case.text.clone(),
        group: case.group.name(),
        status,
        refusal: None,
        expected: expected.iter().map(|b| tf(*b)).collect(),
        target: shown,
        differing_steps: diff,
    }
}

/// Where R2U2 lives and which revision is pinned.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pin {
    /// Source repository.
    pub repository: String,
    /// Exact commit.
    pub revision: String,
}

/// Read the R2U2 pin from `campaign/tool-versions.json`.
pub fn read_pin(repo: &Path) -> Result<Pin, String> {
    let text =
        fs::read_to_string(repo.join("campaign/tool-versions.json")).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    serde_json::from_value(v["r2u2"].clone()).map_err(|e| format!("r2u2 pin: {e}"))
}

/// Digests of one group's inputs and outputs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRecord {
    /// Group name.
    pub group: String,
    /// Admitted cases in the spec.
    pub cases: usize,
    /// SHA-256 of `spec.c2po`.
    pub spec_sha256: String,
    /// SHA-256 of `trace.csv`.
    pub trace_sha256: String,
    /// SHA-256 of `r2u2.stdout`, when run or replayed.
    pub stdout_sha256: Option<String>,
}

/// The differential report.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// [`REPORT_SCHEMA`].
    pub schema_version: String,
    /// `ran`, `replayed` or `tool_unavailable`.
    pub mode: String,
    /// Why R2U2 did not run.
    pub tool_detail: Option<String>,
    /// Pinned R2U2.
    pub pin: Pin,
    /// Observed R2U2 revision and tool versions.
    pub observed: BTreeMap<String, String>,
    /// Per-group digests.
    pub groups: Vec<GroupRecord>,
    /// Status counts.
    pub summary: BTreeMap<String, usize>,
    /// Refusal counts by typed name.
    pub refusals: BTreeMap<String, usize>,
    /// Every case.
    pub cases: Vec<CaseRow>,
    /// `pass` only when R2U2 ran/replayed and no case is unexplained or missing.
    pub outcome: String,
    /// What this report does not claim.
    pub non_claims: Vec<String>,
}

/// Admitted cases of the grid, with source verdicts and expressions, plus refused rows.
pub struct Plan {
    /// Every case.
    pub cases: Vec<GridCase>,
    /// Per case: `Ok((expected, expression))` or the refusal name.
    pub exported: Vec<Result<(Vec<bool>, String), String>>,
}

/// Evaluate and export the whole grid.
pub fn plan() -> Result<Plan, String> {
    let cases = generate_grid();
    let exported = cases
        .iter()
        .map(|c| match export(c) {
            Ok(expr) => expected_verdicts(c).map(|e| Ok((e, expr))),
            Err(r) => Ok(Err(r)),
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Plan { cases, exported })
}

fn group_admitted(plan: &Plan, group: Group) -> Vec<(&GridCase, String)> {
    plan.cases
        .iter()
        .zip(&plan.exported)
        .filter(|(c, _)| c.group == group)
        .filter_map(|(c, e)| e.as_ref().ok().map(|(_, x)| (c, x.clone())))
        .collect()
}

fn groups() -> Vec<Group> {
    let mut g: Vec<Group> = generate_grid().into_iter().map(|c| c.group).collect();
    g.sort();
    g.dedup();
    g
}

fn command_line(prog: &str, args: &[&str], dir: &Path) -> Result<(String, Vec<u8>), String> {
    let out = Command::new(prog)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("{prog}: {e}"))?;
    let mut log = out.stdout.clone();
    log.extend(&out.stderr);
    if out.status.success() {
        Ok((String::from_utf8_lossy(&out.stdout).into_owned(), log))
    } else {
        Err(format!(
            "{prog} {args:?} exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Check the R2U2 checkout; returns observed identities or why it is unusable.
pub fn check_tool(root: &Path, pin: &Pin) -> Result<BTreeMap<String, String>, String> {
    let c2po = root.join("compiler/c2po.py");
    let bin = root.join("monitors/c/build/r2u2");
    if !c2po.is_file() || !bin.is_file() {
        return Err(format!(
            "R2U2 not built at {}: need compiler/c2po.py and monitors/c/build/r2u2",
            root.display()
        ));
    }
    let (rev, _) = command_line("git", &["rev-parse", "HEAD"], root)?;
    let (status, _) = command_line("git", &["status", "--porcelain"], root)?;
    if rev.trim() != pin.revision {
        return Err(format!(
            "R2U2 checkout is {} but the pin is {}",
            rev.trim(),
            pin.revision
        ));
    }
    if !status.trim().is_empty() {
        return Err("R2U2 checkout has uncommitted changes".to_owned());
    }
    let (py, _) = command_line("python3", &["--version"], root)?;
    Ok(BTreeMap::from([
        ("r2u2Revision".to_owned(), rev.trim().to_owned()),
        ("python".to_owned(), py.trim().to_owned()),
        (
            "r2u2Binary".to_owned(),
            sha256_hex(&fs::read(&bin).map_err(|e| e.to_string())?),
        ),
        (
            "c2po".to_owned(),
            sha256_hex(&fs::read(&c2po).map_err(|e| e.to_string())?),
        ),
    ]))
}

/// Where target stdout comes from.
pub enum Source<'a> {
    /// Run R2U2 from this checkout.
    Live(&'a Path),
    /// Read `outputs/<group>/r2u2.stdout` from a recorded run directory.
    Recorded,
    /// No target available.
    None(String),
}

/// Run (or replay) the differential and write inputs/outputs under `out`.
pub fn run(repo: &Path, source: &Source<'_>, out: &Path) -> Result<Report, String> {
    let pin = read_pin(repo)?;
    let plan = plan()?;
    let mut observed = BTreeMap::new();
    let (mut mode, mut detail) = ("replayed", None);
    match source {
        Source::Live(root) => match check_tool(root, &pin) {
            Ok(o) => {
                observed = o;
                mode = "ran";
            }
            Err(e) => (mode, detail) = ("tool_unavailable", Some(e)),
        },
        Source::None(why) => (mode, detail) = ("tool_unavailable", Some(why.clone())),
        Source::Recorded => {}
    }
    let mut rows: BTreeMap<String, CaseRow> = BTreeMap::new();
    let mut records = Vec::new();
    for group in groups() {
        let admitted = group_admitted(&plan, group);
        let inputs = group_inputs(group, &admitted);
        let name = group.name();
        let in_dir = out.join("inputs").join(&name);
        let out_dir = out.join("outputs").join(&name);
        fs::create_dir_all(&in_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        fs::write(in_dir.join("spec.c2po"), &inputs.spec).map_err(|e| e.to_string())?;
        fs::write(in_dir.join("trace.csv"), &inputs.trace).map_err(|e| e.to_string())?;
        let stdout = match source {
            _ if mode == "tool_unavailable" || admitted.is_empty() => None,
            Source::Live(root) => Some(run_group(root, &in_dir, &out_dir)?),
            _ => fs::read_to_string(out_dir.join("r2u2.stdout")).ok(),
        };
        let target = stdout.as_deref().map(parse_r2u2_stdout);
        records.push(GroupRecord {
            group: name.clone(),
            cases: admitted.len(),
            spec_sha256: sha256_hex(inputs.spec.as_bytes()),
            trace_sha256: sha256_hex(inputs.trace.as_bytes()),
            stdout_sha256: stdout.as_deref().map(|s| sha256_hex(s.as_bytes())),
        });
        for (idx, (c, _)) in admitted.iter().enumerate() {
            let pos = plan
                .cases
                .iter()
                .position(|x| x.id == c.id)
                .ok_or("case lost")?;
            let expected = &plan.exported[pos].as_ref().map_err(String::clone)?.0;
            let row = match &target {
                Some(t) => classify(c, expected, idx, t),
                None => CaseRow {
                    status: CaseStatus::NotRun,
                    target: String::new(),
                    ..classify(c, expected, idx, &BTreeMap::new())
                },
            };
            rows.insert(c.id.clone(), row);
        }
    }
    let cases: Vec<CaseRow> = plan
        .cases
        .iter()
        .zip(&plan.exported)
        .map(|(c, e)| match e {
            Ok(_) => rows.remove(&c.id).ok_or_else(|| "missing row".to_owned()),
            Err(r) => Ok(CaseRow {
                id: c.id.clone(),
                formula: c.text.clone(),
                group: c.group.name(),
                status: CaseStatus::Unsupported,
                refusal: Some(r.clone()),
                expected: String::new(),
                target: String::new(),
                differing_steps: Vec::new(),
            }),
        })
        .collect::<Result<_, _>>()?;
    Ok(finish(pin, mode, detail, observed, records, cases))
}

fn run_group(root: &Path, in_dir: &Path, out_dir: &Path) -> Result<String, String> {
    let abs = |p: &Path| fs::canonicalize(p).map_err(|e| e.to_string());
    let (spec, trace, bin) = (
        abs(&in_dir.join("spec.c2po"))?,
        abs(&in_dir.join("trace.csv"))?,
        abs(out_dir)?.join("spec.bin"),
    );
    let s = |p: &PathBuf| p.to_string_lossy().into_owned();
    let (_, log) = command_line(
        "python3",
        &[
            "compiler/c2po.py",
            "--spec",
            &s(&spec),
            "--trace",
            &s(&trace),
            "--output",
            &s(&bin),
        ],
        root,
    )?;
    fs::write(out_dir.join("c2po.log"), log).map_err(|e| e.to_string())?;
    let (stdout, _) = command_line("monitors/c/build/r2u2", &[&s(&bin), &s(&trace)], root)?;
    fs::write(out_dir.join("r2u2.stdout"), &stdout).map_err(|e| e.to_string())?;
    Ok(stdout)
}

fn finish(
    pin: Pin,
    mode: &str,
    tool_detail: Option<String>,
    observed: BTreeMap<String, String>,
    groups: Vec<GroupRecord>,
    cases: Vec<CaseRow>,
) -> Report {
    let mut summary = BTreeMap::new();
    let mut refusals = BTreeMap::new();
    for c in &cases {
        let key = serde_json::to_value(c.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        *summary.entry(key).or_insert(0) += 1;
        if let Some(r) = &c.refusal {
            *refusals.entry(r.clone()).or_insert(0) += 1;
        }
    }
    let bad = cases.iter().any(|c| {
        matches!(
            c.status,
            CaseStatus::Unexplained | CaseStatus::MissingTarget | CaseStatus::NotRun
        )
    });
    let outcome = if mode == "tool_unavailable" {
        "not_run"
    } else if bad {
        "fail"
    } else {
        "pass"
    };
    Report {
        schema_version: REPORT_SCHEMA.to_owned(),
        mode: mode.to_owned(),
        tool_detail,
        pin,
        observed,
        groups,
        summary,
        refusals,
        cases,
        outcome: outcome.to_owned(),
        non_claims: vec![
            "Unsupported cases were refused at export and say nothing about R2U2's behaviour."
                .to_owned(),
            "An origin-profile difference is documented, not parity.".to_owned(),
            "The grid is bounded: depth 3, fixed intervals, 8-step traces over p and q.".to_owned(),
        ],
    }
}
