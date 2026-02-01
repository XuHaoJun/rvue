# Data Model: Rvue Style System

**Feature**: 001-style-system | **Date**: 2026-01-27

## Entity Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Style System Data Model                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐      ┌─────────────┐                           │
│  │  Property   │◄─────┤ Properties  │                           │
│  │  (Trait)    │      │  Container  │                           │
│  └─────────────┘      └─────────────┘                           │
│         │                     │                                  │
│         ▼                     ▼                                  │
│  ┌─────────────┐      ┌─────────────┐                           │
│  │  Concrete   │      │ComputedStyles│                          │
│  │  Properties │      │  (Result)   │                           │
│  └─────────────┘      └─────────────┘                           │
│                           │                                      │
│         ┌─────────────────┼─────────────────┐                   │
│         ▼                 ▼                 ▼                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ StyleRule   │  │ElementState │  │StyleResolver│             │
│  │             │  │             │  │             │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                 │                                    │
│         ▼                 │                                    │
│  ┌─────────────┐          │                                    │
│  │ Stylesheet  │──────────┘                                    │
│  │             │                                               │
│  └─────────────┘                                               │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Entities

### Property (Trait)

**Purpose**: Marker trait for type-safe styling attributes.

```rust
pub trait Property: Default + Send + Sync + 'static {
    fn static_default() -> &'static Self;
}
```

**Implementors**: All concrete property types (Color, Padding, FontSize, etc.)

**Validation Rules**:
- Must implement `Default`
- Must be `Send + Sync + 'static` for GC compatibility
- `static_default()` must return same value as `Default::default()`

---

### Properties (Container)

**Purpose**: Collection of property values for a widget.

```rust
pub struct Properties {
    pub(crate) map: AnyMap,
}

impl Properties {
    pub fn new() -> Self;
    pub fn with<P: Property>(value: P) -> Self;
    pub fn get<P: Property>(&self) -> Option<&P>;
    pub fn insert<P: Property>(&mut self, value: P) -> Option<P>;
    pub fn remove<P: Property>(&mut self) -> Option<P>;
}
```

**Relationships**:
- Contains zero or more `Property` instances
- Each property type has at most one value (last insert wins)
- Ordered by insertion time for cascade resolution

**Lifecycle**:
- Created during widget construction
- Mutated via `set_style()` or property-specific methods
- Shared between parent and child widgets via inheritance

---

### Concrete Properties

#### Color Properties

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Color(pub RgbColor);

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundColor(pub Color);

#[derive(Clone, Debug, PartialEq)]
pub struct TextColor(pub Color);
```

#### Spacing Properties

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
pub struct MarginRect { /* ... */ }
```

#### Layout Properties

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Display {
    Flex,
    Grid,
    Block,
    Inline,
    InlineBlock,
    None,
}

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
```

#### Sizing Properties

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

---

### ElementState

**Purpose**: Tracks widget interaction states for pseudo-class matching.

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

impl ElementState {
    pub fn empty() -> Self;
    pub fn add(&mut self, state: ElementState);
    pub fn remove(&mut self, state: ElementState);
    pub fn contains(&self, state: ElementState) -> bool;
    pub fn toggle(&mut self, state: ElementState);
    pub fn matches_pseudo_class(&self, pseudo: &NonTSPseudoClass) -> bool;
}
```

**State Transitions**:

```
HOVER:    clear → set (mouse enter) → clear (mouse leave)
FOCUS:    clear → set (gain focus) → clear (lose focus)
ACTIVE:   clear → set (mouse down) → clear (mouse up)
DISABLED: clear ↔ set (property change)
CHECKED:  clear ↔ set (property change, toggle)
```

---

### RvueElement<'a>

**Purpose**: Wrapper implementing selectors::Element trait for rvue widgets.

