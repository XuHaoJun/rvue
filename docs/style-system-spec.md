# Rvue Style System Technical Specification

> **Version**: 1.0.0
> **Date**: 2026-01-27
> **Status**: Draft
> **Based on**: Stylo selectors crate + Xilem/Masonry Property system

## 1. Executive Summary

This document specifies the **rvue style system**, a hybrid approach combining:
- **Stylo selectors crate** (`selectors`) for CSS selector parsing and matching
- **Masonry Property trait** for type-safe, compile-time styling
- **Signal integration** for reactive style updates
- **rudo-gc integration** for memory management

### Key Design Principles

1. **Type Safety First**: Use Rust's type system to catch styling errors at compile time
2. **CSS Familiarity**: Provide CSS selector syntax for state-based styling (`:hover`, `:focus`)
3. **Reactivity Native**: Integrate with rvue's signal system for fine-grained updates
4. **GC-Friendly**: Design for rudo-gc memory management from the ground up
5. **Minimal Dependencies**: Use only `selectors` crate, not full Stylo

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      rvue Style System                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │   Selectors      │    │    Property      │                   │
│  │   (CSS Matching) │    │   (Type-Safe)    │                   │
│  └────────┬─────────┘    └────────┬─────────┘                   │
│           │                       │                              │
│           │    ┌──────────────────┘                              │
│           │    │                                                 │
│           ▼    ▼                                                 │
│  ┌──────────────────────────────────────────┐                   │
│  │            Style Resolver                │                   │
│  │  - Selector matching                     │                   │
│  │  - Specificity resolution                │                   │
│  │  - Property cascade                      │                   │
│  └────────────────┬─────────────────────────┘                   │
│                   │                                               │
│                   ▼                                               │
│  ┌──────────────────────────────────────────┐                   │
│  │         Widget + Signal Integration      │                   │
│  └──────────────────────────────────────────┘                   │
│                   │                                               │
│                   ▼                                               │
│  ┌──────────────────────────────────────────┐                   │
│  │              Taffy Layout                 │                   │
│  └──────────────────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Selectors Crate Integration

### 3.1 Overview

The `selectors` crate from Mozilla Stylo provides:
- CSS Selector Level 4 support
- High-performance matching with Bloom filter optimization
- Generic DOM abstraction via `Element` trait
- Pseudo-class matching (`:hover`, `:focus`, `:active`, etc.)

### 3.2 Dependency Configuration

```toml
# crates/rvue-style/Cargo.toml
[dependencies]
selectors = "0.35"
cssparser = "0.36"
bitflags = "2"
rustc-hash = "2"
smallvec = "1.0"
```

### 3.3 Element Trait Implementation

Every rvue widget must implement the `selectors::Element` trait:

```rust
use selectors::Element;

pub struct RvueElement<'a> {
    widget: &'a dyn Widget,
    parent: Option<&'a RvueElement<'a>>,
}

impl<'a> Element for RvueElement<'a> {
    type Impl = RvueSelectorImpl;

    fn parent_element(&self) -> Option<Self> {
        self.widget.parent().map(|p| RvueElement {
            widget: p,
            parent: None,
        })
    }

    fn is_root(&self) -> bool {
        self.widget.parent().is_none()
    }

    fn has_local_name(&self, name: &<Self::Impl as SelectorImpl>::LocalName) -> bool {
        self.widget.widget_name() == name.as_ref()
    }

    fn has_class(&self, name: &<Self::Impl as SelectorImpl>::AtomIdent, _: CaseSensitivity) -> bool {
        self.widget.classes().contains(name.as_ref())
    }

    fn has_id(&self, name: &<Self::Impl as SelectorImpl>::AtomIdent, _: CaseSensitivity) -> bool {
        self.widget.id() == Some(name.as_ref())
    }

    fn attr_matches(&self, attr: &AttrSelector<Self::Impl>) -> bool {
        // Attribute selector matching
        self.widget.get_attribute(attr.name.as_ref())
            .map(|value| attr.operation.eval_str(&value))
            .unwrap_or(false)
    }

    fn match_pseudo_element(
        &self,
        pseudo: &<Self::Impl as SelectorImpl>::PseudoElement,
    ) -> bool {
        // ::before, ::after, etc.
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        pseudo: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
    ) -> bool {
        // :hover, :focus, :active, :disabled, etc.
        self.widget.state().matches_pseudo_class(pseudo)
    }
}
```

