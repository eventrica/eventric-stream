#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod configuration;
mod operation;

// =================================================================================================
// Eventric Core Persistence Reference
// =================================================================================================

// Re-Exports

pub use self::{
    configuration::keyspace,
    operation::{
        get_identifier,
        get_tag,
        insert,
    },
};
