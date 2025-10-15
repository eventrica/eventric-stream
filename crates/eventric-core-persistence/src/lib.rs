#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod model;

// =================================================================================================
// Eventric Core Persistence
// =================================================================================================

// Re-Export

pub use self::model::{
    event::{
        DescriptorHash,
        DescriptorHashRef,
        EventHash,
        EventHashRef,
        IdentifierHash,
        IdentifierHashRef,
        TagHash,
        TagHashRef,
    },
    query::{
        QueryHash,
        QueryItemHash,
        SpecifierHash,
    },
};
