//! W/M canonical interoperability and target loss evidence (FR-017).
//!
//! Replays the retained tl-syntax `corpus/future-operators` bytes through the
//! real `FutureLoweringRequest` and the C2PO mapping. A lowered W/M graph is
//! exported only as its canonical primitive graph, a target profile that cannot
//! preserve the semantics is refused with no manifest, and nothing here counts
//! a foreign parser or monitor as qualification evidence.

use std::{fs, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tl_mltl::{
    map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest, MappingError,
    MappingManifest, MappingSourceIdentity, MappingSourceState, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    Formula, FormulaDocument, FutureLoweringRefusal, FutureLoweringRequest, Node, NodeId,
    OwnedSignalDeclaration, PropositionBinding, PropositionId, RawBounds, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId, FUTURE_LOWERING_REQUEST_V1,
};

/// Retained corpus directory, a byte-identical copy at [`TL_SYNTAX_REVISION`].
const CORPUS: &str = "corpus/future-operators";
/// SHA-256 of the retained `manifest.json`, which in turn pins every case file.
const CORPUS_MANIFEST_SHA256: &str =
    "e38ef2a7bfc49631932c9c8527b9d08ba1087825e8ae3bccff5f326e74605172";
/// Corpus identity and revision recorded by the retained manifest.
const CORPUS_IDENTITY: &str = "tl-syntax.future-operator-corpus/v1";
const CORPUS_MANIFEST_REVISION: u64 = 1;
/// tl-parse revision the corpus sources were cross-checked against upstream.
/// Recorded, never executed, and not a tl-mltl dependency.
const PARSER_REVISION: &str = "9ca856b4c040fc2c3329b6defd26a1c9b57de748";
const MAPPING_LIMITATION: &str =
    "mapping evidence does not establish external monitor timing, memory, or qualification";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> Vec<u8> {
    let path = root().join(relative);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Verifies the retained corpus against its pinned manifest and returns the cases.
fn pinned_cases() -> Vec<Value> {
    let manifest_bytes = read(&format!("{CORPUS}/manifest.json"));
    assert_eq!(sha256_hex(&manifest_bytes), CORPUS_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest["corpus"], CORPUS_IDENTITY);
    assert_eq!(manifest["revision"], CORPUS_MANIFEST_REVISION);
    let files = manifest["files"].as_array().unwrap();
    assert!(!files.is_empty(), "the corpus manifest pins no files");
    for file in files {
        let path = file["path"].as_str().unwrap();
        assert_eq!(
            sha256_hex(&read(&format!("{CORPUS}/{path}"))),
            file["sha256"].as_str().unwrap(),
            "{path} does not match the pinned corpus manifest"
        );
    }
    let cases: Value = serde_json::from_slice(&read(&format!("{CORPUS}/cases.json"))).unwrap();
    assert_eq!(cases["corpus"], CORPUS_IDENTITY);
    cases["cases"].as_array().unwrap().clone()
}

fn profile(case: &Value) -> SemanticProfile {
    serde_json::from_value(case["semantic_profile"].clone()).unwrap()
}

fn raw_bounds(value: &Value) -> Option<RawBounds> {
    (!value.is_null()).then(|| {
        RawBounds::new(
            value["start"].as_u64().unwrap(),
            value["end"].as_u64().unwrap(),
        )
    })
}

/// Replays a case's append and lower steps into a canonical node table.
fn replay(case: &Value) -> Result<Vec<Node>, FutureLoweringRefusal> {
    let profile = profile(case);
    let mut nodes: Vec<Node> = Vec::new();
    for step in case["steps"].as_array().unwrap() {
        if let Some(append) = step.get("append") {
            nodes.push(serde_json::from_value(append.clone()).unwrap());
            continue;
        }
        let lower = &step["lower"];
        let text = |field: &str, default: &Value| -> Vec<u8> {
            lower
                .get(field)
                .unwrap_or(default)
                .as_str()
                .unwrap()
                .as_bytes()
                .to_vec()
        };
        let request_identity = text("request_identity", &Value::from(FUTURE_LOWERING_REQUEST_V1));
        let operator_profile = text("operator_profile", &case["operator_profile"]);
        let semantic_profile = text("semantic_profile", &case["semantic_profile"]);
        let kind = text("kind", &Value::Null);
        let lowered = FutureLoweringRequest {
            request_identity: &request_identity,
            operator_profile: &operator_profile,
            kind: &kind,
            semantic_profile: &semantic_profile,
            formula: Formula::new(profile, last_node(case, &nodes), &nodes).unwrap(),
            left: lower["left"].as_u64().unwrap(),
            right: lower["right"].as_u64().unwrap(),
            interval: raw_bounds(&lower["interval"]),
            operator_span: raw_bounds(&lower["operator_span"]),
            expression_span: raw_bounds(&lower["expression_span"]),
        }
        .lower()?;
        nodes.extend_from_slice(lowered.nodes());
    }
    Ok(nodes)
}

fn last_node(case: &Value, nodes: &[Node]) -> NodeId {
    let last = nodes
        .len()
        .checked_sub(1)
        .unwrap_or_else(|| panic!("{}: a lower step precedes every append", case["id"]));
    NodeId(u32::try_from(last).unwrap_or_else(|_| panic!("{}: node table too large", case["id"])))
}

fn case_formula<'a>(case: &Value, nodes: &'a [Node]) -> Formula<'a> {
    let root = NodeId(u32::try_from(case["root"].as_u64().unwrap()).unwrap());
    Formula::new(profile(case), root, nodes).unwrap()
}

