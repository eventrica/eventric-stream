use proc_macro2::{
    TokenStream,
    TokenTree,
};
use quote::{
    ToTokens,
    TokenStreamExt as _,
    quote,
};

// =================================================================================================
// Tag Macros
// =================================================================================================

#[derive(Debug)]
pub struct TagFunction {
    prefix: String,
    value: TokenStream,
}

impl TagFunction {
    #[rustfmt::skip]
    pub fn new(input: TokenStream) -> darling::Result<Self> {
        let tokens = input.into_iter().collect::<Vec<_>>();

        match &tokens[..] {
            [TokenTree::Ident(ident), TokenTree::Punct(punct), tokens @ ..] if punct.as_char() == ',' => {
                let prefix = ident.to_string();
                let mut value = TokenStream::new();

                value.append_all(tokens);

                Ok(Self { prefix, value })
            }
            _ => Err(darling::Error::unsupported_shape("unexpected tag arguments")),
        }
    }
}

impl ToTokens for TagFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let prefix = &self.prefix;
        let value = &self.value;

        tokens.append_all(quote! {
            eventric_stream::event::Tag::new(format!("{}:{}", #prefix, #value))
        });
    }
}
