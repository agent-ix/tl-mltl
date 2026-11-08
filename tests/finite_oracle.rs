//! TL-35 / TL-224: production finite evaluators against the independent
//! `tl-oracle`, over declared small-scope populations.
//!
//! The expected answer for every case comes from `tl-oracle`, which shares only
//! `tl-syntax` input types with production. Each sweep records every case once
//! in an exact-count `PopulationLedger`; seeded production faults must turn it
//! red, and a disagreement is minimised into a portable regression fixture.
//!
//! Two modes share one engine:
//! * the fast subset runs in every `cargo test` (the local gate);
//! * the full sweep is `#[ignore]`d and runs under `make sweep-full`
//!   (nightly), in release mode.
//!
//! Every population is deterministic. Exhaustive populations are enumerated
//! in a fixed order; the sampled depth-three population draws from a
//! SplitMix64 stream whose seed is a constant recorded in the evidence file.
//! Evidence (tool versions, seed, formula/word/case counts, admitted/refused
//! split, runtime, disagreements) is written as JSON under
//! `$TL_MLTL_EVIDENCE_DIR`, defaulting to `target/tmp/small-scope-evidence`.

use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use serde_json::{json, Value};
use tl_oracle::population::{CaseResult, PopulationError, PopulationLedger, PopulationSummary};
use tl_oracle::Formula as O;

#[path = "support/finite_harness.rs"]
mod harness;
use harness::{
    drop_negation, formula_from_json, formula_to_json, oi, oracle_atom, production, reference,
    shrink_end, swap_dual, word_from_json, word_to_json, Cell, Profile, Word,
};

// ---------------------------------------------------------------------------
// Declared populations
// ---------------------------------------------------------------------------

/// SplitMix64: a tiny, fixed, dependency-free deterministic stream.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[derive(Clone, Copy)]
enum Formulas {
    /// Every formula of exact depth 0..=`depth`.
    Exhaustive { depth: usize },
    /// `count` seeded formulas of exact depth `depth`.
    Sampled { depth: usize, count: usize },
}

#[derive(Clone, Copy)]
enum Traces {
    /// Every word of length 1..=`max_len` over `props` propositions, every position.
    Exhaustive { props: u32, max_len: usize },
    /// `per_formula` seeded (word, position) pairs per formula.
    Sampled { max_len: usize, per_formula: usize },
}

#[derive(Clone, Copy)]
struct Scope {
    name: &'static str,
    seed: u64,
    /// Number of atomic propositions among the leaves (0..2); `False`/`True`
    /// are leaves when `constants` is set.
    atoms: u32,
    constants: bool,
    /// Intervals are every `0 <= a <= b <= max_end`.
    max_end: usize,
    formulas: Formulas,
    traces: Traces,
}

impl Scope {
    fn leaves(&self) -> Vec<O> {
        let mut leaves = Vec::new();
        if self.constants {
            leaves.extend([O::False, O::True]);
        }
        leaves.extend((0..self.atoms).map(oracle_atom));
        leaves
    }

    fn intervals(&self) -> Vec<(usize, usize)> {
        (0..=self.max_end)
            .flat_map(|end| (0..=end).map(move |start| (start, end)))
            .collect()
    }

    /// Every operator applied to operands drawn from `operands`.
    fn unary_ops(&self, p: &O) -> Vec<O> {
        let mut out = vec![
            O::Not(Box::new(p.clone())),
            O::StrongPrevious(Box::new(p.clone())),
        ];
        for r in self.intervals() {
            let r = oi(r);
            out.extend([
                O::Future(r, Box::new(p.clone())),
                O::Globally(r, Box::new(p.clone())),
                O::Once(r, Box::new(p.clone())),
                O::Historically(r, Box::new(p.clone())),
            ]);
        }
        out
    }