### 3.4 Pseudo-Class Support

The style system supports all standard CSS pseudo-classes:

| Category | Pseudo-Classes |
|----------|---------------|
| **User Action** | `:hover`, `:focus`, `:focus-within`, `:active`, `:focus-visible` |
| **State** | `:enabled`, `:disabled`, `:checked`, `:unchecked`, `:indeterminate` |
| **Structural** | `:first-child`, `:last-child`, `:nth-child(n)`, `:nth-last-child(n)`, `:only-child` |
| **Validation** | `:valid`, `:invalid`, `:required`, `:optional`, `:in-range`, `:out-of-range` |
| **Language** | `:lang(en)`, `:dir(ltr)` |

### 3.5 ElementState Tracking

```rust
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ElementState: u64 {
        const HOVER = 1 << 0;
        const FOCUS = 1 << 1;
        const ACTIVE = 1 << 2;
        const DISABLED = 1 << 3;
        const CHECKED = 1 << 4;
        const DRAGGING = 1 << 5;
        const DRAG_OVER = 1 << 6;
        const SELECTED = 1 << 7;
        const EXPANDED = 1 << 8;
        const COLLAPSED = 1 << 9;
        const VISITED = 1 << 10;
        const TARGET = 1 << 11;
        const FOCUS_WITHIN = 1 << 12;
        const FOCUS_VISIBLE = 1 << 13;
    }
}

pub trait ElementStateTracker {
    fn state(&self) -> ElementState;
    fn set_state(&self, state: ElementState);
    fn add_state(&self, state: ElementState);
    fn remove_state(&self, state: ElementState);
}
```

---

## 4. Property System

### 4.1 Property Trait Definition

Inspired by Xilem/Masonry, the property system provides compile-time type safety:

```rust
pub trait Property: Default + Send + Sync + 'static {
    fn static_default() -> &'static Self;
}

pub trait HasProperty<P: Property> {}
```

### 4.2 Properties Container

```rust
pub struct Properties {
    map: AnyMap,
}

impl Properties {
    pub fn new() -> Self {
        Self { map: AnyMap::new() }
    }

    pub fn with<P: Property>(mut self, value: P) -> Self {
        self.map.insert(value);
        self
    }

    pub fn get<P: Property>(&self) -> Option<&P> {
        self.map.get::<P>()
    }

    pub fn insert<P: Property>(&mut self, value: P) -> Option<P> {
        self.map.insert(value)
    }

    pub fn remove<P: Property>(&mut self) -> Option<P> {
        self.map.remove::<P>()
    }
}

impl<P: Property> From<P> for Properties {
    fn from(prop: P) -> Self {
        Self::one(prop)
    }
}
```

### 4.3 Core Properties

#### Background

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Color(Color),
    Gradient(Gradient),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundColor(pub Color);

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundImage(pub Image);

impl Property for Background {
    fn static_default() -> &'static Self {
        static DEFAULT: Background = Background::Color(Color::TRANSPARENT);
        &DEFAULT
    }
}

impl Property for BackgroundColor {
    fn static_default() -> &'static Self {
        static DEFAULT: BackgroundColor = BackgroundColor(Color::TRANSPARENT);
        &DEFAULT
    }
}
```

#### Foreground/Text

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Color(pub RgbColor);

#[derive(Clone, Debug, PartialEq)]
pub struct FontFamily(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct FontSize(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontStyle {
    pub family: FontFamily,
    pub size: FontSize,
    pub weight: FontWeight,
    pub stretch: f32,
}

impl Property for Color { fn static_default() -> &'static Self { ... } }
impl Property for FontFamily { fn static_default() -> &'static Self { ... } }
impl Property for FontSize { fn static_default() -> &'static Self { ... } }
impl Property for FontWeight { fn static_default() -> &'static Self { ... } }
impl Property for FontStyle { fn static_default() -> &'static Self { ... } }
```

