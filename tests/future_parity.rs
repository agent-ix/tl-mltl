//! Direct-versus-lowered W/M parity controls (FR-016).
//!
//! tl-mltl owns no weak-until or strong-release semantics. `W` and `M` exist
//! only as the tl-syntax `tl-syntax.future-operators/v1` lowering, and the
//! evaluator consumes the resulting canonical primitive graph. This file keeps
//! a test-only *direct* reference for bounded W/M finite-trace semantics,
//! written in a first-occurrence form that shares no code or structure with
//! the `Or(U, G)` / `And(R, F)` expansion, and compares it with tl-mltl
//! evaluating the lowered graph. Hand-built mutant expansions show that every
//! control turns red when the lowering is wrong.

use std::{fs, path::Path};

use tl_mltl::{
    analyze_horizon, evaluate_closed_at, evaluate_prefix_at, EvaluationError, EvaluationLimits,
    TruthValue,
};
use tl_syntax::{
    Formula, FutureKind, FutureLoweringRefusal, FutureLoweringRequest, Interval, Node, NodeId,
    NodeKind, PropositionId, RawBounds, SemanticProfile, SourceSpan, FUTURE_LOWERING_NODE_CHARGE,
    FUTURE_LOWERING_REQUEST_V1, FUTURE_OPERATORS_V1, MAX_FORMULA_DOCUMENT_NODES,
};

const PROFILES: [SemanticProfile; 2] = [
    SemanticProfile::ClosedTraceV1,
    SemanticProfile::OnlinePrefixV1,
];
const KINDS: [Derived; 2] = [Derived::WeakUntil, Derived::StrongRelease];

/// A derived future-time operator as the test-local source form names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Derived {
    WeakUntil,
    StrongRelease,
}

impl Derived {
    const fn spelling(self) -> &'static [u8] {
        match self {
            Self::WeakUntil => b"W",
            Self::StrongRelease => b"M",
        }
    }

    const fn other(self) -> Self {
        match self {
            Self::WeakUntil => Self::StrongRelease,
            Self::StrongRelease => Self::WeakUntil,
        }
    }
}

/// Test-local source AST. It is never handed to tl-mltl: only its lowering is.
#[derive(Clone, Debug)]
enum Expr {
    Prop(u32),
    True,
    False,
    Not(Box<Expr>),
    Derived {
        kind: Derived,
        start: u32,
        end: u32,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

fn prop(id: u32) -> Expr {
    Expr::Prop(id)
}

fn not(operand: Expr) -> Expr {
    Expr::Not(Box::new(operand))
}

fn derived(kind: Derived, start: u32, end: u32, left: Expr, right: Expr) -> Expr {
    Expr::Derived {
        kind,
        start,
        end,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// One wrong expansion. Each is a plausible defect in a W/M lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mutation {
    /// `U` and `R` exchanged in the binary node.
    UntilReleaseSwapped,
    /// `G` and `F` exchanged in the unary node.
    GloballyFutureSwapped,
    /// `Or` and `And` exchanged in the root.
    OrAndSwapped,
    /// The other derived operator's expansion used.
    KindSwapped,
    /// `p` and `q` exchanged in the binary node.
    BinaryOperandsSwapped,
    /// The unary node applied to `q` instead of `p`.
    UnaryBindsRight,
    /// The inclusive end widened to `b + 1`.
    EndExtended,
    /// The inclusive start raised to `a + 1`.
    StartRaised,
}

const MUTATIONS: [Mutation; 8] = [
    Mutation::UntilReleaseSwapped,
    Mutation::GloballyFutureSwapped,
    Mutation::OrAndSwapped,
    Mutation::KindSwapped,
    Mutation::BinaryOperandsSwapped,
    Mutation::UnaryBindsRight,
    Mutation::EndExtended,
    Mutation::StartRaised,
];

/// How a derived node reaches the canonical graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lowering {
    /// The tl-syntax `FutureLoweringRequest` under test.
    TlSyntax,
    /// Direct canonical construction written out by hand, optionally mutated.
    Hand(Option<Mutation>),
}

fn interval(start: u32, end: u32) -> Interval {
    Interval::new(start, end).unwrap()
}

/// Hand-built three-node expansion appended at `base`.
fn hand_expansion(
    kind: Derived,
    start: u32,
    end: u32,
    left: NodeId,
    right: NodeId,
    base: u32,
    mutation: Option<Mutation>,
) -> [Node; FUTURE_LOWERING_NODE_CHARGE] {
    let kind = if mutation == Some(Mutation::KindSwapped) {
        kind.other()
    } else {
        kind
    };
    let window = match mutation {
        Some(Mutation::EndExtended) => interval(start, end + 1),
        Some(Mutation::StartRaised) => interval(start + 1, end.max(start + 1)),
        _ => interval(start, end),
    };
    let (binary_left, binary_right) = if mutation == Some(Mutation::BinaryOperandsSwapped) {
        (right, left)
    } else {
        (left, right)
    };
    let unary = if mutation == Some(Mutation::UnaryBindsRight) {
        right
    } else {
        left
    };
    let until_like =
        (kind == Derived::WeakUntil) != (mutation == Some(Mutation::UntilReleaseSwapped));
    let globally_like =
        (kind == Derived::WeakUntil) != (mutation == Some(Mutation::GloballyFutureSwapped));
    let or_like = (kind == Derived::WeakUntil) != (mutation == Some(Mutation::OrAndSwapped));
    let binary = if until_like {
        NodeKind::Until {
            interval: window,
            left: binary_left,
            right: binary_right,
        }
    } else {
        NodeKind::Release {
            interval: window,
            left: binary_left,
            right: binary_right,
        }
    };
    let unary = if globally_like {
        NodeKind::Globally {
            interval: window,
            operand: unary,
        }
    } else {
        NodeKind::Future {
            interval: window,
            operand: unary,
        }
    };
    let (first, second) = (NodeId(base), NodeId(base + 1));
    let root = if or_like {
        NodeKind::Or {
            left: first,
            right: second,
        }
    } else {
        NodeKind::And {
            left: first,
            right: second,
        }
    };
    [Node::new(binary), Node::new(unary), Node::new(root)]
}

fn lower_request<'a>(
    profile: SemanticProfile,
    nodes: &'a [Node],
    kind: Derived,
    bounds: RawBounds,
    left: NodeId,
    right: NodeId,
) -> FutureLoweringRequest<'a> {
    FutureLoweringRequest {
        request_identity: FUTURE_LOWERING_REQUEST_V1.as_bytes(),
        operator_profile: FUTURE_OPERATORS_V1.as_bytes(),
        kind: kind.spelling(),
        semantic_profile: profile.as_str().as_bytes(),
        formula: Formula::new(profile, NodeId(nodes.len() as u32 - 1), nodes).unwrap(),
        left: u64::from(left.0),
        right: u64::from(right.0),
        interval: Some(bounds),
        operator_span: None,
        expression_span: None,
    }
}

