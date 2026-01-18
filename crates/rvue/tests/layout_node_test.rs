//! Unit tests for Taffy layout integration

use rvue::layout::LayoutNode;
use rvue::text::TextContext;
use rvue::{Component, ComponentProps, ComponentType};
use taffy::TaffyTree;

#[test]
fn test_layout_node_creation() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    let layout_node = LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_dirty_marking() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 5.0,
            align_items: "start".to_string(),
            justify_content: "center".to_string(),
        },
    );

    let mut layout_node = LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context);

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
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Hello".to_string(), font_size: None, color: None },
    );

    let layout_node = LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_with_button_component() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component = Component::new(
        1,
        ComponentType::Button,
        ComponentProps::Button { label: "Click".to_string() },
    );

    let layout_node = LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_tree_structure() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    // Test layout nodes for a component tree
    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    let child1 = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Child 1".to_string(), font_size: None, color: None },
    );

    let child2 = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text { content: "Child 2".to_string(), font_size: None, color: None },
    );

    let root_layout = LayoutNode::build_in_tree(&mut taffy, &root, &[], &mut text_context);
    let child1_layout = LayoutNode::build_in_tree(&mut taffy, &child1, &[], &mut text_context);
    let child2_layout = LayoutNode::build_in_tree(&mut taffy, &child2, &[], &mut text_context);

    // All should be dirty initially
    assert!(root_layout.is_dirty());
    assert!(child1_layout.is_dirty());
    assert!(child2_layout.is_dirty());
}

#[test]
fn test_layout_calculation() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
        },
    );

    let mut layout_node = LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context);
    layout_node.calculate_layout(&mut taffy).unwrap();

    assert!(!layout_node.is_dirty());
    let result = layout_node.layout().unwrap();
    assert!(result.size.width >= 0.0);
    assert!(result.size.height >= 0.0);
}
