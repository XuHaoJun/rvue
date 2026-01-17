# Research: Rvue MVP Framework

**Date**: 2026-01-17  
**Purpose**: Resolve technical unknowns and document architectural decisions for Rvue MVP

## 1. Garbage Collection Integration (rudo-gc)

### Decision: Use rudo-gc as the foundation for UI component memory management

**Rationale**:
- rudo-gc provides `Gc<T>` smart pointers with automatic cycle detection
- Hybrid scanning: conservative stack scanning + precise heap scanning via `#[derive(Trace)]`
- BiBOP memory layout enables O(1) allocation and supports interior pointers
- Generational GC handles UI's "die-young" temporary objects efficiently
- `!Send` and `!Sync` align with single-threaded UI requirement

**Alternatives Considered**:
- **Rc<RefCell<T>>**: Rejected - requires manual cycle breaking, poor DX
- **Arena allocation**: Rejected - doesn't handle dynamic UI tree changes well
- **Other GC crates (shredder, gc)**: Rejected - rudo-gc already exists in learn-projects and has desired hybrid scanning

**Integration Points**:
- All UI components stored in `Gc<Component>`
- Signals stored in `Gc<SignalData<T>>`
- Component tree uses `Gc` pointers for parent-child relationships
- Effects capture `Gc<Signal>` references, forming automatic dependency graph

**Risks & Mitigations**:
- **Risk**: GC pause times may exceed 16ms frame budget
- **Mitigation**: MVP uses existing generational GC. Incremental marking deferred to Phase 2 optimization
- **Risk**: Conservative stack scanning may miss pointers in optimized code
- **Mitigation**: BiBOP layout with interior pointer support reduces risk. Monitor in testing

## 2. Procedural Macro Design (view! macro)

### Decision: Implement Leptos-style view! macro with compile-time reactivity analysis

**Rationale**:
- Leptos demonstrates successful Rust macro-based UI DSL
- Compile-time analysis enables fine-grained updates without runtime diffing
- Familiar HTML-like syntax reduces learning curve for web developers
- Macros can generate optimized code paths for static vs dynamic content

**Key Design Decisions**:
- **Static structure**: Components are allocated once, not rebuilt each render
- **Dynamic bindings**: Macros detect signals and generate effect subscriptions
- **Control flow**: Use component-based approach (`<Show>`, `<For>`) rather than trying to rewrite Rust `if`/`for` syntax

**Alternatives Considered**:
- **Svelte-style magic**: Rejected - too complex to parse arbitrary Rust syntax in macros
- **Runtime VDOM**: Rejected - performance overhead, doesn't leverage Rust's compile-time strengths
- **Pure builder pattern**: Rejected - too verbose, poor DX compared to declarative syntax

**Implementation Approach**:
- Parse HTML-like DSL using `syn` crate
- Identify static attributes vs signal-based attributes
- Generate code that:
  1. Allocates component tree once using `Gc::new()`
  2. Creates effects for each signal binding
  3. Directly updates Vello scene properties on signal changes

**Reference Implementation**: `learn-projects/leptos/leptos_macro/src/lib.rs`

## 3. Rendering Integration (Vello)

### Decision: Use Vello for GPU-accelerated 2D rendering

**Rationale**:
- Vello provides modern GPU compute-based rendering (piet-gpu)
- Retained mode scene graph aligns with our "build once, update incrementally" model
- Cross-platform support (Windows, macOS, Linux)
- High performance for 2D UI rendering

**Integration Architecture**:
- **Scene Graph**: Each component maps to Vello scene fragments
- **Update Path**: Signal changes → Effect triggers → Direct Vello buffer updates
- **Resource Bridge**: Vello resources (textures, paths) stored outside GC heap, referenced by GC-managed components

**Key Constraints**:
- Vello `Scene` is `Send` but our `Gc<T>` is `!Send`
- Solution: Generate `Scene` on main thread, pass to render thread via channel
- Don't store GPU resource handles in `Gc<T>` - use explicit resource management

**Alternatives Considered**:
- **Skia**: Rejected - heavier, less modern architecture
- **wgpu directly**: Rejected - too low-level, Vello provides better abstraction
- **Software rendering**: Rejected - performance requirements need GPU acceleration

