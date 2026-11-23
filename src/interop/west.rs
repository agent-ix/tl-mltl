//! WEST formula import and export (FR-092).
//!
//! Grammar, as observed in the retained corpus (`corpus/west-v1`): atoms
//! `p<N>` bound to `PropositionId(N)`; `T` true; `!` false; `~` not; `&` and;
//! `v` or; infix `U` and `R` with `[a:b]`; prefix `G[a:b]` and `F[a:b]`;
//! parentheses around every binary form. Graphs are under
//! `mltl.closed-trace/v1`. Export refuses what WEST cannot spell, with a loss
//! record and no text; it never rewrites.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

use super::loss::{ForeignFormat, LossKind, LossRecord};
use super::{admit_source, ReaderError, ReaderLimits, SourceRange};

/// WEST read refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WestError {
    /// Hostile input.
    Reader(ReaderError),
    /// Not WEST syntax at this byte.
    Syntax {
        /// Byte offset.
        offset: usize,
    },
    /// Malformed or out-of-range interval.
    InvalidInterval {
        /// Byte offset.
        offset: usize,
    },
    /// `G`, `F`, `U` or `R` without an interval: no implicit closure (FR-094).
    MissionClosureRequired {
        /// Byte offset.
        offset: usize,
    },
}

impl From<ReaderError> for WestError {
    fn from(error: ReaderError) -> Self {
        Self::Reader(error)
    }
}

impl fmt::Display for WestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(error) => error.fmt(formatter),
            Self::Syntax { offset } => write!(formatter, "not WEST syntax at byte {offset}"),
            Self::InvalidInterval { offset } => {
                write!(formatter, "invalid interval at byte {offset}")
            }
            Self::MissionClosureRequired { offset } => {
                write!(formatter, "unbounded operator at byte {offset}")
            }
        }
    }
}

impl std::error::Error for WestError {}

impl WestError {
    /// The loss-record view of an import refusal (FR-091).
    pub fn loss_record(&self, requirement: Option<usize>) -> LossRecord {
        let (kind, construct, offset) = match self {
            Self::Reader(error) => {
                return LossRecord::without_span(
                    LossKind::ResourceLimit,
                    ForeignFormat::West,
                    "source",
                    error.to_string(),
                    requirement,
                )
            }
            Self::Syntax { offset } => (LossKind::ForeignConstruct, "syntax", *offset),
            Self::InvalidInterval { offset } => {
                (LossKind::UnsupportedInterval, "interval", *offset)
            }
            Self::MissionClosureRequired { offset } => {
                (LossKind::MissionClosureRequired, "unbounded-operator", *offset)
            }
        };
        LossRecord::at(
            kind,
            ForeignFormat::West,
            construct,
            SourceRange {
                start: offset,
                end: offset + 1,
            },
            requirement,
        )
    }
}

/// One WEST formula as a closed-trace node arena.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WestFormula {
    /// Nodes.
    pub nodes: Vec<Node>,
    /// Root.
    pub root: NodeId,
}

impl WestFormula {
    /// The validated `mltl.closed-trace/v1` view.
    pub fn formula(&self) -> Formula<'_> {
        Formula::new(SemanticProfile::ClosedTraceV1, self.root, &self.nodes)
            .expect("validated at import")
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    nodes: Vec<Node>,
    max_nesting: u32,
    max_tokens: usize,
}

