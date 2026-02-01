# Research: Rvue Style System Implementation

**Feature**: 001-style-system | **Date**: 2026-01-27

## Overview

Research findings from analyzing Xilem/Masonry property system and Stylo selectors crate to inform rvue style system implementation.

---

## Decision 1: Property Trait Design

**Decision**: Adopt Xilem/Masonry `Property` trait with `static_default()` method.

### Rationale

The Xilem/Masonry property system provides:
- Type-safe property storage via `AnyMap`
- Compile-time default values via `static_default()` method
- Zero-cost abstraction for property access
- `Default + Send + Sync + 'static` bounds for safety

From `masonry_core/src/core/properties.rs`:
```rust
pub trait Property: Default + Send + Sync + 'static {
    fn static_default() -> &'static Self;
}
```

This pattern allows `PropertiesRef::get()` to return `&P` (not `Option<&P>`) by falling back to static default.

### Alternatives Considered

1. **Simple Default trait** - Would require Option type for missing properties, less ergonomic
2. **Const generics approach** - Not yet stable in Rust for this use case
3. **External crate (derivative)** - Would add unnecessary dependency

---

## Decision 2: Properties Container with AnyMap

**Decision**: Implement custom `Properties` struct using `rudo_gc`-compatible `AnyMap`.

### Rationale

Xilem/Masonry uses `masonry_core::util::AnyMap` which wraps `std::any::TypeId` keyed map. For rvue, we need:
- `rudo_gc::Trace` implementation for GC compatibility
- Type-safe property insertion and retrieval
- Builder pattern for fluent API

From `masonry_core/src/core/properties.rs`:
```rust
pub struct Properties {
    pub(crate) map: AnyMap,
}

impl Properties {
    pub fn new() -> Self { Self { map: AnyMap::new() } }
    pub fn with<P: Property>(mut self, value: P) -> Self {
        self.map.insert(value);
        self
    }
    pub fn get<P: Property>(&self) -> Option<&P> {
        self.map.get::<P>()
    }
}
```

### Implementation Notes

- Need to implement `rudo_gc::Trace` for `Properties` and all property types
- Use `Gc<ComputedStyles>` for shared style objects (per FR-006)
- Consider lazy computation for performance (only compute when needed)

---

## Decision 3: Selectors Crate Integration

**Decision**: Use Stylo `selectors` crate (not full Stylo) with custom `RvueElement` implementation.

### Rationale

The `selectors` crate provides:
- CSS Selector Level 4 support
- High-performance matching with Bloom filter optimization
- Generic `Element` trait for DOM abstraction
- Pseudo-class matching (`:hover`, `:focus`, etc.)

From `stylo/selectors/lib.rs`:
```rust
pub use crate::parser::{Parser, SelectorImpl, SelectorList};
pub use crate::tree::{Element, OpaqueElement};
```

### Implementation Pattern

From `stylo/selectors/matching.rs`, the key function is:
```rust
pub fn matches_selector<E>(
    selector: &Selector<E::Impl>,
    _pseudo_element: Option<E::PseudoElement>,
    context: &mut MatchingContext<E::Impl>,
    element: &E,
) -> bool
```

### Required: RvueElement Wrapper

Need to implement `Element` trait for rvue widgets:
```rust
pub struct RvueElement<'a> {
    widget: &'a dyn Widget,
    parent: Option<&'a RvueElement<'a>>,
}

impl<'a> Element for RvueElement<'a> {
    type Impl = RvueSelectorImpl;
    // Implement all Element trait methods...
}
```

---

## Decision 4: ElementState Mapping

**Decision**: Map Stylo pseudo-classes to custom `ElementState` bitflags.

### Rationale

CSS pseudo-classes require widget state tracking. Need to map:
- `:hover` → `ElementState::HOVER`
- `:focus` → `ElementState::FOCUS`
- `:active` → `ElementState::ACTIVE`
- `:disabled` → `ElementState::DISABLED`
- `:checked` → `ElementState::CHECKED`