fn direct_pair<'a>(cases: &'a [Value], derived: &Value) -> &'a Value {
    let id = derived["id"].as_str().unwrap();
    let direct_id = format!("{}-direct", id.strip_suffix("-derived").unwrap());
    cases
        .iter()
        .find(|case| case["class"] == "direct" && case["id"] == direct_id.as_str())
        .unwrap_or_else(|| panic!("{id} has no direct pair"))
}

fn source() -> MappingSourceIdentity {
    MappingSourceIdentity {
        revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        state: MappingSourceState::parse(env!("TL_MLTL_SOURCE_STATE")).unwrap(),
    }
}

/// A derived case and its direct pair export under one formula identity.
fn export_id(case: &Value) -> &str {
    let id = case["id"].as_str().unwrap();
    id.strip_suffix("-derived")
        .or_else(|| id.strip_suffix("-direct"))
        .unwrap()
}

fn map(case: &Value, formula: Formula<'_>) -> Result<MappingManifest, MappingError> {
    let expected = case["expected"].as_str().unwrap();
    let formula_bytes = read(&format!("{CORPUS}/{expected}"));
    map_to_c2po(
        formula,
        export_id(case),
        &formula_bytes,
        source(),
        None,
        10_000,
    )
}

/// A catalog binding each proposition to a boolean signal named after it.
fn catalog(propositions: &[u32]) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        propositions
            .iter()
            .map(|id| {
                OwnedSignalDeclaration::new(SignalId(*id), format!("p{id}"), SignalDomain::Boolean)
            })
            .collect(),
        propositions
            .iter()
            .map(|id| PropositionBinding::new(PropositionId(*id), SignalId(*id)))
            .collect(),
    )
    .unwrap()
}

/// The same export through the context-bound v2 C2PO mapping.
fn map_with_context(
    case: &Value,
    formula: Formula<'_>,
    catalog: &SignalCatalogDocument,
) -> Result<ContextualMappingManifest, MappingError> {
    let expected = case["expected"].as_str().unwrap();
    let formula_bytes = read(&format!("{CORPUS}/{expected}"));
    map_to_c2po_with_context(
        formula,
        export_id(case),
        &formula_bytes,
        source(),
        None,
        10_000,
        catalog,
        None,
    )
}

fn derived_cases(cases: &[Value]) -> Vec<&Value> {
    cases
        .iter()
        .filter(|case| case["class"] == "derived")
        .collect()
}

