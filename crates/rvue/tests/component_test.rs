//! Unit tests for Component lifecycle

use rudo_gc::Gc;
use rvue::render::FlexScrollState;
use rvue::{Component, ComponentLifecycle, ComponentType};

#[test]
fn test_component_creation() {
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    assert_eq!(component.id, 1);
    assert_eq!(component.component_type, ComponentType::Text);
    assert!(component.children.read().is_empty());
    assert!(component.parent.read().is_none());
    assert!(component.effects.read().is_empty());
}

#[test]
fn test_component_add_child() {
    let parent =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    let child =
        Component::with_properties(2, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Test add_child works with GcRwLock
    parent.add_child(Gc::clone(&child));
    assert_eq!(parent.children.read().len(), 1);
    assert_eq!(parent.children.read()[0].id, 2);
}

#[test]
fn test_component_lifecycle_mount() {
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Mount should not panic
    component.mount(None);
    component.mount(Some(Gc::clone(&component)));
}

#[test]
fn test_component_lifecycle_unmount() {
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Unmount should not panic
    component.unmount();
}

#[test]
fn test_component_lifecycle_update() {
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Update should not panic (even with no effects)
    component.update();
}

#[test]
fn test_component_types() {
    let text =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());
    assert_eq!(text.component_type, ComponentType::Text);

    let button =
        Component::with_properties(2, ComponentType::Button, rvue::properties::PropertyMap::new());
    assert_eq!(button.component_type, ComponentType::Button);

    let flex =
        Component::with_properties(3, ComponentType::Flex, rvue::properties::PropertyMap::new());
    assert_eq!(flex.component_type, ComponentType::Flex);
}

#[test]
fn test_set_and_get_scroll_state() {
    let component =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());
    let scroll_state = FlexScrollState {
        scroll_offset_x: 10.0,
        scroll_offset_y: 20.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    component.set_scroll_state(scroll_state);
    let retrieved = component.scroll_state();

    assert_eq!(retrieved.scroll_offset_x, 10.0);
    assert_eq!(retrieved.scroll_offset_y, 20.0);
    assert_eq!(retrieved.scroll_width, 100.0);
    assert_eq!(retrieved.scroll_height, 200.0);
}

#[test]
fn test_scroll_state_default_values() {
    let component =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());
    let default_state = component.scroll_state();

    assert_eq!(default_state.scroll_offset_x, 0.0);
    assert_eq!(default_state.scroll_offset_y, 0.0);
    assert_eq!(default_state.scroll_width, 0.0);
    assert_eq!(default_state.scroll_height, 0.0);
}
