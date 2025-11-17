//! See the `eventric-stream` crate for full documentation, including
//! crate-level documentation.

#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]

use eventric_stream_core::macros;
use proc_macro::TokenStream;

// =================================================================================================
// Eventric Stream Macros
// =================================================================================================

/// Attempts to create a new [`Tag`][tag] instance, using a provided
/// identifier-compatible prefix value, and a value which implements display.
///
/// ```ignore
/// tag!(tag_prefix, "tag_value")?
/// ```
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    macros::function::tag(input.into()).into()
}
