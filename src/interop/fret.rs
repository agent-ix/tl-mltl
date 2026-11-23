//! FRET export import: MLTL formula files (FR-087) and the C2PO bundle with
//! its variable map and trace header (FR-088). FRETish text, project files and
//! requirement-editing metadata are out of scope and refused by name.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! SPEC-GAP / UNGROUNDED: no FRET export was available offline. The spellings
//! read here are provisional and must be rewritten against the retrieved
//! fixtures (FR-087): one requirement per non-empty line, an optional
//! `identifier:` prefix, `#` comment lines, `G F U R` with closed `[a,b]`
//! intervals, `! ~ & && | || -> <->`, `true`/`false`. The variable map uses
//! the retained R2U2 `.map` form `variable:signal-id`; the trace header uses
//! `#`-prefixed lines, either the retained R2U2 `# a,b` signal list or
//! `# key: value` fields.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use tl_syntax::{
    Formula, Node, NodeId, NodeKind, PropositionId, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId,
};

use super::attribution::{Attribution, AttributionError};
use super::c2po::{
    import_c2po_requirements, issue_loss_kind, lex, prescan, C2poImportError, ImportedRequirement,
    Issue, Parser, Tok, Token,
};
use super::loss::{ForeignFormat, LossKind, LossRecord};
use super::{admit_source, ReaderError, ReaderLimits, SourceRange};

/// Profile an imported FRET formula is read under; always explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FretProfile {
    /// `mltl.closed-trace/v1`.
    ClosedTrace,
    /// `mltl.online-prefix/v1`.
    OnlinePrefix,
}

impl FretProfile {
    /// The semantic profile.
    pub const fn semantic(self) -> SemanticProfile {
        match self {
            Self::ClosedTrace => SemanticProfile::ClosedTraceV1,
            Self::OnlinePrefix => SemanticProfile::OnlinePrefixV1,
        }
    }
}

/// File-level FRET import refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FretImportError {
    /// Hostile input.
    Reader(ReaderError),
    /// Attribution incomplete; refused before import.
    Attribution(AttributionError),
    /// The bundle's C2PO file was refused at file level.
    C2po(C2poImportError),
}

impl From<ReaderError> for FretImportError {
    fn from(error: ReaderError) -> Self {
        Self::Reader(error)
    }
}

impl fmt::Display for FretImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(error) => error.fmt(formatter),
            Self::Attribution(error) => write!(formatter, "attribution refused: {error:?}"),
            Self::C2po(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FretImportError {}

/// One imported FRET requirement; propositions are `variables` indices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FretRequirement {
    /// Requirement identifier, when the line carries one.
    pub id: Option<String>,
    /// Zero-based requirement index.
    pub index: usize,
    /// Span of the formula text.
    pub span: SourceRange,
    /// Nodes.
    pub nodes: Vec<Node>,
    /// Root.
    pub root: NodeId,
}

impl FretRequirement {
    /// The validated formula view.
    pub fn formula(&self, profile: SemanticProfile) -> Formula<'_> {
        Formula::new(profile, self.root, &self.nodes).expect("validated at import")
    }
}

/// FRET MLTL import result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FretMltlImport {
    /// Selected profile.
    pub profile: SemanticProfile,
    /// Upstream attribution carried into the result.
    pub attribution: Attribution,
    /// Variables in first-occurrence order; variable `i` is `PropositionId(i)`.
    pub variables: Vec<String>,
    /// Requirements the file declares (non-empty, non-comment lines).
    pub declared: usize,
    /// Imported requirements.
    pub imported: Vec<FretRequirement>,
    /// One loss record per requirement not imported.
    pub unimported: Vec<LossRecord>,
}

const FRETISH_WORDS: [&str; 6] = ["shall", "whenever", "upon", "within", "until_", "satisfy"];
const OPERATOR_WORDS: [&str; 14] = [
    "F", "G", "U", "R", "O", "H", "Y", "S", "T", "X", "true", "false", "shall", "whenever",
];

/// Lexes one line with the C2PO lexer and normalizes FRET's single-character
/// spellings (`&`, `|`, `~`); spans are shifted to the file.
fn lex_line(text: &str, offset: usize, limits: ReaderLimits) -> Result<Vec<Token>, ReaderError> {
    let mut tokens = lex(text, limits)?;
    for token in &mut tokens {
        if token.tok == Tok::Unknown {
            token.tok = match &text[token.span.start..token.span.end] {
                "&" => Tok::AndAnd,
                "|" => Tok::OrOr,
                "~" => Tok::Bang,
                _ => Tok::Unknown,
            };
        }
        token.span = SourceRange {
            start: token.span.start + offset,
            end: token.span.end + offset,
        };
    }
    Ok(tokens)
}

