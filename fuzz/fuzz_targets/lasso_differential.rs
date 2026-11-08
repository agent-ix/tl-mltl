//! TL-223: the infinite-trace provider against the independent reference
//! procedures. Any byte string decodes to a formula, optional fairness
//! premises, a partial lasso and a position; the provider and the
//! successor-fixpoint reference must agree on disposition, admitted
//! completions, witness flags and reason, and the two references must agree
//! with each other.
#![no_main]

#[path = "../../tests/support/lasso_kit.rs"]
mod kit;
#[path = "../support/infinite.rs"]
mod support;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| support::lasso_differential(data));
