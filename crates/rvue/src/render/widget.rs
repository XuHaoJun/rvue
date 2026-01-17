//! Widget-to-Vello mapping

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps};
use vello::kurbo::{Rect, RoundedRect};
use vello::peniko::Color;

/// Vello fragment wrapper for scene graph
pub struct VelloFragment {
    pub component: Gc<Component>,
    pub is_dirty: bool,
}

impl VelloFragment {
    /// Create a new Vello fragment for a component
    pub fn new(component: Gc<Component>) -> Self {
        Self {
            component,
            is_dirty: true,
        }
    }

    /// Mark the fragment as dirty (needs update)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if the fragment is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Generate Vello scene items for this fragment
    /// For MVP, this generates basic rendering commands
    pub fn generate_scene_items(&self, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        match &self.component.component_type {
            ComponentType::Text => {
                Self::render_text(&self.component, scene, transform);
            }
            ComponentType::Button => {
                Self::render_button(&self.component, scene, transform);
            }
            ComponentType::Show => {
                Self::render_show(&self.component, scene, transform);
            }
            ComponentType::For => {
                Self::render_for(&self.component, scene, transform);
            }
            ComponentType::Flex => {
                Self::render_flex(&self.component, scene, transform);
            }
            _ => {
                // Other component types will be implemented later
            }
        }
    }

    /// Render a Text widget to the Vello scene
    fn render_text(component: &Component, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if let ComponentProps::Text { content: _ } = &component.props {
            // For MVP, we'll render text as a simple rectangle placeholder
            // Full text rendering with fonts will be implemented later
            let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
            
            // Fill the text area (placeholder - actual text rendering needs font support)
            let bg_color = Color::from_rgba8(200, 200, 200, 255); // Light gray background
            scene.fill(
                vello::peniko::Fill::NonZero,
                transform,
                bg_color,
                None,
                &rect,
            );
            
            // Note: Actual text rendering requires:
            // 1. Font loading and management
            // 2. Text layout (line breaking, etc.)
            // 3. Glyph rendering
            // This will be implemented in a future iteration
        }
    }

    /// Render a Button widget to the Vello scene
    fn render_button(component: &Component, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if let ComponentProps::Button { label: _ } = &component.props {
            // Render button background
            let rect = RoundedRect::new(0.0, 0.0, 120.0, 40.0, 4.0);
            let bg_color = Color::from_rgb8(70, 130, 180); // Steel blue
            
            scene.fill(
                vello::peniko::Fill::NonZero,
                transform,
                bg_color,
                None,
                &rect,
            );
            
            // Render button border
            // For MVP, we'll skip the border stroke - full stroke support will be added later
            // when we have proper peniko::Stroke API access
            
            // Note: Button text rendering will be added when text rendering is fully implemented
        }
    }

    /// Render a Show widget to the Vello scene
    /// Only renders children if when is true
    fn render_show(component: &Component, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if let ComponentProps::Show { when } = &component.props {
            // Only render children if when is true
            if *when {
                // Render all children
                for child in &component.children {
                    // Recursively render children
                    let child_fragment = VelloFragment::new(rudo_gc::Gc::clone(child));
                    child_fragment.generate_scene_items(scene, transform);
                }
            }
            // If when is false, skip rendering (hidden components don't consume rendering resources)
        }
    }

    /// Render a For widget to the Vello scene
    /// Renders all children (list items) efficiently
    fn render_for(component: &Component, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if let ComponentProps::For { item_count: _ } = &component.props {
            // Render all children (list items)
            let mut y_offset = 0.0;
            for child in &component.children {
                // Recursively render children with vertical stacking
                let child_transform = vello::kurbo::Affine::translate((0.0, y_offset));
                let child_fragment = VelloFragment::new(rudo_gc::Gc::clone(child));
                child_fragment.generate_scene_items(scene, transform * child_transform);
                
                // Simple vertical stacking for MVP
                y_offset += 50.0;
            }
        }
    }

    /// Render a Flex widget to the Vello scene
    /// Applies layout results to position children correctly
    fn render_flex(component: &Component, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if let ComponentProps::Flex { .. } = &component.props {
            // For MVP, we'll render children with simple stacking
            // In a full implementation, we'd use layout results from Taffy
            let mut y_offset = 0.0;
            for child in &component.children {
                // Apply layout transform if available
                // For MVP, use simple vertical stacking
                let child_transform = vello::kurbo::Affine::translate((0.0, y_offset));
                let child_fragment = VelloFragment::new(rudo_gc::Gc::clone(child));
                child_fragment.generate_scene_items(scene, transform * child_transform);
                
                y_offset += 50.0;
            }
        }
    }
}
