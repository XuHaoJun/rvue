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
        let setter_call = event_setter(name, handler);

        quote! {
            #id.#setter_call
        }
    });

    quote! {
        #(#handlers;)*
    }
}

fn event_setter(event_name: &str, handler: &Expr) -> TokenStream {
    let arg_count = closure_arg_count(handler);

    match event_name {
        "click" => match arg_count {
            Some(0) => quote! { on_click_0arg(#handler) },
            Some(1) => quote! { on_click_1arg(#handler) },
            Some(2) => quote! { on_click(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "input" => match arg_count {
            Some(0) => quote! { on_input_0arg(#handler) },
            Some(1) => quote! { on_input_1arg(#handler) },
            Some(2) => quote! { on_input(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "change" => match arg_count {
            Some(0) => quote! { on_change_0arg(#handler) },
            Some(1) => quote! { on_change_1arg(#handler) },
            Some(2) => quote! { on_change(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "key_down" => match arg_count {
            Some(0) => quote! { on_key_down_0arg(#handler) },
            Some(1) => quote! { on_key_down_1arg(#handler) },
            Some(2) => quote! { on_key_down(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "key_up" => match arg_count {
            Some(0) => quote! { on_key_up_0arg(#handler) },
            Some(1) => quote! { on_key_up_1arg(#handler) },
            Some(2) => quote! { on_key_up(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "focus" => match arg_count {
            Some(0) => quote! { on_focus_0arg(#handler) },
            Some(1) => quote! { on_focus_1arg(#handler) },
            Some(2) => quote! { on_focus(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "blur" => match arg_count {
            Some(0) => quote! { on_blur_0arg(#handler) },
            Some(1) => quote! { on_blur_1arg(#handler) },
            Some(2) => quote! { on_blur(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "pointer_down" => match arg_count {
            Some(0) => quote! { on_pointer_down_0arg(#handler) },
            Some(1) => quote! { on_pointer_down_1arg(#handler) },
            Some(2) => quote! { on_pointer_down(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "pointer_up" => match arg_count {
            Some(0) => quote! { on_pointer_up_0arg(#handler) },
            Some(1) => quote! { on_pointer_up_1arg(#handler) },
            Some(2) => quote! { on_pointer_up(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "pointer_enter" => match arg_count {
            Some(0) => quote! { on_pointer_enter_0arg(#handler) },
            Some(1) => quote! { on_pointer_enter_1arg(#handler) },
            Some(2) => quote! { on_pointer_enter(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "pointer_leave" => match arg_count {
            Some(0) => quote! { on_pointer_leave_0arg(#handler) },
            Some(1) => quote! { on_pointer_leave_1arg(#handler) },
            Some(2) => quote! { on_pointer_leave(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        "pointer_move" => match arg_count {
            Some(0) => quote! { on_pointer_move_0arg(#handler) },
            Some(1) => quote! { on_pointer_move_1arg(#handler) },
            Some(2) => quote! { on_pointer_move(#handler) },
            _ => panic!("Event handler with more than 2 arguments not supported"),
        },
        _ => panic!("Unknown event: {}", event_name),
    }
}

fn closure_arg_count(expr: &Expr) -> Option<usize> {
    match unwrap_expr(expr) {
        Expr::Closure(closure) => Some(closure.inputs.len()),
        _ => None,
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Block(block) if block.block.stmts.len() == 1 => match block.block.stmts.first() {
            Some(syn::Stmt::Expr(inner, _)) => inner,
            _ => expr,
        },
        Expr::Block(block) => {
            for stmt in &block.block.stmts {
                if let syn::Stmt::Expr(inner, _) = stmt {
                    if let Expr::Closure(_) = inner {
                        return inner;
                    }
                }
            }
            expr
        }
        Expr::Paren(paren) => unwrap_expr(&paren.expr),
        _ => expr,
    }
}

fn generate_text_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let content = extract_prop_value(attrs, "content", || quote! { "" });

    quote! {
        rvue::widgets::Text::new(#content)
    }
}

fn generate_button_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let label = extract_prop_value(attrs, "label", || quote! { "".to_string() });

    quote! {
        rvue::widgets::Button::new(#label.to_string())
    }
}

fn generate_flex_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let direction = extract_prop_value(attrs, "direction", || quote! { "row" });
    let gap = extract_prop_value(attrs, "gap", || quote! { 0.0 });
    let align_items = extract_prop_value(attrs, "align_items", || quote! { "stretch" });
    let justify_content = extract_prop_value(attrs, "justify_content", || quote! { "start" });

    quote! {
        {
            use rvue_style::{FlexDirection, AlignItems, JustifyContent, Gap};
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
                "start" => AlignItems::FlexStart,
                "end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Stretch,
            };
            let justify_str = #justify_content.to_string();
            let justify_enum = match justify_str.as_str() {
                "start" => JustifyContent::FlexStart,
                "end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::FlexStart,
            };
            let gap_val = #gap;

            let flex = rvue::widgets::Flex::new()
                .direction(direction_enum)
                .gap(Gap(gap_val))
                .align_items(align_enum)
                .justify_content(justify_enum);

            flex
        }
    }
}

fn generate_text_input_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { "".to_string() });

    quote! {
        rvue::widgets::TextInput::new(#value)
    }
}

fn generate_number_input_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { 0.0 });

    quote! {
        rvue::widgets::NumberInput::new(#value)
    }
}

fn generate_checkbox_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let checked = extract_prop_value(attrs, "checked", || quote! { false });

    quote! {
        rvue::widgets::Checkbox::new(#checked)
    }
}

fn generate_radio_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let value = extract_prop_value(attrs, "value", || quote! { "".to_string() });
    let checked = extract_prop_value(attrs, "checked", || quote! { false });

    quote! {
        rvue::widgets::Radio::new(#value.to_string(), #checked)
    }
}

fn generate_show_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let when = extract_prop_value(attrs, "when", || quote! { false });

    quote! {
        {
            use rvue::widget::BuildContext;
            rvue::widgets::Show::new(#when, |_ctx: &mut BuildContext| {
                let flex = rvue::widgets::Flex::new();
                let state = flex.build(_ctx);
                rvue::Gc::clone(state.component())
            })
        }
    }
}

fn generate_for_widget(_id: u64, attrs: &[RvueAttribute]) -> TokenStream {
    let items = extract_prop_value(attrs, "each", || quote! { vec![] });
    let key_fn = extract_prop_value(attrs, "key", || quote! { |item| item });
    let view_fn = extract_prop_value(
        attrs,
        "view",
        || quote! { |item| view! { <Text content={format!("{:?}", item)} />} },
    );

    quote! {
        rvue::widgets::For::new(
            #items,
            #key_fn,
            #view_fn
        )
    }
}

fn generate_custom_widget(_id: u64, name: &str, attrs: &[RvueAttribute]) -> TokenStream {
    let widget_name = format_ident!("{}", name);
    let props = attrs.iter().filter(|a| !matches!(a, RvueAttribute::Event { .. })).map(|attr| {
        let name = format_ident!("{}", attr.name());
        let value = extract_attr_value(attr);
        quote! {
            .#name(#value)
        }
    });

    quote! {
        #widget_name::new()#(#props)*
    }
}

fn extract_prop_value<F>(attrs: &[RvueAttribute], name: &str, default: F) -> TokenStream
where
    F: FnOnce() -> TokenStream,
{
    attrs.iter().find(|a| a.name() == name).map(extract_attr_value).unwrap_or_else(default)
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
        RvueAttribute::Slot { .. } => {
            quote! { compile_error!("Unexpected slot attribute in property position") }
        }
    }
}

use syn::Expr;
