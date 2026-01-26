//! Main code generation module
//!
//! This module handles converting rvue AST into Rust code
//! that creates widgets using rvue widget system.

use crate::analysis::{classify_expression, ExpressionKind};
use crate::ast::{RvueAttribute, RvueElement, RvueNode, RvueText, WidgetType};
use crate::widgets::generate_event_handlers;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use rstml::node::{Node, NodeAttribute, NodeElement, NodeName};
use syn::spanned::Spanned;
use syn::{Block, Expr};

/// Generate view code from AST nodes
pub fn generate_view_code(nodes: Vec<RvueNode>) -> TokenStream {
    let ctx_ident = format_ident!("ctx");
    let root_component = match nodes.len() {
        0 => generate_empty_component(&ctx_ident),
        1 => generate_node_code(&nodes[0], &ctx_ident),
        _ => generate_fragment_code(nodes, &ctx_ident),
    };

    quote! {
        {
            use rvue::widget::{BuildContext, Widget};
            use rvue::text::TextContext;
            use rvue::TaffyTree;

            let mut taffy = TaffyTree::new();
            let mut text_context = TextContext::new();
            let mut id_counter = 0u64;
            let mut #ctx_ident = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

            let root_component = #root_component;
            rvue::ViewStruct::new(root_component)
        }
    }
}

