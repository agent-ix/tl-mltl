//! Temporal result mappings owned by tl-mltl.

pub(crate) mod legacy;
mod past;

pub use legacy::{
    map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest,
    ContextualMappingSchemaVersion, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState,
};
#[cfg(feature = "infinite-trace")]
pub(crate) use past::target_equivalent_interval;
pub use past::{map_past_to_c2po, PastMappingError, PastMappingManifest, TargetOriginContract};
