//! Opt-in, trace-scoped infinite MLTL evaluation.
//!
//! This module owns lasso semantics and never calls the finite closed or
//! prefix evaluators. A lasso result is evidence about that trace only.

mod export;
mod periodic;

pub use export::{
    export_safety_monitor, replay_target_step, SafetyExportError, SafetyMappingManifest,
    SafetyReplayDisposition, TargetStepObservation,
};

use std::collections::BTreeMap;

use serde::Serialize;
use tl_syntax::{
    FairnessPremisesDocument, InfiniteFormula, InfiniteFormulaDocument, InfiniteNodeKind,
    LassoTraceDocument, LivenessBackend, LivenessDisposition, LivenessSettlement, LivenessSubject,
    LivenessSubjectKind, NodeId, PropositionId, TemporalInterval, TraceObservation,
};

/// Exact four-valued observation type owned by tl-syntax.
pub use tl_syntax::PartialValue as ObservationValue;

/// Exact provider feature identity.
pub const FEATURE: &str = "infinite-trace";
/// Infinite-trace TL profile identity.
pub const PROFILE: &str = "mltl.infinite-trace/v1";
/// Capability fulfilled by this provider.
pub const CAPABILITY: &str = "tl-syntax.liveness/v1";

/// Typed work ceilings for one infinite evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationLimit {
    /// Maximum formula nodes.
    pub max_nodes: usize,
    /// Maximum materialized ultimately periodic positions per node.
    pub max_positions: usize,
    /// Maximum cells across all materialized valuation rows.
    pub max_valuation_cells: usize,
    /// Maximum materialized Boolean states across all formula nodes.
    pub max_states: usize,
    /// Maximum Boolean completions of partial cells.
    pub max_completions: u64,
    /// Maximum evaluation steps, including temporal iteration.
    pub max_steps: u64,
}

impl Default for EvaluationLimit {
    fn default() -> Self {
        Self {
            max_nodes: 10_000,
            max_positions: 100_000,
            max_valuation_cells: 1_000_000,
            max_states: 2_000_000,
            max_completions: 65_536,
            max_steps: 10_000_000,
        }
    }
}

/// Pre-evaluation refusal or inability to complete semantic work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfiniteError {
    /// The formula reference or proposition map was invalid.
    InvalidFormula,
    /// A lasso had no repeating loop or a malformed observation row.
    InvalidLasso,
    /// A subject or document identity did not match the formula and trace.
    IdentityMismatch,
    /// A configured bound or checked arithmetic prevented complete work.
    ResourceIncomplete,
}

impl core::fmt::Display for InfiniteError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "infinite evaluation refused: {self:?}")
    }
}

impl std::error::Error for InfiniteError {}

/// The five FR-341 semantic result labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// All admitted completions satisfy the trace-scoped claim.
    Proved,
    /// All admitted completions falsify the trace-scoped claim.
    Refuted,
    /// The admitted completions do not settle the claim.
    Inconclusive,
    /// The selected subject or profile has no provider procedure.
    Unsupported,
    /// Evaluation failed, including configured resource exhaustion.
    Failed,
}

/// Execution axis of an infinite result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionDisposition {
    /// Semantic work completed.
    Completed,
    /// The provider has no admitted procedure for this request.
    Unsupported,
    /// Configured resource ceilings were reached.
    ResourceIncomplete,
    /// A provider fault prevented evaluation.
    Failed,
}

/// Truth axis of an infinite result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthAvailability {
    /// All admitted completions satisfy the claim.
    True,
    /// All admitted completions falsify the claim.
    False,
    /// Admitted completions disagree.
    Unsettled,
    /// No truth claim is available.
    Unavailable,
}

/// Evidence basis axis of an infinite result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBasis {
    /// Every admitted Boolean completion of the exact lasso was evaluated.
    ExactTrace,
    /// A finite prefix contains a continuation-invariant violation.
    BadPrefix,
    /// Partial evidence remains pending or conflicting.
    Pending,
    /// No sound proof or refutation basis is available.
    Unavailable,
}

/// Typed reason for a nonconclusive result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultReason {
    /// Missing observations leave both truth values possible.
    MissingObservation,
    /// Contradictory observations leave both truth values possible.
    ConflictingObservation,
    /// No completion satisfies all fairness premises.
    EmptyFairAdmission,
    /// The selected subject is a model and this provider has no model procedure.
    ModelProcedureUnavailable,
    /// Configured work was exhausted.
    ResourceIncomplete,
    /// No decisive safety violation was observed in the supplied prefix.
    FinitePrefixUnsettled,
    /// The graph is outside the exact monitorable safety fragment.
    SafetyFragmentUnsupported,
}