```rust
pub struct RvueElement<'a> {
    widget: &'a dyn Widget,
    parent: Option<&'a RvueElement<'a>>,
}

impl<'a> Element for RvueElement<'a> {
    type Impl = RvueSelectorImpl;

    fn parent_element(&self) -> Option<Self>;
    fn is_root(&self) -> bool;
    fn has_local_name(&self, name: &<Self::Impl as SelectorImpl>::LocalName) -> bool;
    fn has_class(&self, name: &<Self::Impl as SelectorImpl>::AtomIdent, _: CaseSensitivity) -> bool;
    fn has_id(&self, name: &<Self::Impl as SelectorImpl>::AtomIdent, _: CaseSensitivity) -> bool;
    fn attr_matches(&self, attr: &AttrSelector<Self::Impl>) -> bool;
    fn match_pseudo_element(&self, pseudo: &<Self::Impl as SelectorImpl>::PseudoElement) -> bool;
    fn match_non_ts_pseudo_class(&self, pseudo: &<Self::Impl as SelectorImpl>::NonTSPseudoClass) -> bool;
}
```

**Relationships**:
- Represents a widget in the selector matching tree
- Parent relationship enables ancestor/descendant selectors
- Widget provides state and attribute data

---

### StyleRule

**Purpose**: CSS rule containing selector and associated properties.

```rust
pub struct StyleRule<E: Element> {
    pub selector: Selector<E::Impl>,
    pub specificity: Specificity,
    pub properties: Properties,
}
```

**Specificity Calculation**:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub id: u32,       // ID selectors (#id)
    pub class: u32,    // Class (.), attribute, pseudo-class
    pub element: u32,  // Element, pseudo-element
}
```

**Resolution Order**:
1. Higher specificity wins
2. Equal specificity: later declaration wins (source order)
3. User agent styles < user styles < author styles

---

### Stylesheet

**Purpose**: Collection of style rules.

```rust
pub struct Stylesheet {
    pub rules: Vec<StyleRule<RvueElement<'static>>>,
    pub media_queries: Vec<MediaQuery>,
    pub keyframes: Vec<AnimationKeyframes>,
}

impl Stylesheet {
    pub fn new() -> Self;
    pub fn add_rule(&mut self, css: &str) -> Result<(), ParseError>;
    pub fn add_from_file(&mut self, path: &Path) -> Result<(), std::io::Error>;
}
```

**Relationships**:
- Contains multiple StyleRule instances
- Rules are applied in order during resolution
- Media queries and keyframes reserved for future phases

---

### ComputedStyles

**Purpose**: Final resolved style values after cascade.

```rust
#[derive(Default)]
pub struct ComputedStyles {
    pub background_color: Option<BackgroundColor>,
    pub color: Option<TextColor>,
    pub font_size: Option<FontSize>,
    pub font_family: Option<FontFamily>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub width: Option<Width>,
    pub height: Option<Height>,
    pub display: Option<Display>,
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    // ... more properties
}

impl ComputedStyles {
    pub fn merge(&mut self, other: &Properties);
}
```

**Lifecycle**:
- Created empty or from default styles
- Populated by StyleResolver via rule matching
- Used during widget paint/layout passes

---

### StyleResolver

**Purpose**: Matches CSS rules against elements and produces computed styles.

```rust
pub struct StyleResolver {
    stylesheet: Stylesheet,
    caches: SelectorCaches,
}

impl StyleResolver {
    pub fn new(stylesheet: Stylesheet) -> Self;
    pub fn resolve<'a>(&self, element: &RvueElement<'a>) -> ComputedStyles;
}
```

**Algorithm**:
1. Create MatchingContext for element
2. Sort stylesheet rules by specificity (descending)
3. For each rule: if matches_selector() returns true, merge properties
4. Return merged ComputedStyles

---

## Validation Rules

### Property Validation

- Color values must be valid RGB/hex/named colors
- Numeric values (padding, margin, etc.) must be non-negative
- Percentage values must be 0.0 to 1.0
- Enum values must match defined variants

### Selector Validation

- All selectors must parse successfully via selectors crate
- Pseudo-classes must be supported (`:hover`, `:focus`, etc.)
- Attribute selectors must have valid operators

### State Transitions

- State changes must be atomic (no intermediate states)
- State must be consistent (can't be both HOVER and NOT_HOVER)
- State changes trigger re-evaluation of matching rules

---

## Performance Considerations

### Memory

- Properties stored in AnyMap (type-keyed)
- ComputedStyles cloned on write (consider Gc<ComputedStyles>)
- Static defaults avoid per-instance allocation

### Caching

- Selector parsing cached (Bloom filter optimization)
- Matching results cached for unchanged elements
- Incremental re-matching for state changes only
