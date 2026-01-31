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

        if !self.matches_compound_selector(element, first) {
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

    /// Matches a compound selector (e.g., "button.primary", "button:hover", "div.container.large").
    fn matches_compound_selector(&self, element: &RvueElement, selector: &str) -> bool {
        let selector = selector.trim();

        if selector.is_empty() {
            return true;
        }

        let mut remaining = selector;

        while !remaining.is_empty() {
            let (matched, next_remaining) = self.match_simple_selector_prefix(element, remaining);
            if !matched {
                return false;
            }
            remaining = next_remaining;
        }

        true
    }

    /// Matches a simple selector prefix and returns the remaining string.
    fn match_simple_selector_prefix<'a>(
        &self,
        element: &RvueElement,
        selector: &'a str,
    ) -> (bool, &'a str) {
        let selector = selector.trim();

        if selector.is_empty() {
            return (true, "");
        }

        if let Some(class) = selector.strip_prefix('.') {
            let class_name = self.extract_identifier(class);
            let after_class = &class[class_name.len()..];
            let next_start = selector.len() - after_class.len();
            return (element.has_class(class_name), &selector[next_start..]);
        }

        if let Some(id) = selector.strip_prefix('#') {
            let id_name = self.extract_identifier(id);
            let after_id = &id[id_name.len()..];
            let next_start = selector.len() - after_id.len();
            return (element.has_id(id_name), &selector[next_start..]);
        }

        if let Some(state_name) = selector.strip_prefix(':') {
            let pseudo_class = self.extract_identifier(state_name);
            let matched = element.state.matches_pseudo_class(pseudo_class);
            let after_state = &state_name[pseudo_class.len()..];
            let next_start = selector.len() - after_state.len();
            return (matched, &selector[next_start..]);
        }

        let tag_name = self.extract_identifier(selector);
        let matched = element.has_tag_name(tag_name);
        let after_tag = &selector[tag_name.len()..];
        let next_start = selector.len() - after_tag.len();
        return (matched, &selector[next_start..]);
    }

    /// Extracts an identifier (alphanumeric or hyphenated) from the start of a string.
    fn extract_identifier<'a>(&self, s: &'a str) -> &'a str {
        let mut end = 0;
        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        &s[..end]
    }

    /// Matches a simple selector (type, class, ID, state).
    fn matches_simple_selector(&self, element: &RvueElement, selector: &str) -> bool {
        let selector = selector.trim();

        if selector.is_empty() {
            return true;
        }

        if let Some(class) = selector.strip_prefix('.') {
            element.has_class(class)
        } else if let Some(id) = selector.strip_prefix('#') {
            element.has_id(id)
        } else if let Some(state_name) = selector.strip_prefix(':') {
            self.matches_pseudo_class(element, state_name)
        } else {
            element.has_tag_name(selector)
        }
    }

    /// Matches pseudo-class selectors.
    fn matches_pseudo_class(&self, element: &RvueElement, pseudo_class: &str) -> bool {
        element.state.matches_pseudo_class(pseudo_class)
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

    #[test]
    fn test_matches_compound_selector_tag_class() {
        let resolver = StyleResolver::new();
        let element = RvueElement::new("button").with_class("primary");

        assert!(resolver.matches_compound_selector(&element, "button.primary"));
        assert!(!resolver.matches_compound_selector(&element, "button.secondary"));
        assert!(!resolver.matches_compound_selector(&element, "span.primary"));
    }

    #[test]
    fn test_matches_compound_selector_tag_state() {
        let resolver = StyleResolver::new();
        let mut element = RvueElement::new("button");
        element.state.insert(crate::selectors::ElementState::HOVER);

        assert!(resolver.matches_compound_selector(&element, "button:hover"));
        assert!(!resolver.matches_compound_selector(&element, "button:focus"));
    }

    #[test]
    fn test_matches_compound_selector_tag_class_state() {
        let resolver = StyleResolver::new();
        let mut element = RvueElement::new("button").with_class("primary");
        element.state.insert(crate::selectors::ElementState::HOVER);

        assert!(resolver.matches_compound_selector(&element, "button.primary:hover"));
        assert!(!resolver.matches_compound_selector(&element, "button.secondary:hover"));
        assert!(!resolver.matches_compound_selector(&element, "button.primary:focus"));
    }

    #[test]
    fn test_resolve_compound_selector_styles() {
        let resolver = StyleResolver::new();
        let mut stylesheet = Stylesheet::new();

        let mut props = Properties::new();
        props.insert(BackgroundColor(crate::properties::Color::rgb(0, 123, 255)));
        stylesheet.add_rule(StyleRule::new("button.primary".to_string(), props));

        let element = RvueElement::new("button").with_class("primary");
        let resolved = resolver.resolve_styles(&element, &stylesheet);
        assert!(resolved.background_color.is_some());

        let element_no_class = RvueElement::new("button");
        let resolved_no_class = resolver.resolve_styles(&element_no_class, &stylesheet);
        assert!(resolved_no_class.background_color.is_none());
    }
}
