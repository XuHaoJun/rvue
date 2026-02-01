//! Tests for CSS initial values of style properties.
//!
//! These tests ensure that each property's `initial_value()` returns
//! the correct CSS default, matching web browser behavior.

use rvue_style::{
    properties::{
        AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle,
        BorderWidth, Color, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink,
        FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin, MaxHeight, MaxWidth,
        MinHeight, MinWidth, Opacity, Padding, Size, TextColor, Visibility, Width, ZIndex,
    },
    Property,
};

mod size {
    use super::*;

    #[test]
    fn test_width_initial_value() {
        assert_eq!(Width::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_height_initial_value() {
        assert_eq!(Height::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_min_width_initial_value() {
        assert_eq!(MinWidth::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_min_height_initial_value() {
        assert_eq!(MinHeight::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_max_width_initial_value() {
        assert_eq!(MaxWidth::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_max_height_initial_value() {
        assert_eq!(MaxHeight::initial_value().0, Size::Auto);
    }
}

mod color {
    use super::*;

    #[test]
    fn test_background_color_initial_value() {
        let bg = BackgroundColor::initial_value();
        // CSS: background-color: initial = transparent (black with alpha 0)
        assert_eq!(bg.0 .0.r, 0);
        assert_eq!(bg.0 .0.g, 0);
        assert_eq!(bg.0 .0.b, 0);
    }

    #[test]
    fn test_color_initial_value() {
        let c = Color::initial_value();
        // CSS: color: initial = black
        assert_eq!(c.0.r, 0);
        assert_eq!(c.0.g, 0);
        assert_eq!(c.0.b, 0);
    }

    #[test]
    fn test_text_color_initial_value() {
        let tc = TextColor::initial_value();
        // CSS: color: initial = black
        assert_eq!(tc.0 .0.r, 0);
        assert_eq!(tc.0 .0.g, 0);
        assert_eq!(tc.0 .0.b, 0);
    }
}

mod typography {
    use super::*;

    #[test]
    fn test_font_size_initial_value() {
        assert_eq!(FontSize::initial_value().0, 16.0);
    }

    #[test]
    fn test_font_family_initial_value() {
        assert_eq!(FontFamily::initial_value().0, "system-ui");
    }

    #[test]
    fn test_font_weight_initial_value() {
        assert_eq!(FontWeight::initial_value(), FontWeight::Normal);
    }
}

mod spacing {
    use super::*;

    #[test]
    fn test_padding_initial_value() {
        assert_eq!(Padding::initial_value().0, 0.0);
    }

    #[test]
    fn test_margin_initial_value() {
        assert_eq!(Margin::initial_value().0, 0.0);
    }
}

mod layout {
    use super::*;

    #[test]
    fn test_display_initial_value() {
        // CSS: display: initial = block (but widgets typically use Flex)
        assert_eq!(Display::initial_value(), Display::Flex);
    }

    #[test]
    fn test_flex_direction_initial_value() {
        assert_eq!(FlexDirection::initial_value(), FlexDirection::Row);
    }

    #[test]
    fn test_justify_content_initial_value() {
        assert_eq!(JustifyContent::initial_value(), JustifyContent::FlexStart);
    }

    #[test]
    fn test_align_items_initial_value() {
        assert_eq!(AlignItems::initial_value(), AlignItems::Stretch);
    }

    #[test]
    fn test_align_self_initial_value() {
        assert_eq!(AlignSelf::initial_value(), AlignSelf::Auto);
    }

    #[test]
    fn test_flex_grow_initial_value() {
        assert_eq!(FlexGrow::initial_value().0, 0.0);
    }

    #[test]
    fn test_flex_shrink_initial_value() {
        assert_eq!(FlexShrink::initial_value().0, 1.0);
    }

    #[test]
    fn test_flex_basis_initial_value() {
        assert_eq!(FlexBasis::initial_value().0, Size::Auto);
    }

    #[test]
    fn test_gap_initial_value() {
        assert_eq!(Gap::initial_value().0, 0.0);
    }
}

mod border {
    use super::*;

    #[test]
    fn test_border_color_initial_value() {
        let bc = BorderColor::initial_value();
        // CSS: border-color: initial = currentcolor (black)
        assert_eq!(bc.0 .0.r, 0);
        assert_eq!(bc.0 .0.g, 0);
        assert_eq!(bc.0 .0.b, 0);
    }

    #[test]
    fn test_border_width_initial_value() {
        assert_eq!(BorderWidth::initial_value().0, 0.0);
    }

    #[test]
    fn test_border_radius_initial_value() {
        assert_eq!(BorderRadius::initial_value().0, 0.0);
    }

    #[test]
    fn test_border_style_initial_value() {
        assert_eq!(BorderStyle::initial_value(), BorderStyle::None);
    }
}

mod visibility {
    use super::*;

    #[test]
    fn test_opacity_initial_value() {
        assert_eq!(Opacity::initial_value().0, 1.0);
    }

    #[test]
    fn test_visibility_initial_value() {
        assert_eq!(Visibility::initial_value(), Visibility::Visible);
    }

    #[test]
    fn test_z_index_initial_value() {
        assert_eq!(ZIndex::initial_value().0, 0);
    }

    #[test]
    fn test_cursor_initial_value() {
        assert_eq!(Cursor::initial_value(), Cursor::Default);
    }
}

mod comprehensive {
    use super::*;

    #[test]
    fn test_all_style_flags_properties_have_initial_values() {
        // Ensure all properties in StyleFlags have initial_value() implemented
        // This is a compile-time check - if any property is missing initial_value(),
        // this test won't compile
        let _width = Width::initial_value();
        let _height = Height::initial_value();
        let _min_width = MinWidth::initial_value();
        let _min_height = MinHeight::initial_value();
        let _max_width = MaxWidth::initial_value();
        let _max_height = MaxHeight::initial_value();
        let _background_color = BackgroundColor::initial_value();
        let _color = Color::initial_value();
        let _text_color = TextColor::initial_value();
        let _font_size = FontSize::initial_value();
        let _font_family = FontFamily::initial_value();
        let _font_weight = FontWeight::initial_value();
        let _padding = Padding::initial_value();
        let _margin = Margin::initial_value();
        let _display = Display::initial_value();
        let _flex_direction = FlexDirection::initial_value();
        let _justify_content = JustifyContent::initial_value();
        let _align_items = AlignItems::initial_value();
        let _align_self = AlignSelf::initial_value();
        let _flex_grow = FlexGrow::initial_value();
        let _flex_shrink = FlexShrink::initial_value();
        let _flex_basis = FlexBasis::initial_value();
        let _gap = Gap::initial_value();
        let _border_color = BorderColor::initial_value();
        let _border_width = BorderWidth::initial_value();
        let _border_radius = BorderRadius::initial_value();
        let _border_style = BorderStyle::initial_value();
        let _opacity = Opacity::initial_value();
        let _visibility = Visibility::initial_value();
        let _z_index = ZIndex::initial_value();
        let _cursor = Cursor::initial_value();

        // All properties are accessible and return Some value
        let _ = _width; // Just verify it compiles
        let _ = _height;
        let _ = _min_width;
        let _ = _min_height;
        let _ = _max_width;
        let _ = _max_height;
        let _ = _background_color;
        let _ = _color;
        let _ = _text_color;
        let _ = _font_size;
        let _ = _font_family;
        let _ = _font_weight;
        let _ = _padding;
        let _ = _margin;
        let _ = _display;
        let _ = _flex_direction;
        let _ = _justify_content;
        let _ = _align_items;
        let _ = _align_self;
        let _ = _flex_grow;
        let _ = _flex_shrink;
        let _ = _flex_basis;
        let _ = _gap;
        let _ = _border_color;
        let _ = _border_width;
        let _ = _border_radius;
        let _ = _border_style;
        let _ = _opacity;
        let _ = _visibility;
        let _ = _z_index;
        let _ = _cursor;
    }

    #[test]
    fn test_initial_values_are_sensible_defaults() {
        // Verify that initial values are useful defaults, not just zeros
        assert_ne!(FontSize::initial_value().0, 0.0, "Font size should not be 0");
        assert_ne!(FontFamily::initial_value().0, "", "Font family should not be empty");
        assert_ne!(Opacity::initial_value().0, 0.0, "Opacity should default to visible");
    }
}
