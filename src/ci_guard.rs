//! Bind the declared and executed `ci` gate set (NFR-006).
//!
//! This module holds the checkable logic behind `src/bin/ci_guard.rs`, the CI
//! entry point invoked in place of a bare `make ci`. It has three jobs:
//!
//! 1. [`scan_makefile`] refuses to proceed if the Makefile text — or, scanned
//!    recursively, any file it `include`s — carries an execution-control
//!    surface capable of suppressing prerequisite-failure propagation.
//! 2. [`dangerous_makeflags`] refuses to proceed if the calling environment's
//!    `MAKEFLAGS` carries the same suppression through a vector static text
//!    inspection cannot see.
//! 3. [`reconcile`], applied to the completion records each `ci` prerequisite
//!    recipe writes only on its own success, binds what was declared to what
//!    actually ran, independent of Make's own exit code.
//!
//! This remediates Linear TL-65 / `agent-ix/tl-mltl#14`: a single `.IGNORE:`
//! line, or an equivalent execution-control surface, used to make all 15 `ci`
//! prerequisites report success regardless of whether their own recipe
//! failed. See `spec/requirements/NFR-006-gate-set-integrity.md`. This
//! mirrors sibling repository tl-rewrite's merged `NFR-004-gate-set-integrity`
//! (Linear TL-64, `agent-ix/tl-rewrite#11`, PR #48), including both
//! bypass-class fixes (a bare-`;` recipe-chaining check and the wide-boundary
//! MAKEFLAGS/`|| true` matching below) folded in from the start rather than
//! discovered as later review findings.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Stable class of execution-control surface a [`Violation`] names.
///
/// A closed set, not a message: matching on this rather than comparing
/// strings is how a caller distinguishes one refusal reason from another
/// without re-parsing prose (mirrors this crate's other
/// `deserialize_contextual_record!`-style typed-error conventions).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ViolationKind {
    /// `.IGNORE:` special target.
    IgnoreDirective,
    /// `.SILENT:` special target.
    SilentDirective,
    /// `.ONESHELL:` special target.
    OneshellDirective,
    /// `.DEFAULT:` special target.
    DefaultDirective,
    /// `SHELL` assignment.
    ShellAssignment,
    /// `.SHELLFLAGS` assignment.
    ShellflagsAssignment,
    /// `MAKEFLAGS` assignment.
    MakeflagsAssignment,
    /// A `-`-prefixed recipe line at the start of a fresh recipe command.
    DashPrefixedRecipe,
    /// A recipe line containing `|| true`.
    RecipeSwallowsFailure,
    /// A recipe line redirecting stderr to `/dev/null`.
    RecipeHidesStderr,
    /// A recipe line joins two shell commands with a bare `;`, so Make's
    /// exit status for the line is the *last* command's, not the check's.
    RecipeChainsCommands,
    /// `$(eval` anywhere in the text.
    DynamicEval,
    /// The file (or an `include` target) could not be read.
    Unreadable,
}

impl ViolationKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::IgnoreDirective => "ignore-directive",
            Self::SilentDirective => "silent-directive",
            Self::OneshellDirective => "oneshell-directive",
            Self::DefaultDirective => "default-directive",
            Self::ShellAssignment => "shell-assignment",
            Self::ShellflagsAssignment => "shellflags-assignment",
            Self::MakeflagsAssignment => "makeflags-assignment",
            Self::DashPrefixedRecipe => "dash-prefixed-recipe",
            Self::RecipeSwallowsFailure => "recipe-swallows-failure",
            Self::RecipeHidesStderr => "recipe-hides-stderr",
            Self::RecipeChainsCommands => "recipe-chains-commands",
            Self::DynamicEval => "dynamic-eval",
            Self::Unreadable => "unreadable",
        }
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One execution-control surface found while scanning Makefile text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The file the surface was found in (the Makefile, or an `include`d file).
    pub file: PathBuf,
    /// 1-based line number; `0` for a whole-file refusal (e.g. unreadable).
    pub line: usize,
    /// The stable class of surface found.
    pub kind: ViolationKind,
    /// The matched text or a short human-readable explanation.
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} ({})",
            self.file.display(),
            self.line,
            self.kind,
            self.detail
        )
    }
}

