//! Widget-specific code generation
//!
//! This module generates code for each widget type, handling their
//! specific attributes and converting them to builder patterns.

use crate::ast::{RvueAttribute, WidgetType};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

/// Generate code to create a widget instance
pub fn generate_widget_code(
    widget_type: &WidgetType,
    id: u64,
    attributes: &[RvueAttribute],
) -> TokenStream {
    match widget_type {
        WidgetType::Text => generate_text_widget(id, attributes),
        WidgetType::Button => generate_button_widget(id, attributes),
        WidgetType::Flex => generate_flex_widget(id, attributes),
        WidgetType::TextInput => generate_text_input_widget(id, attributes),
        WidgetType::NumberInput => generate_number_input_widget(id, attributes),
        WidgetType::Checkbox => generate_checkbox_widget(id, attributes),
        WidgetType::Radio => generate_radio_widget(id, attributes),
        WidgetType::Show => generate_show_widget(id, attributes),
        WidgetType::For => generate_for_widget(id, attributes),
        WidgetType::Custom(name) => generate_custom_widget(id, name, attributes),
    }
}

/// Generate event handler calls
pub fn generate_event_handlers(id: &Ident, events: &[(&str, &Expr)]) -> TokenStream {
    let handlers = events.iter().map(|(name, handler)| {
        let handler_ident = format_ident!("on_{}", name);
        quote! {
            #id.#handler_ident(#handler)
        }
    });

    quote! {
        #(#handlers)*
    }
}

fn generate_text_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let content = extract_prop_value(attrs, "content", || quote! { "" });
    // Text widget in rvue currently takes only content string in new()
    // Font size and color are not yet supported in new() signature based on provided file
    // But ComponentProps has them. Text::new implementation ignores them?
    // Checking Text::new in rvue: pub fn new(id, content) -> Gc<Component>
    // It creates ComponentProps internally.

    quote! {
        rvue::widgets::Text::new(
            #id,
            #content.to_string()
        )
    }
}

fn generate_button_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let label = extract_prop_value(attrs, "label", || quote! { "".to_string() });

    quote! {
        rvue::widgets::Button::new(
            #id,
            #label.to_string()
        )
    }
}

fn generate_flex_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    // Flex::new takes enums. We need to handle this.
    // For now assuming the user provides expressions that evaluate to the enum or we need conversion.
    // Since smoke_test doesn't test Flex yet, I'll use Default if attributes are missing,
    // or try to parse if I can.
    // But to match the previous implementation's intent (passing strings),
    // I should probably see if I can convert strings to enums in generated code.
    // However, Flex::new takes FlexDirection, not String.
    // I'll assume for now we use defaults or expressions.

    // Check if attributes exist, otherwise use defaults
    // For string literals, we need to convert them to enums
    let direction = extract_prop_value(attrs, "direction", || quote! { "row" });
    let gap = extract_prop_value(attrs, "gap", || quote! { 0.0 });
    let align_items = extract_prop_value(attrs, "align_items", || quote! { "stretch" });
    let justify_content = extract_prop_value(attrs, "justify_content", || quote! { "start" });

    // Convert string literals to enum values
    // This is a temporary solution - in Phase 3, the macro will use the new Widget API
    quote! {
        {
            use rvue::style::{FlexDirection, AlignItems, JustifyContent};
            let direction_str = #direction.to_string();
            let direction_enum = match direction_str.as_str() {
                "row" => FlexDirection::Row,
                "column" => FlexDirection::Column,
                "row-reverse" => FlexDirection::RowReverse,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            };
            let align_str = #align_items.to_string();
            let align_enum = match align_str.as_str() {
                "start" => AlignItems::Start,
                "end" => AlignItems::End,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Stretch,
            };
            let justify_str = #justify_content.to_string();
            let justify_enum = match justify_str.as_str() {
                "start" => JustifyContent::Start,
                "end" => JustifyContent::End,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::Start,
            };
            rvue::widgets::Flex::new(
                #id,
                direction_enum,
                #gap,
                align_enum,
                justify_enum
            )
        }
    }
}

fn generate_text_input_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { "".to_string() });

    quote! {
        rvue::widgets::TextInput::new(
            #id,
            #value
        )
    }
}

fn generate_number_input_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { 0.0 });

    quote! {
        rvue::widgets::NumberInput::new(
            #id,
            #value
        )
    }
}

fn generate_checkbox_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let checked = extract_prop_value(attrs, "checked", || quote! { false });

    quote! {
        rvue::widgets::Checkbox::new(
            #id,
            #checked
        )
    }
}

fn generate_radio_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { "".to_string() });
    let checked = extract_prop_value(attrs, "checked", || quote! { false });

    quote! {
        rvue::widgets::Radio::new(
            #id,
            #value,
            #checked
        )
    }
}

fn generate_show_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let when = extract_prop_value(attrs, "when", || quote! { false });

    quote! {
        rvue::widgets::Show::new(
            #id,
            #when
        )
    }
}

fn generate_for_widget(id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let item_count = extract_prop_value(attrs, "item_count", || quote! { 0 });

    quote! {
        rvue::widgets::For::new(
            #id,
            #item_count
        )
    }
}

fn generate_custom_widget(id: u64, name: &str, attrs: &[RvueAttribute]) -> TokenStream {
    let widget_name = format_ident!("{}", name);
    let props = attrs.iter().filter(|a| !matches!(a, RvueAttribute::Event { .. })).map(|attr| {
        let name = format_ident!("{}", attr.name());
        let value = extract_attr_value(attr);
        quote! {
            .#name(#value)
        }
    });

    quote! {
        #widget_name::new(#id)#(#props)*
    }
}

fn extract_prop_value<F>(attrs: &[RvueAttribute], name: &str, default: F) -> TokenStream
where
    F: FnOnce() -> TokenStream,
{
    attrs.iter().find(|a| a.name() == name).map(|a| extract_attr_value(a)).unwrap_or_else(default)
}

fn extract_optional_prop(attrs: &[RvueAttribute], name: &str) -> TokenStream {
    attrs
        .iter()
        .find(|a| a.name() == name)
        .map(|a| {
            let value = extract_attr_value(a);
            quote! { Some(#value) }
        })
        .unwrap_or_else(|| quote! { None })
}

fn extract_attr_value(attr: &RvueAttribute) -> TokenStream {
    match attr {
        RvueAttribute::Static { value, .. } => quote! { #value },
        RvueAttribute::Dynamic { expr, .. } => quote! { #expr },
        RvueAttribute::Event { .. } => {
            quote! { compile_error!("Unexpected event attribute in property position") }
        }
    }
}

use syn::Expr;
