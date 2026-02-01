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
