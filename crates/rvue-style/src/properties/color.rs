//! Color property.

use crate::property::Property;
use rudo_gc::Trace;

/// RGB color.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Color property value.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct Color(pub RgbColor);

impl Color {
    #[inline]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(RgbColor::rgb(r, g, b))
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::rgb(r, g, b))
    }
}

impl Property for Color {
    fn initial_value() -> Self {
        Self(RgbColor { r: 0, g: 0, b: 0 })
    }
}

/// Text/foreground color property.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct TextColor(pub Color);

impl Property for TextColor {
    fn initial_value() -> Self {
        Self(Color(RgbColor { r: 0, g: 0, b: 0 }))
    }
}