/// Appends the canonical graph for `expr` and returns its root.
fn build(
    expr: &Expr,
    profile: SemanticProfile,
    lowering: Lowering,
    nodes: &mut Vec<Node>,
) -> NodeId {
    let leaf = |nodes: &mut Vec<Node>, kind| {
        nodes.push(Node::new(kind));
        NodeId(nodes.len() as u32 - 1)
    };
    match expr {
        Expr::Prop(id) => leaf(
            nodes,
            NodeKind::Proposition {
                proposition: PropositionId(*id),
            },
        ),
        Expr::True => leaf(nodes, NodeKind::True),
        Expr::False => leaf(nodes, NodeKind::False),
        Expr::Not(operand) => {
            let operand = build(operand, profile, lowering, nodes);
            leaf(nodes, NodeKind::Not { operand })
        }
        Expr::Derived {
            kind,
            start,
            end,
            left,
            right,
        } => {
            let left = build(left, profile, lowering, nodes);
            let right = build(right, profile, lowering, nodes);
            let base = nodes.len() as u32;
            let generated = match lowering {
                Lowering::TlSyntax => {
                    let bounds = RawBounds::new(u64::from(*start), u64::from(*end));
                    let lowered = lower_request(profile, nodes, *kind, bounds, left, right)
                        .lower()
                        .unwrap();
                    assert_eq!(
                        lowered.node_ids(),
                        [NodeId(base), NodeId(base + 1), NodeId(base + 2)]
                    );
                    assert_eq!(lowered.root(), NodeId(base + 2));
                    *lowered.nodes()
                }
                Lowering::Hand(mutation) => {
                    hand_expansion(*kind, *start, *end, left, right, base, mutation)
                }
            };
            nodes.extend_from_slice(&generated);
            NodeId(base + 2)
        }
    }
}

fn graph(expr: &Expr, profile: SemanticProfile, lowering: Lowering) -> Vec<Node> {
    let mut nodes = Vec::new();
    let root = build(expr, profile, lowering, &mut nodes);
    assert_eq!(root.0 as usize, nodes.len() - 1);
    nodes
}

fn formula(profile: SemanticProfile, nodes: &[Node]) -> Formula<'_> {
    Formula::new(profile, NodeId(nodes.len() as u32 - 1), nodes).unwrap()
}

