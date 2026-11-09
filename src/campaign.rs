//! Reproducible campaign run: member inventory, execution, report and
//! source-bound replay.
//!
//! One declarative file, `campaign/members.json`, lists every member that
//! contributes evidence (test, fuzz, mutation, Kani, coverage, sweep,
//! differential). A later member registers by adding one entry there; the
//! inventory check refuses a Make target or fuzz target that looks like
//! evidence but is not covered by any entry.
//!
//! A run executes the registered members, records tool versions, seeds and the
//! digest of any evidence JSON they write, and emits one report. A member with
//! no command, or one whose tool is unavailable, is recorded as non-pass; the
//! aggregate is `accept` only when every required member passed.
//!
//! Replay is bound to the source the report names: it refuses unless the
//! checkout is the recorded commit, clean, with the recorded `Cargo.lock` and
//! members file, and the recorded tool versions. `exact` members must
//! reproduce status and evidence digest; `seeded` and `timed` members must
//! reproduce status and seed, and a rerun is a new attempt, never a
//! replacement of the retained report.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::fuzz_record::{parse_record, RunRecord, RunState};
use crate::kani_record::{self, ProofOutcome, ProofRecord};

/// Schema of the members file.
pub const MEMBERS_SCHEMA: &str = "tl-mltl.campaign-members/v1";
/// Schema of a campaign report.
pub const REPORT_SCHEMA: &str = "tl-mltl.campaign-run/v1";
/// Make-target names containing one of these words are evidence-bearing and
/// must be covered by a member.
const EVIDENCE_WORDS: [&str; 8] = [
    "fuzz",
    "mutants",
    "mutation",
    "kani",
    "cov",
    "sweep",
    "r2u2",
    "differential",
];

/// The members file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberFile {
    /// Must equal [`MEMBERS_SCHEMA`].
    pub schema_version: String,
    /// Tool version probes shared by members.
    pub tools: Vec<ToolProbe>,
    /// Every evidence-contributing member.
    pub members: Vec<Member>,
}

/// A command whose first output line is a tool's version.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolProbe {
    /// Name members refer to.
    pub name: String,
    /// Version command.
    pub command: Vec<String>,
}

/// Whether a member can run today.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberState {
    /// Has a command and runs.
    #[default]
    Registered,
    /// Reserved by a ticket that has not landed; always non-pass.
    Planned,
}

/// How a rerun is compared with the retained result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Determinism {
    /// Status and canonical evidence digest must repeat.
    Exact,
    /// Status and seed must repeat; raw evidence may differ.
    Seeded,
    /// Status must repeat; elapsed time is not compared.
    Timed,
}

/// Evidence JSON a member writes into a directory the runner provides.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceSpec {
    /// Environment variable that receives the directory.
    pub env: String,
    /// Keys removed (recursively) before the canonical digest.
    #[serde(default)]
    pub volatile_keys: Vec<String>,
}

/// One evidence-contributing member.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Member {
    /// Unique id.
    pub id: String,
    /// Technique or milestone label.
    pub technique: String,
    /// Member kind: test, fuzz, mutation, kani, coverage, sweep, differential.
    pub kind: String,
    /// Registration state.
    #[serde(default)]
    pub state: MemberState,
    /// Owning ticket for a planned member.
    #[serde(default)]
    pub ticket: Option<String>,
    /// Argument vector; required for a registered member.
    #[serde(default)]
    pub command: Vec<String>,
    /// Tool probes the member needs.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Rerun comparison rule.
    pub determinism: Determinism,
    /// Recorded seed for seeded members.
    #[serde(default)]
    pub seed: Option<String>,
    /// Evidence JSON the member writes.
    #[serde(default)]
    pub evidence: Option<EvidenceSpec>,
    /// `make:<target>` and `fuzz:<target>` entries this member accounts for.
    #[serde(default)]
    pub covers: Vec<String>,
    /// Required members gate the aggregate. Default true.
    #[serde(default = "yes")]
    pub required: bool,
    /// Free-text note carried into the report.
    #[serde(default)]
    pub note: Option<String>,
}

