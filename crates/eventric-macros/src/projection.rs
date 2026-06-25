use std::collections::{
    HashMap,
    HashSet,
};

use heck::{
    AsSnakeCase,
    AsUpperCamelCase,
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
    Token,
    braced,
    bracketed,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    token::Comma,
};

use crate::event::{
    TagEntry,
    TagInitialize,
};

// =================================================================================================
// Projection
// =================================================================================================

#[derive(Debug)]
pub struct Projection {
    ident: Ident,
    selections: Vec<NamedSelection>,
}

impl Projection {
    pub fn new(input: &DeriveInput) -> darling::Result<Self> {
        let attribute = input
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("projection"))
            .ok_or_else(|| {
                darling::Error::custom("missing `#[projection(..)]` attribute")
                    .with_span(&input.ident)
            })?;

        let args = attribute
            .parse_args::<ProjectionArgs>()
            .map_err(darling::Error::from)?;

        if args.selections.is_empty() {
            return Err(darling::Error::custom(
                "`#[projection(..)]` needs a `selections: { .. }` with at least one named \
                 selection",
            )
            .with_span(attribute));
        }

        Self::validate(&args.selections)?;

        Ok(Self {
            ident: input.ident.clone(),
            selections: args.selections,
        })
    }

    // Reject selection/event collisions that would otherwise surface as opaque
    // downstream compile errors rather than a targeted macro diagnostic: selection
    // names (and the enums they generate) must be distinct, and within a selection
    // no two events may collapse to the same variant name.
    fn validate(selections: &[NamedSelection]) -> darling::Result<()> {
        let mut names = HashSet::new();
        let mut enums = HashMap::new();

        for selection in selections {
            let name = selection.name.to_string();
            if !names.insert(name.clone()) {
                return Err(
                    darling::Error::custom(format!("duplicate selection `{name}`"))
                        .with_span(&selection.name),
                );
            }

            let enum_name = enum_ident(&selection.name).to_string();
            if let Some(other) = enums.insert(enum_name.clone(), name.clone()) {
                return Err(darling::Error::custom(format!(
                    "selections `{other}` and `{name}` both generate the enum `{enum_name}`; \
                     rename one"
                ))
                .with_span(&selection.name));
            }

            let mut variants = HashSet::new();
            for event in &selection.events {
                let variant = variant(event).to_string();
                if !variants.insert(variant.clone()) {
                    return Err(darling::Error::custom(format!(
                        "selection `{name}`: two events map to the same enum variant `{variant}`; \
                         import one under a different name"
                    ))
                    .with_span(event));
                }
            }
        }

        Ok(())
    }
}

// ProjectionArgs — the `#[projection(..)]` body: `selections: { <named
// selection>, .. }`.

#[derive(Debug)]
struct ProjectionArgs {
    selections: Vec<NamedSelection>,
}

impl Parse for ProjectionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key = input.parse::<Ident>()?;
        if key != "selections" {
            return Err(syn::Error::new(
                key.span(),
                format!("unknown key `{key}`; expected `selections`"),
            ));
        }
        input.parse::<Token![:]>()?;

        let content;
        braced!(content in input);
        let selections = Punctuated::<NamedSelection, Comma>::parse_terminated(&content)?
            .into_iter()
            .collect();

        Ok(Self { selections })
    }
}

// NamedSelection — `<name>: { events: [<Type>, ..], filter: { <prefix>:
// <value>, .. } }` (`filter` optional).

#[derive(Debug)]
struct NamedSelection {
    name: Ident,
    events: Vec<Path>,
    filter: Vec<TagEntry>,
}

impl Parse for NamedSelection {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;

        let body;
        braced!(body in input);

        let mut events = None;
        let mut filter = None;

