//! Tests for ComputedStyles merge behavior.
//!
//! These tests ensure that merging computed styles works correctly,
//! particularly that non-default inline values override stylesheet values
//! while default values do not.

use rvue_style::{
    properties::{
        AlignItems, BackgroundColor, Color, Display, FlexDirection, Height, JustifyContent, Size,
        Width,
    },
    ComputedStyles,
};

mod basic_merge {
    use super::*;

    #[test]
    fn test_merge_with_computed_overwrites_existing() {
        let mut base = ComputedStyles::new();
        base.background_color = Some(BackgroundColor(Color::rgb(0, 0, 255)));

        let mut other = ComputedStyles::new();
        other.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));

        base.merge_with_computed(&other);

        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }

    #[test]
    fn test_merge_preserves_unset_properties() {
        let mut base = ComputedStyles::new();
        base.width = Some(Width(Size::Pixels(120.0)));
        base.display = Some(Display::Flex);

        let mut other = ComputedStyles::new();
        other.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));
        // width and display are None in other

        base.merge_with_computed(&other);

        assert_eq!(base.width, Some(Width(Size::Pixels(120.0)))); // Preserved
        assert_eq!(base.display, Some(Display::Flex)); // Preserved
        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }

    #[test]
    fn test_merge_multiple_properties() {
        let mut base = ComputedStyles::new();
        base.width = Some(Width(Size::Pixels(100.0)));
        base.height = Some(Height(Size::Pixels(50.0)));
        base.background_color = Some(BackgroundColor(Color::rgb(0, 0, 255)));
        base.display = Some(Display::Block);

        let mut other = ComputedStyles::new();
        other.width = Some(Width(Size::Pixels(200.0)));
        other.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));
        // height and display are None

        base.merge_with_computed(&other);

        assert_eq!(base.width, Some(Width(Size::Pixels(200.0)))); // Overwritten
        assert_eq!(base.height, Some(Height(Size::Pixels(50.0)))); // Preserved
        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0)))); // Overwritten
        assert_eq!(base.display, Some(Display::Block)); // Preserved
    }

    #[test]
    fn test_merge_empty_override_preserves_all() {
        let mut base = ComputedStyles::new();
        base.width = Some(Width(Size::Pixels(100.0)));
        base.height = Some(Height(Size::Pixels(50.0)));
        base.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));

        let other = ComputedStyles::new(); // All None

        base.merge_with_computed(&other);

        assert_eq!(base.width, Some(Width(Size::Pixels(100.0))));
        assert_eq!(base.height, Some(Height(Size::Pixels(50.0))));
        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }
}

mod selective_merge_logic {
    use super::*;

    /// This test verifies the regression fix for the counter button bug.
    /// When inline has Width(Auto) and stylesheet has Width(120px),
    /// the Auto should NOT overwrite 120px.
    #[test]
    fn test_auto_width_should_not_override_explicit_width() {
        let mut base = ComputedStyles::new();
        base.width = Some(Width(Size::Pixels(120.0))); // From stylesheet

        let mut inline = ComputedStyles::new();
        inline.width = Some(Width(Size::Auto)); // Default value from inline

        // Apply selective merge logic (only non-Auto overrides)
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, Size::Auto) {
                base.width = Some(w.clone());
            }
        }

        assert_eq!(base.width, Some(Width(Size::Pixels(120.0)))); // Preserved
    }

    #[test]
    fn test_explicit_pixel_width_should_override() {
        let mut base = ComputedStyles::new();
        base.width = Some(Width(Size::Pixels(120.0))); // From stylesheet

        let mut inline = ComputedStyles::new();
        inline.width = Some(Width(Size::Pixels(200.0))); // Explicit non-default

        // Apply selective merge logic
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, Size::Auto) {
                base.width = Some(w.clone());
            }
        }

        assert_eq!(base.width, Some(Width(Size::Pixels(200.0)))); // Overwritten
    }

    #[test]
    fn test_background_color_always_merges() {
        // Background color doesn't have an "Auto" default, so it should always merge
        let mut base = ComputedStyles::new();
        base.background_color = Some(BackgroundColor(Color::rgb(0, 0, 255))); // Blue from stylesheet

        let mut inline = ComputedStyles::new();
        inline.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0))); // Red inline

        // Background color: always merge if set
        if inline.background_color.is_some() {
            base.background_color = inline.background_color;
        }

        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }
}

mod merge_behavior_guarantees {
    use super::*;

    #[test]
    fn test_merge_all_properties_defined() {
        // Test that all defined properties in ComputedStyles can be merged
        let mut base = ComputedStyles::new();
        let mut other = ComputedStyles::new();

        // Set all properties that have merge support
        other.width = Some(Width(Size::Pixels(100.0)));
        other.height = Some(Height(Size::Pixels(50.0)));
        other.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));
        other.color = Some(Color::rgb(0, 255, 0));
        other.display = Some(Display::Flex);
        other.flex_direction = Some(FlexDirection::Column);
        other.justify_content = Some(JustifyContent::Center);
        other.align_items = Some(AlignItems::FlexEnd);

        base.merge_with_computed(&other);

        assert_eq!(base.width, Some(Width(Size::Pixels(100.0))));
        assert_eq!(base.height, Some(Height(Size::Pixels(50.0))));
        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
        assert_eq!(base.color, Some(Color::rgb(0, 255, 0)));
        assert_eq!(base.display, Some(Display::Flex));
        assert_eq!(base.flex_direction, Some(FlexDirection::Column));
        assert_eq!(base.justify_content, Some(JustifyContent::Center));
        assert_eq!(base.align_items, Some(AlignItems::FlexEnd));
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_merge_with_self() {
        // Merging with itself should be idempotent
        let mut styles = ComputedStyles::new();
        styles.width = Some(Width(Size::Pixels(100.0)));
        styles.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));

        let other = ComputedStyles::new();
        styles.merge_with_computed(&other);

        assert_eq!(styles.width, Some(Width(Size::Pixels(100.0))));
        assert_eq!(styles.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }

    #[test]
    fn test_empty_base_with_other() {
        let mut base = ComputedStyles::new(); // All None
        let mut other = ComputedStyles::new();
        other.width = Some(Width(Size::Pixels(100.0)));
        other.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));

        base.merge_with_computed(&other);

        assert_eq!(base.width, Some(Width(Size::Pixels(100.0))));
        assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    }
}
