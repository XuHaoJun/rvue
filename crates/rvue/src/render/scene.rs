//! Scene graph management for Vello rendering

use crate::component::build_layout_tree;
use crate::component::Component;
use crate::render::widget::render_component;
use crate::style::Stylesheet;
use crate::text::TextContext;
use rudo_gc::Gc;
use rustc_hash::FxHashSet;
use taffy::prelude::*;
use taffy::TaffyTree;
use vello::kurbo::Affine;

/// Scene structure for managing Vello rendering
pub struct Scene {
    pub vello_scene: Option<vello::Scene>,
    pub root_components: Vec<Gc<Component>>,
    pub is_dirty: bool,
    pub renderer_initialized: bool,
    pub taffy: TaffyTree<()>,
    pub text_context: TextContext,
    pub stylesheet: Option<Stylesheet>,
}

impl Scene {
    /// Create a new scene with lazy renderer initialization
    pub fn new() -> Self {
        Self {
            vello_scene: None,
            root_components: Vec::new(),
            is_dirty: true,
            renderer_initialized: false,
            taffy: TaffyTree::new(),
            text_context: TextContext::new(),
            stylesheet: None,
        }
    }

    /// Set the stylesheet for this scene
    pub fn set_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.stylesheet = Some(stylesheet);
        self.is_dirty = true;
    }

    /// Get a reference to the stylesheet
    pub fn stylesheet(&self) -> Option<&Stylesheet> {
        self.stylesheet.as_ref()
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

        if !self.is_dirty && !any_dirty {
            return;
        }

        self.ensure_initialized();

        if self.is_dirty {
            if let Some(ref mut scene) = self.vello_scene {
                scene.reset();
            }
            self.taffy.clear();
        }

        let mut already_appended = FxHashSet::default();

        for component in &self.root_components {
            crate::effect::set_defer_effect_run(true);

            let layout = build_layout_tree(component, &mut self.taffy, &mut self.text_context);
            component.set_layout_node(layout.clone());

            crate::effect::flush_pending_effects();
            crate::effect::set_defer_effect_run(false);

            if let Some(node_id) = layout.taffy_node() {
                if let Err(e) = self.taffy.compute_layout(node_id, Size::MAX_CONTENT) {
                    eprintln!("Scene layout calculation failed: {:?}", e);
                }
            }

            crate::component::propagate_layout_results(component, &self.taffy);

            if let Some(ref mut scene) = self.vello_scene {
                if component.is_dirty() {
                    *component.vello_cache.borrow_mut() = None;
                    render_component(
                        component,
                        scene,
                        Affine::IDENTITY,
                        &mut already_appended,
                        self.stylesheet.as_ref(),
                    );
                } else if let Some(ref cached) = *component.vello_cache.borrow() {
                    scene.append(&cached.0, Some(Affine::IDENTITY));
                } else {
                    render_component(
                        component,
                        scene,
                        Affine::IDENTITY,
                        &mut already_appended,
                        self.stylesheet.as_ref(),
                    );
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
