use std::{collections::BTreeMap, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use sha2::{Digest, Sha256};
use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, LassoRequest,
};
use tl_mltl::{
    evaluate_closed, evaluate_prefix, map_to_c2po, EvaluationLimits, MappingSourceIdentity,
    MappingSourceState, TruthValue,
};
use tl_syntax::{
    FairnessPremisesDocument, Formula, FormulaDocument, InfiniteClock, InfiniteFormulaDocument,
    InfiniteNode, InfiniteNodeKind, Interval, LassoTraceDocument, Node, NodeId, NodeKind,
    PartialValuation, PartialValue, PropositionId, SemanticProfile, TemporalInterval,
    TraceObservation, UnboundedInterval, ValuationEntry,
};

const P: PropositionId = PropositionId(0);
const DIGESTS: &str = include_str!("input-digests.json");

fn digest(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("{:x}", hasher.finalize())
}

fn finite(size: usize, profile: SemanticProfile) -> FormulaDocument {
    FormulaDocument::new(
        profile,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition { proposition: P }),
            Node::new(NodeKind::Future {
                interval: Interval::new(0, u32::try_from(size - 1).unwrap()).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
    .unwrap()
}

fn observations(size: usize) -> Vec<Vec<PropositionId>> {
    let mut trace = vec![Vec::new(); size];
    trace[size - 1].push(P);
    trace
}

fn borrowed(document: &FormulaDocument) -> Formula<'_> {
    Formula::new(
        document.semantic_profile(),
        document.root(),
        document.nodes(),
    )
    .unwrap()
}

fn run_finite(
    document: &FormulaDocument,
    trace: &[Vec<PropositionId>],
    closed: bool,
) -> TruthValue {
    let limits = EvaluationLimits {
        max_temporal_span: trace.len() as u64,
        ..EvaluationLimits::default()
    };
    let result = if closed {
        evaluate_closed(borrowed(document), "v9", trace, "v9-trace", limits)
    } else {
        evaluate_prefix(borrowed(document), "v9", trace, "v9-trace", true, limits)
    }
    .unwrap();
    result.verdict
}

fn unbounded() -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(0))
}

fn infinite(fair: bool) -> InfiniteFormulaDocument {
    let mut nodes = vec![InfiniteNode::new(InfiniteNodeKind::Proposition {
        proposition: P,
    })];
    if fair {
        nodes.push(InfiniteNode::new(InfiniteNodeKind::Globally {
            interval: unbounded(),
            operand: NodeId(0),
        }));
    }
    let root = NodeId(u32::try_from(nodes.len()).unwrap());
    nodes.push(InfiniteNode::new(InfiniteNodeKind::Future {
        interval: unbounded(),
        operand: NodeId(0),
    }));
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        root,
        nodes,
    )
    .unwrap()
}

fn lasso(size: usize, fair: bool) -> LassoTraceDocument {
    let values = (0..size)
        .map(|position| TraceObservation {
            position: u32::try_from(position).unwrap(),
            valuation: PartialValuation::new(
                "v9-map".to_owned(),
                &[P],
                vec![ValuationEntry {
                    proposition: P,
                    value: if fair || position == size - 1 {
                        PartialValue::True
                    } else {
                        PartialValue::False
                    },
                }],
            )
            .unwrap(),
        })
        .collect::<Vec<_>>();
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "v9-map".to_owned(),
        vec![P],
        values[..size - 1].to_vec(),
        values[size - 1..].to_vec(),
    )
    .unwrap()
}

fn run_infinite(
    graph: &InfiniteFormulaDocument,
    trace: &LassoTraceDocument,
    fairness: Option<&FairnessPremisesDocument>,
) -> Disposition {
    let graph_id = graph.content_identity().unwrap();
    let trace_id = trace.content_identity().unwrap();
    evaluate_lasso(&LassoRequest {
        formula: graph,
        trace,
        fairness,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: 0,
        limit: EvaluationLimit {
            max_nodes: graph.nodes().len(),
            max_positions: trace.prefix().len() + trace.loop_observations().len(),
            max_valuation_cells: trace.prefix().len() + trace.loop_observations().len(),
            max_states: graph.nodes().len()
                * (trace.prefix().len() + trace.loop_observations().len()),
            max_completions: 1,
            ..EvaluationLimit::default()
        },
    })
    .unwrap()
    .disposition
}