fn yes() -> bool {
    true
}

/// Parse and schema-check a members file.
pub fn parse_members(text: &str) -> Result<MemberFile, String> {
    let file: MemberFile = serde_json::from_str(text).map_err(|e| format!("members: {e}"))?;
    if file.schema_version != MEMBERS_SCHEMA {
        return Err(format!("members: schema {:?}", file.schema_version));
    }
    Ok(file)
}

/// Make target names declared in Makefile text.
pub fn make_targets(makefile: &str) -> BTreeSet<String> {
    makefile
        .lines()
        .filter(|l| !l.starts_with(['\t', ' ', '#', '.']))
        .filter_map(|l| l.split_once(':'))
        .filter(|(_, rest)| !rest.starts_with('='))
        .flat_map(|(names, _)| names.split_whitespace())
        .filter(|n| {
            n.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
        })
        .map(str::to_owned)
        .collect()
}

/// Check the member list against the repository. Returns one finding per
/// problem; empty means the inventory is complete and consistent.
pub fn check_inventory(
    file: &MemberFile,
    makefile: &str,
    fuzz_targets: &BTreeSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let probes: BTreeSet<&str> = file.tools.iter().map(|t| t.name.as_str()).collect();
    let mut ids = BTreeSet::new();
    let targets = make_targets(makefile);
    let mut covered = BTreeSet::new();
    for m in &file.members {
        if !ids.insert(m.id.as_str()) {
            out.push(format!("duplicate member id {}", m.id));
        }
        match m.state {
            MemberState::Registered if m.command.is_empty() => {
                out.push(format!("{}: registered member has no command", m.id));
            }
            MemberState::Planned if m.ticket.is_none() || !m.command.is_empty() => {
                out.push(format!(
                    "{}: planned member needs a ticket and no command",
                    m.id
                ));
            }
            _ => {}
        }
        if m.state == MemberState::Registered
            && m.determinism == Determinism::Seeded
            && m.seed.is_none()
        {
            out.push(format!("{}: seeded member records no seed", m.id));
        }
        for t in &m.tools {
            if !probes.contains(t.as_str()) {
                out.push(format!("{}: unknown tool probe {t}", m.id));
            }
        }
        if m.command.first().map(String::as_str) == Some("make") {
            if let Some(t) = m.command.get(1).filter(|t| !targets.contains(*t)) {
                out.push(format!("{}: make target {t} does not exist", m.id));
            }
        }
        for c in &m.covers {
            covered.insert(c.as_str());
            match c.split_once(':') {
                Some(("make", t)) if targets.contains(t) => {}
                Some(("fuzz", t)) if fuzz_targets.contains(t) => {}
                _ => out.push(format!("{}: covers {c} which does not exist", m.id)),
            }
        }
    }
    for t in targets
        .iter()
        .filter(|t| EVIDENCE_WORDS.iter().any(|w| t.contains(w)))
    {
        if !covered.contains(format!("make:{t}").as_str()) {
            out.push(format!(
                "make target {t} is evidence-bearing but no member covers it"
            ));
        }
    }
    for t in fuzz_targets {
        if !covered.contains(format!("fuzz:{t}").as_str()) {
            out.push(format!("fuzz target {t} has no member"));
        }
    }
    out
}