        while !body.is_empty() {
            let key = body.parse::<Ident>()?;
            body.parse::<Token![:]>()?;

            match &*key.to_string() {
                "events" => {
                    let content;
                    bracketed!(content in body);
                    let list = Punctuated::<Path, Comma>::parse_terminated(&content)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    if events.replace(list).is_some() {
                        return Err(syn::Error::new(key.span(), "duplicate `events`"));
                    }
                }
                "filter" => {
                    let content;
                    braced!(content in body);
                    let list = Punctuated::<TagEntry, Comma>::parse_terminated(&content)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    if filter.replace(list).is_some() {
                        return Err(syn::Error::new(key.span(), "duplicate `filter`"));
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{other}`; expected `events` or `filter`"),
                    ));
                }
            }

            if body.is_empty() {
                break;
            }

            body.parse::<Token![,]>()?;
        }

        let events = events.ok_or_else(|| {
            syn::Error::new(
                name.span(),
                format!("selection `{name}` is missing its `events` list"),
            )
        })?;

        if events.is_empty() {
            return Err(syn::Error::new(
                name.span(),
                format!("selection `{name}` has an empty `events` list"),
            ));
        }

        Ok(Self {
            name,
            events,
            filter: filter.unwrap_or_default(),
        })
    }
}

// -------------------------------------------------------------------------------------------------

// Codegen

impl Projection {
    // The companion module's name: the projection ident, snake_cased.
    fn module(&self) -> Ident {
        format_ident!("{}", AsSnakeCase(self.ident.to_string()).to_string())
    }

    // The union of every selection's event types — what `Recognize` decodes.
    // Deduplicated by path, preserving first-occurrence order (so the generated
    // match arms are deterministic).
    fn events(&self) -> Vec<Path> {
        let mut seen = HashSet::new();

        self.selections
            .iter()
            .flat_map(|selection| selection.events.iter())
            .filter(|event| seen.insert(event.to_token_stream().to_string()))
            .cloned()
            .collect()
    }

    fn projection(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::projection::Projection for #ident {}
        }
    }

    // The companion module: one borrowed enum per selection + the `Project` trait
    // (one method per selection) the user implements.
    fn companion(&self) -> TokenStream {
        let module = self.module();

        let enums = self.selections.iter().map(|selection| {
            let enum_ident = enum_ident(&selection.name);
            let variants = selection.events.iter().map(|event| {
                let variant = variant(event);
                let field = enum_field(event);
                quote! { #variant(&'a #field) }
            });

            quote! {
                pub enum #enum_ident<'a> {
                    #(#variants),*
                }
            }
        });

        let methods = self.selections.iter().map(|selection| {
            let method = &selection.name;
            let enum_ident = enum_ident(&selection.name);

            quote! {
                fn #method(
                    &mut self,
                    event: ::eventric_domain::projection::ProjectionEvent<#enum_ident<'_>>,
                );
            }
        });

        quote! {
            pub mod #module {
                #(#enums)*

                pub trait Project {
                    #(#methods)*
                }
            }
        }
    }

    fn select(&self) -> TokenStream {
        let ident = &self.ident;
        let count = self.selections.len();
        let selections = self.selections.iter().map(SelectionExpr);

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::projection::Select for #ident {
                const SELECTIONS: usize = #count;

                fn select(&self) -> ::std::result::Result<
                    ::std::vec::Vec<::eventric_stream::stream::operate::Selection>,
                    ::error_stack::Report<::eventric_domain::error::Error>
                > {
                    ::std::result::Result::Ok(::std::vec![
                        #(#selections),*
                    ])
                }
            }
        }
    }

    fn recognize(&self) -> TokenStream {
        let ident = &self.ident;
        let arms = self.events();
        let arms = arms.iter().map(RecognizeMatchArm);

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
                        #(#arms)*
                        _ => ::std::option::Option::None,
                    };

                    ::std::result::Result::Ok(event)
                }
            }
        }
    }

    fn dispatch(&self) -> TokenStream {
        let ident = &self.ident;
        let module = self.module();
        let arms = self
            .selections
            .iter()
            .enumerate()
            .map(|(index, selection)| DispatchArm {
                module: &module,
                index,
                selection,
            });

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::projection::Dispatch for #ident {
                fn dispatch(
                    &mut self,
                    mask: &[bool],
                    event: &::eventric_domain::projection::DispatchEvent
                ) {
                    #(#arms)*
                }
            }
        }
    }
}

impl ToTokens for Projection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.projection());
        tokens.append_all(self.companion());
        tokens.append_all(self.select());
        tokens.append_all(self.recognize());
        tokens.append_all(self.dispatch());
    }
}

// -------------------------------------------------------------------------------------------------

// Naming

fn enum_ident(name: &Ident) -> Ident {
    format_ident!("{}", AsUpperCamelCase(name.to_string()).to_string())
}

fn variant(event: &Path) -> &Ident {
    &event
        .segments
        .last()
        .expect("an event path has a final segment")
        .ident
}

// The enum's variant field type. The enum lives in a child module, so a
// relative path is re-rooted at the parent with `super::`; a crate-rooted or
// leading-`::` path is already absolute and used unchanged.
fn enum_field(event: &Path) -> TokenStream {
    let absolute = event.leading_colon.is_some()
        || event
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "crate");

    if absolute {
        quote! { #event }
    } else {
        quote! { super::#event }
    }
}

// -------------------------------------------------------------------------------------------------

// Composites

// SelectionExpr — one named selection becomes one `Selection`.
struct SelectionExpr<'a>(&'a NamedSelection);

impl ToTokens for SelectionExpr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SelectionExpr(selection) = *self;
        let event = &selection.events;

        let selector = if selection.filter.is_empty() {
            quote! {
                ::eventric_stream::stream::operate::select::Selector::types(
                    [#(<#event as ::eventric_domain::event::Specifier>::specifier()?),*]
                )
            }
        } else {
            let tag = selection.filter.iter().flat_map(|entry| {
                entry
                    .values
                    .iter()
                    .map(move |value| TagInitialize(&entry.prefix, value))
            });

            quote! {
                ::eventric_stream::stream::operate::select::Selector::types_and_tags(
                    [#(<#event as ::eventric_domain::event::Specifier>::specifier()?),*],
                    [#(::error_stack::ResultExt::change_context(#tag, ::eventric_domain::error::Error)?),*]
                )
            }
        };

        tokens.append_all(quote! {
            ::eventric_stream::stream::operate::Selection::new([#selector])
        });
    }
}

// RecognizeMatchArm — match a persisted event to one event type by hashed name.
struct RecognizeMatchArm<'a>(&'a Path);

impl ToTokens for RecognizeMatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let RecognizeMatchArm(event) = *self;

        tokens.append_all(quote! {
            _ if event.event.facets().ty().name()
                == &<#event as ::eventric_domain::event::Identifier>::type_name()? =>
            {
                ::std::option::Option::Some(
                    ::eventric_domain::projection::DispatchEvent::from_event::<#event>(event)?
                )
            }
        });
    }
}

// DispatchArm — for the selection at `index`, if its mask bit is set, build its
// enum from the once-decoded payload and call its method.
struct DispatchArm<'a> {
    module: &'a Ident,
    index: usize,
    selection: &'a NamedSelection,
}

impl ToTokens for DispatchArm<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let DispatchArm {
            module,
            index,
            selection,
        } = *self;

        let enum_ident = enum_ident(&selection.name);
        let method = &selection.name;

        let branch = selection.events.iter().map(|event| {
            let variant = variant(event);

            quote! {
                if let ::std::option::Option::Some(value) = event.event.downcast_ref::<#event>() {
                    ::std::option::Option::Some(#module::#enum_ident::#variant(value))
                }
            }
        });

        tokens.append_all(quote! {
            if mask[#index] {
                let matched = #(#branch else)* {
                    ::std::option::Option::None
                };

                if let ::std::option::Option::Some(matched) = matched {
                    <Self as #module::Project>::#method(
                        self,
                        ::eventric_domain::projection::ProjectionEvent::new(
                            matched,
                            event.position,
                            event.timestamp,
                        ),
                    );
                }
            }
        });
    }
}
