//! Sizing properties.

use crate::property::Property;
use rudo_gc::Trace;
use std::fmt;

/// Size value.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
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

impl Property for Size {
    fn initial_value() -> Self {
        Self::Auto
    }
}

/// Width.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct Width(pub Size);

impl Property for Width {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Height.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct Height(pub Size);

impl Property for Height {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Minimum width property.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct MinWidth(pub Size);

impl Property for MinWidth {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Minimum height property.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct MinHeight(pub Size);

impl Property for MinHeight {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Maximum width property.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct MaxWidth(pub Size);

impl Property for MaxWidth {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Maximum height property.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct MaxHeight(pub Size);

impl Property for MaxHeight {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}
