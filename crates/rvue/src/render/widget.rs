//! Widget-to-Vello mapping

use crate::component::{Component, ComponentProps, ComponentType};
use rudo_gc::{Gc, GcCell};
use vello::kurbo::{Circle, Rect, RoundedRect};
use vello::peniko::Color;

/// Vello fragment wrapper for scene graph
pub struct VelloFragment {
    pub component: Gc<Component>,
    pub cached_scene: GcCell<Option<vello::Scene>>,
}

impl VelloFragment {
    /// Create a new Vello fragment for a component
    pub fn new(component: Gc<Component>) -> Self {
        Self { component, cached_scene: GcCell::new(None) }
    }

    /// Mark the fragment as dirty (needs update)
    pub fn mark_dirty(&self) {
        self.component.mark_dirty();
        *self.cached_scene.borrow_mut() = None;
    }

    /// Check if the fragment is dirty
    pub fn is_dirty(&self) -> bool {
        self.component.is_dirty() || self.cached_scene.borrow().is_none()
    }

    /// Generate Vello scene items for this fragment
    /// Optimized to use cached scene if not dirty
    pub fn generate_scene_items(&self, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if self.is_dirty() {
            let mut sub_scene = vello::Scene::new();

            // Render the component into the sub-scene (with identity transform)
            match &self.component.component_type {
                ComponentType::Text => {
                    Self::render_text(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Button => {
                    Self::render_button(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Show => {
                    Self::render_show(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::For => {
                    Self::render_for(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Flex => {
                    Self::render_flex(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::TextInput => {
                    Self::render_text_input(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::NumberInput => {
                    Self::render_number_input(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Checkbox => {
                    Self::render_checkbox(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Radio => {
                    Self::render_radio(
                        &self.component,
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                _ => {}
            }

            // Store the sub-scene in cache
            *self.cached_scene.borrow_mut() = Some(sub_scene);
            self.component.clear_dirty();
        }

        // Append the cached sub-scene to the main scene with the provided transform
        if let Some(ref sub_scene) = *self.cached_scene.borrow() {
            scene.append(sub_scene, Some(transform));
        }
    }

    /// Render a Text widget to the Vello scene
    fn render_text(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Text { content: _ } = &*component.props.borrow() {
            // For MVP, we'll render text as a simple rectangle placeholder
            // Full text rendering with fonts will be implemented later
            let rect = Rect::new(0.0, 0.0, 100.0, 20.0);

            // Fill the text area (placeholder - actual text rendering needs font support)
            let bg_color = Color::from_rgba8(200, 200, 200, 255); // Light gray background
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);

            // Note: Actual text rendering requires:
            // 1. Font loading and management
            // 2. Text layout (line breaking, etc.)
            // 3. Glyph rendering
            // This will be implemented in a future iteration
        }
    }

    /// Render a Button widget to the Vello scene
    fn render_button(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Button { label: _ } = &*component.props.borrow() {
            // Render button background
            let rect = RoundedRect::new(0.0, 0.0, 120.0, 40.0, 4.0);
            let bg_color = Color::from_rgb8(70, 130, 180); // Steel blue

            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);

            // Render button border
            // For MVP, we'll skip the border stroke - full stroke support will be added later
            // when we have proper peniko::Stroke API access

            // Note: Button text rendering will be added when text rendering is fully implemented
        }
    }

    /// Render a Show widget to the Vello scene
    /// Only renders children if when is true
    fn render_show(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Show { when } = &*component.props.borrow() {
            // Only render children if when is true
            if *when {
                // Render all children
                for child in component.children.borrow().iter() {
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
    fn render_for(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::For { item_count: _ } = &*component.props.borrow() {
            // Render all children (list items)
            let mut y_offset = 0.0;
            for child in component.children.borrow().iter() {
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
    fn render_flex(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Flex { .. } = &*component.props.borrow() {
            // For MVP, we'll render children with simple stacking
            // In a full implementation, we'd use layout results from Taffy
            let mut y_offset = 0.0;
            for child in component.children.borrow().iter() {
                // Apply layout transform if available
                // For MVP, use simple vertical stacking
                let child_transform = vello::kurbo::Affine::translate((0.0, y_offset));
                let child_fragment = VelloFragment::new(rudo_gc::Gc::clone(child));
                child_fragment.generate_scene_items(scene, transform * child_transform);

                y_offset += 50.0;
            }
        }
    }

    /// Render a TextInput widget to the Vello scene
    fn render_text_input(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::TextInput { value: _ } = &*component.props.borrow() {
            // Render input field background
            let rect = RoundedRect::new(0.0, 0.0, 200.0, 30.0, 2.0);
            let bg_color = Color::from_rgb8(255, 255, 255); // White background

            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);

            // Note: Text rendering and cursor will be added when text rendering is fully implemented
        }
    }

    /// Render a NumberInput widget to the Vello scene
    fn render_number_input(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::NumberInput { value: _ } = &*component.props.borrow() {
            // Render input field background (similar to TextInput)
            let rect = RoundedRect::new(0.0, 0.0, 150.0, 30.0, 2.0);
            let bg_color = Color::from_rgb8(255, 255, 255); // White background

            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    /// Render a Checkbox widget to the Vello scene
    fn render_checkbox(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Checkbox { checked } = &*component.props.borrow() {
            // Render checkbox square
            let rect = Rect::new(0.0, 0.0, 20.0, 20.0);
            let bg_color = if *checked {
                Color::from_rgb8(70, 130, 180) // Steel blue when checked
            } else {
                Color::from_rgb8(255, 255, 255) // White when unchecked
            };

            // Fill background
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);

            // Render border
            // Note: Border rendering will be added when stroke API is fully available
        }
    }

    /// Render a Radio widget to the Vello scene
    fn render_radio(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Radio { value: _, checked } = &*component.props.borrow() {
            // Render radio circle
            let circle = Circle::new((10.0, 10.0), 10.0);
            let bg_color = if *checked {
                Color::from_rgb8(70, 130, 180) // Steel blue when checked
            } else {
                Color::from_rgb8(255, 255, 255) // White when unchecked
            };

            // Fill background
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &circle);

            // Render inner dot if checked
            if *checked {
                let inner_circle = Circle::new((10.0, 10.0), 5.0);
                let dot_color = Color::from_rgb8(255, 255, 255);
                scene.fill(vello::peniko::Fill::NonZero, transform, dot_color, None, &inner_circle);
            }
        }
    }
}
