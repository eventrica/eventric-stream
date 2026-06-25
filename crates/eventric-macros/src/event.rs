use proc_macro2::TokenStream;
use quote::{
    ToTokens,
    TokenStreamExt as _,
    quote,
};
use syn::{
    DeriveInput,
    Expr,
    Ident,
    Token,
    braced,
    bracketed,
    parse::{
        Parse,
        ParseStream,
        discouraged::Speculative as _,
    },
    punctuated::Punctuated,
    token::{
        Bracket,
        Comma,
    },
};

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

        // The body is the declarative grammar `identifier: <ident>, tags: {
        // <prefix>: <value>, .. }` (see `EventArgs`), hand-parsed. The identifier
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
// optional `tags: { <prefix>: <value>, .. }`, as comma-separated `key: value`
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
                    braced!(content in input);
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

// TagEntry — one `<prefix>: <value>` tag declaration, where `<value>` is a
// single value or `[a, b, ..]` for several tags under the same prefix (e.g. a
// transfer tagged `account: [from, to]`).

#[derive(Debug)]
pub(crate) struct TagEntry {
    pub(crate) prefix: Ident,
    pub(crate) values: Vec<Tag>,
}

impl Parse for TagEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let prefix = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;

        let values = if input.peek(Bracket) {
            let content;
            bracketed!(content in input);
            let values = Punctuated::<Tag, Comma>::parse_terminated(&content)?
                .into_iter()
                .collect::<Vec<_>>();

            if values.is_empty() {
                return Err(syn::Error::new(
                    prefix.span(),
                    format!("tag `{prefix}` has an empty value list"),
                ));
            }

            values
        } else {
            vec![input.parse::<Tag>()?]
        };

        Ok(Self { prefix, values })
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
            .flat_map(|entry| {
                entry
                    .values
                    .iter()
                    .map(move |value| TagInitialize(&entry.prefix, value))
            })
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

        // Each value is evaluated with the event in scope as `self` (the tag/filter
        // method takes `&self`), then formatted by `tag!`. No closure is generated
        // for the expression form, so there is no higher-ranked-lifetime coercion
        // (and no `Cow`).
        let value = match tag {
            // Bare field shorthand.
            Tag::Ident(field) => quote! { &self.#field },
            // Closure: bind the event to the closure's *own* parameter name and
            // evaluate its body — sugar for naming (or `_`-ignoring) the receiver
            // when `self` won't do (e.g. a different name, or a multi-statement body).
            Tag::Expr(Expr::Closure(closure)) => {
                let body = &closure.body;

                if let Some(receiver) = closure.inputs.first() {
                    quote! { { let #receiver = self; #body } }
                } else {
                    quote! { { #body } }
                }
            }
            // A plain expression — `self` is the event.
            Tag::Expr(expr) => quote! { #expr },
        };

        tokens.append_all(quote! {
            ::eventric_stream::event::tag!(#prefix, #value)
        });
    }
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
            "identifier: x, tags: { bare: field, borrowed: &self.field, call: this.compute(), \
             path: foo::BAR, closure: |e| e.compute() }",
        )
        .expect("value forms");

        assert_eq!(args.tags.len(), 5);
        assert!(matches!(args.tags[0].values[0], Tag::Ident(_)));
        assert!(matches!(args.tags[1].values[0], Tag::Expr(_)));
        assert!(matches!(args.tags[2].values[0], Tag::Expr(_)));
        assert!(matches!(args.tags[3].values[0], Tag::Expr(_)));
        assert!(matches!(
            args.tags[4].values[0],
            Tag::Expr(Expr::Closure(_))
        ));
    }

    // A `[..]` value declares several tags under one prefix.
    #[test]
    fn list_valued_tag_parses() {
        let args = parse("identifier: x, tags: { account: [from, to] }").expect("list value");

        assert_eq!(args.tags.len(), 1);
        assert_eq!(args.tags[0].prefix.to_string(), "account");
        assert_eq!(args.tags[0].values.len(), 2);
    }

    #[test]
    fn trailing_commas_and_empty_tags_ok() {
        parse("identifier: x,").expect("trailing comma after identifier");
        parse("identifier: x, tags: { a: b, }").expect("trailing comma in tags");
        parse("identifier: x, tags: {}").expect("empty tags");
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
        let input: DeriveInput = syn::parse_str("#[event(tags: { a: x })] struct S { a: String }")
            .expect("derive input");

        let error = Event::new(&input).expect_err("missing identifier");

        assert!(
            error.to_string().contains("missing the required"),
            "{error}"
        );
    }
}
