use std::collections::HashSet;

use heck::AsSnakeCase;
use proc_macro2::TokenStream;
use quote::{
    ToTokens,
    TokenStreamExt as _,
    format_ident,
    quote,
};
use syn::{
    DeriveInput,
    Expr,
    ExprCall,
    ExprPath,
    Ident,
    Path,
    Token,
    braced,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    token::Comma,
};

// =================================================================================================
// Action
// =================================================================================================

#[derive(Debug)]
pub struct Action {
    ident: Ident,
    projections: Vec<ProjectionEntry>,
}

impl Action {
    pub fn new(input: &DeriveInput) -> darling::Result<Self> {
        let attribute = input
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("action"))
            .ok_or_else(|| {
                darling::Error::custom("missing `#[action(..)]` attribute").with_span(&input.ident)
            })?;

        let args = attribute
            .parse_args::<ActionArgs>()
            .map_err(darling::Error::from)?;

        Ok(Self {
            ident: input.ident.clone(),
            projections: args.projections,
        })
    }
}

// ActionArgs — the `#[action(..)]` body: `projections: { <field>: <ctor>, .. }`
// (omit entirely for an action with no projections).

#[derive(Debug)]
struct ActionArgs {
    projections: Vec<ProjectionEntry>,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                projections: Vec::new(),
            });
        }

        let key = input.parse::<Ident>()?;
        if key != "projections" {
            return Err(syn::Error::new(
                key.span(),
                format!("unknown key `{key}`; expected `projections`"),
            ));
        }
        input.parse::<Token![:]>()?;

        let content;
        braced!(content in input);
        let projections: Vec<ProjectionEntry> =
            Punctuated::<ProjectionEntry, Comma>::parse_terminated(&content)?
                .into_iter()
                .collect();

        // Each field name keys a context field, so a duplicate would emit two
        // struct fields of the same name — a targeted error here beats the opaque
        // `field is already declared` rustc would point at the generated tokens.
        let mut seen = HashSet::new();
        for entry in &projections {
            if !seen.insert(entry.field_name.to_string()) {
                return Err(syn::Error::new(
                    entry.field_name.span(),
                    format!("duplicate projection field `{}`", entry.field_name),
                ));
            }
        }

        Ok(Self { projections })
    }
}

// ProjectionEntry — `<field_name>: <Type>::new(..)`. The field name is the
// explicit key (so two slots of the same projection type can coexist); the
// field type is read from the constructor's path (its final segment dropped).

#[derive(Debug)]
struct ProjectionEntry {
    field_name: Ident,
    field_type: Path,
    constructor: Expr,
}

impl Parse for ProjectionEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let field_name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let constructor = input.parse::<Expr>()?;
        let field_type = projection_type(&constructor)?;

        Ok(Self {
            field_name,
            field_type,
            constructor,
        })
    }
}

// The projection type, read from a `Type::new(..)` constructor — the call's
// path with its final (`::new`-style) segment dropped.
fn projection_type(constructor: &Expr) -> syn::Result<Path> {
    if let Expr::Call(ExprCall { func, .. }) = constructor
        && let Expr::Path(ExprPath {
            path, qself: None, ..
        }) = func.as_ref()
        && path.segments.len() >= 2
    {
        return Ok(Path {
            leading_colon: path.leading_colon,
            segments: path
                .segments
                .iter()
                .take(path.segments.len() - 1)
                .cloned()
                .collect(),
        });
    }

    Err(syn::Error::new_spanned(
        constructor,
        "projection constructor must be a `Type::new(..)` call, so the projection type can be \
         inferred",
    ))
}

// -------------------------------------------------------------------------------------------------

// Codegen