fn generate_empty_component(ctx_ident: &Ident) -> TokenStream {
    quote! {
        {
            let widget = rvue::widgets::Flex::new();
            let state = widget.build(&mut #ctx_ident);
            state.component().clone()
        }
    }
}

fn generate_node_code(node: &RvueNode, ctx_ident: &Ident) -> TokenStream {
    match node {
        RvueNode::Element(el) => generate_element_code(el, ctx_ident),
        RvueNode::Text(text) => generate_text_node_code(text, ctx_ident),
        RvueNode::Fragment(nodes) => generate_fragment_code(nodes.clone(), ctx_ident),
        RvueNode::Block(expr, span) => generate_block_node_code(expr, *span, ctx_ident),
    }
}

fn generate_element_code(el: &RvueElement, ctx_ident: &Ident) -> TokenStream {
    let component_ident = format_ident!("component");

    match &el.widget_type {
        WidgetType::Custom(name) => {
            let widget_name = Ident::new(name, el.span);
            let props_struct_name = Ident::new(&format!("{}Props", name), el.span);

            let (slot_attrs, normal_attrs): (Vec<_>, _) =
                el.attributes.iter().partition(|a| a.is_slot());

            let props_init = normal_attrs
                .iter()
                .filter(|a| !matches!(a, RvueAttribute::Event { .. }))
                .map(|attr| {
                    let name = format_ident!("{}", attr.name());
                    let PropValue { value, is_reactive } = extract_attr_value(attr);
                    if is_reactive {
                        quote! { #name: rvue::signal::create_memo(move || #value).into_reactive() }
                    } else {
                        quote! { #name: rvue::widget::IntoReactiveValue::into_reactive(#value) }
                    }
                });

            let events_code = generate_event_handlers_for_element(&component_ident, el);

            let slot_code = if !slot_attrs.is_empty() {
                generate_slot_injection(&slot_attrs, &component_ident)
            } else {
                quote! {}
            };

            quote! {
                {
                    let props = #props_struct_name {
                        #(#props_init),*
                    };

                    let #component_ident = #ctx_ident.create_component(
                        rvue::component::ComponentType::Custom(#name.to_string()),
                        rvue::component::ComponentProps::Custom { data: String::new() }
                    );

                    let view = rvue::runtime::with_owner(#component_ident.clone(), || #widget_name(props));

                    let inner_comp = rvue::prelude::View::into_component(view);
                    #component_ident.add_child(inner_comp);

                    #slot_code

                    #events_code

                    #component_ident
                }
            }
        }
        _ => {
            let widget_code =
                generate_widget_builder_code(&el.widget_type, &el.attributes, el.span);
            let effects_code =
                generate_reactive_effects(&el.widget_type, &el.attributes, &component_ident);
            let children_code = generate_children_code(&el.children, &component_ident, ctx_ident);
            let events_code = generate_event_handlers_for_element(&component_ident, el);

            quote! {
                {
                    let widget = #widget_code;
                    let state = widget.build(&mut #ctx_ident);
                    let #component_ident = state.component().clone();

                    #children_code
                    #events_code
                    #effects_code

                    #component_ident
                }
            }
        }
    }
}

/// Generate code to inject slot content into a parent component
fn generate_slot_injection(slot_attrs: &[&RvueAttribute], component_ident: &Ident) -> TokenStream {
    let slot_injections: Vec<TokenStream> = slot_attrs
        .iter()
        .filter_map(|attr| attr.as_slot())
        .map(|(slot_name, content)| {
            let slot_var = if let Some(name) = slot_name {
                let slot_ident =
                    Ident::new(&convert_to_snake_case(name), proc_macro2::Span::call_site());
                quote! { #slot_ident }
            } else {
                let slot_ident = Ident::new("children", proc_macro2::Span::call_site());
                quote! { #slot_ident }
            };

            quote! {
                {
                    let #slot_var = rvue::slot::ToChildren::to_children(move || {
                        rvue::runtime::with_owner(#component_ident.clone(), || #content)
                    });
                    let slot_view = #slot_var.run();
                    let inner_comp = rvue::prelude::View::into_component(slot_view);
                    #component_ident.add_child(inner_comp);
                }
            }
        })
        .collect();

    quote! {
        #(#slot_injections)*
    }
}

/// Convert a string to snake_case
fn convert_to_snake_case(name: &str) -> String {
    let mut result = String::new();

    for c in name.chars() {
        if c.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

fn generate_text_node_code(text: &RvueText, ctx_ident: &Ident) -> TokenStream {
    let content = &text.content;

    quote! {
        {
            let widget = rvue::widgets::Text::new(#content.to_string());
            let state = widget.build(&mut #ctx_ident);
            state.component().clone()
        }
    }
}

fn generate_block_node_code(expr: &Expr, span: Span, ctx_ident: &Ident) -> TokenStream {
    let kind = classify_expression(expr);

    match kind {
        ExpressionKind::Static => {
            quote! {
                {
                    let widget = rvue::widgets::Text::new((#expr).to_string());
                    let state = widget.build(&mut #ctx_ident);
                    state.component().clone()
                }
            }
        }
        ExpressionKind::Reactive => {
            quote_spanned! { span =>
                {
                    // Pass the signal directly to preserve reactivity
                    // The widget will call .get() internally when building
                    let widget = rvue::widgets::Text::new(#expr);
                    let state = widget.build(&mut #ctx_ident);
                    state.component().clone()
                }
            }
        }
        ExpressionKind::ViewStruct => {
            quote_spanned! { span =>
                {
                    let view_struct = #expr;
                    let inner_comp = rvue::prelude::View::into_component(view_struct);
                    inner_comp
                }
            }
        }
    }
}

fn generate_fragment_code(nodes: Vec<RvueNode>, ctx_ident: &Ident) -> TokenStream {
    match nodes.len() {
        0 => generate_empty_component(ctx_ident),
        1 => generate_node_code(&nodes[0], ctx_ident),
        _ => {
            let root_id = format_ident!("root");
            let children_code = generate_children_code(&nodes, &root_id, ctx_ident);

            quote! {
                {
                    let root_widget = rvue::widgets::Flex::new()
                        .direction(rvue::style::FlexDirection::Row)
                        .gap(0.0)
                        .align_items(rvue::style::AlignItems::Start)
                        .justify_content(rvue::style::JustifyContent::Start);
                    let root_state = root_widget.build(&mut #ctx_ident);
                    let #root_id = root_state.component().clone();

                    #children_code

                    #root_id
                }
            }
        }
    }
}

fn generate_children_code(
    children: &[RvueNode],
    parent_id: &Ident,
    ctx_ident: &Ident,
) -> TokenStream {
    let child_vars = children.iter().map(|child| match child {
        RvueNode::Element(el) => {
            let child_code = generate_element_code(el, ctx_ident);
            quote! {
                {
                    let child = #child_code;
                    #parent_id.add_child(child);
                }
            }
        }
        RvueNode::Text(text) => {
            let text_code = generate_text_node_code(text, ctx_ident);
            quote! {
                {
                    let child = #text_code;
                    #parent_id.add_child(child);
                }
            }
        }
        RvueNode::Block(expr, span) => {
            let block_code = generate_block_node_code(expr, *span, ctx_ident);
            quote! {
                {
                    let child = #block_code;
                    #parent_id.add_child(child);
                }
            }
        }
        RvueNode::Fragment(nodes) => {
            let fragment_children = nodes.iter().map(|node| {
                let child_code = generate_node_code(node, ctx_ident);
                quote! {
                    {
                        let child = #child_code;
                        #parent_id.add_child(child);
                    }
                }
            });
            quote! { #(#fragment_children)* }
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
    span: Span,
) -> TokenStream {
    let props = WidgetProps::new(attributes);

    match widget_type {
        WidgetType::Text => {
            let PropValue { value: content_value, is_reactive } =
                props.value("content", || quote! { "" });
            let widget_ident = Ident::new("Text", span);
            if is_reactive {
                quote! {
                    rvue::widgets::#widget_ident::new(#content_value)
                }
            } else {
                quote! {
                    rvue::widgets::#widget_ident::new(#content_value.to_string())
                }
            }
        }
        WidgetType::Button => {
            let PropValue { value: label_value, is_reactive } =
                props.value("label", || quote! { "" });
            let widget_ident = Ident::new("Button", span);
            if is_reactive {
                quote! {
                    rvue::widgets::#widget_ident::new(#label_value)
                }
            } else {
                quote! {
                    rvue::widgets::#widget_ident::new(#label_value.to_string())
                }
            }
        }
        WidgetType::Flex => {
            let PropValue { value: direction_value, .. } =
                props.value("direction", || quote! { "row" });
            let PropValue { value: gap_value, .. } = props.value("gap", || quote! { 0.0 });
            let PropValue { value: align_items_value, .. } =
                props.value("align_items", || quote! { "stretch" });
            let PropValue { value: justify_content_value, .. } =
                props.value("justify_content", || quote! { "start" });

            let widget_ident = Ident::new("Flex", span);

            quote! {
                {
                    use rvue::style::{FlexDirection, AlignItems, JustifyContent};
                    let direction_str = #direction_value.to_string();
                    let direction_enum = match direction_str.as_str() {
                        "row" => FlexDirection::Row,
                        "column" => FlexDirection::Column,
                        "row-reverse" => FlexDirection::RowReverse,
                        "column-reverse" => FlexDirection::ColumnReverse,
                        _ => FlexDirection::Row,
                    };
                    let align_str = #align_items_value.to_string();
                    let align_enum = match align_str.as_str() {
                        "start" => AlignItems::Start,
                        "end" => AlignItems::End,
                        "center" => AlignItems::Center,
                        "stretch" => AlignItems::Stretch,
                        "baseline" => AlignItems::Baseline,
                        _ => AlignItems::Stretch,
                    };
                    let justify_str = #justify_content_value.to_string();
                    let justify_enum = match justify_str.as_str() {
                        "start" => JustifyContent::Start,
                        "end" => JustifyContent::End,
                        "center" => JustifyContent::Center,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        "space-evenly" => JustifyContent::SpaceEvenly,
                        _ => JustifyContent::Start,
                    };
                    rvue::widgets::#widget_ident::new()
                        .direction(direction_enum)
                        .gap(#gap_value)
                        .align_items(align_enum)
                        .justify_content(justify_enum)
                }
            }
        }
        WidgetType::TextInput => {
            let PropValue { value: value_value, .. } = props.value("value", || quote! { "" });
            let widget_ident = Ident::new("TextInput", span);
            quote! {
                rvue::widgets::#widget_ident::new(#value_value.to_string())
            }
        }
        WidgetType::NumberInput => {
            let PropValue { value: value_value, .. } = props.value("value", || quote! { 0.0 });
            let widget_ident = Ident::new("NumberInput", span);
            quote! {
                rvue::widgets::#widget_ident::new(#value_value)
            }
        }
        WidgetType::Checkbox => {
            let PropValue { value: checked_value, .. } =
                props.value("checked", || quote! { false });
            let widget_ident = Ident::new("Checkbox", span);
            quote! {
                rvue::widgets::#widget_ident::new(#checked_value)
            }
        }
        WidgetType::Radio => {
            let PropValue { value: value_value, .. } = props.value("value", || quote! { "" });
            let PropValue { value: checked_value, .. } =
                props.value("checked", || quote! { false });
            let widget_ident = Ident::new("Radio", span);
            quote! {
                rvue::widgets::#widget_ident::new(#value_value.to_string(), #checked_value)
            }
        }
        WidgetType::Show => {
            let PropValue { value: when_value, .. } = props.value("when", || quote! { false });
            let widget_ident = Ident::new("Show", span);
            quote! {
                rvue::widgets::#widget_ident::new(#when_value)
            }
        }
        WidgetType::For => {
            let PropValue { value: items_value, .. } = props.value("each", || quote! { vec![] });
            let PropValue { value: key_fn, .. } = props.value("key", || quote! { |item| item });
            let PropValue { value: view_fn, .. } = props.value(
                "view",
                || quote! { |item| view! { <Text content={format!("{:?}", item)} />} },
            );
            let widget_ident = Ident::new("For", span);
            quote! {
                rvue::widgets::#widget_ident::new(#items_value, #key_fn, #view_fn)
            }
        }
        WidgetType::Custom(name) => {
            let widget_name = Ident::new(name, span);
            let props = attributes
                .iter()
                .filter(|a| !matches!(a, RvueAttribute::Event { .. }))
                .map(|attr| {
                    let name = format_ident!("{}", attr.name());
                    let PropValue { value, .. } = extract_attr_value(attr);
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

fn generate_reactive_effects(
    _widget_type: &WidgetType,
    _attributes: &[RvueAttribute],
    _component_ident: &Ident,
) -> TokenStream {
    // Widgets handle reactive content internally in their build() methods.
    // The effect is created there if the content is reactive.
    // So we do not need to generate additional effects here.
    quote! {}
}

struct PropValue {
    value: TokenStream,
    is_reactive: bool,
}

struct WidgetProps<'a> {
    attributes: &'a [RvueAttribute],
}

impl<'a> WidgetProps<'a> {
    fn new(attributes: &'a [RvueAttribute]) -> Self {
        Self { attributes }
    }

    fn value<F>(&self, name: &str, default: F) -> PropValue
    where
        F: FnOnce() -> TokenStream,
    {
        self.attributes
            .iter()
            .find(|attr| attr.name() == name)
            .map(extract_attr_value)
            .unwrap_or_else(|| PropValue { value: default(), is_reactive: false })
    }
}

fn extract_attr_value(attr: &RvueAttribute) -> PropValue {
    match attr {
        RvueAttribute::Static { value, .. } => {
            PropValue { value: quote! { #value }, is_reactive: false }
        }
        RvueAttribute::Dynamic { expr, .. } => {
            let is_reactive = matches!(classify_expression(expr), ExpressionKind::Reactive);
            PropValue { value: quote! { #expr }, is_reactive }
        }
        RvueAttribute::Event { .. } => PropValue {
            value: quote! { compile_error!("Unexpected event attribute in property position") },
            is_reactive: false,
        },
        RvueAttribute::Slot { .. } => PropValue {
            value: quote! { compile_error!("Unexpected slot attribute in property position") },
            is_reactive: false,
        },
    }
}

pub fn convert_rstml_to_rvue(node: &Node, _parent_type: Option<&WidgetType>) -> Option<RvueNode> {
    match node {
        Node::Comment(_) => None,
        Node::Doctype(_) => None,
        Node::Fragment(frag) => {
            let children: Vec<RvueNode> = frag
                .children
                .iter()
                .filter_map(|n| convert_rstml_to_rvue(n, _parent_type))
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
                Some(RvueNode::Block(expr, block.span()))
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
    if let Some(syn::Stmt::Expr(syn::Expr::Range(range), _)) = block.stmts.first() {
        return range.start.is_none()
            && matches!(range.limits, syn::RangeLimits::HalfOpen(_))
            && range.end.is_none();
    }
    false
}