/// Exact subject scope of an infinite result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    /// One exact lasso and its admitted Boolean completions.
    Lasso,
    /// One finite prefix, with no claim about an absent continuation.
    FinitePrefix,
    /// One transition system, for which V1 has no model-wide procedure.
    Model,
}

/// Identity bound to an exact trace-scoped evaluation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultIdentity {
    /// Provider feature name.
    pub feature: &'static str,
    /// Provider source revision.
    pub provider_revision: &'static str,
    /// Exact TL profile.
    pub profile: &'static str,
    /// Formula graph identity supplied with the validated graph.
    pub graph_id: String,
    /// Proposition-map identity.
    pub proposition_map_id: String,
    /// Exact subject scope.
    pub subject_kind: SubjectKind,
    /// Subject identity: trace identity for a lasso, model identity for a model.
    pub subject_id: String,
    /// Trace identity, present only for an exact lasso request.
    pub trace_id: Option<String>,
    /// Event-position clock identity.
    pub clock: &'static str,
    /// Selected position of the exact trace.
    pub selected_position: u64,
    /// Fairness roots, in request order.
    pub fairness: Vec<NodeId>,
}

/// Attributable trace-scoped result. No field claims a model-wide theorem.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfiniteResult {
    /// FR-341 label.
    pub disposition: Disposition,
    /// FR-341 execution axis.
    pub execution: ExecutionDisposition,
    /// FR-341 truth axis.
    pub truth: TruthAvailability,
    /// FR-341 evidence basis.
    pub basis: EvidenceBasis,
    /// Typed nonconclusive reason, when present.
    pub reason: Option<ResultReason>,
    /// Exact request/provider attribution.
    pub identity: ResultIdentity,
    /// Number of fair Boolean completions examined.
    pub admitted_completions: u64,
    /// Number of semantic work steps consumed.
    pub evaluation_steps: u64,
}

/// Borrowed complete lasso request. One position in `observations` is one
/// materialized prefix or loop position; `loop_entry` starts the nonempty loop.
struct RawTraceRequest<'a> {
    /// Validated sibling formula edition.
    pub formula: InfiniteFormula<'a>,
    /// Exact graph identity.
    pub graph_id: &'a str,
    /// Exact proposition-map identity.
    pub proposition_map_id: &'a str,
    /// Exact trace identity.
    pub trace_id: &'a str,
    /// Ordered proposition identities in each observation row.
    pub propositions: &'a [PropositionId],
    /// Materialized prefix followed by one nonempty loop.
    pub observations: &'a [Vec<ObservationValue>],
    /// First materialized loop position.
    pub loop_entry: usize,
    /// Selected event position, including repeated loop positions.
    pub selected_position: u64,
    /// Same-graph fairness roots.
    pub fairness: &'a [NodeId],
    /// Configured bounded work.
    pub limit: EvaluationLimit,
}

impl RawTraceRequest<'_> {
    fn identity(&self) -> ResultIdentity {
        ResultIdentity {
            feature: FEATURE,
            provider_revision: crate::TL_MLTL_SOURCE_REVISION,
            profile: PROFILE,
            graph_id: self.graph_id.to_owned(),
            proposition_map_id: self.proposition_map_id.to_owned(),
            subject_kind: SubjectKind::Lasso,
            subject_id: self.trace_id.to_owned(),
            trace_id: Some(self.trace_id.to_owned()),
            clock: "event_position",
            selected_position: self.selected_position,
            fairness: self.fairness.to_vec(),
        }
    }

    fn validate(&self) -> Result<BTreeMap<PropositionId, usize>, InfiniteError> {
        if self.graph_id.is_empty()
            || self.proposition_map_id.is_empty()
            || self.trace_id.is_empty()
        {
            return Err(InfiniteError::IdentityMismatch);
        }
        if self.formula.nodes().len() > self.limit.max_nodes {
            return Err(InfiniteError::ResourceIncomplete);
        }
        if self.observations.is_empty() || self.loop_entry >= self.observations.len() {
            return Err(InfiniteError::InvalidLasso);
        }
        if self.observations.len() > self.limit.max_positions {
            return Err(InfiniteError::ResourceIncomplete);
        }
        if self
            .observations
            .iter()
            .any(|row| row.len() != self.propositions.len())
        {
            return Err(InfiniteError::InvalidLasso);
        }
        let mut index = BTreeMap::new();
        for (offset, proposition) in self.propositions.iter().enumerate() {
            if index.insert(*proposition, offset).is_some() {
                return Err(InfiniteError::InvalidLasso);
            }
        }
        if self.formula.nodes().iter().any(|node| matches!(node.kind, InfiniteNodeKind::Proposition { proposition } if !index.contains_key(&proposition))) {
            return Err(InfiniteError::IdentityMismatch);
        }
        if self
            .fairness
            .iter()
            .any(|root| self.formula.node(*root).is_none())
        {
            return Err(InfiniteError::IdentityMismatch);
        }
        Ok(index)
    }
}

