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
    //! The [`error`][self] module contains the common [`Error`] type
    //! used throughout `eventric-stream`.

    pub use eventric_stream_core::error::Error;
}

pub mod event {
    //! The [`event`][self] module contains the constituent components for
    //! events, both pre- and post- stream append, as well as types related
    //! to specifying events within queries.

    pub use eventric_stream_core::event::{
        CandidateEvent,
        Data,
        Event,
        Identifier,
        Position,
        Range,
        Specifier,
        Tag,
        Timestamp,
        Version,
    };
    pub use eventric_stream_macros::tag;
}

#[rustfmt::skip]
pub mod stream {
    //! The [`stream`][self] module contains the core stream abstraction,
    //! along with support for configuring and opening stream instances.
    //! Sub-modules contain types related to appending events to the stream,
    //! and querying the stream for previously appended events.

    pub use eventric_stream_core::stream::{
        Builder,
        Reader,
        Stream,
        Writer,
    };
    
    pub use eventric_stream_multi_thread::{
        owner::Owner,
        proxy::Proxy,
    };

    pub mod append {
        //! The [`append`][self] module contains types and functionality related
        //! to the [`Stream::append`] operation, such as the
        //! append-specific [`Condition`] type.

        pub use eventric_stream_core::stream::append::{
            Append,
            AppendSelect,
        };
    }

    pub mod iterate {
        //! The [`iterate`][self] module contains types and functionality
        //! related to iteration over a stream, which supports multiple models
        //! of operation.

        pub use eventric_stream_core::stream::iterate::{
            Iter,
            Iterate,
        };
    }

    pub mod select {
        //! The [`select`][self] module contains types and functionality related
        //! to the construction and use instances of [`Stream::query`], used as
        //! part of iteration and append operations via the respective
        //! condition models.

        pub use eventric_stream_core::stream::select::{
            EventAndMask,
            IterSelect,
            IterSelectMultiple,
            Mask,
            Prepared,
            PreparedMultiple,
            Select,
            Selection,
            Selections,
            Selector,
            Specifiers,
            Tags,
        };
    }
}

pub use eventric_stream_core::utils::temp_path;
