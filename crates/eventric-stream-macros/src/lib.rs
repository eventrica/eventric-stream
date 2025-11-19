//! See the `eventric-stream` crate for full documentation, including
//! crate-level documentation.

#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]

pub(crate) mod event;

use proc_macro::TokenStream;
use quote::ToTokens;

use crate::event::tag;

// =================================================================================================
// Eventric Stream Macros
// =================================================================================================

macro_rules! emit_impl_or_error {
    ($e:expr) => {
        match $e {
            Ok(val) => val.into_token_stream(),
            Err(err) => err.write_errors(),
        }
    };
}

// Tag

/// Attempts to create a new [`Tag`][tag] instance, using a provided
/// identifier-compatible prefix value, and a value which implements display.
///
/// ```ignore
/// tag!(tag_prefix, "tag_value")?
/// ```
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(tag::TagFunction::new(input.into())).into()
}
