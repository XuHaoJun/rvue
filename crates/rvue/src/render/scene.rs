//! Scene graph management for Vello rendering

use crate::component::build_layout_tree;
use crate::component::Component;
use crate::render::widget::render_component;
use crate::text::TextContext;
use rudo_gc::Gc;
use taffy::prelude::*;
use taffy::TaffyTree;
use vello::kurbo::Affine;

/// Scene structure for managing Vello rendering
pub struct Scene {
    pub vello_scene: Option<vello::Scene>, // Lazy initialization
    pub root_components: Vec<Gc<Component>>,
    pub is_dirty: bool,
    pub renderer_initialized: bool,
    pub taffy: TaffyTree<()>,
    pub text_context: TextContext,
}

impl Scene {
    /// Create a new scene with lazy renderer initialization
    pub fn new() -> Self {
        Self {
            vello_scene: None, // Defer scene creation until first render
            root_components: Vec::new(),
            is_dirty: true,
            renderer_initialized: false,
            taffy: TaffyTree::new(),
            text_context: TextContext::new(),
        }
    }

    /// Initialize the Vello scene lazily (only when needed)
    fn ensure_initialized(&mut self) {
        if self.vello_scene.is_none() {
            self.vello_scene = Some(vello::Scene::new());
            self.renderer_initialized = true;
        }
    }

    /// Add a root component to the scene
    pub fn add_fragment(&mut self, component: Gc<Component>) {
        self.root_components.push(component);
        self.is_dirty = true;
    }

    /// Update the scene by regenerating dirty fragments
    pub fn update(&mut self) {
        let any_dirty = self.root_components.iter().any(|c| c.is_dirty());

        if !self.is_dirty && !any_dirty {
            return;
        }

        self.ensure_initialized();

        // Only reset scene if structural changes (new/removed components)
        // Per-component dirty state only requires re-appending cached fragments
        if self.is_dirty {
            if let Some(ref mut scene) = self.vello_scene {
                scene.reset();
            }
        }

        for component in &self.root_components {
            // Set parent to None so dirty propagation works correctly
            // (root components don't have a parent, but this allows propagation
            // from children to reach the root)
            component.set_parent(None);

            // 1. Layout Pass (shared tree) - needed for positioning of all components
            let layout = build_layout_tree(component, &mut self.taffy, &mut self.text_context);
            component.set_layout_node(layout.clone());
            if let Some(node_id) = layout.taffy_node() {
                if let Err(e) = self.taffy.compute_layout(node_id, Size::MAX_CONTENT) {
                    eprintln!("Scene layout calculation failed: {:?}", e);
                }
            }

            // Propagate results back
            crate::component::propagate_layout_results(component, &self.taffy);

            // 2. Render Pass - only re-render dirty components
            if component.is_dirty() {
                *component.vello_cache.borrow_mut() = None;
                if let Some(ref mut scene) = self.vello_scene {
                    render_component(component, scene, Affine::IDENTITY);
                }
            } else if let Some(ref cached) = *component.vello_cache.borrow() {
                // Append cached fragment - children are already encoded inside
                if let Some(ref mut scene) = self.vello_scene {
                    scene.append(&cached.0, Some(Affine::IDENTITY));
                }
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
    pub fn vello_scene(&mut self) -> &vello::Scene {
        self.ensure_initialized();
        self.vello_scene.as_ref().unwrap()
    }

    /// Get a mutable reference to the underlying Vello scene
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
