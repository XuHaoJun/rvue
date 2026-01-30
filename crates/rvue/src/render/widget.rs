//! Widget-to-Vello mapping

use crate::component::{Component, ComponentProps, ComponentType, SceneWrapper};
use crate::text::{BrushIndex, ParleyLayoutWrapper};
use parley::Layout;
use rudo_gc::Gc;
use rustc_hash::FxHashSet;
use rvue_style::BorderStyle;
use vello::kurbo::{Affine, Circle, Rect, RoundedRect, Stroke};
use vello::peniko::Color;

pub fn render_component(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    already_appended: &mut FxHashSet<u64>,
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

    if !already_appended.contains(&component.id) {
        if let Some(SceneWrapper(ref local_scene)) = *component.vello_cache.borrow() {
            scene.append(local_scene, Some(transform));
            already_appended.insert(component.id);
        }
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

    let force_render_children = matches!(
        &component.component_type,
        ComponentType::For | ComponentType::Flex | ComponentType::Show
    );

    if should_render_children {
        render_children(component, scene, transform, already_appended, force_render_children);
    }

    is_dirty || cache_was_none
}

fn render_children(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    already_appended: &mut FxHashSet<u64>,
    force_render_children: bool,
) {
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

        let is_dirty = child.is_dirty();
        let cache_was_none = child.vello_cache.borrow().is_none();

        if force_render_children || is_dirty || cache_was_none {
            render_component(child, scene, transform * child_transform, already_appended);
        }
    }
}

fn render_text(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Text { content: _, styles } = &*component.props.borrow() {
        let user_data = component.user_data.borrow();
        let layout_wrapper =
            user_data.as_ref().and_then(|d| d.downcast_ref::<ParleyLayoutWrapper>());

        if let Some(ParleyLayoutWrapper(layout)) = layout_wrapper {
            let brush = styles
                .as_ref()
                .and_then(|s| s.text_color.as_ref())
                .map(|tc| {
                    let rgb = tc.0 .0;
                    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                })
                .unwrap_or(Color::BLACK);

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
    if let ComponentProps::Button { label: _, styles } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(button_layout) = layout.layout() {
                let width = button_layout.size.width as f64;
                let height = button_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .as_ref()
                    .and_then(|s| s.background_color.as_ref())
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(70, 130, 180));

                let border_radius = styles
                    .as_ref()
                    .and_then(|s| s.border_radius.as_ref())
                    .map(|r| r.0 as f64)
                    .unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                if let (Some(border), Some(bw), Some(bs)) = (
                    styles.as_ref().and_then(|s| s.border_color.as_ref()),
                    styles.as_ref().and_then(|s| s.border_width.as_ref()),
                    styles.as_ref().and_then(|s| s.border_style.as_ref()),
                ) {
                    if *bs != BorderStyle::None {
                        let rgb = border.0 .0;
                        let border_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                        let border_width = bw.0;

                        let half_width = border_width as f64 / 2.0;
                        let inner_rect = RoundedRect::new(
                            half_width,
                            half_width,
                            width - half_width,
                            height - half_width,
                            border_radius,
                        );
                        scene.stroke(
                            &Stroke::new(border_width as f64),
                            transform,
                            border_color,
                            None,
                            &inner_rect,
                        );
                    }
                }
            }
        }
    }
}

fn render_text_input(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::TextInput { value: _, styles } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(input_layout) = layout.layout() {
                let width = input_layout.size.width as f64;
                let height = input_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .as_ref()
                    .and_then(|s| s.background_color.as_ref())
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius = styles
                    .as_ref()
                    .and_then(|s| s.border_radius.as_ref())
                    .map(|r| r.0 as f64)
                    .unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                if let (Some(border), Some(bw), Some(bs)) = (
                    styles.as_ref().and_then(|s| s.border_color.as_ref()),
                    styles.as_ref().and_then(|s| s.border_width.as_ref()),
                    styles.as_ref().and_then(|s| s.border_style.as_ref()),
                ) {
                    if *bs != BorderStyle::None {
                        let rgb = border.0 .0;
                        let border_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                        let border_width = bw.0;

                        let half_width = border_width as f64 / 2.0;
                        let inner_rect = RoundedRect::new(
                            half_width,
                            half_width,
                            width - half_width,
                            height - half_width,
                            border_radius,
                        );
                        scene.stroke(
                            &Stroke::new(border_width as f64),
                            transform,
                            border_color,
                            None,
                            &inner_rect,
                        );
                    }
                }
            }
        }
    }
}

