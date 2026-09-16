use agent_ix_baseline_producer::StaticProducerBundle;
use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, QualifiedSubject,
    ResourceLimits, ScopeKind, ScopeSelection, SubjectIdentity, SubjectKind, ValueState,
    Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use tl_mltl::mapping::contract_ir::{self, MappedOutcome, MappingSelection, NonValueKind};
use tl_mltl::past::{history, requirement, result};
use tl_mltl::wire::{command, observation, report, request, trace};
use tl_mltl::wire::{OwnerLimits, OwnerReadErrorCode};
use tl_mltl::{
    analyze_required_history, fixed_sample_instant, ClockBinding, ClockSample, CommandDocument,
    CommandSchemaVersion, ExactNumber, Operation, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TraceDocument,
    TraceSchemaVersion,
};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionEntry, PropositionId,
    PropositionMapDocument, SemanticProfile,
};

struct OwnerViews {
    clock: authority::clock::View,
    progress_open: authority::progress::View,
    progress_closed: authority::progress::View,
    closure_open: authority::closure::View,
    closure_closed: authority::closure::View,
    completeness_complete: authority::completeness::View,
    completeness_incomplete: authority::completeness::View,
    completeness_contradicted: authority::completeness::View,
    availability_available: authority::availability::View,
    availability_not_yet: authority::availability::View,
    availability_producer_unavailable: authority::availability::View,
    availability_contract_unavailable: authority::availability::View,
}

#[derive(Clone, Copy)]
enum FixtureClock {
    EventPosition,
    FixedSample,
}

impl FixtureClock {
    const fn identity(self) -> &'static str {
        match self {
            Self::EventPosition => "clock:event-position",
            Self::FixedSample => "clock:fixed-sample",
        }
    }

    const fn range(self, positions: u64) -> ClockRange {
        match self {
            Self::EventPosition => ClockRange::EventPosition {
                start: 0,
                end_exclusive: positions,
            },
            Self::FixedSample => ClockRange::FixedSample {
                start: 0,
                end_exclusive: positions,
                epoch_nanos: 100,
                period_nanos: 10,
            },
        }
    }

    const fn anchor(self) -> Anchor {
        match self {
            Self::EventPosition => Anchor::EventPosition(0),
            Self::FixedSample => Anchor::FixedSample {
                index: 0,
                epoch_nanos: 100,
                period_nanos: 10,
            },
        }
    }

    fn boundary(self, state: OpenClosed, positions: u64) -> TemporalBoundary {
        let watermark = if state == OpenClosed::Closed {
            positions
        } else {
            positions - 1
        };
        match self {
            Self::EventPosition => TemporalBoundary::EventPosition {
                lower: 0,
                upper_inclusive: positions - 1,
                carrier_end_exclusive: positions,
                watermark,
            },
            Self::FixedSample => TemporalBoundary::FixedSample {
                lower: 0,
                upper_inclusive: positions - 1,
                carrier_end_exclusive: positions,
                watermark,
            },
        }
    }
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn qualified_with_clock(
    tag: &str,
    positions: u64,
    fixture_clock: FixtureClock,
) -> Box<QualifiedObservation> {
    let producer =
        StaticProducerBundle::admit_json(include_bytes!("fixtures/fcd-static-bundle-1.2.json"))
            .unwrap();
    let subject = QualifiedSubject::new(
        &producer,
        SubjectKind::new("ix://agent-ix/commerce/type/Order").unwrap(),
        SubjectIdentity::new(format!("order:{tag}")).unwrap(),
    );
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        producer,
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id("signal:p0"),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("boolean"),
            subject_kind: subject.kind().clone(),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: vec![],
            required_member_identities: vec![id(format!("member:{tag}"))],
            observation_sources: vec![id(format!("source:{tag}"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}"))],
            progress_dependencies: vec![id(format!("progress:{tag}"))],
            clock_identity: id(fixture_clock.identity()),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Snapshot {
                snapshot_identity: id(format!("snapshot:{tag}")),
            },
            range: fixture_clock.range(positions),
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: fixture_clock.anchor(),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject,
            signal_identity: id("signal:p0"),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("boolean"),
            value: ValueState::Present {
                value_type: id("boolean"),
                canonical_value: "true".to_owned(),
            },
            visibility: Visibility::External,
            anchor: fixture_clock.anchor(),
            event_time_nanos: 0,
            ingestion_time_nanos: 1,
            causal_relationship_identity: None,
            clock_identity: id(fixture_clock.identity()),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        ObservationLimits::owner_max(),
    )
    .unwrap();
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .unwrap();
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("observation admission failed: {other:?}"),
    }
}

fn owner_views(tag: &str, positions: u64) -> OwnerViews {
    owner_views_with_clock(tag, positions, FixtureClock::EventPosition)
}