// Trace: TC-081, FR-017-AC-1
#[test]
fn lowered_wm_graphs_export_to_c2po_exactly_as_direct_canonical_graphs() {
    let cases = pinned_cases();
    let mut exported = 0;
    for derived in derived_cases(&cases) {
        let id = derived["id"].as_str().unwrap();
        let lowered_nodes = replay(derived).unwrap_or_else(|refusal| panic!("{id}: {refusal:?}"));
        let lowered = case_formula(derived, &lowered_nodes);

        // The exported graph is the pinned canonical document, spans aside.
        let expected: FormulaDocument = serde_json::from_slice(&read(&format!(
            "{CORPUS}/{}",
            derived["expected"].as_str().unwrap()
        )))
        .unwrap();
        let lowered_document = FormulaDocument::from_formula(lowered).unwrap();
        assert_eq!(
            lowered_document.semantic_view(),
            expected.semantic_view(),
            "{id}"
        );

        if profile(derived) != SemanticProfile::OnlinePrefixV1 {
            continue;
        }
        let direct = direct_pair(&cases, derived);
        let direct_nodes = replay(direct).unwrap();
        let direct_formula = case_formula(direct, &direct_nodes);
        let from_lowered = map(derived, lowered).unwrap();
        let from_direct = map(direct, direct_formula).unwrap();
        // Both calls share input bytes, formula id, and source identity, so only
        // `expression`, `output_sha256`, and `proposition_ids` are graph-derived;
        // the swapped-operand control below shows those fields can differ.
        assert_eq!(from_lowered, from_direct, "{id}");
        assert_eq!(from_lowered.syntax_revision, TL_SYNTAX_REVISION);
        assert_eq!(from_lowered.semantic_profile, "mltl.online-prefix/v1");
        // The evaluator identity is the build-recorded tl-mltl source revision.
        assert_eq!(
            from_lowered.source_revision,
            env!("TL_MLTL_SOURCE_REVISION")
        );
        assert_eq!(from_lowered.source_state, env!("TL_MLTL_SOURCE_STATE"));

        // The context-bound v2 C2PO path exports the same graph.
        let catalog = catalog(&from_lowered.proposition_ids);
        let contextual_lowered = map_with_context(derived, lowered, &catalog).unwrap();
        let contextual_direct = map_with_context(direct, direct_formula, &catalog).unwrap();
        assert_eq!(contextual_lowered, contextual_direct, "{id}");
        assert_eq!(contextual_lowered.syntax_revision, TL_SYNTAX_REVISION);

        // Formula-v1 has no W or M node, so this documents rather than gates;
        // the swapped-operand control below is the discriminating check.
        let tokens: Vec<&str> = from_lowered
            .expression
            .split(|character: char| !character.is_ascii_alphanumeric())
            .collect();
        assert!(
            !tokens.iter().any(|token| *token == "W" || *token == "M"),
            "{id}: the exported expression re-sugars a derived operator: {}",
            from_lowered.expression
        );
        exported += 1;
    }
    assert_eq!(
        exported, 7,
        "every online-prefix W/M corpus case is exported"
    );

    // Negative control: a wrongly lowered graph (operands swapped) does not
    // export as the direct pair's manifest.
    let derived = cases
        .iter()
        .find(|case| case["id"] == "weak-until-online-derived")
        .unwrap();
    let mut swapped = derived.clone();
    for step in swapped["steps"].as_array_mut().unwrap() {
        if let Some(lower) = step.get_mut("lower") {
            let left = lower["left"].take();
            lower["left"] = lower["right"].take();
            lower["right"] = left;
        }
    }
    let swapped_nodes = replay(&swapped).unwrap();
    let direct = direct_pair(&cases, derived);
    let direct_nodes = replay(direct).unwrap();
    let from_swapped = map(&swapped, case_formula(&swapped, &swapped_nodes)).unwrap();
    let from_direct = map(direct, case_formula(direct, &direct_nodes)).unwrap();
    assert_ne!(from_swapped.expression, from_direct.expression);
    assert_ne!(from_swapped, from_direct);
}

