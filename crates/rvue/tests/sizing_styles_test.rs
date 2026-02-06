//! Tests for width/height styling system

use rudo_gc::GcCell;
use rvue::layout::node::LayoutNode;
use rvue::style::{resolve_styles_for_component, Stylesheet};
use rvue::text::TextContext;
use rvue::{Component, ComponentType};
use rvue_style::{BackgroundColor, Color, ComputedStyles, Height, ReactiveStyles, Size, Width};
use taffy::TaffyTree;

#[test]
fn test_button_with_custom_width_height_via_styles() {
    let styles = ReactiveStyles::new()
        .set_width(Width(Size::Pixels(200.0)))
        .set_height(Height(Size::Pixels(60.0)));
    let computed = styles.compute();

    assert!(computed.width.is_some(), "width should be set");
    assert!(computed.height.is_some(), "height should be set");
    assert_eq!(computed.width.as_ref().unwrap().0, Size::Pixels(200.0));
    assert_eq!(computed.height.as_ref().unwrap().0, Size::Pixels(60.0));
}

#[test]
fn test_flex_with_custom_width_height_via_styles() {
    let styles = ReactiveStyles::new()
        .set_width(Width(Size::Pixels(300.0)))
        .set_height(Height(Size::Pixels(150.0)));
    let computed = styles.compute();

    assert!(computed.width.is_some(), "width should be set");
    assert!(computed.height.is_some(), "height should be set");
    assert_eq!(computed.width.as_ref().unwrap().0, Size::Pixels(300.0));
    assert_eq!(computed.height.as_ref().unwrap().0, Size::Pixels(150.0));
}

#[test]
fn test_computed_styles_merge_width_height() {
    let mut styles1 = ComputedStyles::new();
    styles1.width = Some(Width(Size::Pixels(100.0)));

    let mut styles2 = ComputedStyles::new();
    styles2.height = Some(Height(Size::Pixels(50.0)));

    styles1.merge_with_computed(&styles2);

    assert!(styles1.width.is_some());
    assert!(styles1.height.is_some());
}

#[test]
fn test_size_variants() {
    let auto = Size::Auto;
    let pixels = Size::Pixels(100.0);
    let percent = Size::Percent(50.0);

    assert_eq!(auto.to_string(), "auto");
    assert_eq!(pixels.to_string(), "100px");
    assert_eq!(percent.to_string(), "50%");
}

#[test]
fn test_button_with_default_stylesheet_size() {
    let mut taffy = TaffyTree::new();
    let stylesheet = Stylesheet::with_defaults();

    let component =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    let mut layout_node = LayoutNode::build_in_tree(
        &mut taffy,
        &component,
        &[],
        &mut TextContext::new(),
        Some(&stylesheet),
    );

    layout_node.calculate_layout(&mut taffy).unwrap();

    assert!(layout_node.is_dirty() || layout_node.layout().is_some());
    if let Some(layout) = layout_node.layout() {
        assert!(layout.size.width > 0.0, "button should have width from stylesheet");
        assert!(layout.size.height > 0.0, "button should have height from stylesheet");
    }
}

#[test]
fn test_direct_stylesheet_resolution() {
    // Create stylesheet directly using rvue_style::Stylesheet (like unit test)
    let mut direct_stylesheet = rvue_style::Stylesheet::new();
    direct_stylesheet.add_rule(rvue_style::StyleRule::new("button.primary".to_string(), {
        let mut props = rvue_style::Properties::new();
        props.insert(BackgroundColor(Color::rgb(0, 123, 255)));
        props
    }));

    // Test using rvue_style directly (like the unit test does)
    let resolver = rvue_style::StyleResolver::new();
    let element = rvue_style::RvueElement::new("button").with_class("primary");
    let resolved = resolver.resolve_styles(&element, &direct_stylesheet);

    assert!(
        resolved.background_color.is_some(),
        "button with class='primary' should have background_color from stylesheet"
    );
}

#[test]
fn test_button_without_class_gets_default_size() {
    let stylesheet = Stylesheet::with_defaults();

    let component =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    let resolved = resolve_styles_for_component(&component, &stylesheet);

    eprintln!("Resolved width: {:?}", resolved.width);
    eprintln!("Resolved height: {:?}", resolved.height);

    assert!(resolved.width.is_some(), "button should have default width");
    assert!(resolved.height.is_some(), "button should have default height");
}
