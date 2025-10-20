#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub use eventric_core_model::{
    Data,
    Event,
    Identifier,
    Position,
    Query,
    QueryItem,
    SequencedEvent,
    Specifier,
    Tag,
    Version,
};
pub use eventric_core_stream::{
    AppendCondition,
    QueryCache,
    QueryCondition,
    Stream,
};
