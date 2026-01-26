//! Expression analysis utilities for view! macro
//!
//! This module provides simple heuristics to classify expressions
//! as static or reactive for code generation.

use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall};

/// Classification for expressions used in attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Expression appears to be static
    Static,
    /// Expression appears to be reactive (reads signals)
    Reactive,
    /// Expression returns a ViewStruct
    ViewStruct,
}

/// Determine whether an expression is likely reactive
pub fn classify_expression(expr: &Expr) -> ExpressionKind {
    // First check if it's a ViewStruct type
    if is_view_struct_type(expr) {
        return ExpressionKind::ViewStruct;
    }

    // Check if it's a signal variable (heuristic)
    if is_signal_variable(expr) {
        return ExpressionKind::Reactive;
    }

    let mut detector = ReactiveDetector { is_reactive: false };
    detector.visit_expr(expr);
    if detector.is_reactive {
        ExpressionKind::Reactive
    } else {
        ExpressionKind::Static
    }
}

/// Check if an expression is a signal variable (path expression ending with common signal patterns)
fn is_signal_variable(expr: &Expr) -> bool {
    match expr {
        Expr::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                let name = segment.ident.to_string();
                // Common signal variable name patterns
                if name.ends_with("_signal")
                    || name.ends_with("_signal_rc")
                    || name.ends_with("_value")
                    || name.ends_with("_state")
                    || name.ends_with("_clone")
                    || name.ends_with("_s")
                    || name == "signal"
                    || name == "sig"
                    || name == "count"
                    || name == "text"
                    || name == "name"
                    || name == "label"
                    || name == "value"
                    || name == "doubled"
                {
                    return true;
                }
                // Check for lowercase identifiers that might be signals
                // (this is a heuristic - only treat single-char names or common short names as signals)
                // This avoids false positives for things like "children", "view", "slot", etc.
                let first_char = name.chars().next();
                if first_char.is_some_and(|c| c.is_lowercase())
                    && !name.starts_with("is_")
                    && !name.starts_with("has_")
                    && name.len() <= 2
                {
                    // Could be a signal, be conservative and treat as reactive
                    return true;
                }
            }
        }
        Expr::MethodCall(expr_method) => {
            // Check if it's a .clone() call on a signal
            if expr_method.method == "clone" {
                // Recursively check the receiver
                return is_signal_variable(&expr_method.receiver);
            }
        }
        Expr::Block(expr_block) => {
            // Recursively check the inner expression
            if let Some(syn::Stmt::Expr(inner_expr, _)) = expr_block.block.stmts.last() {
                return is_signal_variable(inner_expr);
            }
        }
        _ => {}
    }
    false
}

/// Check if an expression is a ViewStruct by looking at its type path
fn is_view_struct_type(expr: &Expr) -> bool {
    match expr {
        Expr::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                let name = segment.ident.to_string();
                // Check for ViewStruct or ViewStruct::new()
                if name == "ViewStruct" {
                    return true;
                }
                // Also check for lowercase 'view_struct' which might be a variable
                // In this case, we can't know for sure, so we treat it as ViewStruct
                // if it looks like it could be one (single identifier that's not a signal get)
                if name == "view_struct" {
                    return true;
                }
            }
        }
        Expr::Macro(expr_macro) => {
            // Check for macros like `view! { ... }` which might return ViewStruct
            let path = &expr_macro.mac.path;
            if let Some(segment) = path.get_ident() {
                if segment == "view" {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

struct ReactiveDetector {
    is_reactive: bool,
}

impl<'ast> Visit<'ast> for ReactiveDetector {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if self.is_reactive {
            return;
        }

        let method = node.method.to_string();
        if method == "get" || method == "get_untracked" {
            self.is_reactive = true;
            return;
        }

        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if self.is_reactive {
            return;
        }

        if let Expr::Path(path) = &*node.func {
            if let Some(segment) = path.path.segments.last() {
                let name = segment.ident.to_string();
                if name == "get" || name == "get_untracked" {
                    self.is_reactive = true;
                    return;
                }
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        if self.is_reactive {
            return;
        }

        // We cannot inspect macro token streams reliably, so treat as reactive.
        let _ = node;
        self.is_reactive = true;
    }
}
