//! Border properties.

use super::color::Color;
use crate::property::Property;

/// Border style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderColor(pub Color);

impl Default for BorderColor {
    fn default() -> Self {
        Self(Color::rgb(0, 0, 0))
    }
}

impl Property for BorderColor {}

/// Border width property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderWidth(pub f32);

impl Default for BorderWidth {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Property for BorderWidth {}

/// Border radius property (for rounded corners).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderRadius(pub f32);

impl Default for BorderRadius {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Property for BorderRadius {}
