//! Unit tests for Button widget event handling

#[allow(deprecated)]
use rvue::{Component, ComponentProps, ComponentType};

#[test]
fn test_button_widget_creation() {
    #[allow(deprecated)]
    let button = Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    assert_eq!(button.component_type, ComponentType::Button);
    #[allow(deprecated)]
    match &*button.props.borrow() {
        ComponentProps::Button { .. } => {
            // Button created successfully without label
        }
        _ => panic!("Expected Button props"),
    };
}

#[test]
fn test_button_widget_event_handler() {
    // Test that button can be created without label
    // Event handlers will be tested in integration tests
    #[allow(deprecated)]
    let button = Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    #[allow(deprecated)]
    match &*button.props.borrow() {
        ComponentProps::Button { .. } => {
            // Button created successfully
        }
        _ => panic!("Expected Button props"),
    };
}

#[test]
fn test_button_widget_multiple_buttons() {
    // Test creating multiple button widgets
    #[allow(deprecated)]
    let buttons = [
        Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None }),
        Component::new(2, ComponentType::Button, ComponentProps::Button { styles: None }),
        Component::new(3, ComponentType::Button, ComponentProps::Button { styles: None }),
    ];

    assert_eq!(buttons.len(), 3);
    for button in buttons.iter() {
        assert_eq!(button.component_type, ComponentType::Button);
        #[allow(deprecated)]
        match &*button.props.borrow() {
            ComponentProps::Button { .. } => {
                // Button created successfully
            }
            _ => panic!("Expected Button props"),
        };
    }
}