## 4. Layout Integration (Taffy)

### Decision: Use Taffy for flexbox and grid layouts

**Rationale**:
- Taffy is the standard Rust layout engine (used by Bevy, Iced, etc.)
- Supports flexbox and grid (CSS-like semantics)
- Pure Rust, no C dependencies
- Good performance for typical UI layouts

**Integration Architecture**:
- **Layout Tree**: Each component has associated Taffy node
- **Style Mapping**: Component style attributes → Taffy `Style` struct
- **Update Strategy**: Mark layout dirty when content size changes, recalculate on next frame
- **GC Integration**: Taffy nodes stored in `Gc`-managed wrapper to allow bidirectional references

**Layout Calculation Flow**:
1. Component tree created → Taffy nodes allocated
2. Style changes → Taffy `Style` updated, node marked dirty
3. Before render → Compute layout for dirty nodes
4. Layout results → Update Vello scene positions

**Alternatives Considered**:
- **Custom layout**: Rejected - reinventing the wheel, Taffy is proven
- **Manual positioning**: Rejected - doesn't meet "familiar to web developers" requirement

## 5. Reactive System Design (Signals & Effects)

### Decision: Fine-grained reactivity with signals and effects, similar to SolidJS/Leptos

**Rationale**:
- Fine-grained updates only touch changed UI elements (no VDOM diff)
- Signal-based model is familiar to Vue 3 / SolidJS developers
- Compile-time effect generation enables optimal update paths
- GC-managed signals eliminate lifetime complexity

**Signal Design**:
```rust
// Signal API (similar to Leptos)
let (count, set_count) = create_signal(0);
count.get()  // Read value
set_count.set(5)  // Set value
set_count.update(|x| *x += 1)  // Update with closure
```

**Effect Design**:
- Effects automatically track signal dependencies
- When signal changes, effect re-runs
- Effects update Vello scene properties directly (no intermediate representation)
- Effects are GC-managed, automatically cleaned up when component unmounts

**Alternatives Considered**:
- **Observer pattern**: Rejected - more boilerplate, less ergonomic
- **Redux-style state management**: Rejected - too heavy for MVP, not familiar to target audience
- **Flutter-style setState**: Rejected - coarse-grained, requires full widget tree rebuild

**Reference Implementation**: `learn-projects/leptos/reactive_graph/src/signal.rs`

## 6. Component Architecture

### Decision: Retained mode component tree with fine-grained updates (SolidJS model, not Flutter model)

**Rationale**:
- Components allocated once, not rebuilt on each state change
- Matches "build once, bind signals" philosophy
- Enables direct Vello property updates without tree reconstruction
- GC manages component lifecycle automatically

**Component Lifecycle**:
1. **Creation**: Component function called once, returns `impl View`
2. **Mounting**: Component tree allocated in GC heap, Taffy nodes created, Vello fragments initialized
3. **Updates**: Signals trigger effects that update specific Vello properties
4. **Unmounting**: Component dropped, GC automatically cleans up

**Component Composition**:
- Components can contain other components
- Parent-child relationships use `Gc` pointers (bidirectional references allowed)
- Component props passed as function parameters (Rust types, not special prop system)

**Alternatives Considered**:
- **Flutter-style rebuild**: Rejected - performance overhead, doesn't leverage compile-time optimization
- **Virtual DOM**: Rejected - unnecessary diffing overhead, Rust macros can do better

## 7. Threading Model

### Decision: Single-threaded UI with explicit channel-based communication for background work

**Rationale**:
- rudo-gc `Gc<T>` is `!Send` and `!Sync`, enforcing single-threaded usage
- Industry standard: Flutter, DOM, UIKit all use single UI thread
- Simpler mental model, no synchronization complexity
- Vello scene generation can happen on main thread, rendering on separate thread

**Architecture**:
- **Main Thread**: All UI operations, signal updates, layout calculations, Vello scene generation
- **Render Thread** (optional): Consumes Vello `Scene` and renders to GPU
- **Background Work**: Heavy computations run on separate threads, results sent via `mpsc::channel` to main thread