fn result(
    identity: ResultIdentity,
    disposition: Disposition,
    reason: Option<ResultReason>,
    admitted_completions: u64,
    evaluation_steps: u64,
) -> InfiniteResult {
    let (execution, truth, basis) = match disposition {
        Disposition::Proved => (
            ExecutionDisposition::Completed,
            TruthAvailability::True,
            EvidenceBasis::ExactTrace,
        ),
        Disposition::Refuted => (
            ExecutionDisposition::Completed,
            TruthAvailability::False,
            EvidenceBasis::ExactTrace,
        ),
        Disposition::Inconclusive => (
            ExecutionDisposition::Completed,
            TruthAvailability::Unsettled,
            EvidenceBasis::Pending,
        ),
        Disposition::Unsupported => (
            ExecutionDisposition::Unsupported,
            TruthAvailability::Unavailable,
            EvidenceBasis::Unavailable,
        ),
        Disposition::Failed => (
            ExecutionDisposition::ResourceIncomplete,
            TruthAvailability::Unavailable,
            EvidenceBasis::Unavailable,
        ),
    };
    InfiniteResult {
        disposition,
        execution,
        truth,
        basis,
        reason,
        identity,
        admitted_completions,
        evaluation_steps,
    }
}

/// Evaluates all Boolean completions of an exact lasso and filters them through
/// the same-graph fairness roots before settling a trace-scoped claim.
fn evaluate_trace(request: &RawTraceRequest<'_>) -> Result<InfiniteResult, InfiniteError> {
    let index = request.validate()?;
    let mut rows = Vec::with_capacity(request.observations.len());
    let mut unknown = Vec::new();
    let mut missing = false;
    let mut conflicting = false;
    for (row_index, row) in request.observations.iter().enumerate() {
        let mut values = Vec::with_capacity(row.len());
        for (cell_index, value) in row.iter().enumerate() {
            match value {
                ObservationValue::True => values.push(true),
                ObservationValue::False => values.push(false),
                ObservationValue::Missing | ObservationValue::Conflicting => {
                    missing |= *value == ObservationValue::Missing;
                    conflicting |= *value == ObservationValue::Conflicting;
                    unknown.push((row_index, cell_index));
                    values.push(false);
                }
            }
        }
        rows.push(values);
    }
    let combinations = 1_u64.checked_shl(u32::try_from(unknown.len()).unwrap_or(u32::MAX));
    let Some(combinations) = combinations.filter(|count| *count <= request.limit.max_completions)
    else {
        return Ok(result(
            request.identity(),
            Disposition::Failed,
            Some(ResultReason::ResourceIncomplete),
            0,
            0,
        ));
    };
    let selected = usize::try_from(request.selected_position)
        .map_err(|_| InfiniteError::ResourceIncomplete)?;
    let mut all_true = true;
    let mut all_false = true;
    let mut admitted = 0_u64;
    let mut steps = 0_u64;
    for completion in 0..combinations {
        for (bit, (row, cell)) in unknown.iter().enumerate() {
            rows[*row][*cell] = completion & (1_u64 << bit) != 0;
        }
        let mut remaining = request.limit;
        remaining.max_steps = remaining.max_steps.saturating_sub(steps);
        let evaluated = periodic::evaluate(
            request.formula,
            request.formula.root(),
            &rows,
            request.loop_entry,
            &index,
            selected,
            request.fairness,
            remaining,
        );
        let (truth, fair, used) = match evaluated {
            Ok(value) => value,
            Err(InfiniteError::ResourceIncomplete) => {
                return Ok(result(
                    request.identity(),
                    Disposition::Failed,
                    Some(ResultReason::ResourceIncomplete),
                    admitted,
                    steps,
                ))
            }
            Err(other) => return Err(other),
        };
        steps = steps
            .checked_add(used)
            .ok_or(InfiniteError::ResourceIncomplete)?;
        if fair.into_iter().all(|value| value) {
            admitted += 1;
            all_true &= truth;
            all_false &= !truth;
        }
    }
    let (disposition, reason) = if admitted == 0 {
        (
            Disposition::Inconclusive,
            Some(ResultReason::EmptyFairAdmission),
        )
    } else if all_true {
        (Disposition::Proved, None)
    } else if all_false {
        (Disposition::Refuted, None)
    } else if conflicting {
        (
            Disposition::Inconclusive,
            Some(ResultReason::ConflictingObservation),
        )
    } else if missing {
        (
            Disposition::Inconclusive,
            Some(ResultReason::MissingObservation),
        )
    } else {
        (Disposition::Inconclusive, None)
    };
    Ok(result(
        request.identity(),
        disposition,
        reason,
        admitted,
        steps,
    ))
}

