#![no_main]

#[path = "../support/infinite.rs"]
mod support;

use libfuzzer_sys::fuzz_target;
use tl_mltl::infinite::Disposition;
use tl_syntax::PartialValue;

// Trace: TL-223, TC-142/193. Refinement replaces only missing or conflicting
// cells; every formerly conclusive trace verdict must remain conclusive with
// the same truth value.
fuzz_target!(|data: &[u8]| {
    let mut seed = support::Seed::new(data);
    let formula = support::formula(&mut seed);
    let (prefix_len, rows) = support::values(&mut seed, true);
    let position = usize::from(seed.byte()) % (rows.len() + 2);
    let initial = support::trace(prefix_len, &rows);
    let before = support::evaluate(&formula, &initial, None, position);

    let mut refined_rows = rows.clone();
    for row in &mut refined_rows {
        for value in row {
            if matches!(value, PartialValue::Missing | PartialValue::Conflicting) {
                *value = if seed.byte() & 1 == 0 {
                    PartialValue::False
                } else {
                    PartialValue::True
                };
            }
        }
    }
    let refined = support::trace(prefix_len, &refined_rows);
    let after = support::evaluate(&formula, &refined, None, position);
    assert!(before.admitted_completions >= after.admitted_completions);
    if matches!(
        before.disposition,
        Disposition::Proved | Disposition::Refuted
    ) {
        assert_eq!(after.disposition, before.disposition);
    }
});
