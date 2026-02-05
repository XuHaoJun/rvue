//! Spacing properties.

use crate::property::Property;
use rudo_gc::{Trace, Visitor};

/// Padding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Padding(pub f32);

impl Default for Padding {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Padding {
    #[inline]
    pub fn uniform(value: f32) -> Self {
        Self(value.max(0.0))
    }
}

impl Property for Padding {
    fn initial_value() -> Self {
        Self(0.0)
    }
}

unsafe impl Trace for Padding {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

/// Margin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Margin(pub f32);

impl Default for Margin {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Margin {
    #[inline]
    pub fn uniform(value: f32) -> Self {
        Self(value.max(0.0))
    }
}

impl Property for Margin {
    fn initial_value() -> Self {
        Self(0.0)
    }
}

unsafe impl Trace for Margin {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}