/// Names of `fuzz/fuzz_targets/*.rs` under `root`.
pub fn discover_fuzz_targets(root: &Path) -> BTreeSet<String> {
    fs::read_dir(root.join("fuzz/fuzz_targets"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension()? == "rs").then(|| p.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect()
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

fn strip_keys(v: &mut Value, keys: &[String]) {
    match v {
        Value::Object(m) => {
            m.retain(|k, _| !keys.contains(k));
            m.values_mut().for_each(|x| strip_keys(x, keys));
        }
        Value::Array(a) => a.iter_mut().for_each(|x| strip_keys(x, keys)),
        _ => {}
    }
}

/// Digest of the JSON text with `volatile` keys removed; object keys are
/// sorted by serde_json, so equal content gives equal digests.
pub fn canonical_digest(text: &[u8], volatile: &[String]) -> Result<String, String> {
    let mut v: Value = serde_json::from_slice(text).map_err(|e| e.to_string())?;
    strip_keys(&mut v, volatile);
    Ok(sha256_hex(
        &serde_json::to_vec(&v).map_err(|e| e.to_string())?,
    ))
}

/// Outcome of one member in a run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Ran, exit 0, evidence (if any) agreed.
    Pass,
    /// Ran and failed, or evidence disagreed.
    Fail,
    /// A needed tool, or the command itself, is not present on this machine.
    Unavailable,
    /// Planned member with no command yet.
    NotRegistered,
}

/// One retained evidence file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceFile {
    /// File name inside the member's evidence directory.
    pub name: String,
    /// SHA-256 of the raw bytes.
    pub sha256: String,
    /// SHA-256 after removing volatile keys.
    pub canonical_sha256: String,
    /// The record's own `result` field, when present.
    pub result: Option<String>,
}

impl EvidenceFile {
    fn unreadable(name: String, sha256: String) -> Self {
        Self {
            name,
            sha256,
            canonical_sha256: String::new(),
            result: Some(UNREADABLE.to_owned()),
        }
    }
}

/// `result` recorded for an evidence file that could not be read as JSON.
const UNREADABLE: &str = "unreadable";

/// One member's retained result.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemberResult {
    /// Member id.
    pub id: String,
    /// Technique label.
    pub technique: String,
    /// Kind.
    pub kind: String,
    /// Required flag copied from the member.
    pub required: bool,
    /// Outcome.
    pub status: Status,
    /// Why it is not a pass, when it is not.
    pub detail: Option<String>,
    /// Command run.
    pub command: Vec<String>,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Rerun rule.
    pub determinism: Determinism,
    /// Seed.
    pub seed: Option<String>,
    /// Tool versions this member depends on.
    pub tools: BTreeMap<String, String>,
    /// Evidence files, sorted by name.
    pub evidence: Vec<EvidenceFile>,
    /// Elapsed milliseconds; never compared on replay.
    pub elapsed_ms: u64,
    /// Per-run fuzz observations (seed, duration, executions, coverage,
    /// crashes) read from the member's fuzz run record.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuzz_runs: Vec<RunRecord>,
    /// Per-proof outcomes (harness, outcome, Kani and CBMC versions, covers,
    /// bounds) read from the member's Kani proof records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kani_proofs: Vec<ProofRecord>,
}

/// Source identity a report is bound to.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceBinding {
    /// `git rev-parse HEAD`.
    pub commit: String,
    /// True when the tree had uncommitted changes.
    pub dirty: bool,
    /// SHA-256 of `Cargo.lock`.
    pub cargo_lock_sha256: String,
    /// SHA-256 of the members file.
    pub members_sha256: String,
}

/// The campaign report.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Report {
    /// Must equal [`REPORT_SCHEMA`].
    pub schema_version: String,
    /// Source binding.
    pub source: SourceBinding,
    /// Tool probe results.
    pub tools: BTreeMap<String, String>,
    /// Member results in file order.
    pub members: Vec<MemberResult>,
    /// `accept` or `incomplete` or `reject`.
    pub aggregate: String,
    /// Statements the report does not make.
    pub non_claims: Vec<String>,
    /// Digest of this report with this field emptied; set by [`seal`].
    pub report_sha256: String,
}

/// Compute and set `report_sha256` over the report with that field empty.
pub fn seal(report: &mut Report) -> Result<(), String> {
    report.report_sha256.clear();
    let bytes = serde_json::to_vec(report).map_err(|e| e.to_string())?;
    report.report_sha256 = sha256_hex(&bytes);
    Ok(())
}

