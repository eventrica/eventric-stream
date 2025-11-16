#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![allow(missing_docs)]

use eventric_stream_core::macros;
use proc_macro::TokenStream;

// =================================================================================================
// Eventric Surface Macro
// =================================================================================================

#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    macros::function::tag(input.into()).into()
}
