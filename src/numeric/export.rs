//! Numeric C2PO export: the refusal-until-pinned path and the typed loss
//! record (FR-056, FR-057-AC-2), and the contract decomposition export
//! (FR-060) on top of it.
//!
//! Decision 5 (2026-11-23): numeric C2PO/R2U2 export is refused until a
//! numeric target profile is pinned from retained evidence. No profile can be
//! supplied in this build, so every predicate-defined proposition is refused
//! with `TargetNumericProfileUnpinned` and no expression or manifest is
//! produced. Nothing here runs C2PO or R2U2.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tl_syntax::{
    Formula, FormulaDocument, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
    SignalCatalogDocument,
};

use super::model::{Comparison, ExprNode, PredicateTable};
use crate::contract::{bind_contracts, ContractSet};
use crate::mapping::legacy::render_contextual_expression;
use crate::wire::common::read as read_canonical;
use crate::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
use crate::MappingError;

/// Loss record identity.
pub const NUMERIC_LOSS_V1: &str = "tl-mltl.numeric-loss/v1";

/// The closed set of numeric export refusal causes (FR-056).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumericLossCause {
    /// No numeric target profile is pinned (the only cause reachable today).
    TargetNumericProfileUnpinned,
    /// A domain or literal does not fit the target integer width.
    TargetIntegerRangeExceeded,
    /// `R + T >= g/2`: float matching could change a verdict.
    FloatMatchingUnsafe,
    /// A rescale the target cannot represent.
    RescaleBeyondTarget,
    /// An operator the target cannot express.
    UnsupportedNumericOperator,
    /// A proposition neither catalog-bound nor predicate-defined.
    PredicateWithoutDefinition,
    /// An identity that does not belong to the supplied catalog or table.
    ForeignIdentity,
    /// A work or recursion ceiling was reached.
    ResourceExhausted,
}

/// What a loss record is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum NumericLossSubject {
    /// A proposition.
    Proposition(u32),
    /// An expression node index.
    ExpressionNode(u32),
    /// A catalog signal.
    Signal(u32),
    /// The formula or graph as a whole.
    Formula,
    /// One contract of a contract set.
    Contract(u32),
}

/// `tl-mltl.numeric-loss/v1`. A refusal never carries a partial expression.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericLossRecord {
    /// Wire identity.
    pub schema_version: String,
    /// Cause.
    pub cause: NumericLossCause,
    /// Subject.
    pub subject: NumericLossSubject,
    /// Human detail (for `FloatMatchingUnsafe`, the exact rationals).
    pub detail: String,
}

impl NumericLossRecord {
    fn new(
        cause: NumericLossCause,
        subject: NumericLossSubject,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: NUMERIC_LOSS_V1.to_owned(),
            cause,
            subject,
            detail: detail.into(),
        }
    }

    /// Canonical bytes.
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("numeric loss record serializes")
    }

    /// Strict reader through the owner boundary (size, depth, canonical form).
    pub fn from_json_bytes(bytes: &[u8], limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let (record, _) = read_canonical(bytes, limits, |record: &Self, _| {
            if record.schema_version == NUMERIC_LOSS_V1 {
                Ok(OwnerUsage::default())
            } else {
                Err(OwnerReadError::new(
                    OwnerReadErrorCode::InvalidCombination,
                    "schemaVersion",
                    OwnerUsage::default(),
                ))
            }
        })?;
        Ok(record)
    }
}

/// Export class of one numeric node kind (FR-056 partition). With no pinned
/// profile every cell is the same refusal; the match is exhaustive with no
/// wildcard so a new node kind fails to compile until it is classified.
pub const fn numeric_node_class(node: ExprNode) -> NumericLossCause {
    match node {
        ExprNode::Signal { .. }
        | ExprNode::Integer { .. }
        | ExprNode::Decimal { .. }
        | ExprNode::Neg { .. }
        | ExprNode::Abs { .. }
        | ExprNode::Add { .. }
        | ExprNode::Sub { .. }
        | ExprNode::Mul { .. }
        | ExprNode::Rescale { .. } => NumericLossCause::TargetNumericProfileUnpinned,
    }
}

