//! Cursor blink state for text input widgets.
//!
//! Implements the blinking cursor animation that:
//! - Blinks for 10 seconds after user interaction
//! - Shows cursor for 500ms, hides for 500ms
//! - Stays solid after timeout

use rudo_gc::GcCell;
use rudo_gc::Trace;

const CURSOR_BLINK_TIME: u64 = 1000;
const CURSOR_BLINK_TIMEOUT: u64 = 10000;

#[derive(Clone, Copy, Debug)]
pub struct CursorBlinkState {
    pub anim_cursor_visible: bool,
    pub anim_prev_interval: u64,
    pub anim_elapsed: u64,
}

unsafe impl Trace for CursorBlinkState {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl Default for CursorBlinkState {
    fn default() -> Self {
        Self { anim_cursor_visible: true, anim_prev_interval: 0, anim_elapsed: 0 }
    }
}

impl CursorBlinkState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.anim_prev_interval = 0;
        self.anim_elapsed = 0;
        self.anim_cursor_visible = true;
    }

    pub fn update(&mut self, interval_ms: u64, is_focused: bool) -> bool {
        if !is_focused {
            return false;
        }

        let mut visibility_changed = false;

        if self.anim_elapsed < CURSOR_BLINK_TIMEOUT {
            self.anim_prev_interval += interval_ms;
            self.anim_elapsed += interval_ms;

            if self.anim_prev_interval >= CURSOR_BLINK_TIME {
                self.anim_prev_interval = self.anim_prev_interval.rem_euclid(CURSOR_BLINK_TIME);
            }

            let should_be_visible = self.anim_prev_interval < CURSOR_BLINK_TIME / 2;
            if should_be_visible != self.anim_cursor_visible {
                self.anim_cursor_visible = should_be_visible;
                visibility_changed = true;
            }
        } else if !self.anim_cursor_visible {
            self.anim_cursor_visible = true;
            visibility_changed = true;
        }

        visibility_changed
    }

    pub fn is_visible(&self) -> bool {
        self.anim_cursor_visible
    }
}

#[derive(Clone)]
pub struct GcCursorBlinkState(GcCell<CursorBlinkState>);

impl Default for GcCursorBlinkState {
    fn default() -> Self {
        Self(GcCell::new(CursorBlinkState::new()))
    }
}

unsafe impl Trace for GcCursorBlinkState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.0.trace(visitor);
    }
}

impl GcCursorBlinkState {
    pub fn new() -> Self {
        Self(GcCell::new(CursorBlinkState::new()))
    }

    pub fn reset(&self) {
        *self.0.borrow_mut_gen_only() = CursorBlinkState::new();
    }

    pub fn update(&self, interval_ms: u64, is_focused: bool) -> bool {
        let mut state = self.0.borrow_mut_gen_only();
        state.update(interval_ms, is_focused)
    }

    pub fn is_visible(&self) -> bool {
        self.0.borrow().is_visible()
    }

    pub fn state(&self) -> CursorBlinkState {
        *self.0.borrow()
    }
}