**Alternatives Considered**:
- **Multi-threaded UI**: Rejected - incompatible with `!Send` GC, adds complexity
- **Async/await for UI**: Rejected - async state machines complicate conservative stack scanning

## 8. Styling System

### Decision: Support multiple styling approaches (inline attributes, style objects, optional stylesheets)

**Rationale**:
- Flexibility for different developer preferences
- Inline attributes for simple cases (familiar to React/Vue)
- Style objects for complex styling (similar to CSS-in-JS)
- Stylesheets for shared styles (future extensibility)

**Implementation**:
- Inline: `<Text color=Color::Red font_size=16.0 />`
- Style object: `<Text style=Style { color: Color::Red, font_size: 16.0, ..Default::default() } />`
- Stylesheet: MVP may defer, but architecture supports it

**Style Properties**:
- Colors, fonts, spacing, borders, backgrounds
- Layout properties (flexbox/grid) via Taffy integration
- Transitions/animations: Deferred to post-MVP

## 9. Input Components

### Decision: Provide built-in components for common input types (text, number, checkbox, radio, button)

**Rationale**:
- Essential for building forms and interactive UIs
- Common types cover majority of use cases
- Can be extended post-MVP with more specialized inputs

**Component API**:
```rust
// Text input
<TextInput value=text_signal on_input=move |val| text_signal.set(val) />

// Number input  
<NumberInput value=count_signal min=0 max=100 />

// Checkbox
<Checkbox checked=is_checked_signal />

// Radio button
<Radio value="option1" checked=selected_signal />

// Button
<Button on_click=move |_| handle_click()> "Click me" </Button>
```

**Event Handling**:
- Events passed as closures that can access signals
- Closures are GC-managed, can capture `Gc<Signal>` references
- Event types inferred from component (click, input, change, etc.)

## 10. Performance Considerations

### Decision: Optimize for MVP with clear path for future improvements

**MVP Optimizations**:
- Compile-time effect generation (no runtime dependency tracking overhead)
- Direct Vello property updates (no intermediate diffing)
- Generational GC for temporary objects
- Layout dirty marking (only recalculate when needed)

**Deferred Optimizations** (Phase 2+):
- Incremental GC marking to prevent frame drops
- Layout calculation batching
- Virtual scrolling for large lists
- GPU resource pooling
- Scene graph culling for off-screen elements

**Performance Targets**:
- 60fps: 16ms frame budget (achievable with current design)
- Startup: <2s (native compilation, no webview overhead)
- Memory: <100MB initial (GC only manages UI state, not entire app)

## 11. Platform Support

### Decision: Support Windows, macOS, and Linux for MVP

**Window Management**: Use `winit` crate (standard Rust windowing library)

**Rendering Backend**: Vello supports multiple backends:
- **wgpu**: Cross-platform, recommended for MVP
- **skia**: Alternative, may have better platform integration

**Platform-Specific Considerations**:
- Window creation and event loop: `winit` handles this
- Input handling: `winit` provides keyboard/mouse events
- File dialogs, system integration: Deferred to post-MVP

## 12. Error Handling

### Decision: Framework provides error types, developers implement error UI

**Rationale**:
- Flexibility for different error presentation styles
- Framework focuses on reactivity/rendering, not UI patterns
- Developers can use toast notifications, inline messages, modals, etc.

**Error Types**:
- Validation errors (input validation)
- Rendering errors (malformed component definitions)
- Layout errors (invalid layout constraints)
- GC errors (out of memory - rare, but possible)

**Developer API**:
```rust
match validate_input(&input) {
    Ok(value) => signal.set(value),
    Err(e) => {
        // Developer implements error UI
        show_error_toast(&e);
    }
}
```

## Summary

All technical unknowns resolved. The architecture combines:
- **rudo-gc**: Automatic memory management
- **Leptos-style macros**: Compile-time reactivity
- **Vello**: GPU rendering
- **Taffy**: Layout engine
- **Fine-grained signals**: Optimal update performance

This creates a framework that feels like web development (Vue/React) but performs like native code (Rust + GPU).
