# API Contracts: Rvue Style System

**Feature**: 001-style-system | **Date**: 2026-01-27

## Overview

This document specifies the public API contracts for the rvue style system. All contracts are organized by module.

---

## Property Module (`rvue_style::property`)

### Property Trait

```rust
/// A marker trait that indicates a type is a widget property.
///
/// Properties are arbitrary values stored alongside a widget.
/// They provide type-safe styling through Rust's type system.
pub trait Property: Default + Send + Sync + 'static {
    /// Returns a static reference to the default value.
    ///
    /// This is used for property lookup when no value is set.
    /// Should return the same value as `Default::default()`.
    fn static_default() -> &'static Self;
}
```

**Preconditions**:
- `static_default()` must return a valid, immutable default instance

**Postconditions**:
- All property implementations must be cloneable and comparable

**Errors**:
- No errors - trait method always succeeds

---

### Properties Container

```rust
/// A collection of properties for a widget.
#[derive(Default)]
pub struct Properties {
    map: AnyMap,
}

impl Properties {
    /// Creates an empty collection of properties.
    pub fn new() -> Self;

    /// Creates a collection with a single property.
    pub fn one<P: Property>(value: P) -> Self;

    /// Builder-style method to add a property.
    ///
    /// # Arguments
    /// * `value` - The property value to set
    ///
    /// If the property was already set, it's replaced.
    pub fn with<P: Property>(mut self, value: P) -> Self;

    /// Returns value of property `P` if set.
    pub fn get<P: Property>(&self) -> Option<&P>;

    /// Sets property `P` to given value.
    ///
    /// # Returns
    /// The previous value if `P` was already set.
    pub fn insert<P: Property>(&mut self, value: P) -> Option<P>;

    /// Removes property `P`.
    ///
    /// # Returns
    /// The previous value if `P` was set.
    pub fn remove<P: Property>(&mut self) -> Option<P>;
}
```

**Preconditions**:
- `new()`: None
- `with()`: None
- `get()`: None
- `insert()`: None
- `remove()`: None

**Postconditions**:
- `new()` returns empty Properties
- `with(P)` contains P
- `get(Some(P))` returns `Some(&P)` if inserted
- `insert(P)` replaces previous value if exists
- `remove()` clears the property

**Errors**:
- `get()`: Returns `None` if property not found (not an error)
- All operations are infallible

---

## Properties Module (`rvue_style::properties`)

### Color Property

```rust
/// RGB color representation.
#[derive(Clone, Debug, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    /// Creates color from RGB values (0-255).
    pub fn rgb(r: u8, g: u8, b: u8) -> Self;

    /// Creates color from RGBA values (0-255).
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self;

    /// Parses color from hex string (#RGB or #RRGGBB).
    pub fn from_hex(hex: &str) -> Result<Self, ParseError>;

    /// Parses color from CSS color name.
    pub fn from_str(name: &str) -> Result<Self, ParseError>;
}

/// Background color property.
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundColor(pub Color);

impl Property for BackgroundColor {
    fn static_default() -> &'static Self;
}

/// Text color property.
#[derive(Clone, Debug, PartialEq)]
pub struct TextColor(pub Color);

impl Property for TextColor {
    fn static_default() -> &'static Self;
}
```

**Preconditions**:
- `from_hex()`: String must start with `#` and contain valid hex digits
- `from_str()`: Name must be a valid CSS color name

**Postconditions**:
- All constructors return valid Color instances

**Errors**:
- `from_hex()`: `ParseError` for invalid hex format
- `from_str()`: `ParseError` for unknown color names

---

### Spacing Properties

```rust
/// Padding property (uniform on all sides).
#[derive(Clone, Debug, PartialEq)]
pub struct Padding(pub f32);

impl Padding {
    /// Creates uniform padding for all sides.
    pub fn uniform(value: f32) -> Self;

    /// Creates symmetric padding (vertical, horizontal).
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self;

    /// Creates padding for individual sides (top, right, bottom, left).
    pub fn all(top: f32, right: f32, bottom: f32, left: f32) -> Self;
}

impl Property for Padding {
    fn static_default() -> &'static Self;
}

/// Margin property.
#[derive(Clone, Debug, PartialEq)]
pub struct Margin(pub f32);

impl Margin {
    pub fn uniform(value: f32) -> Self;
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self;
    pub fn all(top: f32, right: f32, bottom: f32, left: f32) -> Self;

    /// Creates auto margin (for flexbox centering).
    pub fn auto() -> Self;
}

impl Property for Margin {
    fn static_default() -> &'static Self;
}
```

