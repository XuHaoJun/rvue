//! Unit tests for input components

use rvue::{Component, ComponentType};

#[test]
fn test_text_input_creation() {
    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::with(rvue::properties::TextInputValue("Hello".to_string())),
    );

    assert_eq!(text_input.component_type, ComponentType::TextInput);
}

#[test]
fn test_number_input_creation() {
    let number_input = Component::with_properties(
        1,
        ComponentType::NumberInput,
        rvue::properties::PropertyMap::with(rvue::properties::NumberInputValue(42.0)),
    );

    assert_eq!(number_input.component_type, ComponentType::NumberInput);
}

#[test]
fn test_checkbox_creation() {
    let checkbox = Component::with_properties(
        1,
        ComponentType::Checkbox,
        rvue::properties::PropertyMap::with(rvue::properties::CheckboxChecked(true)),
    );

    assert_eq!(checkbox.component_type, ComponentType::Checkbox);
}

#[test]
fn test_radio_creation() {
    let radio = Component::with_properties(
        1,
        ComponentType::Radio,
        rvue::properties::PropertyMap::new()
            .and(rvue::properties::RadioValue("option1".to_string()))
            .and(rvue::properties::RadioChecked(true)),
    );

    assert_eq!(radio.component_type, ComponentType::Radio);
}

#[test]
fn test_text_input_empty_value() {
    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::with(rvue::properties::TextInputValue(String::new())),
    );

    assert_eq!(text_input.component_type, ComponentType::TextInput);
}

#[test]
fn test_checkbox_unchecked() {
    let checkbox = Component::with_properties(
        1,
        ComponentType::Checkbox,
        rvue::properties::PropertyMap::with(rvue::properties::CheckboxChecked(false)),
    );

    assert_eq!(checkbox.component_type, ComponentType::Checkbox);
}

#[test]
fn test_radio_unchecked() {
    let radio = Component::with_properties(
        1,
        ComponentType::Radio,
        rvue::properties::PropertyMap::new()
            .and(rvue::properties::RadioValue("option2".to_string()))
            .and(rvue::properties::RadioChecked(false)),
    );

    assert_eq!(radio.component_type, ComponentType::Radio);
}

#[test]
fn test_number_input_zero() {
    let number_input = Component::with_properties(
        1,
        ComponentType::NumberInput,
        rvue::properties::PropertyMap::with(rvue::properties::NumberInputValue(0.0)),
    );

    assert_eq!(number_input.component_type, ComponentType::NumberInput);
}
