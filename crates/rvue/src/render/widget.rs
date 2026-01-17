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
}