impl Action {
    fn action(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            impl ::eventric_model::action::Action for #ident {}
        }
    }

    fn projections(&self) -> TokenStream {
        let ident = &self.ident;
        let module = format_ident!("{}", AsSnakeCase(ident.to_string()).to_string());

        let field_name = self.projections.iter().map(|p| &p.field_name);
        // The struct lives in a child module, so a relative projection type is
        // re-rooted at the parent with `super::`.
        let field_type = self
            .projections
            .iter()
            .map(|p| projection_field(&p.field_type));
        let field_init = self.projections.iter().map(ProjectionInit);

        quote! {
            // Generated machinery — not the user's to document.
            #[allow(missing_docs)]
            pub mod #module {
                #[derive(Debug)]
                pub struct Projections {
                  #(pub #field_name: #field_type),*
                }
            }

            #[automatically_derived]
            impl ::eventric_model::action::Context for #ident {
                type Projections = #module::Projections;

                fn projections(&self) -> Self::Projections {
                    #module::Projections {
                      #(#field_init),*
                    }
                }
            }
        }
    }

    fn select(&self) -> TokenStream {
        let ident = &self.ident;

        let field_name = self.projections.iter().map(|p| &p.field_name);

        quote! {
            #[automatically_derived]
            impl ::eventric_model::action::Select for #ident {
                fn select(
                    &self,
                    projections: &Self::Projections
                ) -> ::std::result::Result<
                    ::std::vec::Vec<::eventric_stream::stream::operate::Selection>,
                    ::error_stack::Report<::eventric_model::error::Error>
                > {
                    // Each projection contributes one `Selection` per named
                    // selection; flattened in projection order, they are the mask
                    // layout `update` slices against.
                    let mut selections = ::std::vec::Vec::new();

                  #(selections.extend(
                        ::eventric_model::projection::Select::select(&projections.#field_name)?
                    );)*

                    ::std::result::Result::Ok(selections)
                }
            }
        }
    }

    fn update(&self) -> TokenStream {
        let ident = &self.ident;

        let field_name = self.projections.iter().map(|p| &p.field_name);
        let field_type = self.projections.iter().map(|p| &p.field_type);

        quote! {
            #[automatically_derived]
            impl ::eventric_model::action::Update for #ident {
                fn update(
                    &self,
                    projections: &mut Self::Projections,
                    event: &::eventric_stream::stream::operate::select::EventAndMask
                ) -> ::std::result::Result<(), ::error_stack::Report<::eventric_model::error::Error>> {
                    // Walk the mask one projection-block at a time: each projection
                    // owns `SELECTIONS` consecutive bits. Decode once (shared across
                    // projections of the same event type) and hand each projection
                    // just its own slice of the mask.
                    let mut dispatch_event = ::std::option::Option::None;
                    let mut base = 0usize;

                  #({
                        let count = <#field_type as ::eventric_model::projection::Select>::SELECTIONS;
                        let mask = &::std::convert::AsRef::<[bool]>::as_ref(&event.mask)[base..base + count];

                        if mask.contains(&true) {
                            if dispatch_event.is_none() {
                                dispatch_event = ::eventric_model::projection::Recognize::recognize(
                                    &projections.#field_name,
                                    event,
                                )?;
                            }

                            if let ::std::option::Option::Some(dispatch_event) = dispatch_event.as_ref() {
                                ::eventric_model::projection::Dispatch::dispatch(
                                    &mut projections.#field_name,
                                    mask,
                                    dispatch_event,
                                );
                            }
                        }

                        base += count;
                    })*

                    ::std::result::Result::Ok(())
                }
            }
        }
    }
}

impl ToTokens for Action {
    #[rustfmt::skip]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.action());
        tokens.append_all(self.projections());
        tokens.append_all(self.select());
        tokens.append_all(self.update());
    }
}

// -------------------------------------------------------------------------------------------------

// Composites

// ProjectionInit — `<field_name>: <constructor>`, built inside
// `projections(&self)`, so the constructor's `self` is the action.
struct ProjectionInit<'a>(&'a ProjectionEntry);

impl ToTokens for ProjectionInit<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProjectionInit(entry) = *self;
        let field_name = &entry.field_name;
        let constructor = &entry.constructor;

        tokens.append_all(quote! {
            #field_name: #constructor
        });
    }
}

// The projections struct's field type. The struct lives in a child module, so a
// relative projection type is re-rooted at the parent with `super::`; a
// crate-rooted or leading-`::` path is already absolute and used unchanged.
// (Projections are non-generic, so a relative path here never carries a generic
// argument that the head-only `super::` would leave unresolved — see FUTURE.md
// §2.)
fn projection_field(ty: &Path) -> TokenStream {
    let absolute = ty.leading_colon.is_some()
        || ty
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "crate");

    if absolute {
        quote! { #ty }
    } else {
        quote! { super::#ty }
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_projections() {
        let args: ActionArgs =
            syn::parse_str("projections: { balance: Balance::new(&self.id) }").unwrap();

        assert_eq!(args.projections.len(), 1);
        assert_eq!(args.projections[0].field_name, "balance");
    }

    #[test]
    fn empty_body_is_no_projections() {
        let args: ActionArgs = syn::parse_str("").unwrap();

        assert!(args.projections.is_empty());
    }

    #[test]
    fn duplicate_field_rejected() {
        let err = syn::parse_str::<ActionArgs>(
            "projections: { balance: Balance::new(&self.id), balance: Other::new() }",
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("duplicate projection field `balance`")
        );
    }

    #[test]
    fn non_constructor_rejected() {
        // The projection type is read from the constructor's path, so a value that
        // is not a `Type::new(..)`-style call cannot yield one.
        let err = syn::parse_str::<ActionArgs>("projections: { balance: some_fn() }").unwrap_err();

        assert!(err.to_string().contains("Type::new"));
    }

    #[test]
    fn unknown_key_rejected() {
        let err = syn::parse_str::<ActionArgs>("selections: { }").unwrap_err();

        assert!(err.to_string().contains("expected `projections`"));
    }
}