fn trace_from_bits(length: usize, bits: usize) -> Vec<Vec<PropositionId>> {
    (0..length)
        .map(|time| {
            (0..=1)
                .filter(|proposition| bits & (1 << (time * 2 + proposition)) != 0)
                .map(|proposition| PropositionId(proposition as u32))
                .collect()
        })
        .collect()
}

/// Every trace over propositions 0 and 1 of length at most `max_length`.
fn all_traces(max_length: usize) -> Vec<Vec<Vec<PropositionId>>> {
    (0..=max_length)
        .flat_map(|length| {
            (0..1_usize << (length * 2)).map(move |bits| trace_from_bits(length, bits))
        })
        .collect()
}

/// Proposition value at `time`; instants beyond the trace take `beyond`.
fn observed(trace: &[Vec<PropositionId>], time: u64, proposition: u32, beyond: bool) -> bool {
    usize::try_from(time)
        .ok()
        .and_then(|index| trace.get(index))
        .map_or(beyond, |instant| {
            instant.binary_search(&PropositionId(proposition)).is_ok()
        })
}

/// Direct finite-trace W/M semantics in first-occurrence form.
///
/// `p W[a,b] q` holds at `t` iff `p` holds throughout `[t+a,t+b]`, or `q`
/// holds at some offset no later than the first offset where `p` fails.
/// `p M[a,b] q` holds iff `p` holds at some offset in the window and `q`
/// holds at every offset up to and including the first such offset.
fn direct(expr: &Expr, trace: &[Vec<PropositionId>], time: u64, beyond: bool) -> bool {
    match expr {
        Expr::Prop(id) => observed(trace, time, *id, beyond),
        Expr::True => true,
        Expr::False => false,
        Expr::Not(operand) => !direct(operand, trace, time, beyond),
        Expr::Derived {
            kind,
            start,
            end,
            left,
            right,
        } => {
            let at = |offset: u32| time + u64::from(offset);
            match kind {
                Derived::WeakUntil => {
                    match (*start..=*end).find(|&offset| !direct(left, trace, at(offset), beyond)) {
                        None => true,
                        Some(first_failure) => (*start..=first_failure)
                            .any(|offset| direct(right, trace, at(offset), beyond)),
                    }
                }
                Derived::StrongRelease => {
                    match (*start..=*end).find(|&offset| direct(left, trace, at(offset), beyond)) {
                        None => false,
                        Some(first_hold) => (*start..=first_hold)
                            .all(|offset| direct(right, trace, at(offset), beyond)),
                    }
                }
            }
        }
    }
}

fn truth(value: bool) -> TruthValue {
    if value {
        TruthValue::True
    } else {
        TruthValue::False
    }
}

fn negation_free(expr: &Expr) -> bool {
    match expr {
        Expr::Prop(_) | Expr::True | Expr::False => true,
        Expr::Not(_) => false,
        Expr::Derived { left, right, .. } => negation_free(left) && negation_free(right),
    }
}

/// Exact verdict over every continuation of an open prefix.
///
/// Valid only for negation-free expressions: they are monotone in every
/// observation, so all continuations agree iff the all-false and all-true
/// continuations agree.
fn direct_prefix(expr: &Expr, prefix: &[Vec<PropositionId>], time: u64) -> TruthValue {
    assert!(
        negation_free(expr),
        "the continuation bound needs a monotone expression"
    );
    match (
        direct(expr, prefix, time, false),
        direct(expr, prefix, time, true),
    ) {
        (true, true) => TruthValue::True,
        (false, false) => TruthValue::False,
        _ => TruthValue::Pending,
    }
}

/// Direct lookahead: `b` plus the deeper operand, with checked arithmetic.
fn direct_lookahead(expr: &Expr) -> u64 {
    match expr {
        Expr::Prop(_) | Expr::True | Expr::False => 0,
        Expr::Not(operand) => direct_lookahead(operand),
        Expr::Derived {
            end, left, right, ..
        } => u64::from(*end)
            .checked_add(direct_lookahead(left).max(direct_lookahead(right)))
            .unwrap(),
    }
}

/// Every operand shape the sweeps quantify, for one kind and outer interval.
fn shapes(kind: Derived, start: u32, end: u32) -> Vec<Expr> {
    vec![
        derived(kind, start, end, prop(0), prop(1)),
        derived(kind, start, end, prop(1), prop(0)),
        derived(kind, start, end, prop(0), prop(0)),
        derived(kind, start, end, Expr::True, prop(1)),
        derived(kind, start, end, prop(0), Expr::False),
        derived(
            kind,
            start,
            end,
            prop(0),
            derived(kind.other(), 0, 1, prop(1), prop(0)),
        ),
        derived(
            kind,
            start,
            end,
            derived(kind, 1, 1, prop(0), prop(1)),
            prop(1),
        ),
        // Negated and correlated operands: closed profile only.
        derived(kind, start, end, not(prop(0)), prop(1)),
        derived(kind, start, end, prop(0), not(prop(0))),
    ]
}

