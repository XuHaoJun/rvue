use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, PatIdent};

pub fn component_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse2::<ItemFn>(item).expect("Failed to parse component function");

    let ItemFn { vis, sig, block, .. } = &input;
    let name = &sig.ident;
    let props_name = format_ident!("{}Props", name);

    let mut props_fields = Vec::new();
    let mut fn_args = Vec::new();
    let mut props_init = Vec::new();

    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;

            // Extract identifier from pattern
            if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                props_fields.push(quote! { pub #ident: #ty });
                fn_args.push(ident);
                props_init.push(quote! { let #ident = props.#ident; });
            }
        }
    }

    // Generate the Props struct
    let props_struct = quote! {
        #[derive(Clone, Debug)]
        #vis struct #props_name {
            #(#props_fields),*
        }
    };

    // Generate the modified function
    // It takes `props: ComponentProps` instead of individual args
    let output_fn = quote! {
        #vis fn #name(props: #props_name) -> impl rvue::prelude::View {
            #(#props_init)*
            #block
        }
    };

    quote! {
        #props_struct
        #output_fn
    }
}