/// Validated document request for an exact trace-scoped lasso evaluation.
pub struct LassoRequest<'a> {
    /// Sibling unbounded formula edition.
    pub formula: &'a InfiniteFormulaDocument,
    /// Syntax-validated finite-prefix and repeating-loop observations.
    pub trace: &'a LassoTraceDocument,
    /// Same-graph fairness premises, or no premises.
    pub fairness: Option<&'a FairnessPremisesDocument>,
    /// Exact graph identity.
    pub graph_id: &'a str,
    /// Exact trace identity.
    pub trace_id: &'a str,
    /// Selected event position on the infinite trace.
    pub selected_position: u64,
    /// Configured work ceilings.
    pub limit: EvaluationLimit,
}

/// Evaluates the exact lasso and every common Boolean completion. Identity
/// mismatches refuse before any semantic work.
pub fn evaluate_lasso(request: &LassoRequest<'_>) -> Result<InfiniteResult, InfiniteError> {
    let exact_graph = request
        .formula
        .content_identity()
        .map_err(|_| InfiniteError::IdentityMismatch)?;
    let exact_trace = request
        .trace
        .content_identity()
        .map_err(|_| InfiniteError::IdentityMismatch)?;
    if request.graph_id != exact_graph || request.trace_id != exact_trace {
        return Err(InfiniteError::IdentityMismatch);
    }
    if let Some(fairness) = request.fairness {
        if fairness.graph_identity() != request.graph_id
            || fairness.clock() != request.formula.clock()
        {
            return Err(InfiniteError::IdentityMismatch);
        }
    }
    if request.trace.clock() != request.formula.clock()
        || request.trace.semantic_profile() != request.formula.semantic_profile()
    {
        return Err(InfiniteError::IdentityMismatch);
    }
    let lasso_positions = request
        .trace
        .prefix()
        .len()
        .checked_add(request.trace.loop_observations().len());
    let valuation_cells = lasso_positions
        .and_then(|positions| positions.checked_mul(request.trace.propositions().len()));
    if request.formula.nodes().len() > request.limit.max_nodes
        || lasso_positions.is_none_or(|positions| positions > request.limit.max_positions)
        || valuation_cells.is_none_or(|cells| cells > request.limit.max_valuation_cells)
    {
        let identity = ResultIdentity {
            feature: FEATURE,
            provider_revision: crate::TL_MLTL_SOURCE_REVISION,
            profile: PROFILE,
            graph_id: request.graph_id.to_owned(),
            proposition_map_id: request.trace.proposition_map_identity().to_owned(),
            subject_kind: SubjectKind::Lasso,
            subject_id: request.trace_id.to_owned(),
            trace_id: Some(request.trace_id.to_owned()),
            clock: request.formula.clock().as_str(),
            selected_position: request.selected_position,
            fairness: request
                .fairness
                .map_or_else(Vec::new, |premises| premises.roots().to_vec()),
        };
        return Ok(result(
            identity,
            Disposition::Failed,
            Some(ResultReason::ResourceIncomplete),
            0,
            0,
        ));
    }
    let propositions = request.trace.propositions();
    let observations = request
        .trace
        .prefix()
        .iter()
        .chain(request.trace.loop_observations())
        .map(|observation| {
            observation
                .valuation
                .entries()
                .iter()
                .map(|entry| entry.value)
                .collect()
        })
        .collect::<Vec<Vec<ObservationValue>>>();
    let raw = RawTraceRequest {
        formula: request.formula.formula(),
        graph_id: request.graph_id,
        proposition_map_id: request.trace.proposition_map_identity(),
        trace_id: request.trace_id,
        propositions,
        observations: &observations,
        loop_entry: request.trace.loop_entry(),
        selected_position: request.selected_position,
        fairness: request
            .fairness
            .map_or(&[], FairnessPremisesDocument::roots),
        limit: request.limit,
    };
    match evaluate_trace(&raw) {
        Err(InfiniteError::ResourceIncomplete) => Ok(result(
            raw.identity(),
            Disposition::Failed,
            Some(ResultReason::ResourceIncomplete),
            0,
            0,
        )),
        other => other,
    }
}

