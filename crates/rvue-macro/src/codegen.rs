//! Main code generation module
//!
//! This module handles converting rvue AST into Rust code
//! that creates widgets using rvue widget system.

use crate::ast::{RvueAttribute, RvueElement, RvueNode, RvueText, WidgetType};
use crate::widgets::generate_event_handlers;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use rstml::node::{Node, NodeAttribute, NodeElement, NodeName};
use syn::spanned::Spanned;
use syn::{Block, Expr};

/// Generate view code from AST nodes
pub fn generate_view_code(nodes: Vec<RvueNode>) -> TokenStream {
    match nodes.len() {
        0 => generate_empty_view(),
        1 => generate_node_code(&nodes[0]),
        _ => generate_fragment_code(nodes),
    }
}

fn generate_empty_view() -> TokenStream {
    // For empty view, create a minimal Flex container using new Widget API
    quote! {
        {
            use rvue::widget::{BuildContext, Widget};
            use rvue::text::TextContext;
            use ::taffy::TaffyTree;

            let mut taffy = TaffyTree::new();
            let mut text_context = TextContext::new();
            let mut id_counter = 0u64;
            let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

            let widget = rvue::widgets::FlexWidget::new();
            let state = widget.build(&mut ctx);
            let component = state.component().clone();

            rvue::ViewStruct::new(component)
        }
    }
}

