# API Documentation: Rvue MVP Framework

**Date**: 2026-01-17  
**Version**: 0.1.0

## Core API

### Signal API

#### `create_signal<T>(initial_value: T) -> (ReadSignal<T>, WriteSignal<T>)`

Creates a new reactive signal with an initial value.

**Example**:
```rust
use rvue::prelude::*;

let (count, set_count) = create_signal(0);
set_count.set(10);
assert_eq!(count.get(), 10);
```

**Type Constraints**: `T: Trace + Clone + 'static`

#### `ReadSignal<T>`

Read handle for a signal. Implements `SignalRead<T>` trait.

**Methods**:
- `get() -> T`: Get the current value (automatically tracks dependencies if called in an effect)

#### `WriteSignal<T>`

Write handle for a signal. Implements `SignalWrite<T>` trait.

**Methods**:
- `set(value: T)`: Set the signal value
- `update<F>(f: F)`: Update the signal value using a closure

**Example**:
```rust
let (count, set_count) = create_signal(0);
set_count.update(|x| *x += 1);
```

### Effect API

#### `create_effect<F>(f: F) -> Gc<Effect>`

Creates a reactive effect that automatically re-runs when dependencies change.

**Example**:
```rust
let (count, set_count) = create_signal(0);
let _effect = create_effect({
    let count = count.clone();
    move || {
        println!("Count is: {}", count.get());
    }
});
```

**Behavior**:
- Runs immediately on creation
- Automatically tracks signals accessed during execution
- Re-runs when any tracked signal changes

### Component API

#### `Component::new(id, component_type, props) -> Gc<Component>`

Creates a new component.

**Example**:
```rust
use rvue::{Component, ComponentType, ComponentProps};

let text = Component::new(
    1,
    ComponentType::Text,
    ComponentProps::Text {
        content: "Hello".to_string(),
    },
);
```

#### `ComponentLifecycle` Trait

Lifecycle methods for components:

- `mount(parent: Option<Gc<Component>>)`: Mount component to tree
- `unmount()`: Unmount component from tree
- `update()`: Update component (runs dirty effects)

### Widget API

#### Text Widget

```rust
use rvue::Text;

let text = Text::new(1, "Hello World".to_string());
```

#### Button Widget

```rust
use rvue::Button;

let button = Button::new(1, "Click Me".to_string());
```

#### TextInput Widget

```rust
use rvue::TextInput;

let input = TextInput::new(1, "".to_string());
```

#### NumberInput Widget

```rust
use rvue::NumberInput;

let number = NumberInput::new(1, 0.0);
```

#### Checkbox Widget

```rust
use rvue::Checkbox;

let checkbox = Checkbox::new(1, false);
```

#### Radio Widget

```rust
use rvue::Radio;

let radio = Radio::new(1, "option1".to_string(), true);
```

#### Show Widget (Conditional Rendering)

```rust
use rvue::Show;

let show = Show::new(1, true); // Shows children when true
```

#### For Widget (List Rendering)

```rust
use rvue::For;

let for_component = For::new(1, items.len());
```

#### Flex Widget (Layout)

```rust
use rvue::{Flex, FlexDirection, AlignItems, JustifyContent};

let flex = Flex::new(
    1,
    FlexDirection::Column,
    10.0, // gap
    AlignItems::Center,
    JustifyContent::Start,
);
```

### View API

#### `ViewStruct`

Represents a declarative UI tree.

**Example**:
```rust
use rvue::{ViewStruct, Component, ComponentType, ComponentProps};

let root = Component::new(0, ComponentType::Flex, ComponentProps::Flex { ... });
let view = ViewStruct::new(root);
```

### Application API

#### `run_app<F>(view_fn: F) -> Result<(), AppError>`

Runs the application with the given view function.

**Example**:
```rust
use rvue::run_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app(|| {
        // Create and return view
        ViewStruct::new(root_component)
    })?;
    Ok(())
}
```

### Error Types

#### `AppError`

Application-level errors:

```rust
pub enum AppError {
    WindowCreationFailed(String),
    RendererInitializationFailed(String),
    ComponentCreationFailed(String),
    LayoutCalculationFailed(String),
    GcError(String),
}
```

### Style API

#### `Style` Struct

Component styling properties:

```rust
use rvue::{Style, Color, Spacing, FontWeight};

let style = Style {
    color: Some(Color::Rgb { r: 255, g: 0, b: 0 }),
    font_size: Some(16.0),
    padding: Some(Spacing::uniform(10.0)),
    ..Default::default()
};
```

#### Color Types

```rust
pub enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Named(String),
}
```

#### Layout Enums

```rust
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
```

## Examples

See the `rvue-examples` crate for complete working examples:

- `counter` - Basic counter with reactive state
- `list` - Todo list with dynamic items
- `layout` - Complex nested flexbox layouts
- `form` - Form with all input types
- `benchmark` - Performance benchmarks

## Best Practices

1. **Signal Usage**: Create signals at component level, not inside render functions
2. **Effect Dependencies**: Effects automatically track dependencies - avoid manual dependency management
3. **Component Composition**: Build complex UIs by composing simple components
4. **Memory Management**: GC handles cleanup automatically - no manual memory management needed
5. **Performance**: Use computed values instead of recalculating in effects

## Type Constraints

### Trace Trait

Types containing `Gc` pointers must implement `Trace`:

```rust
use rudo_gc::Trace;

#[derive(Trace)]
struct MyStruct {
    field: Gc<String>,
}
```

## Platform Support

- Windows
- macOS
- Linux

## Version

Current version: 0.1.0 (MVP)