**Preconditions**:
- All values must be non-negative (>= 0.0)

**Postconditions**:
- Valid Padding/Margin instances

**Errors**:
- Panic if negative values provided

---

### Layout Properties

```rust
/// Display property (controls layout box type).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Display {
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}

impl Property for Display {
    fn static_default() -> &'static Self;
}

/// Flexbox direction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Property for FlexDirection {
    fn static_default() -> &'static Self;
}

/// Flexbox main axis alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Property for JustifyContent {
    fn static_default() -> &'static Self;
}

/// Flexbox cross axis alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Property for AlignItems {
    fn static_default() -> &'static Self;
}
```

---

### Sizing Properties

```rust
/// Size value with multiple units.
#[derive(Clone, Debug, PartialEq)]
pub enum Size {
    Auto,
    Pixels(f32),
    Percent(f32),
    MinContent,
    MaxContent,
    FitContent,
}

impl Size {
    pub fn auto() -> Self;
    pub fn pixels(value: f32) -> Self;
    pub fn percent(value: f32) -> Self;
}

/// Width property.
#[derive(Clone, Debug, PartialEq)]
pub struct Width(pub Size);

impl Property for Width {
    fn static_default() -> &'static Self;
}

/// Height property.
#[derive(Clone, Debug, PartialEq)]
pub struct Height(pub Size);

impl Property for Height {
    fn static_default() -> &'static Self;
}

/// Minimum width property.
#[derive(Clone, Debug, PartialEq)]
pub struct MinWidth(pub Size);

impl Property for MinWidth {
    fn static_default() -> &'static Self;
}

/// Maximum width property.
#[derive(Clone, Debug, PartialEq)]
pub struct MaxWidth(pub Size);

impl Property for MaxWidth {
    fn static_default() -> &'static Self;
}
```

---

## Selectors Module (`rvue_style::selectors`)

### ElementState

```rust
/// Widget state flags for pseudo-class matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementState(u64);

impl ElementState {
    /// Creates empty state.
    pub fn empty() -> Self;

    /// Checks if state is empty.
    pub fn is_empty(&self) -> bool;

    /// Adds a state flag.
    pub fn add(&mut self, state: ElementState);

    /// Removes a state flag.
    pub fn remove(&mut self, state: ElementState);

    /// Checks if specific state is set.
    pub fn contains(&self, state: ElementState) -> bool;

    /// Toggles a state flag.
    pub fn toggle(&mut self, state: ElementState);

    /// Checks if state matches a CSS pseudo-class.
    pub fn matches_pseudo_class(&self, pseudo: &NonTSPseudoClass) -> bool;
}

/// State flags (bitflags).
impl ElementState {
    pub const HOVER: ElementState;
    pub const FOCUS: ElementState;
    pub const ACTIVE: ElementState;
    pub const DISABLED: ElementState;
    pub const CHECKED: ElementState;
    pub const DRAGGING: ElementState;
    pub const DRAG_OVER: ElementState;
    pub const SELECTED: ElementState;
    pub const EXPANDED: ElementState;
    pub const COLLAPSED: ElementState;
    pub const VISITED: ElementState;
    pub const TARGET: ElementState;
    pub const FOCUS_WITHIN: ElementState;
    pub const FOCUS_VISIBLE: ElementState;
}
```

---

### RvueElement

```rust
/// Wrapper implementing Element trait for rvue widgets.
pub struct RvueElement<'a> {
    widget: &'a dyn Widget,
    parent: Option<&'a RvueElement<'a>>,
}

impl<'a> RvueElement<'a> {
    /// Creates a new element wrapper.
    pub fn new(widget: &'a dyn Widget) -> Self;

    /// Creates element with explicit parent.
    pub fn with_parent(widget: &'a dyn Widget, parent: &'a RvueElement<'a>) -> Self;
}
```

---

