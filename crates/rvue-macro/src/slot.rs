//! Implementation of the #[slot] macro
//!
//! This macro transforms a struct definition into a slot type that can be used
//! as a component property for accepting content from parents.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemStruct, LitStr, Meta};

pub fn slot_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse2::<ItemStruct>(item).expect("Failed to parse slot struct");

    let docs = extract_docs(&input.attrs);
    let vis = input.vis.clone();
    let name = &input.ident;

    let mut children_field = None;
    let mut other_fields: Vec<(Ident, syn::Type, Vec<syn::Attribute>)> = Vec::new();

    for field in input.fields.iter() {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_ty = field.ty.clone();
        let field_attrs = field.attrs.clone();

        if field_name == "children" {
            children_field = Some((field_name, field_ty));
        } else {
            other_fields.push((field_name, field_ty, field_attrs));
        }
    }

    let children_ty =
        children_field.as_ref().map(|(_, ty)| ty).expect("Slot must have a `children` field");

    let builder_name_doc = LitStr::new(&format!("Props for the [`{name}`] slot."), name.span());

    let struct_fields = if other_fields.is_empty() {
        quote! { pub children: #children_ty }
    } else {
        let field_names = other_fields.iter().map(|(name, _, _)| name);
        let field_types = other_fields.iter().map(|(_, ty, _)| ty);
        let field_attrs = other_fields.iter().map(|(_, _, attrs)| attrs);
        quote! {
            pub children: #children_ty,
            #(#(#field_attrs)* pub #field_names: #field_types),*
        }
    };

    let trace_impl = if other_fields.is_empty() {
        quote! {
            #[automatically_derived]
            unsafe impl ::rudo_gc::Trace for #name {
                fn trace(&self, visitor: &mut ::rudo_gc::Visitor) {
                    self.children.trace(visitor);
                }
            }
        }
    } else {
        let trace_fields = other_fields.iter().map(|(fname, _, _)| {
            quote! { self.#fname.trace(visitor); }
        });
        quote! {
            #[automatically_derived]
            unsafe impl ::rudo_gc::Trace for #name {
                fn trace(&self, visitor: &mut ::rudo_gc::Visitor) {
                    self.children.trace(visitor);
                    #(#trace_fields)*
                }
            }
        }
    };

    let output = quote! {
        #[doc = #builder_name_doc]
        #[doc = ""]
        #docs
        #[derive(Clone)]
        #vis struct #name {
            #struct_fields
        }

        #trace_impl

        impl From<#name> for Vec<#name> {
            fn from(value: #name) -> Self {
                vec![value]
            }
        }
    };

    output.into()
}

fn extract_docs(attrs: &[syn::Attribute]) -> TokenStream {
    let docs: Vec<_> = attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(name_value) = &attr.meta {
                if name_value.path.is_ident("doc") {
                    return Some(name_value.value.clone());
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        return TokenStream::new();
    }

    quote! {
        #(#docs)*
    }
}
