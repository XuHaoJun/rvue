//! Widget-to-Vello mapping

use crate::component::{Component, ComponentType, SceneWrapper};
use crate::style::{resolve_styles_for_component, Stylesheet};
use crate::text::{BrushIndex, ParleyLayoutWrapper};
use crate::widgets::scroll_bar::{render_horizontal_scrollbar, render_vertical_scrollbar};
use parley::Cluster;
use parley::FontStack;
use parley::Layout;
use parley::PositionedLayoutItem;
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
    let mut user_data = component.user_data.borrow_mut_gen_only();
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
    text_context: &mut crate::text::TextContext,
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
                render_text_input(
                    component,
                    &mut local_scene,
                    Affine::IDENTITY,
                    stylesheet,
                    text_context,
                );
            }
            ComponentType::NumberInput => {
                render_number_input(
                    component,
                    &mut local_scene,
                    Affine::IDENTITY,
                    stylesheet,
                    text_context,
                );
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

        *component.vello_cache.borrow_mut_gen_only() = Some(SceneWrapper(local_scene));
        component.clear_dirty();
    }

    if !already_appended.contains(&component.id) {
        if let Some(SceneWrapper(ref local_scene)) = *component.vello_cache.borrow() {
            scene.append(local_scene, Some(transform));
            already_appended.insert(component.id);
        }
    }

    let should_render_children = match &component.component_type {
        ComponentType::Show => component.show_when(),
        ComponentType::For => true,
        ComponentType::Flex => true,
        _ => !component.children.borrow().is_empty(),
    };

    let force_render_children = matches!(
        &component.component_type,
        ComponentType::For | ComponentType::Flex | ComponentType::Show
    );

    // Render children
    if should_render_children {
        render_children(
            component,
            scene,
            transform,
            already_appended,
            force_render_children,
            stylesheet,
            text_context,
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
    text_context: &mut crate::text::TextContext,
) {
    // Check if we need to clip content
    let styles = get_styles(component, stylesheet);
    let overflow_x = styles.overflow_x.unwrap_or(rvue_style::properties::Overflow::Visible);
    let overflow_y = styles.overflow_y.unwrap_or(rvue_style::properties::Overflow::Visible);

    let should_clip = overflow_x.should_clip() || overflow_y.should_clip();

    // Get scroll offset for this container
    let scroll_state = component.scroll_state();
    let scroll_offset_x = scroll_state.scroll_offset_x as f64;
    let scroll_offset_y = scroll_state.scroll_offset_y as f64;

    // Get container layout for clipping
    // Clip is in local coordinates relative to the container
    let container_rect = if should_clip {
        component.layout_node().and_then(|ln| {
            ln.layout().map(|layout| {
                Rect::new(0.0, 0.0, layout.size.width as f64, layout.size.height as f64)
            })
        })
    } else {
        None
    };

    // Push clip layer if needed
    // Clip stays with container - use transform for positioning
    if should_clip {
        if let Some(ref rect) = container_rect {
            scene.push_clip_layer(vello::peniko::Fill::NonZero, transform, rect);
        }
    }

    // Render children
    // Apply scroll offset to content (content moves opposite to scroll)
    for child in component.children.borrow().iter() {
        let child_transform = if let Some(layout_node) = child.layout_node() {
            if let Some(layout) = layout_node.layout() {
                let tx = layout.location.x as f64;
                let ty = layout.location.y as f64;
                // Subtract scroll offset to move content in opposite direction
                if scroll_offset_x != 0.0 || scroll_offset_y != 0.0 {
                    Affine::translate((tx - scroll_offset_x, ty - scroll_offset_y))
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

        let final_transform = transform * child_transform;

        if force_render_children || is_dirty || cache_was_none {
            render_component(
                child,
                scene,
                final_transform,
                already_appended,
                stylesheet,
                text_context,
            );
        }
    }

    // Pop clip layer if it was pushed
    if should_clip && container_rect.is_some() {
        scene.pop_layer();
    }
}

fn get_styles(component: &Gc<Component>, stylesheet: Option<&Stylesheet>) -> ComputedStyles {
    match stylesheet {
        Some(sheet) => resolve_styles_for_component(component, sheet),
        None => component.widget_styles().unwrap_or_default(),
    }
}

fn render_text(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
) {
    let styles = get_styles(component, stylesheet);
    let user_data = component.user_data.borrow();
    let layout_wrapper = user_data.as_ref().and_then(|d| d.downcast_ref::<ParleyLayoutWrapper>());

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

            let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

            let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

            render_border(scene, transform, &Some(styles), 0.0, 0.0, width, height, border_radius);
        }
    }
}

fn render_text_input(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
    text_context: &mut crate::text::TextContext,
) {
    let styles = get_styles(component, stylesheet);
    let layout_node = component.layout_node();

    let text_color = styles
        .text_color
        .as_ref()
        .map(|tc| {
            let rgb = tc.0 .0;
            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
        })
        .unwrap_or(Color::BLACK);

    let font_size = styles.font_size.as_ref().map(|fs| fs.0 as f64).unwrap_or(14.0);
    let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

    if let Some(layout) = layout_node {
        if let Some(input_layout) = layout.layout() {
            let width = input_layout.size.width as f64;
            let height = input_layout.size.height as f64;

            let bg_color = styles
                .background_color
                .as_ref()
                .map(|bg| {
                    let rgb = bg.0 .0;
                    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                })
                .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

            let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

            render_border(scene, transform, &Some(styles), 0.0, 0.0, width, height, border_radius);

            let clip = *component.clip.borrow();
            let clip_rect = Rect::new(0.0, 0.0, width, height);

            if clip {
                scene.push_clip_layer(vello::peniko::Fill::NonZero, transform, &clip_rect);
            }

            let (text_value, composition_range, is_composing) =
                if let Some(editor) = component.text_editor() {
                    let editor_ref = editor.editor();
                    let content = editor_ref.content();
                    let composition = editor_ref.composition();
                    if composition.is_empty() {
                        (content, None, false)
                    } else {
                        let cursor_offset = content.chars().count().min(composition.cursor_start);
                        let left: String = content.chars().take(cursor_offset).collect();
                        let right: String = content.chars().skip(cursor_offset).collect();
                        let full_text = format!("{}{}{}", left, composition.text, right);
                        let composition_start = left.chars().count();
                        let composition_end = composition_start + composition.text.chars().count();
                        (full_text, Some((composition_start, composition_end)), true)
                    }
                } else {
                    (component.text_input_value(), None, false)
                };

            let mut layout_builder = text_context.layout_ctx.ranged_builder(
                &mut text_context.font_ctx,
                &text_value,
                1.0,
                true,
            );
            layout_builder.push_default(parley::style::StyleProperty::FontSize(font_size as f32));
            layout_builder.push_default(parley::style::StyleProperty::Brush(BrushIndex(0)));
            layout_builder.push_default(parley::style::FontStack::Source(
                std::borrow::Cow::Borrowed("sans-serif"),
            ));

            let mut text_layout: Layout<BrushIndex> = layout_builder.build(&text_value);
            text_layout.break_all_lines(None);

            if !text_value.is_empty() {
                render_text_layout(&text_layout, scene, transform, text_color);

                if is_composing {
                    if let Some((comp_start, comp_end)) = composition_range {
                        render_composition_underline(
                            &text_layout,
                            &text_value,
                            comp_start,
                            comp_end,
                            scene,
                            transform,
                            text_color,
                        );
                    }
                }
            }

            if *component.is_focused.borrow() {
                if let Some(editor) = component.text_editor() {
                    let editor_ref = editor.editor();
                    let selection = editor_ref.selection();
                    let is_composing = editor_ref.is_composing();

                    let cursor_idx = if is_composing {
                        let composition = editor_ref.composition();
                        let cursor_offset = editor_ref.composition_cursor_offset();
                        cursor_offset
                            + composition.cursor_start.min(composition.text.chars().count())
                    } else {
                        selection.cursor()
                    };

                    let cursor_pos =
                        get_text_position(&text_value, cursor_idx, font_size, Some(&text_layout));

                    component.set_ime_area(cursor_pos.0 - 1.0, 0.0, 2.0, height);

                    if !selection.is_empty() && !is_composing {
                        let selection_color = Color::from_rgba8(0, 120, 215, 100);

                        let start_pos = get_text_position(
                            &text_value,
                            selection.start.min(selection.end),
                            font_size,
                            Some(&text_layout),
                        );
                        let end_pos = get_text_position(
                            &text_value,
                            selection.start.max(selection.end),
                            font_size,
                            Some(&text_layout),
                        );

                        let selection_rect =
                            Rect::new(start_pos.0 as f64, 0.0, end_pos.0 as f64, height);
                        scene.fill(
                            vello::peniko::Fill::NonZero,
                            transform,
                            selection_color,
                            None,
                            &selection_rect,
                        );
                    }

                    if let Some(blink) = component.cursor_blink() {
                        if blink.is_visible() {
                            let cursor_color = Color::BLACK;

                            let cursor_rect = Rect::new(
                                cursor_pos.0 as f64 - 1.0,
                                0.0,
                                cursor_pos.0 as f64 + 1.0,
                                height,
                            );
                            scene.fill(
                                vello::peniko::Fill::NonZero,
                                transform,
                                cursor_color,
                                None,
                                &cursor_rect,
                            );
                        }
                    }
                } else {
                    component.clear_ime_area();
                }
            }

            if clip {
                scene.pop_layer();
            }
        }
    }
}

fn render_composition_underline(
    layout: &Layout<BrushIndex>,
    text: &str,
    comp_start: usize,
    comp_end: usize,
    scene: &mut vello::Scene,
    transform: Affine,
    text_color: Color,
) {
    let underline_y = layout.height() as f64 - 2.0;

    let start_pos = get_text_position(text, comp_start, layout.height() as f64, Some(layout));
    let end_pos = get_text_position(text, comp_end, layout.height() as f64, Some(layout));

    let underline_width = (end_pos.0 - start_pos.0).max(2.0);
    let underline_rect = Rect::new(start_pos.0 as f64, underline_y, underline_width, 2.0);

    scene.fill(vello::peniko::Fill::NonZero, transform, text_color, None, &underline_rect);
}

fn get_text_position(
    text: &str,
    char_index: usize,
    font_size: f64,
    layout: Option<&Layout<BrushIndex>>,
) -> (f64, f64) {
    if char_index == 0 {
        return (0.0, font_size);
    }

    if let Some(layout) = layout {
        // For cursor position at end of text, use layout.width()
        if char_index >= text.chars().count() {
            return (layout.width() as f64, font_size);
        }

        // Try to get position from clusters
        let byte_index = text.char_indices().nth(char_index).map(|(i, _)| i).unwrap_or(text.len());

        if let Some(cluster) = Cluster::from_byte_index(layout, byte_index) {
            let x = cluster.visual_offset().unwrap_or(0.0) as f64;
            return (x, font_size);
        }

        // Fallback: iterate through glyph runs to find position based on character count
        let mut current_char_pos = 0;
        let mut x_pos = 0.0f64;
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    let char_count = glyph_run.glyphs().count();
                    let run_end = current_char_pos + char_count;

                    // Check if this glyph run covers our char_index
                    if current_char_pos <= char_index && char_index <= run_end {
                        return (x_pos, font_size);
                    }
                    x_pos += glyph_run.advance() as f64;
                    current_char_pos = run_end;
                }
            }
        }

        // Last resort fallback - use layout width proportionally
        let total_chars = text.chars().count();
        if total_chars > 0 {
            let proportion = char_index as f64 / total_chars as f64;
            return (layout.width() as f64 * proportion, font_size);
        }

        return (layout.width() as f64, font_size);
    }

    let char_count = text.chars().count();
    if char_index >= char_count {
        let width = text.len() as f64 * font_size * 0.6;
        return (width, font_size);
    }

    let subtext: String = text.chars().take(char_index).collect();
    let width = subtext.len() as f64 * font_size * 0.6;
    (width, font_size)
}

