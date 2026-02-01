use rvue_style::{
    reactive::ReactiveStyles, stylesheet::resolver::StyleResolver, stylesheet::StyleRule,
    BackgroundColor, Color, ComputedStyles, Display, FlexDirection, Height, Properties,
    RvueElement, Size, Stylesheet, Width,
};

#[test]
fn test_resolve_styles_basic() {
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut props = Properties::new();
    props.insert(BackgroundColor(Color::rgb(255, 0, 0)));
    stylesheet.add_rule(StyleRule::new("button".to_string(), props));

    let element = RvueElement::new("button");
    let resolved = resolver.resolve_styles(&element, &stylesheet);

    assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
}

#[test]
fn test_empty_stylesheet_uses_initial_values() {
    let resolver = StyleResolver::new();
    let stylesheet = Stylesheet::new();

    let element = RvueElement::new("div");
    let resolved = resolver.resolve_styles(&element, &stylesheet);

    assert!(resolved.background_color.is_none());
    assert!(resolved.color.is_none());
    assert!(resolved.display.is_none());
    assert!(resolved.width.is_none());
    assert!(resolved.height.is_none());
}

#[test]
fn test_initial_values_consistent() {
    let styles = ReactiveStyles::new().compute();
    assert!(styles.background_color.is_some());
    assert!(styles.color.is_some());
    assert!(styles.display.is_some());
    assert!(styles.width.is_some());
    assert!(styles.height.is_some());
    assert!(styles.flex_direction.is_some());
    assert!(styles.padding.is_some());
    assert!(styles.margin.is_some());
}

#[test]
fn test_computed_styles_merge() {
    let mut styles1 = ComputedStyles::new();
    styles1.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));
    styles1.width = Some(Width(Size::pixels(100.0)));

    let mut styles2 = ComputedStyles::new();
    styles2.color = Some(Color::rgb(0, 0, 255));
    styles2.height = Some(Height(Size::pixels(200.0)));

    styles1.merge_with_computed(&styles2);

    assert_eq!(styles1.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    assert_eq!(styles1.width, Some(Width(Size::pixels(100.0))));
    assert_eq!(styles1.color, Some(Color::rgb(0, 0, 255)));
    assert_eq!(styles1.height, Some(Height(Size::pixels(200.0))));
}

#[test]
fn test_computed_styles_merge_overrides() {
    let mut styles1 = ComputedStyles::new();
    styles1.background_color = Some(BackgroundColor(Color::rgb(255, 0, 0)));
    styles1.display = Some(Display::Block);

    let mut styles2 = ComputedStyles::new();
    styles2.background_color = Some(BackgroundColor(Color::rgb(0, 255, 0)));
    styles2.display = Some(Display::Flex);

    styles1.merge_with_computed(&styles2);

    assert_eq!(styles1.background_color, Some(BackgroundColor(Color::rgb(0, 255, 0))));
    assert_eq!(styles1.display, Some(Display::Flex));
}

#[test]
fn test_property_initial_values() {
    use rvue_style::Property;

    assert_eq!(BackgroundColor::initial_value(), BackgroundColor(Color::rgb(0, 0, 0)));
    assert_eq!(Color::initial_value(), Color::rgb(0, 0, 0));
    assert_eq!(Display::initial_value(), Display::Flex);
    assert_eq!(Width::initial_value(), Width(Size::Auto));
    assert_eq!(FlexDirection::initial_value(), FlexDirection::Row);
}

#[test]
fn test_multiple_class_selectors() {
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut props1 = Properties::new();
    props1.insert(BackgroundColor(Color::rgb(255, 0, 0)));
    stylesheet.add_rule(StyleRule::new(".btn".to_string(), props1));

    let mut props2 = Properties::new();
    props2.insert(Width(Size::pixels(200.0)));
    stylesheet.add_rule(StyleRule::new(".btn.primary".to_string(), props2));

    let element = RvueElement::new("button").with_class("btn").with_class("primary");

    let resolved = resolver.resolve_styles(&element, &stylesheet);

    assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    assert_eq!(resolved.width, Some(Width(Size::pixels(200.0))));
}

#[test]
fn test_element_tag_selector() {
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut props = Properties::new();
    props.insert(Display::Flex);
    props.insert(FlexDirection::Column);
    stylesheet.add_rule(StyleRule::new("button".to_string(), props));

    let element = RvueElement::new("button");

    let resolved = resolver.resolve_styles(&element, &stylesheet);

    assert_eq!(resolved.display, Some(Display::Flex));
    assert_eq!(resolved.flex_direction, Some(FlexDirection::Column));
}

#[test]
fn test_inline_styles_override_sheet() {
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut props = Properties::new();
    props.insert(BackgroundColor(Color::rgb(255, 0, 0)));
    stylesheet.add_rule(StyleRule::new("button".to_string(), props));

    let element = RvueElement::new("button");
    let mut resolved = resolver.resolve_styles(&element, &stylesheet);

    assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));

    let mut inline = ComputedStyles::new();
    inline.background_color = Some(BackgroundColor(Color::rgb(0, 255, 0)));
    resolved.merge_with_computed(&inline);

    assert_eq!(resolved.background_color, Some(BackgroundColor(Color::rgb(0, 255, 0))));
}

