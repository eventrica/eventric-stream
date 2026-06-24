use std::collections::HashMap;

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
    Ident,
    Token,
    bracketed,
    parse::{
        Parse,
        ParseStream,
        discouraged::Speculative as _,
    },
    punctuated::Punctuated,
    token::Comma,
};

use crate::util::List;

// =================================================================================================
// Event
// =================================================================================================

#[derive(Debug)]
pub struct Event {
    ident: Ident,
    identifier: String,
    tags: Vec<TagEntry>,
}

impl Event {
    pub fn new(input: &DeriveInput) -> darling::Result<Self> {
        let attribute = input
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("event"))
            .ok_or_else(|| {
                darling::Error::custom("missing `#[event(..)]` attribute").with_span(&input.ident)
            })?;

        // The body is the declarative grammar `identifier: <ident>, tags:
        // [<prefix>: <value>, ..]` (see `EventArgs`), hand-parsed. The identifier
        // is a single ident, so it is already a valid Rust identifier; the
        // generated `Identifier::type_name` calls `Name::new` at runtime as the
        // (effectively unreachable) backstop.
        let args = attribute
            .parse_args::<EventArgs>()
            .map_err(darling::Error::from)?;

        // Raised here (not in `EventArgs::parse`) so the span points at the whole
        // `#[event(..)]` attribute, rather than at whichever entry happened to be
        // consumed last.
        let identifier = args.identifier.ok_or_else(|| {
            darling::Error::custom(
                "`#[event(..)]` is missing the required `identifier: <ident>` entry",
            )
            .with_span(attribute)
        })?;

        Ok(Self {
            ident: input.ident.clone(),
            identifier: identifier.to_string(),
            tags: args.tags,
        })
    }
}

// EventArgs — the `#[event(..)]` body grammar: `identifier: <ident>` plus an
// optional `tags: [<prefix>: <value>, ..]`, as comma-separated `key: value`
// entries.

#[derive(Debug)]
struct EventArgs {
    identifier: Option<Ident>,
    tags: Vec<TagEntry>,
}

