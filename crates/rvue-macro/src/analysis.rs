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
    /// Expression is used as an event handler
    EventHandler,
}

/// Determine whether an expression is likely reactive
pub fn classify_expression(expr: &Expr) -> ExpressionKind {
    let mut detector = ReactiveDetector { is_reactive: false };
    detector.visit_expr(expr);
    if detector.is_reactive {
        ExpressionKind::Reactive
    } else {
        ExpressionKind::Static
    }
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