From the spec, `ElementState` uses `bitflags!` for efficient state tracking:
```rust
bitflags! {
    pub struct ElementState: u64 {
        const HOVER = 1 << 0;
        const FOCUS = 1 << 1;
        const ACTIVE = 1 << 2;
        const DISABLED = 1 << 3;
        const CHECKED = 1 << 4;
        // ... more states
    }
}
```

### Implementation

Need to implement `match_non_ts_pseudo_class` in `RvueElement`:
```rust
fn match_non_ts_pseudo_class(
    &self,
    pseudo: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
) -> bool {
    self.widget.state().matches_pseudo_class(pseudo)
}
```

---

## Decision 5: CSS Parsing with cssparser

**Decision**: Use `cssparser` crate for CSS value parsing, implement property-specific parsers.

### Rationale

`cssparser` provides:
- Low-level CSS tokenization
- Color parsing (hex, rgb, named colors)
- Length/value parsing helpers
- Well-tested and maintained

From `stylo/style_derive/to_css.rs` patterns, need:
- `CssValueParser` trait for custom properties
- `ColorParser` for color values
- `LengthParser` for dimension values

### Property Parsing Strategy

```rust
pub trait CssValueParser {
    type Value;
    fn parse(value: &str) -> Result<Self::Value, ParseError>;
    fn to_css(value: &Self::Value) -> String;
}

pub struct ColorParser;
impl CssValueParser for ColorParser {
    type Value = Color;
    fn parse(value: &str) -> Result<Self::Value, ParseError> { ... }
}
```

---

## Decision 6: Reactive Style Integration

**Decision**: Integrate with rvue signal system via `ReactiveProperty<T>` enum.

### Rationale

Need to support both static and signal-driven properties for:
- Static styling (unchanging values)
- Reactive styling (bound to signals)

From the spec:
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
}
```

### Integration Points

- `create_style_effect()` for reactive style updates
- `derive_style()` for derived computed styles
- Widget state signals for pseudo-class matching

---

## Decision 7: Style Resolution and Cascade

**Decision**: Implement CSS cascade with specificity ordering.

### Rationale

CSS specificity rules must be respected:
1. Inline styles (highest)
2. ID selectors
3. Class/attribute/pseudo-class selectors
4. Element selectors (lowest)

From `stylo/selectors/matching.rs`, specificity is tracked in selector:
```rust
pub struct Specificity {
    pub id: u32,
    pub class: u32,
    pub element: u32,
}
```

### Resolution Strategy

```rust
pub struct StyleResolver {
    stylesheet: Stylesheet,
    caches: SelectorCaches,
}

impl StyleResolver {
    pub fn resolve<'a>(&self, element: &RvueElement<'a>) -> ComputedStyles {
        // Sort rules by specificity (descending)
        // Apply matching rules in order
        // Merge properties into ComputedStyles
    }
}
```

---

## Alternatives Considered

| Alternative | Reason for Rejection |
|-------------|---------------------|
| Full Stylo browser engine | Too heavy, unnecessary for GUI widget styling |
| Custom CSS parser from scratch | Re-inventing wheel, cssparser is well-tested |
| Runtime property validation | Violates compile-time safety principle |
| GC-independent design | Would require manual memory management |

---

## Key Files Referenced

| Source | Location | Key Patterns |
|--------|----------|--------------|
| Xilem/Masonry | `learn-projects/xilem/masonry_core/src/core/properties.rs` | Property trait, Properties container |
| Xilem/Masonry | `learn-projects/xilem/masonry/src/properties/background.rs` | Property implementation pattern |
| Stylo Selectors | `learn-projects/stylo/selectors/lib.rs` | Element trait, SelectorImpl |
| Stylo Selectors | `learn-projects/stylo/selectors/matching.rs` | matches_selector, MatchingContext |

---

## Open Questions (Resolved)

1. **AnyMap for rudo-gc** - Will implement custom `rudo_gc::Trace` compatible version
2. **Pseudo-class mapping** - Direct mapping via ElementState bitflags
3. **Signal integration** - Via ReactiveProperty<T> enum pattern
4. **ComputedStyles ownership** - Clone-on-write with Gc<ComputedStyles> for sharing
