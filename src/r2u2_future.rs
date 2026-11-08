//! Live R2U2 differential over an exhaustive small future-time corpus (TL-237).
//!
//! Every depth-one future formula (`F`, `G`, `U`, `R`) with every interval
//! inside `0..=2` is run, in one R2U2 invocation per trace, over every
//! three-step trace of `p0` and `p1`. R2U2 reports a verdict for `formula:time`;
//! the reference is this crate's online-prefix evaluation of the formula at that
//! time. The comparison uses the same status vocabulary as the past grid
//! ([`CaseStatus`]).
//!
//! A reference that is still pending where R2U2 has resolved is a documented
//! profile difference (R2U2 treats the end of the supplied trace as the end of
//! the signal); a conclusive reference that R2U2 contradicts or leaves silent
//! fails the run.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

use crate::campaign::sha256_hex;
use crate::r2u2_live::{
    check_tool, finish, parse_r2u2_stdout, read_pin, read_recorded, run_group,
    verify_recorded_group, CaseRow, CaseStatus, GroupRecord, Report, Source,
};
use crate::{
    evaluate_prefix_at, map_to_c2po, EvaluationLimits, MappingSourceIdentity, MappingSourceState,
    TruthValue,
};

/// Steps in every future-suite trace.
pub const FUTURE_TRACE_LEN: usize = 3;

/// One generated future-time formula.
#[derive(Clone, Debug)]
pub struct FutureCase {
    /// Stable id.
    pub id: String,
    /// Rendered formula text.
    pub text: String,
    nodes: Vec<Node>,
}

const INTERVALS: [(u32, u32); 6] = [(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2)];

fn prop(n: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(n),
    })
}

