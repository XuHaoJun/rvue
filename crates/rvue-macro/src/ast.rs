//! Rvue-specific AST representation
//!
//! This module defines the intermediate AST used for code generation.
//! It converts rstml's generic nodes into rvue-specific structures.

#![allow(dead_code)]

use proc_macro2::{Ident, Span};
use syn::Expr;

/// Top-level node in the rvue AST
#[derive(Debug, Clone)]
pub enum RvueNode {
    /// Element node (widget)
    Element(RvueElement),
    /// Text node
    Text(RvueText),
    /// Fragment (multiple root nodes)
    Fragment(Vec<RvueNode>),
    /// Block expression (e.g., {count.get()})
    Block(Expr, Span),
}

/// Element representing a widget
#[derive(Debug, Clone)]
pub struct RvueElement {
    /// Tag name (e.g., "Text", "Button")
    pub tag_name: String,
    /// Widget type
    pub widget_type: WidgetType,
    /// Attributes for this widget
    pub attributes: Vec<RvueAttribute>,
    /// Child nodes
    pub children: Vec<RvueNode>,
    /// Span for error reporting
    pub span: Span,
}

/// Known widget types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetType {
    Text,
    Button,
    Flex,
    TextInput,
    NumberInput,
    Checkbox,
    Radio,
    Show,
    For,
    /// Custom component (PascalCase function name)
    Custom(String),
}

/// Attribute on a widget
#[derive(Debug, Clone)]
pub enum RvueAttribute {
    /// Static attribute with literal value
    Static { name: String, value: String, span: Span },
    /// Dynamic attribute with expression value
    Dynamic { name: String, expr: Expr, span: Span },
    /// Event handler (on_click, on_key_down, etc.)
    Event { name: String, handler: Expr, span: Span },
}

/// Text node
#[derive(Debug, Clone)]
pub struct RvueText {
    /// Text content
    pub content: String,
    /// Span for error reporting
    pub span: Span,
}

impl RvueAttribute {
    /// Get the span of this attribute
    pub fn span(&self) -> Span {
        match self {
            RvueAttribute::Static { span, .. } => *span,
            RvueAttribute::Dynamic { span, .. } => *span,
            RvueAttribute::Event { span, .. } => *span,
        }
    }

    /// Get the name of this attribute
    pub fn name(&self) -> &str {
        match self {
            RvueAttribute::Static { name, .. } => name,
            RvueAttribute::Dynamic { name, .. } => name,
            RvueAttribute::Event { name, .. } => name,
        }
    }
}

impl WidgetType {
    /// Convert to ComponentType identifier for code generation
    pub fn to_type_ident(&self) -> Ident {
        match self {
            WidgetType::Text => quote::format_ident!("Text"),
            WidgetType::Button => quote::format_ident!("Button"),
            WidgetType::Flex => quote::format_ident!("Flex"),
            WidgetType::TextInput => quote::format_ident!("TextInput"),
            WidgetType::NumberInput => quote::format_ident!("NumberInput"),
            WidgetType::Checkbox => quote::format_ident!("Checkbox"),
            WidgetType::Radio => quote::format_ident!("Radio"),
            WidgetType::Show => quote::format_ident!("Show"),
            WidgetType::For => quote::format_ident!("For"),
            WidgetType::Custom(name) => quote::format_ident!("{}", name),
        }
    }

    /// Check if this is a built-in widget
    pub fn is_builtin(&self) -> bool {
        !matches!(self, WidgetType::Custom(_))
    }
}

impl RvueElement {
    /// Find attribute by name
    pub fn find_attr(&self, name: &str) -> Option<&RvueAttribute> {
        self.attributes.iter().find(|a| a.name() == name)
    }

    /// Get all event attributes
    pub fn events(&self) -> Vec<&RvueAttribute> {
        self.attributes.iter().filter(|a| matches!(a, RvueAttribute::Event { .. })).collect()
    }

    /// Get all non-event attributes
    pub fn props(&self) -> Vec<&RvueAttribute> {
        self.attributes.iter().filter(|a| !matches!(a, RvueAttribute::Event { .. })).collect()
    }
}