fn render_number_input(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    stylesheet: Option<&Stylesheet>,
    text_context: &mut crate::text::TextContext,
) {
    let styles = get_styles(component, stylesheet);
    let layout_node = component.layout_node();
    let number_value = component.number_input_value();
    let text_value = if number_value == 0.0 { String::new() } else { number_value.to_string() };

    let text_color = styles
        .text_color
        .as_ref()
        .map(|tc| {
            let rgb = tc.0 .0;
            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
        })
        .unwrap_or(Color::BLACK);

    let font_size = styles.font_size.as_ref().map(|fs| fs.0).unwrap_or(14.0);
    let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

    if let Some(layout) = layout_node {
        if let Some(input_layout) = layout.layout() {
            let width = input_layout.size.width as f64;
            let height = input_layout.size.height as f64;

            let bg_color = styles
                .background_color
                .as_ref()
                .map(|bg| {
                    let rgb = bg.0 .0;
                    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                })
                .unwrap_or_else(|| Color::from_rgb8(255, 255, 255));

            let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

            render_border(scene, transform, &Some(styles), 0.0, 0.0, width, height, border_radius);

            if !text_value.is_empty() && text_value != "0" {
                let mut layout_builder = text_context.layout_ctx.ranged_builder(
                    &mut text_context.font_ctx,
                    &text_value,
                    1.0,
                    true,
                );
                layout_builder.push_default(parley::style::StyleProperty::FontSize(font_size));
                layout_builder.push_default(parley::style::StyleProperty::Brush(BrushIndex(0)));
                layout_builder
                    .push_default(FontStack::Source(std::borrow::Cow::Borrowed("sans-serif")));

                let mut text_layout: Layout<BrushIndex> = layout_builder.build(&text_value);
                text_layout.break_all_lines(None);

                render_text_layout(&text_layout, scene, transform, text_color);
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
    let styles = get_styles(component, stylesheet);
    let layout_node = component.layout_node();
    let is_checked = component.checkbox_checked();

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

            let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(4.0);

            let size = width.min(height);
            let rounded_rect = RoundedRect::new(0.0, 0.0, size, size, border_radius);
            scene.fill(vello::peniko::Fill::NonZero, transform, bg_color, None, &rounded_rect);

            render_border(
                scene,
                transform,
                &Some(styles.clone()),
                0.0,
                0.0,
                size,
                size,
                border_radius,
            );

            if is_checked {
                let checked_color = styles
                    .color
                    .as_ref()
                    .map(|c| {
                        let rgb = c.0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(0, 120, 215));

                let inner_size = size - 6.0;
                let inner_rect =
                    RoundedRect::new(3.0, 3.0, inner_size, inner_size, border_radius.max(2.0));
                scene.fill(
                    vello::peniko::Fill::NonZero,
                    transform,
                    checked_color,
                    None,
                    &inner_rect,
                );
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
    let styles = get_styles(component, stylesheet);
    let layout_node = component.layout_node();
    let is_checked = component.radio_checked();

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

            let border_style = styles.border_style.as_ref().copied().unwrap_or(BorderStyle::None);

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

            if is_checked {
                let checked_color = styles
                    .color
                    .as_ref()
                    .map(|c| {
                        let rgb = c.0;
                        Color::from_rgb8(rgb.r, rgb.g, rgb.b)
                    })
                    .unwrap_or_else(|| Color::from_rgb8(0, 120, 215));

                let inner_size = size / 3.0;
                let inner_circle = Circle::new((center_x, center_y), inner_size);
                scene.fill(
                    vello::peniko::Fill::NonZero,
                    transform,
                    checked_color,
                    None,
                    &inner_circle,
                );
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
    let styles = get_styles(component, stylesheet);
    let layout_node = component.layout_node();

    if let Some(layout) = layout_node {
        if let Some(flex_layout) = layout.layout() {
            let _layout_x = flex_layout.location.x as f64;
            let _layout_y = flex_layout.location.y as f64;
            let width = flex_layout.size.width as f64;
            let height = flex_layout.size.height as f64;

            let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(0.0);

            let rounded_rect = RoundedRect::new(0.0, 0.0, width, height, border_radius);

            if let Some(bg) = styles.background_color.as_ref() {
                let rgb = bg.0 .0;
                let bg_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                scene.fill(
                    vello::peniko::Fill::NonZero,
                    Affine::IDENTITY,
                    bg_color,
                    None,
                    &rounded_rect,
                );
            }

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
            let overflow_x = styles.overflow_x.unwrap_or(rvue_style::properties::Overflow::Visible);
            let overflow_y = styles.overflow_y.unwrap_or(rvue_style::properties::Overflow::Visible);

            let should_clip = overflow_x.should_clip() || overflow_y.should_clip();

            if should_clip {
                // Get scroll state from user_data
                let scroll_state = get_or_create_scroll_state(component);

                // Check if we need to show scrollbars
                let show_vertical = matches!(overflow_y, rvue_style::properties::Overflow::Scroll)
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
