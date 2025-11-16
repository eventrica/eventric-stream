//! See the `eventric-stream` crate for full documentation, including
//! crate-level documentation.

#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![feature(exclusive_wrapper)]

pub mod error;
pub mod event;
pub mod macros;
pub mod stream;
pub mod utils;

// =================================================================================================
// Eventric Stream
// =================================================================================================