/// True when the stored digest matches the content.
pub fn verify_seal(report: &Report) -> bool {
    let mut copy = report.clone();
    seal(&mut copy).is_ok() && copy.report_sha256 == report.report_sha256
}

/// Aggregate rule: any failed required member rejects; any non-pass required
/// member otherwise leaves the run incomplete. A run over a subset of members
/// is labelled `partial` by [`run_campaign`], never `accept`.
pub fn aggregate(members: &[MemberResult]) -> &'static str {
    let req = || members.iter().filter(|m| m.required);
    if req().any(|m| m.status == Status::Fail) {
        "reject"
    } else if req().all(|m| m.status == Status::Pass) {
        "accept"
    } else {
        "incomplete"
    }
}

fn run_capture(argv: &[String], dir: &Path, env: &[(String, String)]) -> Option<(i32, Vec<u8>)> {
    let (prog, args) = argv.split_first()?;
    let out = Command::new(prog)
        .args(args)
        .current_dir(dir)
        .envs(env.iter().cloned())
        .output()
        .ok()?;
    let mut text = out.stdout;
    text.extend(out.stderr);
    Some((out.status.code().unwrap_or(-1), text))
}

/// Probe every tool; `unavailable` when the command fails.
pub fn probe_tools(file: &MemberFile, dir: &Path) -> BTreeMap<String, String> {
    file.tools
        .iter()
        .map(|t| {
            let v = run_capture(&t.command, dir, &[])
                .filter(|(code, _)| *code == 0)
                .and_then(|(_, o)| String::from_utf8(o).ok())
                .and_then(|s| s.lines().next().map(|l| l.trim().to_owned()))
                .unwrap_or_else(|| "unavailable".to_owned());
            (t.name.clone(), v)
        })
        .collect()
}

fn collect_evidence(dir: &Path, spec: &EvidenceSpec) -> Vec<EvidenceFile> {
    let mut files: Vec<EvidenceFile> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            // An unreadable or unparseable evidence file is kept and marked, so
            // it fails the member instead of vanishing from the report.
            let Ok(bytes) = fs::read(e.path()) else {
                return EvidenceFile::unreadable(name, String::new());
            };
            let sha256 = sha256_hex(&bytes);
            let Ok(canonical_sha256) = canonical_digest(&bytes, &spec.volatile_keys) else {
                return EvidenceFile::unreadable(name, sha256);
            };
            let result = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| v.get("result")?.as_str().map(str::to_owned));
            EvidenceFile {
                name,
                sha256,
                canonical_sha256,
                result,
            }
        })
        .collect();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

fn blank(m: &Member, status: Status, detail: String) -> MemberResult {
    MemberResult {
        id: m.id.clone(),
        technique: m.technique.clone(),
        kind: m.kind.clone(),
        required: m.required,
        status,
        detail: Some(detail),
        command: m.command.clone(),
        exit_code: None,
        determinism: m.determinism,
        seed: m.seed.clone(),
        tools: BTreeMap::new(),
        evidence: Vec::new(),
        elapsed_ms: 0,
        fuzz_runs: Vec::new(),
        kani_proofs: Vec::new(),
    }
}

