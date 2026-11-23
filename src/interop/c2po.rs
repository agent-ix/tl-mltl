//! C2PO import (FR-086) and the export side of its identity check.
//!
//! The supported subset is what the FR-038/FR-041 mapped cells render:
//! `INPUT` declarations of `bool` signals, `FTSPEC` and `PTSPEC` sections,
//! `! && || -> <->`, `true`/`false`, bounded `F G U R` and past `O H Y S T`
//! with closed `[a,b]` intervals. The profile is selected by the caller and
//! never inferred from the section name.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! SPEC-GAP (FR-086 asks the build ticket to reconcile the 10-construct list
//! with the C2PO 4.1.0 grammar): no C2PO grammar is retained in this
//! repository and nothing here was checked against one; the keyword spellings
//! `CONTRACT` and `ORDER` are UNGROUNDED guesses.

use core::fmt;
use std::collections::BTreeMap;

use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

use super::loss::{ForeignFormat, LossKind, LossRecord};
use super::{admit_source, ReaderError, ReaderLimits, SourceRange};
use crate::mapping::legacy::render_contextual_expression;

/// The ten constructs outside the supported subset.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnsupportedConstruct {
    /// `STRUCT` declaration (file-level).
    StructDeclaration,
    /// `DEFINE` macro (file-level).
    DefineMacro,
    /// `+ - * / %` (expression-level).
    ArithmeticOperator,
    /// `< <= > >= == !=` (expression-level).
    RelationalOperator,
    /// `name(...)` other than a temporal operator (expression-level).
    FunctionCall,
    /// `foreach`, `forsome`, `forexactly`, `foratleast`, `foratmost`, `{ }`
    /// (expression-level).
    SetAggregation,
    /// `INPUT` of a type other than `bool` (file-level).
    NonBooleanInputType,
    /// `CONTRACT` section (file-level).
    ContractSection,
    /// `ORDER` section (file-level).
    OrderSection,
    /// A token the reader does not recognize (file-level outside a requirement,
    /// expression-level inside one).
    UnrecognizedToken,
}

impl UnsupportedConstruct {
    /// Every construct, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::StructDeclaration,
        Self::DefineMacro,
        Self::ArithmeticOperator,
        Self::RelationalOperator,
        Self::FunctionCall,
        Self::SetAggregation,
        Self::NonBooleanInputType,
        Self::ContractSection,
        Self::OrderSection,
        Self::UnrecognizedToken,
    ];

    /// Stable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::StructDeclaration => "struct-declaration",
            Self::DefineMacro => "define-macro",
            Self::ArithmeticOperator => "arithmetic-operator",
            Self::RelationalOperator => "relational-operator",
            Self::FunctionCall => "function-call",
            Self::SetAggregation => "set-aggregation",
            Self::NonBooleanInputType => "non-boolean-input-type",
            Self::ContractSection => "contract-section",
            Self::OrderSection => "order-section",
            Self::UnrecognizedToken => "unrecognized-token",
        }
    }

    /// Loss kind in the per-requirement entry point (FR-086 refusal model).
    pub const fn loss_kind(self) -> LossKind {
        match self {
            Self::ArithmeticOperator
            | Self::RelationalOperator
            | Self::FunctionCall
            | Self::SetAggregation => LossKind::UnsupportedOperator,
            Self::StructDeclaration
            | Self::DefineMacro
            | Self::NonBooleanInputType
            | Self::ContractSection
            | Self::OrderSection
            | Self::UnrecognizedToken => LossKind::ForeignConstruct,
        }
    }
}

/// Typed C2PO import refusal; never a partial graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum C2poImportError {
    /// Hostile-input refusal.
    Reader(ReaderError),
    /// A construct outside the subset.
    UnsupportedConstruct {
        /// The construct.
        construct: UnsupportedConstruct,
        /// Where.
        span: SourceRange,
    },
    /// No profile was selected.
    ProfileAbsent,
    /// The selected profile contradicts a section kind.
    ProfileMismatch {
        /// Section keyword.
        section: &'static str,
        /// Selected profile.
        selected: &'static str,
    },
    /// An identifier names no `INPUT` signal.
    UndeclaredSignal {
        /// Name.
        name: String,
        /// Where.
        span: SourceRange,
    },
    /// A malformed or out-of-range interval.
    InvalidInterval {
        /// Where.
        span: SourceRange,
    },
    /// The graph is not admitted under the selected profile.
    OperatorOutsideProfile {
        /// Requirement index.
        requirement: usize,
        /// tl-syntax refusal text.
        detail: String,
    },
}

