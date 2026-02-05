# AGENTS.md - Rvue Development Guidelines

This document provides guidelines for AI agents working on the Rvue codebase.

## Project Overview

Rvue is a high-performance, GPU-accelerated Rust GUI framework inspired by Vue's developer experience and SolidJS's fine-grained reactivity. The project uses:
- Rust 2021 Edition (minimum 1.75+)
- Vello for GPU-accelerated rendering
- Taffy for CSS-like layouts
- rudo-gc for hybrid garbage collection

## Build Commands

```bash
cargo build
cargo build --release
cargo build -p rvue
cargo build -p rvue-macro
cargo run --bin counter
cargo run --bin list
cargo run --bin layout
cargo run --bin form
```

## Lint and Format

```bash
cargo fmt
cargo fmt --check
cargo clippy
cargo clippy --fix
```

## Testing

```bash
cargo test -- --test-threads=1                          # Run all tests
cargo test -p rvue -- --test-threads=1                  # Tests for rvue crate
cargo test -p rvue-macro -- --test-threads=1            # Tests for rvue-macro crate
cargo test --test signal_test -- --test-threads=1       # Run specific test file
cargo test --test effect_test -- --test-threads=1
cargo test --test component_test -- --test-threads=1
cargo test test_create_signal -- --test-threads=1       # Run single test by name
cargo test test_signal_set -- --test-threads=1
cargo test -- --nocapture --test-threads=1           # Show output for debugging
```

## Code Style Guidelines

### General Principles
- Follow Rust standard conventions (RFC 1437)
- Write self-documenting code with clear intent
- Use exhaustive pattern matching for enums
- Prefer Result/Option over unwrap in public APIs

### Formatting
- Max line width: 100 characters
- Indentation: 4 spaces (no tabs)
- Newline style: Unix (LF)
- Use small heuristics for formatting decisions

### Naming Conventions
- **Crates/Modules**: snake_case (e.g., `rvue`, `signal`, `widgets`)
- **Types/Traits**: PascalCase (e.g., `ReadSignal`, `ComponentProps`, `SignalRead`)
- **Functions/Variables**: snake_case (e.g., `create_signal`, `signal_value`)
- **Constants**: SCREAMING_SNAKE_CASE for const, snake_case for static
- **Type parameters**: Short, camelCase (e.g., `T`, `U`)

### Error Handling
- Define custom error types with `thiserror` or manual `impl Error`
- Use `AppError` enum for application-level errors
- Use `ValidationError` for input validation errors
- Propagate errors with `?` operator
- Include context: `"Failed to X: {details}"`

### Imports
- Use absolute paths from crate root (e.g., `crate::signal::create_signal`)
- Group imports: std → external → internal
- Use `prelude::*` for commonly used traits and types

### Type System
- Use generics for reusable components (e.g., `ReadSignal<T>`)
- Use `Gc<T>` and `GcCell<T>` for garbage-collected types
- Derive `Debug` for non-secret-bearing structs
- Use `#[derive(Clone)]` for signal handles

### Reactivity Patterns
- Signals: `create_signal(initial) -> (ReadSignal<T>, WriteSignal<T>)`
- Effects: `create_effect(closure)` for side effects
- Components implement `Component` trait
- Views use `ViewStruct` for declarative UI

### Testing
- Unit tests in `tests/unit/` directory
- Integration tests in `tests/` root
- Name test functions: `test_<feature>_<behavior>`
- Use `assert_eq!`, `assert!`, `assert_ne!` for assertions

### Performance
- Use `#[inline(always)]` for hot-path functions
- Use `AtomicU64` with `Ordering::SeqCst` for version tracking
- Use `Gc::clone()` for GC-managed types
- Release borrows before running effects

## Project Structure

```
crates/
  rvue/              # Core framework
    src/
      signal.rs      # Fine-grained reactivity
      effect.rs      # Effect tracking
      component.rs   # Component system
      view.rs        # View declaration
      style.rs       # Styling types
      widgets/       # Built-in widgets
      layout/        # Layout (Taffy)
      render/        # Vello rendering
      app.rs         # Application entry
      error.rs       # Error types
    tests/           # Integration tests
  rvue-macro/        # Procedural macros
  rvue-examples/     # Example applications
```

## Common Patterns

### Using Signals
```rust
let (count, set_count) = create_signal(0);
let value = count.get();
set_count(value + 1);
```

### Error Handling
```rust
fn some_function() -> Result<T, AppError> {
    something()?;
    Ok(result)
}
```

## Cursor Rules Integration

Cursor-specific rules are defined in `.cursor/rules/specify-rules.mdc`. Key points:
- Active technology: Rust (latest stable, minimum 1.75+)
- Code style: Follow standard Rust conventions
- Primary commands: `cargo test`, `cargo clippy`

## Notes

- Tests run single-threaded (`[test] threads = 1`)
- Minimum Rust version: 1.75 (from `clippy.toml`)
- Project is in MVP stage - expect evolving APIs

## Active Technologies
- Rust 2021 Edition (minimum 1.75+) + `selectors = "0.35"` (Stylo), `cssparser = "0.36"`, `bitflags = "2"`, `rudo-gc` (001-style-system)
- N/A (in-memory style computation) (001-style-system)

## Recent Changes

### Props Enum Migration (COMPLETED)