#### Spacing

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Padding(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct PaddingRect {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Margin(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct MarginRect {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub fn uniform(value: f32) -> Self { Self(value) }
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self { /* ... */ }
    pub fn all(top: f32, right: f32, bottom: f32, left: f32) -> Self { /* ... */ }
}
```

#### Border

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct BorderColor(pub Color);

#[derive(Clone, Debug, PartialEq)]
pub struct BorderWidth(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct BorderRadius(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Border {
    pub width: BorderWidth,
    pub color: BorderColor,
    pub style: BorderStyle,
}
```

#### Layout

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlexGrow(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct FlexShrink(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct FlexBasis(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct Gap(pub f32);

#[derive(Clone, Debug, PartialEq)]
pub struct ZIndex(pub i32);
```

#### Sizing

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Size {
    Auto,
    Pixels(f32),
    Percent(f32),
    MinContent,
    MaxContent,
    FitContent,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Width(pub Size);

#[derive(Clone, Debug, PartialEq)]
pub struct Height(pub Size);

#[derive(Clone, Debug, PartialEq)]
pub struct MinWidth(pub Size);

#[derive(Clone, Debug, PartialEq)]
pub struct MinHeight(pub Size);

#[derive(Clone, Debug, PartialEq)]
pub struct MaxWidth(pub Size);

#[derive(Clone, Debug, PartialEq)]
pub struct MaxHeight(pub Size);
```

#### Visibility

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opacity(pub f32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Display {
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}
```

#### Effects

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub translate_x: Option<f32>,
    pub translate_y: Option<f32>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub rotate: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor(pub CursorIcon);
```

### 4.4 Property Change Handlers

Each property can define how changes affect the widget:

```rust
impl Property for BackgroundColor {
    fn static_default() -> &'static Self {
        static DEFAULT: BackgroundColor = BackgroundColor(Color::TRANSPARENT);
        &DEFAULT
    }
}

impl BackgroundColor {
    pub fn on_change(ctx: &mut UpdateCtx<'_>) {
        ctx.request_paint();
    }
}

impl Property for Width {
    fn static_default() -> &'static Self {
        static DEFAULT: Width = Width(Size::Auto);
        &DEFAULT
    }
}

impl Width {
    pub fn on_change(ctx: &mut UpdateCtx<'_>) {
        ctx.request_layout();
    }
}
```

---

## 5. Style Rule System

### 5.1 StyleRule Definition

```rust
pub struct StyleRule<E: Element> {
    pub selector: Selector<E::Impl>,
    pub specificity: Specificity,
    pub properties: Properties,
}

pub struct Stylesheet {
    rules: Vec<StyleRule<RvueElement<'static>>>,
    media_queries: Vec<MediaQuery>,
    keyframes: Vec<AnimationKeyframes>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            media_queries: Vec::new(),
            keyframes: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, css: &str) -> Result<(), ParseError> {
        let parsed = parse_stylesheet(css)?;
        self.rules.extend(parsed);
        Ok(())
    }
}
```

### 5.2 Selector Parsing

```rust
pub fn parse_stylesheet(css: &str) -> Result<Vec<StyleRule<RvueElement<'static>>>, ParseError> {
    let mut rules = Vec::new();
    let mut parser = CssParser::new(css);

    while !parser.is_exhausted() {
        let rule = parse_style_rule(&mut parser)?;
        rules.push(rule);
    }

    Ok(rules)
}

fn parse_style_rule<'a>(parser: &mut CssParser<'a>) -> Result<StyleRule<'a>, ParseError> {
    let selector_text = parser.expect_ident()?;

    // Parse the selector
    let selector = Selector::parse(&selector_text)?;

    // Parse properties block
    parser.expect_curly_bracket_open()?;
    let mut properties = Properties::new();

    while !parser.is_exhausted() && !parser.is_at_end_of_block() {
        let property_name = parser.expect_ident()?;
        parser.expect_colon()?;
        let value = parse_property_value(&property_name, parser)?;
        properties.insert(value);
        parser.expect_semicolon()?;
    }

    parser.expect_curly_bracket_close()?;

    Ok(StyleRule {
        selector,
        specificity: selector.specificity(),
        properties,
    })
}
```

### 5.3 Specificity Resolution

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub id: u32,
    pub class: u32,
    pub element: u32,
}

impl Specificity {
    pub fn new(id: u32, class: u32, element: u32) -> Self {
        Self { id, class, element }
    }

    pub fn max() -> Self {
        Self {
            id: u32::MAX,
            class: u32::MAX,
            element: u32::MAX,
        }
    }
}

impl PartialEq for StyleRule<E> {
    fn eq(&self, other: &Self) -> bool {
        self.specificity == other.specificity
    }
}

impl PartialOrd for StyleRule<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.specificity.partial_cmp(&other.specificity)
    }
}
```

### 5.4 Matching and Cascade

```rust
pub struct StyleResolver {
    stylesheet: Stylesheet,
    caches: SelectorCaches,
}

impl StyleResolver {
    pub fn new(stylesheet: Stylesheet) -> Self {
        Self {
            stylesheet,
            caches: SelectorCaches::new(),
        }
    }

    pub fn resolve<'a>(&self, element: &RvueElement<'a>) -> ComputedStyles {
        let mut context = MatchingContext::new(
            MatchingMode::Normal,
            None,
            &mut self.caches,
            QuirksMode::NoQuirks,
            NeedsSelectorFlags::No,
        );

        let mut computed = ComputedStyles::default();

        // Sort rules by specificity (descending)
        let mut sorted_rules: Vec<_> = self.stylesheet.rules.iter().collect();
        sorted_rules.sort_by(|a, b| b.specificity.cmp(&a.specificity));

        // Apply matching rules
        for rule in sorted_rules {
            if matches_selector(&rule.selector, 0, None, element, &mut context) {
                computed.merge(&rule.properties);
            }
        }

        computed
    }
}

#[derive(Default)]
pub struct ComputedStyles {
    background_color: Option<BackgroundColor>,
    color: Option<Color>,
    font_size: Option<FontSize>,
    // ... more fields
}

impl ComputedStyles {
    pub fn merge(&mut self, other: &Properties) {
        if let Some(bg) = other.get::<BackgroundColor>() {
            self.background_color = Some(bg.clone());
        }
        if let Some(c) = other.get::<Color>() {
            self.color = Some(c.clone());
        }
        // ... more properties
    }
}
```

---

## 6. Signal Integration

### 6.1 Reactive Properties

The style system integrates with rvue's signal system for reactive updates:

```rust
pub enum ReactiveProperty<T: Clone + 'static> {
    Static(T),
    Signal(ReadSignal<T>),
}

impl<T: Clone + 'static> ReactiveProperty<T> {
    pub fn get(&self) -> T {
        match self {
            Self::Static(value) => value.clone(),
            Self::Signal(signal) => signal.get(),
        }
    }

    pub fn is_reactive(&self) -> bool {
        matches!(self, Self::Signal(_))
    }
}

impl<T: Clone + 'static> From<ReadSignal<T>> for ReactiveProperty<T> {
    fn from(signal: ReadSignal<T>) -> Self {
        Self::Signal(signal)
    }
}