impl From<ReaderError> for C2poImportError {
    fn from(error: ReaderError) -> Self {
        Self::Reader(error)
    }
}

impl fmt::Display for C2poImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(error) => error.fmt(formatter),
            Self::UnsupportedConstruct { construct, span } => write!(
                formatter,
                "unsupported C2PO construct {} at byte {}",
                construct.name(),
                span.start
            ),
            Self::ProfileAbsent => write!(formatter, "no semantic profile selected"),
            Self::ProfileMismatch { section, selected } => {
                write!(formatter, "section {section} contradicts selected profile {selected}")
            }
            Self::UndeclaredSignal { name, span } => {
                write!(formatter, "undeclared signal {name:?} at byte {}", span.start)
            }
            Self::InvalidInterval { span } => {
                write!(formatter, "invalid interval at byte {}", span.start)
            }
            Self::OperatorOutsideProfile {
                requirement,
                detail,
            } => write!(formatter, "requirement {requirement} outside profile: {detail}"),
        }
    }
}

impl std::error::Error for C2poImportError {}

/// One imported requirement: a node arena over `PropositionId(i)` = signal `i`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedRequirement {
    /// Zero-based index among the file's requirements.
    pub index: usize,
    /// Source span of the requirement (without its `;`).
    pub span: SourceRange,
    /// Topologically ordered nodes.
    pub nodes: Vec<Node>,
    /// Root.
    pub root: NodeId,
}

impl ImportedRequirement {
    /// The validated formula view under `profile`.
    pub fn formula(&self, profile: SemanticProfile) -> Formula<'_> {
        Formula::new(profile, self.root, &self.nodes).expect("validated at import")
    }
}

/// Strict import result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct C2poImport {
    /// Selected profile.
    pub profile: SemanticProfile,
    /// `INPUT` signals in declaration order; signal `i` is `PropositionId(i)`.
    pub signals: Vec<String>,
    /// Requirements in file order.
    pub requirements: Vec<ImportedRequirement>,
}

impl C2poImport {
    /// Signal name to proposition map in signal order.
    pub fn proposition_map(&self) -> Vec<(String, PropositionId)> {
        self.signals
            .iter()
            .enumerate()
            .map(|(index, name)| (name.clone(), PropositionId(index as u32)))
            .collect()
    }
}

/// Per-requirement import result (FR-086 refusal model, FR-091 conservation).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct C2poRequirementImport {
    /// Selected profile.
    pub profile: SemanticProfile,
    /// `INPUT` signals in declaration order.
    pub signals: Vec<String>,
    /// Number of `;`-terminated requirements the file declares.
    pub declared: usize,
    /// Imported requirements.
    pub imported: Vec<ImportedRequirement>,
    /// One loss record per requirement not imported.
    pub unimported: Vec<LossRecord>,
}

// ------------------------------------------------------------------- lexer

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Tok {
    Ident(String),
    Number(u64),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    Bang,
    AndAnd,
    OrOr,
    Arrow,
    Iff,
    Arith,
    Rel,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) tok: Tok,
    pub(crate) span: SourceRange,
}

