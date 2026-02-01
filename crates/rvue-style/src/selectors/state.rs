//! Element state tracking for pseudo-class matching.

use bitflags::bitflags;

bitflags! {
    /// Represents the state of a widget element for pseudo-class matching.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct ElementState: u64 {
        const HOVER = 1 << 0;
        const FOCUS = 1 << 1;
        const ACTIVE = 1 << 2;
        const DISABLED = 1 << 3;
        const CHECKED = 1 << 4;
        const DRAGGING = 1 << 5;
        const DRAG_OVER = 1 << 6;
        const SELECTED = 1 << 7;
        const EXPANDED = 1 << 8;
        const COLLAPSED = 1 << 9;
        const VISITED = 1 << 10;
        const TARGET = 1 << 11;
        const FOCUS_WITHIN = 1 << 12;
        const FOCUS_VISIBLE = 1 << 13;
        const EMPTY = 0;
    }
}

impl ElementState {
    /// Checks if the element matches a pseudo-class.
    pub fn matches_pseudo_class(&self, pseudo_class: &str) -> bool {
        match pseudo_class {
            "hover" => self.intersects(Self::HOVER),
            "focus" => self.intersects(Self::FOCUS),
            "active" => self.intersects(Self::ACTIVE),
            "disabled" => self.intersects(Self::DISABLED),
            "checked" => self.intersects(Self::CHECKED),
            "dragging" => self.intersects(Self::DRAGGING),
            "drag-over" | "drag_over" => self.intersects(Self::DRAG_OVER),
            "selected" => self.intersects(Self::SELECTED),
            "expanded" => self.intersects(Self::EXPANDED),
            "collapsed" => self.intersects(Self::COLLAPSED),
            "visited" => self.intersects(Self::VISITED),
            "target" => self.intersects(Self::TARGET),
            "focus-within" | "focus_within" => self.intersects(Self::FOCUS_WITHIN),
            "focus-visible" | "focus_visible" => self.intersects(Self::FOCUS_VISIBLE),
            _ => false,
        }
    }
}
