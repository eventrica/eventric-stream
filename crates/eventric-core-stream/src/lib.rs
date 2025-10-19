#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![feature(trait_alias)]

mod stream;

// =================================================================================================
// Eventric Core Stream
// =================================================================================================

// Re-Exports

pub use self::stream::{
    Events,
    SequencedEvents,
    Stream,
    StreamConfigurator,
};