/// Reports the V1 absence of a model-wide procedure without treating one lasso
/// as proof of all traces of a transition system.
pub fn evaluate_model(
    formula: &InfiniteFormulaDocument,
    graph_id: &str,
    model_id: &str,
    proposition_map_id: &str,
) -> Result<InfiniteResult, InfiniteError> {
    let exact_graph = formula
        .content_identity()
        .map_err(|_| InfiniteError::IdentityMismatch)?;
    if graph_id != exact_graph || model_id.is_empty() || proposition_map_id.is_empty() {
        return Err(InfiniteError::IdentityMismatch);
    }
    let identity = ResultIdentity {
        feature: FEATURE,
        provider_revision: crate::TL_MLTL_SOURCE_REVISION,
        profile: PROFILE,
        graph_id: graph_id.to_owned(),
        proposition_map_id: proposition_map_id.to_owned(),
        subject_kind: SubjectKind::Model,
        subject_id: model_id.to_owned(),
        trace_id: None,
        clock: formula.clock().as_str(),
        selected_position: 0,
        fairness: Vec::new(),
    };
    Ok(result(
        identity,
        Disposition::Unsupported,
        Some(ResultReason::ModelProcedureUnavailable),
        0,
        0,
    ))
}

/// One finite prefix under the infinite profile. Positions and four-valued
/// valuations are syntax-owned; this request adds exact graph/map attribution.
pub struct PrefixRequest<'a> {
    /// The validated infinite formula document.
    pub formula: &'a InfiniteFormulaDocument,
    /// Exact formula graph identity.
    pub graph_id: &'a str,
    /// Exact proposition-map identity shared by every row.
    pub proposition_map_id: &'a str,
    /// Canonical proposition order for every row.
    pub propositions: &'a [PropositionId],
    /// Origin-based event positions, with no claimed closure.
    pub observations: &'a [TraceObservation],
    /// Configured work ceilings.
    pub limit: EvaluationLimit,
}

impl PrefixRequest<'_> {
    /// Validates the graph and each origin-based observation against one map.
    pub fn validate_shape(&self) -> Result<(), InfiniteError> {
        let graph_id = self
            .formula
            .content_identity()
            .map_err(|_| InfiniteError::IdentityMismatch)?;
        if self.graph_id != graph_id || self.proposition_map_id.is_empty() {
            return Err(InfiniteError::IdentityMismatch);
        }
        let mut known = BTreeMap::new();
        for (at, proposition) in self.propositions.iter().enumerate() {
            if known.insert(*proposition, at).is_some() {
                return Err(InfiniteError::IdentityMismatch);
            }
        }
        if self.formula.nodes().iter().any(|node| {
            matches!(node.kind,
            InfiniteNodeKind::Proposition { proposition } if !known.contains_key(&proposition))
        }) {
            return Err(InfiniteError::IdentityMismatch);
        }
        for (at, observation) in self.observations.iter().enumerate() {
            if observation.position
                != u32::try_from(at).map_err(|_| InfiniteError::ResourceIncomplete)?
                || observation.valuation.proposition_map_identity() != self.proposition_map_id
                || observation.valuation.entries().len() != self.propositions.len()
                || observation
                    .valuation
                    .entries()
                    .iter()
                    .zip(self.propositions)
                    .any(|(entry, proposition)| entry.proposition != *proposition)
            {
                return Err(InfiniteError::IdentityMismatch);
            }
        }
        Ok(())
    }

    /// Domain-separated identity of the exact map and finite observation rows.
    pub fn content_identity(&self) -> Result<String, InfiniteError> {
        crate::context::domain_sha256(
            "tl-mltl.infinite-prefix/v1",
            &(
                self.proposition_map_id,
                self.propositions,
                self.observations,
            ),
        )
        .map_err(|_| InfiniteError::IdentityMismatch)
    }
}

