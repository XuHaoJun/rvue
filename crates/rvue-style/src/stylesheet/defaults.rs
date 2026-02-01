//! Built-in default stylesheet (similar to browser default stylesheet).
//!
//! These defaults provide sensible starting sizes for all components.
//! They can be overridden by user-provided stylesheets via CSS selectors.

use crate::properties::{Height, Size, Width};
use crate::property::Properties;
use crate::stylesheet::{StyleRule, Stylesheet};

fn add_size_rule(sheet: &mut Stylesheet, selector: &'static str, width: Width, height: Height) {
    let mut props = Properties::new();
    props.insert(width);
    props.insert(height);
    sheet.add_rule(StyleRule::parse(selector, props));
}

/// Creates a stylesheet with built-in default styles for all components.
///
/// This is similar to a browser's default stylesheet - it provides sensible
/// starting points for component sizing that can be overridden by user CSS.
///
/// # Example
///
/// ```rust
/// use rvue_style::default_stylesheet;
///
/// let defaults = default_stylesheet();
/// // User can now override these defaults with their own stylesheet
/// ```
pub fn default_stylesheet() -> Stylesheet {
    let mut sheet = Stylesheet::new();

    // Button defaults
    add_size_rule(&mut sheet, "button", Width(Size::Pixels(120.0)), Height(Size::Pixels(40.0)));

    // Input defaults (general input tag)
    add_size_rule(&mut sheet, "input", Width(Size::Pixels(200.0)), Height(Size::Pixels(30.0)));

    // Checkbox defaults - use class selector for now
    add_size_rule(&mut sheet, "checkbox", Width(Size::Pixels(20.0)), Height(Size::Pixels(20.0)));

    // Radio defaults - use class selector for now
    add_size_rule(&mut sheet, "radio", Width(Size::Pixels(20.0)), Height(Size::Pixels(20.0)));

    // Progress bar defaults
    add_size_rule(&mut sheet, "progress", Width(Size::Pixels(200.0)), Height(Size::Pixels(20.0)));

    // Select defaults
    add_size_rule(&mut sheet, "select", Width(Size::Pixels(200.0)), Height(Size::Pixels(35.0)));

    // Textarea defaults
    add_size_rule(&mut sheet, "textarea", Width(Size::Pixels(300.0)), Height(Size::Pixels(100.0)));

    sheet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selectors::RvueElement;
    use crate::stylesheet::StyleResolver;

    #[test]
    fn test_default_stylesheet_has_button_rules() {
        let sheet = default_stylesheet();
        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        assert!(resolved.width.is_some(), "button should have default width");
        assert!(resolved.height.is_some(), "button should have default height");
    }

    #[test]
    fn test_default_stylesheet_has_input_rules() {
        let sheet = default_stylesheet();
        let element = RvueElement::new("input");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // input tag should match the "input" selector
        assert!(resolved.width.is_some(), "input should have default width");
        assert!(resolved.height.is_some(), "input should have default height");
    }

    #[test]
    fn test_default_stylesheet_is_not_empty() {
        let sheet = default_stylesheet();
        assert!(!sheet.is_empty(), "default stylesheet should not be empty");
    }

    #[test]
    fn test_default_stylesheet_has_checkbox_rules() {
        let sheet = default_stylesheet();
        let element = RvueElement::new("checkbox");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        assert!(resolved.width.is_some(), "checkbox should have default width");
        assert!(resolved.height.is_some(), "checkbox should have default height");
    }
}