fn lex(text: &str, limits: ReaderLimits) -> Result<Vec<Token>, ReaderError> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let tok = if c.is_ascii_alphabetic() || c == b'_' {
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            Tok::Ident(text[start..i].to_owned())
        } else if c.is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            match text[start..i].parse::<u64>() {
                Ok(value) => Tok::Number(value),
                Err(_) => Tok::Unknown,
            }
        } else {
            let next = bytes.get(i + 1).copied();
            let third = bytes.get(i + 2).copied();
            let (tok, width) = match (c, next, third) {
                (b'(', _, _) => (Tok::LParen, 1),
                (b')', _, _) => (Tok::RParen, 1),
                (b'[', _, _) => (Tok::LBracket, 1),
                (b']', _, _) => (Tok::RBracket, 1),
                (b'{', _, _) => (Tok::LBrace, 1),
                (b'}', _, _) => (Tok::RBrace, 1),
                (b',', _, _) => (Tok::Comma, 1),
                (b';', _, _) => (Tok::Semi, 1),
                (b':', _, _) => (Tok::Colon, 1),
                (b'!', Some(b'='), _) => (Tok::Rel, 2),
                (b'!', _, _) => (Tok::Bang, 1),
                (b'&', Some(b'&'), _) => (Tok::AndAnd, 2),
                (b'|', Some(b'|'), _) => (Tok::OrOr, 2),
                (b'-', Some(b'>'), _) => (Tok::Arrow, 2),
                (b'<', Some(b'-'), Some(b'>')) => (Tok::Iff, 3),
                (b'<', Some(b'='), _) | (b'>', Some(b'='), _) | (b'=', Some(b'='), _) => {
                    (Tok::Rel, 2)
                }
                (b'<', _, _) | (b'>', _, _) => (Tok::Rel, 1),
                (b'+', _, _) | (b'-', _, _) | (b'*', _, _) | (b'/', _, _) | (b'%', _, _) => {
                    (Tok::Arith, 1)
                }
                _ => {
                    // One whole UTF-8 character.
                    let width = text[i..].chars().next().map_or(1, char::len_utf8);
                    (Tok::Unknown, width)
                }
            };
            i += width;
            tok
        };
        tokens.push(Token {
            tok,
            span: SourceRange { start, end: i },
        });
        if tokens.len() > limits.max_tokens {
            return Err(ReaderError::TooManyTokens {
                limit: limits.max_tokens,
            });
        }
    }
    Ok(tokens)
}

// ----------------------------------------------------------------- scanner

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Section {
    Future,
    Past,
}

impl Section {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Future => "FTSPEC",
            Self::Past => "PTSPEC",
        }
    }

    const fn profile(self) -> SemanticProfile {
        match self {
            Self::Future => SemanticProfile::OnlinePrefixV1,
            Self::Past => SemanticProfile::OriginCompleteHistoryV1,
        }
    }
}

struct RawRequirement {
    first: usize,
    end: usize,
    span: SourceRange,
}

struct Scanned {
    tokens: Vec<Token>,
    signals: Vec<String>,
    requirements: Vec<RawRequirement>,
}

fn keyword(token: &Token) -> Option<&str> {
    match &token.tok {
        Tok::Ident(name) => match name.as_str() {
            "INPUT" | "FTSPEC" | "PTSPEC" | "STRUCT" | "DEFINE" | "CONTRACT" | "ORDER" => {
                Some(name.as_str())
            }
            _ => None,
        },
        _ => None,
    }
}

fn unsupported(construct: UnsupportedConstruct, span: SourceRange) -> C2poImportError {
    C2poImportError::UnsupportedConstruct { construct, span }
}

