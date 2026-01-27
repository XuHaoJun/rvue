//! Visibility and opacity properties.

use crate::property::Property;

/// Visibility property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

impl Property for Visibility {}

/// Opacity property (0.0 to 1.0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f32);

impl Default for Opacity {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Property for Opacity {}

/// Z-index property for stacking order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZIndex(pub i32);

impl Default for ZIndex {
    fn default() -> Self {
        Self(0)
    }
}

impl Property for ZIndex {}

/// Cursor property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cursor {
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

impl Default for Cursor {
    fn default() -> Self {
        Self::Default
    }
}

impl Property for Cursor {}