/// Splits an optional `identifier:` prefix.
fn split_id(line: &str) -> (Option<&str>, usize) {
    if let Some(colon) = line.find(':') {
        let id = line[..colon].trim();
        let plain = !id.is_empty()
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
        if plain {
            return (Some(id), colon + 1);
        }
    }
    (None, 0)
}

/// Classifies a requirement's tokens before parsing.
fn foreign_issue(tokens: &[Token]) -> Option<(LossKind, String, SourceRange)> {
    for token in tokens {
        if let Tok::Ident(name) = &token.tok {
            if FRETISH_WORDS.contains(&name.as_str()) {
                return Some((LossKind::ForeignConstruct, "fretish".to_owned(), token.span));
            }
            if name == "X" {
                return Some((LossKind::AmbiguousNext, "X".to_owned(), token.span));
            }
        }
        if matches!(token.tok, Tok::LBrace) {
            return Some((LossKind::ForeignConstruct, "project-file".to_owned(), token.span));
        }
    }
    None
}

fn register_variables(tokens: &[Token], variables: &mut Vec<String>, names: &mut BTreeMap<String, u32>) {
    for (index, token) in tokens.iter().enumerate() {
        if let Tok::Ident(name) = &token.tok {
            let operator_like = OPERATOR_WORDS.contains(&name.as_str())
                || matches!(
                    tokens.get(index + 1).map(|t| &t.tok),
                    Some(Tok::LParen) | Some(Tok::LBracket)
                );
            if !operator_like && !names.contains_key(name) {
                names.insert(name.clone(), variables.len() as u32);
                variables.push(name.clone());
            }
        }
    }
}

/// Imports a FRET-exported MLTL file requirement by requirement.
///
/// A bare next is `AmbiguousNext`, an operator with no interval is
/// `MissionClosureRequired`, FRETish or project-file content is
/// `ForeignConstruct`; none is lowered or approximated.
///
/// Implements: FR-087, FR-089 (attribution), FR-091 (loss records)
pub fn import_fret_mltl(
    bytes: &[u8],
    profile: FretProfile,
    attribution: &Attribution,
    limits: ReaderLimits,
) -> Result<FretMltlImport, FretImportError> {
    attribution
        .validate_fret()
        .map_err(FretImportError::Attribution)?;
    let text = admit_source(bytes, limits)?;
    let semantic = profile.semantic();
    let mut variables = Vec::new();
    let mut names = BTreeMap::new();
    let mut imported = Vec::new();
    let mut unimported = Vec::new();
    let mut declared = 0usize;
    let mut line_start = 0usize;
    for line in text.split_inclusive('\n') {
        let offset = line_start;
        line_start += line.len();
        let body = line.trim_end_matches(['\n', '\r']);
        if body.trim().is_empty() || body.trim_start().starts_with('#') {
            continue;
        }
        let index = declared;
        declared += 1;
        let (id, skip) = split_id(body);
        let formula_text = &body[skip..];
        let span = SourceRange {
            start: offset + skip,
            end: offset + body.len(),
        };
        let tokens = lex_line(formula_text, offset + skip, limits)?;
        if let Some((kind, construct, at)) = foreign_issue(&tokens) {
            unimported.push(LossRecord::at(kind, ForeignFormat::FretMltl, construct, at, Some(index)));
            continue;
        }
        if let Some(issue) = prescan(&tokens) {
            unimported.push(issue_record(issue, index));
            continue;
        }
        register_variables(&tokens, &mut variables, &mut names);
        let mut parser = Parser {
            tokens: &tokens,
            position: 0,
            nodes: Vec::new(),
            names: &names,
            max_nesting: limits.max_nesting,
        };
        let root = match parser.parse_all() {
            Ok(root) => root,
            Err(issue) => {
                unimported.push(issue_record(issue, index));
                continue;
            }
        };
        if let Err(error) = Formula::new(semantic, root, &parser.nodes) {
            unimported.push(LossRecord::at(
                LossKind::UnsupportedOperator,
                ForeignFormat::FretMltl,
                format!("{error:?}"),
                span,
                Some(index),
            ));
            continue;
        }
        imported.push(FretRequirement {
            id: id.map(str::to_owned),
            index,
            span,
            nodes: parser.nodes,
            root,
        });
    }
    if declared == 0 {
        return Err(ReaderError::Empty.into());
    }
    Ok(FretMltlImport {
        profile: semantic,
        attribution: attribution.clone(),
        variables,
        declared,
        imported,
        unimported,
    })
}

