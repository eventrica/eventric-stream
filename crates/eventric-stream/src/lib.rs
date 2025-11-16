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
    //! The [`error`][self] module contains the common [`Error`][error] type
    //! used throughout `eventric-stream`.
    //!
    //! [error]: enum@crate::error::Error

    pub use eventric_stream_core::error::Error;
}

pub mod event {
    //! The [`event`][self] module contains the constituent components for
    //! events, both pre- and post- stream append, as well as types related
    //! to specifying events within queries.

    pub use eventric_stream_core::event::{
        AnyRange,
        Data,
        EphemeralEvent,
        Identifier,
        PersistentEvent,
        Position,
        Specifier,
        Tag,
        Version,
    };
    pub use eventric_stream_macros::tag;
}

pub mod stream {
    //! The [`stream`][self] module contains the core stream abstraction,
    //! along with support for configuring and opening stream instances.
    //! Sub-modules contain types related to appending events to the stream,
    //! and querying the stream for previously appended events.

    pub use eventric_stream_core::stream::{
        Builder,
        Stream,
    };

    pub mod append {
        //! The [`append`][self] module contains types and functionality related
        //! to the [`Stream::append`] operation, such as the
        //! append-specific [`Condition`] type.

        pub use eventric_stream_core::stream::append::Condition;
    }

    pub mod query {
        //! The [`query`][self] module contains types and functionality related
        //! to the [`Stream::query`] operation, such as the [`Cache`],
        //! query-specific [`Condition`], and [`Options`] types, as well
        //! as the fundamental [`Query`] type and its components.

        pub use eventric_stream_core::stream::query::{
            Cache,
            Condition,
            Options,
            Query,
            QueryIterator,
            Selector,
            Specifiers,
            Tags,
        };
    }
}

pub use eventric_stream_core::temp_path;
