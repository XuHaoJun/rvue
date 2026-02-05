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
    let mut has_args = false;

    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            has_args = true;
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;

            // Extract identifier from pattern
            if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                // Each parameter becomes a field in the props struct (not wrapped in a "props" field)
                props_fields.push(quote! { pub #ident: rvue::widget::ReactiveValue<#ty> });
                fn_args.push(ident);
                // Unwrap ReactiveValue for each parameter
                props_init.push(quote! { let #ident = props.#ident.get(); });
            }
        }
    }

    // Generate trace impl that only traces ReactiveValue fields
    let trace_impl = if has_args {
        let trace_fields = fn_args.iter().map(|ident| {
            quote! { self.#ident.trace(visitor); }
        });
        quote! {
            unsafe impl rudo_gc::Trace for #props_name {
                fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
                    #(#trace_fields)*
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate the Props struct (empty if no args)
    let props_struct = if has_args {
        quote! {
            #[derive(Clone, Debug)]
            #vis struct #props_name {
                #(#props_fields),*
            }
        }
    } else {
        quote! {
            #[derive(Clone, Debug)]
            #vis struct #props_name;
        }
    };

    // Generate the Props impl for default (for zero-arg components)
    let props_default_impl = if !has_args {
        quote! {
            impl #props_name {
                pub fn new() -> Self {
                    Self
                }
            }

            impl Default for #props_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate the modified function
    // For zero-arg components, use Default::default() for props
    let output_fn = if has_args {
        quote! {
            #[allow(non_snake_case)]
            #vis fn #name(props: #props_name) -> impl rvue::prelude::View {
                #(#props_init)*
                #block
            }
        }
    } else {
        quote! {
            #[allow(non_snake_case)]
            #vis fn #name(props: #props_name) -> impl rvue::prelude::View {
                #block
            }
        }
    };

    quote! {
        #props_struct

        #trace_impl

        #props_default_impl

        #output_fn
    }
}
