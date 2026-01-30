//! Unit tests for Component lifecycle

use rudo_gc::Gc;
use rvue::{Component, ComponentLifecycle, ComponentProps, ComponentType};

#[test]
fn test_component_creation() {
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Hello".to_string(), font_size: None, color: None },
    );

    assert_eq!(component.id, 1);
    assert_eq!(component.component_type, ComponentType::Text);
    assert!(component.children.borrow().is_empty());
    assert!(component.parent.borrow().is_none());
    assert!(component.effects.borrow().is_empty());
}

#[test]
fn test_component_add_child() {
    let parent = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    let child = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text { content: "Child".to_string(), font_size: None, color: None },
    );

    // Test add_child works with GcCell
    parent.add_child(Gc::clone(&child));
    assert_eq!(parent.children.borrow().len(), 1);
    assert_eq!(parent.children.borrow()[0].id, 2);
}

#[test]
fn test_component_lifecycle_mount() {
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Test".to_string(), font_size: None, color: None },
    );

    // Mount should not panic
    component.mount(None);
    component.mount(Some(Gc::clone(&component)));
}

#[test]
fn test_component_lifecycle_unmount() {
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Test".to_string(), font_size: None, color: None },
    );

    // Unmount should not panic
    component.unmount();
}

#[test]
fn test_component_lifecycle_update() {
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Test".to_string(), font_size: None, color: None },
    );

    // Update should not panic (even with no effects)
    component.update();
}

#[test]
fn test_component_types() {
    let text = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "".to_string(), font_size: None, color: None },
    );
    assert_eq!(text.component_type, ComponentType::Text);

    let button = Component::new(
        2,
        ComponentType::Button,
        ComponentProps::Button { label: "Click".to_string() },
    );
    assert_eq!(button.component_type, ComponentType::Button);

    let flex = Component::new(
        3,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 5.0,
            align_items: "start".to_string(),
            justify_content: "center".to_string(),
            styles: None,
        },
    );
    assert_eq!(flex.component_type, ComponentType::Flex);
}

#[test]
fn test_component_props() {
    let text_props =
        ComponentProps::Text { content: "Hello World".to_string(), font_size: None, color: None };

    match text_props {
        ComponentProps::Text { content, .. } => {
            assert_eq!(content, "Hello World");
        }
        _ => panic!("Expected Text props"),
    }

    let button_props = ComponentProps::Button { label: "Submit".to_string() };

    match button_props {
        ComponentProps::Button { label } => {
            assert_eq!(label, "Submit");
        }
        _ => panic!("Expected Button props"),
    }
}
