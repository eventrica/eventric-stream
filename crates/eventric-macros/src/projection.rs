#![allow(clippy::needless_continue)]

use std::collections::{
    HashMap,
    HashSet,
};

use darling::{
    FromDeriveInput,
    FromMeta,
};
use proc_macro2::TokenStream;
use quote::{
    ToTokens,
    TokenStreamExt as _,
    format_ident,
    quote,
};
use syn::{
    DeriveInput,
    Ident,
    Path,
};

use crate::{
    event,
    util::List,
};

// =================================================================================================
// Projection
// =================================================================================================

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(projection), supports(struct_named))]
pub struct Projection {
    ident: Ident,
    #[darling(multiple, rename = "select")]
    selectors: Vec<Selector>,
}

impl Projection {
    pub fn new(input: &DeriveInput) -> darling::Result<Self> {
        Self::from_derive_input(input)
    }
}

impl Projection {
    fn events(&self) -> Vec<Path> {
        self.selectors
            .iter()
            .flat_map(|selector| selector.events.as_ref())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn selectors(&self) -> &Vec<Selector> {
        &self.selectors
    }
}

impl Projection {
    pub fn dispatch(&self) -> TokenStream {
        let ident = &self.ident;
        let event = self.events();

        let dispatch_trait = format_ident!("{ident}Dispatch");

        quote! {
            pub trait #dispatch_trait: #(::eventric_domain::projection::Project<#event>)+* {}

            #[automatically_derived]
            impl #dispatch_trait for #ident {}

            #[automatically_derived]
            impl ::eventric_domain::projection::Dispatch for #ident {
                fn dispatch(&mut self, event: &::eventric_domain::projection::DispatchEvent) {
                    match event {
                      #(_ if let std::option::Option::Some(event) = event.as_projection_event::<#event>() => self.project(event),)*
                        _ => {}
                    }
                }
            }
        }
    }

    fn projection(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            impl ::eventric_domain::projection::Projection for #ident {}
        }
    }

    fn recognize(&self) -> TokenStream {
        let ident = &self.ident;
        let event = self.events();

        let recognize_match_arm = event.iter().map(RecognizeMatchArm);

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::projection::Recognize for #ident {
                fn recognize(
                    &self,
                    event: &::eventric_stream::stream::operate::select::EventAndMask
                ) -> ::std::result::Result<
                    ::std::option::Option<::eventric_domain::projection::DispatchEvent>,
                    ::error_stack::Report<::eventric_domain::error::Error>
                > {
                    let event = match event {
                     #(#recognize_match_arm),*
                        _ => ::std::option::Option::None,
                    };

                    ::std::result::Result::Ok(event)
                }
            }
        }
    }

    fn select(&self) -> TokenStream {
        let ident = &self.ident;
        let selectors = self.selectors();

        let selector_initialize = selectors
            .iter()
            .map(|selector| SelectorInitialize(ident, selector));

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::projection::Select for #ident {
                fn select(&self) -> ::std::result::Result<
                    ::eventric_stream::stream::operate::Selection,
                    ::error_stack::Report<::eventric_domain::error::Error>
                > {
                    ::std::result::Result::Ok(::eventric_stream::stream::operate::Selection::new([
                     #(#selector_initialize),*
                    ]))
                }
            }
        }
    }
}

impl ToTokens for Projection {
    #[rustfmt::skip]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.projection());
        tokens.append_all(self.dispatch());
        tokens.append_all(self.recognize());
        tokens.append_all(self.select());
    }
}

// -------------------------------------------------------------------------------------------------

// Recognize

pub struct RecognizeMatchArm<'a>(&'a Path);

impl ToTokens for RecognizeMatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let RecognizeMatchArm(event) = *self;

        tokens.append_all(quote! {
            _ if event.event.facets().ty().name() == &<#event as ::eventric_domain::event::Identifier>::type_name()? => {
                ::std::option::Option::Some(
                    ::eventric_domain::projection::DispatchEvent::from_event::<#event>(event)?
                )
            }
        });
    }
}

// -------------------------------------------------------------------------------------------------

// Select

#[derive(Debug, FromMeta)]
pub struct Selector {
    pub events: List<Path>,
    #[darling(map = "event::tags_map")]
    pub filter: Option<HashMap<Ident, List<event::Tag>>>,
}

// Selector Composites

pub struct SelectorInitialize<'a>(pub &'a Ident, pub &'a Selector);

impl ToTokens for SelectorInitialize<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SelectorInitialize(ident, selector) = *self;

        let event = selector.events.as_ref();
        let tag = event::tags_fold(ident, selector.filter.as_ref());

        if tag.is_empty() {
            tokens.append_all(quote! {
                ::eventric_stream::stream::operate::select::Selector::types(
                    [#(<#event as ::eventric_domain::event::Specifier>::specifier()?),*]
                )
            });
        } else {
            tokens.append_all(quote! {
                ::eventric_stream::stream::operate::select::Selector::types_and_tags(
                    [#(<#event as ::eventric_domain::event::Specifier>::specifier()?),*],
                    [#(::error_stack::ResultExt::change_context(#tag, ::eventric_domain::error::Error)?),*]
                )
            });
        }
    }
}
