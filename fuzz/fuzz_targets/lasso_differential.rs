#![no_main]

#[path = "../support/infinite.rs"]
mod support;

use libfuzzer_sys::fuzz_target;
use tl_mltl::infinite::{Disposition, ResultReason};
use tl_oracle::{evaluate_documents, Limits, OracleError, Verdict};
use tl_syntax::{FairnessPremisesDocument, InfiniteClock, NodeId};

// Trace: TL-223, TC-193/194. Every byte string yields a bounded, valid lasso
// and one of the infinite temporal operators. The reference evaluation is
// owned by tl-oracle, which is a fuzz-only dependency.
fuzz_target!(|data: &[u8]| {
    let mut seed = support::Seed::new(data);
    let formula = support::formula(&mut seed);
    let (prefix_len, rows) = support::values(&mut seed, false);
    let trace = support::trace(prefix_len, &rows);
    let position = usize::from(seed.byte()) % (rows.len() + 2);
    let fairness = (seed.byte() & 1 != 0).then(|| {
        FairnessPremisesDocument::new(
            &formula,
            formula.content_identity().unwrap(),
            InfiniteClock::EventPosition,
            vec![NodeId(0)],
        )
        .unwrap()
    });
    let production = support::evaluate(&formula, &trace, fairness.as_ref(), position);
    let fairness_roots: &[NodeId] = if fairness.is_some() {
        &[NodeId(0)]
    } else {
        &[]
    };
    match evaluate_documents(
        &formula,
        &trace,
        formula.formula().root(),
        fairness_roots,
        position,
        Limits::default(),
    ) {
        Ok(reference) => {
            let expected = match reference.verdict {
                Verdict::Proved => Disposition::Proved,
                Verdict::Refuted => Disposition::Refuted,
                Verdict::Inconclusive => Disposition::Inconclusive,
            };
            assert_eq!(production.disposition, expected);
            assert_eq!(
                production.admitted_completions,
                u64::try_from(reference.fair_completions).unwrap()
            );
        }
        Err(OracleError::EmptyFairAdmission) => {
            assert_eq!(production.disposition, Disposition::Inconclusive);
            assert_eq!(production.reason, Some(ResultReason::EmptyFairAdmission));
            assert_eq!(production.admitted_completions, 0);
        }
        Err(error) => panic!("generated oracle request was refused: {error:?}"),
    }
});
