//! Scene graph management for Vello rendering

use crate::component::build_layout_tree;
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
        let fragments_dirty = self.fragments.iter().any(|f| f.is_dirty());

        if !self.is_dirty && !fragments_dirty {
            return;
        }

        self.ensure_initialized();

        if let Some(ref mut scene) = self.vello_scene {
            scene.reset();
        }

        for fragment in &self.fragments {
            // Build and compute layout
            let mut layout = build_layout_tree(&fragment.component);
            if layout.is_dirty() {
                if let Err(e) = layout.calculate_layout() {
                    eprintln!("Scene layout calculation failed: {:?}", e);
                }
            }
            fragment.component.set_layout_node(layout.clone());

            // Get layout position for transform
            let layout_opt = fragment.component.layout_node();
            let transform = if let Some(layout) = layout_opt {
                if let Some(l) = layout.layout() {
                    Affine::translate((l.location.x as f64, l.location.y as f64))
                } else {
                    Affine::IDENTITY
                }
            } else {
                Affine::IDENTITY
            };

            if let Some(ref mut scene) = self.vello_scene {
                fragment.generate_scene_items(scene, transform);
            }
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