fn issue_record(issue: Issue, index: usize) -> LossRecord {
    let kind = issue_loss_kind(&issue);
    let (construct, span) = match issue {
        Issue::Construct(construct, span) => (construct.name().to_owned(), span),
        Issue::Undeclared(name, span) => (name, span),
        Issue::Interval(span) => ("interval".to_owned(), span),
        Issue::Nesting(span) => ("nesting".to_owned(), span),
        Issue::Unbounded(span) => ("unbounded-operator".to_owned(), span),
    };
    LossRecord::at(kind, ForeignFormat::FretMltl, construct, span, Some(index))
}

// ------------------------------------------------------------------ bundle

/// One variable bound to one catalog signal and its proposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableBinding {
    /// FRET / C2PO variable name.
    pub variable: String,
    /// Catalog signal.
    pub signal: SignalId,
    /// Catalog proposition bound to that signal.
    pub proposition: PropositionId,
}

/// Recognized trace-header fields.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraceHeader {
    /// Signal order (`# a,b` or `# signals: a,b`).
    pub signals: Option<Vec<String>>,
    /// Mission time (`# mission-time: N`), passed to the FR-090 report.
    pub mission_time: Option<u32>,
}

/// FRET bundle import result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FretBundleImport {
    /// Selected profile.
    pub profile: SemanticProfile,
    /// Attribution.
    pub attribution: Attribution,
    /// Bound variables.
    pub bindings: Vec<VariableBinding>,
    /// Recognized header fields.
    pub header: TraceHeader,
    /// Variable-map and header loss records (never abort the import).
    pub bundle_losses: Vec<LossRecord>,
    /// Requirements the C2PO file declares.
    pub declared: usize,
    /// Imported requirements, propositions rebound to the catalog.
    pub imported: Vec<ImportedRequirement>,
    /// One loss record per requirement not imported.
    pub unimported: Vec<LossRecord>,
}

fn bundle_loss(kind: LossKind, construct: impl Into<String>, span: SourceRange) -> LossRecord {
    LossRecord::at(kind, ForeignFormat::FretBundle, construct, span, None)
}

fn read_variable_map(
    text: &str,
    catalog: &tl_syntax::SignalCatalog<'_>,
    losses: &mut Vec<LossRecord>,
) -> BTreeMap<String, VariableBinding> {
    let mut by_variable: BTreeMap<String, VariableBinding> = BTreeMap::new();
    let mut seen_variables = BTreeSet::new();
    let mut seen_signals = BTreeSet::new();
    let mut line_start = 0usize;
    for line in text.split_inclusive('\n') {
        let offset = line_start;
        line_start += line.len();
        let body = line.trim_end_matches(['\n', '\r']);
        if body.trim().is_empty() {
            continue;
        }
        let span = SourceRange {
            start: offset,
            end: offset + body.len(),
        };
        let Some((variable, signal)) = body.split_once(':') else {
            losses.push(bundle_loss(LossKind::ForeignConstruct, body, span));
            continue;
        };
        let variable = variable.trim().to_owned();
        let Ok(signal) = signal.trim().parse::<u32>() else {
            losses.push(bundle_loss(LossKind::ForeignConstruct, body, span));
            continue;
        };
        let duplicate_variable = !seen_variables.insert(variable.clone());
        let duplicate_signal = !seen_signals.insert(signal);
        if duplicate_variable || duplicate_signal {
            // Neither entry binds: a duplicate never silently picks one.
            by_variable.remove(&variable);
            by_variable.retain(|_, binding| binding.signal != SignalId(signal));
            losses.push(bundle_loss(LossKind::DuplicateVariable, variable, span));
            continue;
        }
        let Some(declaration) = catalog.signal(SignalId(signal)) else {
            losses.push(bundle_loss(LossKind::UnmappedVariable, variable, span));
            continue;
        };
        if declaration.domain() != SignalDomain::Boolean {
            losses.push(bundle_loss(LossKind::UnsupportedSignalType, variable, span));
            continue;
        }
        let proposition = catalog
            .bindings()
            .iter()
            .find(|binding| binding.signal() == SignalId(signal))
            .map(|binding| binding.proposition());
        let Some(proposition) = proposition else {
            losses.push(bundle_loss(LossKind::UnmappedVariable, variable, span));
            continue;
        };
        by_variable.insert(
            variable.clone(),
            VariableBinding {
                variable,
                signal: SignalId(signal),
                proposition,
            },
        );
    }
    by_variable
}