const DIRECTIVE_TARGETS: [(&str, ViolationKind); 4] = [
    (".IGNORE:", ViolationKind::IgnoreDirective),
    (".SILENT:", ViolationKind::SilentDirective),
    (".ONESHELL:", ViolationKind::OneshellDirective),
    (".DEFAULT:", ViolationKind::DefaultDirective),
];

const ASSIGNED_VARS: [(&str, ViolationKind); 3] = [
    ("SHELL", ViolationKind::ShellAssignment),
    (".SHELLFLAGS", ViolationKind::ShellflagsAssignment),
    ("MAKEFLAGS", ViolationKind::MakeflagsAssignment),
];

/// Scan `path` (and, recursively, any `include`d file) for the
/// execution-control surfaces NFR-006-AC-1 names: `SHELL`, `.SHELLFLAGS`,
/// `MAKEFLAGS` assignment; `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, `.SILENT:`
/// as special targets; a `-`-prefixed recipe line; a recipe containing
/// `|| true`, a stderr-to-`/dev/null` redirect, or a bare `;` joining two
/// shell commands (Make's exit status for a recipe line is its *last*
/// command's); and `$(eval` anywhere.
///
/// Returns every violation found; an empty result means the text is clean.
/// An unreadable file — including an `include` target that cannot be
/// resolved — is itself a violation rather than a silent skip: this check
/// fails closed on anything it cannot read, exactly as it fails closed on
/// anything it can read and does not like.
pub fn scan_makefile(path: &Path) -> Vec<Violation> {
    let mut visited = BTreeSet::new();
    let mut out = Vec::new();
    scan_file(path, &mut visited, &mut out);
    out
}

fn scan_file(path: &Path, visited: &mut BTreeSet<PathBuf>, out: &mut Vec<Violation>) {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return; // already scanned on this walk; avoid an include cycle
    }
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            out.push(Violation {
                file: path.to_path_buf(),
                line: 0,
                kind: ViolationKind::Unreadable,
                detail: format!("cannot read {}: {err}", path.display()),
            });
            return;
        }
    };
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    // Tracks whether the current recipe line is a `\`-continuation of the
    // previous one, since Make's `-`/`@`/`+` prefix only means anything at
    // the start of a fresh recipe command, never on a continuation line
    // (where a leading `-` is just a wrapped command-line flag, e.g.
    // `--manifest ...` on its own wrapped line).
    let mut recipe_continues = false;
    for (idx, line) in text.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            recipe_continues = false; // a recipe block cannot span a blank/comment line
            continue;
        }
        if let Some(body) = line.strip_prefix('\t') {
            scan_recipe_line(path, lineno, body, !recipe_continues, out);
            recipe_continues = body.trim_end().ends_with('\\');
        } else {
            recipe_continues = false;
            scan_directive_line(path, lineno, trimmed, base_dir, visited, out);
        }
    }
}

fn scan_recipe_line(
    path: &Path,
    lineno: usize,
    body: &str,
    is_command_start: bool,
    out: &mut Vec<Violation>,
) {
    let mut push = |kind: ViolationKind, detail: String| {
        out.push(Violation {
            file: path.to_path_buf(),
            line: lineno,
            kind,
            detail,
        });
    };
    if is_command_start && recipe_prefix_carries_dash(body) {
        push(ViolationKind::DashPrefixedRecipe, body.trim().to_string());
    }
    if body.contains("|| true") || body.contains("|| :") {
        push(
            ViolationKind::RecipeSwallowsFailure,
            "`|| true`/`|| :` suppresses a non-zero exit".to_string(),
        );
    }
    if body.contains("2>/dev/null") || body.contains("2> /dev/null") {
        push(
            ViolationKind::RecipeHidesStderr,
            "stderr redirected to /dev/null".to_string(),
        );
    }
    if body.contains("$(eval") {
        push(
            ViolationKind::DynamicEval,
            "`$(eval` is refused outright, not analyzed".to_string(),
        );
    }
    if has_bare_command_separator(body) {
        push(
            ViolationKind::RecipeChainsCommands,
            "a bare `;` joins shell commands on one recipe line, so Make's \
             exit status for the line is the last command's, not an earlier \
             check's — e.g. `false; ci_guard record gate` reports success \
             because `ci_guard record` always exits 0"
                .to_string(),
        );
    }
}

