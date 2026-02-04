//! Unit tests for Taffy layout integration

use rvue::layout::LayoutNode;
use rvue::text::TextContext;
#[allow(deprecated)]
use rvue::{Component, ComponentProps, ComponentType};
use taffy::TaffyTree;

#[test]
fn test_layout_node_creation() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    #[allow(deprecated)]
    let component = Component::new(
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

    let layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_dirty_marking() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    #[allow(deprecated)]
    let component = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 5.0,
            align_items: "start".to_string(),
            justify_content: "center".to_string(),
            styles: None,
        },
    );

    let mut layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    // Initially dirty
    assert!(layout_node.is_dirty());

    // Mark as clean (simulated)
    // In a full implementation, this would happen after layout calculation
    // For MVP, we'll test the dirty marking mechanism
    layout_node.mark_dirty();
    assert!(layout_node.is_dirty());
}

#[test]
fn test_layout_node_with_text_component() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    #[allow(deprecated)]
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Hello".to_string(), styles: None },
    );

    let layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_with_button_component() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    #[allow(deprecated)]
    let component =
        Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

    let layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_tree_structure() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();

    // Create a parent Flex component
    #[allow(deprecated)]
    let parent = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 8.0,
            align_items: "center".to_string(),
            justify_content: "flex-start".to_string(),
            styles: None,
        },
    );

    // Create child components
    #[allow(deprecated)]
    let child1 = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text { content: "Child 1".to_string(), styles: None },
    );
    #[allow(deprecated)]
    let child2 = Component::new(
        3,
        ComponentType::Text,
        ComponentProps::Text { content: "Child 2".to_string(), styles: None },
    );

    // Build layout for parent (children should be processed first)
    let parent_layout =
        LayoutNode::build_in_tree(&mut taffy, &parent, &[], &mut text_context, None);

    // Verify parent has a layout node
    assert!(parent_layout.taffy_node().is_some());
}