fn small_intervals() -> impl Iterator<Item = (u32, u32)> {
    (0..=3).flat_map(|start| (start..=3).map(move |end| (start, end)))
}

fn closed_verdict(nodes: &[Node], trace: &[Vec<PropositionId>], time: u64) -> TruthValue {
    evaluate_closed_at(
        formula(SemanticProfile::ClosedTraceV1, nodes),
        "wm-parity",
        trace,
        "wm-trace",
        time,
        EvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

fn prefix_verdict(
    nodes: &[Node],
    trace: &[Vec<PropositionId>],
    closed: bool,
    time: u64,
) -> TruthValue {
    evaluate_prefix_at(
        formula(SemanticProfile::OnlinePrefixV1, nodes),
        "wm-parity",
        trace,
        "wm-trace",
        closed,
        time,
        EvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

/// Counts direct-reference disagreements for one lowering over the sweep.
///
/// Closed profile: every shape. Online-prefix profile: every negation-free
/// shape, open against the exact continuation verdict and explicitly closed
/// against the closed direct verdict.
fn disagreements(kind: Derived, lowering: Lowering, traces: &[Vec<Vec<PropositionId>>]) -> usize {
    let mut disagreements = 0;
    for (start, end) in small_intervals() {
        for expr in shapes(kind, start, end) {
            let closed_nodes = graph(&expr, SemanticProfile::ClosedTraceV1, lowering);
            let prefix_nodes = graph(&expr, SemanticProfile::OnlinePrefixV1, lowering);
            let monotone = negation_free(&expr);
            for trace in traces {
                for time in 0..=2 {
                    let expected = truth(direct(&expr, trace, time, false));
                    disagreements +=
                        usize::from(closed_verdict(&closed_nodes, trace, time) != expected);
                    disagreements +=
                        usize::from(prefix_verdict(&prefix_nodes, trace, true, time) != expected);
                    if monotone {
                        let open = direct_prefix(&expr, trace, time);
                        disagreements +=
                            usize::from(prefix_verdict(&prefix_nodes, trace, false, time) != open);
                    }
                }
            }
        }
    }
    disagreements
}

// Trace: TC-076, FR-016-AC-1
#[test]
fn lowered_wm_matches_direct_closed_semantics_over_boundary_windows_and_traces() {
    let traces = all_traces(5);
    for kind in KINDS {
        let mut cases = 0_usize;
        for (start, end) in small_intervals() {
            for expr in shapes(kind, start, end) {
                let nodes = graph(&expr, SemanticProfile::ClosedTraceV1, Lowering::TlSyntax);
                for trace in &traces {
                    for time in 0..=2 {
                        let expected = truth(direct(&expr, trace, time, false));
                        assert_eq!(
                            closed_verdict(&nodes, trace, time),
                            expected,
                            "{kind:?} {expr:?} trace={trace:?} time={time}"
                        );
                        cases += 1;
                    }
                }
            }
        }
        // 10 windows x 9 shapes x 1365 traces x 3 verdict times.
        assert_eq!(cases, 368_550, "the closed sweep population moved");
    }
}

// Trace: TC-077, FR-016-AC-2
#[test]
fn lowered_wm_matches_direct_prefix_semantics_and_progress() {
    let traces = all_traces(5);
    for kind in KINDS {
        let mut pending = 0_usize;
        for (start, end) in small_intervals() {
            for expr in shapes(kind, start, end) {
                let nodes = graph(&expr, SemanticProfile::OnlinePrefixV1, Lowering::TlSyntax);
                // The negation-free restriction applies only to open prefixes:
                // an explicitly closed prefix is exact for every shape.
                let monotone = negation_free(&expr);
                for trace in &traces {
                    for time in 0..=2 {
                        if monotone {
                            let open = prefix_verdict(&nodes, trace, false, time);
                            assert_eq!(
                                open,
                                direct_prefix(&expr, trace, time),
                                "open {kind:?} {expr:?} trace={trace:?} time={time}"
                            );
                            pending += usize::from(open == TruthValue::Pending);
                        }
                        assert_eq!(
                            prefix_verdict(&nodes, trace, true, time),
                            truth(direct(&expr, trace, time, false)),
                            "closed prefix {kind:?} {expr:?} trace={trace:?} time={time}"
                        );
                    }
                }
            }
        }
        assert!(
            pending > 0,
            "{kind:?}: the open-prefix sweep never exercised pending"
        );
    }

    // Progress: along every prefix of a longer trace, the open verdict is the
    // exact continuation verdict, never retracts once decisive, and is decisive
    // and equal to the closed verdict once the prefix covers the horizon.
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let long_traces: Vec<_> = (0..96)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            trace_from_bits(7, (state & 0x3fff) as usize)
        })
        .collect();
    for kind in KINDS {
        for (start, end) in small_intervals() {
            for expr in shapes(kind, start, end).into_iter().filter(negation_free) {
                let nodes = graph(&expr, SemanticProfile::OnlinePrefixV1, Lowering::TlSyntax);
                let lookahead = direct_lookahead(&expr);
                for trace in &long_traces {
                    for time in 0..=2 {
                        let mut decided: Option<TruthValue> = None;
                        for length in 0..=trace.len() {
                            let prefix = &trace[..length];
                            let open = prefix_verdict(&nodes, prefix, false, time);
                            assert_eq!(open, direct_prefix(&expr, prefix, time));
                            if let Some(decided) = decided {
                                assert_eq!(
                                    open, decided,
                                    "{kind:?} {expr:?} retracted at {length}"
                                );
                            } else if open != TruthValue::Pending {
                                decided = Some(open);
                            }
                            if length as u64 > time + lookahead {
                                assert_eq!(
                                    open,
                                    truth(direct(&expr, prefix, time, false)),
                                    "{kind:?} {expr:?} undecided beyond its horizon"
                                );
                            }
                        }
                        assert!(
                            time + lookahead < trace.len() as u64,
                            "progress traces must cover every horizon"
                        );
                    }
                }
            }
        }
    }
}

// Trace: TC-078, FR-016-AC-3
#[test]
fn lowered_wm_horizon_matches_direct_lookahead_including_maximum_bounds() {
    let max = u32::MAX;
    let mut cases = Vec::new();
    for kind in KINDS {
        for (start, end) in [(0, 0), (0, 3), (2, 2), (0, max), (max, max)] {
            cases.push(derived(kind, start, end, prop(0), prop(1)));
            cases.push(derived(
                kind,
                start,
                end,
                prop(0),
                derived(kind.other(), max, max, prop(1), prop(0)),
            ));
            cases.push(derived(
                kind,
                start,
                end,
                derived(kind, 0, 0, prop(0), prop(1)),
                not(prop(1)),
            ));
        }
    }
    let nested_maximum = derived(
        Derived::WeakUntil,
        max,
        max,
        derived(Derived::StrongRelease, max, max, prop(0), prop(1)),
        prop(1),
    );
    cases.push(nested_maximum.clone());

    for expr in &cases {
        for profile in PROFILES {
            let lowered = graph(expr, profile, Lowering::TlSyntax);
            let direct_graph = graph(expr, profile, Lowering::Hand(None));
            let report = analyze_horizon(formula(profile, &lowered), "wm-horizon").unwrap();
            assert_eq!(
                report,
                analyze_horizon(formula(profile, &direct_graph), "wm-horizon").unwrap()
            );
            assert_eq!(report.lookahead, direct_lookahead(expr), "{expr:?}");
            assert_eq!(report.propagation_delay, report.lookahead);
            assert_eq!(report.required_buffer, report.lookahead + 1);
            assert_eq!(report.semantic_profile, profile.as_str());
            assert_eq!(report.formula_root as usize, lowered.len() - 1);
        }
    }
    let report = analyze_horizon(
        formula(
            SemanticProfile::OnlinePrefixV1,
            &graph(
                &nested_maximum,
                SemanticProfile::OnlinePrefixV1,
                Lowering::TlSyntax,
            ),
        ),
        "nested-maximum",
    )
    .unwrap();
    assert_eq!(report.lookahead, 2 * u64::from(max));
    assert_eq!(report.required_buffer, 2 * u64::from(max) + 1);

    // The evaluation record carries the same horizon as the analysis, for both
    // operators under both profiles.
    for kind in KINDS {
        let expr = derived(kind, 1, 3, prop(0), prop(1));
        for profile in PROFILES {
            let nodes = graph(&expr, profile, Lowering::TlSyntax);
            let analysis = analyze_horizon(formula(profile, &nodes), "wm").unwrap();
            let limits = EvaluationLimits::default();
            let record = match profile {
                SemanticProfile::ClosedTraceV1 => {
                    evaluate_closed_at(formula(profile, &nodes), "wm", &[], "empty", 0, limits)
                }
                SemanticProfile::OnlinePrefixV1 => evaluate_prefix_at(
                    formula(profile, &nodes),
                    "wm",
                    &[],
                    "empty",
                    false,
                    0,
                    limits,
                ),
            }
            .unwrap();
            assert_eq!(record.horizon, analysis.lookahead, "{kind:?} {profile:?}");
            assert_eq!(
                record.horizon,
                direct_lookahead(&expr),
                "{kind:?} {profile:?}"
            );
            if profile == SemanticProfile::OnlinePrefixV1 {
                assert_eq!(record.verdict, TruthValue::Pending);
            }
        }
    }
    let expr = derived(Derived::StrongRelease, 1, 3, prop(0), prop(1));

    // A widened endpoint is visible to the horizon control.
    let mutated = graph(
        &expr,
        SemanticProfile::OnlinePrefixV1,
        Lowering::Hand(Some(Mutation::EndExtended)),
    );
    assert_ne!(
        analyze_horizon(formula(SemanticProfile::OnlinePrefixV1, &mutated), "wm")
            .unwrap()
            .lookahead,
        direct_lookahead(&expr)
    );
}

fn evaluate_both(
    profile: SemanticProfile,
    nodes: &[Node],
    trace: &[Vec<PropositionId>],
    time: u64,
    limits: EvaluationLimits,
) -> Result<TruthValue, EvaluationError> {
    let formula = formula(profile, nodes);
    match profile {
        SemanticProfile::ClosedTraceV1 => {
            evaluate_closed_at(formula, "wm", trace, "t", time, limits)
        }
        SemanticProfile::OnlinePrefixV1 => {
            evaluate_prefix_at(formula, "wm", trace, "t", false, time, limits)
        }
    }
    .map(|report| report.verdict)
}

// Trace: TC-079, FR-016-AC-4
#[test]
fn lowered_wm_resource_outcomes_match_direct_canonical_construction() {
    let max = u32::MAX;
    let defaults = EvaluationLimits::default();
    let trace = trace_from_bits(4, 0b1001_0110);
    for kind in KINDS {
        for profile in PROFILES {
            // Parity itself is structural: the evaluator is a deterministic
            // function of the canonical graph, so byte-identical lowered and
            // direct graphs share every resource decision. Comparing the two
            // outcomes would restate that equality, so instead each
            // construction is held to the exact expected outcome on its own.
            let nested = derived(
                kind,
                0,
                3,
                prop(0),
                derived(kind.other(), 1, 2, prop(1), prop(0)),
            );
            let full_window = derived(kind, 0, max, prop(0), prop(1));
            let span_window = derived(kind, 2, 6, prop(0), prop(1));
            let last_window = derived(kind, max, max, prop(0), prop(1));
            for expr in [&nested, &full_window, &span_window, &last_window] {
                assert_eq!(
                    graph(expr, profile, Lowering::TlSyntax),
                    graph(expr, profile, Lowering::Hand(None)),
                    "{kind:?} {profile:?} {expr:?}"
                );
            }

            let mut work_boundaries = Vec::new();
            for construction in [Lowering::TlSyntax, Lowering::Hand(None)] {
                let nodes = graph(&nested, profile, construction);

                // Work limit: every limit below the requirement is refused
                // with that exact limit; the requirement itself is admitted.
                let required = (1..=10_000)
                    .find(|&limit| {
                        evaluate_both(
                            profile,
                            &nodes,
                            &trace,
                            1,
                            EvaluationLimits {
                                max_node_evaluations: limit,
                                ..defaults
                            },
                        )
                        .is_ok()
                    })
                    .expect("the nested expansion fits in 10000 node evaluations");
                for limit in [1, required - 1, required, required + 1] {
                    let outcome = evaluate_both(
                        profile,
                        &nodes,
                        &trace,
                        1,
                        EvaluationLimits {
                            max_node_evaluations: limit,
                            ..defaults
                        },
                    );
                    if limit < required {
                        assert_eq!(outcome, Err(EvaluationError::WorkLimitExceeded { limit }));
                    } else {
                        assert!(outcome.is_ok(), "{construction:?} limit={limit}");
                    }
                }
                work_boundaries.push(required);

                // Recursion depth: Or/And -> U/R -> Or/And -> U/R -> proposition.
                for depth in 0..=5 {
                    let limits = EvaluationLimits {
                        max_recursion_depth: depth,
                        ..defaults
                    };
                    let outcome = evaluate_both(profile, &nodes, &trace, 0, limits);
                    if depth < 4 {
                        assert_eq!(
                            outcome,
                            Err(EvaluationError::RecursionDepthExceeded { limit: depth }),
                            "{kind:?} {profile:?} {construction:?} depth={depth}"
                        );
                    } else {
                        assert!(outcome.is_ok(), "{construction:?} depth={depth}");
                    }
                }

                // Temporal span: the full u32 window is refused before any verdict.
                let full = graph(&full_window, profile, construction);
                assert_eq!(
                    evaluate_both(profile, &full, &[], 0, defaults),
                    Err(EvaluationError::TemporalSpanExceeded {
                        requested: 1_u64 << 32,
                        limit: 100_000,
                    })
                );
                let window = graph(&span_window, profile, construction);
                for (span, admitted) in [(4, false), (5, true)] {
                    let limits = EvaluationLimits {
                        max_temporal_span: span,
                        ..defaults
                    };
                    let outcome = evaluate_both(profile, &window, &trace, 0, limits);
                    if admitted {
                        assert!(outcome.is_ok());
                    } else {
                        assert_eq!(
                            outcome,
                            Err(EvaluationError::TemporalSpanExceeded {
                                requested: 5,
                                limit: 4
                            })
                        );
                    }
                }

                // Time arithmetic: `[u32::MAX, u32::MAX]` at the last
                // representable verdict time evaluates; one instant later is
                // refused, not wrapped.
                let last = graph(&last_window, profile, construction);
                let latest = u64::MAX - u64::from(max);
                let expected = match profile {
                    SemanticProfile::ClosedTraceV1 => TruthValue::False,
                    SemanticProfile::OnlinePrefixV1 => TruthValue::Pending,
                };
                assert_eq!(
                    evaluate_both(profile, &last, &[], latest, defaults),
                    Ok(expected)
                );
                assert_eq!(
                    evaluate_both(profile, &last, &[], latest + 1, defaults),
                    Err(EvaluationError::TimeOverflow)
                );
            }
            assert_eq!(
                work_boundaries[0], work_boundaries[1],
                "{kind:?} {profile:?}: work boundary differs between constructions"
            );
        }
    }
}

// Trace: TC-079, FR-016-AC-4
#[test]
fn lowered_wm_nodes_reports_and_node_budget_equal_direct_construction() {
    for kind in KINDS {
        for profile in PROFILES {
            let nodes = [
                Node::new(NodeKind::Proposition {
                    proposition: PropositionId(3),
                }),
                Node::new(NodeKind::Proposition {
                    proposition: PropositionId(4),
                }),
            ];
            let mut request = lower_request(
                profile,
                &nodes,
                kind,
                RawBounds::new(1, 5),
                NodeId(0),
                NodeId(1),
            );
            request.operator_span = Some(RawBounds::new(3, 7));
            request.expression_span = Some(RawBounds::new(0, 12));
            let lowered = request.lower().unwrap();
            let span = SourceSpan::new(0, 12).unwrap();
            let expected = hand_expansion(kind, 1, 5, NodeId(0), NodeId(1), 2, None)
                .map(|node| Node::with_span(node.kind, span));
            assert_eq!(lowered.nodes(), &expected, "{kind:?} {profile:?}");
            let report = lowered.report();
            let kind_name = match kind {
                Derived::WeakUntil => FutureKind::WeakUntil,
                Derived::StrongRelease => FutureKind::StrongRelease,
            };
            assert_eq!(report.kind(), kind_name);
            assert_eq!(report.semantic_profile(), profile);
            assert_eq!((report.left(), report.right()), (NodeId(0), NodeId(1)));
            assert_eq!(
                (report.first_generated(), report.root()),
                (NodeId(2), NodeId(4))
            );
            assert_eq!(report.generated_count(), FUTURE_LOWERING_NODE_CHARGE);
            assert_eq!(report.operator_span(), Some(SourceSpan::new(3, 7).unwrap()));
            assert_eq!(report.expression_span(), Some(span));

            // Diagnostic spans never change a verdict or horizon.
            let mut spanned = nodes.to_vec();
            spanned.extend_from_slice(lowered.nodes());
            let expr = derived(kind, 1, 5, prop(3), prop(4));
            let unspanned = graph(&expr, profile, Lowering::TlSyntax);
            let trace = [vec![PropositionId(4)], vec![PropositionId(3)], vec![]];
            for time in 0..=2 {
                assert_eq!(
                    evaluate_both(profile, &spanned, &trace, time, EvaluationLimits::default()),
                    evaluate_both(
                        profile,
                        &unspanned,
                        &trace,
                        time,
                        EvaluationLimits::default()
                    )
                );
            }
            assert_eq!(
                analyze_horizon(formula(profile, &spanned), "wm").unwrap(),
                analyze_horizon(formula(profile, &unspanned), "wm").unwrap()
            );
        }
    }

    // The node budget: a lowering that lands exactly on the formula-v1 limit is
    // admitted and evaluates; one more base node is refused.
    let base = MAX_FORMULA_DOCUMENT_NODES - FUTURE_LOWERING_NODE_CHARGE;
    let mut nodes = vec![Node::new(NodeKind::True); base];
    nodes[0] = Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    });
    let lowered = lower_request(
        SemanticProfile::ClosedTraceV1,
        &nodes,
        Derived::WeakUntil,
        RawBounds::new(0, 2),
        NodeId(0),
        NodeId(1),
    )
    .lower()
    .unwrap();
    nodes.extend_from_slice(lowered.nodes());
    assert_eq!(nodes.len(), MAX_FORMULA_DOCUMENT_NODES);
    assert_eq!(
        evaluate_both(
            SemanticProfile::ClosedTraceV1,
            &nodes,
            &[vec![], vec![PropositionId(0)]],
            0,
            EvaluationLimits::default()
        ),
        Ok(TruthValue::True)
    );
    let over = vec![Node::new(NodeKind::True); base + 1];
    assert_eq!(
        lower_request(
            SemanticProfile::ClosedTraceV1,
            &over,
            Derived::StrongRelease,
            RawBounds::new(0, 2),
            NodeId(0),
            NodeId(1),
        )
        .lower(),
        Err(FutureLoweringRefusal::DocumentNodeLimitExceeded {
            node_count: MAX_FORMULA_DOCUMENT_NODES + 1,
            limit: MAX_FORMULA_DOCUMENT_NODES,
        })
    );
}