/// True if `body` joins two shell commands with a bare `;` (a plain command
/// separator, not part of a `for`/`while`/`case` control-flow keyword or a
/// `;;` case-statement terminator). Make runs each recipe line as one shell
/// invocation whose exit status is the *last* command's; `cmd1; cmd2`
/// silently discards `cmd1`'s failure the same way `|| true` does, and is
/// how a recipe combining a check with `ci_guard record <gate>` (which
/// always exits 0) can report success despite the check failing — the
/// static scan must see it precisely because reconciliation cannot: a
/// record written that way is genuinely present and genuinely from this run.
/// This is not a hypothetical for this repository class: it defeated an
/// early version of the identical mechanism in sibling repositories
/// tl-rewrite and tl-parse before their fix, and this module bakes the fix
/// in from the start (NFR-006 Statement).
///
/// Textual, not shell-grammar-aware: a `;` inside a quoted string is still
/// flagged, the same accepted false-positive-over-false-negative trade-off
/// as this module's other recipe-content checks.
fn has_bare_command_separator(body: &str) -> bool {
    const CONTROL_WORDS: [&str; 7] = ["do", "done", "then", "else", "elif", "fi", "esac"];
    let mut chars = body.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if ch != ';' {
            continue;
        }
        if body[idx + 1..].starts_with(';') {
            chars.next(); // `;;` case-statement terminator, not a separator
            continue;
        }
        let rest = body[idx + 1..].trim_start();
        let is_control_word = CONTROL_WORDS.iter().any(|kw| {
            rest.strip_prefix(kw)
                .is_some_and(|after| !after.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
        });
        if !is_control_word {
            return true;
        }
    }
    false
}

/// `true` if `body`'s leading run of GNU Make recipe-prefix characters
/// (`@`, `-`, `+`, in any order and any repeat, immediately following each
/// other with no space) includes `-` (ignore this line's exit status).
///
/// Make does not require `-` to be the literal first character: `@-cmd` and
/// `+-cmd` are exactly as error-ignoring as `-cmd`, just also silent or
/// always-run respectively. A check anchored to `starts_with('-')` alone
/// misses every ordering where `-` is not first, which is not a hypothetical
/// — a recipe line reading `@-false` runs `false`, discards its failure via
/// the `-` prefix, and reaches its own `ci_guard record` call exactly as if
/// unprefixed, while `starts_with('-')` sees only the leading `@` and stays
/// silent.
fn recipe_prefix_carries_dash(body: &str) -> bool {
    let mut has_dash = false;
    for ch in body.trim_start().chars() {
        match ch {
            '-' => has_dash = true,
            '@' | '+' => {}
            _ => break,
        }
    }
    has_dash
}

fn scan_directive_line(
    path: &Path,
    lineno: usize,
    trimmed: &str,
    base_dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
    out: &mut Vec<Violation>,
) {
    for (target, kind) in DIRECTIVE_TARGETS {
        if trimmed.starts_with(target) {
            out.push(Violation {
                file: path.to_path_buf(),
                line: lineno,
                kind,
                detail: target.to_string(),
            });
        }
    }
    for (name, kind) in ASSIGNED_VARS {
        if is_assignment(trimmed, name) {
            out.push(Violation {
                file: path.to_path_buf(),
                line: lineno,
                kind,
                detail: trimmed.to_string(),
            });
        }
    }
    if trimmed.contains("$(eval") {
        out.push(Violation {
            file: path.to_path_buf(),
            line: lineno,
            kind: ViolationKind::DynamicEval,
            detail: "`$(eval` is refused outright, not analyzed".to_string(),
        });
    }
    if let Some(target) = include_target(trimmed) {
        let included = base_dir.join(target);
        scan_file(&included, visited, out);
    }
}