/// Run one member from `repo`, writing its evidence under `out/evidence/<id>`.
pub fn run_member(
    m: &Member,
    tools: &BTreeMap<String, String>,
    repo: &Path,
    out: &Path,
) -> MemberResult {
    if m.state == MemberState::Planned {
        let t = m.ticket.as_deref().unwrap_or("?");
        return blank(
            m,
            Status::NotRegistered,
            format!("planned under {t}; no command yet"),
        );
    }
    let used: BTreeMap<String, String> = m
        .tools
        .iter()
        .map(|t| {
            (
                t.clone(),
                tools
                    .get(t)
                    .cloned()
                    .unwrap_or_else(|| "unavailable".into()),
            )
        })
        .collect();
    if let Some((name, _)) = used.iter().find(|(_, v)| v.as_str() == "unavailable") {
        let mut r = blank(
            m,
            Status::Unavailable,
            format!("tool {name} is unavailable"),
        );
        r.tools = used;
        return r;
    }
    let ev_dir: Option<PathBuf> = m
        .evidence
        .as_ref()
        .map(|_| out.join("evidence").join(&m.id));
    let mut env = Vec::new();
    if let (Some(d), Some(spec)) = (&ev_dir, &m.evidence) {
        let _ = fs::remove_dir_all(d);
        let _ = fs::create_dir_all(d);
        env.push((spec.env.clone(), d.to_string_lossy().into_owned()));
    }
    let start = Instant::now();
    let ran = run_capture(&m.command, repo, &env);
    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let Some((code, log)) = ran else {
        let mut r = blank(
            m,
            Status::Unavailable,
            "command could not be started".into(),
        );
        r.tools = used;
        return r;
    };
    let _ = fs::create_dir_all(out.join("logs"));
    let _ = fs::write(out.join("logs").join(format!("{}.log", m.id)), &log);
    let evidence = match (&ev_dir, &m.evidence) {
        (Some(d), Some(spec)) => collect_evidence(d, spec),
        _ => Vec::new(),
    };
    let (fuzz_runs, fuzz_problem) = read_fuzz_runs(m, ev_dir.as_deref(), &evidence);
    let (kani_proofs, kani_problem) = read_kani_proofs(repo, m, ev_dir.as_deref(), &evidence);
    let fuzz_problem = fuzz_problem.or(kani_problem);
    let disagree = evidence
        .iter()
        .find(|e| e.result.as_deref().is_some_and(|r| r != "agree"));
    let (status, detail) = if code != 0 {
        (Status::Fail, Some(format!("exit {code}")))
    } else if let Some(e) = disagree {
        (
            Status::Fail,
            Some(format!("evidence {} result {:?}", e.name, e.result)),
        )
    } else if let Some(problem) = fuzz_problem {
        (Status::Fail, Some(problem))
    } else if m.evidence.is_some() && evidence.is_empty() {
        (
            Status::Fail,
            Some("declared evidence was not written".to_owned()),
        )
    } else {
        (Status::Pass, None)
    };
    MemberResult {
        status,
        detail,
        exit_code: Some(code),
        tools: used,
        evidence,
        elapsed_ms,
        fuzz_runs,
        kani_proofs,
        ..blank(m, Status::Pass, String::new())
    }
}

/// Read the fuzz run records among a member's evidence files. A record that
/// does not parse strictly, or a run that did not complete, fails the member.
fn read_fuzz_runs(
    m: &Member,
    dir: Option<&Path>,
    evidence: &[EvidenceFile],
) -> (Vec<RunRecord>, Option<String>) {
    let mut runs = Vec::new();
    let Some(dir) = dir.filter(|_| m.kind == "fuzz") else {
        return (runs, None);
    };
    // Every evidence file of a fuzz member is a run record: a wrong schema or
    // an unreadable file is an error, and a member with no record fails.
    for file in evidence {
        let text = match fs::read(dir.join(&file.name)) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(e) => return (Vec::new(), Some(format!("evidence {}: {e}", file.name))),
        };
        match parse_record(&text) {
            Ok(record) => runs.extend(record.runs),
            Err(e) => return (Vec::new(), Some(format!("evidence {}: {e}", file.name))),
        }
    }
    if runs.is_empty() {
        return (runs, Some("fuzz member wrote no run record".to_owned()));
    }
    let problem = runs
        .iter()
        .find(|r| r.state != RunState::Completed)
        .map(|r| format!("fuzz run {} seed {}: {:?}", r.target, r.seed, r.state));
    (runs, problem)
}