fn owner_views_with_clock(tag: &str, positions: u64, fixture_clock: FixtureClock) -> OwnerViews {
    let qualified = qualified_with_clock(tag, positions, fixture_clock);
    let owner = AuthoritySelection {
        definition_identity: id(format!("definition:{tag}")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    };
    let subject = SubjectSelection {
        scope_identity: id(format!("snapshot:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let clock_selection = authority::clock::Selection::new(id(fixture_clock.identity()), id("1"));
    let clock_document =
        authority::clock::derive(context, &clock_selection, ObservationLimits::owner_max())
            .unwrap();
    let clock = authority::clock::read(
        clock_document.bytes(),
        context,
        &clock_selection,
        ObservationLimits::owner_max(),
    )
    .unwrap();

    let progress = |state| {
        let selection = authority::progress::Selection::new(
            clock_selection.clone(),
            vec![id(format!("source:{tag}"))],
            fixture_clock.boundary(state, positions),
            state,
            id(format!("trigger:{tag}")),
            cutoff(fixture_clock),
            id(format!("restoration:{tag}")),
        );
        let document =
            authority::progress::derive(context, &selection, ObservationLimits::owner_max())
                .unwrap();
        authority::progress::read(
            document.bytes(),
            context,
            &selection,
            ObservationLimits::owner_max(),
        )
        .unwrap()
    };
    let closure = |state| {
        let selection = authority::closure::Selection::new(
            id(fixture_clock.identity()),
            id("1"),
            vec![id(format!("source:{tag}"))],
            fixture_clock.boundary(state, positions),
            state,
        );
        let document =
            authority::closure::derive(context, &selection, ObservationLimits::owner_max())
                .unwrap();
        authority::closure::read(
            document.bytes(),
            context,
            &selection,
            ObservationLimits::owner_max(),
        )
        .unwrap()
    };
    let completeness = |status: authority::completeness::FactStatus, observed: bool| {
        let selection = authority::completeness::Selection::new(
            id(format!("boundary:{tag}")),
            vec![authority::completeness::Fact::new(
                id(format!("member:{tag}")),
                observed.then(|| qualified.records()[0].identity.clone()),
                status,
            )],
        );
        let document =
            authority::completeness::derive(context, &selection, ObservationLimits::owner_max())
                .unwrap();
        authority::completeness::read(
            document.bytes(),
            context,
            &selection,
            ObservationLimits::owner_max(),
        )
        .unwrap()
    };
    let availability = |available: bool,
                        producer: authority::availability::DependencyState,
                        contract: authority::availability::DependencyState| {
        let required = vec![id(format!("required-result:{tag}"))];
        let selection = authority::availability::Selection::new(
            required.clone(),
            if available { required } else { Vec::new() },
            producer,
            contract,
        );
        let document =
            authority::availability::derive(context, &selection, ObservationLimits::owner_max())
                .unwrap();
        authority::availability::read(
            document.bytes(),
            context,
            &selection,
            ObservationLimits::owner_max(),
        )
        .unwrap()
    };

    OwnerViews {
        clock,
        progress_open: progress(OpenClosed::Open),
        progress_closed: progress(OpenClosed::Closed),
        closure_open: closure(OpenClosed::Open),
        closure_closed: closure(OpenClosed::Closed),
        completeness_complete: completeness(authority::completeness::FactStatus::Available, true),
        completeness_incomplete: completeness(
            authority::completeness::FactStatus::Incomplete,
            false,
        ),
        completeness_contradicted: completeness(
            authority::completeness::FactStatus::Contradicted,
            true,
        ),
        availability_available: availability(
            true,
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        availability_not_yet: availability(
            false,
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        availability_producer_unavailable: availability(
            false,
            authority::availability::DependencyState::Unavailable,
            authority::availability::DependencyState::Available,
        ),
        availability_contract_unavailable: availability(
            false,
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Unavailable,
        ),
    }
}

fn cutoff(fixture_clock: FixtureClock) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(fixture_clock.identity()),
        id("1"),
        2,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}

fn proposition_map() -> PropositionMapDocument {
    PropositionMapDocument::new(vec![PropositionEntry {
        id: PropositionId(0),
        name: "p0".to_owned(),
    }])
    .unwrap()
}

fn future_formula(profile: SemanticProfile) -> FormulaDocument {
    FormulaDocument::new(
        profile,
        NodeId(0),
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        })],
    )
    .unwrap()
}

fn past_formula() -> FormulaDocument {
    FormulaDocument::new_v2(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            Node::new(NodeKind::Once {
                interval: Interval::new(0, 1).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
    .unwrap()
}

fn trace_document(id: &str, closed: bool) -> TraceDocument {
    TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: id.to_owned(),
        closed,
        instants: vec![vec![PropositionId(0)], vec![]],
    }
}

fn history_document(id: &str, revision: u64, values: [bool; 2]) -> PositionHistoryDocument {
    PositionHistoryDocument::new(
        id,
        revision,
        0,
        1,
        Some(ClockBinding::EventPosition),
        values
            .into_iter()
            .enumerate()
            .map(|(position, value)| {
                PositionObservation::new(
                    u64::try_from(position).unwrap(),
                    value.then_some(PropositionId(0)).into_iter().collect(),
                    None,
                )
            })
            .collect(),
    )
    .unwrap()
}

fn fixed_history_document(id: &str) -> PositionHistoryDocument {
    let epoch = ExactNumber::new(100, 1).unwrap();
    let period = ExactNumber::new(10, 1).unwrap();
    PositionHistoryDocument::new(
        id,
        1,
        0,
        1,
        Some(ClockBinding::FixedSample {
            epoch,
            period,
            unit: "nanoseconds".to_owned(),
        }),
        [true, false]
            .into_iter()
            .enumerate()
            .map(|(position, value)| {
                let position = u64::try_from(position).unwrap();
                PositionObservation::new(
                    position,
                    value.then_some(PropositionId(0)).into_iter().collect(),
                    Some(ClockSample {
                        instant: fixed_sample_instant(epoch, period, position).unwrap(),
                        unit: "nanoseconds".to_owned(),
                    }),
                )
            })
            .collect(),
    )
    .unwrap()
}

fn observations<'a>(
    decision: &'a OwnerViews,
    surrounding: &'a OwnerViews,
    decision_progress: bool,
    decision_closure: bool,
    surrounding_progress: bool,
    surrounding_closure: bool,
    completeness: &'a authority::completeness::View,
    availability: &'a authority::availability::View,
) -> request::ObservationInputs<'a> {
    request::ObservationInputs {
        decision_scope_progress: if decision_progress {
            &decision.progress_closed
        } else {
            &decision.progress_open
        },
        decision_scope_closure: if decision_closure {
            &decision.closure_closed
        } else {
            &decision.closure_open
        },
        surrounding_execution_progress: if surrounding_progress {
            &surrounding.progress_closed
        } else {
            &surrounding.progress_open
        },
        surrounding_execution_closure: if surrounding_closure {
            &surrounding.closure_closed
        } else {
            &surrounding.closure_open
        },
        completeness,
        availability,
    }
}

fn future_request_input<'a>(
    formula: &'a FormulaDocument,
    propositions: &'a PropositionMapDocument,
    trace: &'a trace::ValidatedTrace,
    decision: &'a OwnerViews,
    _surrounding: &'a OwnerViews,
    observation_inputs: request::ObservationInputs<'a>,
) -> request::RequestInput<'a> {
    request::RequestInput {
        formula,
        proposition_map: propositions,
        input: request::TemporalInput::Future(trace),
        clock: &decision.clock,
        subject_identity: "native-subject:order-1",
        correspondence_identity: "native-tl-correspondence:order-1",
        anchor: 0,
        observations: observation_inputs,
    }
}

