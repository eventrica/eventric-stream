//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::event::tag;

// =================================================================================================
// Derive
// =================================================================================================

macro_rules! emit_impl_or_error {
    ($e:expr) => {
        match $e {
            Ok(val) => val.into_token_stream(),
            Err(err) => err.write_errors(),
        }
    };
}

// -------------------------------------------------------------------------------------------------

// Tag

#[doc(hidden)]
#[must_use]
pub fn tag(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(tag::macros::Tag::new(input))
}
