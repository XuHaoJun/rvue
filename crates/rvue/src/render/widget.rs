//! Widget-to-Vello mapping

use crate::component::{Component, ComponentProps, ComponentType, SceneWrapper};
use crate::style::{resolve_styles_for_component, Stylesheet};
use crate::text::{BrushIndex, ParleyLayoutWrapper};
use crate::widgets::scroll_bar::{render_horizontal_scrollbar, render_vertical_scrollbar};
use parley::Layout;
use rudo_gc::Gc;
use rustc_hash::FxHashSet;
use rvue_style::{BorderStyle, ComputedStyles};
use vello::kurbo::{Affine, Circle, Rect, RoundedRect, Stroke};
use vello::peniko::Color;

/// Scroll state for Flex widgets with overflow
#[derive(Clone, Copy, Debug, Default)]
pub struct FlexScrollState {
    pub scroll_offset_x: f32,
    pub scroll_offset_y: f32,
    pub scroll_width: f32,
    pub scroll_height: f32,
    pub container_width: f32,
    pub container_height: f32,
}

impl FlexScrollState {
    /// Check if horizontal scrollbar should be visible
    pub fn should_show_horizontal_scrollbar(
        &self,
        overflow_y: rvue_style::properties::Overflow,
    ) -> bool {
        !matches!(overflow_y, rvue_style::properties::Overflow::Visible)
            && (matches!(overflow_y, rvue_style::properties::Overflow::Scroll)
                || self.scroll_width > 0.0)
    }

    /// Check if vertical scrollbar should be visible
    pub fn should_show_vertical_scrollbar(
        &self,
        overflow_x: rvue_style::properties::Overflow,
    ) -> bool {
        !matches!(overflow_x, rvue_style::properties::Overflow::Visible)
            && (matches!(overflow_x, rvue_style::properties::Overflow::Scroll)
                || self.scroll_height > 0.0)
    }
}

fn get_or_create_scroll_state(component: &Gc<Component>) -> FlexScrollState {
    let mut user_data = component.user_data.borrow_mut();
    if let Some(state) = user_data.as_mut().and_then(|d| d.downcast_mut::<FlexScrollState>()) {
        return *state;
    }
    FlexScrollState::default()
}