impl Parse for EventArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut identifier = None;
        let mut tags = None;

        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;

            match &*key.to_string() {
                "identifier" => {
                    if identifier.replace(input.parse::<Ident>()?).is_some() {
                        return Err(syn::Error::new(key.span(), "duplicate `identifier`"));
                    }
                }
                "tags" => {
                    let content;
                    bracketed!(content in input);
                    let entries = Punctuated::<TagEntry, Comma>::parse_terminated(&content)?
                        .into_iter()
                        .collect();

                    if tags.replace(entries).is_some() {
                        return Err(syn::Error::new(key.span(), "duplicate `tags`"));
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{other}`; expected `identifier` or `tags`"),
                    ));
                }
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            identifier,
            tags: tags.unwrap_or_default(),
        })
    }
}

// TagEntry — one `<prefix>: <value>` tag declaration.

#[derive(Debug)]
struct TagEntry {
    prefix: Ident,
    value: Tag,
}

impl Parse for TagEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let prefix = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let value = input.parse::<Tag>()?;

        Ok(Self { prefix, value })
    }
}

impl Event {
    fn event(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::event::Event for #ident {}
        }
    }

    fn identifier(&self) -> TokenStream {
        let ident = &self.ident;
        let identifier = &self.identifier;

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::event::Identifier for #ident {
                fn identifier() -> &'static str {
                    #identifier
                }
            }
        }
    }

    fn tags(&self) -> TokenStream {
        let ident = &self.ident;

        let tag = self
            .tags
            .iter()
            .map(|entry| TagInitialize(&entry.prefix, &entry.value))
            .collect::<Vec<_>>();
        let tag_count = tag.len();

        quote! {
            #[automatically_derived]
            impl ::eventric_domain::event::Tags for #ident {
                fn tags(&self) -> ::std::result::Result<
                    ::std::vec::Vec<::eventric_stream::event::Tag<::std::string::String>>,
                    ::error_stack::Report<::eventric_domain::error::Error>
                > {
                    let mut tags = ::std::vec::Vec::with_capacity(#tag_count);

                    #(tags.push(
                        ::error_stack::ResultExt::change_context(
                            #tag,
                            ::eventric_domain::error::Error,
                        )?
                    );)*

                    ::std::result::Result::Ok(tags)
                }
            }
        }
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.event());
        tokens.append_all(self.identifier());
        tokens.append_all(self.tags());
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(Debug)]
pub enum Tag {
    Ident(Ident),
    Expr(Expr),
}

impl Parse for Tag {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // The bare-ident shorthand (`prefix: field` -> `&self.field`) applies only
        // when the identifier is the *entire* value. Speculate, so an expression
        // that merely *starts* with an ident (`this.id`, `this.id()`, `foo::BAR`)
        // is taken as an expression rather than mis-read as the shorthand.
        let fork = input.fork();
        if let Ok(ident) = Ident::parse(&fork)
            && (fork.is_empty() || fork.peek(Token![,]))
        {
            input.advance_to(&fork);
            return Ok(Self::Ident(ident));
        }

        input.parse::<Expr>().map(Self::Expr)
    }
}

// Composites

pub struct TagInitialize<'a>(pub &'a Ident, pub &'a Tag);

impl ToTokens for TagInitialize<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TagInitialize(prefix, tag) = *self;

        // Every value is evaluated in place with the event bound as a receiver,
        // then formatted by `tag!`. No actual closure is generated, so there is no
        // higher-ranked-lifetime coercion (and so no `Cow`).
        let value = match tag {
            // Bare field shorthand.
            Tag::Ident(field) => quote! { &self.#field },
            // Closure: bind the receiver to the closure's *own* parameter name and
            // evaluate its body — pure sugar for the `let`-block below, letting you
            // name (or `_`-ignore) the receiver.
            Tag::Expr(Expr::Closure(closure)) => {
                let body = &closure.body;

                if let Some(receiver) = closure.inputs.first() {
                    quote! { { let #receiver = self; #body } }
                } else {
                    quote! { { #body } }
                }
            }
            // Expression with the event bound as `this` (`&Self`).
            Tag::Expr(expr) => quote! { { let this = self; #expr } },
        };

        tokens.append_all(quote! {
            ::eventric_stream::event::tag!(#prefix, #value)
        });
    }
}

// Functions

pub fn tags_map(tags: Option<HashMap<String, List<Tag>>>) -> Option<HashMap<Ident, List<Tag>>> {
    tags.map(|tags| {
        tags.into_iter()
            .map(|(prefix, tags)| (format_ident!("{prefix}"), tags))
            .collect()
    })
}

// `_ident` is retained only for the (still-darling) Projection caller; the
// value codegen no longer needs the owning type's name. It goes when the
// Projection derive is migrated.
pub fn tags_fold<'a>(
    _ident: &'a Ident,
    tags: Option<&'a HashMap<Ident, List<Tag>>>,
) -> Vec<TagInitialize<'a>> {
    tags.as_ref()
        .map(|tags| {
            tags.iter().fold(Vec::new(), |mut acc, (prefix, tags)| {
                for tag in tags.as_ref() {
                    acc.push(TagInitialize(prefix, tag));
                }

                acc
            })
        })
        .unwrap_or_default()
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> syn::Result<EventArgs> {
        syn::parse_str::<EventArgs>(input)
    }

    #[test]
    fn minimal_identifier_only() {
        let args = parse("identifier: thing_happened").expect("parse");

        assert_eq!(args.identifier.unwrap().to_string(), "thing_happened");
        assert!(args.tags.is_empty());
    }

    // Every value form parses: `bare` is the field shorthand; the rest are
    // expressions bound to the event — a borrow, a method call, a path, and a
    // closure that names its own receiver. `call`/`path` are exactly the
    // bare-ident-LED expressions the speculative fork keeps from being mis-read
    // as the shorthand.
    #[test]
    fn tag_value_forms_all_parse() {
        let args = parse(
            "identifier: x, tags: [bare: field, borrowed: &this.field, call: this.compute(), \
             path: foo::BAR, closure: |e| e.compute()]",
        )
        .expect("value forms");

        assert_eq!(args.tags.len(), 5);
        assert!(matches!(args.tags[0].value, Tag::Ident(_)));
        assert!(matches!(args.tags[1].value, Tag::Expr(_)));
        assert!(matches!(args.tags[2].value, Tag::Expr(_)));
        assert!(matches!(args.tags[3].value, Tag::Expr(_)));
        assert!(matches!(args.tags[4].value, Tag::Expr(Expr::Closure(_))));
    }

    #[test]
    fn trailing_commas_and_empty_tags_ok() {
        parse("identifier: x,").expect("trailing comma after identifier");
        parse("identifier: x, tags: [a: b,]").expect("trailing comma in tags");
        parse("identifier: x, tags: []").expect("empty tags");
    }

    #[test]
    fn unknown_key_is_named() {
        let error = parse("colour: red").expect_err("unknown key");

        assert!(error.to_string().contains("unknown key"), "{error}");
    }

    #[test]
    fn duplicate_identifier_rejected() {
        let error = parse("identifier: a, identifier: b").expect_err("duplicate");

        assert!(error.to_string().contains("duplicate"), "{error}");
    }

    // Missing identifier is raised by `Event::new` (against the attribute span),
    // not by `EventArgs::parse`.
    #[test]
    fn missing_identifier_errors_from_new() {
        let input: DeriveInput =
            syn::parse_str("#[event(tags: [a: x])] struct S { a: String }").expect("derive input");

        let error = Event::new(&input).expect_err("missing identifier");

        assert!(
            error.to_string().contains("missing the required"),
            "{error}"
        );
    }
}