// Trace: TC-082, FR-017-AC-2, NFR-002-AC-1
#[test]
fn unpreservable_targets_and_refused_lowerings_emit_no_manifest() {
    let cases = pinned_cases();
    let mut profile_refusals = 0;
    for derived in derived_cases(&cases) {
        if profile(derived) != SemanticProfile::ClosedTraceV1 {
            continue;
        }
        let id = derived["id"].as_str().unwrap();
        let lowered_nodes = replay(derived).unwrap();
        let direct = direct_pair(&cases, derived);
        let direct_nodes = replay(direct).unwrap();
        let lowered = case_formula(derived, &lowered_nodes);
        let direct_formula = case_formula(direct, &direct_nodes);
        let catalog = catalog(&[]);
        for result in [
            map(derived, lowered).map(|_| ()),
            map(direct, direct_formula).map(|_| ()),
            map_with_context(derived, lowered, &catalog).map(|_| ()),
            map_with_context(direct, direct_formula, &catalog).map(|_| ()),
        ] {
            assert!(
                matches!(
                    result,
                    Err(MappingError::UnsupportedProfile { actual })
                        if actual == SemanticProfile::ClosedTraceV1.as_str()
                ),
                "{id}: closed-trace W/M must not be reinterpreted for C2PO: {result:?}"
            );
        }
        profile_refusals += 1;
    }
    assert_eq!(profile_refusals, 8);

    let mut lowering_refusals = 0;
    for refused in cases.iter().filter(|case| case["class"] == "refused") {
        let id = refused["id"].as_str().unwrap();
        let refusal = replay(refused).expect_err(id);
        assert_eq!(
            refusal.code(),
            refused["expected_refusal"]["code"].as_str().unwrap(),
            "{id}"
        );
        lowering_refusals += 1;
    }
    assert_eq!(lowering_refusals, 16);
}

// Trace: TC-083, FR-017-AC-3
#[test]
fn foreign_parser_and_monitor_acceptance_is_never_qualification_evidence() {
    let cases = pinned_cases();
    let manifest: Value =
        serde_json::from_slice(&read(&format!("{CORPUS}/manifest.json"))).unwrap();
    assert_eq!(manifest["source_cross_check"]["parser"], "tl-parse");
    assert_eq!(manifest["source_cross_check"]["revision"], PARSER_REVISION);

    // The parser is a recorded upstream cross-check, not a tl-mltl input,
    // neither direct nor transitive.
    let cargo = String::from_utf8(read("Cargo.toml")).unwrap();
    assert!(!cargo.contains("tl-parse"), "tl-parse became a dependency");
    let lock = String::from_utf8(read("Cargo.lock")).unwrap();
    assert!(
        !lock.contains("name = \"tl-parse\""),
        "tl-parse became a transitive dependency"
    );
    // The corpus is consumed from the compiled tl-syntax revision.
    let corpus_readme = String::from_utf8(read("corpus/README.md")).unwrap();
    let pinned_sentence = format!(
        "`future-operators/` is a byte-identical copy of `corpus/future-operators` at\nthe compiled revision `{TL_SYNTAX_REVISION}`"
    );
    assert!(
        corpus_readme.contains(&pinned_sentence),
        "corpus/README.md does not contain: {pinned_sentence}"
    );

    for derived in derived_cases(&cases) {
        if profile(derived) != SemanticProfile::OnlinePrefixV1 {
            continue;
        }
        let nodes = replay(derived).unwrap();
        let manifest = map(derived, case_formula(derived, &nodes)).unwrap();
        assert_eq!(manifest.external_tool, None);
        assert_eq!(manifest.limitation, MAPPING_LIMITATION);
    }

    // FRETish stays output-only elsewhere: this crate has no FRETish emitter,
    // importer, or target name at any depth under src/.
    let mut pending = vec![root().join("src")];
    let mut inspected = 0;
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
            continue;
        }
        let text = fs::read_to_string(&path).unwrap().to_ascii_lowercase();
        assert!(
            !text.contains("fretish"),
            "{} names FRETish",
            path.display()
        );
        inspected += 1;
    }
    assert!(inspected > 0);
}
