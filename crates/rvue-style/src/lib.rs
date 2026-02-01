//! rvue-style - GPU-accelerated GUI styling system
//!
//! This crate provides a CSS-compatible styling system for rvue, combining:
//! - Type-safe property system with `Property` trait
//! - CSS selector parsing and matching
//! - rudo-gc integration for memory management

#![warn(missing_docs)]
#![allow(
    clippy::unnecessary_map_or,
    clippy::should_implement_trait,
    clippy::redundant_closure,
    clippy::manual_strip,
    clippy::clone_on_copy,
    missing_docs
)]

pub mod properties;
pub mod property;
pub mod reactive;
pub mod selectors;
pub mod shared;
pub mod stylesheet;
pub mod widget;

pub use properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, ComputedStyles, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink,
    FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Overflow,
    Padding, Size, TextColor, Visibility, Width, ZIndex,
};
pub use property::{Properties, Property, StyleStore};
pub use reactive::{
    create_reactive_signal, create_style_effect, on_style_cleanup, ReactiveProperty,
    ReactiveReadSignal, ReactiveSignal, ReactiveSignalWrite, ReactiveStyles,
};
pub use selectors::{ElementState, RvueElement};
pub use shared::{
    shared_background_color, shared_centered_flex, shared_flex_container, shared_margin,
    shared_padding, shared_text_color, SharedComputedStyles, SharedStyleBuilder,
    WeakSharedComputedStyles,
};
pub use stylesheet::{default_stylesheet, StyleResolver, StyleRule, Stylesheet};
pub use widget::styled::{StyleData, WidgetStyles};
pub use widget::{StyledWidget, StyledWidgetExt};
