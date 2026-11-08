//! Evidence for the FR-027..034 matrix rows that need exhaustive or
//! property-style coverage (TC-142, TC-143, TC-144, TC-146, TC-152, TC-153,
//! TC-154, TC-156, TC-158). Every expectation is derived inside this file from
//! brute-force enumeration of Boolean completions or from a direct
//! ultimately-periodic-word evaluator; none of it calls back into the
//! provider's own completion machinery.
#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::{
    evaluate_lasso, evaluate_model, Disposition, EvaluationLimit, EvidenceBasis, EvidenceClosure,
    ExecutionDisposition, InfiniteError, InfiniteProvider, InfiniteResult, LassoRequest,
    ObservationValue as V, ProviderRegistry, ProviderRequest, ResultReason, SettlementEvidence,
    TruthAvailability,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, LassoTraceDocument, LivenessDisposition, LivenessSubject,
    LivenessSubjectKind, NodeId, PartialValuation, PropositionId, SemanticProfile,
    TemporalInterval, TraceObservation, UnboundedInterval, ValuationEntry,
};

const P: PropositionId = PropositionId(0);
const Q: PropositionId = PropositionId(1);

fn open(start: u32) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(start))
}
fn closed(start: u32, end: u32) -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(start, end).unwrap())
}

/// Incremental graph builder: nodes are appended and referenced by id.
struct Graph(Vec<InfiniteNode>);
impl Graph {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn add(&mut self, kind: K) -> NodeId {
        self.0.push(InfiniteNode::new(kind));
        NodeId(u32::try_from(self.0.len() - 1).unwrap())
    }
    fn prop(&mut self, p: PropositionId) -> NodeId {
        self.add(K::Proposition { proposition: p })
    }
    fn not(&mut self, operand: NodeId) -> NodeId {
        self.add(K::Not { operand })
    }
    fn done(self, root: NodeId) -> InfiniteFormulaDocument {
        InfiniteFormulaDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            root,
            self.0,
        )
        .unwrap()
    }
}

type Cell = [V; 2];

fn lasso(prefix: &[Cell], tail: &[Cell]) -> LassoTraceDocument {
    let props = [P, Q];
    let observation = |position: usize, cell: Cell| TraceObservation {
        position: u32::try_from(position).unwrap(),
        valuation: PartialValuation::new(
            "map".to_owned(),
            &props,
            props
                .iter()
                .zip(cell)
                .map(|(proposition, value)| ValuationEntry {
                    proposition: *proposition,
                    value,
                })
                .collect(),
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        props.to_vec(),
        prefix
            .iter()
            .enumerate()
            .map(|(i, c)| observation(i, *c))
            .collect(),
        tail.iter()
            .enumerate()
            .map(|(i, c)| observation(prefix.len() + i, *c))
            .collect(),
    )
    .unwrap()
}

fn eval(
    graph: &InfiniteFormulaDocument,
    word: &LassoTraceDocument,
    position: u64,
    fairness: Option<&FairnessPremisesDocument>,
    closure: EvidenceClosure,
) -> InfiniteResult {
    let graph_id = graph.content_identity().unwrap();
    let trace_id = word.content_identity().unwrap();
    evaluate_lasso(&LassoRequest {
        formula: graph,
        trace: word,
        fairness,
        evidence_closure: closure,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: position,
        limit: EvaluationLimit::default(),
    })
    .unwrap()
}

fn premises(graph: &InfiniteFormulaDocument, roots: Vec<NodeId>) -> FairnessPremisesDocument {
    FairnessPremisesDocument::new(
        graph,
        graph.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        roots,
    )
    .unwrap()
}

/// Cells of a lasso as `[prefix.., loop..]` rows.
fn rows(word: &LassoTraceDocument) -> Vec<Cell> {
    word.prefix()
        .iter()
        .chain(word.loop_observations())
        .map(|o| {
            let e = o.valuation.entries();
            [e[0].value, e[1].value]
        })
        .collect()
}

fn rebuild(word: &LassoTraceDocument, all: &[Cell]) -> LassoTraceDocument {
    let split = word.prefix().len();
    lasso(&all[..split], &all[split..])
}

fn unknown_cells(all: &[Cell]) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    for (row, cell) in all.iter().enumerate() {
        for (col, value) in cell.iter().enumerate() {
            if matches!(value, V::Missing | V::Conflicting) {
                cells.push((row, col));
            }
        }
    }
    cells
}

