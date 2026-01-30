//! Layout node wrapper around Taffy

use crate::component::{Component, ComponentProps, ComponentType};
use crate::text::{BrushIndex, ParleyLayoutWrapper, TextContext};
use parley::Layout;
use rudo_gc::Trace;
use taffy::prelude::*;
use taffy::TaffyTree;

/// Layout node wrapper holding calculation results
#[derive(Clone)]
pub struct LayoutNode {
    pub taffy_node: Option<NodeId>,
    pub is_dirty: bool,
    pub layout_result: Option<taffy::Layout>,
}

unsafe impl Trace for LayoutNode {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // LayoutNode contains only primitive types and Taffy types, no GC pointers
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutNode {
    /// Create a new layout node
    pub fn new() -> Self {
        Self { taffy_node: None, is_dirty: true, layout_result: None }
    }

    /// Build a single layout node with given children in a specific TaffyTree
    /// Reuses existing NodeId if the component already has one, to avoid invalidating references
    pub fn build_in_tree(
        taffy: &mut TaffyTree<()>,
        component: &Component,
        child_nodes: &[NodeId],
        text_context: &mut TextContext,
    ) -> Self {
        // Control-flow components (For, Show) have no layout of their own
        // They are transparent to the layout system
        if matches!(component.component_type, ComponentType::For | ComponentType::Show) {
            return Self { taffy_node: None, is_dirty: true, layout_result: None };
        }

        let style = Self::component_to_taffy_style(component);

        // Handle text measurement
        if let ComponentType::Text = component.component_type {
            return Self::build_text_node(taffy, component, style, text_context);
        }

        let taffy_node =
            if let Some(existing_node) = component.layout_node().and_then(|ln| ln.taffy_node()) {
                if taffy.set_style(existing_node, style.clone()).is_ok() {
                    if !child_nodes.is_empty() {
                        let _ = taffy.set_children(existing_node, child_nodes);
                    }
                    Some(existing_node)
                } else if child_nodes.is_empty() {
                    taffy.new_leaf(style).ok()
                } else {
                    taffy.new_with_children(style, child_nodes).ok()
                }
            } else if child_nodes.is_empty() {
                taffy.new_leaf(style).ok()
            } else {
                taffy.new_with_children(style, child_nodes).ok()
            };

        Self { taffy_node, is_dirty: true, layout_result: None }
    }

    fn build_text_node(
        taffy: &mut TaffyTree<()>,
        component: &Component,
        mut style: Style,
        text_context: &mut TextContext,
    ) -> Self {
        let content = if let ComponentProps::Text { content, .. } = &*component.props.borrow() {
            content.clone()
        } else {
            String::new()
        };

        // Eagerly build text layout to get dimensions
        let mut layout_builder =
            text_context.layout_ctx.ranged_builder(&mut text_context.font_ctx, &content, 1.0, true);
        layout_builder.push_default(parley::style::StyleProperty::FontSize(16.0));
        layout_builder.push_default(parley::style::StyleProperty::Brush(BrushIndex(0)));
        // Use a more robust font stack
        layout_builder.push_default(parley::style::FontStack::Source(std::borrow::Cow::Borrowed(
            "sans-serif",
        )));

        let mut layout: Layout<BrushIndex> = layout_builder.build(&content);
        layout.break_all_lines(None); // Generate lines (no max advance)

        let width = layout.width();
        let height = layout.height();

        // Store layout in user_data for rendering
        {
            let mut user_data = component.user_data.borrow_mut();
            *user_data = Some(Box::new(ParleyLayoutWrapper(layout)));
        }

        // Set explicit size in style based on text measurement
        // If width/height is 0, provide a small default to avoid collapse in tests if fonts aren't loaded
        let width = if width > 0.0 { width } else { 10.0 * content.len() as f32 };
        let height = if height > 0.0 { height } else { 20.0 };

        style.size = Size { width: length(width), height: length(height) };

        let taffy_node = taffy.new_leaf(style).ok();

        Self { taffy_node, is_dirty: true, layout_result: None }
    }

    /// Convert component props to Taffy style
    pub fn component_to_taffy_style(component: &Component) -> Style {
        match &component.component_type {
            ComponentType::Flex => {
                if let ComponentProps::Flex {
                    direction, gap, align_items, justify_content, ..
                } = &*component.props.borrow()
                {
                    let flex_direction = match direction.to_lowercase().replace('_', "-").as_str() {
                        "row" => taffy::prelude::FlexDirection::Row,
                        "column" => taffy::prelude::FlexDirection::Column,
                        "row-reverse" | "rowreverse" => taffy::prelude::FlexDirection::RowReverse,
                        "column-reverse" | "columnreverse" => {
                            taffy::prelude::FlexDirection::ColumnReverse
                        }
                        _ => taffy::prelude::FlexDirection::Row,
                    };

                    let align_items_taffy = match align_items.to_lowercase().as_str() {
                        "start" => taffy::prelude::AlignItems::Start,
                        "end" => taffy::prelude::AlignItems::End,
                        "center" => taffy::prelude::AlignItems::Center,
                        "stretch" => taffy::prelude::AlignItems::Stretch,
                        "baseline" => taffy::prelude::AlignItems::Baseline,
                        _ => taffy::prelude::AlignItems::Stretch,
                    };

                    let justify_content_taffy =
                        match justify_content.to_lowercase().replace('_', "-").as_str() {
                            "start" => taffy::prelude::JustifyContent::Start,
                            "end" => taffy::prelude::JustifyContent::End,
                            "center" => taffy::prelude::JustifyContent::Center,
                            "space-between" | "spacebetween" => {
                                taffy::prelude::JustifyContent::SpaceBetween
                            }
                            "space-around" | "spacearound" => {
                                taffy::prelude::JustifyContent::SpaceAround
                            }
                            "space-evenly" | "spaceevenly" => {
                                taffy::prelude::JustifyContent::SpaceEvenly
                            }
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
            ComponentType::Text => Style { ..Default::default() },
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
            // For is a control-flow component, not a visual component
            // It has no layout of its own - children are managed by parent container
            ComponentType::For | ComponentType::Show => Style::default(),
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
    pub fn layout(&self) -> Option<&taffy::Layout> {
        self.layout_result.as_ref()
    }

    /// Get the taffy_node ID
    pub fn taffy_node(&self) -> Option<NodeId> {
        self.taffy_node
    }
}