// Trace: TC-080, FR-016-AC-5
#[test]
fn every_wrong_wm_expansion_fails_the_direct_parity_controls() {
    let traces = all_traces(3);
    for kind in KINDS {
        assert_eq!(
            disagreements(kind, Lowering::TlSyntax, &traces),
            0,
            "{kind:?}: the tl-syntax lowering disagrees with the direct reference"
        );
        assert_eq!(
            disagreements(kind, Lowering::Hand(None), &traces),
            0,
            "{kind:?}: the unmutated hand construction disagrees with the direct reference"
        );
        for mutation in MUTATIONS {
            assert!(
                disagreements(kind, Lowering::Hand(Some(mutation)), &traces) > 0,
                "{kind:?}: mutation {mutation:?} survived every parity control"
            );
            // The mutant must also differ from the graph tl-syntax emits.
            let expr = derived(kind, 0, 2, prop(0), prop(1));
            assert_ne!(
                graph(
                    &expr,
                    SemanticProfile::ClosedTraceV1,
                    Lowering::Hand(Some(mutation))
                ),
                graph(&expr, SemanticProfile::ClosedTraceV1, Lowering::TlSyntax),
                "{kind:?}: mutation {mutation:?} is not a different graph"
            );
        }
    }
}

/// Names every canonical node kind with no wildcard arm. A derived variant
/// added to `NodeKind` makes this test file fail to compile, so the evaluator
/// input vocabulary cannot silently acquire W/M.
const fn canonical_vocabulary(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::False => "false",
        NodeKind::True => "true",
        NodeKind::Proposition { .. } => "proposition",
        NodeKind::Not { .. } => "not",
        NodeKind::And { .. } => "and",
        NodeKind::Or { .. } => "or",
        NodeKind::Implies { .. } => "implies",
        NodeKind::Equivalent { .. } => "equivalent",
        NodeKind::Future { .. } => "future",
        NodeKind::Globally { .. } => "globally",
        NodeKind::Until { .. } => "until",
        NodeKind::Release { .. } => "release",
    }
}

