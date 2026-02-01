//! Font and text properties.

use crate::property::Property;
use std::fmt;

/// Font family property.
#[derive(Clone, Debug, PartialEq)]
pub struct FontFamily(pub String);

impl Default for FontFamily {
    fn default() -> Self {
        Self("system-ui".to_string())
    }
}

impl fmt::Display for FontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Property for FontFamily {
    fn initial_value() -> Self {
        Self("system-ui".to_string())
    }
}

/// Font size property.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontSize(pub f32);

impl Default for FontSize {
    fn default() -> Self {
        Self(16.0)
    }
}

impl Property for FontSize {
    fn initial_value() -> Self {
        Self(16.0)
    }
}

/// Font weight property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    #[default]
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl Property for FontWeight {
    fn initial_value() -> Self {
        Self::Normal
    }
}

impl FontWeight {
    #[inline]
    pub fn from_numeric(value: u16) -> Self {
        match value {
            100 => Self::Thin,
            200 => Self::ExtraLight,
            300 => Self::Light,
            400 => Self::Normal,
            500 => Self::Medium,
            600 => Self::SemiBold,
            700 => Self::Bold,
            800 => Self::ExtraBold,
            900 => Self::Black,
            _ if value < 100 => Self::Thin,
            _ => Self::Black,
        }
    }
}
