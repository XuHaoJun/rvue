//! Background properties.

use super::color::{Color, RgbColor};
use crate::property::Property;
use rudo_gc::Trace;

/// Background color property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct BackgroundColor(pub Color);

impl Property for BackgroundColor {
    fn initial_value() -> Self {
        Self(Color(RgbColor { r: 0, g: 0, b: 0 }))
    }
}
