//! Main code generation module
//!
//! This module handles converting rvue AST into Rust code
//! that creates widgets using rvue widget system.

use crate::ast::{RvueAttribute, RvueElement, RvueNode, RvueText, WidgetType};
use crate::widgets::{generate_event_handlers, generate_widget_code};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use rstml::node::{Node, NodeAttribute, NodeElement, NodeName};
use std::sync::atomic::{AtomicU64, Ordering};
use syn::spanned::Spanned;
use syn::{Block, Expr};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Generate view code from AST nodes
pub fn generate_view_code(nodes: Vec<RvueNode>) -> TokenStream {
    match nodes.len() {
        0 => generate_empty_view(),
        1 => generate_node_code(&nodes[0]),
        _ => generate_fragment_code(nodes),
    }
}

fn generate_empty_view() -> TokenStream {
    quote! {
        rvue::ViewStruct::new(
            rvue::Component::new_empty()
        )
    }
}

fn generate_node_code(node: &RvueNode) -> TokenStream {
    match node {
        RvueNode::Element(el) => generate_element_code(el),
        RvueNode::Text(text) => generate_text_node_code(text),
        RvueNode::Fragment(nodes) => generate_fragment_code(nodes.clone()),
    }
}

fn generate_unique_id() -> (u64, Ident) {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    (id, format_ident!("comp_{}", id))
}

fn generate_element_code(el: &RvueElement) -> TokenStream {
    let (id_val, id_ident) = generate_unique_id();

    let widget_code = generate_widget_code(&el.widget_type, id_val, &el.attributes);

    let children_code = if !el.children.is_empty() {
        generate_children_code(&el.children, &id_ident)
    } else {
        quote! {}
    };

    let events = el
        .events()
        .iter()
        .filter_map(|a| match a {
            RvueAttribute::Event { name, handler, .. } => Some((name.as_str(), handler)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let events_code = if !events.is_empty() {
        generate_event_handlers(&id_ident, &events)
    } else {
        quote! {}
    };

    quote! {
        let #id_ident = #widget_code;
        #children_code
        #events_code
        #id_ident
    }
}

fn generate_text_node_code(text: &RvueText) -> TokenStream {
    let (id_val, _id_ident) = generate_unique_id();
    let content = &text.content;

    quote! {
        rvue::widgets::Text::new(
            #id_val,
            #content.to_string()
        )
    }
}

fn generate_fragment_code(nodes: Vec<RvueNode>) -> TokenStream {
    match nodes.len() {
        0 => generate_empty_view(),
        1 => {
            let root = generate_node_code(&nodes[0]);
            quote! {
                rvue::ViewStruct::new(#root)
            }
        }
        _ => {
            let (root_id_val, root_id_ident) = generate_unique_id();
            let root = quote! {
                rvue::Component::new(
                    #root_id_val,
                    rvue::ComponentType::Flex,
                    rvue::ComponentProps::Flex {
                        direction: "row".to_string(),
                        gap: 0.0,
                        align_items: "start".to_string(),
                        justify_content: "start".to_string(),
                    }
                )
            };

            let children_code = generate_children_code(&nodes, &root_id_ident);

            quote! {
                let #root_id_ident = #root;
                #children_code
                rvue::ViewStruct::new(#root_id_ident)
            }
        }
    }
}

fn generate_children_code(children: &[RvueNode], parent_id: &Ident) -> TokenStream {
    let child_vars = children.iter().map(|child| {
        let child_code = generate_node_code(child);
        quote! {
            let child = #child_code;
            #parent_id.add_child(child);
        }
    });

    quote! {
        #(#child_vars)*
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