fn finite_horizons(formula: InfiniteFormula<'_>) -> Result<(NodeId, u64), InfiniteError> {
    let Some(root) = formula.node(formula.root()) else {
        return Err(InfiniteError::InvalidFormula);
    };
    let InfiniteNodeKind::Globally {
        interval: TemporalInterval::Unbounded(outer),
        operand,
    } = root.kind
    else {
        return Err(InfiniteError::InvalidFormula);
    };
    if outer.start() != 0 {
        return Err(InfiniteError::InvalidFormula);
    }
    let mut horizons: Vec<u64> = Vec::with_capacity(formula.nodes().len());
    for (at, node) in formula.nodes().iter().enumerate() {
        let operands = node.kind.operands();
        let children = operands
            .into_iter()
            .flatten()
            .map(|child| {
                usize::try_from(child.0)
                    .ok()
                    .and_then(|at| horizons.get(at))
                    .copied()
                    .ok_or(InfiniteError::InvalidFormula)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let child_horizon = children.into_iter().max().unwrap_or(0);
        let horizon = match node.kind {
            InfiniteNodeKind::Future { interval, .. }
            | InfiniteNodeKind::Globally { interval, .. }
            | InfiniteNodeKind::Until { interval, .. }
            | InfiniteNodeKind::Release { interval, .. } => match interval {
                TemporalInterval::Closed(closed) => child_horizon
                    .checked_add(u64::from(closed.end()))
                    .ok_or(InfiniteError::ResourceIncomplete)?,
                TemporalInterval::Unbounded(_)
                    if u32::try_from(at).ok().map(NodeId) == Some(formula.root()) =>
                {
                    0
                }
                TemporalInterval::Unbounded(_) => return Err(InfiniteError::InvalidFormula),
            },
            InfiniteNodeKind::False
            | InfiniteNodeKind::True
            | InfiniteNodeKind::Proposition { .. }
            | InfiniteNodeKind::Not { .. }
            | InfiniteNodeKind::And { .. }
            | InfiniteNodeKind::Or { .. }
            | InfiniteNodeKind::Implies { .. }
            | InfiniteNodeKind::Equivalent { .. }
            | InfiniteNodeKind::Once { .. }
            | InfiniteNodeKind::Historically { .. }
            | InfiniteNodeKind::StrongPrevious { .. }
            | InfiniteNodeKind::Since { .. }
            | InfiniteNodeKind::Triggered { .. } => child_horizon,
        };
        horizons.push(horizon);
    }
    let inner = usize::try_from(operand.0).map_err(|_| InfiniteError::InvalidFormula)?;
    Ok((
        operand,
        *horizons.get(inner).ok_or(InfiniteError::InvalidFormula)?,
    ))
}

/// Refutes only when a finite prefix contains a violation of the exact
/// `G[0,)ψ` safety fragment that every infinite continuation preserves.
/// Otherwise the result remains inconclusive; it never proves a liveness claim.
pub fn evaluate_prefix_safety(
    request: &PrefixRequest<'_>,
) -> Result<InfiniteResult, InfiniteError> {
    evaluate_prefix_safety_at(request, None)
}

fn evaluate_prefix_safety_at(
    request: &PrefixRequest<'_>,
    selected: Option<u64>,
) -> Result<InfiniteResult, InfiniteError> {
    request.validate_shape()?;
    let prefix_id = request.content_identity()?;
    let identity = ResultIdentity {
        feature: FEATURE,
        provider_revision: crate::TL_MLTL_SOURCE_REVISION,
        profile: PROFILE,
        graph_id: request.graph_id.to_owned(),
        proposition_map_id: request.proposition_map_id.to_owned(),
        subject_kind: SubjectKind::FinitePrefix,
        subject_id: prefix_id.clone(),
        trace_id: Some(prefix_id),
        clock: request.formula.clock().as_str(),
        selected_position: 0,
        fairness: Vec::new(),
    };
    if request.formula.nodes().len() > request.limit.max_nodes
        || request.observations.len() > request.limit.max_positions
        || request
            .observations
            .len()
            .checked_mul(request.propositions.len())
            .is_none_or(|cells| cells > request.limit.max_valuation_cells)
    {
        return Ok(result(
            identity,
            Disposition::Failed,
            Some(ResultReason::ResourceIncomplete),
            0,
            0,
        ));
    }
    let mut index = BTreeMap::new();
    for (at, proposition) in request.propositions.iter().enumerate() {
        if index.insert(*proposition, at).is_some() {
            return Err(InfiniteError::IdentityMismatch);
        }
    }
    let mut rows = Vec::with_capacity(request.observations.len() + 1);
    let mut unknown = Vec::new();
    for (at, observation) in request.observations.iter().enumerate() {
        if observation.position
            != u32::try_from(at).map_err(|_| InfiniteError::ResourceIncomplete)?
            || observation.valuation.proposition_map_identity() != request.proposition_map_id
            || observation.valuation.entries().len() != request.propositions.len()
        {
            return Err(InfiniteError::IdentityMismatch);
        }
        let mut row = Vec::with_capacity(request.propositions.len());
        for (cell, (entry, proposition)) in observation
            .valuation
            .entries()
            .iter()
            .zip(request.propositions)
            .enumerate()
        {
            if entry.proposition != *proposition {
                return Err(InfiniteError::IdentityMismatch);
            }
            match entry.value {
                ObservationValue::True => row.push(true),
                ObservationValue::False => row.push(false),
                ObservationValue::Missing | ObservationValue::Conflicting => {
                    unknown.push((at, cell));
                    row.push(false);
                }
            }
        }
        rows.push(row);
    }
    if request.formula.nodes().iter().any(|node| matches!(node.kind, InfiniteNodeKind::Proposition { proposition } if !index.contains_key(&proposition))) {
        return Err(InfiniteError::IdentityMismatch);
    }
    let (inner, horizon) = match finite_horizons(request.formula.formula()) {
        Ok(value) => value,
        Err(InfiniteError::InvalidFormula) => {
            return Ok(result(
                identity,
                Disposition::Unsupported,
                Some(ResultReason::SafetyFragmentUnsupported),
                0,
                0,
            ));
        }
        Err(InfiniteError::ResourceIncomplete) => {
            return Ok(result(
                identity,
                Disposition::Failed,
                Some(ResultReason::ResourceIncomplete),
                0,
                0,
            ));
        }
        Err(other) => return Err(other),
    };
    let combinations = 1_u64
        .checked_shl(u32::try_from(unknown.len()).unwrap_or(u32::MAX))
        .filter(|count| *count <= request.limit.max_completions);
    let Some(combinations) = combinations else {
        return Ok(result(
            identity,
            Disposition::Failed,
            Some(ResultReason::ResourceIncomplete),
            0,
            0,
        ));
    };
    let prefix_len = rows.len();
    rows.push(vec![false; request.propositions.len()]);
    let mut steps = 0_u64;
    for position in 0..prefix_len {
        if selected.is_some_and(|selected| u64::try_from(position).ok() != Some(selected)) {
            continue;
        }
        let enough = u64::try_from(position)
            .ok()
            .and_then(|at| at.checked_add(horizon))
            .is_some_and(|last| last < u64::try_from(prefix_len).unwrap_or(u64::MAX));
        if !enough {
            break;
        }
        let mut all_false = true;
        for completion in 0..combinations {
            for (bit, (row, cell)) in unknown.iter().enumerate() {
                rows[*row][*cell] = completion & (1_u64 << bit) != 0;
            }
            let mut remaining = request.limit;
            remaining.max_steps = remaining.max_steps.saturating_sub(steps);
            let evaluated = periodic::evaluate(
                request.formula.formula(),
                inner,
                &rows,
                prefix_len,
                &index,
                position,
                &[],
                remaining,
            );
            let (truth, _, used) = match evaluated {
                Ok(value) => value,
                Err(InfiniteError::ResourceIncomplete) => {
                    return Ok(result(
                        identity,
                        Disposition::Failed,
                        Some(ResultReason::ResourceIncomplete),
                        0,
                        steps,
                    ));
                }
                Err(other) => return Err(other),
            };
            steps = steps
                .checked_add(used)
                .ok_or(InfiniteError::ResourceIncomplete)?;
            all_false &= !truth;
            if !all_false {
                break;
            }
        }
        if all_false {
            let mut report = result(identity, Disposition::Refuted, None, combinations, steps);
            report.basis = EvidenceBasis::BadPrefix;
            report.identity.selected_position =
                u64::try_from(position).map_err(|_| InfiniteError::ResourceIncomplete)?;
            return Ok(report);
        }
    }
    Ok(result(
        identity,
        Disposition::Inconclusive,
        Some(ResultReason::FinitePrefixUnsettled),
        combinations,
        steps,
    ))
}

/// Subject-specific request served by one deployment provider.
pub enum ProviderRequest<'a> {
    /// One complete lasso with possible partial observations.
    Lasso(LassoRequest<'a>),
    /// One finite prefix, eligible only for decisive safety refutation.
    FinitePrefix(PrefixRequest<'a>),
}

/// One deployment's selected infinite provider, bound to an exact subject.
pub struct InfiniteProvider<'a> {
    /// The exact request served when the router selects this backend.
    pub request: ProviderRequest<'a>,
}

impl LivenessBackend for InfiniteProvider<'_> {
    fn settle(
        &self,
        formula: &InfiniteFormulaDocument,
        subject: LivenessSubject<'_>,
    ) -> LivenessDisposition {
        let evaluated = match (&self.request, subject.kind) {
            (ProviderRequest::Lasso(request), LivenessSubjectKind::LassoTrace)
                if formula == request.formula && subject.identity == request.trace_id =>
            {
                evaluate_lasso(request)
            }
            (ProviderRequest::FinitePrefix(request), LivenessSubjectKind::FinitePrefix)
                if formula == request.formula
                    && request
                        .content_identity()
                        .is_ok_and(|id| id == subject.identity) =>
            {
                evaluate_prefix_safety(request)
            }
            (ProviderRequest::Lasso(_), LivenessSubjectKind::FinitePrefix)
            | (ProviderRequest::Lasso(_), LivenessSubjectKind::Model)
            | (ProviderRequest::FinitePrefix(_), LivenessSubjectKind::Model)
            | (ProviderRequest::FinitePrefix(_), LivenessSubjectKind::LassoTrace)
            | (ProviderRequest::Lasso(_), LivenessSubjectKind::LassoTrace)
            | (ProviderRequest::FinitePrefix(_), LivenessSubjectKind::FinitePrefix) => {
                return LivenessDisposition::Unsupported;
            }
        };
        match evaluated {
            Ok(report) => match report.disposition {
                Disposition::Proved => LivenessDisposition::Proved,
                Disposition::Refuted => LivenessDisposition::Refuted,
                Disposition::Inconclusive => LivenessDisposition::Inconclusive,
                Disposition::Unsupported => LivenessDisposition::Unsupported,
                Disposition::Failed => match report.execution {
                    ExecutionDisposition::ResourceIncomplete => {
                        LivenessDisposition::ResourceIncomplete
                    }
                    ExecutionDisposition::Failed => LivenessDisposition::Failed,
                    ExecutionDisposition::Completed | ExecutionDisposition::Unsupported => {
                        LivenessDisposition::Failed
                    }
                },
            },
            Err(_) => LivenessDisposition::Unsupported,
        }
    }
}

/// A deployment-local exactly-once registration of this provider.
#[derive(Default)]
pub struct ProviderRegistry<'a> {
    provider: Option<&'a InfiniteProvider<'a>>,
}

/// Registration refusal before any formula is routed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistrationError {
    /// The deployment already selected a provider for this capability.
    DuplicateProvider,
}

impl<'a> ProviderRegistry<'a> {
    /// Registers this module's provider exactly once.
    pub fn register(
        &mut self,
        provider: &'a InfiniteProvider<'a>,
    ) -> Result<(), RegistrationError> {
        if self.provider.is_some() {
            return Err(RegistrationError::DuplicateProvider);
        }
        self.provider = Some(provider);
        Ok(())
    }

    /// Routes through the syntax-owned absence or selected-provider path.
    pub fn settle<'formula, 'subject>(
        &self,
        formula: &'formula InfiniteFormulaDocument,
        subject: LivenessSubject<'subject>,
    ) -> LivenessSettlement<'formula, 'subject> {
        tl_syntax::settle_liveness(
            formula,
            subject,
            self.provider
                .map(|provider| provider as &dyn LivenessBackend),
        )
    }
}