fn boolean(value: bool) -> V {
    if value {
        V::True
    } else {
        V::False
    }
}

/// Every Boolean completion of the unknown cells, as complete rows.
fn completions(all: &[Cell]) -> Vec<Vec<Cell>> {
    let unknown = unknown_cells(all);
    (0..1_u32 << unknown.len())
        .map(|mask| {
            let mut rows = all.to_vec();
            for (bit, (row, col)) in unknown.iter().enumerate() {
                rows[*row][*col] = boolean(mask >> bit & 1 == 1);
            }
            rows
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Independent direct evaluator on complete ultimately periodic words.
// ---------------------------------------------------------------------------

struct Word {
    values: Vec<[bool; 2]>,
    entry: usize,
}
impl Word {
    fn new(all: &[Cell], entry: usize) -> Self {
        Self {
            values: all
                .iter()
                .map(|c| [c[0] == V::True, c[1] == V::True])
                .collect(),
            entry,
        }
    }
    fn canon(&self, n: usize) -> usize {
        let len = self.values.len();
        if n < len {
            n
        } else {
            self.entry + (n - self.entry) % (len - self.entry)
        }
    }
    fn at(&self, n: usize, p: usize) -> bool {
        self.values[self.canon(n)][p]
    }
    /// One full period past `base` plus the prefix covers every distinct
    /// continuation of a search that starts at `base`.
    fn span(&self) -> usize {
        self.values.len() * 2 + 2
    }
    /// `l U[a,) r` at `i`: some `j >= i+a` has `r`, with `l` on `[i, j)`.
    fn until(
        &self,
        i: usize,
        a: usize,
        l: &dyn Fn(usize) -> bool,
        r: &dyn Fn(usize) -> bool,
    ) -> bool {
        (i + a..=i + a + self.span()).any(|j| r(j) && (i..j).all(l))
    }
    /// `l R[a,) r` is the Boolean dual of `until`.
    fn release(
        &self,
        i: usize,
        a: usize,
        l: &dyn Fn(usize) -> bool,
        r: &dyn Fn(usize) -> bool,
    ) -> bool {
        !self.until(i, a, &|n| !l(n), &|n| !r(n))
    }
}

fn short_lassos() -> Vec<LassoTraceDocument> {
    let cells: [Cell; 4] = [
        [V::False, V::False],
        [V::True, V::False],
        [V::False, V::True],
        [V::True, V::True],
    ];
    let mut out = Vec::new();
    for prefix_len in 0..=1_usize {
        for loop_len in 1..=3_usize {
            let n = prefix_len + loop_len;
            for code in 0..4_usize.pow(u32::try_from(n).unwrap()) {
                let all: Vec<Cell> = (0..n)
                    .map(|k| cells[code / 4_usize.pow(k as u32) % 4])
                    .collect();
                out.push(lasso(&all[..prefix_len], &all[prefix_len..]));
            }
        }
    }
    out
}

fn expect(result: &InfiniteResult, truth: bool, context: &str) {
    let expected = if truth {
        Disposition::Proved
    } else {
        Disposition::Refuted
    };
    assert_eq!(result.disposition, expected, "{context}");
}

// Trace: TC-146; FR-031-AC-1
#[test]
fn until_release_match_direct_word_oracle_induction_and_duality_with_nonzero_lower_bounds() {
    let mut checked = 0_u32;
    for a in [0_u32, 1, 2] {
        // Graph nodes: 0=p, 1=q, 2=U[a,)(p,q), 3=R[a,)(p,q), 4=U[a-1,)(p,q) (a>0),
        // 5=!p, 6=!q, 7=U[a,)(!p,!q), 8=!7.
        let mut g = Graph::new();
        let p = g.prop(P);
        let q = g.prop(Q);
        let until = g.add(K::Until {
            interval: open(a),
            left: p,
            right: q,
        });
        let release = g.add(K::Release {
            interval: open(a),
            left: p,
            right: q,
        });
        let np = g.not(p);
        let nq = g.not(q);
        let dual_until = g.add(K::Until {
            interval: open(a),
            left: np,
            right: nq,
        });
        let dual = g.not(dual_until);
        let lower = if a > 0 {
            Some(g.add(K::Until {
                interval: open(a - 1),
                left: p,
                right: q,
            }))
        } else {
            None
        };
        let base = g.0.clone();
        let rebuild_with = |root: NodeId| {
            InfiniteFormulaDocument::new(
                SemanticProfile::InfiniteTraceV1,
                InfiniteClock::EventPosition,
                root,
                base.clone(),
            )
            .unwrap()
        };
        let u_doc = rebuild_with(until);
        let r_doc = rebuild_with(release);
        let d_doc = rebuild_with(dual);
        let l_doc = lower.map(rebuild_with);
        for word_doc in short_lassos() {
            let all = rows(&word_doc);
            let w = Word::new(&all, word_doc.loop_entry());
            for i in 0..all.len() + 3 {
                let lp = |n: usize| w.at(n, 0);
                let rq = |n: usize| w.at(n, 1);
                let a_us = a as usize;
                let want_u = w.until(i, a_us, &lp, &rq);
                let want_r = w.release(i, a_us, &lp, &rq);
                let ctx = format!("a={a} i={i} word={all:?}");
                let got_u = eval(&u_doc, &word_doc, i as u64, None, EvidenceClosure::Closed);
                expect(&got_u, want_u, &format!("U {ctx}"));
                let got_r = eval(&r_doc, &word_doc, i as u64, None, EvidenceClosure::Closed);
                expect(&got_r, want_r, &format!("R {ctx}"));
                // Duality: R == !U(!p, !q), evaluated by the provider itself.
                let got_d = eval(&d_doc, &word_doc, i as u64, None, EvidenceClosure::Closed);
                assert_eq!(got_r.disposition, got_d.disposition, "dual {ctx}");
                // Induction on the lower bound: U[a,) at i == p_i && U[a-1,) at i+1.
                if let Some(doc) = &l_doc {
                    let next = eval(
                        doc,
                        &word_doc,
                        (i + 1) as u64,
                        None,
                        EvidenceClosure::Closed,
                    );
                    expect(
                        &got_u,
                        w.at(i, 0) && next.disposition == Disposition::Proved,
                        &format!("induction {ctx}"),
                    );
                } else {
                    // a = 0: U at i == q_i || (p_i && U at i+1)
                    let next = eval(
                        &u_doc,
                        &word_doc,
                        (i + 1) as u64,
                        None,
                        EvidenceClosure::Closed,
                    );
                    expect(
                        &got_u,
                        w.at(i, 1) || (w.at(i, 0) && next.disposition == Disposition::Proved),
                        &format!("induction0 {ctx}"),
                    );
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 5_000, "checked {checked}");
}

/// Graph set shared by the possibility rows.
fn possibility_graphs() -> Vec<(&'static str, InfiniteFormulaDocument)> {
    let mut out = Vec::new();
    let mut add = |name, build: &dyn Fn(&mut Graph) -> NodeId| {
        let mut g = Graph::new();
        let root = build(&mut g);
        out.push((name, g.done(root)));
    };
    add("p", &|g| g.prop(P));
    add("p&!p", &|g| {
        let p = g.prop(P);
        let n = g.not(p);
        g.add(K::And { left: p, right: n })
    });
    add("p<->Yp", &|g| {
        let p = g.prop(P);
        let y = g.add(K::StrongPrevious { operand: p });
        g.add(K::Equivalent { left: p, right: y })
    });
    add("F[0,)p", &|g| {
        let p = g.prop(P);
        g.add(K::Future {
            interval: open(0),
            operand: p,
        })
    });
    add("G[1,)p", &|g| {
        let p = g.prop(P);
        g.add(K::Globally {
            interval: open(1),
            operand: p,
        })
    });
    add("p U[0,) q", &|g| {
        let p = g.prop(P);
        let q = g.prop(Q);
        g.add(K::Until {
            interval: open(0),
            left: p,
            right: q,
        })
    });
    add("p R[2,) q", &|g| {
        let p = g.prop(P);
        let q = g.prop(Q);
        g.add(K::Release {
            interval: open(2),
            left: p,
            right: q,
        })
    });
    add("O[0,2]p", &|g| {
        let p = g.prop(P);
        g.add(K::Once {
            interval: closed(0, 2),
            operand: p,
        })
    });
    add("p S[0,) q", &|g| {
        let p = g.prop(P);
        let q = g.prop(Q);
        g.add(K::Since {
            interval: open(0),
            left: p,
            right: q,
        })
    });
    out
}

fn partial_words() -> Vec<LassoTraceDocument> {
    use V::{Conflicting as C, False as F, Missing as M, True as T};
    vec![
        lasso(&[], &[[M, F]]),
        lasso(&[], &[[C, T]]),
        lasso(&[], &[[M, M]]),
        lasso(&[], &[[M, T], [F, C]]),
        lasso(&[[M, F]], &[[T, C], [F, F]]),
        lasso(&[[C, M], [F, T]], &[[M, F]]),
        lasso(&[[T, F]], &[[F, T], [T, F]]),
    ]
}

/// Brute-force settlement of a partial lasso: evaluate every Boolean
/// completion, admit the ones whose fairness premise `p` (node 0 of a graph
/// whose node 0 is `p`) is true at some loop position by direct inspection of
/// the cells, and fold.
fn brute_force(
    graph: &InfiniteFormulaDocument,
    word: &LassoTraceDocument,
    position: u64,
    fair_on_p: bool,
) -> (Disposition, u64) {
    let all = rows(word);
    let entry = word.loop_entry();
    let mut admitted = 0_u64;
    let (mut proved, mut refuted) = (0_u64, 0_u64);
    for total in completions(&all) {
        if fair_on_p && !total[entry..].iter().any(|c| c[0] == V::True) {
            continue;
        }
        admitted += 1;
        let done = rebuild(word, &total);
        match eval(graph, &done, position, None, EvidenceClosure::Closed).disposition {
            Disposition::Proved => proved += 1,
            Disposition::Refuted => refuted += 1,
            other => panic!("complete trace settled {other:?}"),
        }
    }
    let disposition = if admitted == 0 || (proved > 0 && refuted > 0) {
        Disposition::Inconclusive
    } else if proved > 0 {
        Disposition::Proved
    } else {
        Disposition::Refuted
    };
    (disposition, admitted)
}

// Trace: TC-142, TC-144; FR-030-AC-1, FR-030-AC-3
#[test]
fn partial_cells_yield_exactly_the_common_completion_possibilities() {
    let mut checked = 0;
    for (name, graph) in possibility_graphs() {
        for word in partial_words() {
            let all = rows(&word);
            for position in 0..all.len() as u64 + 2 {
                let (expected, admitted) = brute_force(&graph, &word, position, false);
                let got = eval(&graph, &word, position, None, EvidenceClosure::Closed);
                let ctx = format!("{name} pos={position} word={all:?}");
                assert_eq!(got.disposition, expected, "{ctx}");
                assert_eq!(got.admitted_completions, admitted, "{ctx}");
                assert_eq!(
                    admitted,
                    1_u64 << unknown_cells(&all).len(),
                    "one completion per assignment to the distinct lasso cells: {ctx}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 60, "{checked}");
}

// Trace: TC-143; FR-030-AC-2
#[test]
fn missing_and_conflicting_share_possibilities_but_keep_reason_and_closure_and_never_flip() {
    let mut conclusive_checked = 0;
    for (name, graph) in possibility_graphs() {
        for word in partial_words() {
            let all = rows(&word);
            let unknown = unknown_cells(&all);
            for position in 0..all.len() as u64 + 2 {
                let base = eval(&graph, &word, position, None, EvidenceClosure::Closed);
                // Swap every unknown cell to the other unknown kind.
                let mut swapped = all.clone();
                for (row, col) in &unknown {
                    swapped[*row][*col] = if swapped[*row][*col] == V::Missing {
                        V::Conflicting
                    } else {
                        V::Missing
                    };
                }
                let other = eval(
                    &graph,
                    &rebuild(&word, &swapped),
                    position,
                    None,
                    EvidenceClosure::Closed,
                );
                assert_eq!(other.disposition, base.disposition, "{name} {all:?}");
                assert_eq!(other.admitted_completions, base.admitted_completions);
                // Closure: the same inconclusive evidence is indeterminate when closed
                // and pending when progress is declared.
                if base.disposition == Disposition::Inconclusive {
                    let progress = eval(
                        &graph,
                        &word,
                        position,
                        None,
                        EvidenceClosure::ProgressDeclared,
                    );
                    assert_eq!(base.basis, EvidenceBasis::Indeterminate, "{name} {all:?}");
                    assert_eq!(progress.basis, EvidenceBasis::Pending, "{name} {all:?}");
                    assert_eq!(base.reason, progress.reason);
                }
                // Monotonicity: resolving any one unknown cell to a Boolean never
                // flips a conclusive result.
                if matches!(base.disposition, Disposition::Proved | Disposition::Refuted) {
                    for (row, col) in &unknown {
                        for value in [true, false] {
                            let mut refined = all.clone();
                            refined[*row][*col] = boolean(value);
                            let more = eval(
                                &graph,
                                &rebuild(&word, &refined),
                                position,
                                None,
                                EvidenceClosure::Closed,
                            );
                            assert_eq!(
                                more.disposition, base.disposition,
                                "{name} {all:?} -> {refined:?}"
                            );
                        }
                    }
                    conclusive_checked += 1;
                }
            }
        }
    }
    assert!(conclusive_checked > 20, "{conclusive_checked}");

    // Reasons differ for equal possibility sets.
    let mut g = Graph::new();
    let p = g.prop(P);
    let graph = g.done(p);
    let missing = eval(
        &graph,
        &lasso(&[], &[[V::Missing, V::False]]),
        0,
        None,
        EvidenceClosure::Closed,
    );
    let conflict = eval(
        &graph,
        &lasso(&[], &[[V::Conflicting, V::False]]),
        0,
        None,
        EvidenceClosure::Closed,
    );
    assert_eq!(missing.admitted_completions, conflict.admitted_completions);
    assert_eq!(missing.reason, Some(ResultReason::MissingObservation));
    assert_eq!(conflict.reason, Some(ResultReason::ConflictingObservation));

    // An empty possibility set is unrepresentable on the wire: a lasso with no
    // observations is refused at admission, and an empty fair admission is a
    // typed inconclusive cause with no proof or refutation evidence.
    assert!(LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        vec![P, Q],
        Vec::new(),
        Vec::new(),
    )
    .is_err());
    let fair = premises(&graph, vec![p]);
    let empty = eval(
        &graph,
        &lasso(&[], &[[V::False, V::False]]),
        0,
        Some(&fair),
        EvidenceClosure::Closed,
    );
    assert_eq!(empty.reason, Some(ResultReason::EmptyFairAdmission));
    assert_eq!(empty.admitted_completions, 0);
    assert!(empty.evidence.is_none());
    assert_ne!(empty.disposition, Disposition::Proved);
    assert_ne!(empty.disposition, Disposition::Refuted);
}

// Trace: TC-144; FR-030-AC-3
#[test]
fn foreign_proposition_map_clock_and_profile_refuse_before_evaluation() {
    let mut g = Graph::new();
    let p = g.prop(P);
    let graph = g.done(p);
    let word = lasso(&[], &[[V::True, V::False]]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = word.content_identity().unwrap();
    let request = |graph_id: &str, trace_id: &str| {
        evaluate_lasso(&LassoRequest {
            formula: &graph,
            trace: &word,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id,
            trace_id,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        })
    };
    assert!(request(&graph_id, &trace_id).is_ok());
    assert_eq!(
        request(&graph_id, "foreign-map-or-trace"),
        Err(InfiniteError::IdentityMismatch)
    );
    assert_eq!(
        request("foreign-graph", &trace_id),
        Err(InfiniteError::IdentityMismatch)
    );

    // A trace whose proposition map omits the graph's proposition refuses.
    let mut g2 = Graph::new();
    let outside = g2.prop(PropositionId(9));
    let foreign_graph = g2.done(outside);
    let foreign_id = foreign_graph.content_identity().unwrap();
    assert!(evaluate_lasso(&LassoRequest {
        formula: &foreign_graph,
        trace: &word,
        fairness: None,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &foreign_id,
        trace_id: &trace_id,
        selected_position: 0,
        limit: EvaluationLimit::default(),
    })
    .is_err());

    // The only V1 clock is event_position and the only admitted profile is
    // infinite-trace/v1: a closed-trace profile cannot build either document.
    assert!(InfiniteFormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        InfiniteClock::EventPosition,
        NodeId(0),
        vec![InfiniteNode::new(K::True)],
    )
    .is_err());
    assert!(LassoTraceDocument::new(
        SemanticProfile::ClosedTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        vec![P],
        Vec::new(),
        Vec::new(),
    )
    .is_err());
}

// Trace: TC-152, TC-153; FR-032-AC-2
#[test]
fn fairness_admission_matches_brute_force_and_empty_set_admits_everything() {
    let mut checked = 0;
    for (name, graph) in possibility_graphs() {
        let none = None;
        let empty = premises(&graph, Vec::new());
        // Premise: node 0, which is `p` in every graph above.
        let on_p = premises(&graph, vec![NodeId(0)]);
        for word in partial_words() {
            let all = rows(&word);
            for position in 0..all.len() as u64 + 1 {
                let ctx = format!("{name} pos={position} {all:?}");
                // Empty premise set == no premise document, and admits every completion.
                let a = eval(&graph, &word, position, none, EvidenceClosure::Closed);
                let b = eval(
                    &graph,
                    &word,
                    position,
                    Some(&empty),
                    EvidenceClosure::Closed,
                );
                assert_eq!(a.disposition, b.disposition, "{ctx}");
                assert_eq!(a.admitted_completions, b.admitted_completions, "{ctx}");
                assert_eq!(a.admitted_completions, 1_u64 << unknown_cells(&all).len());
                // Fairness on `p`: brute-force filter on "p true at some loop position".
                let fair = eval(
                    &graph,
                    &word,
                    position,
                    Some(&on_p),
                    EvidenceClosure::Closed,
                );
                let (expected, admitted) = brute_force(&graph, &word, position, true);
                assert_eq!(fair.disposition, expected, "{ctx}");
                assert_eq!(fair.admitted_completions, admitted, "{ctx}");
                if admitted == 0 {
                    assert_eq!(fair.reason, Some(ResultReason::EmptyFairAdmission), "{ctx}");
                    assert!(fair.evidence.is_none());
                }
                // Weakening (dropping) the premise never removes an admitted trace.
                assert!(a.admitted_completions >= fair.admitted_completions, "{ctx}");
                checked += 1;
            }
        }
    }
    assert!(checked >= 60, "{checked}");
}

// Trace: TC-154; FR-032-AC-3
#[test]
fn malformed_and_foreign_fairness_and_lasso_identities_refuse_without_a_result() {
    let mut g = Graph::new();
    let p = g.prop(P);
    let graph = g.done(p);
    let word = lasso(&[], &[[V::True, V::False]]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = word.content_identity().unwrap();

    // Fairness root outside the graph is refused when the premise document is built.
    assert!(FairnessPremisesDocument::new(
        &graph,
        graph_id.clone(),
        InfiniteClock::EventPosition,
        vec![NodeId(99)],
    )
    .is_err());
    // Fairness built for a foreign graph refuses at the provider.
    let mut other = Graph::new();
    let x = other.prop(P);
    let y = other.not(x);
    let other_graph = other.done(y);
    let foreign = premises(&other_graph, vec![NodeId(0)]);
    let refusal = evaluate_lasso(&LassoRequest {
        formula: &graph,
        trace: &word,
        fairness: Some(&foreign),
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: 0,
        limit: EvaluationLimit::default(),
    });
    assert_eq!(refusal, Err(InfiniteError::IdentityMismatch));
    // A malformed lasso (empty loop) cannot be admitted.
    assert!(LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        vec![P, Q],
        Vec::new(),
        Vec::new(),
    )
    .is_err());
    // Unfair trace: empty fair admission, neither proof nor refutation.
    let fair = premises(&graph, vec![p]);
    let unfair = lasso(&[[V::True, V::False]], &[[V::False, V::False]]);
    let result = eval(&graph, &unfair, 0, Some(&fair), EvidenceClosure::Closed);
    assert_eq!(result.disposition, Disposition::Inconclusive);
    assert_eq!(result.reason, Some(ResultReason::EmptyFairAdmission));
    assert!(result.evidence.is_none());
    // Registry route: a foreign subject identity yields a refusal, never a partial result.
    let provider = InfiniteProvider {
        request: ProviderRequest::Lasso(LassoRequest {
            formula: &graph,
            trace: &word,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        }),
    };
    let mut registry = ProviderRegistry::default();
    registry.register(&provider).unwrap();
    let subject = LivenessSubject {
        kind: LivenessSubjectKind::LassoTrace,
        identity: "foreign",
    };
    assert!(registry.settle_detailed(&graph, subject).is_err());
}

// Trace: TC-156; FR-033-AC-1
#[test]
fn witness_and_counterexample_details_replay_against_the_full_admitted_loop() {
    let mut replayed = 0;
    for (name, graph) in possibility_graphs() {
        let on_p = premises(&graph, vec![NodeId(0)]);
        for word in partial_words() {
            let all = rows(&word);
            let unknown = unknown_cells(&all);
            for position in 0..all.len() as u64 + 1 {
                for fairness in [None, Some(&on_p)] {
                    let result = eval(&graph, &word, position, fairness, EvidenceClosure::Closed);
                    let ctx = format!("{name} pos={position} fair={} {all:?}", fairness.is_some());
                    let Some(SettlementEvidence::ExhaustiveTrace(evidence)) = &result.evidence
                    else {
                        assert!(
                            result.admitted_completions == 0
                                || result.disposition == Disposition::Failed,
                            "{ctx}"
                        );
                        continue;
                    };
                    assert_eq!(
                        evidence.all_admitted_fair_completions, result.admitted_completions,
                        "{ctx}"
                    );
                    for (example, want) in [
                        (&evidence.satisfying, Disposition::Proved),
                        (&evidence.falsifying, Disposition::Refuted),
                    ] {
                        let Some(example) = example else { continue };
                        // The example assigns exactly the unknown cells, once each.
                        let mut cells: Vec<(usize, usize)> = example
                            .assignments
                            .iter()
                            .map(|a| (a.position as usize, usize::from(a.proposition == Q)))
                            .collect();
                        cells.sort_unstable();
                        assert_eq!(cells, unknown, "{ctx}");
                        // Replaying it on the full loop settles to the claimed side and
                        // satisfies the fairness premise (p true in the loop).
                        let mut total = all.clone();
                        for a in &example.assignments {
                            let col = usize::from(a.proposition == Q);
                            total[a.position as usize][col] = boolean(a.value);
                        }
                        let entry = word.loop_entry();
                        if fairness.is_some() {
                            assert!(total[entry..].iter().any(|c| c[0] == V::True), "{ctx}");
                        }
                        let done = rebuild(&word, &total);
                        let replay = eval(&graph, &done, position, None, EvidenceClosure::Closed);
                        assert_eq!(replay.disposition, want, "{ctx}");
                        replayed += 1;
                    }
                    // Side presence agrees with the disposition.
                    match result.disposition {
                        Disposition::Proved => {
                            assert!(
                                evidence.satisfying.is_some() && evidence.falsifying.is_none(),
                                "{ctx}"
                            )
                        }
                        Disposition::Refuted => {
                            assert!(
                                evidence.falsifying.is_some() && evidence.satisfying.is_none(),
                                "{ctx}"
                            )
                        }
                        Disposition::Inconclusive => {
                            assert!(
                                evidence.satisfying.is_some() && evidence.falsifying.is_some(),
                                "{ctx}"
                            )
                        }
                        other => panic!("{ctx}: {other:?}"),
                    }
                }
            }
        }
    }
    assert!(replayed > 300, "{replayed}");
}

// Trace: TC-158; FR-033-AC-3
#[test]
fn unsupported_and_both_failed_dispositions_keep_exact_axes_through_the_public_routes() {
    let mut g = Graph::new();
    let p = g.prop(P);
    let graph = g.done(p);
    let word = lasso(&[], &[[V::True, V::False]]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = word.content_identity().unwrap();

    // unsupported / unsupported: model subject.
    let model = evaluate_model(&graph, &graph_id, "model", "map").unwrap();
    assert_eq!(
        (
            model.disposition,
            model.execution,
            model.truth,
            model.basis,
            model.reason
        ),
        (
            Disposition::Unsupported,
            ExecutionDisposition::Unsupported,
            TruthAvailability::Unavailable,
            EvidenceBasis::Unavailable,
            Some(ResultReason::ModelProcedureUnavailable),
        )
    );
    assert!(model.evidence.is_none());

    // failed / resource-incomplete: each limit axis one under need, and an expired deadline.
    let tight = [
        EvaluationLimit {
            max_nodes: 0,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            max_positions: 0,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            max_valuation_cells: 0,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            max_steps: 0,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            deadline: Some(std::time::Instant::now()),
            ..EvaluationLimit::default()
        },
    ];
    for limit in tight {
        let result = evaluate_lasso(&LassoRequest {
            formula: &graph,
            trace: &word,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: 0,
            limit,
        })
        .unwrap();
        assert_eq!(
            (
                result.disposition,
                result.execution,
                result.truth,
                result.basis,
                result.reason
            ),
            (
                Disposition::Failed,
                ExecutionDisposition::ResourceIncomplete,
                TruthAvailability::Unavailable,
                EvidenceBasis::Unavailable,
                Some(ResultReason::ResourceIncomplete),
            ),
            "{limit:?}"
        );
        assert!(result.evidence.is_none());
        // The coarse tl-syntax route never turns it into a Boolean verdict.
        let provider = InfiniteProvider {
            request: ProviderRequest::Lasso(LassoRequest {
                formula: &graph,
                trace: &word,
                fairness: None,
                evidence_closure: EvidenceClosure::Closed,
                graph_id: &graph_id,
                trace_id: &trace_id,
                selected_position: 0,
                limit,
            }),
        };
        let mut registry = ProviderRegistry::default();
        registry.register(&provider).unwrap();
        let subject = LivenessSubject {
            kind: LivenessSubjectKind::LassoTrace,
            identity: &trace_id,
        };
        assert_eq!(
            registry.settle(&graph, subject).disposition,
            LivenessDisposition::ResourceIncomplete
        );
    }
    // failed / failed (provider fault) is reachable only through an internal
    // invariant violation; its exact axes are asserted by the in-module test
    // `provider_fault_and_resource_exhaustion_have_distinct_execution_axes`
    // (TC-158), which this test complements for the public routes.
}
