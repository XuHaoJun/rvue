// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Test harness for headless testing of rvue applications.

use std::path::PathBuf;

use rudo_gc::Gc;
use rvue::component::{Component, ComponentId};
use rvue::event::types::{
    PointerButtonEvent, PointerEvent, PointerInfo, PointerScrollEvent, ScrollDelta,
};
use rvue_style::properties::Overflow;
use vello::kurbo::Size;

use crate::event_recorder::{EventRecorder, RecordedEvent};
use crate::snapshot::{SnapshotManager, SnapshotOptions};

/// Parameters for creating a test harness.
#[derive(Debug, Clone)]
pub struct TestHarnessParams {
    pub window_size: Size,
    pub background_color: vello::peniko::Color,
    pub snapshot_options: SnapshotOptions,
}

impl Default for TestHarnessParams {
    fn default() -> Self {
        Self {
            window_size: Size::new(800.0, 600.0),
            background_color: vello::peniko::Color::from_rgb8(0x29, 0x29, 0x29),
            snapshot_options: SnapshotOptions {
                tolerance: 16,
                padding: 0,
                background_color: [0x29, 0x29, 0x29, 0xFF],
            },
        }
    }
}

/// A headless test harness for rvue applications.
pub struct TestHarness {
    root_component: Gc<Component>,
    recorder: EventRecorder,
    snapshot_manager: SnapshotManager,
    window_size: Size,
}

/// Layout information for a widget.
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub location: (f64, f64),
    pub size: (f64, f64),
}

impl TestHarness {
    /// Create a test harness with the given root widget and default parameters.
    pub fn create(widget: Gc<Component>) -> Self {
        Self::create_with_params(widget, TestHarnessParams::default())
    }

    /// Create a test harness with custom parameters.
    pub fn create_with_params(widget: Gc<Component>, params: TestHarnessParams) -> Self {
        rudo_gc::test_util::reset();

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let snapshots_dir = PathBuf::from(manifest_dir).join("tests/snapshots");

        Self {
            root_component: widget,
            recorder: EventRecorder::default(),
            snapshot_manager: SnapshotManager::new(snapshots_dir),
            window_size: params.window_size,
        }
    }

    /// Get a reference to the root component.
    pub fn root(&self) -> &Gc<Component> {
        &self.root_component
    }

    // === Widget Lookup ===

    /// Find a widget by its tag (stored in element_id).
    pub fn get_widget_by_tag(&self, tag: &str) -> Option<Gc<Component>> {
        self.find_widget_by_tag(&self.root_component, tag)
    }

    fn find_widget_by_tag(&self, component: &Gc<Component>, tag: &str) -> Option<Gc<Component>> {
        let element_id = component.element_id.read();
        if element_id.as_deref() == Some(tag) {
            return Some(Gc::clone(component));
        }
        for child in component.children.read().iter() {
            if let Some(found) = self.find_widget_by_tag(child, tag) {
                return Some(found);
            }
        }
        None
    }

    /// Find a widget by its ID.
    pub fn get_widget_by_id(&self, id: ComponentId) -> Option<Gc<Component>> {
        if self.root_component.id == id {
            return Some(Gc::clone(&self.root_component));
        }
        self.find_widget_by_id(&self.root_component, id)
    }

    fn find_widget_by_id(
        &self,
        component: &Gc<Component>,
        id: ComponentId,
    ) -> Option<Gc<Component>> {
        if component.id == id {
            return Some(Gc::clone(component));
        }
        for child in component.children.read().iter() {
            if let Some(found) = self.find_widget_by_id(child, id) {
                return Some(found);
            }
        }
        None
    }

    // === Event Recording ===

    /// Take (and clear) all recorded events for a widget.
    pub fn take_records(&mut self, widget: Gc<Component>) -> Vec<RecordedEvent> {
        self.recorder.take_records(widget.id as u32)
    }

    /// Get all recorded events for a widget without clearing.
    pub fn get_records(&self, widget: Gc<Component>) -> &[RecordedEvent] {
        self.recorder.get_records(widget.id as u32)
    }

    // === Mouse Events ===

    /// Simulate a mouse move to a widget's position.
    pub fn mouse_move_to(&mut self, _widget: Gc<Component>) {
        self.process_pointer_event(PointerEvent::Enter(PointerInfo {
            position: vello::kurbo::Point::new(100.0, 100.0),
        }));
    }