pub fn render_component(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    already_appended: &mut FxHashSet<u64>,
    stylesheet: Option<&Stylesheet>,
) -> bool {
    let is_dirty = component.is_dirty();
    let cache_was_none = component.vello_cache.borrow().is_none();

    if is_dirty || cache_was_none {
        let mut local_scene = vello::Scene::new();

        match &component.component_type {
            ComponentType::Text => {
                render_text(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::Button => {
                render_button(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::TextInput => {
                render_text_input(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::NumberInput => {
                render_number_input(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::Checkbox => {
                render_checkbox(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::Radio => {
                render_radio(component, &mut local_scene, Affine::IDENTITY, stylesheet);
            }
            ComponentType::Flex => {
                render_flex_background(component, &mut local_scene, stylesheet);
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
        render_children(
            component,
            scene,
            transform,
            already_appended,
            force_render_children,
            stylesheet,
        );
    }

    is_dirty || cache_was_none
}

fn render_children(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    already_appended: &mut FxHashSet<u64>,
    force_render_children: bool,
    stylesheet: Option<&Stylesheet>,
) {
    // Check if we need to apply scroll offset
    let styles = get_styles(component, stylesheet);
    let overflow_x = styles.overflow_x.unwrap_or(rvue_style::properties::Overflow::Visible);
    let overflow_y = styles.overflow_y.unwrap_or(rvue_style::properties::Overflow::Visible);

    let should_clip = overflow_x.should_clip() || overflow_y.should_clip();

    let scroll_offset = if should_clip {
        let mut user_data = component.user_data.borrow_mut();
        if let Some(scroll_state) =
            user_data.as_mut().and_then(|d| d.downcast_mut::<FlexScrollState>())
        {
            let offset_x = scroll_state.scroll_offset_x;
            let offset_y = scroll_state.scroll_offset_y;
            if offset_x != 0.0 || offset_y != 0.0 {
                Some((offset_x as f64, offset_y as f64))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Get container layout for clipping
    let container_rect = if should_clip {
        component.layout_node().and_then(|ln| {
            ln.layout().map(|layout| {
                Rect::new(
                    layout.location.x as f64,
                    layout.location.y as f64,
                    (layout.location.x + layout.size.width) as f64,
                    (layout.location.y + layout.size.height) as f64,
                )
            })
        })
    } else {
        None
    };

    // Push clip layer if needed
    if should_clip {
        if let Some(ref rect) = container_rect {
            scene.push_clip_layer(vello::peniko::Fill::NonZero, Affine::IDENTITY, rect);
        }
    }

    for child in component.children.borrow().iter() {
        let child_transform = if let Some(layout_node) = child.layout_node() {
            if let Some(layout) = layout_node.layout() {
                let tx = layout.location.x as f64;
                let ty = layout.location.y as f64;
                // Apply scroll offset if needed
                if let Some((sx, sy)) = scroll_offset {
                    Affine::translate((tx - sx, ty - sy))
                } else {
                    Affine::translate((tx, ty))
                }
            } else {
                Affine::IDENTITY
            }
        } else {
            Affine::IDENTITY
        };

        let is_dirty = child.is_dirty();
        let cache_was_none = child.vello_cache.borrow().is_none();

        if force_render_children || is_dirty || cache_was_none {
            render_component(
                child,
                scene,
                transform * child_transform,
                already_appended,
                stylesheet,
            );
        }
    }

    // Pop clip layer if it was pushed
    if should_clip && container_rect.is_some() {
        scene.pop_layer();
    }
}

fn get_styles(component: &Gc<Component>, stylesheet: Option<&Stylesheet>) -> ComputedStyles {
    let result = match stylesheet {
        Some(sheet) => {
            // Use resolve_styles_for_component which is the unified resolution path
            // This ensures layout and rendering use the same style resolution
            resolve_styles_for_component(component, sheet)
        }
        None => {
            let props = component.props.borrow();
            match &*props {
                ComponentProps::Text { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::Button { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::TextInput { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::NumberInput { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::Checkbox { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::Radio { styles, .. } => styles.clone().unwrap_or_default(),
                ComponentProps::Flex { styles, .. } => styles.clone().unwrap_or_default(),
                _ => ComputedStyles::default(),
            }
        }
    };
    result
}

fn render_text(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Text { content: _, styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let user_data = component.user_data.borrow();
        let layout_wrapper =
            user_data.as_ref().and_then(|d| d.downcast_ref::<ParleyLayoutWrapper>());

        if let Some(ParleyLayoutWrapper(layout)) = layout_wrapper {
            let brush = styles
                .text_color
                .as_ref()
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

fn render_button(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Button { styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(button_layout) = layout.layout() {
                let width = button_layout.size.width as f64;
                let height = button_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .background_color
                    .as_ref()
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(70, 130, 180));

                let border_radius =
                    styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                render_border(
                    scene,
                    transform,
                    &Some(styles),
                    0.0,
                    0.0,
                    width,
                    height,
                    border_radius,
                );
            }
        }
    }
}

fn render_text_input(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::TextInput { value: _, styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(input_layout) = layout.layout() {
                let width = input_layout.size.width as f64;
                let height = input_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .background_color
                    .as_ref()
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius =
                    styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                render_border(
                    scene,
                    transform,
                    &Some(styles),
                    0.0,
                    0.0,
                    width,
                    height,
                    border_radius,
                );
            }
        }
    }
}

fn render_number_input(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::NumberInput { value: _, styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(input_layout) = layout.layout() {
                let width = input_layout.size.width as f64;
                let height = input_layout.size.height as f64;
                let _rect = Rect::new(0.0, 0.0, width, height);

                let bg_color = styles
                    .background_color
                    .as_ref()
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius =
                    styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

                let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                render_border(
                    scene,
                    transform,
                    &Some(styles),
                    0.0,
                    0.0,
                    width,
                    height,
                    border_radius,
                );
            }
        }
    }
}

fn render_checkbox(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Checkbox { checked: _, styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(checkbox_layout) = layout.layout() {
                let width = checkbox_layout.size.width as f64;
                let height = checkbox_layout.size.height as f64;

                let bg_color = styles
                    .background_color
                    .as_ref()
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let border_radius =
                    styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

                let _rect = Rect::new(0.0, 0.0, width.min(height), height.min(width));
                let size = width.min(height);
                let rounded_rect = RoundedRect::new(0.0, 0.0, size, size, border_radius);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

                render_border(scene, transform, &Some(styles), 0.0, 0.0, size, size, border_radius);
            }
        }
    }
}

fn render_radio(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Radio { value: _, checked: _, styles: _ } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(radio_layout) = layout.layout() {
                let size = radio_layout.size.width.min(radio_layout.size.height) as f64;
                let center_x = radio_layout.size.width / 2.0;
                let center_y = radio_layout.size.height / 2.0;

                let bg_color = styles
                    .background_color
                    .as_ref()
                    .map(|bg| {
                        let rgb = bg.0 .0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

                let circle = Circle::new((center_x, center_y), size / 2.0);
                scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &circle);

                let border_style =
                    styles.border_style.as_ref().copied().unwrap_or(BorderStyle::None);

                if border_style != BorderStyle::None {
                    let border_color = styles
                        .border_color
                        .as_ref()
                        .map(|border| {
                            let rgb = border.0 .0;
                            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                        })
                        .unwrap_or_else(|| Color::from_rgb8(100, 100, 100));

                    let border_width = styles.border_width.as_ref().map(|bw| bw.0).unwrap_or(1.0);

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

fn render_border(
    scene: &mut vello::Scene,
    transform: Affine,
    styles: &Option<ComputedStyles>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    border_radius: f64,
) {
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
                x + half_width,
                y + half_width,
                x + width - half_width,
                y + height - half_width,
                border_radius,
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

fn render_flex_background(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Flex { styles: _, .. } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(flex_layout) = layout.layout() {
                let _layout_x = flex_layout.location.x as f64;
                let _layout_y = flex_layout.location.y as f64;
                let width = flex_layout.size.width as f64;
                let height = flex_layout.size.height as f64;

                // Render background at LOCAL (0,0) - transform will position it correctly
                // This fixes the coordinate mismatch where background was at layout.location
                // but children use transforms
                let rect = Rect::new(0.0, 0.0, width, height);

                if let Some(bg) = styles.background_color.as_ref() {
                    let rgb = bg.0 .0;
                    let bg_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                    scene.fill(
                        vello::peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        bg_color,
                        None,
                        &rect,
                    );
                }

                let border_radius =
                    styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(0.0);

                render_border(
                    scene,
                    Affine::IDENTITY,
                    &Some(styles.clone()),
                    0.0,
                    0.0,
                    width,
                    height,
                    border_radius,
                );

                // Render scrollbars if overflow is set
                let overflow_x =
                    styles.overflow_x.unwrap_or(rvue_style::properties::Overflow::Visible);
                let overflow_y =
                    styles.overflow_y.unwrap_or(rvue_style::properties::Overflow::Visible);

                let should_clip = overflow_x.should_clip() || overflow_y.should_clip();

                if should_clip {
                    // Get scroll state from user_data
                    let scroll_state = get_or_create_scroll_state(component);

                    // Check if we need to show scrollbars
                    let show_vertical =
                        matches!(overflow_y, rvue_style::properties::Overflow::Scroll)
                            || (matches!(overflow_y, rvue_style::properties::Overflow::Auto)
                                && scroll_state.scroll_height > 0.0);
                    let show_horizontal =
                        matches!(overflow_x, rvue_style::properties::Overflow::Scroll)
                            || (matches!(overflow_x, rvue_style::properties::Overflow::Auto)
                                && scroll_state.scroll_width > 0.0);

                    // Render vertical scrollbar if needed
                    if show_vertical {
                        render_vertical_scrollbar(
                            scene,
                            0.0,
                            0.0,
                            width,
                            height,
                            scroll_state.scroll_offset_y as f64,
                            scroll_state.scroll_height as f64,
                        );
                    }

                    // Render horizontal scrollbar if needed
                    if show_horizontal {
                        render_horizontal_scrollbar(
                            scene,
                            0.0,
                            0.0,
                            width,
                            height,
                            scroll_state.scroll_offset_x as f64,
                            scroll_state.scroll_width as f64,
                        );
                    }
                }
            }
        }
    }
}