    fn binary_ops(&self, p: &O, q: &O) -> Vec<O> {
        let (a, b) = (Box::new(p.clone()), Box::new(q.clone()));
        let mut out = vec![
            O::And(a.clone(), b.clone()),
            O::Or(a.clone(), b.clone()),
            O::Implies(a.clone(), b.clone()),
            O::Equivalent(a.clone(), b.clone()),
        ];
        for r in self.intervals() {
            let r = oi(r);
            out.extend([
                O::Until(r, a.clone(), b.clone()),
                O::Release(r, a.clone(), b.clone()),
                O::Since(r, a.clone(), b.clone()),
                O::Triggered(r, a.clone(), b.clone()),
            ]);
        }
        out
    }

    fn exhaustive_formulas(&self, depth: usize) -> Vec<O> {
        let mut exact: Vec<Vec<O>> = vec![self.leaves()];
        for level in 1..=depth {
            let all: Vec<O> = exact.iter().flatten().cloned().collect();
            let frontier_start = all.len() - exact[level - 1].len();
            let mut next = Vec::new();
            for p in &all[frontier_start..] {
                next.extend(self.unary_ops(p));
            }
            for (i, p) in all.iter().enumerate() {
                for (j, q) in all.iter().enumerate() {
                    // At least one operand sits on the frontier, so each tree
                    // appears exactly once at its own depth.
                    if i >= frontier_start || j >= frontier_start {
                        next.extend(self.binary_ops(p, q));
                    }
                }
            }
            exact.push(next);
        }
        exact.into_iter().flatten().collect()
    }

    fn sampled_formula(&self, rng: &mut Rng, depth: usize, leaves: &[O]) -> O {
        if depth == 0 {
            return leaves[rng.below(leaves.len())].clone();
        }
        let ivs = self.intervals();
        let iv = oi(ivs[rng.below(ivs.len())]);
        // One child carries the full remaining depth so the root has exact depth.
        let deep = self.sampled_formula(rng, depth - 1, leaves);
        let other_depth = rng.below(depth);
        let shallow = self.sampled_formula(rng, other_depth, leaves);
        let (a, b) = if rng.below(2) == 0 {
            (Box::new(deep), Box::new(shallow))
        } else {
            (Box::new(shallow), Box::new(deep))
        };
        match rng.below(14) {
            0 => O::Not(a),
            1 => O::StrongPrevious(a),
            2 => O::Future(iv, a),
            3 => O::Globally(iv, a),
            4 => O::Once(iv, a),
            5 => O::Historically(iv, a),
            6 => O::And(a, b),
            7 => O::Or(a, b),
            8 => O::Implies(a, b),
            9 => O::Equivalent(a, b),
            10 => O::Until(iv, a, b),
            11 => O::Release(iv, a, b),
            12 => O::Since(iv, a, b),
            _ => O::Triggered(iv, a, b),
        }
    }

    fn formula_list(&self) -> Vec<O> {
        match self.formulas {
            Formulas::Exhaustive { depth } => self.exhaustive_formulas(depth),
            Formulas::Sampled { depth, count } => {
                let leaves = self.leaves();
                let mut rng = Rng(self.seed);
                (0..count)
                    .map(|_| self.sampled_formula(&mut rng, depth, &leaves))
                    .collect()
            }
        }
    }

    fn exhaustive_words(props: u32, max_len: usize) -> Vec<Word> {
        let cells: Vec<Cell> = (0..1u32 << props)
            .map(|bits| [bits & 1 != 0, bits & 2 != 0])
            .collect();
        let mut words: Vec<Word> = Vec::new();
        let mut frontier: Vec<Word> = vec![Vec::new()];
        for _ in 0..max_len {
            let mut next = Vec::new();
            for prefix in &frontier {
                for cell in &cells {
                    let mut word = prefix.clone();
                    word.push(*cell);
                    next.push(word);
                }
            }
            words.extend(next.iter().cloned());
            frontier = next;
        }
        words
    }