fn generate_node_code(node: &RvueNode) -> TokenStream {
    match node {
        RvueNode::Element(el) => generate_element_code_wrapped(el),
        RvueNode::Text(text) => {
            let text_code = generate_text_node_code(text);
            quote! {
                rvue::ViewStruct::new(#text_code)
            }
        }
        RvueNode::Fragment(nodes) => generate_fragment_code(nodes.clone()),
    }
}

fn generate_element_code(el: &RvueElement) -> TokenStream {
    let id_ident = format_ident!("comp");
    let widget_code = generate_widget_builder_code(&el.widget_type, &el.attributes);
    let children_code = generate_children_code(&el.children, &id_ident);
    let events_code = generate_event_handlers_for_element(&id_ident, el);

    quote! {
        {
            use rvue::widget::{BuildContext, Widget};
            use rvue::text::TextContext;
            use ::taffy::TaffyTree;

            let mut taffy = TaffyTree::new();
            let mut text_context = TextContext::new();
            let mut id_counter = 0u64;
            let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

            let widget = #widget_code;
            let state = widget.build(&mut ctx);
            let #id_ident = state.component().clone();

            #children_code
            #events_code

            #id_ident
        }
    }
}

fn generate_element_code_wrapped(el: &RvueElement) -> TokenStream {
    let id_ident = format_ident!("comp");
    let widget_code = generate_widget_builder_code(&el.widget_type, &el.attributes);
    let children_code = generate_children_code(&el.children, &id_ident);
    let events_code = generate_event_handlers_for_element(&id_ident, el);

    quote! {
        {
            use rvue::widget::{BuildContext, Widget};
            use rvue::text::TextContext;
            use ::taffy::TaffyTree;

            let mut taffy = TaffyTree::new();
            let mut text_context = TextContext::new();
            let mut id_counter = 0u64;
            let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

            let widget = #widget_code;
            let state = widget.build(&mut ctx);
            let #id_ident = state.component().clone();

            #children_code
            #events_code

            rvue::ViewStruct::new(#id_ident)
        }
    }
}

fn generate_text_node_code(text: &RvueText) -> TokenStream {
    let content = &text.content;

    quote! {
        {
            use rvue::widget::{BuildContext, Widget};
            use rvue::text::TextContext;
            use ::taffy::TaffyTree;

            let mut taffy = TaffyTree::new();
            let mut text_context = TextContext::new();
            let mut id_counter = 0u64;
            let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

            let widget = rvue::widgets::TextWidget::new(#content.to_string());
            let state = widget.build(&mut ctx);
            state.component().clone()
        }
    }
}

fn generate_fragment_code(nodes: Vec<RvueNode>) -> TokenStream {
    match nodes.len() {
        0 => generate_empty_view(),
        1 => generate_node_code(&nodes[0]),
        _ => {
            let root_id = format_ident!("root");
            let children_code = generate_children_code(&nodes, &root_id);

            quote! {
                {
                    use rvue::widget::{BuildContext, Widget};
                    use rvue::text::TextContext;
                    use taffy::TaffyTree;

                    let mut taffy = TaffyTree::new();
                    let mut text_context = TextContext::new();
                    let mut id_counter = 0u64;
                    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

                    let root_widget = rvue::widgets::FlexWidget::new()
                        .direction(rvue::style::FlexDirection::Row)
                        .gap(0.0)
                        .align_items(rvue::style::AlignItems::Start)
                        .justify_content(rvue::style::JustifyContent::Start);
                    let root_state = root_widget.build(&mut ctx);
                    let #root_id = root_state.component().clone();

                    #children_code

                    rvue::ViewStruct::new(#root_id)
                }
            }
        }
    }
}

fn generate_children_code(children: &[RvueNode], parent_id: &Ident) -> TokenStream {
    let child_vars = children.iter().map(|child| {
        // For children, we need Gc<Component>, not ViewStruct
        // So we generate element code directly (not wrapped)
        match child {
            RvueNode::Element(el) => {
                let child_code = generate_element_code(el);
                quote! {
                    {
                        let child = #child_code;
                        #parent_id.add_child(child);
                    }
                }
            }
            RvueNode::Text(text) => {
                let text_code = generate_text_node_code(text);
                quote! {
                    {
                        let child = #text_code;
                        #parent_id.add_child(child);
                    }
                }
            }
            RvueNode::Fragment(_) => {
                // Fragments as children are not supported - skip
                quote! {}
            }
        }
    });

    quote! {
        #(#child_vars)*
    }
}

fn generate_event_handlers_for_element(component_id: &Ident, el: &RvueElement) -> TokenStream {
    let events = el
        .events()
        .iter()
        .filter_map(|a| match a {
            RvueAttribute::Event { name, handler, .. } => Some((name.as_str(), handler)),
            _ => None,
        })
        .collect::<Vec<_>>();

    if !events.is_empty() {
        generate_event_handlers(component_id, &events)
    } else {
        quote! {}
    }
}

fn generate_widget_builder_code(
    widget_type: &WidgetType,
    attributes: &[RvueAttribute],
) -> TokenStream {
    match widget_type {
        WidgetType::Text => {
            let content = extract_prop_value(attributes, "content", || quote! { "" });
            quote! {
                rvue::widgets::TextWidget::new(#content.to_string())
            }
        }
        WidgetType::Button => {
            let label = extract_prop_value(attributes, "label", || quote! { "" });
            quote! {
                rvue::widgets::ButtonWidget::new(#label.to_string())
            }
        }
        WidgetType::Flex => {
            let direction = extract_prop_value(attributes, "direction", || quote! { "row" });
            let gap = extract_prop_value(attributes, "gap", || quote! { 0.0 });
            let align_items =
                extract_prop_value(attributes, "align_items", || quote! { "stretch" });
            let justify_content =
                extract_prop_value(attributes, "justify_content", || quote! { "start" });
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
                    rvue::widgets::FlexWidget::new()
                        .direction(direction_enum)
                        .gap(#gap)
                        .align_items(align_enum)
                        .justify_content(justify_enum)
                }
            }
        }
        WidgetType::TextInput => {
            let value = extract_prop_value(attributes, "value", || quote! { "" });
            quote! {
                rvue::widgets::TextInputWidget::new(#value.to_string())
            }
        }
        WidgetType::NumberInput => {
            let value = extract_prop_value(attributes, "value", || quote! { 0.0 });
            quote! {
                rvue::widgets::NumberInputWidget::new(#value)
            }
        }
        WidgetType::Checkbox => {
            let checked = extract_prop_value(attributes, "checked", || quote! { false });
            quote! {
                rvue::widgets::CheckboxWidget::new(#checked)
            }
        }
        WidgetType::Radio => {
            let value = extract_prop_value(attributes, "value", || quote! { "" });
            let checked = extract_prop_value(attributes, "checked", || quote! { false });
            quote! {
                rvue::widgets::RadioWidget::new(#value.to_string(), #checked)
            }
        }
        WidgetType::Show => {
            let when = extract_prop_value(attributes, "when", || quote! { false });
            quote! {
                rvue::widgets::ShowWidget::new(#when)
            }
        }
        WidgetType::For => {
            let item_count = extract_prop_value(attributes, "item_count", || quote! { 0 });
            quote! {
                rvue::widgets::ForWidget::new(#item_count)
            }
        }
        WidgetType::Custom(name) => {
            let widget_name = format_ident!("{}", name);
            let props = attributes
                .iter()
                .filter(|a| !matches!(a, RvueAttribute::Event { .. }))
                .map(|attr| {
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
    }
}

fn extract_prop_value<F>(attrs: &[RvueAttribute], name: &str, default: F) -> TokenStream
where
    F: FnOnce() -> TokenStream,
{
    attrs.iter().find(|a| a.name() == name).map(|a| extract_attr_value(a)).unwrap_or_else(default)
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

pub fn convert_rstml_to_rvue(node: &Node, parent_type: Option<&WidgetType>) -> Option<RvueNode> {
    match node {
        Node::Comment(_) => None,
        Node::Doctype(_) => None,
        Node::Fragment(frag) => {
            let children: Vec<RvueNode> = frag
                .children
                .iter()
                .filter_map(|n| convert_rstml_to_rvue(n, parent_type))
                .collect();

            if children.is_empty() {
                None
            } else if children.len() == 1 {
                children.into_iter().next()
            } else {
                Some(RvueNode::Fragment(children))
            }
        }
        Node::Block(block) => {
            let expr = syn::parse2::<Expr>(block.to_token_stream());
            if let Ok(expr) = expr {
                Some(RvueNode::Element(RvueElement {
                    tag_name: "block".to_string(),
                    widget_type: WidgetType::Custom("Block".to_string()),
                    attributes: vec![RvueAttribute::Dynamic {
                        name: "value".to_string(),
                        expr,
                        span: block.span(),
                    }],
                    children: vec![],
                    span: block.span(),
                }))
            } else {
                None
            }
        }
        Node::Text(text) => {
            Some(RvueNode::Text(RvueText { content: text.value_string(), span: text.value.span() }))
        }
        Node::RawText(raw) => {
            let content = raw.to_string_best();
            let span = Span::call_site();
            Some(RvueNode::Text(RvueText { content, span }))
        }
        Node::Element(el_node) => convert_element(el_node),
    }
}

fn convert_element(el_node: &NodeElement) -> Option<RvueNode> {
    if is_spread_element(el_node) {
        return None;
    }

    let tag_name = el_node.name().to_string();
    let widget_type = crate::attrs::classify_widget(&tag_name);

    let mut attributes = vec![];
    for attr in el_node.attributes() {
        if let NodeAttribute::Attribute(_k_attr) = attr {
            if let Ok(rvue_attr) = crate::attrs::parse_attribute(attr) {
                attributes.push(rvue_attr);
            }
        }
    }

    let children = el_node
        .children
        .iter()
        .filter_map(|n| convert_rstml_to_rvue(n, Some(&widget_type)))
        .collect();

    Some(RvueNode::Element(RvueElement {
        tag_name,
        widget_type,
        attributes,
        children,
        span: el_node.name().span(),
    }))
}

fn is_spread_element(el: &NodeElement) -> bool {
    match el.name() {
        NodeName::Block(block) => {
            matches!(block, Block { .. } if is_block_spread_marker(block))
        }
        _ => false,
    }
}

fn is_block_spread_marker(block: &Block) -> bool {
    if let Some(stmt) = block.stmts.first() {
        if let syn::Stmt::Expr(expr, _) = stmt {
            if let syn::Expr::Range(range) = expr {
                return range.start.is_none()
                    && matches!(range.limits, syn::RangeLimits::HalfOpen(_))
                    && range.end.is_none();
            }
        }
    }
    false
}
