//! Unit tests for input components

use rvue::{Component, ComponentProps, ComponentType};

#[test]
fn test_text_input_creation() {
    let text_input = Component::new(
        1,
        ComponentType::TextInput,
        ComponentProps::TextInput { value: "Hello".to_string(), styles: None },
    );

    assert_eq!(text_input.component_type, ComponentType::TextInput);
    match &*text_input.props.borrow() {
        ComponentProps::TextInput { value, .. } => {
            assert_eq!(value, "Hello");
        }
        _ => panic!("Expected TextInput props"),
    };
}

#[test]
fn test_number_input_creation() {
    let number_input = Component::new(
        1,
        ComponentType::NumberInput,
        ComponentProps::NumberInput { value: 42.0, styles: None },
    );

    assert_eq!(number_input.component_type, ComponentType::NumberInput);
    match &*number_input.props.borrow() {
        ComponentProps::NumberInput { value, .. } => {
            assert_eq!(*value, 42.0);
        }
        _ => panic!("Expected NumberInput props"),
    };
}

#[test]
fn test_checkbox_creation() {
    let checkbox = Component::new(
        1,
        ComponentType::Checkbox,
        ComponentProps::Checkbox { checked: true, styles: None },
    );

    assert_eq!(checkbox.component_type, ComponentType::Checkbox);
    match &*checkbox.props.borrow() {
        ComponentProps::Checkbox { checked, .. } => {
            assert!(*checked);
        }
        _ => panic!("Expected Checkbox props"),
    };
}

#[test]
fn test_radio_creation() {
    let radio = Component::new(
        1,
        ComponentType::Radio,
        ComponentProps::Radio { value: "option1".to_string(), checked: true, styles: None },
    );

    assert_eq!(radio.component_type, ComponentType::Radio);
    match &*radio.props.borrow() {
        ComponentProps::Radio { value, checked, .. } => {
            assert_eq!(value, "option1");
            assert!(*checked);
        }
        _ => panic!("Expected Radio props"),
    };
}

#[test]
fn test_text_input_empty_value() {
    let text_input = Component::new(
        1,
        ComponentType::TextInput,
        ComponentProps::TextInput { value: String::new(), styles: None },
    );

    match &*text_input.props.borrow() {
        ComponentProps::TextInput { value, .. } => {
            assert_eq!(value, "");
        }
        _ => panic!("Expected TextInput props"),
    };
}

#[test]
fn test_checkbox_unchecked() {
    let checkbox = Component::new(
        1,
        ComponentType::Checkbox,
        ComponentProps::Checkbox { checked: false, styles: None },
    );

    match &*checkbox.props.borrow() {
        ComponentProps::Checkbox { checked, .. } => {
            assert!(!*checked);
        }
        _ => panic!("Expected Checkbox props"),
    };
}

#[test]
fn test_radio_unchecked() {
    let radio = Component::new(
        1,
        ComponentType::Radio,
        ComponentProps::Radio { value: "option2".to_string(), checked: false, styles: None },
    );

    match &*radio.props.borrow() {
        ComponentProps::Radio { value, checked, .. } => {
            assert_eq!(value, "option2");
            assert!(!*checked);
        }
        _ => panic!("Expected Radio props"),
    };
}

#[test]
fn test_number_input_zero() {
    let number_input = Component::new(
        1,
        ComponentType::NumberInput,
        ComponentProps::NumberInput { value: 0.0, styles: None },
    );

    match &*number_input.props.borrow() {
        ComponentProps::NumberInput { value, .. } => {
            assert_eq!(*value, 0.0);
        }
        _ => panic!("Expected NumberInput props"),
    };
}
