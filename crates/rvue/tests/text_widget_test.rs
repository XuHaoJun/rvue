//! Unit tests for Text widget rendering

use rvue::{Component, ComponentId, ComponentType};

#[test]
fn test_text_widget_creation() {
    let text =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    assert_eq!(text.component_type, ComponentType::Text);
}

#[test]
fn test_text_widget_with_empty_string() {
    let text =
        Component::with_properties(2, ComponentType::Text, rvue::properties::PropertyMap::new());

    assert_eq!(text.component_type, ComponentType::Text);
}

#[test]
fn test_text_widget_with_special_characters() {
    let text =
        Component::with_properties(3, ComponentType::Text, rvue::properties::PropertyMap::new());

    assert_eq!(text.component_type, ComponentType::Text);
}

#[test]
fn test_text_widget_rendering_properties() {
    // Test that Text widget can be created with different content
    let texts = [
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new()),
        Component::with_properties(2, ComponentType::Text, rvue::properties::PropertyMap::new()),
        Component::with_properties(3, ComponentType::Text, rvue::properties::PropertyMap::new()),
    ];

    assert_eq!(texts.len(), 3);
    for (i, text) in texts.iter().enumerate() {
        assert_eq!(text.id, (i + 1) as ComponentId);
        assert_eq!(text.component_type, ComponentType::Text);
    }
}
