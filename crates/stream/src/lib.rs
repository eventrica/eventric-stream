#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![doc = include_utils::include_md!("README.md:overview")]

// =================================================================================================
// Eventric Stream
// =================================================================================================

// Re-Exports

pub mod error {
    //! The [`error`][self] module contains the common [`Error`] type and the
    //! [`Conflict`] marker (attached when an append is rejected by its
    //! condition), along with the crate [`Result`] alias.

    pub use eventric_stream_core::stream::{
        Conflict,
        Error,
        Result,
    };
}

pub mod event {
    //! The [`event`][self] module contains the constituent components for
    //! events: the payload [`Data`], the [`Type`] (a [`Name`] plus
    //! [`Version`]), and [`Tag`]s, along with the [`tag`] macro.

    pub use eventric_stream_core::event::{
        Data,
        Event,
        Facets,
        Name,
        Tag,
        Type,
        Version,
    };
    pub use eventric_stream_macros::tag;
}

#[rustfmt::skip]
pub mod stream {
    //! The [`stream`][self] module contains the core stream abstraction, the
    //! [`Reader`]/[`Writer`] split and the multi-threaded [`Owner`]/[`Proxy`]
    //! wrapper, the [`Append`] and [`Select`] operations, and the [`Condition`]
    //! query/concurrency model.

    pub use eventric_stream_core::stream::{
        Append,
        Builder,
        Condition,
        EventAndMask,
        Mask,
        Position,
        Reader,
        Select,
        SelectIter,
        Selection,
        Selector,
        Stream,
        Timestamp,
        TypeSelector,
        VersionSelector,
        Writer,
    };

    pub use eventric_stream_multi_thread::{
        owner::Owner,
        proxy::Proxy,
    };
}

pub use eventric_stream_core::utils::temp_path;
