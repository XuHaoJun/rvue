//! ScrollBar widget for overflow scrolling

use crate::component::{Component, ComponentType};
use crate::widget::{BuildContext, Mountable, Widget};
use rudo_gc::{Gc, Trace};
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Color, Fill};

/// ScrollBar widget configuration
#[derive(Clone)]
pub struct ScrollBar {
    #[allow(dead_code)]
    axis: ScrollAxis,
    portal_size: f64,
    content_size: f64,
    scroll_offset: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

impl ScrollBar {
    /// Create a new horizontal scrollbar
    pub fn horizontal(portal_size: f64, content_size: f64, scroll_offset: f64) -> Self {
        Self { axis: ScrollAxis::Horizontal, portal_size, content_size, scroll_offset }
    }

    /// Create a new vertical scrollbar
    pub fn vertical(portal_size: f64, content_size: f64, scroll_offset: f64) -> Self {
        Self { axis: ScrollAxis::Vertical, portal_size, content_size, scroll_offset }
    }

    /// Calculate the thumb (handle) length based on portal/content ratio
    pub fn thumb_length(&self, track_length: f64) -> f64 {
        const MIN_THUMB_LENGTH: f64 = 20.0;
        if self.content_size <= self.portal_size {
            return track_length;
        }
        let ratio = self.portal_size / self.content_size;
        (ratio * track_length).max(MIN_THUMB_LENGTH)
    }

    /// Calculate the thumb position based on scroll offset
    pub fn thumb_position(&self, track_length: f64, thumb_length: f64) -> f64 {
        let scrollable_length = self.content_size - self.portal_size;
        if scrollable_length <= 0.0 {
            return 0.0;
        }
        (self.scroll_offset / scrollable_length) * (track_length - thumb_length)
    }

    /// Convert scroll position (pixels) to cursor progress (0.0 to 1.0)
    pub fn scroll_to_progress(&self, scroll_pos: f64) -> f64 {
        let scrollable = (self.content_size - self.portal_size).max(0.0);
        if scrollable <= 0.0 {
            return 0.0;
        }
        (scroll_pos / scrollable).clamp(0.0, 1.0)
    }

    /// Convert cursor progress to scroll position
    pub fn progress_to_scroll(&self, progress: f64) -> f64 {
        let scrollable = (self.content_size - self.portal_size).max(0.0);
        progress.clamp(0.0, 1.0) * scrollable
    }

    /// Calculate the vertical thumb rect
    pub fn vertical_thumb_rect(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        scroll_offset_y: f64,
        scroll_height: f64,
    ) -> Option<Rect> {
        let scrollbar_width = default_scrollbar::WIDTH;
        let thumb_min_length = default_scrollbar::THUMB_MIN_LENGTH;

        let container_height = height;
        let content_height = container_height + scroll_height;

        if content_height <= container_height {
            return None;
        }

        let track_x = x + width - scrollbar_width;
        let thumb_ratio = container_height / content_height;
        let thumb_height = (thumb_ratio * container_height).max(thumb_min_length);
        let thumb_y =
            y + (scroll_offset_y / scroll_height.max(1.0)) * (container_height - thumb_height);

        Some(Rect::new(
            track_x + default_scrollbar::TRACK_MARGIN,
            thumb_y + default_scrollbar::TRACK_MARGIN,
            track_x + scrollbar_width - default_scrollbar::TRACK_MARGIN,
            thumb_y + thumb_height - default_scrollbar::TRACK_MARGIN,
        ))
    }

    /// Calculate the horizontal thumb rect
    pub fn horizontal_thumb_rect(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        scroll_offset_x: f64,
        scroll_width: f64,
    ) -> Option<Rect> {
        let scrollbar_height = default_scrollbar::WIDTH;
        let thumb_min_length = default_scrollbar::THUMB_MIN_LENGTH;

        let container_width = width;
        let content_width = container_width + scroll_width;

        if content_width <= container_width {
            return None;
        }

        let track_y = y + height - scrollbar_height;
        let thumb_ratio = container_width / content_width;
        let thumb_width = (thumb_ratio * container_width).max(thumb_min_length);
        let thumb_x =
            x + (scroll_offset_x / scroll_width.max(1.0)) * (container_width - thumb_width);

        Some(Rect::new(
            thumb_x + default_scrollbar::TRACK_MARGIN,
            track_y + default_scrollbar::TRACK_MARGIN,
            thumb_x + thumb_width - default_scrollbar::TRACK_MARGIN,
            track_y + scrollbar_height - default_scrollbar::TRACK_MARGIN,
        ))
    }

    /// Check if a point is within the vertical scrollbar track
    pub fn is_point_in_vertical_track(
        point_x: f64,
        point_y: f64,
        container_x: f64,
        container_y: f64,
        container_width: f64,
        container_height: f64,
    ) -> bool {
        let scrollbar_width = default_scrollbar::WIDTH;
        let track_x = container_x + container_width - scrollbar_width;
        point_x >= track_x
            && point_x <= container_x + container_width
            && point_y >= container_y
            && point_y <= container_y + container_height
    }

