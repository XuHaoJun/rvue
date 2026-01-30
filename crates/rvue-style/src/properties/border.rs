//! Border properties.

use super::color::Color;
use crate::property::Property;
use rudo_gc::Trace;

/// Border style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl Property for BorderStyle {}

/// Border color property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderColor(pub Color);

impl Property for BorderColor {}

/// Border width property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderWidth(pub f32);

impl Property for BorderWidth {}

/// Border radius property (for rounded corners).
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderRadius(pub f32);

impl Property for BorderRadius {}
