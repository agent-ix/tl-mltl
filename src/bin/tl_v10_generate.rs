//! Deterministic V10 C2PO input producer. It does not execute a foreign tool.
//! Trace: FR-052-AC-1, FR-055-AC-1, TC-191, TC-197.

use std::{fs, path::{Path, PathBuf}};

use serde::Serialize;
use sha2::{Digest, Sha256};

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
#[serde(rename_all = "camelCase")]
struct GeneratedCase {
    id: String,
    formulas: usize,
    trace_positions: usize,
    spec_digest: String,
    trace_digest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema: &'static str,
    formula_trace_cases: usize,
    per_step_cells: usize,
    cases: Vec<GeneratedCase>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn expression(operator: &str, interval: Option<(u32, u32)>, depth: usize) -> String {
    let mut expression = String::from("p");
    for _ in 0..depth {
        expression = match operator {
            "once" => format!(
                "O[{},{}]({expression})",
                interval.unwrap().0,
                interval.unwrap().1
            ),
            "historically" => format!(
                "H[{},{}]({expression})",
                interval.unwrap().0,
                interval.unwrap().1
            ),
            "since" => format!(
                "({expression} S[{},{}] q)",
                interval.unwrap().0,
                interval.unwrap().1
            ),
            "triggered" => format!(
                "(!((!{expression}) S[{},{}] (!q)))",
                interval.unwrap().0,
                interval.unwrap().1
            ),
            "previous" => format!("O[1,1]({expression})"),
            _ => unreachable!("fixed operator inventory"),
        };
    }
    expression
}

fn trace(kind: &str, boundary: usize) -> String {
    let mut text = String::from("# p,q\n");
    for position in 0..STEPS {
        let (p, q) = match kind {
            "all-true" => (true, true),
            "all-false" => (false, false),
            "boundary-toggle" => {
                let p = position >= boundary;
                (p, !p)
            }
            _ => unreachable!("fixed trace inventory"),
        };
        text.push_str(&format!("{},{}\n", u8::from(p), u8::from(q)));
    }
    text
}

fn emit(
    out: &Path,
    id: &str,
    expressions: &[String],
    trace: &str,
) -> Result<GeneratedCase, String> {
    let spec = format!(
        "INPUT\n p,q: bool;\nPTSPEC\n{}\n",
        expressions
            .iter()
            .map(|item| format!(" {item};"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(out.join(format!("{id}.c2po")), &spec).map_err(|error| error.to_string())?;
    fs::write(out.join(format!("{id}.csv")), trace).map_err(|error| error.to_string())?;
    Ok(GeneratedCase {
        id: id.to_owned(),
        formulas: expressions.len(),
        trace_positions: trace.lines().count() - 1,
        spec_digest: digest(spec.as_bytes()),
        trace_digest: digest(trace.as_bytes()),
    })
}

fn run() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 5 || args[1] != "--output-dir" || args[3] != "--manifest" {
        return Err("usage: tl_v10_generate --output-dir DIR --manifest FILE".into());
    }
    let out = PathBuf::from(&args[2]);
    fs::create_dir_all(&out).map_err(|error| error.to_string())?;
    if fs::read_dir(&out)
        .map_err(|error| error.to_string())?
        .next()
        .is_some()
    {
        return Err("V10 output directory is not empty".into());
    }
    let mut cases = Vec::new();
    for (group, interval, boundary) in INTERVALS
        .map(|(name, a, b)| (name, Some((a, b)), b as usize))
        .into_iter()
        .chain(std::iter::once(("previous", None, 1)))
    {
        let operators: &[&str] = if interval.is_some() {
            &["once", "historically", "since", "triggered"]
        } else {
            &["previous"]
        };
        let expressions = operators
            .iter()
            .flat_map(|operator| (1..=3).map(move |depth| expression(operator, interval, depth)))
            .collect::<Vec<_>>();
        for trace_kind in ["all-true", "all-false", "boundary-toggle"] {
            let id = format!("{group}-{trace_kind}");
            cases.push(emit(&out, &id, &expressions, &trace(trace_kind, boundary))?);
        }
    }
    let safety_spec = "INPUT\n q: bool;\nFTSPEC\n q;\n";
    let safety_trace = "# q\n0\n1\n";
    fs::write(out.join("safety.c2po"), safety_spec).map_err(|error| error.to_string())?;
    fs::write(out.join("safety.csv"), safety_trace).map_err(|error| error.to_string())?;
    cases.push(GeneratedCase {
        id: "safety".into(),
        formulas: 1,
        trace_positions: 2,
        spec_digest: digest(safety_spec.as_bytes()),
        trace_digest: digest(safety_trace.as_bytes()),
    });
    let grid_cases: usize = cases[..21].iter().map(|case| case.formulas).sum();
    let manifest = Manifest {
        schema: "tl-mltl.v10-input-manifest/v1",
        formula_trace_cases: grid_cases,
        per_step_cells: grid_cases * STEPS,
        cases,
    };
    if manifest.formula_trace_cases != 225 || manifest.per_step_cells != 1350 {
        return Err("V10 grid axis count changed".into());
    }
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(&args[4], bytes).map_err(|error| error.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trace: FR-052-AC-1, TC-191
    #[test]
    fn grid_is_exactly_1350_cells() {
        let directory = tempfile::tempdir().unwrap();
        let out = directory.path().join("inputs");
        fs::create_dir(&out).unwrap();
        let mut cases = 0;
        for (group, interval, _) in INTERVALS
            .map(|(name, a, b)| (name, Some((a, b)), b as usize))
            .into_iter()
            .chain(std::iter::once(("previous", None, 1)))
        {
            let operators: &[&str] = if interval.is_some() {
                &["once", "historically", "since", "triggered"]
            } else {
                &["previous"]
            };
            let expressions = operators
                .iter()
                .flat_map(|operator| {
                    (1..=3).map(move |depth| expression(operator, interval, depth))
                })
                .collect::<Vec<_>>();
            let case = emit(&out, group, &expressions, &trace("boundary-toggle", 2)).unwrap();
            cases += case.formulas * case.trace_positions * 3;
        }
        assert_eq!(cases, 1350);
    }
}
