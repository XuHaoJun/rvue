//! Layout node wrapper around Taffy

use crate::component::{Component, ComponentProps, ComponentType};
use taffy::prelude::*;
use taffy::TaffyTree;

/// Layout node wrapper holding calculation results
#[derive(Clone, Debug, Default)]
pub struct LayoutNode {
    pub taffy_node: Option<NodeId>,
    pub is_dirty: bool,
    pub layout_result: Option<Layout>,
}

impl LayoutNode {
    /// Create a new layout node
    pub fn new() -> Self {
        Self { taffy_node: None, is_dirty: true, layout_result: None }
    }

    /// Build a single layout node with given children in a specific TaffyTree
    pub fn build_in_tree(
        taffy: &mut TaffyTree<()>,
        component: &Component,
        child_nodes: &[NodeId],
    ) -> Self {
        let style = Self::component_to_taffy_style(component);
        let taffy_node = if child_nodes.is_empty() {
            taffy.new_leaf(style).ok()
        } else {
            taffy.new_with_children(style, child_nodes).ok()
        };

        Self { taffy_node, is_dirty: true, layout_result: None }
    }

    /// Convert component props to Taffy style
    pub fn component_to_taffy_style(component: &Component) -> Style {
        match &component.component_type {
            ComponentType::Flex => {
                if let ComponentProps::Flex { direction, gap, align_items, justify_content } =
                    &*component.props.borrow()
                {
                    let flex_direction = match direction.as_str() {
                        "row" => taffy::prelude::FlexDirection::Row,
                        "column" => taffy::prelude::FlexDirection::Column,
                        "row-reverse" => taffy::prelude::FlexDirection::RowReverse,
                        "column-reverse" => taffy::prelude::FlexDirection::ColumnReverse,
                        _ => taffy::prelude::FlexDirection::Row,
                    };

                    let align_items_taffy = match align_items.as_str() {
                        "start" => taffy::prelude::AlignItems::Start,
                        "end" => taffy::prelude::AlignItems::End,
                        "center" => taffy::prelude::AlignItems::Center,
                        "stretch" => taffy::prelude::AlignItems::Stretch,
                        "baseline" => taffy::prelude::AlignItems::Baseline,
                        _ => taffy::prelude::AlignItems::Stretch,
                    };

                    let justify_content_taffy = match justify_content.as_str() {
                        "start" => taffy::prelude::JustifyContent::Start,
                        "end" => taffy::prelude::JustifyContent::End,
                        "center" => taffy::prelude::JustifyContent::Center,
                        "space-between" => taffy::prelude::JustifyContent::SpaceBetween,
                        "space-around" => taffy::prelude::JustifyContent::SpaceAround,
                        "space-evenly" => taffy::prelude::JustifyContent::SpaceEvenly,
                        _ => taffy::prelude::JustifyContent::Start,
                    };

                    Style {
                        display: Display::Flex,
                        flex_direction,
                        gap: Size { width: length(*gap), height: length(*gap) },
                        align_items: Some(align_items_taffy),
                        justify_content: Some(justify_content_taffy),
                        ..Default::default()
                    }
                } else {
                    Style::default()
                }
            }
            ComponentType::Text => Style {
                size: Size { width: length(100.0), height: length(20.0) },
                ..Default::default()
            },
            ComponentType::Button => Style {
                size: Size { width: length(120.0), height: length(40.0) },
                ..Default::default()
            },
            ComponentType::TextInput => Style {
                size: Size { width: length(200.0), height: length(30.0) },
                ..Default::default()
            },
            ComponentType::NumberInput => Style {
                size: Size { width: length(150.0), height: length(30.0) },
                ..Default::default()
            },
            ComponentType::Checkbox | ComponentType::Radio => Style {
                size: Size { width: length(20.0), height: length(20.0) },
                ..Default::default()
            },
            _ => Style::default(),
        }
    }

    /// Calculate layout for this node and its children using the provided TaffyTree
    pub fn calculate_layout(&mut self, taffy: &mut TaffyTree<()>) -> Result<(), taffy::TaffyError> {
        if let Some(node_id) = self.taffy_node {
            taffy.compute_layout(node_id, Size::MAX_CONTENT)?;
            self.update_results_recursive(taffy, node_id)?;
            self.is_dirty = false;
        }
        Ok(())
    }

    /// Recursively update layout results from the TaffyTree
    /// This is internal but could be made public if needed
    fn update_results_recursive(
        &mut self,
        taffy: &TaffyTree<()>,
        node_id: NodeId,
    ) -> Result<(), taffy::TaffyError> {
        self.layout_result = Some(*taffy.layout(node_id)?);
        Ok(())
    }

    /// Mark the layout node as dirty (needs recalculation)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if the layout node is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Get the calculated layout result
    pub fn layout(&self) -> Option<&Layout> {
        self.layout_result.as_ref()
    }

    /// Get the taffy_node ID
    pub fn taffy_node(&self) -> Option<NodeId> {
        self.taffy_node
    }
}