#### Created PropertyMap Infrastructure
- **`crates/rvue/src/properties/`** - Xilem-inspired trait-based property system
  - `mod.rs` - Module exports with `defaults` submodule for global theming
  - `traits.rs` - `WidgetProperty` trait with `static_default()` method
  - `types.rs` - 20+ built-in properties:
    - `TextContent`, `WidgetStyles`, `ShowCondition`, `ForItemCount`
    - `TextInputValue`, `NumberInputValue`, `CheckboxChecked`, `RadioValue`
    - `FlexDirection`, `FlexGap`, `FlexAlignItems`, `FlexJustifyContent`
  - `map.rs` - `PropertyMap` with builder-style `.and()` method for chaining

#### Updated Component
- Added `properties: GcCell<PropertyMap>` field to `Component` struct
- Added `Component::with_properties()` and `Component::with_global_id_and_properties()` constructors
- Added getter/setter methods with full backward compatibility:
  - PropertyMap → ComponentProps → Defaults (three-tier fallback)
- Getters: `text_content()`, `flex_direction()`, `flex_gap()`, etc.
- Setters: `set_text_content()`, `set_flex_direction()`, etc. (update both PropertyMap + ComponentProps)

#### Updated Widgets to Initialize PropertyMap Directly
- **Text**: Initializes `PropertyMap::with(TextContent(...))` for non-reactive content
- **Flex**: Initializes all flex properties via `PropertyMap::new().and(...).and(...)`
- **Checkbox**: Initializes `PropertyMap::with(CheckboxChecked(...))` for non-reactive state
- **Radio**: Initializes `PropertyMap` with `RadioValue` and `CheckboxChecked`
- **TextInput/NumberInput**: Initialize `PropertyMap` with input values
- **Show**: Initializes `PropertyMap::with(ShowCondition(...))`
- **For**: Initializes `PropertyMap::with(ForItemCount(...))`

#### Global Theming (DefaultProperties)
- Added `properties::defaults` module with thread-safe global defaults:
  - `set_default_text_content()`, `get_default_text_content()`
  - `set_default_flex_direction()`, `get_default_flex_direction()`
  - `set_default_flex_gap()`, `get_default_flex_gap()`
  - `set_default_flex_align_items()`, `get_default_flex_align_items()`
  - `set_default_flex_justify_content()`, `get_default_flex_justify_content()`
- Getters fall back to defaults when values not in PropertyMap or ComponentProps

#### Tests & Quality
- All 252+ tests pass across all crates
- Removed unused `as_any` method from `DynProperty`
- Cleaned up unused imports in widgets

#### Bug Fix: WidgetStyles Initialization
- **Issue**: Widget styles (height, width) not applied because `WidgetStyles` was not initialized in PropertyMap
- **Fix**: Updated all widget builders (Text, Flex, Button, Checkbox, Radio, TextInput, NumberInput) to call `component.set_widget_styles(styles)` after component creation
- **Also fixed**: `styles_effect` in Flex now properly updates PropertyMap when styles change
- **Result**: Scroll container height/width now correctly applied (scroll example works)

### ComponentProps Deprecation Migration (COMPLETED)

Successfully completed the migration from `ComponentProps` enum to `PropertyMap` system.

#### Phase 1: Deprecation Warnings (COMPLETED)
- Marked `ComponentProps` enum as `#[deprecated]`
- Marked all getters/setters as `#[deprecated]`
- Added `#[allow(deprecated)]` blocks for backward compatibility

#### Phase 2: Updated Tests (COMPLETED)
- Updated `widget_api_test.rs` to use new API
- Updated `component_test.rs` with `#[allow(deprecated)]`
- Updated `button_widget_test.rs`, `layout_node_test.rs`, `slot_test.rs`, `context_gc_test.rs`
- All tests pass

#### Phase 3: Macro Codegen Update (COMPLETED)
- Updated `codegen.rs` to use `PropertyMap::new()` instead of `ComponentProps::Custom`
- Custom widgets now use `Component::with_global_id(type, PropertyMap::new())`

#### Phase 4: Core Refactoring (COMPLETED)
- Removed `props: GcCell<ComponentProps>` field from `Component` struct
- Removed `unsafe impl Trace for ComponentProps`
- Updated `Component::with_properties()` to only take `PropertyMap`
- Updated `Component::with_global_id()` for slot usage
- Removed fallback logic from getters/setters
- Simplified `style.rs::get_inline_styles()` to use PropertyMap only
- Updated `event/path.rs` and `slot.rs` to use PropertyMap

#### Phase 5: Cleanup (COMPLETED)
- Removed `ComponentProps` enum definition entirely
- Removed from `lib.rs` and `prelude.rs` exports
- Updated 25+ test files to use new API
- Updated all example applications (benchmark, slots, etc.)
- Updated `rvue-testing` crate

#### New API
```rust
// Create component with properties
Component::with_properties(id, component_type, PropertyMap::new())
Component::with_global_id(component_type, PropertyMap::new())

// Widget initialization pattern
PropertyMap::with(TextContent("hello".to_string()))
PropertyMap::new().and(FlexDirection("row".to_string())).and(FlexGap(10.0))
```

#### Verification
- Clippy passes with 0 warnings
- Core tests: 45+ passing
- All widgets updated to use PropertyMap system