impl<T: Clone + 'static> From<T> for ReactiveProperty<T> {
    fn from(value: T) -> Self {
        Self::Static(value)
    }
}
```

### 6.2 Derived Styles

```rust
pub fn derive_style<F>(
    base_style: ComputedStyles,
    derive_fn: F,
) -> ReadSignal<ComputedStyles>
where
    F: Fn(&ComputedStyles) -> ComputedStyles + 'static,
{
    create_memo(move |_| derive_fn(&base_style))
}
```

### 6.3 Style Effects

```rust
pub fn create_style_effect<F>(
    stylesheet: Stylesheet,
    element: Gc<Component>,
    f: F,
) -> Gc<Effect>
where
    F: Fn(&ComputedStyles) + 'static,
{
    create_effect(move || {
        let resolved = resolve_styles_for_element(&stylesheet, element);
        f(&resolved);
    })
}
```

---

## 7. GC Integration

### 7.1 rudo-gc Compatibility

All style objects implement `rudo_gc::Trace`:

```rust
unsafe impl rudo_gc::Trace for Properties {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.map.values().for_each(|prop| {
            prop.trace(visitor);
        });
    }
}

unsafe impl rudo_gc::Trace for ComputedStyles {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.background_color.trace(visitor);
        self.color.trace(visitor);
        self.font_size.trace(visitor);
        // ... more properties
    }
}
```

### 7.2 Shared Styles

Styles can be shared across widgets using `Gc`:

```rust
pub type SharedStyles = Gc<ComputedStyles>;

