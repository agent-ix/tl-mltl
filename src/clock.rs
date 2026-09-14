//! Exact event-position and fixed-sample clock primitives.

pub use crate::past::{
    fixed_sample_instant, ClockBinding, ClockError, ClockSample, ExactNumber, ExactNumberError,
    UnsupportedClockKind,
};
