//! See the `eventric-stream` crate for full documentation, including
//! crate-level documentation.

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![deny(unsafe_code)]
#![feature(bool_to_result)]
#![feature(exclusive_wrapper)]

pub mod error;
pub mod event;
pub mod event_new;
pub mod stream;
pub mod stream_new;
pub mod utils;

// =================================================================================================
// Eventric Stream Core
// =================================================================================================