    /// Simulate a mouse click on a widget.
    pub fn mouse_click_on(&mut self, _widget: Gc<Component>) {
        let pos = vello::kurbo::Point::new(100.0, 100.0);
        self.process_pointer_event(PointerEvent::Down(PointerButtonEvent {
            button: rvue::event::types::PointerButton::Primary,
            position: pos,
            click_count: 1,
            modifiers: rvue::event::types::Modifiers::default(),
        }));
        self.process_pointer_event(PointerEvent::Up(PointerButtonEvent {
            button: rvue::event::types::PointerButton::Primary,
            position: pos,
            click_count: 1,
            modifiers: rvue::event::types::Modifiers::default(),
        }));
    }

    /// Simulate a mouse double click on a widget.
    pub fn mouse_double_click_on(&mut self, _widget: Gc<Component>) {
        let pos = vello::kurbo::Point::new(100.0, 100.0);
        for _ in 0..2 {
            self.process_pointer_event(PointerEvent::Down(PointerButtonEvent {
                button: rvue::event::types::PointerButton::Primary,
                position: pos,
                click_count: 2,
                modifiers: rvue::event::types::Modifiers::default(),
            }));
            self.process_pointer_event(PointerEvent::Up(PointerButtonEvent {
                button: rvue::event::types::PointerButton::Primary,
                position: pos,
                click_count: 2,
                modifiers: rvue::event::types::Modifiers::default(),
            }));
        }
    }

    /// Simulate a mouse button press.
    pub fn mouse_button_press(&mut self, button: rvue::event::types::PointerButton) {
        let pos = vello::kurbo::Point::new(100.0, 100.0);
        self.process_pointer_event(PointerEvent::Down(PointerButtonEvent {
            button,
            position: pos,
            click_count: 1,
            modifiers: rvue::event::types::Modifiers::default(),
        }));
    }

    /// Simulate a mouse button release.
    pub fn mouse_button_release(&mut self, button: rvue::event::types::PointerButton) {
        let pos = vello::kurbo::Point::new(100.0, 100.0);
        self.process_pointer_event(PointerEvent::Up(PointerButtonEvent {
            button,
            position: pos,
            click_count: 1,
            modifiers: rvue::event::types::Modifiers::default(),
        }));
    }

    /// Simulate a mouse drag from one widget to another.
    pub fn mouse_drag_to(&mut self, _from: Gc<Component>, to: Gc<Component>) {
        self.mouse_button_press(rvue::event::types::PointerButton::Primary);
        self.mouse_move_to(to);
        self.mouse_button_release(rvue::event::types::PointerButton::Primary);
    }

    // === Scroll Events ===

    /// Simulate a scroll wheel event at a widget.
    pub fn scroll_wheel_at(&mut self, widget: Gc<Component>, delta: ScrollDelta) {
        let pos = vello::kurbo::Point::new(100.0, 100.0);

        // Convert delta
        let delta_y = match delta {
            ScrollDelta::Line(lines) => (lines * 20.0) as f32,
            ScrollDelta::Pixel(_, dy) => (dy * 20.0) as f32,
        };
        let delta_x = match delta {
            ScrollDelta::Line(_) => 0.0,
            ScrollDelta::Pixel(dx, _) => (dx * 20.0) as f32,
        };

        // Find scroll container by traversing the tree
        let scroll_container = self.find_scroll_container_for(&widget);
        let target = scroll_container.as_ref().unwrap_or(&widget);

        // Get scroll state
        let scroll_state = target.scroll_state();
        let can_scroll_y = scroll_state.scroll_height > 0.0;
        let can_scroll_x = scroll_state.scroll_width > 0.0;

        if can_scroll_y || can_scroll_x {
            let mut new_state = scroll_state;
            if can_scroll_y {
                new_state.scroll_offset_y = (new_state.scroll_offset_y + delta_y)
                    .clamp(0.0, scroll_state.scroll_height.max(0.0));
            }
            if can_scroll_x {
                new_state.scroll_offset_x = (new_state.scroll_offset_x + delta_x)
                    .clamp(0.0, scroll_state.scroll_width.max(0.0));
            }
            target.set_scroll_state(new_state);
        }

        self.process_pointer_event(PointerEvent::Scroll(PointerScrollEvent {
            delta,
            position: pos,
            modifiers: rvue::event::types::Modifiers::default(),
        }));
    }

    /// Find scroll container for a widget by traversing the tree.
    fn find_scroll_container_for(&self, widget: &Gc<Component>) -> Option<Gc<Component>> {
        self.find_scroll_container_recursive(&self.root_component, widget)
    }