impl Reader<'_> {
    fn push(&mut self, kind: NodeKind) -> NodeId {
        self.nodes.push(Node::new(kind));
        NodeId((self.nodes.len() - 1) as u32)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), WestError> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(WestError::Syntax {
                offset: self.position,
            })
        }
    }

    fn enter(&self, depth: u32) -> Result<u32, WestError> {
        if depth >= self.max_nesting {
            return Err(ReaderError::NestingTooDeep {
                limit: self.max_nesting,
                span: SourceRange {
                    start: self.position,
                    end: self.position + 1,
                },
            }
            .into());
        }
        Ok(depth + 1)
    }

    fn number(&mut self) -> Option<u32> {
        let start = self.position;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.position += 1;
        }
        if start == self.position {
            return None;
        }
        // Digits only, so the slice is ASCII.
        core::str::from_utf8(&self.bytes[start..self.position])
            .ok()
            .and_then(|digits| digits.parse().ok())
    }

    fn interval(&mut self, operator_offset: usize) -> Result<Interval, WestError> {
        if self.peek() != Some(b'[') {
            return Err(WestError::MissionClosureRequired {
                offset: operator_offset,
            });
        }
        let start_offset = self.position;
        let invalid = WestError::InvalidInterval {
            offset: start_offset,
        };
        self.position += 1;
        let start = self.number().ok_or_else(|| invalid.clone())?;
        self.expect(b':').map_err(|_| invalid.clone())?;
        let end = self.number().ok_or_else(|| invalid.clone())?;
        self.expect(b']').map_err(|_| invalid.clone())?;
        Interval::new(start, end).map_err(|_| invalid)
    }

    fn expression(&mut self, depth: u32) -> Result<NodeId, WestError> {
        // Each call adds at most one node: the ceiling bounds parse work.
        if self.nodes.len() >= self.max_tokens {
            return Err(ReaderError::TooManyTokens {
                limit: self.max_tokens,
            }
            .into());
        }
        let offset = self.position;
        match self.peek() {
            Some(b'~') => {
                let depth = self.enter(depth)?;
                self.position += 1;
                let operand = self.expression(depth)?;
                Ok(self.push(NodeKind::Not { operand }))
            }
            Some(b'G') | Some(b'F') => {
                let depth = self.enter(depth)?;
                let globally = self.peek() == Some(b'G');
                self.position += 1;
                let interval = self.interval(offset)?;
                let operand = self.expression(depth)?;
                Ok(self.push(if globally {
                    NodeKind::Globally { interval, operand }
                } else {
                    NodeKind::Future { interval, operand }
                }))
            }
            Some(b'(') => {
                let depth = self.enter(depth)?;
                self.position += 1;
                let left = self.expression(depth)?;
                let operator_offset = self.position;
                let kind = match self.peek() {
                    Some(b'&') => {
                        self.position += 1;
                        let right = self.expression(depth)?;
                        NodeKind::And { left, right }
                    }
                    Some(b'v') => {
                        self.position += 1;
                        let right = self.expression(depth)?;
                        NodeKind::Or { left, right }
                    }
                    Some(b'U') | Some(b'R') => {
                        let until = self.peek() == Some(b'U');
                        self.position += 1;
                        let interval = self.interval(operator_offset)?;
                        let right = self.expression(depth)?;
                        if until {
                            NodeKind::Until {
                                interval,
                                left,
                                right,
                            }
                        } else {
                            NodeKind::Release {
                                interval,
                                left,
                                right,
                            }
                        }
                    }
                    _ => {
                        return Err(WestError::Syntax {
                            offset: operator_offset,
                        })
                    }
                };
                self.expect(b')')?;
                Ok(self.push(kind))
            }
            Some(b'T') => {
                self.position += 1;
                Ok(self.push(NodeKind::True))
            }
            Some(b'!') => {
                self.position += 1;
                Ok(self.push(NodeKind::False))
            }
            Some(b'p') => {
                self.position += 1;
                let index = self.number().ok_or(WestError::Syntax { offset })?;
                Ok(self.push(NodeKind::Proposition {
                    proposition: PropositionId(index),
                }))
            }
            _ => Err(WestError::Syntax { offset }),
        }
    }
}

/// Reads one WEST formula line (no line terminator).
///
/// Implements: FR-092
pub fn read_west(line: &[u8], limits: ReaderLimits) -> Result<WestFormula, WestError> {
    let text = admit_source(line, limits)?;
    let bytes = text.as_bytes();
    let mut reader = Reader {
        bytes,
        position: 0,
        nodes: Vec::new(),
        max_nesting: limits.max_nesting,
        max_tokens: limits.max_tokens,
    };
    let root = reader.expression(0)?;
    if reader.position != bytes.len() {
        return Err(WestError::Syntax {
            offset: reader.position,
        });
    }
    Formula::new(SemanticProfile::ClosedTraceV1, root, &reader.nodes)
        .map_err(|_| WestError::Syntax { offset: 0 })?;
    Ok(WestFormula {
        nodes: reader.nodes,
        root,
    })
}

/// Reads every line of a WEST formula file (each line `\n`-terminated).
pub fn read_west_file(bytes: &[u8], limits: ReaderLimits) -> Result<Vec<WestFormula>, WestError> {
    let text = admit_source(bytes, limits)?;
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| read_west(line.as_bytes(), limits))
        .collect()
}

fn refuse(construct: &str, kind: LossKind) -> LossRecord {
    LossRecord::without_span(
        kind,
        ForeignFormat::West,
        construct,
        "graph export: the construct is a graph node, not source text",
        None,
    )
}