fn mapping(size: usize) -> FormulaDocument {
    let mut nodes = vec![Node::new(NodeKind::Proposition { proposition: P })];
    let mut root = NodeId(0);
    for _ in 0..size {
        root = NodeId(u32::try_from(nodes.len()).unwrap());
        nodes.push(Node::new(NodeKind::Future {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(root.0 - 1),
        }));
    }
    FormulaDocument::new(SemanticProfile::OnlinePrefixV1, root, nodes).unwrap()
}

fn run_mapping(document: &FormulaDocument) -> usize {
    map_to_c2po(
        borrowed(document),
        "v9-map",
        b"v9-formula",
        MappingSourceIdentity {
            revision: "v9-source".to_owned(),
            state: MappingSourceState::Clean,
        },
        None,
        document.nodes().len() as u64,
    )
    .unwrap()
    .expression
    .len()
}

fn workloads(c: &mut Criterion) {
    let expected: BTreeMap<String, String> = serde_json::from_str(DIGESTS).unwrap();
    assert_eq!(expected.len(), 15, "V9 workload census changed");
    let mut group = c.benchmark_group("v9_workloads");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    for (scale, size) in [("small", 2), ("median", 24), ("near_cap", 96)] {
        let trace = observations(size);
        let trace_wire = serde_json::to_vec(&trace).unwrap();
        for (family, profile, closed) in [
            ("closed", SemanticProfile::ClosedTraceV1, true),
            ("prefix", SemanticProfile::OnlinePrefixV1, false),
        ] {
            let name = format!("{family}_{scale}");
            let document = finite(size, profile);
            let wire = document.canonical_json_bytes().unwrap();
            let actual = digest(&[name.as_bytes(), &wire, &trace_wire]);
            assert_eq!(actual, expected[&name], "V9 input changed: {name}");
            assert_eq!(run_finite(&document, &trace, closed), TruthValue::True);
            group.throughput(Throughput::Elements(size as u64));
            group.bench_function(&name, |b| {
                b.iter(|| black_box(run_finite(black_box(&document), black_box(&trace), closed)));
            });
        }
        for fair in [false, true] {
            let family = if fair { "fairness" } else { "lasso" };
            let name = format!("{family}_{scale}");
            let graph = infinite(fair);
            let trace = lasso(size, fair);
            let fairness = fair.then(|| {
                FairnessPremisesDocument::new(
                    &graph,
                    graph.content_identity().unwrap(),
                    InfiniteClock::EventPosition,
                    vec![NodeId(1)],
                )
                .unwrap()
            });
            let graph_wire = graph.canonical_json_bytes().unwrap();
            let trace_wire = trace.canonical_json_bytes().unwrap();
            let fairness_wire = fairness
                .as_ref()
                .map(|value| value.canonical_json_bytes().unwrap())
                .unwrap_or_default();
            let actual = digest(&[name.as_bytes(), &graph_wire, &trace_wire, &fairness_wire]);
            assert_eq!(actual, expected[&name], "V9 input changed: {name}");
            assert_eq!(
                run_infinite(&graph, &trace, fairness.as_ref()),
                Disposition::Proved
            );
            group.throughput(Throughput::Elements(size as u64));
            group.bench_function(&name, |b| {
                b.iter(|| {
                    black_box(run_infinite(
                        black_box(&graph),
                        black_box(&trace),
                        fairness.as_ref(),
                    ))
                });
            });
        }
        let name = format!("c2po_{scale}");
        let document = mapping(size);
        let wire = document.canonical_json_bytes().unwrap();
        let actual = digest(&[name.as_bytes(), &wire]);
        assert_eq!(actual, expected[&name], "V9 input changed: {name}");
        assert!(run_mapping(&document) > 0);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(&name, |b| {
            b.iter(|| black_box(run_mapping(black_box(&document))));
        });
    }
    group.finish();
}

criterion_group!(benches, workloads);
criterion_main!(benches);
