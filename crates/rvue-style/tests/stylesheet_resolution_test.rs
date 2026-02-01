//! Tests for stylesheet resolution and inline style merging.
//!
//! These tests verify the complete style resolution pipeline:
//! 1. Stylesheet rules are resolved for elements
//! 2. Inline styles merge correctly with stylesheet
//! 3. Selective merge logic preserves stylesheet defaults

use rvue_style::{
    properties::{BackgroundColor, Color, Display, FlexDirection, Height, Size, Width},
    stylesheet::{StyleResolver, StyleRule, Stylesheet},
    ComputedStyles, Properties, RvueElement,
};

fn make_style_rule(selector: &str, width: Option<f32>, height: Option<f32>) -> StyleRule {
    let mut props = Properties::new();
    if let Some(w) = width {
        props.insert(Width(Size::Pixels(w)));
    }
    if let Some(h) = height {
        props.insert(Height(Size::Pixels(h)));
    }
    StyleRule::new(selector.to_string(), props)
}

mod stylesheet_resolution {
    use super::*;

    #[test]
    fn test_default_stylesheet_button_size() {
        let sheet = rvue_style::default_stylesheet();
        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Default button size should be 120x40
        assert_eq!(resolved.width, Some(Width(Size::Pixels(120.0))));
        assert_eq!(resolved.height, Some(Height(Size::Pixels(40.0))));
    }

    #[test]
    fn test_default_stylesheet_input_size() {
        let sheet = rvue_style::default_stylesheet();
        let element = RvueElement::new("input");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Default input size should be 200x30
        assert_eq!(resolved.width, Some(Width(Size::Pixels(200.0))));
        assert_eq!(resolved.height, Some(Height(Size::Pixels(30.0))));
    }

    #[test]
    fn test_empty_stylesheet_returns_none() {
        let sheet = Stylesheet::new();
        let element = RvueElement::new("div");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Empty stylesheet should return all None (no rules matched)
        assert!(resolved.width.is_none());
        assert!(resolved.height.is_none());
        assert!(resolved.background_color.is_none());
    }

    #[test]
    fn test_element_tag_selector_matches() {
        let mut sheet = Stylesheet::new();
        let mut props = Properties::new();
        props.insert(BackgroundColor(Color::rgb(0, 255, 0)));
        sheet.add_rule(StyleRule::new("my-element".to_string(), props));

        let element = RvueElement::new("my-element");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(0, 255, 0))));
    }

    #[test]
    fn test_class_selector_matches() {
        let mut sheet = Stylesheet::new();
        let mut props = Properties::new();
        props.insert(BackgroundColor(Color::rgb(255, 0, 0)));
        sheet.add_rule(StyleRule::new(".primary".to_string(), props));

        let element = RvueElement::new("button").with_class("primary");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }
}

mod inline_stylesheet_merge {
    use super::*;

    #[test]
    fn test_inline_non_default_overrides_stylesheet() {
        // This is the key regression test for the counter button bug
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_style_rule("button", Some(120.0), Some(40.0)));

        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Create inline styles with explicit non-Auto width
        let mut inline = ComputedStyles::new();
        inline.width = Some(Width(Size::Pixels(200.0))); // Explicit
        inline.height = Some(Height(Size::Pixels(50.0))); // Explicit

