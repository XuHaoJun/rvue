//! Widget-to-Vello mapping

use crate::component::{Component, ComponentProps, ComponentType, SceneWrapper};
use crate::text::{BrushIndex, ParleyLayoutWrapper};
use parley::Layout;
use rudo_gc::Gc;
use vello::kurbo::{Affine, Circle, Rect, RoundedRect};
use vello::peniko::Color;

pub fn render_component(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
) -> bool {
    let is_dirty = component.is_dirty();
    let cache_was_none = component.vello_cache.borrow().is_none();

    if is_dirty || cache_was_none {
        let mut local_scene = vello::Scene::new();

        match &component.component_type {
            ComponentType::Text => {
                render_text(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::Button => {
                render_button(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::TextInput => {
                render_text_input(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::NumberInput => {
                render_number_input(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::Checkbox => {
                render_checkbox(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::Radio => {
                render_radio(component, &mut local_scene, Affine::IDENTITY);
            }
            ComponentType::Flex => {
                render_flex_background(component, &mut local_scene, Affine::IDENTITY);
            }
            _ => {}
        }

        *component.vello_cache.borrow_mut() = Some(SceneWrapper(local_scene));
        component.clear_dirty();
    }

    if let Some(SceneWrapper(ref local_scene)) = *component.vello_cache.borrow() {
        scene.append(local_scene, Some(transform));
    }

    let should_render_children = match &component.component_type {
        ComponentType::Show => {
            if let ComponentProps::Show { when } = &*component.props.borrow() {
                *when
            } else {
                false
            }
        }
        ComponentType::For => true,
        ComponentType::Flex => true,
        _ => !component.children.borrow().is_empty(),
    };

    if should_render_children {
        render_children(component, scene, transform);
    }

    is_dirty || cache_was_none
}

fn render_children(component: &Gc<Component>, scene: &mut vello::Scene, transform: Affine) {
    for child in component.children.borrow().iter() {
        let child_transform = if let Some(layout_node) = child.layout_node() {
            if let Some(layout) = layout_node.layout() {
                Affine::translate((layout.location.x as f64, layout.location.y as f64))
            } else {
                Affine::IDENTITY
            }
        } else {
            Affine::IDENTITY
        };
        *child.vello_cache.borrow_mut() = None;
        child.is_dirty.store(true, std::sync::atomic::Ordering::SeqCst);
        render_component(child, scene, transform * child_transform);
    }
}

fn render_text(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Text { content: _, font_size: _, color } = &*component.props.borrow() {
        let user_data = component.user_data.borrow();
        let layout_wrapper =
            user_data.as_ref().and_then(|d| d.downcast_ref::<ParleyLayoutWrapper>());

        if let Some(ParleyLayoutWrapper(layout)) = layout_wrapper {
            let brush = color.unwrap_or(Color::BLACK);
            if layout.lines().next().is_none() {
                let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
                let bg_color = Color::from_rgb8(255, 165, 0);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
            } else {
                render_text_layout(layout, scene, transform, brush);
            }
        } else if user_data.is_some() {
            let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
            let bg_color = Color::from_rgb8(255, 0, 0);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        } else {
            let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
            let bg_color = Color::from_rgba8(200, 200, 200, 255);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
        }
    }
}

fn render_text_layout(
    layout: &Layout<BrushIndex>,
    scene: &mut vello::Scene,
    transform: Affine,
    color: Color,
) {
    use parley::PositionedLayoutItem;
    use vello::peniko::Fill;

    let fill = Fill::NonZero;

    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();

                let synthesis = run.synthesis();
                let glyph_xform = synthesis
                    .skew()
                    .map(|angle| vello::kurbo::Affine::skew(angle.to_radians().tan() as f64, 0.0));
                let coords = run.normalized_coords();

                let vello_glyphs: Vec<vello::Glyph> = glyph_run
                    .glyphs()
                    .map(|g| {
                        let gx = x + g.x;
                        let gy = y - g.y;
                        x += g.advance;
                        vello::Glyph { id: g.id, x: gx, y: gy }
                    })
                    .collect();

                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .transform(transform)
                    .glyph_transform(glyph_xform)
                    .normalized_coords(coords)
                    .brush(color)
                    .hint(true)
                    .draw(fill, vello_glyphs.into_iter());
            }
        }
    }
}

fn render_button(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Button { label: _ } = &*component.props.borrow() {
        let rect = RoundedRect::new(0.0, 0.0, 120.0, 40.0, 4.0);
        let bg_color = Color::from_rgb8(70, 130, 180);
        scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
    }
}

fn render_flex_background(component: &Component, scene: &mut vello::Scene, transform: Affine) {
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
        }
    }
}

fn render_text_input(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::TextInput { value: _ } = &*component.props.borrow() {
        let rect = RoundedRect::new(0.0, 0.0, 200.0, 30.0, 2.0);
        let bg_color = Color::from_rgb8(255, 255, 255);
        scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
    }
}

fn render_number_input(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::NumberInput { value: _ } = &*component.props.borrow() {
        let rect = RoundedRect::new(0.0, 0.0, 150.0, 30.0, 2.0);
        let bg_color = Color::from_rgb8(255, 255, 255);
        scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
    }
}

fn render_checkbox(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Checkbox { checked } = &*component.props.borrow() {
        let rect = Rect::new(0.0, 0.0, 20.0, 20.0);
        let bg_color =
            if *checked { Color::from_rgb8(70, 130, 180) } else { Color::from_rgb8(255, 255, 255) };
        scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
    }
}

fn render_radio(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Radio { value: _, checked } = &*component.props.borrow() {
        let circle = Circle::new((10.0, 10.0), 10.0);
        let bg_color =
            if *checked { Color::from_rgb8(70, 130, 180) } else { Color::from_rgb8(255, 255, 255) };
        scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &circle);
        if *checked {
            let inner_circle = Circle::new((10.0, 10.0), 5.0);
            let dot_color = Color::from_rgb8(255, 255, 255);
            scene.fill(vello::peniko::Fill::NonZero, transform, dot_color, None, &inner_circle);
        }
    }
}