impl ComputedStyles {
    pub fn shared(self) -> SharedStyles {
        Gc::new(self)
    }
}

pub struct WidgetStyles {
    base: SharedStyles,
    overrides: Properties,
}

impl WidgetStyles {
    pub fn new(base: SharedStyles) -> Self {
        Self {
            base,
            overrides: Properties::new(),
        }
    }

    pub fn with_override<P: Property>(mut self, property: P) -> Self {
        self.overrides.insert(property);
        self
    }

    pub fn computed(&self) -> ComputedStyles {
        let mut computed = (*self.base).clone();
        computed.merge(&self.overrides);
        computed
    }
}
```

---

## 8. CSS Parsing Support

### 8.1 Supported CSS Properties

| Category | Properties |
|----------|------------|
| **Color** | `color`, `background-color`, `border-color` |
| **Typography** | `font-family`, `font-size`, `font-weight`, `line-height` |
| **Spacing** | `padding`, `padding-top`, `padding-right`, `padding-bottom`, `padding-left`, `margin`, `margin-top`, etc. |
| **Border** | `border`, `border-width`, `border-style`, `border-color`, `border-radius` |
| **Layout** | `display`, `flex-direction`, `justify-content`, `align-items`, `gap`, `flex-grow`, `flex-shrink` |
| **Sizing** | `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height` |
| **Visibility** | `opacity`, `visibility`, `z-index` |
| **Effects** | `box-shadow`, `transform`, `cursor` |

### 8.2 CSS Value Parsing

```rust
pub trait CssValueParser {
    type Value;

    fn parse(value: &str) -> Result<Self::Value, ParseError>;
    fn to_css(value: &Self::Value) -> String;
}

pub struct ColorParser;

impl CssValueParser for ColorParser {
    type Value = Color;

    fn parse(value: &str) -> Result<Self::Value, ParseError> {
        if value.starts_with('#') {
            parse_hex_color(value)
        } else if value.starts_with("rgb") {
            parse_rgb_color(value)
        } else {
            parse_named_color(value)
        }
    }

    fn to_css(value: &Self::Value) -> String {
        format!("rgb({}, {}, {})", value.r, value.g, value.b)
    }
}

pub struct LengthParser;

impl CssValueParser for LengthParser {
    type Value = f32;

    fn parse(value: &str) -> Result<Self::Value, ParseError> {
        if value.ends_with("px") {
            let num = &value[..value.len() - 2];
            Ok(num.parse::<f32>()?)
        } else if value.ends_with('%') {
            let num = &value[..value.len() - 1];
            Ok(num.parse::<f32>()? / 100.0)
        } else {
            Err(ParseError::new("Invalid length value"))
        }
    }
}
```

---

## 9. Widget Integration

### 9.1 Style Trait

```rust
pub trait StyledWidget: Widget {
    type Style: Property;

    fn style(&self) -> &Properties;

    fn set_style(&mut self, style: Properties);
}

pub trait StyledWidgetExt: Widget {
    fn with_style<P: Property>(self, property: P) -> Self;

    fn style_background<C: Into<Color>>(self, color: C) -> Self;

    fn style_padding(self, padding: f32) -> Self;

    fn style_margin(self, margin: f32) -> Self;

    fn style_font_size(self, size: f32) -> Self;