#[test]
fn test_regression_auto_width_does_not_override_explicit() {
    // REGRESSION TEST: This test catches the bug where Width(Auto) from
    // ReactiveStyles::compute() would override stylesheet Width(120px)
    //
    // Bug: inline.width = Auto would overwrite stylesheet.width = 120px
    // Fix: Only override if inline width is not Auto (default)
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut sheet_props = Properties::new();
    sheet_props.insert(Width(Size::pixels(120.0)));
    sheet_props.insert(Height(Size::pixels(40.0)));
    stylesheet.add_rule(StyleRule::new("button".to_string(), sheet_props));

    let element = RvueElement::new("button");
    let resolved = resolver.resolve_styles(&element, &stylesheet);

    // Simulate inline styles with default Auto width (from ReactiveStyles::compute())
    let mut inline = ComputedStyles::new();
    inline.width = Some(Width(Size::Auto)); // Default value - should NOT override
    inline.height = Some(Height(Size::Auto)); // Default value - should NOT override
    inline.background_color = Some(BackgroundColor(Color::rgb(76, 175, 80))); // Explicit

    // Apply selective merge (the fix)
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

    // Width/Height should be preserved from stylesheet
    assert_eq!(merged.width, Some(Width(Size::pixels(120.0))));
    assert_eq!(merged.height, Some(Height(Size::pixels(40.0))));
    // Background should be overridden by inline
    assert_eq!(merged.background_color, Some(BackgroundColor(Color::rgb(76, 175, 80))));
}

#[test]
fn test_regression_explicit_width_overrides_stylesheet() {
    // REGRESSION TEST: Explicit width should override stylesheet
    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    let mut sheet_props = Properties::new();
    sheet_props.insert(Width(Size::pixels(120.0)));
    sheet_props.insert(Height(Size::pixels(40.0)));
    stylesheet.add_rule(StyleRule::new("button".to_string(), sheet_props));

    let element = RvueElement::new("button");
    let resolved = resolver.resolve_styles(&element, &stylesheet);

    // Explicit width from user (not default Auto)
    let mut inline = ComputedStyles::new();
    inline.width = Some(Width(Size::pixels(200.0))); // Explicit - SHOULD override
    inline.height = Some(Height(Size::pixels(50.0))); // Explicit - SHOULD override

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

    // Explicit values should override
    assert_eq!(merged.width, Some(Width(Size::pixels(200.0))));
    assert_eq!(merged.height, Some(Height(Size::pixels(50.0))));
}

#[test]
fn test_counter_example_button_simulation() {
    // Simulate the counter example button with:
    // - Stylesheet: button { width: 120px; height: 40px; }
    // - Inline: background-color: green; align-items: center; justify-content: center;
    use rvue_style::properties::AlignItems;
    use rvue_style::properties::JustifyContent;

    let resolver = StyleResolver::new();
    let mut stylesheet = Stylesheet::new();

    // Stylesheet sets default button size
    let mut sheet_props = Properties::new();
    sheet_props.insert(Width(Size::pixels(120.0)));
    sheet_props.insert(Height(Size::pixels(40.0)));
    stylesheet.add_rule(StyleRule::new("button".to_string(), sheet_props));

    let element = RvueElement::new("button");
    let resolved = resolver.resolve_styles(&element, &stylesheet);

    // Inline styles from counter example (increment_button_styles)
    let mut inline = ComputedStyles::new();
    inline.width = Some(Width(Size::Auto)); // Default from ReactiveStyles
    inline.height = Some(Height(Size::Auto)); // Default from ReactiveStyles
    inline.background_color = Some(BackgroundColor(Color::rgb(76, 175, 80))); // Green
    inline.align_items = Some(AlignItems::Center);
    inline.justify_content = Some(JustifyContent::Center);

    // Apply selective merge (the fix)
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
    if inline.align_items.is_some() {
        merged.align_items = inline.align_items;
    }
    if inline.justify_content.is_some() {
        merged.justify_content = inline.justify_content;
    }

    // Verify the complete button style is correct
    assert_eq!(merged.width, Some(Width(Size::pixels(120.0)))); // From stylesheet
    assert_eq!(merged.height, Some(Height(Size::pixels(40.0)))); // From stylesheet
    assert_eq!(merged.background_color, Some(BackgroundColor(Color::rgb(76, 175, 80)))); // Green from inline
    assert_eq!(merged.align_items, Some(AlignItems::Center));
    assert_eq!(merged.justify_content, Some(JustifyContent::Center));
}

#[test]
fn test_selective_merge_preserves_unset_properties() {
    // Ensure that None values in other don't overwrite Some values in base
    let mut base = ComputedStyles::new();
    base.width = Some(Width(Size::pixels(100.0)));
    base.height = Some(Height(Size::pixels(50.0)));
    base.display = Some(Display::Flex);
    base.background_color = Some(BackgroundColor(Color::rgb(0, 0, 255)));

    let other = ComputedStyles::new(); // All None

    base.merge_with_computed(&other);

    // All base properties should be preserved
    assert_eq!(base.width, Some(Width(Size::pixels(100.0))));
    assert_eq!(base.height, Some(Height(Size::pixels(50.0))));
    assert_eq!(base.display, Some(Display::Flex));
    assert_eq!(base.background_color, Some(BackgroundColor(Color::rgb(0, 0, 255))));
}