/// File-level scan: sections, `INPUT` declarations and requirement boundaries.
fn scan(
    bytes: &[u8],
    profile: Option<SemanticProfile>,
    limits: ReaderLimits,
) -> Result<(Scanned, SemanticProfile), C2poImportError> {
    let text = admit_source(bytes, limits)?;
    let profile = profile.ok_or(C2poImportError::ProfileAbsent)?;
    let tokens = lex(text, limits)?;
    let mut signals: Vec<String> = Vec::new();
    let mut requirements = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        match keyword(&tokens[i]) {
            Some("STRUCT") => {
                return Err(unsupported(UnsupportedConstruct::StructDeclaration, tokens[i].span))
            }
            Some("DEFINE") => {
                return Err(unsupported(UnsupportedConstruct::DefineMacro, tokens[i].span))
            }
            Some("CONTRACT") => {
                return Err(unsupported(UnsupportedConstruct::ContractSection, tokens[i].span))
            }
            Some("ORDER") => {
                return Err(unsupported(UnsupportedConstruct::OrderSection, tokens[i].span))
            }
            Some("INPUT") => {
                i += 1;
                i = scan_inputs(&tokens, i, &mut signals)?;
            }
            Some(section_keyword) => {
                let section = if section_keyword == "FTSPEC" {
                    Section::Future
                } else {
                    Section::Past
                };
                if section.profile() != profile {
                    return Err(C2poImportError::ProfileMismatch {
                        section: section.keyword(),
                        selected: profile.as_str(),
                    });
                }
                i += 1;
                while i < tokens.len() && keyword(&tokens[i]).is_none() {
                    let first = i;
                    while i < tokens.len() && tokens[i].tok != Tok::Semi {
                        if keyword(&tokens[i]).is_some() {
                            // A section keyword before `;`: boundaries are lost.
                            return Err(unsupported(
                                UnsupportedConstruct::UnrecognizedToken,
                                tokens[i].span,
                            ));
                        }
                        i += 1;
                    }
                    if i == tokens.len() {
                        return Err(unsupported(
                            UnsupportedConstruct::UnrecognizedToken,
                            tokens[first].span,
                        ));
                    }
                    if first == i {
                        return Err(unsupported(
                            UnsupportedConstruct::UnrecognizedToken,
                            tokens[i].span,
                        ));
                    }
                    requirements.push(RawRequirement {
                        first,
                        end: i,
                        span: SourceRange {
                            start: tokens[first].span.start,
                            end: tokens[i - 1].span.end,
                        },
                    });
                    i += 1; // `;`
                }
            }
            None => {
                return Err(unsupported(
                    UnsupportedConstruct::UnrecognizedToken,
                    tokens[i].span,
                ))
            }
        }
    }
    if requirements.is_empty() {
        return Err(ReaderError::Empty.into());
    }
    Ok((
        Scanned {
            tokens,
            signals,
            requirements,
        },
        profile,
    ))
}

/// `a, b, c: bool;` lines until the next section keyword.
fn scan_inputs(
    tokens: &[Token],
    mut i: usize,
    signals: &mut Vec<String>,
) -> Result<usize, C2poImportError> {
    while i < tokens.len() && keyword(&tokens[i]).is_none() {
        let mut names = Vec::new();
        loop {
            match tokens.get(i).map(|token| &token.tok) {
                Some(Tok::Ident(name)) => names.push((name.clone(), tokens[i].span)),
                _ => {
                    let span = tokens.get(i).map_or(tokens[i - 1].span, |t| t.span);
                    return Err(unsupported(UnsupportedConstruct::UnrecognizedToken, span));
                }
            }
            i += 1;
            match tokens.get(i).map(|token| &token.tok) {
                Some(Tok::Comma) => i += 1,
                Some(Tok::Colon) => {
                    i += 1;
                    break;
                }
                _ => {
                    let span = tokens.get(i).map_or(tokens[i - 1].span, |t| t.span);
                    return Err(unsupported(UnsupportedConstruct::UnrecognizedToken, span));
                }
            }
        }
        match tokens.get(i).map(|token| &token.tok) {
            Some(Tok::Ident(kind)) if kind == "bool" => {}
            Some(Tok::Ident(_)) => {
                return Err(unsupported(
                    UnsupportedConstruct::NonBooleanInputType,
                    tokens[i].span,
                ))
            }
            _ => {
                let span = tokens.get(i).map_or(tokens[i - 1].span, |t| t.span);
                return Err(unsupported(UnsupportedConstruct::UnrecognizedToken, span));
            }
        }
        i += 1;
        if tokens.get(i).map(|token| &token.tok) != Some(&Tok::Semi) {
            let span = tokens.get(i).map_or(tokens[i - 1].span, |t| t.span);
            return Err(unsupported(UnsupportedConstruct::UnrecognizedToken, span));
        }
        i += 1;
        for (name, span) in names {
            if signals.contains(&name) {
                return Err(ReaderError::DuplicateSignal { name, span }.into());
            }
            signals.push(name);
        }
    }
    Ok(i)
}

