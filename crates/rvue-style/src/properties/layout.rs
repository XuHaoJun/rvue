//! Layout properties for flexbox and display.

use crate::property::Property;

/// Display type for a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}

impl Property for Display {}

/// Flex direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Property for FlexDirection {}

/// Justify content (main axis alignment).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Property for JustifyContent {}

/// Align items (cross axis alignment for flex container).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Property for AlignItems {}

/// Align self (cross axis alignment for flex item).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlignSelf {
    #[default]
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
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
