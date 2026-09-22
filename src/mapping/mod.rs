//! Temporal result mappings owned by tl-mltl.

mod legacy;

pub use legacy::{
    map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest,
    ContextualMappingSchemaVersion, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState,
};
