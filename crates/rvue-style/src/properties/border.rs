//! Border properties.

use super::color::{Color, RgbColor};
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

impl Property for BorderStyle {
    fn initial_value() -> Self {
        Self::None
    }
}

/// Border color property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderColor(pub Color);

impl Property for BorderColor {
    fn initial_value() -> Self {
        Self(Color(RgbColor { r: 0, g: 0, b: 0 }))
    }
}

/// Border width property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderWidth(pub f32);

impl Property for BorderWidth {
    fn initial_value() -> Self {
        Self(0.0)
    }
}

/// Border radius property (for rounded corners).
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BorderRadius(pub f32);

impl Property for BorderRadius {
    fn initial_value() -> Self {
        Self(0.0)
    }
}
