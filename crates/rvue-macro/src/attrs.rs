//! Attribute parsing utilities
//!
//! This module provides utilities for parsing and classifying attributes
//! from rstml nodes into rvue-specific attribute representations.

use crate::ast::{RvueAttribute, WidgetType};
use proc_macro2::Span;
use proc_macro_error2::abort;
use quote::ToTokens;
use rstml::node::{KeyedAttribute, NodeAttribute};
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Lit};

/// Parse an attribute from rstml to rvue representation
pub fn parse_attribute(attr: &NodeAttribute) -> Result<RvueAttribute, AttributeError> {
    match attr {
        NodeAttribute::Attribute(k_attr) => parse_keyed_attribute(k_attr),
        NodeAttribute::Block(_) => Err(AttributeError::BlockAttribute),
    }
}

/// Parse a keyed attribute
fn parse_keyed_attribute(attr: &KeyedAttribute) -> Result<RvueAttribute, AttributeError> {
    let name = attr.key.to_string();
    let span = attr.key.span();

    if let Some(slot_name) = name.strip_prefix("slot:") {
        return parse_slot_attr(attr, Some(slot_name.to_string()), span);
    } else if name == "slot" {
        return parse_slot_attr(attr, None, span);
    }

    if let Some(event_name) = name.strip_prefix("on_") {
        parse_event_attr(attr, event_name, span)
    } else if name == "on:click" {
        parse_event_attr(attr, "click", span)
    } else if name == "on:key_down" {
        parse_event_attr(attr, "key_down", span)
    } else if name == "on:pointer_down" {
        parse_event_attr(attr, "pointer_down", span)
    } else {
        parse_prop_attr(attr, &name, span)
    }
}

/// Parse a slot attribute
fn parse_slot_attr(
    attr: &KeyedAttribute,
    slot_name: Option<String>,
    span: Span,
) -> Result<RvueAttribute, AttributeError> {
    let value = attr.value().ok_or(AttributeError::NoValue)?;
    let tokens = value.to_token_stream();
    Ok(RvueAttribute::Slot { name: slot_name, content: tokens, span })
}

/// Parse an event attribute (on_click, on_key_down, etc.)
fn parse_event_attr(
    attr: &KeyedAttribute,
    event_name: &str,
    span: Span,
) -> Result<RvueAttribute, AttributeError> {
    let value_expr = attr.value().ok_or(AttributeError::NoValue)?;

    let handler_expr = parse_handler_expression(value_expr, span)?;

    Ok(RvueAttribute::Event { name: event_name.to_string(), handler: handler_expr, span })
}

/// Parse a property attribute
fn parse_prop_attr(
    attr: &KeyedAttribute,
    name: &str,
    span: Span,
) -> Result<RvueAttribute, AttributeError> {
    match attr.value() {
        Some(expr) => match extract_value_from_expr(expr) {
            Value::Static(s) => {
                Ok(RvueAttribute::Static { name: name.to_string(), value: s, span })
            }
            Value::Dynamic(e) => {
                Ok(RvueAttribute::Dynamic { name: name.to_string(), expr: e, span })
            }
        },
        None => Ok(RvueAttribute::Static { name: name.to_string(), value: String::new(), span }),
    }
}

/// Parse handler expression from attribute value
fn parse_handler_expression(expr: &Expr, _span: Span) -> Result<Expr, AttributeError> {
    Ok(expr.clone())
}

/// Extract value from expression
fn extract_value_from_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => Value::Static(s.value()),
        _ => Value::Dynamic(expr.clone()),
    }
}

/// Classify a tag name into a widget type
pub fn classify_widget(tag_name: &str) -> WidgetType {
    match tag_name {
        "Text" => WidgetType::Text,
        "Button" => WidgetType::Button,
        "Flex" => WidgetType::Flex,
        "TextInput" => WidgetType::TextInput,
        "NumberInput" => WidgetType::NumberInput,
        "Checkbox" => WidgetType::Checkbox,
        "Radio" => WidgetType::Radio,
        "Show" => WidgetType::Show,
        "For" => WidgetType::For,
        _ => {
            if is_pascal_case(tag_name) {
                WidgetType::Custom(tag_name.to_string())
            } else {
                abort!(
                    Span::call_site(),
                    "Unknown widget type: '{}'",
                    tag_name;
                    help = "Available built-in widgets: Text, Button, Flex, TextInput, NumberInput, Checkbox, Radio, Show, For\n\
                            Custom components must be in PascalCase."
                )
            }
        }
    }
}

/// Check if a string is in PascalCase
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_upper = s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

    let has_lowercase = s.chars().any(|c| c.is_lowercase());
    let has_separator = s.contains('_') || s.contains('-');

    first_upper && !has_separator && has_lowercase
}

/// Possible attribute values
#[derive(Debug, Clone)]
enum Value {
    Static(String),
    Dynamic(Expr),
}

/// Attribute parsing errors
#[derive(Debug)]
pub enum AttributeError {
    BlockAttribute,
    NoValue,
}

impl AttributeError {}
