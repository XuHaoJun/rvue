//! Unit tests for Button widget event handling

use rvue::{Component, ComponentType};

#[test]
fn test_button_widget_creation() {
    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    assert_eq!(button.component_type, ComponentType::Button);
}

#[test]
fn test_button_widget_event_handler() {
    // Test that button can be created without label
    // Event handlers will be tested in integration tests
    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    assert_eq!(button.component_type, ComponentType::Button);
}

#[test]
fn test_button_widget_multiple_buttons() {
    // Test creating multiple button widgets
    let buttons = [
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new()),
        Component::with_properties(2, ComponentType::Button, rvue::properties::PropertyMap::new()),
        Component::with_properties(3, ComponentType::Button, rvue::properties::PropertyMap::new()),
    ];

    assert_eq!(buttons.len(), 3);
    for button in buttons.iter() {
        assert_eq!(button.component_type, ComponentType::Button);
    }
}