// ------------------------------------------------------------------ parser

/// Why one requirement could not be imported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Issue {
    Construct(UnsupportedConstruct, SourceRange),
    Undeclared(String, SourceRange),
    Interval(SourceRange),
    Nesting(SourceRange),
}

const TEMPORAL_KEYWORDS: [&str; 11] = ["F", "G", "O", "H", "Y", "U", "R", "S", "T", "true", "false"];
const AGGREGATION_KEYWORDS: [&str; 5] = ["foreach", "forsome", "forexactly", "foratleast", "foratmost"];

/// The first expression-level construct in token order, if any.
fn prescan(tokens: &[Token]) -> Option<Issue> {
    for (index, token) in tokens.iter().enumerate() {
        let construct = match &token.tok {
            Tok::Arith => Some(UnsupportedConstruct::ArithmeticOperator),
            Tok::Rel => Some(UnsupportedConstruct::RelationalOperator),
            Tok::LBrace | Tok::RBrace => Some(UnsupportedConstruct::SetAggregation),
            Tok::Unknown | Tok::Colon => Some(UnsupportedConstruct::UnrecognizedToken),
            Tok::Ident(name) if AGGREGATION_KEYWORDS.contains(&name.as_str()) => {
                Some(UnsupportedConstruct::SetAggregation)
            }
            Tok::Ident(name)
                if !TEMPORAL_KEYWORDS.contains(&name.as_str())
                    && tokens.get(index + 1).map(|t| &t.tok) == Some(&Tok::LParen) =>
            {
                Some(UnsupportedConstruct::FunctionCall)
            }
            _ => None,
        };
        if let Some(construct) = construct {
            return Some(Issue::Construct(construct, token.span));
        }
    }
    None
}

