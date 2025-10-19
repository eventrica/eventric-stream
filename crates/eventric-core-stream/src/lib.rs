#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![feature(trait_alias)]

mod condition;
mod event;
mod stream;

// =================================================================================================
// Eventric Core Stream
// =================================================================================================

// Re-Exports

pub use self::{
    condition::{
        AppendCondition,
        QueryCondition,
    },
    stream::{
        Stream,
        StreamConfigurator,
    },
};
