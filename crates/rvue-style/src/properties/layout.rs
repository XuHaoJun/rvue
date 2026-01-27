//! Layout properties for flexbox and display.

use crate::property::Property;

/// Display type for a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}

impl Default for Display {
    fn default() -> Self {
        Self::Flex
    }
}

impl Property for Display {}

/// Flex direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Row
    }
}

impl Property for FlexDirection {}

/// Justify content (main axis alignment).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::FlexStart
    }
}

impl Property for JustifyContent {}

/// Align items (cross axis alignment for flex container).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::Stretch
    }
}

impl Property for AlignItems {}

/// Align self (cross axis alignment for flex item).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Default for AlignSelf {
    fn default() -> Self {
        Self::Auto
    }
}

impl Property for AlignSelf {}

/// Flex grow factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexGrow(pub f32);

impl Default for FlexGrow {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Property for FlexGrow {}

/// Flex shrink factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexShrink(pub f32);

impl Default for FlexShrink {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Property for FlexShrink {}

/// Flex basis size.
#[derive(Clone, Debug, PartialEq)]
pub struct FlexBasis(pub Size);

impl Default for FlexBasis {
    fn default() -> Self {
        Self(Size::Auto)
    }
}

impl Property for FlexBasis {}

/// Gap between flex/grid items.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap(pub f32);

impl Default for Gap {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Property for Gap {}

use super::sizing::Size;
