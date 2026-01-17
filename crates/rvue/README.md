# Rvue - Rust GUI Framework

A Rust GUI framework with Vue-like syntax and GC-managed memory, designed for building native desktop applications.

## Features

- **Reactive State Management**: Signals and effects for fine-grained reactivity
- **Component-Based Architecture**: Build UIs with reusable components
- **GC-Managed Memory**: Automatic memory management with rudo-gc
- **GPU-Accelerated Rendering**: Vello for high-performance 2D rendering
- **Flexbox Layouts**: Taffy integration for CSS-like layouts
- **Familiar Syntax**: Vue Composition API-inspired patterns

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rvue = { path = "../rvue" }  # For local development
# rvue = "0.1.0"  # When published
```

Basic example:

```rust
use rvue::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (count, set_count) = create_signal(0);
    
    let view = ViewStruct::new(
        Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text {
                content: format!("Count: {}", count.get()),
            },
        ),
    );
    
    rvue::run_app(|| view)?;
    Ok(())
}
```

## Core Concepts

### Signals

Signals are reactive data containers:

```rust
let (read, write) = create_signal(0);
write.set(10);
assert_eq!(read.get(), 10);
```

### Effects

Effects automatically re-run when dependencies change:

```rust
let (count, set_count) = create_signal(0);
let _effect = create_effect({
    let count = count.clone();
    move || {
        println!("Count is now: {}", count.get());
    }
});
```

### Components

Components are reusable UI building blocks:

```rust
let text = Component::new(
    1,
    ComponentType::Text,
    ComponentProps::Text {
        content: "Hello".to_string(),
    },
);
```

### Widgets

Built-in widgets for common UI elements:

- `Text` - Display text
- `Button` - Interactive button
- `TextInput` - Text input field
- `NumberInput` - Numeric input field
- `Checkbox` - Boolean checkbox
- `Radio` - Radio button
- `Show` - Conditional rendering
- `For` - List rendering
- `Flex` - Flexbox container

## Examples

See the `rvue-examples` crate for complete examples:

- `counter` - Basic counter application
- `list` - Todo list with dynamic items
- `layout` - Complex nested layouts
- `form` - Form with all input types
- `benchmark` - Performance benchmarks

## Architecture

- **Reactivity Layer**: Signals and effects for state management
- **Component Layer**: Component tree with lifecycle management
- **Layout Layer**: Taffy integration for flexbox/grid layouts
- **Render Layer**: Vello integration for GPU-accelerated rendering
- **App Layer**: Winit integration for window and event management

## Performance

- **Startup Time**: < 2 seconds (target)
- **Memory Footprint**: < 100MB initial (target)
- **Frame Rate**: 60fps (16ms frame budget)
- **Optimizations**: Lazy renderer initialization, optimized component creation

## Requirements

- Rust 1.75+ (nightly recommended for some features)
- GPU support for Vello rendering

## Documentation

- [API Documentation](https://docs.rs/rvue) (when published)
- [Quickstart Guide](../specs/002-rvue-mvp/quickstart.md)
- [Performance Benchmarks](../docs/performance.md)

## License

MIT OR Apache-2.0