## Stylesheet Module (`rvue_style::stylesheet`)

### Stylesheet

```rust
/// Collection of CSS style rules.
pub struct Stylesheet {
    rules: Vec<StyleRule>,
    media_queries: Vec<MediaQuery>,
    keyframes: Vec<AnimationKeyframes>,
}

impl Stylesheet {
    /// Creates an empty stylesheet.
    pub fn new() -> Self;

    /// Adds CSS rules from string.
    ///
    /// # Arguments
    /// * `css` - CSS rule string
    ///
    /// # Returns
    /// Result indicating success or parse error.
    pub fn add_rule(&mut self, css: &str) -> Result<(), ParseError>;

    /// Adds CSS rules from file.
    pub fn add_from_file(&mut self, path: &Path) -> Result<(), std::io::Error>;

    /// Adds a single style rule directly.
    pub fn add(&mut self, rule: StyleRule);
}
```

**Preconditions**:
- `add_rule()`: CSS string must be valid UTF-8
- `add_from_file()`: Path must exist and be readable

**Postconditions**:
- Rules are added to the stylesheet

**Errors**:
- `add_rule()`: `ParseError` for invalid CSS syntax
- `add_from_file()`: `std::io::Error` for file access errors

---

### StyleResolver

```rust
/// Resolves CSS rules against widgets.
pub struct StyleResolver {
    stylesheet: Stylesheet,
    caches: SelectorCaches,
}

impl StyleResolver {
    /// Creates resolver with given stylesheet.
    pub fn new(stylesheet: Stylesheet) -> Self;

    /// Resolves styles for a widget.
    ///
    /// # Arguments
    /// * `widget` - The widget to resolve styles for
    ///
    /// # Returns
    /// ComputedStyles with all matching rules applied.
    pub fn resolve(&self, widget: &dyn Widget) -> ComputedStyles;
}
```

---

## Reactive Module (`rvue_style::reactive`)

### ReactiveProperty

```rust
/// Property that can be either static or reactive.
pub enum ReactiveProperty<T: Clone + 'static> {
    /// Static value that never changes.
    Static(T),
    /// Value derived from a signal.
    Signal(ReadSignal<T>),
}

impl<T: Clone + 'static> ReactiveProperty<T> {
    /// Gets the current value.
    pub fn get(&self) -> T;

    /// Checks if this property is reactive.
    pub fn is_reactive(&self) -> bool;
}

impl<T: Clone + 'static> From<T> for ReactiveProperty<T> {
    fn from(value: T) -> Self;
}

impl<T: Clone + 'static> From<ReadSignal<T>> for ReactiveProperty<T> {
    fn from(signal: ReadSignal<T>) -> Self;
}
```

---

## Widget Module (`rvue_style::widget`)

### StyledWidgetExt

```rust
/// Extension trait for adding styles to widgets.
pub trait StyledWidgetExt: Sized {
    /// Adds a property to the widget.
    fn with_style<P: Property>(self, property: P) -> Self;

    /// Sets background color.
    fn style_background<C: Into<Color>>(self, color: C) -> Self;

    /// Sets text color.
    fn style_color<C: Into<Color>>(self, color: C) -> Self;

    /// Sets padding.
    fn style_padding(self, padding: f32) -> Self;

    /// Sets margin.
    fn style_margin(self, margin: f32) -> Self;

    /// Sets font size.
    fn style_font_size(self, size: f32) -> Self;
}
```

---

## Prelude

```rust
/// Commonly used types from the style system.
pub mod prelude {
    pub use super::property::{Property, Properties};
    pub use super::properties::{
        BackgroundColor, Color, Display, FlexDirection, JustifyContent,
        AlignItems, Padding, Margin, Size, TextColor, Width, Height,
    };
    pub use super::selectors::{ElementState, RvueElement};
    pub use super::stylesheet::{Stylesheet, StyleResolver};
    pub use super::widget::StyledWidgetExt;

    // Re-export from rvue for convenience
    pub use rvue::prelude::{Gc, ReadSignal};
}
```

---

## Error Types

### ParseError

```rust
/// Error during CSS parsing.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    message: String,
    position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for ParseError {}
```

**Fields**:
- `message`: Human-readable error description
- `position`: Byte position in input where error occurred
