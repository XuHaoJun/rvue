//! Widget-to-Vello mapping

use crate::component::build_layout_tree;
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
    pub fn new(component: Gc<Component>) -> Self {
        Self { component, cached_scene: GcCell::new(None) }
    }

    pub fn mark_dirty(&self) {
        self.component.mark_dirty();
        *self.cached_scene.borrow_mut() = None;
    }

    pub fn is_dirty(&self) -> bool {
        self.component.is_dirty() || self.cached_scene.borrow().is_none()
    }

    pub fn generate_scene_items(&self, scene: &mut vello::Scene, transform: vello::kurbo::Affine) {
        if self.is_dirty() {
            // Build and compute layout if dirty
            let mut layout = build_layout_tree(&self.component);
            if layout.is_dirty() {
                if let Err(e) = layout.calculate_layout() {
                    eprintln!("Layout calculation failed: {:?}", e);
                }
            }
            self.component.set_layout_node(layout);

            let mut sub_scene = vello::Scene::new();

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
                        Gc::clone(&self.component),
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::For => {
                    Self::render_for(
                        Gc::clone(&self.component),
                        &mut sub_scene,
                        vello::kurbo::Affine::IDENTITY,
                    );
                }
                ComponentType::Flex => {
                    Self::render_flex(
                        Gc::clone(&self.component),
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

            *self.cached_scene.borrow_mut() = Some(sub_scene);
            self.component.clear_dirty();
        }

        if let Some(ref sub_scene) = *self.cached_scene.borrow() {
            scene.append(sub_scene, Some(transform));
        }
    }

    fn render_text(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Text { content: _ } = &*component.props.borrow() {
            let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
            let bg_color = Color::from_rgba8(200, 200, 200, 255);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    fn render_button(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Button { label: _ } = &*component.props.borrow() {
            let rect = RoundedRect::new(0.0, 0.0, 120.0, 40.0, 4.0);
            let bg_color = Color::from_rgb8(70, 130, 180);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    fn render_show(
        component: Gc<Component>,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Show { when } = &*component.props.borrow() {
            if *when {
                for child in component.children.borrow().iter() {
                    let child_fragment = VelloFragment::new(Gc::clone(child));
                    child_fragment.generate_scene_items(scene, transform);
                }
            }
        }
    }

    fn render_for(
        component: Gc<Component>,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::For { item_count: _ } = &*component.props.borrow() {
            for child in component.children.borrow().iter() {
                let child_fragment = VelloFragment::new(Gc::clone(child));
                child_fragment.generate_scene_items(scene, transform);
            }
        }
    }

    fn render_flex(
        component: Gc<Component>,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Flex { .. } = &*component.props.borrow() {
            let layout_node = component.layout_node();

            if let Some(layout) = layout_node {
                if let Some(flex_layout) = layout.layout() {
                    let rect = Rect::new(
                        0.0,
                        0.0,
                        flex_layout.size.width as f64,
                        flex_layout.size.height as f64,
                    );
                    let bg_color = Color::from_rgba8(245, 245, 245, 255);
                    scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
                }

                let taffy = layout.taffy();
                if let Some(taffy_node) = layout.taffy_node() {
                    let child_nodes: Vec<_> = match taffy.children(taffy_node) {
                        Ok(nodes) => nodes,
                        Err(e) => {
                            eprintln!("Failed to get children: {:?}", e);
                            Vec::new()
                        }
                    };

                    for (index, child) in component.children.borrow().iter().enumerate() {
                        if index < child_nodes.len() {
                            let child_taffy_node = child_nodes[index];
                            if let Ok(child_layout) = taffy.layout(child_taffy_node) {
                                let child_transform = vello::kurbo::Affine::translate((
                                    child_layout.location.x as f64,
                                    child_layout.location.y as f64,
                                ));
                                let child_fragment = VelloFragment::new(Gc::clone(child));
                                child_fragment
                                    .generate_scene_items(scene, transform * child_transform);
                            }
                        } else {
                            let child_fragment = VelloFragment::new(Gc::clone(child));
                            child_fragment.generate_scene_items(scene, transform);
                        }
                    }
                }
            } else {
                for child in component.children.borrow().iter() {
                    let child_fragment = VelloFragment::new(Gc::clone(child));
                    child_fragment.generate_scene_items(scene, transform);
                }
            }
        }
    }

    fn render_text_input(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::TextInput { value: _ } = &*component.props.borrow() {
            let rect = RoundedRect::new(0.0, 0.0, 200.0, 30.0, 2.0);
            let bg_color = Color::from_rgb8(255, 255, 255);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    fn render_number_input(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::NumberInput { value: _ } = &*component.props.borrow() {
            let rect = RoundedRect::new(0.0, 0.0, 150.0, 30.0, 2.0);
            let bg_color = Color::from_rgb8(255, 255, 255);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    fn render_checkbox(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Checkbox { checked } = &*component.props.borrow() {
            let rect = Rect::new(0.0, 0.0, 20.0, 20.0);
            let bg_color = if *checked {
                Color::from_rgb8(70, 130, 180)
            } else {
                Color::from_rgb8(255, 255, 255)
            };
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }

    fn render_radio(
        component: &Component,
        scene: &mut vello::Scene,
        transform: vello::kurbo::Affine,
    ) {
        if let ComponentProps::Radio { value: _, checked } = &*component.props.borrow() {
            let circle = Circle::new((10.0, 10.0), 10.0);
            let bg_color = if *checked {
                Color::from_rgb8(70, 130, 180)
            } else {
                Color::from_rgb8(255, 255, 255)
            };
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &circle);
            if *checked {
                let inner_circle = Circle::new((10.0, 10.0), 5.0);
                let dot_color = Color::from_rgb8(255, 255, 255);
                scene.fill(vello::peniko::Fill::NonZero, transform, dot_color, None, &inner_circle);
            }
        }
    }
}