/// Depth and visit budget of one write.
struct WriteBudget {
    visits: u64,
    limit: u64,
}

fn write_node(
    formula: Formula<'_>,
    node: NodeId,
    depth: u32,
    budget: &mut WriteBudget,
    out: &mut String,
) -> Result<(), LossRecord> {
    budget.visits = budget.visits.saturating_add(1);
    if depth > crate::MAX_RECURSION_DEPTH || budget.visits > budget.limit {
        return Err(refuse("resource-limit", LossKind::ResourceLimit));
    }
    let depth = depth + 1;
    let iv = |interval: Interval| format!("[{}:{}]", interval.start(), interval.end());
    match formula.nodes()[node.0 as usize].kind {
        NodeKind::False => out.push('!'),
        NodeKind::True => out.push('T'),
        NodeKind::Proposition { proposition } => out.push_str(&format!("p{}", proposition.0)),
        NodeKind::Not { operand } => {
            out.push('~');
            write_node(formula, operand, depth, budget, out)?;
        }
        NodeKind::And { left, right } | NodeKind::Or { left, right } => {
            let symbol = if matches!(formula.nodes()[node.0 as usize].kind, NodeKind::And { .. }) {
                "&"
            } else {
                "v"
            };
            out.push('(');
            write_node(formula, left, depth, budget, out)?;
            out.push_str(symbol);
            write_node(formula, right, depth, budget, out)?;
            out.push(')');
        }
        NodeKind::Future { interval, operand } => {
            out.push('F');
            out.push_str(&iv(interval));
            write_node(formula, operand, depth, budget, out)?;
        }
        NodeKind::Globally { interval, operand } => {
            out.push('G');
            out.push_str(&iv(interval));
            write_node(formula, operand, depth, budget, out)?;
        }
        NodeKind::Until {
            interval,
            left,
            right,
        }
        | NodeKind::Release {
            interval,
            left,
            right,
        } => {
            let until = matches!(formula.nodes()[node.0 as usize].kind, NodeKind::Until { .. });
            let symbol = if until {
                'U'
            } else {
                'R'
            };
            out.push('(');
            write_node(formula, left, depth, budget, out)?;
            out.push(symbol);
            out.push_str(&iv(interval));
            write_node(formula, right, depth, budget, out)?;
            out.push(')');
        }
        NodeKind::Implies { .. } => {
            return Err(refuse("implies", LossKind::UnsupportedOperator));
        }
        NodeKind::Equivalent { .. } => {
            return Err(refuse("equivalent", LossKind::UnsupportedOperator));
        }
        NodeKind::Once { .. }
        | NodeKind::Historically { .. }
        | NodeKind::StrongPrevious { .. }
        | NodeKind::Since { .. }
        | NodeKind::Triggered { .. } => {
            return Err(refuse("past-operator", LossKind::UnsupportedOperator));
        }
        NodeKind::LtlfFuture { .. }
        | NodeKind::LtlfGlobally { .. }
        | NodeKind::LtlfUntil { .. }
        | NodeKind::LtlfRelease { .. } => {
            return Err(refuse("non-closed-interval", LossKind::UnsupportedInterval));
        }
        NodeKind::StrongNext { .. } | NodeKind::WeakNext { .. } => {
            return Err(refuse("next", LossKind::UnsupportedOperator));
        }
    }
    Ok(())
}

/// Writes a closed-trace graph as one WEST line, or refuses with a loss record
/// and no text.
///
/// Implements: FR-092
pub fn write_west(formula: Formula<'_>) -> Result<String, LossRecord> {
    write_west_bounded(formula, 1_000_000)
}

/// [`write_west`] with an explicit node-visit budget. A shared DAG is written
/// as a tree, so the budget bounds the text produced.
pub fn write_west_bounded(formula: Formula<'_>, work_limit: u64) -> Result<String, LossRecord> {
    // SPEC-GAP: FR-092 lists `W`/`M` as an unexportable class, but the tl-syntax
    // graph has no `W` or `M` node (they are derived source forms), so no graph
    // can carry one to this writer.
    let mut out = String::new();
    let mut budget = WriteBudget {
        visits: 0,
        limit: work_limit,
    };
    write_node(formula, formula.root(), 0, &mut budget, &mut out)?;
    // Node refusals come first; a graph of only WEST-spellable nodes under
    // another profile (online-prefix) still means something else in WEST.
    if formula.profile() != SemanticProfile::ClosedTraceV1 {
        return Err(refuse("profile", LossKind::ForeignConstruct));
    }
    Ok(out)
}