/// Every depth-one future formula in a fixed order.
pub fn future_corpus() -> Vec<FutureCase> {
    let mut out = Vec::new();
    for (a, b) in INTERVALS {
        let i = Interval::new(a, b).expect("ordered");
        let unary = [
            (
                "F",
                NodeKind::Future {
                    interval: i,
                    operand: NodeId(0),
                },
            ),
            (
                "G",
                NodeKind::Globally {
                    interval: i,
                    operand: NodeId(0),
                },
            ),
        ];
        for (sym, kind) in unary {
            out.push(FutureCase {
                id: format!("{}[{a},{b}]", sym.to_lowercase()),
                text: format!("{sym}[{a},{b}] p0"),
                nodes: vec![prop(0), Node::new(kind)],
            });
        }
        let binary = [
            (
                "U",
                NodeKind::Until {
                    interval: i,
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ),
            (
                "R",
                NodeKind::Release {
                    interval: i,
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ),
        ];
        for (sym, kind) in binary {
            out.push(FutureCase {
                id: format!("{}[{a},{b}]", sym.to_lowercase()),
                text: format!("p0 {sym}[{a},{b}] p1"),
                nodes: vec![prop(0), prop(1), Node::new(kind)],
            });
        }
    }
    out
}

/// Every trace of [`FUTURE_TRACE_LEN`] rows over `(p0, p1)`, as bit words.
pub fn future_traces() -> Vec<Vec<[bool; 2]>> {
    (0..4u32.pow(FUTURE_TRACE_LEN as u32))
        .map(|w| {
            (0..FUTURE_TRACE_LEN)
                .map(|t| {
                    let row = (w >> (2 * t)) & 3;
                    [row & 1 == 1, row & 2 == 2]
                })
                .collect()
        })
        .collect()
}

fn trace_name(trace: &[[bool; 2]]) -> String {
    trace
        .iter()
        .map(|r| format!("{}{}", u8::from(r[0]), u8::from(r[1])))
        .collect::<Vec<_>>()
        .join("-")
}

fn graph(case: &FutureCase) -> Result<Formula<'_>, String> {
    let root = NodeId(u32::try_from(case.nodes.len() - 1).map_err(|e| e.to_string())?);
    Formula::new(SemanticProfile::OnlinePrefixV1, root, &case.nodes).map_err(|e| format!("{e:?}"))
}

/// C2PO expression for `case`, or the typed refusal name.
pub fn future_expression(case: &FutureCase) -> Result<String, String> {
    map_to_c2po(
        graph(case)?,
        case.id.clone(),
        case.text.as_bytes(),
        MappingSourceIdentity {
            revision: "grid".to_owned(),
            state: MappingSourceState::Clean,
        },
        None,
        10_000,
    )
    .map(|m| m.expression)
    .map_err(|e| {
        format!("{e:?}")
            .split(['(', ' ', '{'])
            .next()
            .unwrap_or("Unknown")
            .to_owned()
    })
}

/// Reference verdict of `case` at `time` on `trace`: `None` while pending.
pub fn reference_at(
    case: &FutureCase,
    trace: &[[bool; 2]],
    time: u64,
) -> Result<Option<bool>, String> {
    let props: Vec<Vec<PropositionId>> = trace
        .iter()
        .map(|r| {
            [(r[0], 0), (r[1], 1)]
                .into_iter()
                .filter_map(|(v, id)| v.then_some(PropositionId(id)))
                .collect()
        })
        .collect();
    let report = evaluate_prefix_at(
        graph(case)?,
        case.id.clone(),
        &props,
        "future-trace",
        false,
        time,
        EvaluationLimits::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    Ok(match report.verdict {
        TruthValue::True => Some(true),
        TruthValue::False => Some(false),
        TruthValue::Pending => None,
    })
}

/// Classify one reference/target pair.
pub fn classify_step(reference: Option<bool>, target: Option<bool>) -> CaseStatus {
    match (reference, target) {
        (Some(r), Some(t)) if r == t => CaseStatus::Agree,
        (Some(_), Some(_)) => CaseStatus::Unexplained,
        (Some(_), None) => CaseStatus::MissingTarget,
        (None, None) => CaseStatus::Agree,
        (None, Some(_)) => CaseStatus::PendingResolvedByTarget,
    }
}

fn worst(a: CaseStatus, b: CaseStatus) -> CaseStatus {
    let rank = |s: CaseStatus| match s {
        CaseStatus::Unexplained => 4,
        CaseStatus::MissingTarget => 3,
        CaseStatus::PendingResolvedByTarget => 2,
        _ => 1,
    };
    if rank(b) > rank(a) {
        b
    } else {
        a
    }
}

fn sym(v: Option<bool>) -> char {
    match v {
        Some(true) => 'T',
        Some(false) => 'F',
        None => '?',
    }
}

/// Run (or replay) the future suite under `out`.
pub fn run_future(repo: &Path, source: &Source<'_>, out: &Path) -> Result<Report, String> {
    let pin = read_pin(repo)?;
    let corpus = future_corpus();
    let exprs: Vec<String> = corpus
        .iter()
        .map(future_expression)
        .collect::<Result<_, _>>()?;
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
    let mut spec = String::from("INPUT\n  p0,p1: bool;\nFTSPEC\n");
    exprs
        .iter()
        .for_each(|e| spec.push_str(&format!("  {e};\n")));
    let recorded = match source {
        Source::Recorded => Some(read_recorded(out)?),
        _ => None,
    };
    let mut records = Vec::new();
    let mut rows = Vec::new();
    for trace in future_traces() {
        let name = trace_name(&trace);
        let mut csv = String::from("# p0,p1\n");
        trace
            .iter()
            .for_each(|r| csv.push_str(&format!("{},{}\n", u8::from(r[0]), u8::from(r[1]))));
        let (in_dir, out_dir) = (
            out.join("inputs").join(&name),
            out.join("outputs").join(&name),
        );
        if let Some(rec) = &recorded {
            verify_recorded_group(rec, &name, &spec, &csv, &out_dir)?;
        }
        for d in [&in_dir, &out_dir] {
            fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        fs::write(in_dir.join("spec.c2po"), &spec).map_err(|e| e.to_string())?;
        fs::write(in_dir.join("trace.csv"), &csv).map_err(|e| e.to_string())?;
        let stdout = match source {
            _ if mode == "tool_unavailable" => None,
            Source::Live(root) => Some(run_group(root, &in_dir, &out_dir)?),
            _ => fs::read_to_string(out_dir.join("r2u2.stdout")).ok(),
        };
        let target = stdout.as_deref().map(parse_r2u2_stdout);
        records.push(GroupRecord {
            group: name.clone(),
            cases: corpus.len(),
            spec_sha256: sha256_hex(spec.as_bytes()),
            trace_sha256: sha256_hex(csv.as_bytes()),
            stdout_sha256: stdout.as_deref().map(|s| sha256_hex(s.as_bytes())),
        });
        for (idx, case) in corpus.iter().enumerate() {
            let mut status = CaseStatus::Agree;
            let (mut want, mut got, mut diff) = (String::new(), String::new(), Vec::new());
            for t in 0..FUTURE_TRACE_LEN as u64 {
                let r = reference_at(case, &trace, t)?;
                want.push(sym(r));
                if let Some(tv) = &target {
                    let g = tv.get(&(idx, t)).copied();
                    got.push(sym(g));
                    let s = classify_step(r, g);
                    if s != CaseStatus::Agree {
                        diff.push(t);
                    }
                    status = worst(status, s);
                }
            }
            rows.push(CaseRow {
                id: format!("{}/{name}", case.id),
                formula: case.text.clone(),
                group: name.clone(),
                status: if target.is_some() {
                    status
                } else {
                    CaseStatus::NotRun
                },
                refusal: None,
                expected: want,
                target: got,
                differing_steps: diff,
            });
        }
    }
    let mut report = finish(pin, mode, detail, observed, records, rows);
    report.suite = "future".to_owned();
    Ok(report)
}
