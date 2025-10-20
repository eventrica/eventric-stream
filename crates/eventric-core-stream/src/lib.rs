#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod append;
mod query;
mod stream;

// =================================================================================================
// Eventric Core Stream
// =================================================================================================

// Re-Exports

pub use self::{
    append::{
        AppendCondition,
        AppendConditionBuilder,
    },
    query::{
        QueryCondition,
        QueryConditionBuilder,
    },
    stream::{
        Stream,
        StreamConfigurator,
    },
};
