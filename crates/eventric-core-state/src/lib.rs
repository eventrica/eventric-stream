#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod context;
mod io;

// =================================================================================================
// Eventric Core State
// =================================================================================================

// Re-Exports

pub use self::{
    context::{
        Context,
        Keyspaces,
    },
    io::{
        Read,
        Write,
    },
};