    /// The (word, position) pairs checked for formula number `index`.
    fn cases_for(&self, index: usize, exhaustive: &[Word]) -> Vec<(Word, usize)> {
        match self.traces {
            Traces::Exhaustive { .. } => exhaustive
                .iter()
                .flat_map(|w| (0..w.len()).map(move |at| (w.clone(), at)))
                .collect(),
            Traces::Sampled {
                max_len,
                per_formula,
            } => {
                let mut rng = Rng(self.seed ^ (index as u64).wrapping_mul(0xA24B_AED4_963E_E407));
                (0..per_formula)
                    .map(|_| {
                        let len = 1 + rng.below(max_len);
                        let word: Word = (0..len)
                            .map(|_| {
                                let bits = rng.next();
                                [bits & 1 != 0, bits & 2 != 0]
                            })
                            .collect();
                        let at = rng.below(len);
                        (word, at)
                    })
                    .collect()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The sweep
// ---------------------------------------------------------------------------

struct Disagreement {
    formula: O,
    word: Word,
    at: usize,
}

struct Sweep {
    summary: Result<PopulationSummary, PopulationError>,
    formulas: usize,
    declared: u64,
    elapsed: Duration,
    disagreement: Option<Disagreement>,
}

/// Runs the whole declared population; `candidate` may corrupt the formula the
/// production side receives.
fn sweep(profile: Profile, scope: &Scope, candidate: &dyn Fn(&O) -> O) -> Sweep {
    let started = Instant::now();
    let formulas = scope.formula_list();
    let words = match scope.traces {
        Traces::Exhaustive { props, max_len } => Scope::exhaustive_words(props, max_len),
        Traces::Sampled { .. } => Vec::new(),
    };
    let declared: u64 = match scope.traces {
        Traces::Exhaustive { .. } => {
            formulas.len() as u64 * words.iter().map(Vec::len).sum::<usize>() as u64
        }
        Traces::Sampled { per_formula, .. } => (formulas.len() * per_formula) as u64,
    };
    let mut ledger = PopulationLedger::new(declared).unwrap();
    let mut id = 0;
    let mut disagreement = None;
    'outer: for (index, f) in formulas.iter().enumerate() {
        let mutated = candidate(f);
        for (word, at) in scope.cases_for(index, &words) {
            let recorded = ledger.record(
                id,
                reference(profile, f, &word, at),
                production(profile, &mutated, &word, at),
            );
            if recorded.is_err() {
                disagreement = Some(Disagreement {
                    formula: f.clone(),
                    word,
                    at,
                });
                // The ledger keeps the first failure; later cases add nothing.
                break 'outer;
            }
            id += 1;
        }
    }
    Sweep {
        summary: ledger.finish(),
        formulas: formulas.len(),
        declared,
        elapsed: started.elapsed(),
        disagreement,
    }
}

// ---------------------------------------------------------------------------
// Minimisation and regression fixtures
// ---------------------------------------------------------------------------

fn disagrees(profile: Profile, candidate: &dyn Fn(&O) -> O, d: &Disagreement) -> bool {
    if d.word.is_empty() || d.at >= d.word.len() {
        return false;
    }
    reference(profile, &d.formula, &d.word, d.at)
        != production(profile, &candidate(&d.formula), &d.word, d.at)
}

fn children(f: &O) -> Vec<O> {
    use O::*;
    match f {
        Not(a)
        | StrongPrevious(a)
        | Future(_, a)
        | Globally(_, a)
        | Once(_, a)
        | Historically(_, a) => vec![(**a).clone()],
        And(a, b)
        | Or(a, b)
        | Implies(a, b)
        | Equivalent(a, b)
        | Until(_, a, b)
        | Release(_, a, b)
        | Since(_, a, b)
        | Triggered(_, a, b) => {
            vec![(**a).clone(), (**b).clone()]
        }
        False | True | Atom(_) => Vec::new(),
    }
}

fn node_count(f: &O) -> usize {
    1 + children(f).iter().map(node_count).sum::<usize>()
}

/// Greedy delta-debugging: replace the formula by a disagreeing child, then
/// drop trailing/leading instants and clear true propositions, until fixpoint.
fn minimise(profile: Profile, candidate: &dyn Fn(&O) -> O, mut d: Disagreement) -> Disagreement {
    loop {
        let mut next = None;
        for child in children(&d.formula) {
            let t = Disagreement {
                formula: child,
                word: d.word.clone(),
                at: d.at,
            };
            if disagrees(profile, candidate, &t) {
                next = Some(t);
                break;
            }
        }
        if next.is_none() && d.word.len() > d.at + 1 {
            let mut word = d.word.clone();
            word.pop();
            next = Some(Disagreement {
                formula: d.formula.clone(),
                word,
                at: d.at,
            })
            .filter(|t| disagrees(profile, candidate, t));
        }
        if next.is_none() && d.at > 0 {
            let word = d.word[1..].to_vec();
            next = Some(Disagreement {
                formula: d.formula.clone(),
                word,
                at: d.at - 1,
            })
            .filter(|t| disagrees(profile, candidate, t));
        }
        if next.is_none() {
            'bits: for i in 0..d.word.len() {
                for p in 0..2 {
                    if d.word[i][p] {
                        let mut word = d.word.clone();
                        word[i][p] = false;
                        let t = Disagreement {
                            formula: d.formula.clone(),
                            word,
                            at: d.at,
                        };
                        if disagrees(profile, candidate, &t) {
                            next = Some(t);
                            break 'bits;
                        }
                    }
                }
            }
        }
        match next {
            Some(t) => d = t,
            None => return d,
        }
    }
}

/// The committed regression-fixture shape; `expected` is the oracle's answer.
fn fixture(profile: Profile, id: &str, d: &Disagreement) -> Value {
    let expected = match reference(profile, &d.formula, &d.word, d.at) {
        CaseResult::Admitted(v) => json!(v),
        CaseResult::Refused => json!("refused"),
    };
    json!({
        "id": id,
        "profile": profile.token(),
        "formula": formula_to_json(&d.formula),
        "trace": word_to_json(&d.word),
        "at": d.at,
        "expected": expected,
    })
}

fn profile_from_token(token: &str) -> Profile {
    [Profile::ClosedFuture, Profile::OriginPast]
        .into_iter()
        .find(|p| p.token() == token)
        .unwrap_or_else(|| panic!("unknown profile `{token}`"))
}

/// Replays one fixture: the oracle must still say what was recorded and the
/// production evaluator must agree with it.
fn replay(f: &Value) -> Result<(), String> {
    let profile = profile_from_token(f["profile"].as_str().ok_or("profile")?);
    let (formula, word) = (
        formula_from_json(&f["formula"]),
        word_from_json(&f["trace"]),
    );
    let at = f["at"].as_u64().ok_or("at")? as usize;
    let want = match &f["expected"] {
        Value::Bool(b) => CaseResult::Admitted(*b),
        Value::String(s) if s == "refused" => CaseResult::Refused,
        other => return Err(format!("bad expected {other}")),
    };
    let (oracle, prod) = (
        reference(profile, &formula, &word, at),
        production(profile, &formula, &word, at),
    );
    if oracle != want {
        return Err(format!("oracle {oracle:?} != recorded {want:?}"));
    }
    if prod != want {
        return Err(format!("production {prod:?} != expected {want:?}"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

fn tool_version(program: &str) -> String {
    Command::new(program)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unavailable".to_owned(), |s| s.trim().to_owned())
}

fn locked_package(name: &str) -> String {
    let lock =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")).unwrap_or_default();
    lock.split("[[package]]")
        .find(|block| block.contains(&format!("name = \"{name}\"")))
        .map(|block| {
            let field = |key: &str| {
                block
                    .lines()
                    .find_map(|l| l.strip_prefix(&format!("{key} = ")))
                    .map(|v| v.trim_matches('"').to_owned())
                    .unwrap_or_default()
            };
            format!("{} {}", field("version"), field("source"))
        })
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn evidence_dir() -> PathBuf {
    std::env::var_os("TL_MLTL_EVIDENCE_DIR").map_or_else(
        || PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("small-scope-evidence"),
        PathBuf::from,
    )
}

/// Writes one evidence record per (scope, profile) sweep and returns it.
fn write_evidence(
    profile: Profile,
    scope: &Scope,
    run: &Sweep,
    regression: Option<Value>,
) -> Value {
    let (declared, admitted, refused, result) = match &run.summary {
        Ok(s) => (s.declared, s.admitted, s.refused, "agree".to_owned()),
        Err(e) => (run.declared, 0, 0, format!("{e:?}")),
    };
    let record = json!({
        "scope": scope.name,
        "profile": profile.token(),
        "seed": scope.seed,
        "formulaCount": run.formulas,
        "declaredCases": declared,
        "admitted": admitted,
        "refused": refused,
        "result": result,
        "runtimeMs": run.elapsed.as_millis() as u64,
        "tools": {
            "tl-mltl": env!("CARGO_PKG_VERSION"),
            "tl-oracle": locked_package("tl-oracle"),
            "tl-syntax": locked_package("tl-syntax"),
            "rustc": tool_version("rustc"),
            "cargo": tool_version("cargo"),
        },
        "disagreement": regression,
    });
    let dir = evidence_dir();
    if fs::create_dir_all(&dir).is_ok() {
        let path = dir.join(format!(
            "{}-{}.json",
            scope.name,
            profile.token().replace('/', "-")
        ));
        let _ = fs::write(path, serde_json::to_vec_pretty(&record).unwrap());
    }
    record
}

fn assert_agrees(profile: Profile, scope: &Scope) -> Value {
    let run = sweep(profile, scope, &Clone::clone);
    let regression = run.disagreement.as_ref().map(|d| {
        fixture(
            profile,
            "minimised",
            &minimise(
                profile,
                &Clone::clone,
                Disagreement {
                    formula: d.formula.clone(),
                    word: d.word.clone(),
                    at: d.at,
                },
            ),
        )
    });
    let record = write_evidence(profile, scope, &run, regression.clone());
    let summary = run.summary.unwrap_or_else(|e| {
        panic!(
            "{} {profile:?}: {e:?}; commit this minimised regression fixture: {}",
            scope.name,
            regression.unwrap_or(Value::Null)
        )
    });
    assert!(
        summary.admitted > 0 && summary.refused > 0,
        "{} {profile:?}",
        scope.name
    );
    assert_eq!(summary.admitted + summary.refused, summary.declared);
    record
}

// ---------------------------------------------------------------------------
// Scopes
// ---------------------------------------------------------------------------

/// Depth one over all four leaves, intervals within `0..=2`, words up to length 3.
const DEPTH_ONE: Scope = Scope {
    name: "depth-one",
    seed: 0x544C_0035,
    atoms: 2,
    constants: true,
    max_end: 2,
    formulas: Formulas::Exhaustive { depth: 1 },
    traces: Traces::Exhaustive {
        props: 2,
        max_len: 3,
    },
};

/// Fast subset: every depth-two formula over one proposition, intervals within
/// `0..=1`, every one-proposition word up to length 3.
const DEPTH_TWO_FAST: Scope = Scope {
    name: "depth-two-fast",
    seed: 0x544C_0224,
    atoms: 1,
    constants: false,
    max_end: 1,
    formulas: Formulas::Exhaustive { depth: 2 },
    traces: Traces::Exhaustive {
        props: 1,
        max_len: 3,
    },
};

/// Fast subset for the origin-past profile, whose production evaluator is two
/// orders of magnitude slower per case: point intervals `[0,0]` only.
const DEPTH_TWO_FAST_PAST: Scope = Scope {
    name: "depth-two-fast-past",
    max_end: 0,
    ..DEPTH_TWO_FAST
};

/// Full: every depth-one formula over all four leaves, every interval within
/// `0..=4`, every two-proposition word up to length 6.
const DEPTH_ONE_WIDE: Scope = Scope {
    name: "depth-one-wide",
    seed: 0x544C_0225,
    atoms: 2,
    constants: true,
    max_end: 4,
    formulas: Formulas::Exhaustive { depth: 1 },
    traces: Traces::Exhaustive {
        props: 2,
        max_len: 6,
    },
};

/// Full: every depth-two formula over one proposition, intervals within
/// `0..=2`, every one-proposition word up to length 6.
const DEPTH_TWO_FULL: Scope = Scope {
    name: "depth-two-full",
    seed: 0x544C_0226,
    atoms: 1,
    constants: false,
    max_end: 2,
    formulas: Formulas::Exhaustive { depth: 2 },
    traces: Traces::Exhaustive {
        props: 1,
        max_len: 6,
    },
};

/// Full: seeded depth-three formulas over every operator and every interval
/// within `0..=4`, each against seeded two-proposition words up to length 6.
/// Exhaustive depth three is not enumerable (the depth-two count alone exceeds
/// 10^8 at this width; see `exhaustive_populations_have_independently_computed_sizes`).
const DEPTH_THREE_SAMPLED: Scope = Scope {
    name: "depth-three-sampled",
    seed: 0x544C_0227,
    atoms: 2,
    constants: true,
    max_end: 4,
    formulas: Formulas::Sampled {
        depth: 3,
        count: 100_000,
    },
    traces: Traces::Sampled {
        max_len: 6,
        per_formula: 32,
    },
};

const PROFILES: [Profile; 2] = [Profile::ClosedFuture, Profile::OriginPast];

// Trace: TC-107
#[test]
fn closed_future_agrees_with_oracle_over_complete_finite_partition() {
    assert_agrees(Profile::ClosedFuture, &DEPTH_ONE);
}

// Trace: TC-107
#[test]
fn origin_past_agrees_with_oracle_over_complete_finite_partition() {
    assert_agrees(Profile::OriginPast, &DEPTH_ONE);
}

// Trace: TC-107
#[test]
fn fast_depth_two_subset_agrees_with_oracle_closed_future() {
    let record = assert_agrees(Profile::ClosedFuture, &DEPTH_TWO_FAST);
    // 15,811 formulas x (2 + 8 + 24) positions: the declared denominator is
    // computed independently of the enumerator.
    assert_eq!(record["formulaCount"], 15_811);
    assert_eq!(record["declaredCases"], 15_811u64 * 34);
}

// Trace: TC-107
#[test]
fn fast_depth_two_subset_agrees_with_oracle_origin_past() {
    let record = assert_agrees(Profile::OriginPast, &DEPTH_TWO_FAST_PAST);
    // [0,0] only: 6 unary and 8 binary operator forms over one leaf.
    assert_eq!(record["formulaCount"], 1_891);
    assert_eq!(record["declaredCases"], 1_891u64 * 34);
}

/// The nightly full sweep; run by `make sweep-full` in release mode.
// Trace: TC-107
#[test]
#[ignore = "nightly full sweep: run `make sweep-full`"]
fn full_small_scope_sweep_agrees_with_oracle() {
    for scope in [&DEPTH_ONE_WIDE, &DEPTH_TWO_FULL, &DEPTH_THREE_SAMPLED] {
        for profile in PROFILES {
            let record = assert_agrees(profile, scope);
            eprintln!("{record}");
        }
    }
}

#[test]
fn exhaustive_populations_have_independently_computed_sizes() {
    // Depth one: 4 leaves; per leaf Not, Y plus 4 ops x 6 intervals; per leaf
    // pair 4 Boolean plus 4 ops x 6 intervals.
    let depth_one = 4 + (2 + 4 * 6) * 4 + (4 + 4 * 6) * 16;
    assert_eq!(depth_one, 4 + 104 + 448);
    assert_eq!(DEPTH_ONE.formula_list().len(), depth_one);
    // Depth two over one leaf, 3 intervals: exact counts by closed form.
    let (unary, binary) = (2 + 4 * 3, 4 + 4 * 3);
    let d1 = 1 + unary + binary;
    let d2 = d1 + unary * (unary + binary) + binary * (d1 * d1 - 1);
    assert_eq!(DEPTH_TWO_FAST.formula_list().len(), d2);
    assert_eq!(
        DEPTH_TWO_FAST_PAST.formula_list().len(),
        15 + 6 * 14 + 8 * (15 * 15 - 1)
    );
    // Depth three is out of reach for exhaustive enumeration, so it is sampled.
    let (u5, b5) = (2 + 4 * 15, 4 + 4 * 15);
    let d1_wide = 4 + u5 * 4 + b5 * 16;
    let d2_wide_lower_bound = u5 * d1_wide + b5 * d1_wide * d1_wide;
    assert!(d2_wide_lower_bound > 100_000_000);
}

// Trace: TC-107
#[test]
fn sampled_population_is_deterministic_for_its_seed() {
    let a = DEPTH_THREE_SAMPLED.formula_list();
    let b = DEPTH_THREE_SAMPLED.formula_list();
    assert_eq!(a, b);
    assert_eq!(a.len(), 100_000);
    let other = Scope {
        seed: DEPTH_THREE_SAMPLED.seed + 1,
        ..DEPTH_THREE_SAMPLED
    };
    assert_ne!(a, other.formula_list());
    assert_eq!(
        DEPTH_THREE_SAMPLED.cases_for(7, &[]),
        DEPTH_THREE_SAMPLED.cases_for(7, &[])
    );
}

#[test]
fn disagreements_are_minimised_into_replayable_fixtures() {
    let scope = Scope {
        formulas: Formulas::Exhaustive { depth: 2 },
        ..DEPTH_TWO_FAST
    };
    for profile in PROFILES {
        let run = sweep(profile, &scope, &shrink_end);
        assert!(matches!(run.summary, Err(PopulationError::Mismatch { .. })));
        let found = run.disagreement.expect("disagreement recorded");
        let before = node_count(&found.formula) + found.word.len();
        let minimal = minimise(profile, &shrink_end, found);
        assert!(node_count(&minimal.formula) + minimal.word.len() <= before);
        // One temporal operator over a leaf and at most two instants.
        assert!(node_count(&minimal.formula) <= 2, "{:?}", minimal.formula);
        assert!(minimal.word.len() <= 2, "{:?}", minimal.word);
        let f = fixture(profile, "TL-224-minimised", &minimal);
        // Without the fault the committed fixture replays clean...
        replay(&f).expect("a fixed evaluator replays the fixture");
        // ...and with the fault the same case is still a disagreement.
        assert!(disagrees(profile, &shrink_end, &minimal));
    }
}

/// Every committed regression fixture must keep agreeing with the oracle.
#[test]
fn committed_regression_fixtures_replay_clean() {
    let dir = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/small-scope-regressions"
    ));
    let Ok(entries) = fs::read_dir(&dir) else {
        return; // no disagreement has ever been committed
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            replay(&value).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
    }
}

/// Seeded production faults (interval off-by-one, swapped duals, lost
/// negation) must each turn the comparison red in both profiles, at depth one
/// and in the fast depth-two subset.
// Trace: TC-107
#[test]
fn seeded_production_faults_turn_the_comparison_red() {
    type Fault = fn(&O) -> O;
    let faults: [(&str, Fault); 3] = [
        ("interval end off by one", shrink_end),
        ("dual operator swapped", swap_dual),
        ("negation dropped", drop_negation),
    ];
    for scope in [&DEPTH_ONE, &DEPTH_TWO_FAST] {
        for profile in PROFILES {
            for (name, fault) in faults {
                assert!(
                    matches!(
                        sweep(profile, scope, &fault).summary,
                        Err(PopulationError::Mismatch { .. })
                    ),
                    "{} {profile:?}: seeded fault `{name}` was not detected",
                    scope.name
                );
            }
        }
    }
}