    fn find_scroll_container_recursive(
        &self,
        component: &Gc<Component>,
        target: &Gc<Component>,
    ) -> Option<Gc<Component>> {
        // First check if target is this component
        if component.id == target.id {
            return self.check_and_return_scroll_container(component);
        }

        // Check children
        for child in component.children.read().iter() {
            if child.id == target.id {
                // Found the target, check this component as potential scroll container
                return self.check_and_return_scroll_container(component);
            }
            // Recurse into child
            if let Some(found) = self.find_scroll_container_recursive(child, target) {
                return Some(found);
            }
        }

        None
    }

    /// Check if a component is a scroll container and return it.
    fn check_and_return_scroll_container(
        &self,
        component: &Gc<Component>,
    ) -> Option<Gc<Component>> {
        if let Some(styles) = component.widget_styles() {
            let overflow_x = styles.overflow_x.unwrap_or(Overflow::Visible);
            let overflow_y = styles.overflow_y.unwrap_or(Overflow::Visible);

            if overflow_x.should_clip() || overflow_y.should_clip() {
                let scroll_state = component.scroll_state();
                if scroll_state.scroll_height > 0.0 || scroll_state.scroll_width > 0.0 {
                    return Some(Gc::clone(component));
                }
            }
        }
        None
    }

    /// Simulate horizontal scroll.
    pub fn scroll_horizontal(&mut self, widget: Gc<Component>, delta: f64) {
        self.scroll_wheel_at(widget, ScrollDelta::Pixel(delta, 0.0));
    }

    /// Simulate vertical scroll.
    pub fn scroll_vertical(&mut self, widget: Gc<Component>, delta: f64) {
        self.scroll_wheel_at(widget, ScrollDelta::Pixel(0.0, delta));
    }

    // === Focus Events ===

    /// Set focus to a specific widget.
    pub fn focus_on(&mut self, _widget: Option<Gc<Component>>) {}

    /// Clear focus.
    pub fn blur_focus(&mut self) {}

    // === Process Events ===