/// `true` if `line` assigns `name` via one of Make's assignment operators,
/// with or without a leading `export` keyword (`export MAKEFLAGS := -i` is
/// exactly as effective at suppressing failure propagation for the current
/// `make` invocation as the bare `MAKEFLAGS := -i` form already caught, and
/// unlike the bare form it is invisible to the separate calling-environment
/// `MAKEFLAGS` check, since it is set from inside the Makefile rather than
/// inherited — this repeats the same wide-boundary fix
/// [`dangerous_makeflags`] applies to the environment vector, on the Makefile
/// text vector instead).
fn is_assignment(line: &str, name: &str) -> bool {
    let candidate = strip_export_keyword(line);
    let Some(rest) = candidate.strip_prefix(name) else {
        return false;
    };
    let rest = rest.trim_start();
    ["=", ":=", "?=", "+=", "!="]
        .iter()
        .any(|op| rest.starts_with(op))
}

/// Strip a leading `export` keyword followed by required whitespace, e.g.
/// `export MAKEFLAGS := -i` -> `MAKEFLAGS := -i`. `export` must be a
/// standalone word — `exportable := 1` (a differently named variable that
/// merely starts with the same letters) is left untouched rather than
/// misread as `export able := 1`.
fn strip_export_keyword(line: &str) -> &str {
    match line.strip_prefix("export") {
        Some(rest) if rest.starts_with(|c: char| c.is_whitespace()) => rest.trim_start(),
        _ => line,
    }
}

fn include_target(line: &str) -> Option<&str> {
    for prefix in ["include ", "-include ", "sinclude "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

/// Parse the declared prerequisite set of `target`'s rule (e.g. `ci`) from
/// Makefile text, following `\`-continued lines. `None` if the target has no
/// rule head in the text.
pub fn parse_prerequisites(text: &str, target: &str) -> Option<BTreeSet<String>> {
    let head = format!("{target}:");
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_start() == line && line.starts_with(&head) {
            let mut collected = String::new();
            let mut cur = line[head.len()..].to_string();
            loop {
                let continues = cur.trim_end().ends_with('\\');
                let clean = cur.trim_end().trim_end_matches('\\').trim();
                collected.push(' ');
                collected.push_str(clean);
                if !continues {
                    break;
                }
                i += 1;
                if i >= lines.len() {
                    break;
                }
                cur = lines[i].to_string();
            }
            return Some(collected.split_whitespace().map(str::to_string).collect());
        }
        i += 1;
    }
    None
}

/// A flag or bundled short-flag token in `MAKEFLAGS` equivalent to ignoring
/// or continuing past a recipe failure. `-S`/`--no-keep-going`/`--stop`
/// cancel `-k` rather than cause it, and are deliberately not flagged.
///
/// This is a conservative, not exhaustive, parser: it recognizes the
/// documented long/short forms and Make's own bundled-letter form, in both
/// the space-free token Make itself uses when re-exporting `MAKEFLAGS` to a
/// sub-make (e.g. `ik`) and a `-`-prefixed bundle a caller might set by hand
/// (e.g. `-ik`) — a single re-exported token carries `-i` without an exact
/// bare `i`/`-i` word either way. It fails closed on either form rather than
/// trying to parse every possible flag combination. This wide-boundary
/// match is specified from the start here per NFR-006-AC-2, matching the
/// fix sibling repositories tl-rewrite and tl-parse needed after their
/// narrower initial versions let a bundled `-ik`-equivalent token through.
pub fn dangerous_makeflags(value: &str) -> Option<String> {
    for token in value.split_whitespace() {
        if token == "-i" || token == "--ignore-errors" {
            return Some(format!("MAKEFLAGS carries {token}"));
        }
        if token == "-k" || token == "--keep-going" {
            return Some(format!("MAKEFLAGS carries {token}"));
        }
        if token == "-S" || token == "--no-keep-going" || token == "--stop" {
            continue; // cancels -k rather than causing it; never flagged
        }
        if token.starts_with("--") {
            continue; // some other explicit long flag, not a bundled short form
        }
        // Either a bare bundled token (`ik`) or a single-dash-prefixed one
        // (`-ik`): strip at most one leading `-` before checking whether
        // every remaining character is a short-flag letter carrying `i` or
        // `k`. A flag taking a value (`-j4`, `-j 4`) or containing `=` is
        // left alone rather than misread as a letter bundle.
        let bundle = token.strip_prefix('-').unwrap_or(token);
        if !bundle.is_empty()
            && !bundle.contains('=')
            && bundle.chars().all(|c| c.is_ascii_alphabetic())
            && (bundle.contains('i') || bundle.contains('k'))
        {
            return Some(format!(
                "MAKEFLAGS carries a bundled short flag in {token:?}"
            ));
        }
    }
    None
}

/// A completion record one `ci` prerequisite's recipe writes only on its own
/// successful completion (NFR-006-AC-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateRecord {
    /// The `ci` prerequisite this record names (e.g. `fmt-check`).
    pub gate: String,
    /// The [`reconcile`] run this record belongs to; a record from a
    /// different run counts as absent (NFR-006-AC-5).
    pub run_id: String,
}

