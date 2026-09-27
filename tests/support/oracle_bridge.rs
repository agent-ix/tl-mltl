//! Test-only wire bridge between the provisional TL-15 syntax pin and the
//! independently pinned oracle syntax. No production type crosses this seam.

use tl_oracle::{Limits, OracleError, Outcome};
use tl_syntax::{InfiniteFormulaDocument, LassoTraceDocument, NodeId};

pub fn evaluate_documents(
    formula: &InfiniteFormulaDocument,
    trace: &LassoTraceDocument,
    claim_root: NodeId,
    fairness_roots: &[NodeId],
    position: usize,
    limits: Limits,
) -> Result<Outcome, OracleError> {
    let formula_bytes = formula
        .canonical_json_bytes()
        .expect("validated formula wire");
    let trace_bytes = serde_json::to_vec(trace).expect("validated lasso wire");
    let oracle_formula =
        serde_json::from_slice(&formula_bytes).expect("oracle syntax admits formula wire");
    let oracle_trace =
        serde_json::from_slice(&trace_bytes).expect("oracle syntax admits lasso wire");
    let oracle_root = serde_json::from_value(serde_json::to_value(claim_root).unwrap()).unwrap();
    let oracle_fairness: Vec<_> = fairness_roots
        .iter()
        .map(|root| serde_json::from_value(serde_json::to_value(root).unwrap()).unwrap())
        .collect();
    tl_oracle::evaluate_documents(
        &oracle_formula,
        &oracle_trace,
        oracle_root,
        &oracle_fairness,
        position,
        limits,
    )
}
