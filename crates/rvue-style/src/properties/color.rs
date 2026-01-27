//! Color property.

use crate::property::Property;

/// RGB color.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub RgbColor);

impl Default for Color {
    fn default() -> Self {
        Self(RgbColor::rgb(0, 0, 0))
    }
}

impl Color {
    #[inline]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(RgbColor::rgb(r, g, b))
    }

    pub fn from_hex(hex: &str) -> Result<Self, ()> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(());
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
        Ok(Self::rgb(r, g, b))
    }
}

impl Property for Color {}

/// Text/foreground color property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextColor(pub Color);

impl Default for TextColor {
    fn default() -> Self {
        Self(Color::rgb(0, 0, 0))
    }
}

impl Property for TextColor {}
