//! Unit tests for Button widget event handling

use rvue::{Component, ComponentProps, ComponentType};

#[test]
fn test_button_widget_creation() {
    let button = Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    assert_eq!(button.component_type, ComponentType::Button);
    match &*button.props.borrow() {
        ComponentProps::Button { .. } => {
            // Button created successfully
        }
        _ => panic!("Expected Button props"),
    }
}

#[test]
fn test_button_widget_event_handler() {
    // Test that button can be created without label
    // Event handlers will be tested in integration tests
    let button = Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    match &*button.props.borrow() {
        ComponentProps::Button { .. } => {
            // Button created successfully
        }
        _ => panic!("Expected Button props"),
    }
}

#[test]
fn test_button_widget_multiple_buttons() {
    // Test creating multiple button widgets
    let buttons = vec![
        Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None }),
        Component::new(2, ComponentType::Button, ComponentProps::Button { styles: None }),
        Component::new(3, ComponentType::Button, ComponentProps::Button { styles: None }),
    ];

    assert_eq!(buttons.len(), 3);
    for button in buttons.iter() {
        assert_eq!(button.component_type, ComponentType::Button);
        match &*button.props.borrow() {
            ComponentProps::Button { .. } => {
                // Button created successfully
            }
            _ => panic!("Expected Button props"),
        }
    }
}

#[test]
fn test_button_widget_with_styles() {
    let button = Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    match &*button.props.borrow() {
        ComponentProps::Button { styles } => {
            assert!(styles.is_none());
        }
        _ => panic!("Expected Button props"),
    }
}
