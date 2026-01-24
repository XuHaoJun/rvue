//! RSX parser using rstml
//!
//! This module handles the parsing of HTML-like RSX syntax into
//! structured AST nodes using the rstml crate.

use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error2::abort;
use rstml::node::Node;
use rstml::ParserConfig;

/// Parse view macro input into rstml nodes
///
/// Returns `Result` with nodes on success, or a TokenStream of compile errors
pub fn parse_view(input: TokenStream) -> Result<Vec<Node>, TokenStream> {
    let config = ParserConfig::default().recover_block(true);

    let parser = rstml::Parser::new(config);
    let (nodes, errors) = parser.parse_recoverable(input).split_vec();

    if !errors.is_empty() {
        let error_tokens: Vec<TokenStream> =
            errors.iter().map(|e| e.clone().emit_as_expr_tokens()).collect();
        Err(quote::quote! {
            #(#error_tokens)*
        })
    } else {
        Ok(nodes)
    }
}

/// Parse a global class declaration (class = value,)
///
/// Returns None if no global class is present, otherwise returns class expression
pub fn parse_global_class(input: TokenStream) -> Option<TokenTree> {
    let mut tokens = input.into_iter();

    let first = tokens.next();
    let second = tokens.next();
    let third = tokens.next();
    let fourth = tokens.next();

    match (&first, &second) {
        (Some(TokenTree::Ident(first)), Some(TokenTree::Punct(eq)))
            if *first == "class" && eq.as_char() == '=' =>
        {
            match &fourth {
                Some(TokenTree::Punct(comma)) if comma.as_char() == ',' => third.clone(),
                _ => {
                    abort!(
                        second,
                        "To create a global class with view! macro you must put a comma `,` after the value";
                        help = r#"e.g., view!{ class=\"my-class\", <div>...</div>}"#
                    )
                }
            }
        }
        _ => None,
    }
}

/// Extract remaining tokens after a global class declaration
pub fn strip_global_class(input: TokenStream, global_class: Option<&TokenTree>) -> TokenStream {
    if global_class.is_some() {
        let mut tokens = input.into_iter();
        tokens.next();
        tokens.next();
        tokens.next();
        tokens.next();
        tokens.collect()
    } else {
        input
    }
}