/// Read the Kani proof records among a member's evidence files. A record that
/// does not parse strictly, or a proof that did not verify, fails the member;
/// each non-verified outcome keeps its own name in the failure detail.
fn read_kani_proofs(
    repo: &Path,
    m: &Member,
    dir: Option<&Path>,
    evidence: &[EvidenceFile],
) -> (Vec<ProofRecord>, Option<String>) {
    let mut proofs = Vec::new();
    let Some(dir) = dir.filter(|_| m.kind == "kani") else {
        return (proofs, None);
    };
    for file in evidence {
        let text = match fs::read(dir.join(&file.name)) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(e) => return (Vec::new(), Some(format!("evidence {}: {e}", file.name))),
        };
        match kani_record::parse_record(&text) {
            Ok(record) => proofs.extend(record.proofs),
            Err(e) => return (Vec::new(), Some(format!("evidence {}: {e}", file.name))),
        }
    }
    if proofs.is_empty() {
        return (proofs, Some("kani member wrote no proof record".to_owned()));
    }
    if let Some(missing) = expected_harnesses(repo)
        .into_iter()
        .find(|h| !proofs.iter().any(|p| &p.harness == h))
    {
        return (proofs, Some(format!("kani proof {missing}: no record")));
    }
    let problem = proofs
        .iter()
        .find(|p| p.outcome != ProofOutcome::Verified)
        .map(|p| format!("kani proof {}: {:?}", p.harness, p.outcome));
    (proofs, problem)
}

