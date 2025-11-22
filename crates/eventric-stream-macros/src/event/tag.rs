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
pub struct Tag {
    prefix: String,
    value: TokenStream,
}

impl Tag {
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

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let prefix = &self.prefix;
        let value = &self.value;

        let tag_type = quote! { eventric_stream::event::Tag };

        tokens.append_all(quote! {
            #tag_type::new(format!("{}:{}", #prefix, #value))
        });
    }
}
