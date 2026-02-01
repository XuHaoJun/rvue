//! Layout properties for flexbox and display.

use crate::property::Property;
use rudo_gc::Trace;

/// Display type for a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum Display {
    #[default]
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}

impl Property for Display {
    fn initial_value() -> Self {
        Self::Flex
    }
}

/// Flex direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlexDirection::Row => "row",
            FlexDirection::RowReverse => "row-reverse",
            FlexDirection::Column => "column",
            FlexDirection::ColumnReverse => "column-reverse",
        }
    }
}

impl Property for FlexDirection {
    fn initial_value() -> Self {
        Self::Row
    }
}

/// Justify content (main axis alignment).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl JustifyContent {
    pub fn as_str(&self) -> &'static str {
        match self {
            JustifyContent::FlexStart => "start",
            JustifyContent::FlexEnd => "end",
            JustifyContent::Center => "center",
            JustifyContent::SpaceBetween => "space-between",
            JustifyContent::SpaceAround => "space-around",
            JustifyContent::SpaceEvenly => "space-evenly",
        }
    }
}

impl Property for JustifyContent {
    fn initial_value() -> Self {
        Self::FlexStart
    }
}

/// Align items (cross axis alignment for flex container).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl AlignItems {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlignItems::Stretch => "stretch",
            AlignItems::FlexStart => "start",
            AlignItems::FlexEnd => "end",
            AlignItems::Center => "center",
            AlignItems::Baseline => "baseline",
        }
    }
}

impl Property for AlignItems {
    fn initial_value() -> Self {
        Self::Stretch
    }
}

/// Align self (cross axis alignment for flex item).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Trace)]
pub enum AlignSelf {
    #[default]
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Property for AlignSelf {
    fn initial_value() -> Self {
        Self::Auto
    }
}

/// Flex grow factor.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct FlexGrow(pub f32);

impl Property for FlexGrow {
    fn initial_value() -> Self {
        Self(0.0)
    }
}

/// Flex shrink factor.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct FlexShrink(pub f32);

impl Property for FlexShrink {
    fn initial_value() -> Self {
        Self(1.0)
    }
}

/// Flex basis size.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct FlexBasis(pub Size);

impl Property for FlexBasis {
    fn initial_value() -> Self {
        Self(Size::Auto)
    }
}

/// Gap between flex/grid items.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct Gap(pub f32);

impl Property for Gap {
    fn initial_value() -> Self {
        Self(0.0)
    }
}

use super::sizing::Size;