/// Export class of one comparison operator (same rule).
pub const fn comparison_class(comparison: Comparison) -> NumericLossCause {
    match comparison {
        Comparison::Lt
        | Comparison::Le
        | Comparison::Gt
        | Comparison::Ge
        | Comparison::Eq
        | Comparison::Ne => NumericLossCause::TargetNumericProfileUnpinned,
    }
}

/// Result of a numeric-aware export of one formula.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NumericExportOutcome {
    /// The formula reads no predicate: the existing Boolean renderer's
    /// expression, byte-identical to `map_to_c2po_with_context`'s.
    Boolean {
        /// C2PO expression.
        expression: String,
    },
}

fn propositions(formula: Formula<'_>) -> BTreeSet<PropositionId> {
    formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition),
            _ => None,
        })
        .collect()
}

fn first_refusal(
    formula: Formula<'_>,
    table: &PredicateTable,
    catalog: &SignalCatalogDocument,
) -> Option<NumericLossRecord> {
    let validated = catalog
        .validate()
        .expect("a constructed catalog document validates");
    for proposition in propositions(formula) {
        if let Some(predicate) = table
            .predicates
            .iter()
            .find(|predicate| predicate.proposition == proposition)
        {
            return Some(NumericLossRecord::new(
                comparison_class(predicate.comparison),
                NumericLossSubject::Proposition(proposition.0),
                "numeric C2PO export is refused until a target numeric profile is pinned from \
                 retained R2U2 evidence (tl-mltl ADR-014, decision 5)",
            ));
        }
        if validated.signal_for_proposition(proposition).is_none() {
            return Some(NumericLossRecord::new(
                NumericLossCause::PredicateWithoutDefinition,
                NumericLossSubject::Proposition(proposition.0),
                "proposition is neither catalog-bound nor predicate-defined",
            ));
        }
    }
    None
}

fn mapping_loss(error: MappingError) -> NumericLossRecord {
    let cause = match error {
        MappingError::WorkLimitExceeded { .. } | MappingError::RecursionDepthExceeded { .. } => {
            NumericLossCause::ResourceExhausted
        }
        MappingError::Binding(_)
        | MappingError::Identity(_)
        | MappingError::UnsupportedProfile { .. }
        | MappingError::UnsupportedSignalName { .. } => NumericLossCause::ForeignIdentity,
    };
    NumericLossRecord::new(cause, NumericLossSubject::Formula, error.to_string())
}

/// Exports a formula whose propositions may be predicate-defined.
///
/// Every predicate-defined proposition refuses with
/// `TargetNumericProfileUnpinned`; a formula with none is rendered by the
/// existing FR-004 renderer, unchanged.
///
/// Implements: FR-056 (refusal path), FR-057-AC-2
pub fn map_numeric_to_c2po(
    formula: Formula<'_>,
    table: &PredicateTable,
    catalog: &SignalCatalogDocument,
    work_limit: u64,
) -> Result<NumericExportOutcome, NumericLossRecord> {
    if formula.profile() != SemanticProfile::OnlinePrefixV1 {
        return Err(NumericLossRecord::new(
            NumericLossCause::ForeignIdentity,
            NumericLossSubject::Formula,
            format!(
                "C2PO export requires mltl.online-prefix/v1, found {}",
                formula.profile().as_str()
            ),
        ));
    }
    if let Some(record) = first_refusal(formula, table, catalog) {
        return Err(record);
    }
    let expression =
        render_contextual_expression(formula, catalog, work_limit).map_err(mapping_loss)?;
    Ok(NumericExportOutcome::Boolean { expression })
}

