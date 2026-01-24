//! Main code generation module
//!
//! This module handles converting rvue AST into Rust code
//! that creates widgets using rvue widget system.

use crate::analysis::{classify_expression, ExpressionKind};
use crate::ast::{RvueAttribute, RvueElement, RvueNode, RvueText, WidgetType};
use crate::widgets::generate_event_handlers;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
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
    }
}

fn generate_element_code(el: &RvueElement, ctx_ident: &Ident) -> TokenStream {
    let component_ident = format_ident!("component");

    match &el.widget_type {
        WidgetType::Custom(name) => {
            let widget_name = Ident::new(name, el.span);
            let props_struct_name = Ident::new(&format!("{}Props", name), el.span);

            let props_init =
                el.attributes.iter().filter(|a| !matches!(a, RvueAttribute::Event { .. })).map(
                    |attr| {
                        let name = format_ident!("{}", attr.name());
                        let PropValue { value, .. } = extract_attr_value(attr);
                        quote! { #name: #value }
                    },
                );

            let events_code = generate_event_handlers_for_element(&component_ident, el);

            quote! {
                {
                    let props = #props_struct_name {
                        #(#props_init),*
                    };
                    let view = #widget_name(props);
                    let #component_ident = rvue::prelude::View::into_component(view);

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
            let PropValue { value: content_value, .. } = props.value("content", || quote! { "" });
            let widget_ident = Ident::new("Text", span);
            quote! {
                rvue::widgets::#widget_ident::new(#content_value.to_string())
            }
        }
        WidgetType::Button => {
            let PropValue { value: label_value, .. } = props.value("label", || quote! { "" });
            let widget_ident = Ident::new("Button", span);
            quote! {
                rvue::widgets::#widget_ident::new(#label_value.to_string())
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
            let PropValue { value: item_count_value, .. } =
                props.value("item_count", || quote! { 0 });
            let widget_ident = Ident::new("For", span);
            quote! {
                rvue::widgets::#widget_ident::new(#item_count_value)
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
    widget_type: &WidgetType,
    attributes: &[RvueAttribute],
    component_ident: &Ident,
) -> TokenStream {
    let props = WidgetProps::new(attributes);
    let mut effects = Vec::new();

    match widget_type {
        WidgetType::Text => {
            let content = props.value("content", || quote! { "" });
            if content.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_text_content },
                    &content.value,
                ));
            }
        }
        WidgetType::Button => {
            let label = props.value("label", || quote! { "" });
            if label.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_button_label },
                    &label.value,
                ));
            }
        }
        WidgetType::Flex => {
            let direction = props.value("direction", || quote! { "row" });
            if direction.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_flex_direction },
                    &direction.value,
                ));
            }
            let gap = props.value("gap", || quote! { 0.0 });
            if gap.is_reactive {
                effects.push(generate_effect(component_ident, quote! { set_flex_gap }, &gap.value));
            }
            let align_items = props.value("align_items", || quote! { "stretch" });
            if align_items.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_flex_align_items },
                    &align_items.value,
                ));
            }
            let justify_content = props.value("justify_content", || quote! { "start" });
            if justify_content.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_flex_justify_content },
                    &justify_content.value,
                ));
            }
        }
        WidgetType::TextInput => {
            let value = props.value("value", || quote! { "" });
            if value.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_text_input_value },
                    &value.value,
                ));
            }
        }
        WidgetType::NumberInput => {
            let value = props.value("value", || quote! { 0.0 });
            if value.is_reactive {
                effects.push(generate_effect(
                    component_ident,
                    quote! { set_number_input_value },
                    &value.value,
                ));
            }
        }
        WidgetType::Checkbox => {
            let checked = props.value("checked", || quote! { false });
            if checked.is_reactive {
                effects.push(generate_effect(
                    component_ident,
                    quote! { set_checkbox_checked },
                    &checked.value,
                ));
            }
        }
        WidgetType::Radio => {
            let value = props.value("value", || quote! { "" });
            if value.is_reactive {
                effects.push(generate_string_effect(
                    component_ident,
                    quote! { set_radio_value },
                    &value.value,
                ));
            }
            let checked = props.value("checked", || quote! { false });
            if checked.is_reactive {
                effects.push(generate_effect(
                    component_ident,
                    quote! { set_radio_checked },
                    &checked.value,
                ));
            }
        }
        WidgetType::Show => {
            let when = props.value("when", || quote! { false });
            if when.is_reactive {
                effects.push(generate_effect(
                    component_ident,
                    quote! { set_show_when },
                    &when.value,
                ));
            }
        }
        WidgetType::For => {
            let item_count = props.value("item_count", || quote! { 0 });
            if item_count.is_reactive {
                effects.push(generate_effect(
                    component_ident,
                    quote! { set_for_item_count },
                    &item_count.value,
                ));
            }
        }
        WidgetType::Custom(_) => {}
    }

    quote! {
        #(#effects)*
    }
}

fn generate_effect(
    component_ident: &Ident,
    setter: TokenStream,
    expr: &TokenStream,
) -> TokenStream {
    quote! {
        {
            let comp = #component_ident.clone();
            let effect = create_effect(move || {
                let new_value = #expr;
                comp.#setter(new_value);
            });
            #component_ident.add_effect(effect);
        }
    }
}

fn generate_string_effect(
    component_ident: &Ident,
    setter: TokenStream,
    expr: &TokenStream,
) -> TokenStream {
    quote! {
        {
            let comp = #component_ident.clone();
            let effect = create_effect(move || {
                let new_value = (#expr).to_string();
                comp.#setter(new_value);
            });
            #component_ident.add_effect(effect);
        }
    }
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
