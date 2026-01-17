//! Scene graph management for Vello rendering

use crate::component::Component;
use crate::render::widget::VelloFragment;
use rudo_gc::Gc;
use vello::kurbo::Affine;

/// Scene structure for managing Vello rendering
pub struct Scene {
    pub vello_scene: Option<vello::Scene>, // Lazy initialization
    pub fragments: Vec<VelloFragment>,
    pub is_dirty: bool,
    pub renderer_initialized: bool,
}

impl Scene {
    /// Create a new scene with lazy renderer initialization
    pub fn new() -> Self {
        Self {
            vello_scene: None, // Defer scene creation until first render
            fragments: Vec::new(),
            is_dirty: true,
            renderer_initialized: false,
        }
    }

    /// Initialize the Vello scene lazily (only when needed)
    fn ensure_initialized(&mut self) {
        if self.vello_scene.is_none() {
            self.vello_scene = Some(vello::Scene::new());
            self.renderer_initialized = true;
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

        // Lazy initialization: Only create scene when actually needed
        self.ensure_initialized();

        // Clear the Vello scene
        if let Some(ref mut scene) = self.vello_scene {
            *scene = vello::Scene::new();
        }

        // Regenerate all fragments
        let mut y_offset = 0.0;
        for fragment in &self.fragments {
            let transform = Affine::translate((0.0, y_offset));
            if let Some(ref mut scene) = self.vello_scene {
                fragment.generate_scene_items(scene, transform);
            }

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
    /// Initializes the scene if it hasn't been created yet
    pub fn vello_scene(&mut self) -> &vello::Scene {
        self.ensure_initialized();
        self.vello_scene.as_ref().unwrap()
    }

    /// Get a mutable reference to the underlying Vello scene
    /// Initializes the scene if it hasn't been created yet
    pub fn vello_scene_mut(&mut self) -> &mut vello::Scene {
        self.ensure_initialized();
        self.vello_scene.as_mut().unwrap()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
