#![cfg(feature = "infinite-trace")]

use std::{fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::infinite::{evaluate_lasso, Disposition, EvaluationLimit, LassoRequest, ResultReason};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, LassoTraceDocument, NodeId,
    PartialValuation, PartialValue, PropositionEntry, PropositionId, PropositionMapDocument,
    SemanticProfile, SyntaxArtifactLimits, TraceObservation, ValuationEntry, CORPUS_DIR,
};

#[derive(Deserialize)]
struct CorpusTrace {
    schema_version: String,
    semantic_profile: String,
    clock: String,
    proposition_map: Vec<PropositionEntry>,
    prefix: Vec<CorpusObservation>,
    #[serde(rename = "loop")]
    loop_observations: Vec<CorpusObservation>,
}

#[derive(Deserialize)]
struct CorpusObservation {
    position: u32,
    valuation: Vec<CorpusValue>,
}

#[derive(Deserialize)]
struct CorpusValue {
    proposition: PropositionId,
    state: PartialValue,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn observations(
    rows: Vec<CorpusObservation>,
    map_id: &str,
    propositions: &[PropositionId],
) -> Vec<TraceObservation> {
    rows.into_iter()
        .map(|row| TraceObservation {
            position: row.position,
            valuation: PartialValuation::new(
                map_id.to_owned(),
                propositions,
                row.valuation
                    .into_iter()
                    .map(|cell| ValuationEntry {
                        proposition: cell.proposition,
                        value: cell.state,
                    })
                    .collect(),
            )
            .unwrap(),
        })
        .collect()
}

// Owner corpus, strict formula reader, and all settlement/refusal cases.
// Trace: TC-138, TC-141, TC-143, TC-146, TC-155, TC-162; FR-027-AC-1, FR-030-AC-1, FR-030-AC-2, FR-032-AC-1, FR-034-AC-1, FR-041-AC-1
#[test]
fn owner_infinite_corpus_replays_from_the_compiled_syntax_revision() {
    let root = Path::new(CORPUS_DIR).join("infinite-trace");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let sums = fs::read_to_string(root.join("SHA256SUMS")).unwrap();
    for pin in manifest["files"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        let hash = digest(&fs::read(root.join(name)).unwrap());
        assert_eq!(hash, pin["sha256"].as_str().unwrap());
        assert!(sums.contains(&format!("{hash}  {name}\n")));
    }
    let manifest_hash = digest(&fs::read(root.join("manifest.json")).unwrap());
    assert!(sums.contains(&format!("{manifest_hash}  manifest.json\n")));
    let corpus: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(
        cases.len(),
        manifest["case_count"].as_u64().unwrap() as usize
    );

    for case in cases {
        let id = case["id"].as_str().unwrap();
        let admitted: Result<InfiniteFormulaDocument, _> =
            serde_json::from_value(case["formula"].clone());
        let formula = admitted.and_then(|document| {
            let canonical = document.canonical_json_bytes()?;
            Ok(InfiniteFormulaDocument::from_json_bytes(
                &canonical,
                SyntaxArtifactLimits::default(),
            )
            .expect("owner canonical formula must pass its strict reader"))
        });
        let axis = case["expected"]["axis"].as_str();
        if matches!(axis, Some("profile" | "operator")) {
            assert!(formula.is_err(), "{id}: invalid formula admitted");
            continue;
        }
        let formula = formula.unwrap_or_else(|error| panic!("{id}: {error:?}"));
        let wire: CorpusTrace = serde_json::from_value(case["trace"].clone()).unwrap();
        assert_eq!(wire.schema_version, tl_syntax::LASSO_TRACE_V1);
        assert_eq!(wire.semantic_profile, "mltl.infinite-trace/v1");
        if axis == Some("clock") {
            assert_ne!(wire.clock, "event_position", "{id}");
            continue;
        }
        assert_eq!(wire.clock, "event_position", "{id}");
        let map = PropositionMapDocument::new(wire.proposition_map).unwrap();
        let map_id = map.content_identity().unwrap();
        let propositions: Vec<_> = map.propositions().iter().map(|entry| entry.id).collect();
        let prefix = observations(wire.prefix, &map_id, &propositions);
        let loop_observations = observations(wire.loop_observations, &map_id, &propositions);
        if axis == Some("fairness") {
            assert!(loop_observations.is_empty(), "{id}");
            continue;
        }
        let trace = LassoTraceDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            map_id,
            propositions,
            prefix,
            loop_observations,
        )
        .unwrap();
        let graph_id = formula.content_identity().unwrap();
        let roots: Vec<NodeId> = serde_json::from_value(case["fairness"]["roots"].clone()).unwrap();
        let fairness = FairnessPremisesDocument::new(
            &formula,
            graph_id.clone(),
            InfiniteClock::EventPosition,
            roots,
        )
        .unwrap();
        let trace_id = trace.content_identity().unwrap();
        let report = evaluate_lasso(&LassoRequest {
            formula: &formula,
            trace: &trace,
            fairness: Some(&fairness),
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: case["anchor"].as_u64().unwrap(),
            limit: EvaluationLimit::default(),
        })
        .unwrap();
        let expected = case["expected"]["value"].as_str().unwrap();
        if id == "conflicting-value-is-inconclusive" {
            assert_eq!(expected, "inconclusive");
            assert_eq!(report.disposition, Disposition::Inconclusive);
            assert_eq!(report.reason, Some(ResultReason::ConflictingObservation));
        }
        let actual = match report.disposition {
            Disposition::Proved => "proved",
            Disposition::Refuted => "refuted",
            Disposition::Inconclusive => "inconclusive",
            Disposition::Unsupported => "unsupported",
            Disposition::Failed => "failed",
        };
        assert_eq!(actual, expected, "{id}");
    }
}
