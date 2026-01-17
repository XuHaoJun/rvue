# API Contracts: Rvue MVP Framework

**Date**: 2026-01-17  
**Purpose**: Define public API contracts for Rvue framework

## Core API

### Signal API

```rust
// Create a signal
pub fn create_signal<T: Trace + 'static>(
    initial_value: T
) -> (ReadSignal<T>, WriteSignal<T>);

// Read signal
pub trait ReadSignal<T> {
    fn get(&self) -> T;
}

// Write signal
pub trait WriteSignal<T> {
    fn set(&self, value: T);
    fn update<F>(&self, f: F) where F: FnOnce(&mut T);
}
```

**Contract**:
- `create_signal` returns a tuple of read and write handles
- `get()` returns a clone of the current value
- `set()` immediately updates value and notifies subscribers
- `update()` allows in-place mutation
- Signals are `Copy` for ergonomic closure usage

### Effect API

```rust
// Create an effect
pub fn create_effect<F>(f: F) -> Effect
where
    F: Fn() + 'static;

// Effect automatically tracks signal dependencies
// Re-runs when any dependency changes
```

**Contract**:
- Effects run once immediately on creation
- Effects automatically track signals accessed during execution
- Effects re-run when any tracked signal changes
- Effects are cleaned up when dropped (GC-managed)

### Component API

```rust
// Component trait
pub trait Component {
    fn render(&self) -> impl View;
}

// Component macro
#[component]
fn MyComponent() -> impl View {
    // Component implementation
    view! { /* ... */ }
}
```

**Contract**:
- Components are functions that return `impl View`
- `#[component]` macro handles GC allocation and lifecycle
- Components can own signals and effects
- Components can contain child components

### View Macro API

```rust
// Basic view macro
view! {
    <ComponentName prop=value>
        {children}
    </ComponentName>
}

// Conditional rendering
view! {
    <Show when=condition_signal>
        {content}
    </Show>
}

// List rendering
view! {
    <For each=items_signal key=|item| item.id>
        |item| view! { <ItemComponent item=item /> }
    </For>
}
```

**Contract**:
- HTML-like syntax for component structure
- Attributes can be static values or signals
- Signal attributes automatically create reactive bindings
- Children can be static or dynamic (signals/closures)
- Control flow uses component-based approach (`<Show>`, `<For>`)

### Widget Components API

```rust
// Text widget
<Text value=text_signal style=text_style />

// Button widget
<Button on_click=click_handler>
    {button_label}
</Button>

// Text input widget
<TextInput 
    value=text_signal 
    on_input=move |val| text_signal.set(val)
    placeholder="Enter text..."
/>

// Number input widget
<NumberInput 
    value=number_signal 
    min=0 
    max=100
    on_input=move |val| number_signal.set(val)
/>

// Checkbox widget
<Checkbox 
    checked=bool_signal 
    on_change=move |val| bool_signal.set(val)
/>

// Radio button widget
<Radio 
    value="option1" 
    checked=selected_signal 
    on_change=move |val| selected_signal.set(val)
/>
```

**Contract**:
- Widgets are built-in components with specific props
- Value props accept signals for two-way binding
- Event handlers are closures that can access signals
- Style props accept `Style` struct or inline attributes

### Layout API

```rust
// Flexbox container
<Flex 
    direction=FlexDirection::Column 
    gap=16.0
    align_items=AlignItems::Center
>
    {children}
</Flex>

// Grid container (future)
<Grid 
    columns=3 
    rows=2
    gap=8.0
>
    {children}
</Grid>
```

**Contract**:
- Layout components use Taffy for calculations
- Layout properties map to CSS flexbox/grid semantics
- Layout recalculates when content size changes
- Layout results update Vello scene positions

### Styling API

```rust
// Inline style attributes
<Text color=Color::Red font_size=16.0 />

// Style object
<Text style=Style {
    color: Color::Blue,
    font_size: 18.0,
    padding: Spacing::all(10.0),
    ..Default::default()
} />

// Reactive styling
<Text color=move || if is_active.get() { Color::Green } else { Color::Gray } />
```

**Contract**:
- Style properties can be static or signal-based
- Inline attributes are convenient for simple cases
- Style objects support complex styling
- Reactive styles update when signals change

### Application API

```rust
// Create and run application
pub fn run_app<F>(app_fn: F) -> Result<(), AppError>
where
    F: Fn() -> impl View + 'static;
```

**Contract**:
- `run_app` initializes window, event loop, and rendering
- Application function returns root view
- Application runs until window is closed
- All UI operations occur on main thread

## Error Types

```rust
pub enum AppError {
    WindowCreationFailed(String),
    RendererInitializationFailed(String),
    ComponentCreationFailed(String),
    LayoutCalculationFailed(String),
    GcError(String),
}

pub enum ValidationError {
    InvalidInput(String),
    OutOfRange { value: f64, min: f64, max: f64 },
    InvalidFormat(String),
}
```

**Contract**:
- Framework provides error types
- Developers implement error UI (toast, inline message, etc.)
- Errors are returned via `Result` types
- Error messages are user-friendly strings

## Platform API

```rust
// Window management (via winit, not directly exposed)
// Event handling (via winit, not directly exposed)

// Platform-specific features deferred to post-MVP
```

**Contract**:
- Window creation and event loop handled internally
- Keyboard/mouse events passed to component event handlers
- Platform-specific features (file dialogs, etc.) deferred

## Memory Management API

```rust
// GC management (mostly internal, but exposed for advanced use)
pub fn collect() -> CollectInfo;
pub fn set_collect_condition(f: impl Fn(CollectInfo) -> bool);
```

**Contract**:
- GC runs automatically based on heuristics
- Manual collection available for testing/debugging
- Collection condition can be customized
- GC statistics available via `CollectInfo`

## Type Constraints

### Trace Trait

```rust
pub unsafe trait Trace {
    fn trace(&self, visitor: &mut impl Visitor);
}
```

**Contract**:
- Types containing `Gc` pointers must implement `Trace`
- `#[derive(Trace)]` automatically implements for structs/enums
- Manual implementation must visit all `Gc` fields
- Unsafe trait requires careful implementation

### View Trait

```rust
pub trait View {
    fn into_component(self) -> Gc<Component>;
}
```

**Contract**:
- Views can be converted to components
- Component tree is GC-managed
- Views are typically created via `view!` macro

## API Stability

**MVP Status**: API is subject to change based on feedback and usage patterns.

**Breaking Changes**: Will be documented in changelog with migration guides.

**Deprecation Policy**: Deprecated APIs will be marked and removed in next major version.