pub(crate) struct Parser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) position: usize,
    pub(crate) nodes: Vec<Node>,
    pub(crate) names: &'a BTreeMap<String, u32>,
    pub(crate) max_nesting: u32,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.position).map(|token| &token.tok)
    }

    fn span(&self) -> SourceRange {
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .map_or(SourceRange { start: 0, end: 0 }, |token| token.span)
    }

    fn push(&mut self, kind: NodeKind) -> NodeId {
        self.nodes.push(Node::new(kind));
        NodeId((self.nodes.len() - 1) as u32)
    }

    fn unrecognized(&self) -> Issue {
        Issue::Construct(UnsupportedConstruct::UnrecognizedToken, self.span())
    }

    fn enter(&self, depth: u32) -> Result<u32, Issue> {
        if depth >= self.max_nesting {
            return Err(Issue::Nesting(self.span()));
        }
        Ok(depth + 1)
    }

    pub(crate) fn parse_all(&mut self) -> Result<NodeId, Issue> {
        let root = self.equivalence(0)?;
        if self.position != self.tokens.len() {
            return Err(self.unrecognized());
        }
        Ok(root)
    }

    fn equivalence(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        let mut left = self.implication(depth)?;
        while self.peek() == Some(&Tok::Iff) {
            self.position += 1;
            let right = self.implication(depth)?;
            left = self.push(NodeKind::Equivalent { left, right });
        }
        Ok(left)
    }

    fn implication(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        let left = self.disjunction(depth)?;
        if self.peek() == Some(&Tok::Arrow) {
            self.position += 1;
            let right = self.implication(depth)?;
            return Ok(self.push(NodeKind::Implies { left, right }));
        }
        Ok(left)
    }

    fn disjunction(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        let mut left = self.conjunction(depth)?;
        while self.peek() == Some(&Tok::OrOr) {
            self.position += 1;
            let right = self.conjunction(depth)?;
            left = self.push(NodeKind::Or { left, right });
        }
        Ok(left)
    }

    fn conjunction(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        let mut left = self.binary_temporal(depth)?;
        while self.peek() == Some(&Tok::AndAnd) {
            self.position += 1;
            let right = self.binary_temporal(depth)?;
            left = self.push(NodeKind::And { left, right });
        }
        Ok(left)
    }

    fn binary_temporal(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        let mut left = self.unary(depth)?;
        loop {
            let operator = match self.peek() {
                Some(Tok::Ident(name)) if matches!(name.as_str(), "U" | "R" | "S" | "T") => {
                    name.clone()
                }
                _ => break,
            };
            self.position += 1;
            let interval = self.interval()?;
            let right = self.unary(depth)?;
            let kind = match operator.as_str() {
                "U" => NodeKind::Until {
                    interval,
                    left,
                    right,
                },
                "R" => NodeKind::Release {
                    interval,
                    left,
                    right,
                },
                "S" => NodeKind::Since {
                    interval,
                    left,
                    right,
                },
                _ => NodeKind::Triggered {
                    interval,
                    left,
                    right,
                },
            };
            left = self.push(kind);
        }
        Ok(left)
    }

    fn unary(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let depth = self.enter(depth)?;
        match self.peek().cloned() {
            Some(Tok::Bang) => {
                self.position += 1;
                let operand = self.unary(depth)?;
                Ok(self.push(NodeKind::Not { operand }))
            }
            Some(Tok::Ident(name)) if matches!(name.as_str(), "F" | "G" | "O" | "H") => {
                self.position += 1;
                let interval = self.interval()?;
                let operand = self.unary(depth)?;
                Ok(self.push(match name.as_str() {
                    "F" => NodeKind::Future { interval, operand },
                    "G" => NodeKind::Globally { interval, operand },
                    "O" => NodeKind::Once { interval, operand },
                    _ => NodeKind::Historically { interval, operand },
                }))
            }
            Some(Tok::Ident(name)) if name == "Y" => {
                self.position += 1;
                let operand = self.unary(depth)?;
                Ok(self.push(NodeKind::StrongPrevious { operand }))
            }
            _ => self.primary(depth),
        }
    }

    fn primary(&mut self, depth: u32) -> Result<NodeId, Issue> {
        let span = self.span();
        match self.peek().cloned() {
            Some(Tok::LParen) => {
                self.position += 1;
                let inner = self.equivalence(depth)?;
                if self.peek() != Some(&Tok::RParen) {
                    return Err(self.unrecognized());
                }
                self.position += 1;
                Ok(inner)
            }
            Some(Tok::Ident(name)) => {
                self.position += 1;
                match name.as_str() {
                    "true" => Ok(self.push(NodeKind::True)),
                    "false" => Ok(self.push(NodeKind::False)),
                    _ => match self.names.get(&name) {
                        Some(index) => Ok(self.push(NodeKind::Proposition {
                            proposition: PropositionId(*index),
                        })),
                        None => Err(Issue::Undeclared(name, span)),
                    },
                }
            }
            _ => Err(self.unrecognized()),
        }
    }

    fn interval(&mut self) -> Result<Interval, Issue> {
        let start_span = self.span();
        let expect = |parser: &mut Self, tok: Tok| -> Result<(), Issue> {
            if parser.peek() == Some(&tok) {
                parser.position += 1;
                Ok(())
            } else {
                Err(Issue::Interval(start_span))
            }
        };
        expect(self, Tok::LBracket)?;
        let start = self.number().ok_or(Issue::Interval(start_span))?;
        expect(self, Tok::Comma)?;
        let end = self.number().ok_or(Issue::Interval(start_span))?;
        expect(self, Tok::RBracket)?;
        let start = u32::try_from(start).map_err(|_| Issue::Interval(start_span))?;
        let end = u32::try_from(end).map_err(|_| Issue::Interval(start_span))?;
        Interval::new(start, end).map_err(|_| Issue::Interval(start_span))
    }

    fn number(&mut self) -> Option<u64> {
        match self.peek() {
            Some(Tok::Number(value)) => {
                let value = *value;
                self.position += 1;
                Some(value)
            }
            _ => None,
        }
    }
}