/// `true` if `gate` is safe to use as a bare filename component: every
/// declared `ci` prerequisite name in this repository's Makefile is
/// lowercase ASCII letters, digits, and `-` (e.g. `fmt-check`,
/// `check-corpus`), so that is the admitted alphabet. Rejects anything that
/// could escape the completion-record directory (`/`, `..`, a leading `.`)
/// along with anything simply outside the expected shape.
fn is_valid_gate_name(gate: &str) -> bool {
    !gate.is_empty()
        && gate
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Write `gate`'s completion record into `dir`, stamped with `run_id`.
///
/// `gate` is a CLI argument in practice (`ci_guard record GATE`, called from
/// a Makefile recipe with a literal gate name); not reachable with an
/// attacker-controlled value today, since every caller is a fixed literal in
/// this repository's own trusted Makefile. Validated anyway, matching this
/// module's fail-closed-on-untrusted-shape default: an invalid `gate` is
/// refused with an error rather than silently building a path that could
/// escape `dir` (NFR-006's own threat model is exactly "a recipe edit
/// reopens something this control was supposed to close").
pub fn write_record(dir: &Path, gate: &str, run_id: &str) -> std::io::Result<()> {
    if !is_valid_gate_name(gate) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("refusing to write a completion record for invalid gate name {gate:?}"),
        ));
    }
    fs::create_dir_all(dir)?;
    let record = GateRecord {
        gate: gate.to_string(),
        run_id: run_id.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&record)
        .expect("GateRecord serialization is infallible for these field types");
    fs::write(dir.join(format!("{gate}.json")), bytes)
}

/// Read every completion record in `dir`. A directory that does not exist,
/// or a record file that cannot be parsed, contributes no entry — malformed
/// or absent is the same "no record" input to [`reconcile`], never a pass.
pub fn read_records(dir: &Path) -> BTreeMap<String, GateRecord> {
    let mut out = BTreeMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_slice::<GateRecord>(&bytes) else {
            continue;
        };
        out.insert(record.gate.clone(), record);
    }
    out
}

/// Delete and recreate `dir` so a fresh run starts from no completion
/// records at all — nothing a prior invocation wrote can leak into this
/// run's reconciliation as a false pass.
pub fn reset_gates_dir(dir: &Path) -> std::io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)
}

/// The result of comparing a declared prerequisite set to the completion
/// records actually observed for `run_id` (NFR-006-AC-4/AC-5).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    /// Declared gates with no valid record for this run.
    pub missing: Vec<String>,
    /// Records for this run naming a gate that was not declared.
    pub unexpected: Vec<String>,
}

impl Reconciliation {
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.unexpected.is_empty()
    }
}

