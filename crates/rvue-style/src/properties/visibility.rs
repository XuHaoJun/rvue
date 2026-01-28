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

impl Property for Visibility {}

/// Opacity property (0.0 to 1.0).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Opacity(pub f32);

impl Property for Opacity {}

/// Z-index property for stacking order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ZIndex(pub i32);

impl Property for ZIndex {}

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

impl Property for Cursor {}