/// Parses one requirement's tokens into a validated graph.
fn parse_requirement(
    scanned: &Scanned,
    names: &BTreeMap<String, u32>,
    requirement: &RawRequirement,
    profile: SemanticProfile,
    limits: ReaderLimits,
) -> Result<(Vec<Node>, NodeId), Result<Issue, String>> {
    let tokens = &scanned.tokens[requirement.first..requirement.end];
    if let Some(issue) = prescan(tokens) {
        return Err(Ok(issue));
    }
    let mut parser = Parser {
        tokens,
        position: 0,
        nodes: Vec::new(),
        names,
        max_nesting: limits.max_nesting,
    };
    let root = parser.parse_all().map_err(Ok)?;
    Formula::new(profile, root, &parser.nodes).map_err(|error| Err(format!("{error:?}")))?;
    Ok((parser.nodes, root))
}

fn names(signals: &[String]) -> BTreeMap<String, u32> {
    signals
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index as u32))
        .collect()
}

fn issue_error(issue: Issue, limits: ReaderLimits) -> C2poImportError {
    match issue {
        Issue::Construct(construct, span) => unsupported(construct, span),
        Issue::Undeclared(name, span) => C2poImportError::UndeclaredSignal { name, span },
        Issue::Interval(span) => C2poImportError::InvalidInterval { span },
        Issue::Nesting(span) => ReaderError::NestingTooDeep {
            limit: limits.max_nesting,
            span,
        }
        .into(),
    }
}

/// Strict import: any construct outside the subset aborts the whole file.
/// Used for export-then-import identity.
///
/// Implements: FR-086
pub fn import_c2po(
    bytes: &[u8],
    profile: Option<SemanticProfile>,
    limits: ReaderLimits,
) -> Result<C2poImport, C2poImportError> {
    let (scanned, profile) = scan(bytes, profile, limits)?;
    let names = names(&scanned.signals);
    let mut requirements = Vec::with_capacity(scanned.requirements.len());
    for (index, raw) in scanned.requirements.iter().enumerate() {
        let (nodes, root) = parse_requirement(&scanned, &names, raw, profile, limits).map_err(
            |failure| match failure {
                Ok(issue) => issue_error(issue, limits),
                Err(detail) => C2poImportError::OperatorOutsideProfile {
                    requirement: index,
                    detail,
                },
            },
        )?;
        requirements.push(ImportedRequirement {
            index,
            span: raw.span,
            nodes,
            root,
        });
    }
    Ok(C2poImport {
        profile,
        signals: scanned.signals,
        requirements,
    })
}

/// Per-requirement import: file-level constructs abort; an expression-level
/// construct, undeclared signal, bad interval or nesting overflow in one
/// requirement yields one loss record for it and the others are imported.
///
/// Implements: FR-086 (FR-086-AC-4), FR-091 conservation
pub fn import_c2po_requirements(
    bytes: &[u8],
    profile: Option<SemanticProfile>,
    limits: ReaderLimits,
) -> Result<C2poRequirementImport, C2poImportError> {
    let (scanned, profile) = scan(bytes, profile, limits)?;
    let names = names(&scanned.signals);
    let mut imported = Vec::new();
    let mut unimported = Vec::new();
    for (index, raw) in scanned.requirements.iter().enumerate() {
        match parse_requirement(&scanned, &names, raw, profile, limits) {
            Ok((nodes, root)) => imported.push(ImportedRequirement {
                index,
                span: raw.span,
                nodes,
                root,
            }),
            Err(Ok(issue)) => unimported.push(issue_loss(issue, index)),
            Err(Err(detail)) => unimported.push(LossRecord::at(
                LossKind::UnsupportedOperator,
                ForeignFormat::C2po,
                detail,
                raw.span,
                Some(index),
            )),
        }
    }
    Ok(C2poRequirementImport {
        profile,
        signals: scanned.signals,
        declared: scanned.requirements.len(),
        imported,
        unimported,
    })
}

fn issue_loss(issue: Issue, index: usize) -> LossRecord {
    let (kind, construct, span) = match issue {
        Issue::Construct(construct, span) => (construct.loss_kind(), construct.name().to_owned(), span),
        Issue::Undeclared(name, span) => (LossKind::UnmappedVariable, name, span),
        Issue::Interval(span) => (LossKind::UnsupportedInterval, "interval".to_owned(), span),
        Issue::Nesting(span) => (LossKind::ResourceLimit, "nesting".to_owned(), span),
    };
    LossRecord::at(kind, ForeignFormat::C2po, construct, span, Some(index))
}