/// Harness names the member must have a record for: the lines of
/// `scripts/kani-harnesses.txt` in `repo`. A repository without the list
/// requires none.
fn expected_harnesses(repo: &Path) -> Vec<String> {
    fs::read_to_string(repo.join("scripts/kani-harnesses.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Bind a report to the checkout at `repo`.
pub fn bind_source(repo: &Path, members_text: &str) -> Result<SourceBinding, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let argv: Vec<String> = std::iter::once("git")
            .chain(args.iter().copied())
            .map(Into::into)
            .collect();
        match run_capture(&argv, repo, &[]) {
            Some((0, o)) => Ok(String::from_utf8_lossy(&o).trim().to_owned()),
            _ => Err(format!("git {args:?} failed")),
        }
    };
    let lock = fs::read(repo.join("Cargo.lock")).map_err(|e| format!("Cargo.lock: {e}"))?;
    Ok(SourceBinding {
        commit: git(&["rev-parse", "HEAD"])?,
        dirty: !git(&["status", "--porcelain"])?.is_empty(),
        cargo_lock_sha256: sha256_hex(&lock),
        members_sha256: sha256_hex(members_text.as_bytes()),
    })
}

/// Non-claims every report carries.
pub fn standard_non_claims() -> Vec<String> {
    vec![
        "A planned or unavailable member is non-pass, never skipped-as-pass.".to_owned(),
        "A seeded or timed member's rerun is a new attempt compared under its declared rule."
            .to_owned(),
        "The retained R2U2 replay reads recorded bytes and does not execute R2U2.".to_owned(),
    ]
}

/// Run the selected members (all when `only` is empty) and seal the report.
pub fn run_campaign(repo: &Path, out: &Path, only: &[String]) -> Result<Report, String> {
    let text = fs::read_to_string(repo.join("campaign/members.json"))
        .map_err(|e| format!("campaign/members.json: {e}"))?;
    let file = parse_members(&text)?;
    let findings = check_inventory(
        &file,
        &fs::read_to_string(repo.join("Makefile")).unwrap_or_default(),
        &discover_fuzz_targets(repo),
    );
    if !findings.is_empty() {
        return Err(format!("inventory incomplete: {}", findings.join("; ")));
    }
    if let Some(x) = only
        .iter()
        .find(|o| !file.members.iter().any(|m| &m.id == *o))
    {
        return Err(format!("unknown member {x}"));
    }
    let tools = probe_tools(&file, repo);
    let members: Vec<MemberResult> = file
        .members
        .iter()
        .filter(|m| only.is_empty() || only.contains(&m.id))
        .map(|m| run_member(m, &tools, repo, out))
        .collect();
    let mut report = Report {
        schema_version: REPORT_SCHEMA.to_owned(),
        source: bind_source(repo, &text)?,
        tools,
        aggregate: if members.len() < file.members.len() {
            "partial".to_owned()
        } else {
            aggregate(&members).to_owned()
        },
        members,
        non_claims: standard_non_claims(),
        report_sha256: String::new(),
    };
    seal(&mut report)?;
    Ok(report)
}

/// Refusals that stop a replay before any member runs.
pub fn replay_preconditions(report: &Report, repo: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if !verify_seal(report) {
        out.push("report digest does not match its content".to_owned());
    }
    if report.source.dirty {
        out.push("report was produced from a dirty tree and cannot be replayed".to_owned());
    }
    let text = fs::read_to_string(repo.join("campaign/members.json")).unwrap_or_default();
    match bind_source(repo, &text) {
        Err(e) => out.push(e),
        Ok(now) => {
            if now.commit != report.source.commit {
                out.push(format!(
                    "checkout is {} but report is bound to {}",
                    now.commit, report.source.commit
                ));
            }
            if now.dirty {
                out.push("checkout has uncommitted changes".to_owned());
            }
            if now.cargo_lock_sha256 != report.source.cargo_lock_sha256 {
                out.push("Cargo.lock differs from the report".to_owned());
            }
            if now.members_sha256 != report.source.members_sha256 {
                out.push("campaign/members.json differs from the report".to_owned());
            }
        }
    }
    if let Ok(file) = parse_members(&text) {
        for (k, v) in probe_tools(&file, repo) {
            if report.tools.get(&k) != Some(&v) {
                out.push(format!(
                    "tool {k}: report has {:?}, now {v:?}",
                    report.tools.get(&k)
                ));
            }
        }
    }
    out
}

/// Compare a rerun member with the retained one under the retained rule.
pub fn compare_member(old: &MemberResult, new: &MemberResult) -> Vec<String> {
    let mut d = Vec::new();
    if old.status != new.status {
        d.push(format!(
            "{}: status {:?} -> {:?}",
            old.id, old.status, new.status
        ));
    }
    if old.seed != new.seed {
        d.push(format!("{}: seed {:?} -> {:?}", old.id, old.seed, new.seed));
    }
    let key = |m: &MemberResult| -> Vec<(String, u64, RunState)> {
        m.fuzz_runs
            .iter()
            .map(|r| (r.target.clone(), r.seed, r.state))
            .collect()
    };
    if key(old) != key(new) {
        d.push(format!(
            "{}: fuzz run targets, seeds or states differ",
            old.id
        ));
    }
    let proofs = |m: &MemberResult| -> Vec<(String, ProofOutcome, String, String)> {
        m.kani_proofs
            .iter()
            .map(|p| {
                (
                    p.harness.clone(),
                    p.outcome,
                    p.kani_version.clone(),
                    p.cbmc_version.clone(),
                )
            })
            .collect()
    };
    if proofs(old) != proofs(new) {
        d.push(format!(
            "{}: kani proof harnesses, outcomes or tool versions differ",
            old.id
        ));
    }
    if old.determinism == Determinism::Exact {
        let key = |m: &MemberResult| -> Vec<(String, String)> {
            m.evidence
                .iter()
                .map(|e| (e.name.clone(), e.canonical_sha256.clone()))
                .collect()
        };
        if key(old) != key(new) {
            d.push(format!("{}: evidence digests differ", old.id));
        }
    }
    d
}

/// Replay every member in `report`; returns the new attempt and the
/// differences against the retained report. The retained report is untouched.
pub fn replay(report: &Report, repo: &Path, out: &Path) -> Result<(Report, Vec<String>), String> {
    let refusals = replay_preconditions(report, repo);
    if !refusals.is_empty() {
        return Err(refusals.join("; "));
    }
    let only: Vec<String> = report.members.iter().map(|m| m.id.clone()).collect();
    let attempt = run_campaign(repo, out, &only)?;
    let diffs = report
        .members
        .iter()
        .zip(&attempt.members)
        .flat_map(|(o, n)| compare_member(o, n))
        .collect();
    Ok((attempt, diffs))
}
