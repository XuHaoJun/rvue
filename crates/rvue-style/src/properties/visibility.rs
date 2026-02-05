//! Visibility and opacity properties.

use crate::property::Property;
use rudo_gc::{Trace, Visitor};

/// Visibility property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

impl Property for Visibility {
    fn initial_value() -> Self {
        Self::Visible
    }
}

unsafe impl Trace for Visibility {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

/// Opacity property (0.0 to 1.0).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Opacity(pub f32);

impl Property for Opacity {
    fn initial_value() -> Self {
        Self(1.0)
    }
}

unsafe impl Trace for Opacity {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

/// Z-index property for stacking order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ZIndex(pub i32);

impl Property for ZIndex {
    fn initial_value() -> Self {
        Self(0)
    }
}

unsafe impl Trace for ZIndex {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

/// Cursor property.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Default,
    Pointer,
    Text,
    Move,
    NotAllowed,
    Progress,
    Wait,
    Crosshair,
    Help,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
}

impl Property for Cursor {
    fn initial_value() -> Self {
        Self::Default
    }
}

unsafe impl Trace for Cursor {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}
