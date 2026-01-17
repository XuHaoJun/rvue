//! Scene graph management for Vello rendering

use rudo_gc::Gc;
use crate::component::Component;
use crate::render::widget::VelloFragment;
use vello::kurbo::Affine;

/// Scene structure for managing Vello rendering
pub struct Scene {
    pub vello_scene: vello::Scene,
    pub fragments: Vec<VelloFragment>,
    pub is_dirty: bool,
}

impl Scene {
    /// Create a new scene
    pub fn new() -> Self {
        Self {
            vello_scene: vello::Scene::new(),
            fragments: Vec::new(),
            is_dirty: true,
        }
    }

    /// Add a component fragment to the scene
    pub fn add_fragment(&mut self, component: Gc<Component>) {
        let fragment = VelloFragment::new(component);
        self.fragments.push(fragment);
        self.is_dirty = true;
    }

    /// Update the scene by regenerating all fragments
    pub fn update(&mut self) {
        if !self.is_dirty {
            return;
        }

        // Clear the Vello scene
        self.vello_scene = vello::Scene::new();

        // Regenerate all fragments
        let mut y_offset = 0.0;
        for fragment in &self.fragments {
            let transform = Affine::translate((0.0, y_offset));
            fragment.generate_scene_items(&mut self.vello_scene, transform);
            
            // Simple vertical stacking for MVP
            y_offset += 50.0;
        }

        self.is_dirty = false;
    }

    /// Mark the scene as dirty (needs re-render)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if the scene is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Get a reference to the underlying Vello scene
    pub fn vello_scene(&self) -> &vello::Scene {
        &self.vello_scene
    }

    /// Get a mutable reference to the underlying Vello scene
    pub fn vello_scene_mut(&mut self) -> &mut vello::Scene {
        &mut self.vello_scene
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