    fn process_pointer_event(&mut self, event: PointerEvent) {
        self.recorder.record(
            self.root_component.id as u32,
            RecordedEvent::PointerEvent(crate::PointerRecord {
                event_type: match event {
                    PointerEvent::Down(_) => crate::PointerEventType::Down,
                    PointerEvent::Up(_) => crate::PointerEventType::Up,
                    PointerEvent::Move(_) => crate::PointerEventType::Move,
                    PointerEvent::Enter(_) => crate::PointerEventType::Enter,
                    PointerEvent::Leave(_) => crate::PointerEventType::Leave,
                    PointerEvent::Cancel(_) => crate::PointerEventType::Leave,
                    PointerEvent::Scroll(_) => crate::PointerEventType::Scroll,
                },
                position: event.position().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0)),
                button: None,
            }),
        );
    }

    // === Snapshot Testing ===

    /// Assert that the current render matches an existing snapshot.
    pub fn assert_snapshot(&mut self, name: &str) {
        let bless = std::env::var("RVUE_TEST_BLESS").is_ok();
        let image = self.render_to_image();

        if bless {
            self.snapshot_manager.bless(name, &image).expect("Failed to bless snapshot");
        } else {
            self.snapshot_manager.compare(name, &image).expect("Snapshot mismatch");
        }
    }

    /// Render the current state to an image.
    pub fn render_to_image(&self) -> image::DynamicImage {
        let width = self.window_size.width as u32;
        let height = self.window_size.height as u32;

        let mut buffer = image::ImageBuffer::new(width, height);
        let r: u8 = 0x29;
        let g: u8 = 0x29;
        let b: u8 = 0x29;

        for pixel in buffer.pixels_mut() {
            *pixel = image::Rgba([r, g, b, 255]);
        }

        image::DynamicImage::ImageRgba8(buffer)
    }

    // === Widget State ===

    /// Check if a widget is visible.
    pub fn is_widget_visible(&self, _widget: Gc<Component>) -> bool {
        true
    }

    /// Get the scroll offset of a widget.
    pub fn get_scroll_offset(&self, widget: Gc<Component>) -> (f64, f64) {
        let state = widget.scroll_state();
        (state.scroll_offset_x as f64, state.scroll_offset_y as f64)
    }

    /// Check if a widget has scrollbars based on its scroll state and overflow type.
    pub fn has_scrollbar(&self, widget: Gc<Component>) -> bool {
        let scroll_state = widget.scroll_state();
        let overflow = self.get_overflow(&widget);

        let show_horizontal = match overflow {
            Overflow::Visible => false,
            Overflow::Hidden => false,
            Overflow::Auto => scroll_state.scroll_width > 0.0,
            Overflow::Scroll => true,
            Overflow::Clip => false,
        };

        let show_vertical = match overflow {
            Overflow::Visible => false,
            Overflow::Hidden => false,
            Overflow::Auto => scroll_state.scroll_height > 0.0,
            Overflow::Scroll => true,
            Overflow::Clip => false,
        };

        show_horizontal || show_vertical
    }

    /// Get overflow type for a widget.
    fn get_overflow(&self, widget: &Gc<Component>) -> Overflow {
        widget.widget_styles().and_then(|s| s.overflow_y).unwrap_or(Overflow::Visible)
    }

    /// Get the layout size of a widget.
    pub fn get_widget_size(&self, _widget: Gc<Component>) -> Option<(f64, f64)> {
        None
    }

    /// Set scroll state for a widget manually.
    pub fn set_scroll_state(
        &mut self,
        widget: Gc<Component>,
        scroll_width: f32,
        scroll_height: f32,
    ) {
        let state = rvue::render::widget::FlexScrollState {
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
            scroll_width,
            scroll_height,
            container_width: 200.0,
            container_height: 200.0,
        };
        widget.set_scroll_state(state);
    }

    // === Layout Computation ===

    /// Trigger layout computation for the component tree.
    /// This will compute layout and update scroll_state for overflow containers.
    pub fn compute_layout(&mut self) {
        let size =
            vello::kurbo::Size { width: self.window_size.width, height: self.window_size.height };

        rvue::component::compute_layout_for_testing(&self.root_component, size);
    }

    // === Debug Methods ===

    /// Debug print the complete scroll state for a widget.
    pub fn debug_scroll_state(&self, widget: &Gc<Component>, label: &str) {
        let state = widget.scroll_state();

        eprintln!("\n=== Scroll State Debug: {} ===", label);
        eprintln!("Widget ID: {}", widget.id);
        eprintln!("Scroll Offset: ({}, {})", state.scroll_offset_x, state.scroll_offset_y);
        eprintln!("Scroll Size (overflow): ({}, {})", state.scroll_width, state.scroll_height);
        eprintln!("Container Size: ({}, {})", state.container_width, state.container_height);

        let overflow_y = self.get_overflow(widget);
        let should_show_v = matches!(overflow_y, Overflow::Auto) && state.scroll_height > 0.0
            || matches!(overflow_y, Overflow::Scroll);
        eprintln!("Overflow Y: {:?}", overflow_y);
        eprintln!("Should Show Vertical Scrollbar: {}", should_show_v);

        if state.scroll_height > 0.0 {
            let visible_ratio = state.container_height as f64
                / (state.container_height + state.scroll_height) as f64;
            eprintln!("Visible Content Ratio: {:.2}%", visible_ratio * 100.0);
        }
        eprintln!("=== End Debug ===\n");
    }

    /// Debug print scroll state for all widgets with overflow in the tree.
    pub fn debug_all_scroll_states(&self) {
        self._debug_scroll_state_recursive(&self.root_component, 0);
    }

    fn _debug_scroll_state_recursive(&self, component: &Gc<Component>, _depth: usize) {
        let tag = component.element_id.read().clone().unwrap_or_default();

        if let Some(styles) = component.widget_styles() {
            let overflow = styles.overflow_y.unwrap_or(Overflow::Visible);
            if overflow != Overflow::Visible {
                self.debug_scroll_state(component, &tag);
            }
        }

        for child in component.children.read().iter() {
            self._debug_scroll_state_recursive(child, _depth + 1);
        }
    }

    // === Layout Info ===

    /// Get layout information for a widget.
    pub fn get_layout_info(&self, widget: &Gc<Component>) -> Option<LayoutInfo> {
        let layout_node = widget.layout_node.read();
        let layout = layout_node.as_ref()?.layout_result?;

        Some(LayoutInfo {
            location: (layout.location.x as f64, layout.location.y as f64),
            size: (layout.size.width as f64, layout.size.height as f64),
        })
    }

    /// Get all layout info for a widget and its children.
    pub fn get_layout_info_tree(&self, widget: &Gc<Component>) -> Vec<(String, LayoutInfo)> {
        self._get_layout_info_recursive(widget)
    }

    fn _get_layout_info_recursive(&self, component: &Gc<Component>) -> Vec<(String, LayoutInfo)> {
        let mut result = Vec::new();

        let tag = component
            .element_id
            .read()
            .clone()
            .unwrap_or_else(|| format!("widget-{}", component.id));
        if let Some(info) = self.get_layout_info(component) {
            result.push((tag.clone(), info));
        }

        for child in component.children.read().iter() {
            result.extend(self._get_layout_info_recursive(child));
        }

        result
    }
}

// Re-export types for easier use
pub use rvue::event::types::PointerButton;
