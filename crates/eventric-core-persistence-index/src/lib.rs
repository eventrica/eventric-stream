#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod configuration;
mod iter;
mod operation;

// =================================================================================================
// Eventric Core Persistence Index
// =================================================================================================

// Re-Export

pub use self::{
    configuration::keyspace,
    operation::{
        insert,
        query,
    },
};
