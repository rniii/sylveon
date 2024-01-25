use std::str::FromStr;

use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn opt(tokens: TokenStream) -> TokenStream {
    match tokens.into_iter().next() {
        Some(TokenTree::Ident(v)) => {
            let ident = v.to_string();
            let mut chars = ident.chars();
            TokenStream::from_str(&match (chars.next(), chars.next()) {
                (Some(c), None) => format!("::sylveon::Opt::Short('{c}')"),
                _ => format!("::sylveon::Opt::Long(\"{ident}\")"),
            })
            .unwrap()
        }
        _ => unreachable!(),
    }
}