    /// Check if a point is within the horizontal scrollbar track
    pub fn is_point_in_horizontal_track(
        point_x: f64,
        point_y: f64,
        container_x: f64,
        container_y: f64,
        container_width: f64,
        container_height: f64,
    ) -> bool {
        let scrollbar_height = default_scrollbar::WIDTH;
        let track_y = container_y + container_height - scrollbar_height;
        point_x >= container_x
            && point_x <= container_x + container_width
            && point_y >= track_y
            && point_y <= container_y + container_height
    }
}

unsafe impl Trace for ScrollBar {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

/// State for a mounted ScrollBar
pub struct ScrollBarState {
    component: Gc<Component>,
}

unsafe impl Trace for ScrollBarState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
    }
}

impl ScrollBarState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

impl Mountable for ScrollBarState {
    fn mount(&self, parent: Option<Gc<Component>>) {
        self.component.set_parent(parent.clone());
        if let Some(parent) = parent {
            parent.add_child(Gc::clone(&self.component));
        }
    }

    fn unmount(&self) {
        self.component.set_parent(None);
    }
}

impl Widget for ScrollBar {
    type State = ScrollBarState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let component = Component::with_properties(
            id,
            ComponentType::Custom("ScrollBar".to_string()),
            crate::properties::PropertyMap::new(),
        );

        ScrollBarState { component }
    }

    fn rebuild(self, _state: &mut Self::State) {}
}

/// Default scrollbar constants
pub mod default_scrollbar {
    pub const WIDTH: f64 = 10.0;
    pub const THUMB_MIN_LENGTH: f64 = 20.0;
    pub const BORDER_RADIUS: f64 = 4.0;
    pub const TRACK_MARGIN: f64 = 2.0;
}

/// Get the scrollbar track color
fn track_color() -> Color {
    Color::from_rgba8(0, 0, 0, 77)
}

/// Get the scrollbar thumb color
fn thumb_color() -> Color {
    Color::from_rgba8(255, 255, 255, 200)
}

/// Render a vertical scrollbar into the scene
pub fn render_vertical_scrollbar(
    scene: &mut vello::Scene,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scroll_offset_y: f64,
    scroll_height: f64,
) {
    let scrollbar_width = default_scrollbar::WIDTH;
    let thumb_min_length = default_scrollbar::THUMB_MIN_LENGTH;

    let container_height = height;
    let content_height = container_height + scroll_height;

    // Track position (right side)
    let track_x = x + width - scrollbar_width;
    let track_width = scrollbar_width;

    // Render track
    let track_rect = Rect::new(track_x, y, track_x + track_width, y + container_height);
    scene.fill(Fill::NonZero, Affine::IDENTITY, track_color(), None, &track_rect);

    // Calculate and render thumb
    if content_height > container_height {
        let thumb_ratio = container_height / content_height;
        let thumb_height = (thumb_ratio * container_height).max(thumb_min_length);
        let thumb_y =
            y + (scroll_offset_y / scroll_height.max(1.0)) * (container_height - thumb_height);

        let thumb_rect = RoundedRect::new(
            track_x + default_scrollbar::TRACK_MARGIN,
            thumb_y + default_scrollbar::TRACK_MARGIN,
            track_x + scrollbar_width - default_scrollbar::TRACK_MARGIN,
            thumb_y + thumb_height - default_scrollbar::TRACK_MARGIN,
            default_scrollbar::BORDER_RADIUS,
        );
        scene.fill(Fill::NonZero, Affine::IDENTITY, thumb_color(), None, &thumb_rect);
    }
}

/// Render a horizontal scrollbar into the scene
pub fn render_horizontal_scrollbar(
    scene: &mut vello::Scene,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scroll_offset_x: f64,
    scroll_width: f64,
) {
    let scrollbar_height = default_scrollbar::WIDTH;
    let thumb_min_length = default_scrollbar::THUMB_MIN_LENGTH;

    let container_width = width;
    let content_width = container_width + scroll_width;

    // Track position (bottom)
    let track_y = y + height - scrollbar_height;
    let track_height = scrollbar_height;

    // Render track
    let track_rect = Rect::new(x, track_y, x + container_width, track_y + track_height);
    scene.fill(Fill::NonZero, Affine::IDENTITY, track_color(), None, &track_rect);

    // Calculate and render thumb
    if content_width > container_width {
        let thumb_ratio = container_width / content_width;
        let thumb_width = (thumb_ratio * container_width).max(thumb_min_length);
        let thumb_x =
            x + (scroll_offset_x / scroll_width.max(1.0)) * (container_width - thumb_width);

        let thumb_rect = RoundedRect::new(
            thumb_x + default_scrollbar::TRACK_MARGIN,
            track_y + default_scrollbar::TRACK_MARGIN,
            thumb_x + thumb_width - default_scrollbar::TRACK_MARGIN,
            track_y + scrollbar_height - default_scrollbar::TRACK_MARGIN,
            default_scrollbar::BORDER_RADIUS,
        );
        scene.fill(Fill::NonZero, Affine::IDENTITY, thumb_color(), None, &thumb_rect);
    }
}
