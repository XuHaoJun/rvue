//! Implementation of the #[slot] macro
//!
//! This macro transforms a struct definition into a slot type that can be used
//! as a component property for accepting content from parents.
//!
//! # Naming Convention
//!
//! Slot attributes in the `view!` macro use snake_case (e.g., `slot:body`).
//! The corresponding slot struct field should be `children` or `Children`.
//! Named slots are converted to snake_case variable names for the slot closure.
//!
//! # Slot Props
//!
//! Slot struct fields (other than `children`) are treated as props.
//! They get builder methods for setting values.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemStruct, LitStr};

struct SlotField {
    name: Ident,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

pub fn slot_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse2::<ItemStruct>(item).expect("Failed to parse slot struct");

    let struct_attrs: Vec<_> = input.attrs.iter().collect();
    let vis = input.vis.clone();
    let name = &input.ident;

    let mut children_field: Option<(Ident, syn::Type)> = None;
    let mut prop_fields: Vec<SlotField> = Vec::new();

    for field in input.fields.iter() {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_ty = field.ty.clone();
        let field_attrs = field.attrs.clone();

        if field_name == "children" || field_name == "Children" {
            children_field = Some((field_name, field_ty));
        } else {
            prop_fields.push(SlotField { name: field_name, ty: field_ty, attrs: field_attrs });
        }
    }

    let children_ty =
        children_field.as_ref().map(|(_, ty)| ty).expect("Slot must have a `children` field");

    let builder_name_doc = LitStr::new(&format!("Props for the [`{name}`] slot."), name.span());

    let struct_fields = if prop_fields.is_empty() {
        quote! { pub children: #children_ty }
    } else {
        let field_names = prop_fields.iter().map(|f| &f.name);
        let field_types = prop_fields.iter().map(|f| &f.ty);
        let field_attrs = prop_fields.iter().map(|f| &f.attrs);
        quote! {
            pub children: #children_ty,
            #(#(#field_attrs)* pub #field_names: #field_types),*
        }
    };

    let trace_impl = if prop_fields.is_empty() {
        quote! {
            #[automatically_derived]
            unsafe impl ::rudo_gc::Trace for #name {
                fn trace(&self, visitor: &mut impl ::rudo_gc::Visitor) {
                    self.children.trace(visitor);
                }
            }
        }
    } else {
        let trace_fields = prop_fields.iter().map(|f| {
            let fname = &f.name;
            quote! { self.#fname.trace(visitor); }
        });
        quote! {
            #[automatically_derived]
            unsafe impl ::rudo_gc::Trace for #name {
                fn trace(&self, visitor: &mut impl ::rudo_gc::Visitor) {
                    self.children.trace(visitor);
                    #(#trace_fields)*
                }
            }
        }
    };

    let new_impl = if prop_fields.is_empty() {
        quote! {
            impl #name {
                pub fn new(children: #children_ty) -> Self {
                    Self { children }
                }
            }
        }
    } else {
        let field_names: Vec<_> = prop_fields.iter().map(|f| &f.name).collect();
        let _field_types: Vec<_> = prop_fields.iter().map(|f| &f.ty).collect();
        let set_methods = prop_fields.iter().map(|f| {
            let fname = &f.name;
            let fty = &f.ty;
            let method_name = fname;
            quote! {
                pub fn #method_name(mut self, value: #fty) -> Self {
                    self.#fname = value;
                    self
                }
            }
        });

        quote! {
            impl #name {
                pub fn new(children: #children_ty) -> Self {
                    Self {
                        children,
                        #(#field_names: ::core::default::Default::default()),*
                    }
                }

                #(#set_methods)*
            }
        }
    };

    let output = if struct_attrs.is_empty() {
        quote! {
            #[doc = #builder_name_doc]
            #[doc = ""]
            #[derive(Clone)]
            #vis struct #name {
                #struct_fields
            }

            #trace_impl

            #new_impl

            impl From<#name> for Vec<#name> {
                fn from(value: #name) -> Self {
                    vec![value]
                }
            }

            impl rvue::widget::IntoReactiveValue<#name> for #name {
                fn into_reactive(self) -> rvue::widget::ReactiveValue<#name> {
                    rvue::widget::ReactiveValue::Static(self)
                }
            }
        }
    } else {
        quote! {
            #(#struct_attrs)*
            #[doc = #builder_name_doc]
            #[doc = ""]
            #[derive(Clone)]
            #vis struct #name {
                #struct_fields
            }

            #trace_impl

            #new_impl

            impl From<#name> for Vec<#name> {
                fn from(value: #name) -> Self {
                    vec![value]
                }
            }

            impl rvue::widget::IntoReactiveValue<#name> for #name {
                fn into_reactive(self) -> rvue::widget::ReactiveValue<#name> {
                    rvue::widget::ReactiveValue::Static(self)
                }
            }
        }
    };

    output
}