    fn style_color<C: Into<Color>>(self, color: C) -> Self;
}

impl<T: Widget + Sized> StyledWidgetExt for T {
    fn with_style<P: Property>(mut self, property: P) -> Self {
        self.style_mut().insert(property);
        self
    }

    fn style_background<C: Into<Color>>(self, color: C) -> Self {
        self.with_style(BackgroundColor(color.into()))
    }

    fn style_padding(self, padding: f32) -> Self {
        self.with_style(Padding(padding))
    }

    fn style_margin(self, margin: f32) -> Self {
        self.with_style(Margin(margin))
    }

    fn style_font_size(self, size: f32) -> Self {
        self.with_style(FontSize(size))
    }

    fn style_color<C: Into<Color>>(self, color: C) -> Self {
        self.with_style(Color(color.into()))
    }
}
```

### 9.2 Widget Implementation Example

```rust
pub struct Button {
    text: String,
    style: Properties,
    state: ElementState,
}

impl StyledWidget for Button {
    type Style = ButtonStyle;

    fn style(&self) -> &Properties {
        &self.style
    }

    fn set_style(&mut self, style: Properties) {
        self.style = style;
    }
}

impl Widget for Button {
    fn build(&mut self, ctx: &mut BuildContext) -> Gc<Self::State> {
        let state = ButtonState::new(ctx, self);
        state
    }

    fn paint(&self, ctx: &mut PaintCtx, styles: &PropertiesRef) {
        let bg = styles.get::<BackgroundColor>();
        let radius = styles.get::<BorderRadius>();

        ctx.fill(ctx.bounds(), bg.0);
        ctx.rounded_rect(ctx.bounds(), radius.0, |ctx| {
            ctx.fill(ctx.bounds(), bg.0);
        });
    }

    fn layout(&self, ctx: &mut LayoutCtx, styles: &PropertiesRef) {
        let width = styles.get::<Width>();
        let height = styles.get::<Height>();
        let padding = styles.get::<Padding>();

        // Calculate size based on content + padding
        let content_size = self.text.size(styles.get::<FontSize>().0);
        let size = match &width.0 {
            Size::Auto => content_size.width + padding.0 * 2.0,
            Size::Pixels(v) => *v,
            Size::Percent(p) => ctx.parent_width() * *p,
            _ => 100.0, // Default
        };

        ctx.set_size(size);
    }
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Properties::new(),
            state: ElementState::empty(),
        }
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.click_handler = Some(Box::new(handler));
        self
    }
}
```

---

## 10. API Reference

### 10.1 Property Creators

```rust
// Color
Color::rgb(r: u8, g: u8, b: u8) -> Color
Color::rgba(r: u8, g: u8, b: u8, a: u8) -> Color
Color::from_hex(hex: &str) -> Result<Color, ParseError>

// Background
Background::color(color: Color) -> Background
Background::gradient(gradient: Gradient) -> Background

// Spacing
Padding::uniform(value: f32) -> Padding
Padding::symmetric(vertical: f32, horizontal: f32) -> Padding
Padding::all(top: f32, right: f32, bottom: f32, left: f32) -> Padding

// Sizing
Size::auto() -> Size
Size::pixels(value: f32) -> Size
Size::percent(value: f32) -> Size
```

### 10.2 Stylesheet Functions

```rust
Stylesheet::new() -> Stylesheet
Stylesheet::add_rule(&mut self, css: &str) -> Result<(), ParseError>
Stylesheet::add_from_file(&mut self, path: &Path) -> Result<(), std::io::Error>

fn stylesheet(css: &str) -> Stylesheet {
    let mut sheet = Stylesheet::new();
    sheet.add_rule(css).unwrap();
    sheet
}
```

### 10.3 Element State

```rust
ElementState::empty() -> ElementState
ElementState::HOVER
ElementState::FOCUS
ElementState::ACTIVE
ElementState::DISABLED
ElementState::CHECKED