/// Bind `declared` to `records`. A record only counts if its `run_id`
/// matches the run under evaluation — a record left over from, replayed
/// from, or backdated to a different run is treated as no record at all,
/// not a stale-but-valid pass (NFR-006-AC-5).
pub fn reconcile(
    declared: &BTreeSet<String>,
    records: &BTreeMap<String, GateRecord>,
    run_id: &str,
) -> Reconciliation {
    let mut result = Reconciliation::default();
    for gate in declared {
        match records.get(gate) {
            Some(record) if record.run_id == run_id => {}
            _ => result.missing.push(gate.clone()),
        }
    }
    for (gate, record) in records {
        if record.run_id == run_id && !declared.contains(gate) {
            result.unexpected.push(gate.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(dir: &Path, name: &str, contents: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_clean_control_has_no_violations() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "CARGO ?= cargo\n\n.PHONY: ci\nci: fmt-check\n\nfmt-check:\n\t$(CARGO) fmt --all -- --check\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_ignore_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".IGNORE:\nci:\n\tfalse\n");
        let violations = scan_makefile(&path);
        assert!(violations
            .iter()
            .any(|v| v.kind == ViolationKind::IgnoreDirective));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_silent_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".SILENT:\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::SilentDirective));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_oneshell_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".ONESHELL:\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::OneshellDirective));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_default_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".DEFAULT:\n\t@true\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DefaultDirective));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_shell_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "SHELL := /bin/false\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::ShellAssignment));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_shellflags_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".SHELLFLAGS := -ec\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::ShellflagsAssignment));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_makeflags_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "MAKEFLAGS += -i\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::MakeflagsAssignment));
    }

    // Quoted/trailing-comment variations must still be caught — a too-narrow
    // regex (e.g. anchored on end-of-line) would let this slip through.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_makeflags_assignment_with_quoted_value_and_comment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "MAKEFLAGS := \"-i\" # keep going on failure\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::MakeflagsAssignment));
    }

    // SR-055/FND-002: `export MAKEFLAGS := -i` genuinely suppresses
    // prerequisite-failure propagation for the current `make` invocation,
    // not only a sub-make, and is invisible to the calling-environment
    // MAKEFLAGS check because it is set from inside the Makefile. A bare
    // (non-`export`) line in the same position was already caught before
    // this fix, isolating `export` as the exact gap.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_exported_makeflags_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "export MAKEFLAGS := -i\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::MakeflagsAssignment));
    }

    // A variable that merely starts with the letters "export" is a
    // different name and must not be misread as an exported assignment.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_does_not_misread_a_differently_named_variable_as_export() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "exportable := 1\nci:\n\tfalse\n");
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_dash_prefixed_recipe() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t-false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // SR-055/FND-001: GNU Make accepts `@`/`-`/`+` recipe prefixes in any
    // order, immediately following each other; `@-false` and `+-false` are
    // exactly as error-ignoring as `-false`, just also silent or
    // always-run respectively. A check anchored to a literal leading `-`
    // misses both orderings.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_dash_prefixed_recipe_with_at_prefix_first() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t@-false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_dash_prefixed_recipe_with_plus_prefix_first() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t+-false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // A recipe using only `@`/`+` (silent/always-run) without `-` does not
    // ignore the command's own exit status and must not be flagged.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_does_not_flag_at_or_plus_prefix_without_dash() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t@+echo hi\n");
        assert!(scan_makefile(&path).is_empty());
    }

    // Regression: a `\`-continued recipe line whose wrapped text happens to
    // start with `--flag` is not the Make error-ignoring `-` prefix and must
    // not be flagged.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_does_not_flag_a_wrapped_flag_on_a_continuation_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tcargo run --quiet --example x -- \\\n\t\t--manifest corpus/x.json\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // The actual `-` prefix is still caught even when the same recipe line
    // itself continues onto a further line.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_still_flags_dash_prefix_on_first_line_of_a_continued_recipe() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t-false \\\n\t\t--flag\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_recipe_swallowing_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse || true\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeSwallowsFailure));
    }

    // `|| :` is the POSIX no-op builtin, functionally identical to `|| true`
    // for this purpose; the detector must not be narrow to the `true` spelling.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_recipe_swallowing_failure_with_colon_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse || :\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeSwallowsFailure));
    }

    // `|| true` must be caught wherever it appears on the line, including
    // immediately before a trailing `;` and further commands — a boundary
    // anchored to end-of-line/whitespace would miss this.
    // Trace: TC-130, NFR-006-AC-1, NFR-006-AC-2
    #[test]
    fn scan_detects_recipe_swallowing_failure_before_trailing_semicolon() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse || true; echo done\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeSwallowsFailure));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_stderr_to_null() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse 2>/dev/null\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeHidesStderr));
    }

    // A recipe line joining the check and the record call with a bare `;`
    // is invisible to every other check here (no .IGNORE, no `-` prefix, no
    // `|| true`, no stderr redirect, no $(eval)) yet defeats reconciliation,
    // because Make's exit status for the line is `ci_guard record`'s (always
    // 0), not the check's. This is the exact bypass class NFR-006-AC-1
    // specifies from the start (not a later fix).
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_semicolon_chained_check_and_record() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci: gate-a\ngate-a:\n\tfalse; \"/path/to/ci_guard\" record gate-a\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeChainsCommands));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_simple_semicolon_chain() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\ttrue; false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeChainsCommands));
    }

    // A `for`/`do`/`done` loop's structural semicolons are not a command
    // separator hiding a failure and must not be flagged — over-flagging
    // ordinary shell control flow would make this check impractical to
    // leave on for any recipe using a loop.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_allows_for_loop_control_flow_semicolons() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tfor f in a b c; do echo $$f; done\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_allows_while_loop_control_flow_semicolons() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\twhile read -r line; do echo $$line; done < f\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_allows_if_control_flow_semicolons() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tif true; then echo yes; else echo no; fi\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // `;;` terminates a `case` branch; it is not two bare command separators.
    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_allows_case_statement_terminators() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tcase $$x in a) true ;; b) true ;; esac\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_detects_dynamic_eval() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "$(eval .IGNORE:)\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DynamicEval));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_recurses_into_include_and_flags_it() {
        let dir = tempfile::tempdir().unwrap();
        write_temp(dir.path(), "extra.mk", ".IGNORE:\n");
        let path = write_temp(dir.path(), "Makefile", "include extra.mk\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::IgnoreDirective && v.file.ends_with("extra.mk")));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_flags_unresolvable_include_as_a_violation() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "include does-not-exist.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::Unreadable));
    }

    // Trace: TC-130, NFR-006-AC-1
    #[test]
    fn scan_ignores_directives_named_only_in_comments() {
        // The Makefile's own header prose discusses .IGNORE:, $(eval), and
        // include as documentation; scanning must not self-trigger on it.
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "# .IGNORE: .SILENT: MAKEFLAGS := -i $(eval foo) include x.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-130, NFR-006-AC-4
    #[test]
    fn parse_prerequisites_handles_continuation_lines() {
        let text = "ci: fmt-check lint \\\n\tdeny audit-unsafe\n";
        let declared = parse_prerequisites(text, "ci").unwrap();
        let expected: BTreeSet<String> = ["fmt-check", "lint", "deny", "audit-unsafe"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(declared, expected);
    }

    // Trace: TC-130, NFR-006-AC-4
    #[test]
    fn parse_prerequisites_returns_none_when_target_absent() {
        assert!(parse_prerequisites("fmt-check:\n\tcargo fmt\n", "ci").is_none());
    }

    // Trace: TC-131, NFR-006-AC-2
    #[test]
    fn dangerous_makeflags_detects_dashed_ignore_errors() {
        assert!(dangerous_makeflags("-i").is_some());
    }

    // Trace: TC-131, NFR-006-AC-2
    #[test]
    fn dangerous_makeflags_detects_dashed_keep_going() {
        assert!(dangerous_makeflags("-k").is_some());
    }

    // GNU Make's own flag-bundling: a single re-exported token can carry
    // `-i` without an exact bare `i` word (e.g. the space-free `ik` Make
    // itself uses when re-exporting MAKEFLAGS to a sub-make).
    // Trace: TC-131, NFR-006-AC-2
    #[test]
    fn dangerous_makeflags_detects_bundled_short_flags() {
        assert!(dangerous_makeflags("ik").is_some());
        assert!(dangerous_makeflags("wik").is_some());
        assert!(dangerous_makeflags("-ik").is_some());
    }

    // Trace: TC-131, NFR-006-AC-2
    #[test]
    fn dangerous_makeflags_allows_clean_flags() {
        assert!(dangerous_makeflags("").is_none());
        assert!(dangerous_makeflags("w").is_none());
        assert!(dangerous_makeflags("--no-print-directory").is_none());
    }

    // Trace: TC-131, NFR-006-AC-2
    #[test]
    fn dangerous_makeflags_does_not_flag_stop_negation() {
        // -S/--no-keep-going CANCELS -k; it must never itself be flagged.
        assert!(dangerous_makeflags("-S").is_none());
        assert!(dangerous_makeflags("--no-keep-going").is_none());
    }

    // Trace: TC-133, NFR-006-AC-4
    #[test]
    fn reconcile_reports_missing_and_unexpected() {
        let declared: BTreeSet<String> = ["a", "b"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        records.insert(
            "a".to_string(),
            GateRecord {
                gate: "a".to_string(),
                run_id: "run-1".to_string(),
            },
        );
        records.insert(
            "c".to_string(),
            GateRecord {
                gate: "c".to_string(),
                run_id: "run-1".to_string(),
            },
        );
        let result = reconcile(&declared, &records, "run-1");
        assert_eq!(result.missing, vec!["b".to_string()]);
        assert_eq!(result.unexpected, vec!["c".to_string()]);
        assert!(!result.is_clean());
    }

    // Trace: TC-133, NFR-006-AC-4
    #[test]
    fn reconcile_is_clean_on_exact_match() {
        let declared: BTreeSet<String> = ["a", "b"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        for gate in ["a", "b"] {
            records.insert(
                gate.to_string(),
                GateRecord {
                    gate: gate.to_string(),
                    run_id: "run-1".to_string(),
                },
            );
        }
        assert!(reconcile(&declared, &records, "run-1").is_clean());
    }

    // Trace: TC-134, NFR-006-AC-5
    #[test]
    fn reconcile_treats_wrong_run_id_record_as_absent() {
        // A record left over from (or replayed/backdated from) a different
        // run must not count as a pass for the current run (NFR-006-AC-5).
        let declared: BTreeSet<String> = ["a"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        records.insert(
            "a".to_string(),
            GateRecord {
                gate: "a".to_string(),
                run_id: "stale-run".to_string(),
            },
        );
        let result = reconcile(&declared, &records, "current-run");
        assert_eq!(result.missing, vec!["a".to_string()]);
        assert!(result.unexpected.is_empty());
    }

    // Trace: TC-132, NFR-006-AC-3
    #[test]
    fn read_records_skips_malformed_and_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_records(dir.path().join("absent").as_path()).is_empty());

        let records_dir = dir.path().join("records");
        fs::create_dir_all(&records_dir).unwrap();
        fs::write(records_dir.join("broken.json"), b"not json").unwrap();
        assert!(read_records(&records_dir).is_empty());
    }

    // Trace: TC-132, NFR-006-AC-3
    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        write_record(dir.path(), "fmt-check", "run-1").unwrap();
        let records = read_records(dir.path());
        let record = records.get("fmt-check").unwrap();
        assert_eq!(record.run_id, "run-1");
    }

    // `gate` reaches the filesystem path unvalidated; a traversal-shaped
    // name must be refused rather than escaping `dir`.
    // Trace: TC-132, NFR-006-AC-3
    #[test]
    fn write_record_rejects_path_traversal_gate_names() {
        let dir = tempfile::tempdir().unwrap();
        for gate in ["../escape", "a/b", "/etc/passwd", "..", "", "Fmt-Check"] {
            let result = write_record(dir.path(), gate, "run-1");
            assert!(result.is_err(), "expected {gate:?} to be refused");
        }
        // Nothing escaped `dir`.
        assert!(read_records(dir.path()).is_empty());
        assert!(!dir.path().parent().unwrap().join("escape").exists());
    }

    // The exact 15 declared `ci` prerequisites this repository's Makefile
    // names (NFR-006 Scope), in the same order the Makefile declares them.
    // Trace: TC-132, NFR-006-AC-3
    #[test]
    fn write_record_accepts_every_real_gate_name() {
        let dir = tempfile::tempdir().unwrap();
        let gates = [
            "fmt-check",
            "lint",
            "kani-check",
            "test",
            "check-corpus",
            "conformance",
            "differential",
            "cli-conformance",
            "test-census",
            "deny",
            "audit-unsafe",
            "spec",
            "msrv",
            "rustdoc",
            "assurance",
        ];
        assert_eq!(gates.len(), 15);
        for gate in gates {
            write_record(dir.path(), gate, "run-1").unwrap();
        }
        assert_eq!(read_records(dir.path()).len(), 15);
    }

    // Trace: TC-134, NFR-006-AC-5
    #[test]
    fn reset_gates_dir_clears_prior_contents() {
        let dir = tempfile::tempdir().unwrap();
        let gates = dir.path().join("gates");
        write_record(&gates, "stale", "old-run").unwrap();
        reset_gates_dir(&gates).unwrap();
        assert!(read_records(&gates).is_empty());
    }
}