// ------------------------------------------------------------------ export

/// Export refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum C2poExportError {
    /// The future-time exporter refused (FR-041 partition).
    Mapping(crate::MappingError),
    /// The signal catalog for rendering could not be built.
    Catalog(String),
}

fn catalog(signals: &[String]) -> Result<SignalCatalogDocument, C2poExportError> {
    SignalCatalogDocument::new(
        signals
            .iter()
            .enumerate()
            .map(|(index, name)| {
                OwnedSignalDeclaration::new(SignalId(index as u32), name.clone(), SignalDomain::Boolean)
            })
            .collect(),
        (0..signals.len())
            .map(|index| PropositionBinding::new(PropositionId(index as u32), SignalId(index as u32)))
            .collect(),
    )
    .map_err(|error| C2poExportError::Catalog(format!("{error:?}")))
}

/// Renders a past-time graph in the C2PO surface the importer reads.
fn render_past(formula: Formula<'_>, node: NodeId, signals: &[String]) -> String {
    let kind = formula.nodes()[node.0 as usize].kind;
    let r = |id: NodeId| render_past(formula, id, signals);
    let iv = |interval: Interval| format!("[{},{}]", interval.start(), interval.end());
    match kind {
        NodeKind::False => "false".to_owned(),
        NodeKind::True => "true".to_owned(),
        NodeKind::Proposition { proposition } => signals[proposition.0 as usize].clone(),
        NodeKind::Not { operand } => format!("(!{})", r(operand)),
        NodeKind::And { left, right } => format!("({} && {})", r(left), r(right)),
        NodeKind::Or { left, right } => format!("({} || {})", r(left), r(right)),
        NodeKind::Implies { left, right } => format!("({} -> {})", r(left), r(right)),
        NodeKind::Equivalent { left, right } => format!("({} <-> {})", r(left), r(right)),
        NodeKind::Once { interval, operand } => format!("O{}({})", iv(interval), r(operand)),
        NodeKind::Historically { interval, operand } => {
            format!("H{}({})", iv(interval), r(operand))
        }
        NodeKind::StrongPrevious { operand } => format!("Y({})", r(operand)),
        NodeKind::Since {
            interval,
            left,
            right,
        } => format!("({} S{} {})", r(left), iv(interval), r(right)),
        NodeKind::Triggered {
            interval,
            left,
            right,
        } => format!("({} T{} {})", r(left), iv(interval), r(right)),
        // The origin-complete profile admits no future or LTLf node.
        NodeKind::Future { .. }
        | NodeKind::Globally { .. }
        | NodeKind::Until { .. }
        | NodeKind::Release { .. }
        | NodeKind::LtlfFuture { .. }
        | NodeKind::LtlfGlobally { .. }
        | NodeKind::LtlfUntil { .. }
        | NodeKind::LtlfRelease { .. }
        | NodeKind::StrongNext { .. }
        | NodeKind::WeakNext { .. } => unreachable!("origin-complete graphs are past-only"),
    }
}

/// Exports an import back to C2PO text: future requirements through the
/// production FR-004 renderer, past requirements through [`render_past`].
pub fn export_c2po(import: &C2poImport, work_limit: u64) -> Result<String, C2poExportError> {
    let catalog = catalog(&import.signals)?;
    let mut text = String::from("INPUT\n    ");
    text.push_str(&import.signals.join(","));
    text.push_str(": bool;\n\n");
    let past = import.profile == SemanticProfile::OriginCompleteHistoryV1;
    text.push_str(if past { "PTSPEC\n" } else { "FTSPEC\n" });
    for requirement in &import.requirements {
        let formula = requirement.formula(import.profile);
        let expression = if past {
            render_past(formula, formula.root(), &import.signals)
        } else {
            render_contextual_expression(formula, &catalog, work_limit)
                .map_err(C2poExportError::Mapping)?
        };
        text.push_str("    ");
        text.push_str(&expression);
        text.push_str(";\n");
    }
    Ok(text)
}
