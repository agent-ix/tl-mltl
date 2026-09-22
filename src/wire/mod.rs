//! Canonical temporal owner documents and strict public readers.

pub mod command;
pub mod common;
mod legacy;
pub mod trace;

pub use command::ValidatedCommand;
pub use common::{OwnerDocument, OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
pub use legacy::{
    CommandDocument, CommandSchemaVersion, Operation, TraceDocument, TraceSchemaVersion,
};
pub use trace::ValidatedTrace;
