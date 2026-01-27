//! Sizing properties.

use crate::property::Property;
use std::fmt;

/// Size value.
#[derive(Clone, Debug, PartialEq)]
pub enum Size {
    Auto,
    Pixels(f32),
    Percent(f32),
    MinContent,
    MaxContent,
}

impl Default for Size {
    fn default() -> Self {
        Self::Auto
    }
}

impl Size {
    #[inline]
    pub fn auto() -> Self {
        Self::Auto
    }

    #[inline]
    pub fn pixels(value: f32) -> Self {
        Self::Pixels(value.max(0.0))
    }

    #[inline]
    pub fn percent(value: f32) -> Self {
        Self::Percent(value.clamp(0.0, 100.0))
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Pixels(v) => write!(f, "{v}px"),
            Self::Percent(v) => write!(f, "{v}%"),
            Self::MinContent => write!(f, "min-content"),
            Self::MaxContent => write!(f, "max-content"),
        }
    }
}

/// Width.
#[derive(Clone, Debug, PartialEq)]
pub struct Width(pub Size);

impl Default for Width {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

/// Height.
#[derive(Clone, Debug, PartialEq)]
pub struct Height(pub Size);

impl Default for Height {
    fn default() -> Self {
        Self(Size::Auto)
    }
}
