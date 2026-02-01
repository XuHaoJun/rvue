//! Visibility and opacity properties.

use crate::property::Property;

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

/// Opacity property (0.0 to 1.0).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Opacity(pub f32);

impl Property for Opacity {
    fn initial_value() -> Self {
        Self(1.0)
    }
}

/// Z-index property for stacking order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ZIndex(pub i32);

impl Property for ZIndex {
    fn initial_value() -> Self {
        Self(0)
    }
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
