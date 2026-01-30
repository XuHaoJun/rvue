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

impl Property for Display {}

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

impl Property for FlexDirection {}

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

impl Property for JustifyContent {}

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

impl Property for AlignItems {}

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

impl Property for AlignSelf {}

/// Flex grow factor.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct FlexGrow(pub f32);

impl Property for FlexGrow {}

/// Flex shrink factor.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct FlexShrink(pub f32);

impl Property for FlexShrink {}

/// Flex basis size.
#[derive(Clone, Debug, PartialEq, Default, Trace)]
pub struct FlexBasis(pub Size);

impl Property for FlexBasis {}

/// Gap between flex/grid items.
#[derive(Clone, Copy, Debug, PartialEq, Default, Trace)]
pub struct Gap(pub f32);

impl Property for Gap {}

use super::sizing::Size;
