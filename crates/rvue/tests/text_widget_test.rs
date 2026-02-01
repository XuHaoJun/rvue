//! Unit tests for Text widget rendering

use rvue::{Component, ComponentId, ComponentProps, ComponentType};

#[test]
fn test_text_widget_creation() {
    let text = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Hello World".to_string(), styles: None },
    );

    assert_eq!(text.component_type, ComponentType::Text);
    match &*text.props.borrow() {
        ComponentProps::Text { content, .. } => {
            assert_eq!(content, "Hello World");
        }
        _ => panic!("Expected Text props"),
    };
}

#[test]
fn test_text_widget_with_empty_string() {
    let text = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text { content: String::new(), styles: None },
    );

    match &*text.props.borrow() {
        ComponentProps::Text { content, .. } => {
            assert_eq!(content, "");
        }
        _ => panic!("Expected Text props"),
    };
}

#[test]
fn test_text_widget_with_special_characters() {
    let text = Component::new(
        3,
        ComponentType::Text,
        ComponentProps::Text { content: "Hello\nWorld\t!".to_string(), styles: None },
    );

    match &*text.props.borrow() {
        ComponentProps::Text { content, .. } => {
            assert_eq!(content, "Hello\nWorld\t!");
        }
        _ => panic!("Expected Text props"),
    };
}

#[test]
fn test_text_widget_rendering_properties() {
    // Test that Text widget can be created with different content
    let texts = [
        Component::new(
            1,
            ComponentType::Text,
            ComponentProps::Text { content: "A".to_string(), styles: None },
        ),
        Component::new(
            2,
            ComponentType::Text,
            ComponentProps::Text { content: "B".to_string(), styles: None },
        ),
        Component::new(
            3,
            ComponentType::Text,
            ComponentProps::Text { content: "C".to_string(), styles: None },
        ),
    ];

    assert_eq!(texts.len(), 3);
    for (i, text) in texts.iter().enumerate() {
        assert_eq!(text.id, (i + 1) as ComponentId);
        assert_eq!(text.component_type, ComponentType::Text);
    }
}
