//! TL-223: more information never flips a conclusive verdict. A partial lasso
//! and a refinement of it (each missing value replaced by a decoded Boolean)
//! are evaluated by the provider; a conclusive verdict on the coarse lasso must
//! survive on the refined one, unless fairness removes every completion. The
//! refined result must also match the successor-fixpoint reference.
#![no_main]

#[path = "../../tests/support/lasso_kit.rs"]
mod kit;
#[path = "../support/infinite.rs"]
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| support::partial_valuation_monotone(data));
