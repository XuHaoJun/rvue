//! Sizing properties.

use crate::property::Property;
use std::fmt;

/// Size value.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Size {
    #[default]
    Auto,
    Pixels(f32),
    Percent(f32),
    MinContent,
    MaxContent,
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

impl Property for Size {}

/// Width.
#[derive(Clone, Debug, PartialEq)]
pub struct Width(pub Size);

impl Default for Width {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for Width {}

/// Height.
#[derive(Clone, Debug, PartialEq)]
pub struct Height(pub Size);

impl Default for Height {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for Height {}

/// Minimum width property.
#[derive(Clone, Debug, PartialEq)]
pub struct MinWidth(pub Size);

impl Default for MinWidth {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for MinWidth {}

/// Minimum height property.
#[derive(Clone, Debug, PartialEq)]
pub struct MinHeight(pub Size);

impl Default for MinHeight {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for MinHeight {}

/// Maximum width property.
#[derive(Clone, Debug, PartialEq)]
pub struct MaxWidth(pub Size);

impl Default for MaxWidth {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for MaxWidth {}

/// Maximum height property.
#[derive(Clone, Debug, PartialEq)]
pub struct MaxHeight(pub Size);

impl Default for MaxHeight {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for MaxHeight {}
