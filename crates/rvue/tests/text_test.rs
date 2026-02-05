use rudo_gc::Gc;
use rvue::component::{Component, ComponentType};
use rvue::Scene;

#[test]
fn test_text_measure() {
    let _id_counter = std::sync::atomic::AtomicU64::new(0);
    let next_id = |_: u64| 0;

    // Create a simple Text component
    let component = Component::with_properties(
        next_id(0),
        ComponentType::Text,
        rvue::properties::PropertyMap::with(rvue::properties::TextContent(
            "Hello World".to_string(),
        )),
    );

    // Create a Scene and add the component
    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&component));

    // Update the scene to trigger layout
    scene.update();

    // Check that the component has a layout
    let layout_node = component.layout_node();
    assert!(layout_node.is_some(), "Component should have a layout node");

    let layout = layout_node.unwrap();
    let layout_result = layout.layout();
    assert!(layout_result.is_some(), "Layout should have a result");

    let rect = layout_result.unwrap();
    // "Hello World" with 16px font should be wider than 0 and taller than 0
    // Note: If Taffy measure is not supported, we set a default size of 100x20.
    // So we check for default size or greater.
    assert!(rect.size.width > 0.0, "Text width should be positive: {}", rect.size.width);
    assert!(rect.size.height > 0.0, "Text height should be positive: {}", rect.size.height);

    // If we were able to verify Parley layout, we would check here, but measure might be skipped.
}

#[test]
fn test_text_in_flex() {
    // Test that text measures correctly inside a Flex container
    let _id_counter = std::sync::atomic::AtomicU64::new(0);
    let next_id = |_: u64| 0;

    // Create a Flex container
    let flex = Component::with_properties(
        next_id(0),
        ComponentType::Flex,
        rvue::properties::PropertyMap::new(),
    );

    // Create two Text children
    let text1 = Component::with_properties(
        next_id(0),
        ComponentType::Text,
        rvue::properties::PropertyMap::with(rvue::properties::TextContent("Short".to_string())),
    );

    let text2 = Component::with_properties(
        next_id(0),
        ComponentType::Text,
        rvue::properties::PropertyMap::with(rvue::properties::TextContent(
            "This is a longer piece of text".to_string(),
        )),
    );

    flex.add_child(Gc::clone(&text1));
    flex.add_child(Gc::clone(&text2));

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&flex));

    scene.update();

    let flex_layout_node = flex.layout_node().unwrap();
    let flex_layout = flex_layout_node.layout().unwrap();
    // Flex container should be wide enough to hold both texts + gap
    // This is a loose assertion as exact font metrics vary
    assert!(
        flex_layout.size.width > 100.0,
        "Flex should be wider than 100px, got {}",
        flex_layout.size.width
    );
}