fn render_number_input(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::NumberInput { value: _, styles } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(input_layout) = layout.layout() {
                let width = input_layout.size.width as f64;
                let height = input_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .as_ref()
                    .and_then(|s| s.background_color.as_ref())
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius = styles
                    .as_ref()
                    .and_then(|s| s.border_radius.as_ref())
                    .map(|r| r.0 as f64)
                    .unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                if let (Some(border), Some(bw), Some(bs)) = (
                    styles.as_ref().and_then(|s| s.border_color.as_ref()),
                    styles.as_ref().and_then(|s| s.border_width.as_ref()),
                    styles.as_ref().and_then(|s| s.border_style.as_ref()),
                ) {
                    if *bs != BorderStyle::None {
                        let rgb = border.0 .0;
                        let border_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                        let border_width = bw.0;

                        let half_width = border_width as f64 / 2.0;
                        let inner_rect = RoundedRect::new(
                            half_width,
                            half_width,
                            width - half_width,
                            height - half_width,
                            border_radius,
                        );
                        scene.stroke(
                            &Stroke::new(border_width as f64),
                            transform,
                            border_color,
                            None,
                            &inner_rect,
                        );
                    }
                }
            }
        }
    }
}

fn render_checkbox(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Checkbox { checked: _, styles } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(checkbox_layout) = layout.layout() {
                let width = checkbox_layout.size.width as f64;
                let height = checkbox_layout.size.height as f64;

                let bg_color = styles
                    .as_ref()
                    .and_then(|s| s.background_color.as_ref())
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius = styles
                    .as_ref()
                    .and_then(|s| s.border_radius.as_ref())
                    .map(|r| r.0 as f64)
                    .unwrap_or(4.0);

                let _rect = Rect::new(0.0, 0.0, width.min(height), height.min(width));
                let size = width.min(height);
                let rounded_rect = RoundedRect::new(0.0, 0.0, size, size, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                if let (Some(border), Some(bw), Some(bs)) = (
                    styles.as_ref().and_then(|s| s.border_color.as_ref()),
                    styles.as_ref().and_then(|s| s.border_width.as_ref()),
                    styles.as_ref().and_then(|s| s.border_style.as_ref()),
                ) {
                    if *bs != BorderStyle::None {
                        let rgb = border.0 .0;
                        let border_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                        let border_width = bw.0;

                        let half_width = border_width as f64 / 2.0;
                        let border_rect = RoundedRect::new(
                            half_width,
                            half_width,
                            size - half_width,
                            size - half_width,
                            border_radius,
                        );
                        scene.stroke(
                            &Stroke::new(border_width as f64),
                            transform,
                            border_color,
                            None,
                            &border_rect,
                        );
                    }
                }
            }
        }
    }
}

fn render_radio(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Radio { value: _, checked: _, styles } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(radio_layout) = layout.layout() {
                let size = radio_layout.size.width.min(radio_layout.size.height) as f64;
                let center_x = radio_layout.size.width / 2.0;
                let center_y = radio_layout.size.height / 2.0;

                let bg_color = styles
                    .as_ref()
                    .and_then(|s| s.background_color.as_ref())
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let circle = Circle::new((center_x, center_y), size / 2.0);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &circle);

                let border_style = styles
                    .as_ref()
                    .and_then(|s| s.border_style.as_ref())
                    .copied()
                    .unwrap_or(BorderStyle::None);

                if border_style != BorderStyle::None {
                    let border_color = styles
                        .as_ref()
                        .and_then(|s| s.border_color.as_ref())
                        .map(|border| {
                            let rgb = border.0 .0;
                            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                        })
                        .unwrap_or_else(|| Color::from_rgb8(100, 100, 100));

                    let border_width = styles
                        .as_ref()
                        .and_then(|s| s.border_width.as_ref())
                        .map(|bw| bw.0)
                        .unwrap_or(1.0);

                    let half_width = border_width as f64 / 2.0;
                    let border_circle = Circle::new((center_x, center_y), size / 2.0 - half_width);
                    scene.stroke(
                        &Stroke::new(border_width as f64),
                        transform,
                        border_color,
                        None,
                        &border_circle,
                    );
                }
            }
        }
    }
}

fn render_flex_background(component: &Component, scene: &mut vello::Scene, transform: Affine) {
    if let ComponentProps::Flex { styles, .. } = &*component.props.borrow() {
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(flex_layout) = layout.layout() {
                let width = flex_layout.size.width as f64;
                let height = flex_layout.size.height as f64;
                let rect = Rect::new(0.0, 0.0, width, height);

                if let Some(bg) = styles.as_ref().and_then(|s| s.background_color.as_ref()) {
                    let rgb = bg.0 .0;
                    let bg_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                    scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rect);
                }

                if let (Some(border), Some(bw), Some(bs)) = (
                    styles.as_ref().and_then(|s| s.border_color.as_ref()),
                    styles.as_ref().and_then(|s| s.border_width.as_ref()),
                    styles.as_ref().and_then(|s| s.border_style.as_ref()),
                ) {
                    if *bs != BorderStyle::None {
                        let rgb = border.0 .0;
                        let border_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                        let border_width = bw.0;

                        let half_width = border_width as f64 / 2.0;
                        let rounded_rect = RoundedRect::new(
                            half_width,
                            half_width,
                            width - half_width,
                            height - half_width,
                            0.0,
                        );
                        scene.stroke(
                            &Stroke::new(border_width as f64),
                            transform,
                            border_color,
                            None,
                            &rounded_rect,
                        );
                    }
                }
            }
        }
    }
}
