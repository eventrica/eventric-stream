#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod configuration;
mod data;
mod operation;

// =================================================================================================
// Eventric Core Persistence Data
// =================================================================================================

// Re-Exports

pub use self::{
    configuration::keyspace,
    data::Data,
    operation::{
        get,
        insert,
        is_empty,
        len,
    },
};
