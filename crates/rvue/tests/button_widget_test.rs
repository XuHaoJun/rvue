//! Unit tests for Button widget event handling

use rvue::{Component, ComponentProps, ComponentType};

#[test]
fn test_button_widget_creation() {
    let button = Component::new(
        1,
        ComponentType::Button,
        ComponentProps::Button { label: "Click Me".to_string() },
    );

    assert_eq!(button.component_type, ComponentType::Button);
    match &*button.props.borrow() {
        ComponentProps::Button { label } => {
            assert_eq!(label, "Click Me");
        }
        _ => panic!("Expected Button props"),
    };
}

#[test]
fn test_button_widget_event_handler() {
    // Test that button can be created with a label
    // Event handlers will be tested in integration tests
    let button = Component::new(
        1,
        ComponentType::Button,
        ComponentProps::Button { label: "Submit".to_string() },
    );

    match &*button.props.borrow() {
        ComponentProps::Button { label } => {
            assert_eq!(label, "Submit");
        }
        _ => panic!("Expected Button props"),
    };
}

#[test]
fn test_button_widget_multiple_buttons() {
    // Test creating multiple button widgets
    let buttons = [
        Component::new(
            1,
            ComponentType::Button,
            ComponentProps::Button { label: "OK".to_string() },
        ),
        Component::new(
            2,
            ComponentType::Button,
            ComponentProps::Button { label: "Cancel".to_string() },
        ),
        Component::new(
            3,
            ComponentType::Button,
            ComponentProps::Button { label: "Apply".to_string() },
        ),
    ];

    assert_eq!(buttons.len(), 3);
    let labels = ["OK", "Cancel", "Apply"];
    for (i, button) in buttons.iter().enumerate() {
        assert_eq!(button.component_type, ComponentType::Button);
        match &*button.props.borrow() {
            ComponentProps::Button { label } => {
                assert_eq!(label, labels[i]);
            }
            _ => panic!("Expected Button props"),
        };
    }
}

#[test]
fn test_button_widget_with_empty_label() {
    let button =
        Component::new(1, ComponentType::Button, ComponentProps::Button { label: String::new() });

    match &*button.props.borrow() {
        ComponentProps::Button { label } => {
            assert_eq!(label, "");
        }
        _ => panic!("Expected Button props"),
    };
}
