#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod context;
mod model;
mod state;

// =================================================================================================
// Eventric Core Persistence
// =================================================================================================

// Re-Export

pub use self::{
    context::Context,
    model::{
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
    },
    state::{
        Keyspaces,
        Read,
        Write,
    },
};
