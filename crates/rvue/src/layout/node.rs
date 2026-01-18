//! Layout node wrapper around Taffy

use crate::component::{Component, ComponentProps, ComponentType};
use rudo_gc::Gc;
use taffy::prelude::*;
use taffy::TaffyTree;

/// Layout node wrapper around Taffy node
pub struct LayoutNode {
    pub component: Gc<Component>,
    pub taffy_node: Option<NodeId>,
    pub taffy: TaffyTree<()>,
    pub is_dirty: bool,
    pub layout_result: Option<Layout>,
}

impl LayoutNode {
    /// Create a new layout node for a component
    pub fn new(component: Gc<Component>) -> Self {
        let mut taffy = TaffyTree::new();
        let taffy_node = taffy.new_leaf(Self::component_to_taffy_style(&component)).ok();

        Self { component, taffy_node, taffy, is_dirty: true, layout_result: None }
    }

    /// Convert component props to Taffy style
    fn component_to_taffy_style(component: &Component) -> Style {
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
            _ => {
                // For non-flex components, use default style
                Style::default()
            }
        }
    }

    /// Calculate layout for this node and its children
    pub fn calculate_layout(&mut self) -> Result<(), taffy::TaffyError> {
        if !self.is_dirty {
            return Ok(());
        }

        if let Some(node_id) = self.taffy_node {
            // Compute layout (returns () on success)
            self.taffy.compute_layout(node_id, Size::MAX_CONTENT)?;
            // Get the layout result
            self.layout_result = Some(*self.taffy.layout(node_id)?);
            self.is_dirty = false;
        }

        Ok(())
    }

    /// Mark the layout node as dirty (needs recalculation)
    /// This should be called when style or content changes
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        // Also mark children as dirty
        // In a full implementation, we'd traverse the component tree
    }

    /// Update Taffy style when component props change
    pub fn update_style(&mut self) -> Result<(), taffy::TaffyError> {
        if let Some(node_id) = self.taffy_node {
            let new_style = Self::component_to_taffy_style(&self.component);
            self.taffy.set_style(node_id, new_style)?;
            self.mark_dirty();
        }
        Ok(())
    }

    /// Check if the layout node is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Get the calculated layout result
    pub fn layout(&self) -> Option<&Layout> {
        self.layout_result.as_ref()
    }
}
