//! Integration test for Flexbox layout

use rvue::{AlignItems, Component, ComponentProps, ComponentType, FlexDirection, JustifyContent};

#[test]
fn test_flexbox_layout_creation() {
    // Test that Flexbox layouts can be created
    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    assert_eq!(flex.component_type, ComponentType::Flex);
    match &flex.props {
        ComponentProps::Flex { direction, gap, align_items, justify_content } => {
            assert_eq!(direction, "row");
            assert_eq!(*gap, 10.0);
            assert_eq!(align_items, "center");
            assert_eq!(justify_content, "start");
        }
        _ => panic!("Expected Flex props"),
    }
}

#[test]
fn test_flexbox_nested_layouts() {
    // Test nested flexbox layouts
    let outer = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 20.0,
            align_items: "stretch".to_string(),
            justify_content: "center".to_string(),
        },
    );

    let inner = Component::new(
        2,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 5.0,
            align_items: "center".to_string(),
            justify_content: "space-between".to_string(),
        },
    );

    assert_eq!(outer.component_type, ComponentType::Flex);
    assert_eq!(inner.component_type, ComponentType::Flex);
}

#[test]
fn test_flexbox_spacing() {
    // Test that spacing (gap) works correctly
    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 15.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
        },
    );

    match &flex.props {
        ComponentProps::Flex { gap, .. } => {
            assert_eq!(*gap, 15.0);
        }
        _ => panic!("Expected Flex props"),
    }
}

#[test]
fn test_flexbox_alignment() {
    // Test alignment properties
    let test_cases = vec![
        ("start", "center"),
        ("center", "end"),
        ("end", "start"),
        ("stretch", "space-between"),
    ];

    for (align_items, justify_content) in test_cases {
        let flex = Component::new(
            1,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "row".to_string(),
                gap: 0.0,
                align_items: align_items.to_string(),
                justify_content: justify_content.to_string(),
            },
        );

        match &flex.props {
            ComponentProps::Flex { align_items: ai, justify_content: jc, .. } => {
                assert_eq!(ai, align_items);
                assert_eq!(jc, justify_content);
            }
            _ => panic!("Expected Flex props"),
        }
    }
}