/// Operands of one node (explicit over every kind, no wildcard).
fn operands(kind: NodeKind) -> [Option<NodeId>; 2] {
    match kind {
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => [None, None],
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. }
        | NodeKind::Once { operand, .. }
        | NodeKind::Historically { operand, .. }
        | NodeKind::StrongPrevious { operand }
        | NodeKind::LtlfFuture { operand }
        | NodeKind::LtlfGlobally { operand }
        | NodeKind::StrongNext { operand }
        | NodeKind::WeakNext { operand } => [Some(operand), None],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. }
        | NodeKind::Since { left, right, .. }
        | NodeKind::Triggered { left, right, .. }
        | NodeKind::LtlfUntil { left, right }
        | NodeKind::LtlfRelease { left, right } => [Some(left), Some(right)],
    }
}

fn remap(kind: NodeKind, map: &[u32]) -> NodeKind {
    let m = |id: NodeId| NodeId(map[id.0 as usize]);
    match kind {
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => kind,
        NodeKind::Not { operand } => NodeKind::Not { operand: m(operand) },
        NodeKind::And { left, right } => NodeKind::And { left: m(left), right: m(right) },
        NodeKind::Or { left, right } => NodeKind::Or { left: m(left), right: m(right) },
        NodeKind::Implies { left, right } => NodeKind::Implies { left: m(left), right: m(right) },
        NodeKind::Equivalent { left, right } => {
            NodeKind::Equivalent { left: m(left), right: m(right) }
        }
        NodeKind::Future { interval, operand } => {
            NodeKind::Future { interval, operand: m(operand) }
        }
        NodeKind::Globally { interval, operand } => {
            NodeKind::Globally { interval, operand: m(operand) }
        }
        NodeKind::Until { interval, left, right } => {
            NodeKind::Until { interval, left: m(left), right: m(right) }
        }
        NodeKind::Release { interval, left, right } => {
            NodeKind::Release { interval, left: m(left), right: m(right) }
        }
        NodeKind::Once { interval, operand } => NodeKind::Once { interval, operand: m(operand) },
        NodeKind::Historically { interval, operand } => {
            NodeKind::Historically { interval, operand: m(operand) }
        }
        NodeKind::StrongPrevious { operand } => NodeKind::StrongPrevious { operand: m(operand) },
        NodeKind::Since { interval, left, right } => {
            NodeKind::Since { interval, left: m(left), right: m(right) }
        }
        NodeKind::Triggered { interval, left, right } => {
            NodeKind::Triggered { interval, left: m(left), right: m(right) }
        }
        NodeKind::LtlfFuture { operand } => NodeKind::LtlfFuture { operand: m(operand) },
        NodeKind::LtlfGlobally { operand } => NodeKind::LtlfGlobally { operand: m(operand) },
        NodeKind::LtlfUntil { left, right } => {
            NodeKind::LtlfUntil { left: m(left), right: m(right) }
        }
        NodeKind::LtlfRelease { left, right } => {
            NodeKind::LtlfRelease { left: m(left), right: m(right) }
        }
        NodeKind::StrongNext { operand } => NodeKind::StrongNext { operand: m(operand) },
        NodeKind::WeakNext { operand } => NodeKind::WeakNext { operand: m(operand) },
    }
}

/// The sub-arena reachable from `root`, in the original relative order, so a
/// clause is not refused for nodes of other clauses sharing the graph.
fn reachable(nodes: &[Node], root: NodeId) -> (Vec<Node>, NodeId) {
    let last = root.0 as usize;
    let mut keep = vec![false; last + 1];
    keep[last] = true;
    for index in (0..=last).rev() {
        if keep[index] {
            for operand in operands(nodes[index].kind).into_iter().flatten() {
                keep[operand.0 as usize] = true;
            }
        }
    }
    let mut map = vec![u32::MAX; last + 1];
    let mut compact = Vec::new();
    for index in 0..=last {
        if keep[index] {
            map[index] = compact.len() as u32;
            let mut node = nodes[index];
            node.kind = remap(node.kind, &map);
            compact.push(node);
        }
    }
    let root = NodeId((compact.len() - 1) as u32);
    (compact, root)
}