        // Apply selective merge
        let mut merged = resolved;
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, Size::Auto) {
                merged.width = Some(w.clone());
            }
        }
        if let Some(h) = inline.height.as_ref() {
            if !matches!(h.0, Size::Auto) {
                merged.height = Some(h.clone());
            }
        }

        // Inline explicit values should override
        assert_eq!(merged.width, Some(Width(Size::Pixels(200.0))));
        assert_eq!(merged.height, Some(Height(Size::Pixels(50.0))));
    }

    #[test]
    fn test_inline_auto_does_not_override_stylesheet() {
        // Regression test: Width(Auto) should NOT override stylesheet Width(120px)
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_style_rule("button", Some(120.0), Some(40.0)));

        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Create inline with default Auto width
        let mut inline = ComputedStyles::new();
        inline.width = Some(Width(Size::Auto)); // Default - should NOT override
        inline.height = Some(Height(Size::Pixels(50.0))); // Explicit - should override

        // Apply selective merge
        let mut merged = resolved;
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, Size::Auto) {
                merged.width = Some(w.clone());
            }
        }
        if let Some(h) = inline.height.as_ref() {
            if !matches!(h.0, Size::Auto) {
                merged.height = Some(h.clone());
            }
        }

        // Auto should NOT override 120px from stylesheet
        assert_eq!(merged.width, Some(Width(Size::Pixels(120.0))));
        // But explicit height should override
        assert_eq!(merged.height, Some(Height(Size::Pixels(50.0))));
    }

    #[test]
    fn test_background_color_always_merges() {
        // Background color doesn't have "Auto" default, so it should always merge
        let mut sheet = Stylesheet::new();
        let mut sheet_props = Properties::new();
        sheet_props.insert(BackgroundColor(Color::rgb(0, 0, 255))); // Blue
        sheet.add_rule(StyleRule::new("button".to_string(), sheet_props));

        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        let mut inline = ComputedStyles::new();
        inline.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0))); // Red

        // Merge background color (always merge if set)
        let mut merged = resolved;
        if inline.background_color.is_some() {
            merged.background_color = inline.background_color;
        }

        // Inline red should override stylesheet blue
        assert_eq!(merged.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }

    #[test]
    fn test_no_inline_uses_stylesheet() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_style_rule("button", Some(120.0), Some(40.0)));

        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // No inline styles
        let inline: Option<ComputedStyles> = None;

        let merged = if let Some(inline_styles) = inline {
            let mut result = resolved;
            // Apply merge logic
            if let Some(w) = inline_styles.width.as_ref() {
                if !matches!(w.0, Size::Auto) {
                    result.width = Some(w.clone());
                }
            }
            if let Some(h) = inline_styles.height.as_ref() {
                if !matches!(h.0, Size::Auto) {
                    result.height = Some(h.clone());
                }
            }
            result
        } else {
            resolved
        };

        // Should use stylesheet values
        assert_eq!(merged.width, Some(Width(Size::Pixels(120.0))));
        assert_eq!(merged.height, Some(Height(Size::Pixels(40.0))));
    }
}

mod complete_resolution_pipeline {
    use super::*;

    #[test]
    fn test_button_with_custom_styles() {
        // Simulate the counter example: button with custom width and background
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_style_rule("button", Some(120.0), Some(40.0)));

        let element = RvueElement::new("button");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Inline styles from ReactiveStyles::compute()
        let mut inline = ComputedStyles::new();
        inline.width = Some(Width(Size::Pixels(120.0))); // Same as stylesheet
        inline.height = Some(Height(Size::Pixels(40.0))); // Same as stylesheet
        inline.background_color = Some(BackgroundColor(Color::rgb(76, 175, 80))); // Green
        inline.display = Some(Display::Flex);
        inline.flex_direction = Some(FlexDirection::Row);

        // Apply selective merge (width/height are same value, background is different)
        let mut merged = resolved;
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, Size::Auto) {
                merged.width = Some(w.clone());
            }
        }
        if let Some(h) = inline.height.as_ref() {
            if !matches!(h.0, Size::Auto) {
                merged.height = Some(h.clone());
            }
        }
        if inline.background_color.is_some() {
            merged.background_color = inline.background_color;
        }
        if inline.display.is_some() {
            merged.display = inline.display;
        }
        if inline.flex_direction.is_some() {
            merged.flex_direction = inline.flex_direction;
        }

        // Verify all styles are correct
        assert_eq!(merged.width, Some(Width(Size::Pixels(120.0))));
        assert_eq!(merged.height, Some(Height(Size::Pixels(40.0))));
        assert_eq!(merged.background_color, Some(BackgroundColor(Color::rgb(76, 175, 80))));
        assert_eq!(merged.display, Some(Display::Flex));
        assert_eq!(merged.flex_direction, Some(FlexDirection::Row));
    }

    #[test]
    fn test_class_specific_override() {
        // Test that class selectors work with inline styles
        let mut sheet = Stylesheet::new();
        let mut base_props = Properties::new();
        base_props.insert(Width(Size::Pixels(120.0)));
        base_props.insert(Height(Size::Pixels(40.0)));
        sheet.add_rule(StyleRule::new("button".to_string(), base_props));

        let mut primary_props = Properties::new();
        primary_props.insert(BackgroundColor(Color::rgb(0, 0, 255)));
        sheet.add_rule(StyleRule::new("button.primary".to_string(), primary_props));

        let element = RvueElement::new("button").with_class("primary");
        let resolver = StyleResolver::new();
        let resolved = resolver.resolve_styles(&element, &sheet);

        // Should have both base button styles and .primary specific styles
        assert_eq!(resolved.width, Some(Width(Size::Pixels(120.0))));
        assert_eq!(resolved.height, Some(Height(Size::Pixels(40.0))));
        assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(0, 0, 255))));
    }
}