fn ElementState::add(&mut self, state: ElementState)
fn ElementState::remove(&mut self, state: ElementState)
fn ElementState::contains(&self, state: ElementState) -> bool
fn ElementState::toggle(&mut self, state: ElementState)
```

---

## 11. Performance Considerations

### 11.1 Selector Matching Optimization

- **Bloom Filter**: Use bloom filters for descendant selectors to fast-reject non-matching elements
- **Selector Caching**: Cache parsed selectors and matching results
- **Incremental Matching**: Only re-match affected elements when styles change

### 11.2 Memory Efficiency

- **Shared Styles**: Use `Gc<ComputedStyles>` for common styles
- **Lazy Computation**: Only compute styles when needed
- **Property Templates**: Use static defaults to avoid allocation

### 11.3 Update Strategy

```rust
pub enum UpdateStrategy {
    Full,           // Recompute all styles
    Incremental,    // Only changed properties
    Debounced,      // Batch updates
}
```

---

## 12. Implementation Plan

### Phase 1: Core Property System (Week 1)

- [ ] Define `Property` trait
- [ ] Implement `Properties` container with AnyMap
- [ ] Implement core properties (Color, Padding, Margin, etc.)
- [ ] Add rudo-gc Trace implementation
- [ ] Create unit tests

### Phase 2: Selectors Integration (Week 2)

- [ ] Implement `RvueElement` for selectors crate
- [ ] Implement `Element` trait
- [ ] Add `ElementState` tracking
- [ ] Create `SelectorMatcher` utility
- [ ] Implement pseudo-class matching (`:hover`, `:focus`, etc.)

### Phase 3: CSS Parsing (Week 3)

- [ ] Implement CSS value parsers
- [ ] Create stylesheet parser
- [ ] Add support for CSS properties
- [ ] Implement specificity calculation
- [ ] Create stylesheet macro

### Phase 4: Widget Integration (Week 4)

- [ ] Integrate with Widget trait
- [ ] Add StyledWidget extension trait
- [ ] Implement style change handlers
- [ ] Add example widgets (Button, Label, Box)
- [ ] Create examples demonstrating stylesheets

---

## 13. Future Enhancements

- [ ] **CSS Variables**: `--variable-name` support
- [ ] **Animations**: `@keyframes` and transition support
- [ ] **Media Queries**: `@media` breakpoint handling
- [ ] **Theme System**: Dark/light mode themes
- [ ] **Hot Reload**: Development-time style updates
- [ ] **CSS Grid**: Grid layout properties
- [ ] **Text Styling**: Rich text, text shadows, text decoration

---

## 14. Appendix A: File Structure

```
crates/rvue-style/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── property.rs           # Property trait and Properties container
│   ├── properties/
│   │   ├── mod.rs
│   │   ├── background.rs
│   │   ├── border.rs
│   │   ├── color.rs
│   │   ├── font.rs
│   │   ├── layout.rs
│   │   ├── sizing.rs
│   │   ├── spacing.rs
│   │   └── visibility.rs
│   ├── selectors/
│   │   ├── mod.rs
│   │   ├── element.rs        # RvueElement implementation
│   │   └── state.rs          # ElementState tracking
│   ├── stylesheet/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── rule.rs
│   ├── css/
│   │   ├── mod.rs
│   │   ├── value_parser.rs
│   │   └── properties.rs
│   ├── reactive/
│   │   ├── mod.rs
│   │   └── style_signal.rs
│   └── widget/
│       ├── mod.rs
│       └── styled.rs
├── tests/
│   ├── property_test.rs
│   ├── selector_test.rs
│   └── stylesheet_test.rs
└── examples/
    ├── basic_styling.rs
    ├── stylesheet.rs
    └── reactive_styles.rs
```

---

## 15. Appendix B: Reference Implementation

### Xilem/Masonry Property System
- Location: `/learn-projects/xilem/masonry_core/src/core/properties.rs`
- Location: `/learn-projects/xilem/masonry/src/properties/`

### Stylo Selectors Crate
- Location: `/learn-projects/stylo/selectors/`
- Key files: `lib.rs`, `matching.rs`, `parser.rs`, `attr.rs`

---

## 16. Appendix C: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-27 | Initial specification |

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-27
**Next Review**: MVP completion