/// One contract exported as its three decomposition formulas (FR-060,
/// UNGROUNDED design: nothing shows R2U2 decodes status this way).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDecomposition {
    /// Contract identity.
    pub id: u32,
    /// `A` (inactive is `!A`).
    pub assumption: String,
    /// `A && G` (verified).
    pub verified: String,
    /// `A && !G` (invalid).
    pub invalid: String,
}

/// Contract export result: whole contracts exported or refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractExport {
    /// Exported contracts.
    pub exported: Vec<ContractDecomposition>,
    /// Refused contracts with their loss records; nothing partial is exported.
    pub refused: Vec<(u32, NumericLossRecord)>,
    /// Always states that the native C2PO contract construct is DEFERRED.
    pub note: &'static str,
}

/// Exports each contract as `A`, `A && G`, `A && !G`, or refuses the whole
/// contract with the first loss record of any clause. The set is bound to the
/// graph first (identity, profile, root range, as in FR-059); a binding
/// failure refuses the whole set with one `ForeignIdentity` record.
///
/// Implements: FR-060 (decomposition export, refusal-until-pinned)
pub fn export_contracts_c2po(
    graph: &FormulaDocument,
    contracts: &ContractSet,
    table: &PredicateTable,
    catalog: &SignalCatalogDocument,
    work_limit: u64,
) -> Result<ContractExport, NumericLossRecord> {
    let formula = bind_contracts(graph, contracts).map_err(|error| {
        NumericLossRecord::new(
            NumericLossCause::ForeignIdentity,
            NumericLossSubject::Formula,
            error.to_string(),
        )
    })?;
    let mut exported = Vec::new();
    let mut refused = Vec::new();
    for contract in &contracts.contracts {
        let mut nodes: Vec<Node> = formula.nodes().to_vec();
        let push = |nodes: &mut Vec<Node>, kind: NodeKind| {
            nodes.push(Node::new(kind));
            NodeId((nodes.len() - 1) as u32)
        };
        let (a, g) = (contract.assumption, contract.guarantee);
        let verified = push(&mut nodes, NodeKind::And { left: a, right: g });
        let not_g = push(&mut nodes, NodeKind::Not { operand: g });
        let invalid = push(&mut nodes, NodeKind::And { left: a, right: not_g });
        let mut rendered = Vec::with_capacity(3);
        let mut failure = None;
        for root in [a, verified, invalid] {
            let (clause_nodes, clause_root) = reachable(&nodes, root);
            let result = Formula::new(formula.profile(), clause_root, &clause_nodes)
                .map_err(|error| {
                    NumericLossRecord::new(
                        NumericLossCause::ForeignIdentity,
                        NumericLossSubject::Contract(contract.id),
                        format!("{error:?}"),
                    )
                })
                .and_then(|clause| map_numeric_to_c2po(clause, table, catalog, work_limit));
            match result {
                Ok(NumericExportOutcome::Boolean { expression }) => rendered.push(expression),
                Err(record) => {
                    failure = Some(record);
                    break;
                }
            }
        }
        match failure {
            Some(record) => refused.push((contract.id, record)),
            None => {
                let mut parts = rendered.into_iter();
                exported.push(ContractDecomposition {
                    id: contract.id,
                    assumption: parts.next().expect("three clauses"),
                    verified: parts.next().expect("three clauses"),
                    invalid: parts.next().expect("three clauses"),
                });
            }
        }
    }
    Ok(ContractExport {
        exported,
        refused,
        note: "decomposition export only; the native C2PO contract construct is DEFERRED \
               until its syntax is pinned from a retained R2U2 exchange",
    })
}