fn admit_request<'a>(
    input: request::RequestInput<'a>,
    limits: OwnerLimits,
) -> request::ValidatedTemporalRequest {
    let document = request::derive(input, limits).unwrap();
    request::read(document.bytes(), input, limits).unwrap()
}

fn admit_history(history: &PositionHistoryDocument) -> history::ValidatedPositionHistory {
    let document = history::derive(history, OwnerLimits::default()).unwrap();
    history::read(document.bytes(), history, OwnerLimits::default()).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn exact_wire_identity(contract: &str, bytes: &[u8], identity: &str) -> String {
    let member = format!(",\"identity\":\"{identity}\"");
    let position = bytes
        .windows(member.len())
        .position(|window| window == member.as_bytes())
        .unwrap();
    let mut preimage = Vec::with_capacity(bytes.len() - member.len());
    preimage.extend_from_slice(&bytes[..position]);
    preimage.extend_from_slice(&bytes[position + member.len()..]);
    let mut digest = Sha256::new();
    digest.update(contract.as_bytes());
    digest.update([0]);
    digest.update(preimage);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_closed_schema(value: &Value) {
    match value {
        Value::Object(object) => {
            if object.get("type") == Some(&Value::String("object".to_owned())) {
                assert_eq!(
                    object.get("additionalProperties"),
                    Some(&Value::Bool(false))
                );
            }
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                assert!(reference.starts_with("#/$defs/"));
            }
            for child in object.values() {
                assert_closed_schema(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_closed_schema(child);
            }
        }
        _ => {}
    }
}

fn structural_mutations(
    bytes: &[u8],
    version_field: &str,
    version: &str,
    required_field: &str,
) -> Vec<Vec<u8>> {
    let mut unknown: Value = serde_json::from_slice(bytes).unwrap();
    unknown["unknown"] = Value::Bool(true);
    let mut missing: Value = serde_json::from_slice(bytes).unwrap();
    missing.as_object_mut().unwrap().remove(required_field);
    let reordered: Value = serde_json::from_slice(bytes).unwrap();
    let duplicate = String::from_utf8(bytes.to_vec())
        .unwrap()
        .replacen('{', &format!("{{\"{version_field}\":\"{version}\","), 1)
        .into_bytes();
    let mut trailing = bytes.to_vec();
    trailing.extend_from_slice(b"\nnull");
    vec![
        serde_json::to_vec(&unknown).unwrap(),
        serde_json::to_vec(&missing).unwrap(),
        serde_json::to_vec(&reordered).unwrap(),
        duplicate,
        trailing,
    ]
}

// Trace: TC-084, FR-018-AC-1
#[test]
fn tc_084_future_and_past_requests_evaluate_and_strict_read() {
    let decision = owner_views("decision", 2);
    let surrounding = owner_views("surrounding", 2);
    let propositions = proposition_map();
    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let trace = trace::ValidatedTrace::admit(
        &trace_document("trace:future", true),
        OwnerLimits::default(),
    )
    .unwrap();
    let future_input = future_request_input(
        &formula,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            false,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    );
    let future_request = admit_request(future_input, OwnerLimits::default());
    assert_eq!(future_request.lane(), request::TemporalLane::Future);
    assert_eq!(
        future_request.semantic_profile(),
        SemanticProfile::ClosedTraceV1
    );
    assert_eq!(future_request.input_artifact().contract(), trace::CONTRACT);
    assert_eq!(future_request.clock_identity(), "clock:event-position");
    assert_eq!(future_request.clock_revision(), "1");
    assert_eq!(future_request.anchor(), 0);
    assert_eq!(
        future_request.completeness().population_identity(),
        future_request
            .decision_scope_progress()
            .population_identity()
    );
    assert_eq!(
        future_request.identity(),
        exact_wire_identity(
            request::CONTRACT,
            future_request.bytes(),
            future_request.identity(),
        )
    );
    let result_document = report::evaluate(
        &future_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let result = report::read(
        result_document.bytes(),
        &future_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(result.truth(), report::TemporalTruth::Satisfied);
    assert_eq!(
        result.decision_scope_progress().authority_identity(),
        future_request
            .decision_scope_progress()
            .authority_identity()
    );
    assert_eq!(result.relation_kind(), report::ResultRelationKind::Original);
    assert_eq!(result.direct_predecessor_identity(), None);
    assert_eq!(
        result.identity(),
        exact_wire_identity(report::CONTRACT, result.bytes(), result.identity())
    );
    assert_eq!(result.usage().evaluation_steps, 1);
    let selection = MappingSelection::for_result(&result);
    let mapped_document = contract_ir::map(&result, &selection, OwnerLimits::default()).unwrap();
    let mapped = contract_ir::read(
        mapped_document.bytes(),
        &result,
        &selection,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(mapped.outcome(), MappedOutcome::Value { value: true });
    assert_eq!(mapped.source().identity(), result.identity());
    assert_eq!(
        mapped.identity(),
        exact_wire_identity(contract_ir::CONTRACT, mapped.bytes(), mapped.identity())
    );

    let formula = past_formula();
    let history_value = history_document("history:past", 1, [true, false]);
    let history = admit_history(&history_value);
    let past_input = request::RequestInput {
        formula: &formula,
        proposition_map: &propositions,
        input: request::TemporalInput::Past(&history),
        clock: &decision.clock,
        subject_identity: "native-subject:order-1",
        correspondence_identity: "native-tl-correspondence:order-1",
        anchor: 1,
        observations: observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    };
    let past_request = admit_request(past_input, OwnerLimits::default());
    let past_document = report::evaluate(
        &past_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let past = report::read(
        past_document.bytes(),
        &past_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(past.truth(), report::TemporalTruth::Satisfied);
    assert!(past.is_final());
}

// Trace: TC-084, FR-018-AC-1, FR-018-AC-5
#[test]
fn tc_084_all_profiles_past_operators_and_supported_clocks_use_one_owner_path() {
    let decision = owner_views("decision-profiles", 2);
    let surrounding = owner_views("surrounding-profiles", 2);
    let propositions = proposition_map();
    let history_value = history_document("history:operators", 1, [true, false]);
    let history = admit_history(&history_value);
    let axes = observations(
        &decision,
        &surrounding,
        true,
        true,
        true,
        true,
        &decision.completeness_complete,
        &decision.availability_available,
    );
    let past_cases = [
        (
            FormulaDocument::new_v2(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(1),
                vec![
                    Node::new(NodeKind::Proposition {
                        proposition: PropositionId(0),
                    }),
                    Node::new(NodeKind::Once {
                        interval: Interval::new(0, 1).unwrap(),
                        operand: NodeId(0),
                    }),
                ],
            )
            .unwrap(),
            report::TemporalTruth::Satisfied,
        ),
        (
            FormulaDocument::new_v2(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(1),
                vec![
                    Node::new(NodeKind::Proposition {
                        proposition: PropositionId(0),
                    }),
                    Node::new(NodeKind::Historically {
                        interval: Interval::new(1, 3).unwrap(),
                        operand: NodeId(0),
                    }),
                ],
            )
            .unwrap(),
            report::TemporalTruth::Violated,
        ),
        (
            FormulaDocument::new_v2(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(1),
                vec![
                    Node::new(NodeKind::Proposition {
                        proposition: PropositionId(0),
                    }),
                    Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
                ],
            )
            .unwrap(),
            report::TemporalTruth::Satisfied,
        ),
        (
            FormulaDocument::new_v2(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(2),
                vec![
                    Node::new(NodeKind::True),
                    Node::new(NodeKind::Proposition {
                        proposition: PropositionId(0),
                    }),
                    Node::new(NodeKind::Since {
                        interval: Interval::new(1, 1).unwrap(),
                        left: NodeId(0),
                        right: NodeId(1),
                    }),
                ],
            )
            .unwrap(),
            report::TemporalTruth::Satisfied,
        ),
        (
            FormulaDocument::new_v2(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(2),
                vec![
                    Node::new(NodeKind::True),
                    Node::new(NodeKind::True),
                    Node::new(NodeKind::Triggered {
                        interval: Interval::new(0, 1).unwrap(),
                        left: NodeId(0),
                        right: NodeId(1),
                    }),
                ],
            )
            .unwrap(),
            report::TemporalTruth::Satisfied,
        ),
    ];
    for (formula, expected) in past_cases {
        let input = request::RequestInput {
            formula: &formula,
            proposition_map: &propositions,
            input: request::TemporalInput::Past(&history),
            clock: &decision.clock,
            subject_identity: "native-subject:operator-catalog",
            correspondence_identity: "native-tl-correspondence:operator-catalog",
            anchor: 1,
            observations: axes,
        };
        let request = admit_request(input, OwnerLimits::default());
        let document = report::evaluate(
            &request,
            report::ResultRelationInput::Original,
            OwnerLimits::default(),
        )
        .unwrap();
        let result = report::read(
            document.bytes(),
            &request,
            report::ResultRelationInput::Original,
            OwnerLimits::default(),
        )
        .unwrap();
        assert_eq!(result.truth(), expected);
    }

    let prefix_decision = owner_views("decision-prefix", 1);
    let prefix_surrounding = owner_views("surrounding-prefix", 1);
    let prefix_formula = FormulaDocument::new(
        SemanticProfile::OnlinePrefixV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            Node::new(NodeKind::Future {
                interval: Interval::new(1, 1).unwrap(),
                operand: NodeId(0),
            }),
        ],
    )
    .unwrap();
    let prefix_trace = trace::ValidatedTrace::admit(
        &TraceDocument {
            schema_version: TraceSchemaVersion::V1,
            trace_id: "trace:open-prefix".to_owned(),
            closed: false,
            instants: vec![vec![]],
        },
        OwnerLimits::default(),
    )
    .unwrap();
    let prefix_input = future_request_input(
        &prefix_formula,
        &propositions,
        &prefix_trace,
        &prefix_decision,
        &prefix_surrounding,
        observations(
            &prefix_decision,
            &prefix_surrounding,
            false,
            false,
            false,
            false,
            &prefix_decision.completeness_complete,
            &prefix_decision.availability_available,
        ),
    );
    let prefix_request = admit_request(prefix_input, OwnerLimits::default());
    let prefix_document = report::evaluate(
        &prefix_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let prefix_result = report::read(
        prefix_document.bytes(),
        &prefix_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(prefix_result.truth(), report::TemporalTruth::Pending);
    let selection = MappingSelection::for_result(&prefix_result);
    let document = contract_ir::map(&prefix_result, &selection, OwnerLimits::default()).unwrap();
    let mapped = contract_ir::read(
        document.bytes(),
        &prefix_result,
        &selection,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(
        mapped.outcome(),
        MappedOutcome::NonValue {
            reason: NonValueKind::Pending
        }
    );

    let fixed_decision = owner_views_with_clock("decision-fixed", 2, FixtureClock::FixedSample);
    let fixed_surrounding =
        owner_views_with_clock("surrounding-fixed", 2, FixtureClock::FixedSample);
    let fixed_formula = past_formula();
    let fixed_history_value = fixed_history_document("history:fixed");
    let fixed_history = admit_history(&fixed_history_value);
    let fixed_input = request::RequestInput {
        formula: &fixed_formula,
        proposition_map: &propositions,
        input: request::TemporalInput::Past(&fixed_history),
        clock: &fixed_decision.clock,
        subject_identity: "native-subject:fixed-clock",
        correspondence_identity: "native-tl-correspondence:fixed-clock",
        anchor: 1,
        observations: observations(
            &fixed_decision,
            &fixed_surrounding,
            true,
            true,
            true,
            true,
            &fixed_decision.completeness_complete,
            &fixed_decision.availability_available,
        ),
    };
    let fixed_request = admit_request(fixed_input, OwnerLimits::default());
    let fixed_document = report::evaluate(
        &fixed_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let fixed_result = report::read(
        fixed_document.bytes(),
        &fixed_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(fixed_result.truth(), report::TemporalTruth::Satisfied);
}

// Trace: TC-084, FR-018-AC-2
#[test]
fn tc_084_schemas_are_pinned_and_all_readers_fail_closed() {
    for (bytes, digest) in [
        (trace::SCHEMA_BYTES, trace::SCHEMA_SHA256),
        (command::SCHEMA_BYTES, command::SCHEMA_SHA256),
        (history::SCHEMA_BYTES, history::SCHEMA_SHA256),
        (requirement::SCHEMA_BYTES, requirement::SCHEMA_SHA256),
        (result::SCHEMA_BYTES, result::SCHEMA_SHA256),
        (request::SCHEMA_BYTES, request::SCHEMA_SHA256),
        (report::SCHEMA_BYTES, report::SCHEMA_SHA256),
        (contract_ir::SCHEMA_BYTES, contract_ir::SCHEMA_SHA256),
    ] {
        assert_eq!(sha256(bytes), digest);
        let schema: Value = serde_json::from_slice(bytes).unwrap();
        assert_closed_schema(&schema);
    }

    let trace_document = trace_document("trace:strict", true);
    let trace_owner = trace::derive(&trace_document, OwnerLimits::default()).unwrap();
    trace::read(trace_owner.bytes(), &trace_document, OwnerLimits::default()).unwrap();
    for mutated in structural_mutations(
        trace_owner.bytes(),
        "schemaVersion",
        trace::CONTRACT,
        "traceId",
    ) {
        assert!(trace::read(&mutated, &trace_document, OwnerLimits::default()).is_err());
    }

    let command = CommandDocument {
        schema_version: CommandSchemaVersion::V1,
        operation: Operation::Evaluate,
        formula_id: "formula:strict".to_owned(),
        formula: future_formula(SemanticProfile::ClosedTraceV1),
        trace: Some(trace_document.clone()),
    };
    let command_owner = command::derive(&command, OwnerLimits::default()).unwrap();
    command::read(command_owner.bytes(), &command, OwnerLimits::default()).unwrap();
    for mutated in structural_mutations(
        command_owner.bytes(),
        "schemaVersion",
        command::CONTRACT,
        "formulaId",
    ) {
        assert!(command::read(&mutated, &command, OwnerLimits::default()).is_err());
    }

    let history_value = history_document("history:strict", 1, [true, false]);
    let history_owner = history::derive(&history_value, OwnerLimits::default()).unwrap();
    history::read(
        history_owner.bytes(),
        &history_value,
        OwnerLimits::default(),
    )
    .unwrap();
    for mutated in structural_mutations(
        history_owner.bytes(),
        "schemaVersion",
        tl_mltl::POSITION_HISTORY_V1,
        "historyId",
    ) {
        assert!(history::read(&mutated, &history_value, OwnerLimits::default()).is_err());
    }

    let requirement_value =
        analyze_required_history(past_formula().validate().unwrap(), "formula:strict-past")
            .unwrap();
    let requirement_owner =
        requirement::derive(&requirement_value, OwnerLimits::default()).unwrap();
    requirement::read(
        requirement_owner.bytes(),
        &requirement_value,
        OwnerLimits::default(),
    )
    .unwrap();
    for mutated in structural_mutations(
        requirement_owner.bytes(),
        "schemaVersion",
        tl_mltl::HISTORY_REQUIREMENT_V1,
        "formulaId",
    ) {
        assert!(requirement::read(&mutated, &requirement_value, OwnerLimits::default(),).is_err());
    }

    let past_value = tl_mltl::evaluate_past(
        past_formula().validate().unwrap(),
        "formula:strict-past",
        &history_value,
        1,
        "map:strict",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    let past_owner = result::derive(&past_value, OwnerLimits::default()).unwrap();
    result::read(past_owner.bytes(), &past_value, OwnerLimits::default()).unwrap();
    for mutated in structural_mutations(
        past_owner.bytes(),
        "schemaVersion",
        tl_mltl::PAST_EVALUATION_V1,
        "resultSha256",
    ) {
        assert!(result::read(&mutated, &past_value, OwnerLimits::default()).is_err());
    }
}

// Trace: TC-084, FR-018-AC-3, FR-018-AC-4
#[test]
fn tc_084_owner_axes_remain_independent_and_non_values_are_total() {
    let decision = owner_views("decision-combinations", 2);
    let surrounding = owner_views("surrounding-combinations", 2);
    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let propositions = proposition_map();
    let trace = trace::ValidatedTrace::admit(
        &trace_document("trace:combinations", true),
        OwnerLimits::default(),
    )
    .unwrap();
    let completeness = [
        &decision.completeness_complete,
        &decision.completeness_incomplete,
        &decision.completeness_contradicted,
    ];
    let availability = [
        &decision.availability_available,
        &decision.availability_not_yet,
        &decision.availability_producer_unavailable,
        &decision.availability_contract_unavailable,
    ];
    let mut combinations = 0usize;
    for decision_progress in [false, true] {
        for decision_closure in [false, true] {
            for surrounding_progress in [false, true] {
                for surrounding_closure in [false, true] {
                    for completeness in completeness {
                        for availability in availability {
                            let input = future_request_input(
                                &formula,
                                &propositions,
                                &trace,
                                &decision,
                                &surrounding,
                                observations(
                                    &decision,
                                    &surrounding,
                                    decision_progress,
                                    decision_closure,
                                    surrounding_progress,
                                    surrounding_closure,
                                    completeness,
                                    availability,
                                ),
                            );
                            let request = admit_request(input, OwnerLimits::default());
                            let document = report::evaluate(
                                &request,
                                report::ResultRelationInput::Original,
                                OwnerLimits::default(),
                            )
                            .unwrap();
                            let result = report::read(
                                document.bytes(),
                                &request,
                                report::ResultRelationInput::Original,
                                OwnerLimits::default(),
                            )
                            .unwrap();
                            let selection = MappingSelection::for_result(&result);
                            let document =
                                contract_ir::map(&result, &selection, OwnerLimits::default())
                                    .unwrap();
                            let mapped = contract_ir::read(
                                document.bytes(),
                                &result,
                                &selection,
                                OwnerLimits::default(),
                            )
                            .unwrap();
                            let expected = match completeness.payload().state() {
                                authority::completeness::State::Incomplete => {
                                    MappedOutcome::NonValue {
                                        reason: NonValueKind::Incomplete,
                                    }
                                }
                                authority::completeness::State::Contradicted => {
                                    MappedOutcome::NonValue {
                                        reason: NonValueKind::Contradicted,
                                    }
                                }
                                authority::completeness::State::Complete
                                    if availability.payload().state()
                                        != authority::availability::State::Available =>
                                {
                                    MappedOutcome::NonValue {
                                        reason: NonValueKind::Unavailable,
                                    }
                                }
                                authority::completeness::State::Complete => {
                                    MappedOutcome::Value { value: true }
                                }
                            };
                            assert_eq!(mapped.outcome(), expected);
                            combinations += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(combinations, 192);
}

// Trace: TC-084, FR-018-AC-2, FR-018-AC-5
#[test]
fn tc_084_request_result_map_and_lineage_reject_substitution() {
    let decision = owner_views("decision-lineage", 2);
    let surrounding = owner_views("surrounding-lineage", 2);
    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let propositions = proposition_map();
    let first_trace =
        trace::ValidatedTrace::admit(&trace_document("trace:first", true), OwnerLimits::default())
            .unwrap();
    let axes = observations(
        &decision,
        &surrounding,
        true,
        true,
        true,
        true,
        &decision.completeness_complete,
        &decision.availability_available,
    );
    let first_input = future_request_input(
        &formula,
        &propositions,
        &first_trace,
        &decision,
        &surrounding,
        axes,
    );
    let first_request_document = request::derive(first_input, OwnerLimits::default()).unwrap();
    let first_request = request::read(
        first_request_document.bytes(),
        first_input,
        OwnerLimits::default(),
    )
    .unwrap();
    let first_document = report::evaluate(
        &first_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let first = report::read(
        first_document.bytes(),
        &first_request,
        report::ResultRelationInput::Original,
        OwnerLimits::default(),
    )
    .unwrap();
    let original_bytes = first.bytes().to_vec();

    let second_trace = trace::ValidatedTrace::admit(
        &trace_document("trace:corrected", true),
        OwnerLimits::default(),
    )
    .unwrap();
    let second_input = future_request_input(
        &formula,
        &propositions,
        &second_trace,
        &decision,
        &surrounding,
        axes,
    );
    let second_request = admit_request(second_input, OwnerLimits::default());
    let corrected_document = report::evaluate(
        &second_request,
        report::ResultRelationInput::Superseding(&first),
        OwnerLimits::default(),
    )
    .unwrap();
    let corrected = report::read(
        corrected_document.bytes(),
        &second_request,
        report::ResultRelationInput::Superseding(&first),
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(corrected.revision(), 2);
    assert_eq!(first.bytes(), original_bytes);

    assert_eq!(
        report::read(
            corrected.bytes(),
            &second_request,
            report::ResultRelationInput::Invalidating(&first),
            OwnerLimits::default(),
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );

    let selection = MappingSelection::for_result(&corrected);
    let map = contract_ir::map(&corrected, &selection, OwnerLimits::default()).unwrap();
    let wrong_selection = MappingSelection::for_result(&first);
    assert_eq!(
        contract_ir::read(
            map.bytes(),
            &corrected,
            &wrong_selection,
            OwnerLimits::default(),
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );

    for mutated in structural_mutations(
        first_request_document.bytes(),
        "contractVersion",
        request::CONTRACT,
        "identity",
    ) {
        assert!(request::read(&mutated, first_input, OwnerLimits::default()).is_err());
    }
    for mutated in structural_mutations(
        corrected.bytes(),
        "contractVersion",
        report::CONTRACT,
        "identity",
    ) {
        assert!(report::read(
            &mutated,
            &second_request,
            report::ResultRelationInput::Superseding(&first),
            OwnerLimits::default(),
        )
        .is_err());
    }
    for mutated in structural_mutations(
        map.bytes(),
        "contractVersion",
        contract_ir::CONTRACT,
        "identity",
    ) {
        assert!(
            contract_ir::read(&mutated, &corrected, &selection, OwnerLimits::default(),).is_err()
        );
    }

    let mut mutated_request = first_request_document.bytes().to_vec();
    let identity = first_request.identity();
    let replacement = if identity.starts_with('0') { '1' } else { '0' };
    let position = mutated_request
        .windows(identity.len())
        .position(|window| window == identity.as_bytes())
        .unwrap();
    mutated_request[position] = replacement as u8;
    assert_eq!(
        request::read(&mutated_request, first_input, OwnerLimits::default())
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::IdentityMismatch
    );
}

// Trace: TC-084, FR-018-AC-6
#[test]
fn tc_084_exact_and_one_over_limits_cover_each_owner_dimension() {
    let value = trace_document("trace:limits", true);
    let document = trace::derive(&value, OwnerLimits::default()).unwrap();
    let lexical = document.usage();
    trace::read(
        document.bytes(),
        &value,
        OwnerLimits {
            max_input_bytes: document.bytes().len(),
            max_output_bytes: document.bytes().len(),
            max_depth: lexical.depth,
            max_string_bytes: lexical.string_bytes,
            max_positions: 2,
            max_propositions: 1,
            max_visited_fields: lexical.visited_fields,
            ..OwnerLimits::default()
        },
    )
    .unwrap();
    trace::derive(
        &value,
        OwnerLimits {
            max_output_bytes: document.bytes().len(),
            ..OwnerLimits::default()
        },
    )
    .unwrap();
    for limits in [
        OwnerLimits {
            max_input_bytes: document.bytes().len() - 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_output_bytes: document.bytes().len() - 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_depth: lexical.depth - 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_string_bytes: lexical.string_bytes - 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_positions: 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_propositions: 0,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_visited_fields: lexical.visited_fields - 1,
            ..OwnerLimits::default()
        },
    ] {
        let error = if limits.max_output_bytes < document.bytes().len() {
            trace::derive(&value, limits).unwrap_err()
        } else {
            trace::read(document.bytes(), &value, limits).unwrap_err()
        };
        assert_eq!(error.code(), OwnerReadErrorCode::ResourceIncomplete);
    }

    let history_value = history_document("history:limits", 1, [true, false]);
    let history_document = history::derive(&history_value, OwnerLimits::default()).unwrap();
    history::read(
        history_document.bytes(),
        &history_value,
        OwnerLimits {
            max_history_span: 1,
            ..OwnerLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        history::read(
            history_document.bytes(),
            &history_value,
            OwnerLimits {
                max_history_span: 0,
                ..OwnerLimits::default()
            },
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );

    let nested = br#"{"a":{"b":{"c":true}}}"#;
    let trace_error = trace::ValidatedTrace::from_json_bytes(
        nested,
        OwnerLimits {
            max_depth: 2,
            ..OwnerLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(trace_error.code(), OwnerReadErrorCode::ResourceIncomplete);

    let decision = owner_views("decision-limits", 2);
    let surrounding = owner_views("surrounding-limits", 2);
    let propositions = proposition_map();
    let formula = FormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
        ],
    )
    .unwrap();
    let trace = trace::ValidatedTrace::admit(&value, OwnerLimits::default()).unwrap();
    let input = future_request_input(
        &formula,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    );
    let request_limits = OwnerLimits {
        max_formula_nodes: 2,
        max_formula_depth: 2,
        ..OwnerLimits::default()
    };
    let request = admit_request(input, request_limits);
    for limits in [
        OwnerLimits {
            max_formula_nodes: 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_formula_depth: 1,
            ..OwnerLimits::default()
        },
    ] {
        assert_eq!(
            request::derive(input, limits).unwrap_err().code(),
            OwnerReadErrorCode::ResourceIncomplete
        );
    }

    let exact_execution = report::evaluate(
        &request,
        report::ResultRelationInput::Original,
        OwnerLimits {
            max_evaluation_steps: 2,
            max_recursion_depth: 1,
            max_support: 1,
            ..OwnerLimits::default()
        },
    )
    .unwrap();
    assert_eq!(exact_execution.usage().evaluation_steps, 2);
    assert_eq!(exact_execution.usage().recursion_depth, 1);
    assert_eq!(exact_execution.usage().support, 1);
    for limits in [
        OwnerLimits {
            max_evaluation_steps: 1,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_recursion_depth: 0,
            ..OwnerLimits::default()
        },
        OwnerLimits {
            max_support: 0,
            ..OwnerLimits::default()
        },
    ] {
        let document =
            report::evaluate(&request, report::ResultRelationInput::Original, limits).unwrap();
        let result = report::read(
            document.bytes(),
            &request,
            report::ResultRelationInput::Original,
            limits,
        )
        .unwrap();
        assert_eq!(
            result.execution(),
            report::AssessmentExecution::ResourceIncomplete
        );
        assert!(!result.is_final());
        assert!(result.decision_support().is_empty());
    }
}

// Trace: TC-084, FR-018-AC-5, FR-018-AC-6
#[test]
fn tc_084_profile_clock_and_resource_refusals_never_coerce_boolean() {
    let decision = owner_views("decision-refusal", 2);
    let surrounding = owner_views("surrounding-refusal", 2);
    let propositions = proposition_map();
    let trace = trace::ValidatedTrace::admit(
        &trace_document("trace:refusal", true),
        OwnerLimits::default(),
    )
    .unwrap();
    let past = past_formula();
    let mismatched = future_request_input(
        &past,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    );
    assert_eq!(
        request::derive(mismatched, OwnerLimits::default())
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );

    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let input = future_request_input(
        &formula,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    );
    let request = admit_request(input, OwnerLimits::default());
    let constrained = OwnerLimits {
        max_evaluation_steps: 0,
        ..OwnerLimits::default()
    };
    let document =
        report::evaluate(&request, report::ResultRelationInput::Original, constrained).unwrap();
    let result = report::read(
        document.bytes(),
        &request,
        report::ResultRelationInput::Original,
        constrained,
    )
    .unwrap();
    assert_eq!(
        result.execution(),
        report::AssessmentExecution::ResourceIncomplete
    );
    assert_eq!(result.truth(), report::TemporalTruth::Unavailable);
    let selection = MappingSelection::for_result(&result);
    let document = contract_ir::map(&result, &selection, constrained).unwrap();
    let mapped = contract_ir::read(document.bytes(), &result, &selection, constrained).unwrap();
    assert_eq!(
        mapped.outcome(),
        MappedOutcome::NonValue {
            reason: NonValueKind::ResourceIncomplete
        }
    );
}

// Trace: TC-084, FR-018-AC-3, FR-018-AC-5
#[test]
fn tc_084_owner_evidence_contexts_cannot_be_cross_wired() {
    let decision = owner_views("decision-cross-wire", 2);
    let surrounding = owner_views("surrounding-cross-wire", 2);
    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let propositions = proposition_map();
    let trace = trace::ValidatedTrace::admit(
        &trace_document("trace:cross-wire", true),
        OwnerLimits::default(),
    )
    .unwrap();
    let input = future_request_input(
        &formula,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &surrounding.completeness_complete,
            &surrounding.availability_available,
        ),
    );
    assert_eq!(
        request::derive(input, OwnerLimits::default())
            .unwrap_err()
            .field(),
        "decisionEvidenceContext"
    );
}

// Trace: TC-085, FR-019-AC-1, FR-019-AC-2, FR-019-AC-3
#[test]
fn tc_085_qobs_c00_temporal_dispatch_and_unsupported_contracts_are_exact() {
    const C00_REVISION: &str = "924006300f45b38483be1cbdf99b68f899b7d368";

    let decision = owner_views("decision-c00", 2);
    let surrounding = owner_views("surrounding-c00", 2);
    let formula = future_formula(SemanticProfile::ClosedTraceV1);
    let propositions = proposition_map();
    let trace =
        trace::ValidatedTrace::admit(&trace_document("trace:c00", true), OwnerLimits::default())
            .unwrap();
    let input = future_request_input(
        &formula,
        &propositions,
        &trace,
        &decision,
        &surrounding,
        observations(
            &decision,
            &surrounding,
            true,
            true,
            true,
            true,
            &decision.completeness_complete,
            &decision.availability_available,
        ),
    );

    let direct = request::derive(input, OwnerLimits::default()).unwrap();
    let dispatched = observation::consume_temporal(input, OwnerLimits::default()).unwrap();
    assert_eq!(dispatched, direct);
    let direct_wire: Value =
        serde_json::from_slice(direct.bytes()).expect("temporal request is canonical JSON");
    assert_eq!(direct_wire["observationRevision"], C00_REVISION);

    let tightening = OwnerLimits {
        max_output_bytes: direct.bytes().len(),
        ..OwnerLimits::default()
    };
    let tightened = request::derive(input, tightening).unwrap();
    let exact = OwnerLimits {
        max_output_bytes: tightened.bytes().len(),
        ..OwnerLimits::default()
    };
    let exact_direct = request::derive(input, exact).unwrap();
    assert_eq!(exact_direct.bytes().len(), exact.max_output_bytes);
    let exact_dispatched = observation::consume_temporal(input, exact).unwrap();
    assert_eq!(exact_dispatched, exact_direct);
    let one_over = OwnerLimits {
        max_output_bytes: exact.max_output_bytes - 1,
        ..OwnerLimits::default()
    };
    let one_over_direct = request::derive(input, one_over);
    let one_over_dispatched = observation::consume_temporal(input, one_over);
    assert_eq!(one_over_dispatched, one_over_direct);
    assert_eq!(
        one_over_direct.unwrap_err().code(),
        OwnerReadErrorCode::ResourceIncomplete
    );

    for (dimension, limits) in [
        (
            "maxInputBytes",
            OwnerLimits {
                max_input_bytes: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxOutputBytes",
            OwnerLimits {
                max_output_bytes: exact.max_output_bytes,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxDepth",
            OwnerLimits {
                max_depth: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxStringBytes",
            OwnerLimits {
                max_string_bytes: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxFormulaNodes",
            OwnerLimits {
                max_formula_nodes: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxFormulaDepth",
            OwnerLimits {
                max_formula_depth: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxPositions",
            OwnerLimits {
                max_positions: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxPropositions",
            OwnerLimits {
                max_propositions: 0,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxSupport",
            OwnerLimits {
                max_support: 0,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxHistorySpan",
            OwnerLimits {
                max_history_span: 0,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxEvaluationSteps",
            OwnerLimits {
                max_evaluation_steps: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxRecursionDepth",
            OwnerLimits {
                max_recursion_depth: 1,
                ..OwnerLimits::default()
            },
        ),
        (
            "maxVisitedFields",
            OwnerLimits {
                max_visited_fields: 1,
                ..OwnerLimits::default()
            },
        ),
    ] {
        assert_eq!(
            observation::consume_temporal(input, limits),
            request::derive(input, limits),
            "dispatch must forward {dimension} without reconstruction"
        );
    }

    let observation::Compatibility::Supported(supported) =
        observation::compatibility(observation::Contract::TemporalAssessment)
    else {
        panic!("temporal assessment contract must remain supported by TL");
    };
    assert_eq!(
        supported.contract(),
        observation::Contract::TemporalAssessment
    );
    assert_eq!(supported.contract_label(), request::CONTRACT);
    assert_eq!(
        supported.observation_revision(),
        tl_mltl::QUIRE_OBSERVATION_REVISION
    );
    for (contract, expected_label) in [
        (
            observation::Contract::RepairPlan,
            authority::repair::CONTRACT,
        ),
        (
            observation::Contract::ClosedPopulationQuery,
            authority::query::CONTRACT,
        ),
    ] {
        let observation::Compatibility::Unsupported(unsupported) =
            observation::compatibility(contract)
        else {
            panic!("QObs-owned contract must remain unsupported by TL");
        };
        assert_eq!(unsupported.contract(), contract);
        assert_eq!(unsupported.contract_label(), expected_label);
        assert_eq!(
            unsupported.observation_revision(),
            tl_mltl::QUIRE_OBSERVATION_REVISION
        );
    }

    assert_eq!(tl_mltl::QUIRE_OBSERVATION_REVISION, C00_REVISION);
    let manifest_entry = include_str!("../Cargo.toml")
        .lines()
        .find(|line| line.starts_with("quire-observation = "))
        .expect("manifest has one direct QObs dependency");
    assert_eq!(
        manifest_entry,
        format!(
            "quire-observation = {{ version = \"=0.1.0\", git = \
             \"https://github.com/agent-ix/quire-observation\", rev = \"{C00_REVISION}\" }}"
        )
    );
    let lock_entry = include_str!("../Cargo.lock")
        .split("[[package]]")
        .find(|entry| {
            entry
                .lines()
                .any(|line| line == "name = \"quire-observation\"")
        })
        .expect("lockfile has the QObs package");
    let lock_source = lock_entry
        .lines()
        .find(|line| line.starts_with("source = "))
        .expect("QObs lock entry has an exact source");
    assert_eq!(
        lock_source,
        format!(
            "source = \"git+https://github.com/agent-ix/quire-observation?rev={0}#{0}\"",
            C00_REVISION
        )
    );
}
