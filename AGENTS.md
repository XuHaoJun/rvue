# AGENTS.md - Rvue Development Guidelines

This document provides guidelines for AI agents working on the Rvue codebase.

## Project Overview

Rvue is a high-performance, GPU-accelerated Rust GUI framework inspired by Vue's developer experience and SolidJS's fine-grained reactivity. The project uses:

- Rust 2021 Edition (minimum 1.84+)
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

## Rudo-GC Guidelines

### When to Use `borrow_mut()` vs `borrow_mut_gen_only()`

**Use `borrow_mut()` when:**

- The type contains `Gc<T>` pointers (e.g., `GcCell<Vec<Gc<Component>>>`)
- You need SATB barrier correctness for incremental marking
- The operation is not performance-critical

**Use `borrow_mut_gen_only()` when:**

- The type does NOT contain `Gc<T>` pointers (e.g., `GcCell<i32>`, `GcCell<String>`)
- The operation is in a hot path (e.g., signal updates, frequent property changes)
- Performance is critical and barriers are proven to be the bottleneck

**Example:**

```rust
// Signal updates (hot path, no Gc<T>) - use borrow_mut_gen_only()
pub fn set(&self, value: T) {
    *self.value.borrow_mut_gen_only() = value;
}

// Component tree operations (contains Gc<T>) - use borrow_mut()
pub fn add_child(&self, child: Gc<Component>) {
    self.children.borrow_mut().push(Gc::clone(&child));
}
```

### Using `#[derive(GcCell)]`

**When to derive:**

- Struct types (not enums or generics) that contain `Gc<T>` fields
- Types that will be used with `GcCell<T>` and need `borrow_mut()`

**Limitations:**

- ❌ Enums: Not supported (use manual `GcCapture` implementation)
- ❌ Generic types: Not supported (use `borrow_mut_gen_only()` or manual impl)
- ❌ Recursive types: Not supported (use manual implementation)

**Example:**

```rust
#[derive(Trace, GcCell)]
pub struct Component {
    pub children: GcCell<Vec<Gc<Component>>>,  // Contains Gc<T>
    // ...
}

// For generic types like SignalData<T>, use borrow_mut_gen_only()
pub struct SignalData<T: Clone + 'static> {
    pub value: GcCell<T>,  // T is generic, may not contain Gc<T>
}
```

### Performance Considerations

- **Barrier overhead**: `borrow_mut()` adds ~10-15 ops + 1 atomic (generational barrier)
- **Hot paths**: Signal updates use `borrow_mut_gen_only()` for 3-10x performance improvement
- **GC pause budget**: GUI requires 60 FPS = 16ms frame budget; monitor GC pauses
- **Incremental marking**: Enabled by default; reduces GC pause times for better UI responsiveness

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
