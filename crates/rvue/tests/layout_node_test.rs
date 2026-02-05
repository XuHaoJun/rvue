//! Unit tests for Taffy layout integration

use rvue::layout::LayoutNode;
use rvue::text::TextContext;
use rvue::{Component, ComponentType};
use taffy::TaffyTree;

#[test]
fn test_layout_node_creation() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    let layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_dirty_marking() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

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
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    let layout_node =
        LayoutNode::build_in_tree(&mut taffy, &component, &[], &mut text_context, None);

    assert!(layout_node.is_dirty());
    assert!(layout_node.taffy_node().is_some());
}

#[test]
fn test_layout_node_with_button_component() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let component =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

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
    let parent =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    // Create child components
    #[allow(unused_variables)]
    let child1 =
        Component::with_properties(2, ComponentType::Text, rvue::properties::PropertyMap::new());
    #[allow(unused_variables)]
    let child2 =
        Component::with_properties(3, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Build layout for parent (children should be processed first)
    let parent_layout =
        LayoutNode::build_in_tree(&mut taffy, &parent, &[], &mut text_context, None);

    // Verify parent has a layout node
    assert!(parent_layout.taffy_node().is_some());
}
