//! Background properties.

use super::color::Color;
use crate::property::Property;

/// Background color property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundColor(pub Color);

impl Default for BackgroundColor {
    fn default() -> Self {
        Self(Color::rgb(0, 0, 0))
    }
}

impl Property for BackgroundColor {}
