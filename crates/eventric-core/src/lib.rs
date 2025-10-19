#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub use eventric_core_model::{
    Condition,
    // ConditionBuilder,
    Data,
    Descriptor,
    // DescriptorRef,
    Event,
    Identifier,
    Position,
    Query,
    QueryItem,
    // SequencedEventRef,
    Specifier,
    Tag,
    // TagRef,
    Version,
};
pub use eventric_core_stream::{
    // Events,
    // SequencedEvents,
    Stream,
    // StreamConfigurator,
};
