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
            DescriptorRef,
            EventRef,
            IdentifierRef,
            TagRef,
        },
        query::{
            QueryItemRef,
            QueryRef,
            SpecifierRef,
        },
    },
    state::{
        Keyspaces,
        Read,
        Write,
    },
};
