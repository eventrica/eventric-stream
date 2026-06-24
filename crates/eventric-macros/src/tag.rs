use proc_macro2::TokenStream;
use quote::{
    ToTokens,
    TokenStreamExt as _,
    quote,
};
use syn::{
    Ident,
    Token,
    parse::{
        Parse,
        ParseStream,
    },
};

// =================================================================================================
// Tag Macros
// =================================================================================================

#[derive(Debug)]
pub struct Tag {
    prefix: String,
    value: TokenStream,
}

impl Tag {
    pub fn new(input: TokenStream) -> darling::Result<Self> {
        syn::parse2(input).map_err(darling::Error::from)
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let prefix = input.parse::<Ident>()?.to_string();
        input.parse::<Token![,]>()?;
        let value = input.parse::<TokenStream>()?;

        Ok(Self { prefix, value })
    }
}

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let prefix = &self.prefix;
        let value = &self.value;

        tokens.append_all(quote! {
            ::eventric_stream::event::Tag::prefixed(#prefix, #value)
        });
    }
}