fn read_header(text: &str, losses: &mut Vec<LossRecord>) -> TraceHeader {
    let mut header = TraceHeader::default();
    let mut line_start = 0usize;
    for line in text.split_inclusive('\n') {
        let offset = line_start;
        line_start += line.len();
        let body = line.trim_end_matches(['\n', '\r']);
        let Some(field) = body.trim_start().strip_prefix('#') else {
            continue;
        };
        let span = SourceRange {
            start: offset,
            end: offset + body.len(),
        };
        let field = field.trim();
        match field.split_once(':') {
            None => {
                header.signals = Some(field.split(',').map(|name| name.trim().to_owned()).collect());
            }
            Some((key, value)) => match key.trim() {
                "signals" => {
                    header.signals =
                        Some(value.split(',').map(|name| name.trim().to_owned()).collect());
                }
                "mission-time" => match value.trim().parse::<u32>() {
                    Ok(time) => header.mission_time = Some(time),
                    Err(_) => losses.push(bundle_loss(LossKind::ForeignConstruct, field, span)),
                },
                other => losses.push(bundle_loss(LossKind::UnknownHeaderField, other, span)),
            },
        }
    }
    header
}

/// Imports a FRET export bundle: the C2PO file through the per-requirement
/// entry point, each variable bound to exactly one catalog signal, header
/// fields carried. Unbound variables make their requirements unimported.
///
/// Implements: FR-088
#[allow(clippy::too_many_arguments)]
pub fn import_fret_bundle(
    c2po: &[u8],
    variable_map: &[u8],
    trace_header: &[u8],
    catalog: &SignalCatalogDocument,
    profile: SemanticProfile,
    attribution: &Attribution,
    limits: ReaderLimits,
) -> Result<FretBundleImport, FretImportError> {
    attribution
        .validate_fret()
        .map_err(FretImportError::Attribution)?;
    let requirements =
        import_c2po_requirements(c2po, Some(profile), limits).map_err(FretImportError::C2po)?;
    let map_text = admit_source(variable_map, limits)?;
    let header_text = core::str::from_utf8(trace_header)
        .map_err(|error| ReaderError::NonUtf8 {
            offset: error.valid_up_to(),
        })?;
    let validated = catalog
        .validate()
        .expect("a constructed catalog document validates");
    let mut bundle_losses = Vec::new();
    let map = read_variable_map(map_text, &validated, &mut bundle_losses);
    let header = read_header(header_text, &mut bundle_losses);
    for signal in &requirements.signals {
        let already = bundle_losses
            .iter()
            .any(|loss| &loss.construct == signal);
        if !map.contains_key(signal) && !already {
            bundle_losses.push(LossRecord::without_span(
                LossKind::UnmappedVariable,
                ForeignFormat::FretBundle,
                signal.clone(),
                "the C2PO INPUT signal has no variable-map entry",
                None,
            ));
        }
    }
    let mut imported = Vec::new();
    let mut unimported = requirements.unimported;
    for mut requirement in requirements.imported {
        let mut missing = None;
        for node in &mut requirement.nodes {
            if let NodeKind::Proposition { proposition } = &mut node.kind {
                let name = &requirements.signals[proposition.0 as usize];
                match map.get(name) {
                    Some(binding) => *proposition = binding.proposition,
                    None => {
                        missing = Some(name.clone());
                        break;
                    }
                }
            }
        }
        match missing {
            Some(name) => unimported.push(LossRecord::at(
                LossKind::UnmappedVariable,
                ForeignFormat::FretBundle,
                name,
                requirement.span,
                Some(requirement.index),
            )),
            None => imported.push(requirement),
        }
    }
    unimported.sort_by_key(|loss| loss.requirement);
    let bindings = requirements
        .signals
        .iter()
        .filter_map(|signal| map.get(signal).cloned())
        .collect();
    Ok(FretBundleImport {
        profile,
        attribution: attribution.clone(),
        bindings,
        header,
        bundle_losses,
        declared: requirements.declared,
        imported,
        unimported,
    })
}