// Trace: TC-080, FR-016-AC-6
#[test]
fn evaluator_has_no_derived_future_branch() {
    for kind in KINDS {
        for profile in PROFILES {
            let nodes = graph(
                &derived(kind, 0, 1, prop(0), prop(1)),
                profile,
                Lowering::TlSyntax,
            );
            let vocabulary: Vec<_> = nodes
                .iter()
                .map(|node| canonical_vocabulary(node.kind))
                .collect();
            let expected = match kind {
                Derived::WeakUntil => ["proposition", "proposition", "until", "globally", "or"],
                Derived::StrongRelease => {
                    ["proposition", "proposition", "release", "future", "and"]
                }
            };
            assert_eq!(vocabulary, expected);
        }
    }

    // Walk `src/` recursively so a derived branch in a new module directory
    // cannot hide below the top level.
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    let mut sources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                sources.push(path);
            }
        }
    }
    let mut scanned = 0;
    for path in sources {
        let text = fs::read_to_string(&path).unwrap();
        for needle in [
            "FutureKind",
            "FutureLowering",
            "FUTURE_OPERATORS",
            "FUTURE_LOWERING",
            "WeakUntil",
            "StrongRelease",
            "future-operators",
        ] {
            assert!(
                !text.contains(needle),
                "{} names the derived future vocabulary `{needle}`",
                path.display()
            );
        }
        scanned += 1;
    }
    assert_eq!(scanned, 8, "the evaluator source population changed");
}
