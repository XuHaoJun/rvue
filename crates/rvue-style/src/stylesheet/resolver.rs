//! Style resolver for matching styles to elements.

use crate::properties::ComputedStyles;
use crate::property::Properties;
use crate::selectors::RvueElement;
use crate::stylesheet::rule::Stylesheet;

/// Resolves matched styles for an element from a stylesheet.
#[derive(Debug, Default)]
pub struct StyleResolver;

impl StyleResolver {
    /// Creates a new style resolver.
    pub fn new() -> Self {
        Self
    }

    /// Resolves styles for an element from a stylesheet.
    pub fn resolve_styles(&self, element: &RvueElement, stylesheet: &Stylesheet) -> ComputedStyles {
        let mut result = ComputedStyles::default();

        for rule in stylesheet.rules() {
            if self.matches_selector(element, &rule.selector) {
                result.merge(&rule.properties);
            }
        }

        result
    }

    /// Resolves styles using raw selector and properties.
    pub fn resolve_single(
        &self,
        element: &RvueElement,
        selector: &str,
        properties: &Properties,
    ) -> ComputedStyles {
        let mut result = ComputedStyles::default();

        if self.matches_selector(element, selector) {
            result.merge(properties);
        }

        result
    }

    /// Checks if an element matches a CSS selector.
    fn matches_selector(&self, element: &RvueElement, selector: &str) -> bool {
        let selector = selector.trim();

        if selector.is_empty() {
            return false;
        }

        let parts: Vec<&str> = selector.split(' ').collect();
        self.matches_selector_chain(element, &parts)
    }

    /// Matches a chain of selectors (for descendant selectors).
    fn matches_selector_chain(&self, element: &RvueElement, parts: &[&str]) -> bool {
        if parts.is_empty() {
            return true;
        }

        let Some((first, rest)) = parts.split_first() else {
            return true;
        };

        if !self.matches_simple_selector(element, first) {
            return false;
        }

        if rest.is_empty() {
            return true;
        }

        let mut current = element.parent.as_deref();
        while let Some(parent) = current {
            if self.matches_selector_chain(parent, rest) {
                return true;
            }
            current = parent.parent.as_deref();
        }

        false
    }

    /// Matches a simple selector (type, class, ID, state).
    fn matches_simple_selector(&self, element: &RvueElement, selector: &str) -> bool {
        let selector = selector.trim();

        if selector.is_empty() {
            return true;
        }

        if selector.starts_with('.') {
            let class = &selector[1..];
            element.has_class(class)
        } else if selector.starts_with('#') {
            let id = &selector[1..];
            element.has_id(id)
        } else if selector.starts_with(':') {
            let state_name = &selector[1..];
            self.matches_pseudo_class(element, state_name)
        } else {
            element.has_tag_name(selector)
        }
    }

    /// Matches pseudo-class selectors.
    fn matches_pseudo_class(&self, element: &RvueElement, pseudo_class: &str) -> bool {
        match pseudo_class {
            "hover" => element.is_in_state(crate::selectors::ElementState::HOVER),
            "focus" => element.is_in_state(crate::selectors::ElementState::FOCUS),
            "active" => element.is_in_state(crate::selectors::ElementState::ACTIVE),
            "disabled" => element.is_in_state(crate::selectors::ElementState::DISABLED),
            "checked" => element.is_in_state(crate::selectors::ElementState::CHECKED),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::BackgroundColor;
    use crate::property::Properties;
    use crate::stylesheet::rule::StyleRule;

    #[test]
    fn test_matches_tag_selector() {
        let resolver = StyleResolver::new();
        let element = RvueElement::new("div");

        assert!(resolver.matches_simple_selector(&element, "div"));
        assert!(!resolver.matches_simple_selector(&element, "span"));
    }

    #[test]
    fn test_matches_class_selector() {
        let resolver = StyleResolver::new();
        let element = RvueElement::new("div").with_class("container");

        assert!(resolver.matches_simple_selector(&element, ".container"));
        assert!(!resolver.matches_simple_selector(&element, ".header"));
    }

    #[test]
    fn test_matches_id_selector() {
        let resolver = StyleResolver::new();
        let element = RvueElement::new("div").with_id("main");

        assert!(resolver.matches_simple_selector(&element, "#main"));
        assert!(!resolver.matches_simple_selector(&element, "#other"));
    }

    #[test]
    fn test_matches_pseudo_class() {
        let resolver = StyleResolver::new();
        let mut element = RvueElement::new("button");
        element.state.insert(crate::selectors::ElementState::HOVER);

        assert!(resolver.matches_simple_selector(&element, ":hover"));
        assert!(!resolver.matches_simple_selector(&element, ":focus"));
    }

    #[test]
    fn test_resolve_styles() {
        let resolver = StyleResolver::new();
        let mut stylesheet = Stylesheet::new();

        let mut props = Properties::new();
        props.insert(BackgroundColor(crate::properties::Color::rgb(255, 0, 0)));
        stylesheet.add_rule(StyleRule::new("button".to_string(), props));

        let element = RvueElement::new("button");
        let resolved = resolver.resolve_styles(&element, &stylesheet);

        assert!(resolved.background_color.is_some());
    }
}
