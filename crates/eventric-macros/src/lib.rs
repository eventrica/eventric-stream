//! Procedural macros for the `eventric` crate: the `tag!` function-like macro
//! and the `Event`/`Action`/`Projection` derives. These are re-exported from
//! `eventric`, so consumers never name this crate directly.

#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![allow(missing_docs)]

pub(crate) mod action;
pub(crate) mod event;
pub(crate) mod projection;
pub(crate) mod tag;
pub(crate) mod util;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use crate::{
    action::Action,
    event::Event,
    projection::Projection,
    tag::Tag,
};

// =================================================================================================
// Eventric Macros
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

/// Creates an `eventric::event::Tag` from an identifier-compatible prefix and a
/// value which implements `Display`, e.g. `tag!(student, &self.id)?`.
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Tag::new(input.into())).into()
}

// Action

#[proc_macro_derive(Action, attributes(action))]
pub fn action(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Action::new(&parse_macro_input!(input))).into()
}

// Event

#[proc_macro_derive(Event, attributes(event))]
pub fn event(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Event::new(&parse_macro_input!(input))).into()
}

// Projection

#[proc_macro_derive(Projection, attributes(projection))]
pub fn projection(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Projection::new(&parse_macro_input!(input))).into()
}
