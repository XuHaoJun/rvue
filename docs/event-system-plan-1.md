# Rvue Event System Technical Specification

## Executive Summary

This document specifies the event system architecture for Rvue, a Rust GUI framework combining GC-managed memory (`rudo-gc`), fine-grained reactivity (Solid.js-like signals), and GPU rendering (Vello). The design draws inspiration from **Xilem/Masonry**'s battle-tested event infrastructure while adapting it to Rvue's unique "setup-once, update via signals" paradigm.

**Authors (Parallel World Collaboration):**
- Alex Crichton – Low-level Rust integration, GC safety considerations
- Leptos Team – Procedural macro ergonomics for event handlers
- 尤雨溪 (Evan You) – Developer experience and API design
- Ryan Carniato – Fine-grained reactivity integration

---

## 1. Design Goals

### 1.1 Primary Objectives

| Goal | Description |
|------|-------------|
| **Fine-Grained Event Binding** | Event handlers should be closures that directly modify signals, not trigger tree rebuilds |
| **GC-Friendly Lifecycle** | Event handlers stored in `Gc<T>` must be trace-able and properly collected |
| **Zero-Cost Hit Testing** | Leverage layout tree (Taffy) for O(log n) pointer event targeting |
| **Platform Agnostic** | Abstract over winit events; future backends (Android, iOS) possible |
| **Accessibility First** | Integrate with accesskit from day one |

### 1.2 Non-Goals (Phase 1)

- Gesture recognition (pinch, swipe) – deferred to Phase 2
- Advanced IME composition – basic text input only
- Multi-window support – single window MVP
- Touch events – pointer abstraction covers mouse first

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Winit Event Loop                        │
│                    (WindowEvent, DeviceEvent)                   │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Rvue Event Dispatcher                      │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────────┐ │
│  │ RawEvent    │  │ EventTarget  │  │ EventContext            │ │
│  │ Translation │→ │ Resolution   │→ │ (GC-Safe Callbacks)     │ │
│  └─────────────┘  └──────────────┘  └─────────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            ▼                   ▼                   ▼
   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
   │ PointerPass    │  │ TextPass       │  │ AccessPass     │
   │ (Click, Move)  │  │ (Keyboard, IME)│  │ (A11y Actions) │
   └────────────────┘  └────────────────┘  └────────────────┘
            │                   │                   │
            └───────────────────┴───────────────────┘
                                │
                                ▼
            ┌─────────────────────────────────────────┐
            │         Signal/Effect Update            │
            │    (Fine-Grained Reactivity Layer)      │
            └─────────────────────────────────────────┘
                                │
                                ▼
            ┌─────────────────────────────────────────┐
            │      Vello Scene Fragment Update        │
            │    (Only affected fragments repaint)    │
            └─────────────────────────────────────────┘
```

---

## 3. Event Types

### 3.1 Raw Event Categories (from Winit)

Following Xilem/Masonry's proven categorization:

```rust
/// Window-level events (not targeted at widgets)
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// DPI/scale factor change
    Rescale(f64),
    /// Window resized
    Resize(PhysicalSize<u32>),
    /// Animation frame tick (for requestAnimationFrame-like behavior)
    AnimFrame(Duration),
    /// Window gained/lost focus
    FocusChange(bool),
    /// Close requested
    CloseRequested,
}

/// Pointer events (mouse, touch, stylus abstraction)
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    /// Pointer button pressed
    Down(PointerButtonEvent),
    /// Pointer button released  
    Up(PointerButtonEvent),
    /// Pointer moved
    Move(PointerMoveEvent),
    /// Pointer entered window
    Enter(PointerInfo),
    /// Pointer left window
    Leave(PointerInfo),
    /// Interaction cancelled (window lost focus, etc.)
    Cancel(PointerInfo),
    /// Scroll (wheel or trackpad)
    Scroll(PointerScrollEvent),
}

/// Text/keyboard events
#[derive(Debug, Clone, PartialEq)]
pub enum TextEvent {
    /// Keyboard key event
    Keyboard(KeyboardEvent),
    /// IME composition event
    Ime(ImeEvent),
    /// Clipboard paste
    Paste(String),
}

/// Accessibility events (from accesskit)
#[derive(Debug, Clone, PartialEq)]
pub struct AccessEvent {
    pub action: accesskit::Action,
    pub data: Option<accesskit::ActionData>,
}
```

### 3.2 Rvue-Specific Event Wrapper

```rust
/// Unified event type for Rvue's internal dispatch
pub enum RvueEvent {
    Window(WindowEvent),
    Pointer(PointerEvent),
    Text(TextEvent),
    Access(AccessEvent),
    /// Custom application event (user-defined)
    Custom(Box<dyn std::any::Any + Send>),
}
```

---

## 4. Event Target Resolution

### 4.1 Hit Testing Algorithm

Rvue leverages its existing Taffy layout tree for efficient hit testing:

```rust
/// Finds the deepest component containing the given point
pub fn hit_test(
    root: &Gc<Component>,
    point: Point,
) -> Option<Gc<Component>> {
    hit_test_recursive(root, point, Affine::IDENTITY)
}

fn hit_test_recursive(
    component: &Gc<Component>,
    point: Point,
    parent_transform: Affine,
) -> Option<Gc<Component>> {
    let layout_node = component.layout_node()?;
    let layout = layout_node.layout()?;
    
    // Compute this component's bounds in window coordinates
    let local_origin = Point::new(
        layout.location.x as f64,
        layout.location.y as f64,
    );
    let local_size = Size::new(
        layout.size.width as f64,
        layout.size.height as f64,
    );
    let transform = parent_transform * Affine::translate(local_origin.to_vec2());
    let bounds = Rect::from_origin_size(Point::ZERO, local_size);
    
    // Transform point to local coordinates
    let local_point = transform.inverse() * point;
    
    if !bounds.contains(local_point) {
        return None;
    }
    
    // Check children in reverse order (last child is on top)
    for child in component.children.borrow().iter().rev() {
        if let Some(hit) = hit_test_recursive(child, point, transform) {
            return Some(hit);
        }
    }
    
    // If no child hit, and this component accepts pointer interaction
    if component.accepts_pointer_interaction() {
        Some(Gc::clone(component))
    } else {
        None
    }
}
```

### 4.2 Focus Management

For text/keyboard events, Rvue maintains a focus stack:

```rust
/// Global focus state (stored in AppState)
pub struct FocusState {
    /// Currently focused component (receives keyboard events)
    pub focused: Option<Gc<Component>>,
    /// Focus fallback (receives events when nothing is focused)
    pub fallback: Option<Gc<Component>>,
    /// Widget that should receive focus on next frame
    pub pending_focus: Option<Gc<Component>>,
    /// Navigation anchor (for Tab focus navigation)
    pub focus_anchor: Option<Gc<Component>>,
}
```

---

## 5. Event Handlers

### 5.1 Handler Storage in Component

Extend the existing `Component` struct to store event handlers:

```rust
/// Event handler closure type (GC-managed)
pub type EventHandler<E> = Gc<dyn Fn(&E, &mut EventContext) + 'static>;

/// Event handlers for a component
#[derive(Default)]
pub struct EventHandlers {
    pub on_pointer_down: Option<EventHandler<PointerButtonEvent>>,
    pub on_pointer_up: Option<EventHandler<PointerButtonEvent>>,
    pub on_pointer_move: Option<EventHandler<PointerMoveEvent>>,
    pub on_pointer_enter: Option<EventHandler<PointerInfo>>,
    pub on_pointer_leave: Option<EventHandler<PointerInfo>>,
    pub on_click: Option<EventHandler<PointerButtonEvent>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_key_up: Option<EventHandler<KeyboardEvent>>,
    pub on_focus: Option<EventHandler<FocusEvent>>,
    pub on_blur: Option<EventHandler<FocusEvent>>,
    pub on_input: Option<EventHandler<InputEvent>>,
}

// Add to Component struct
pub struct Component {
    // ... existing fields ...
    pub event_handlers: GcCell<EventHandlers>,
}

// Implement Trace for EventHandlers
unsafe impl Trace for EventHandlers {
    fn trace(&self, visitor: &mut impl Visitor) {
        // Trace all handler Gc pointers
        if let Some(ref h) = self.on_pointer_down { h.trace(visitor); }
        if let Some(ref h) = self.on_pointer_up { h.trace(visitor); }
        // ... trace all handlers ...
    }
}
```

### 5.2 Handler Registration API

```rust
impl Component {
    /// Register a click handler
    pub fn on_click<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&PointerButtonEvent, &mut EventContext) + 'static,
    {
        let handler = Gc::new(handler) as EventHandler<PointerButtonEvent>;
        self.event_handlers.borrow_mut().on_click = Some(handler);
    }
    
    /// Register a key down handler
    pub fn on_key_down<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&KeyboardEvent, &mut EventContext) + 'static,
    {
        let handler = Gc::new(handler) as EventHandler<KeyboardEvent>;
        self.event_handlers.borrow_mut().on_key_down = Some(handler);
    }
    
    // ... other handler registration methods ...
}
```

### 5.3 Signal Integration

The key innovation: handlers directly modify signals, triggering fine-grained updates:

```rust
// Example: Counter button
let (count, set_count) = create_signal(0);

let button = Button::new(1, "Increment".to_string());
button.on_click(move |_event, _ctx| {
    set_count.update(|c| *c += 1);
    // Signal update automatically triggers:
    // 1. Effect re-runs (if any depend on `count`)
    // 2. Component marked dirty
    // 3. Vello scene fragment regenerated
});
```

---

## 6. Event Context

### 6.1 EventContext Structure

Provides event handlers with controlled access to the system:

```rust
/// Context passed to event handlers
pub struct EventContext<'a> {
    /// The component receiving the event
    pub target: Gc<Component>,
    /// Global application state reference
    pub app_state: &'a mut AppState,
    /// Whether the event has been handled (stops propagation)
    is_handled: bool,
    /// Pointer capture state
    pointer_capture: &'a mut Option<Gc<Component>>,
}

impl<'a> EventContext<'a> {
    // --- Event Flow Control ---
    
    /// Mark event as handled, stopping bubbling
    pub fn stop_propagation(&mut self) {
        self.is_handled = true;
    }
    
    /// Check if event was handled
    pub fn is_handled(&self) -> bool {
        self.is_handled
    }
    
    // --- Pointer Capture ---
    
    /// Capture pointer to this component (receives all pointer events)
    pub fn capture_pointer(&mut self) {
        *self.pointer_capture = Some(Gc::clone(&self.target));
    }
    
    /// Release pointer capture
    pub fn release_pointer(&mut self) {
        *self.pointer_capture = None;
    }
    
    // --- Focus Management ---
    
    /// Request focus for this component
    pub fn request_focus(&mut self) {
        self.app_state.focus_state.pending_focus = Some(Gc::clone(&self.target));
    }
    
    /// Resign focus from this component
    pub fn resign_focus(&mut self) {
        if self.app_state.focus_state.focused.as_ref()
            .map_or(false, |f| Gc::ptr_eq(f, &self.target))
        {
            self.app_state.focus_state.pending_focus = None;
        }
    }
    
    // --- Coordinate Transformation ---
    
    /// Convert window coordinates to component-local coordinates
    pub fn local_position(&self, window_pos: Point) -> Point {
        if let Some(layout_node) = self.target.layout_node() {
            if let Some(layout) = layout_node.layout() {
                let origin = Point::new(
                    layout.location.x as f64,
                    layout.location.y as f64,
                );
                return window_pos - origin.to_vec2();
            }
        }
        window_pos
    }
    
    // --- Render Requests ---
    
    /// Request a repaint (marks component dirty)
    pub fn request_paint(&mut self) {
        self.target.mark_dirty();
    }
    
    /// Request layout recalculation
    pub fn request_layout(&mut self) {
        if let Some(ref mut layout_node) = *self.target.layout_node.borrow_mut() {
            layout_node.is_dirty = true;
        }
        self.target.mark_dirty();
    }
}
```

---

## 7. Event Dispatch Passes

### 7.1 Pointer Event Pass

```rust
pub fn run_pointer_event_pass(
    app_state: &mut AppState,
    event: &PointerEvent,
) -> Handled {
    let event_pos = event.position();
    
    // 1. Update last known pointer position
    app_state.last_pointer_pos = event_pos;
    
    // 2. Determine target component
    let target = if let Some(captured) = &app_state.pointer_capture {
        // Pointer is captured - all events go to capturing component
        Some(Gc::clone(captured))
    } else if let Some(pos) = event_pos {
        // Hit test to find component under pointer
        hit_test(&app_state.root_component, pos.into())
    } else {
        None
    };
    
    // 3. Update hover state
    let new_hovered = if app_state.pointer_capture.is_some() {
        None // Don't update hover during capture
    } else {
        target.clone()
    };
    update_hover_state(app_state, new_hovered);
    
    // 4. Dispatch event with bubbling
    if let Some(target) = target {
        dispatch_with_bubbling(app_state, &target, event)
    } else {
        Handled::No
    }
}

fn dispatch_with_bubbling<E>(
    app_state: &mut AppState,
    target: &Gc<Component>,
    event: &E,
) -> Handled
where
    E: Clone,
{
    let mut current = Some(Gc::clone(target));
    let mut handled = Handled::No;
    
    while let Some(component) = current {
        let mut ctx = EventContext {
            target: Gc::clone(&component),
            app_state,
            is_handled: false,
            pointer_capture: &mut app_state.pointer_capture,
        };
        
        // Call handler if registered
        if let Some(handler) = get_handler_for_event(&component, event) {
            handler(event, &mut ctx);
        }
        
        if ctx.is_handled() {
            handled = Handled::Yes;
            break;
        }
        
        // Bubble up to parent
        current = component.parent.borrow().clone();
    }
    
    // Auto-release pointer on Up/Cancel events
    if matches!(event, PointerEvent::Up(_) | PointerEvent::Cancel(_)) {
        app_state.pointer_capture = None;
    }
    
    handled
}
```

### 7.2 Text Event Pass

```rust
pub fn run_text_event_pass(
    app_state: &mut AppState,
    event: &TextEvent,
) -> Handled {
    // Text events go to focused component (or fallback)
    let target = app_state.focus_state.focused.clone()
        .or_else(|| app_state.focus_state.fallback.clone());
    
    if let Some(target) = target {
        // Handle Tab key for focus navigation
        if let TextEvent::Keyboard(key_event) = event {
            if key_event.key == Key::Named(NamedKey::Tab) 
                && key_event.state == KeyState::Down 
            {
                let forward = !key_event.modifiers.shift();
                if let Some(next) = find_next_focusable(app_state, &target, forward) {
                    app_state.focus_state.pending_focus = Some(next);
                    return Handled::Yes;
                }
            }
        }
        
        dispatch_with_bubbling(app_state, &target, event)
    } else {
        Handled::No
    }
}
```

### 7.3 Focus Navigation

```rust
/// Find the next focusable component in tab order
fn find_next_focusable(
    app_state: &AppState,
    current: &Gc<Component>,
    forward: bool,
) -> Option<Gc<Component>> {
    // Collect all focusable components in tree order
    let focusables = collect_focusables(&app_state.root_component);
    
    if focusables.is_empty() {
        return None;
    }
    
    // Find current index
    let current_idx = focusables.iter()
        .position(|c| Gc::ptr_eq(c, current))
        .unwrap_or(0);
    
    // Calculate next index with wrapping
    let len = focusables.len();
    let next_idx = if forward {
        (current_idx + 1) % len
    } else {
        (current_idx + len - 1) % len
    };
    
    Some(Gc::clone(&focusables[next_idx]))
}

fn collect_focusables(root: &Gc<Component>) -> Vec<Gc<Component>> {
    let mut result = Vec::new();
    collect_focusables_recursive(root, &mut result);
    result
}

fn collect_focusables_recursive(
    component: &Gc<Component>,
    result: &mut Vec<Gc<Component>>,
) {
    if component.accepts_focus() {
        result.push(Gc::clone(component));
    }
    for child in component.children.borrow().iter() {
        collect_focusables_recursive(child, result);
    }
}
```

---

## 8. Widget Status Updates

Following Masonry's Update event pattern for status changes:

```rust
/// Status update events (generated by the framework)
#[derive(Debug, Clone, PartialEq)]
pub enum StatusUpdate {
    /// Component was added to tree
    Mounted,
    /// Component is being removed from tree
    Unmounting,
    /// Hover status changed
    HoveredChanged(bool),
    /// Focus status changed
    FocusChanged(bool),
    /// Component enabled/disabled status changed
    DisabledChanged(bool),
    /// Component visibility changed
    VisibilityChanged(bool),
}

impl Component {
    /// Called when component status changes
    pub fn on_status_update(&self, update: &StatusUpdate) {
        match update {
            StatusUpdate::Mounted => {
                self.mount(self.parent.borrow().clone());
            }
            StatusUpdate::Unmounting => {
                self.unmount();
            }
            StatusUpdate::HoveredChanged(hovered) => {
                // Could trigger :hover pseudo-class style updates
                self.mark_dirty();
            }
            StatusUpdate::FocusChanged(focused) => {
                // Could trigger :focus pseudo-class style updates
                self.mark_dirty();
            }
            _ => {}
        }
    }
}
```

---

## 9. Integration with App Loop

### 9.1 Modified AppState

```rust
pub struct AppState<'a> {
    // ... existing fields ...
    
    // Event system additions
    pub focus_state: FocusState,
    pub pointer_capture: Option<Gc<Component>>,
    pub last_pointer_pos: Option<Point>,
    pub hovered_component: Option<Gc<Component>>,
}
```

### 9.2 Window Event Handler Integration

```rust
impl ApplicationHandler for AppState<'_> {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size);
            }
            WindowEvent::RedrawRequested => {
                // Apply pending focus changes before rendering
                self.apply_pending_focus();
                self.render_frame();
            }
            // --- NEW: Pointer events ---
            WindowEvent::CursorMoved { position, .. } => {
                let event = PointerEvent::Move(PointerMoveEvent {
                    position: position.into(),
                    // ... other fields
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let event = match state {
                    ElementState::Pressed => PointerEvent::Down(PointerButtonEvent {
                        button: map_button(button),
                        position: self.last_pointer_pos.unwrap_or_default(),
                        // ... other fields
                    }),
                    ElementState::Released => PointerEvent::Up(PointerButtonEvent {
                        button: map_button(button),
                        position: self.last_pointer_pos.unwrap_or_default(),
                        // ... other fields
                    }),
                };
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::CursorEntered { .. } => {
                run_pointer_event_pass(self, &PointerEvent::Enter(PointerInfo::default()));
            }
            WindowEvent::CursorLeft { .. } => {
                run_pointer_event_pass(self, &PointerEvent::Leave(PointerInfo::default()));
                // Clear hover state when cursor leaves window
                update_hover_state(self, None);
            }
            // --- NEW: Keyboard events ---
            WindowEvent::KeyboardInput { event, .. } => {
                let text_event = TextEvent::Keyboard(KeyboardEvent {
                    key: event.logical_key.clone(),
                    code: event.physical_key.into(),
                    state: if event.state.is_pressed() { KeyState::Down } else { KeyState::Up },
                    modifiers: get_current_modifiers(),
                    repeat: event.repeat,
                });
                run_text_event_pass(self, &text_event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::Ime(ime_event) => {
                let text_event = TextEvent::Ime(map_ime_event(ime_event));
                run_text_event_pass(self, &text_event);
                self.request_redraw_if_dirty();
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    // Clear pointer capture and active state on window defocus
                    self.pointer_capture = None;
                    update_hover_state(self, None);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let event = PointerEvent::Scroll(PointerScrollEvent {
                    delta: map_scroll_delta(delta),
                    position: self.last_pointer_pos.unwrap_or_default(),
                });
                run_pointer_event_pass(self, &event);
                self.request_redraw_if_dirty();
            }
            _ => {}
        }
    }
}

impl AppState<'_> {
    fn apply_pending_focus(&mut self) {
        if let Some(pending) = self.focus_state.pending_focus.take() {
            let old_focused = self.focus_state.focused.take();
            
            // Notify old focused component it lost focus
            if let Some(ref old) = old_focused {
                old.on_status_update(&StatusUpdate::FocusChanged(false));
            }
            
            // Notify new focused component it gained focus
            pending.on_status_update(&StatusUpdate::FocusChanged(true));
            self.focus_state.focused = Some(pending);
        }
    }
    
    fn request_redraw_if_dirty(&self) {
        if let Some(ref window) = self.window {
            if let Some(ref view) = self.view {
                if view.root_component.is_dirty() {
                    window.request_redraw();
                }
            }
        }
    }
}
```

---

## 10. Macro Support (Phase 2)

### 10.1 Declarative Event Binding

Future macro syntax for ergonomic event handling:

```rust
view! {
    <Button
        label="Click me"
        on:click=move |_| set_count.update(|c| *c += 1)
        on:hover=move |_| set_hovered.set(true)
    />
    <TextInput
        value=text
        on:input=move |e: &InputEvent| set_text.set(e.value.clone())
        on:keydown=move |e: &KeyboardEvent| {
            if e.key == Key::Named(NamedKey::Enter) {
                submit_form();
            }
        }
    />
}
```

### 10.2 Macro Expansion

The macro would expand to:

```rust
{
    let button = Button::new(1, "Click me".to_string());
    button.on_click(move |_, _| {
        set_count.update(|c| *c += 1);
    });
    button.on_pointer_enter(move |_, _| {
        set_hovered.set(true);
    });
    button
}
```

---

## 11. Comparison with Xilem/Masonry

| Aspect | Xilem/Masonry | Rvue |
|--------|---------------|------|
| **Memory Model** | Rust ownership (`&mut`, `Rc`) | GC (`Gc<T>`) – cyclic refs allowed |
| **Event Handlers** | Methods on Widget trait | Closures in `Gc<Component>` |
| **State Updates** | Full rebuild + diff | Signal mutation → targeted repaint |
| **Hit Testing** | Widget tree traversal | Taffy layout tree traversal |
| **Focus** | WidgetId-based | `Gc<Component>` references |
| **Actions** | Type-erased Action submission | Direct signal modification |

**Key Insight**: Rvue's event system is *simpler* than Masonry's because:
1. No need for `WidgetId` lookup – direct `Gc` references
2. No Action indirection – handlers modify signals directly
3. GC handles cleanup – no manual lifecycle management

---

## 12. Implementation Phases

### Phase 1: Core Event Infrastructure (MVP)
- [x] Define event types (`PointerEvent`, `TextEvent`, `AccessEvent`)
- [ ] Hit testing using Taffy layout
- [ ] Event handler storage in `Component`
- [ ] Pointer capture mechanism
- [ ] Basic focus management
- [ ] Event bubbling
- [ ] Integration with `app.rs` event loop

### Phase 2: Enhanced Interactivity
- [ ] Tab navigation
- [ ] Hover state tracking with `:hover` pseudo-class
- [ ] Cursor icon management
- [ ] Scroll event handling
- [ ] IME text input
- [ ] Accessibility events (accesskit)

### Phase 3: Advanced Features
- [ ] Touch event support
- [ ] Gesture recognition
- [ ] Drag and drop
- [ ] Multi-pointer support
- [ ] Custom event types

---

## 13. Open Questions

1. **Event Handler Ordering**: When multiple handlers exist (e.g., parent and child), should we support `capture` phase (top-down) in addition to `bubbling` (bottom-up)?

2. **Synthetic Events**: Should `click` be a first-class event, or synthesized from `down` + `up`? (Currently proposing first-class for simplicity)

3. **GC Pressure**: Frequent event dispatch may create GC pressure. Should handlers be pre-allocated or use a pool?

4. **Async Handlers**: Should `async` event handlers be supported? If so, how do we handle signal access across await points?

---

## 14. Required Crates

This section documents external dependencies required to implement the Rvue event system.

### 14.1 Core Event Handling

#### `ui-events` (v0.3.0)
**Source**: [crates.io/crates/ui-events](https://crates.io/crates/ui-events) | [docs.rs](https://docs.rs/ui-events)

Cross-platform input event abstraction following W3C UI Events specification. Used by Xilem/Masonry for interoperability.

```toml
[dependencies]
ui-events = { version = "0.3.0", default-features = false, features = ["kurbo"] }
```

**Provides:**
- `PointerEvent` - Down/Move/Up with pressure, tilt, pointer type
- `KeyboardEvent` - Key codes, modifiers, location
- `PointerButton` - Button enumeration (Primary, Secondary, etc.)
- W3C-compatible event model

**Why use it:**
- Battle-tested in Xilem/Masonry
- Consistent event model across platforms
- Built-in kurbo integration for coordinates
- Future-proof for cross-framework interoperability

---

#### `ui-events-winit` (v0.3.0)
**Source**: [crates.io/crates/ui-events-winit](https://crates.io/crates/ui-events-winit)

Adapter to convert `winit` events into `ui-events` types.

```toml
[dependencies]
ui-events-winit = { version = "0.3.0", default-features = false }
```

**Provides:**
- `winit::event::WindowEvent` → `ui_events::PointerEvent` conversion
- `winit::event::KeyboardInput` → `ui_events::KeyboardEvent` conversion
- Handles platform-specific quirks

**Why use it:**
- Eliminates boilerplate event mapping code
- Handles edge cases (modifier state, repeat detection)
- Maintained alongside `ui-events`

---

### 14.2 Accessibility

#### `accesskit` (v0.21.1)
**Source**: [accesskit.dev](https://accesskit.dev) | [GitHub](https://github.com/AccessKit/accesskit)

Cross-platform accessibility framework for screen reader support.

```toml
[dependencies]
accesskit = "0.21.1"
```

**Provides:**
- `Action` - Accessibility actions (Click, Focus, Scroll, etc.)
- `ActionData` - Additional action data
- `Node` - Accessibility tree node representation
- `Role` - Widget roles (Button, TextInput, etc.)

**Why use it:**
- Industry-standard for Rust GUI accessibility
- Used by egui, Bevy, Slint, Xilem
- Platform adapters for Windows (UIA), macOS (NSAccessibility), Linux (AT-SPI)

---

#### `accesskit_winit` (v0.29.2)
**Source**: [crates.io/crates/accesskit_winit](https://crates.io/crates/accesskit_winit)

Integration layer between `accesskit` and `winit`.

```toml
[dependencies]
accesskit_winit = { version = "0.29.2", default-features = false, features = [
    "accesskit_unix",
    "tokio",
    "rwh_06",
] }
```

**Provides:**
- Window-level accessibility tree management
- Event bridging between windowing system and accessibility APIs
- Platform-specific adapters

---

### 14.3 Coordinate Systems

#### `dpi` (v0.1.2)
**Source**: [crates.io/crates/dpi](https://crates.io/crates/dpi)

Types for handling HiDPI/scaling and physical/logical coordinates.

```toml
[dependencies]
dpi = "0.1.2"
```

**Provides:**
- `PhysicalPosition<T>` / `PhysicalSize<T>` - Device-pixel coordinates
- `LogicalPosition<T>` / `LogicalSize<T>` - Scaled coordinates
- Conversion methods with scale factor

**Why use it:**
- Consistent coordinate handling across high-DPI displays
- Type-safe distinction between physical and logical coordinates
- Same types used by winit

---

#### `kurbo` (v0.13.0)
**Source**: [crates.io/crates/kurbo](https://crates.io/crates/kurbo) | Part of Linebender ecosystem

2D geometry library with affine transforms.

```toml
[dependencies]
kurbo = "0.13.0"
```

**Provides:**
- `Point`, `Vec2`, `Size`, `Rect` - 2D primitives
- `Affine` - 2D affine transformation matrices
- Hit testing utilities (`Rect::contains`)

**Already in rvue**: Via `vello` dependency

---

### 14.4 Cursor Management

#### `cursor-icon` (v1.2.0)
**Source**: [crates.io/crates/cursor-icon](https://crates.io/crates/cursor-icon)

Cross-platform cursor icon enumeration.

```toml
[dependencies]
cursor-icon = "1.2.0"
```

**Provides:**
- `CursorIcon` enum with 36+ cursor types
- Platform-agnostic cursor representation
- Supports: Default, Pointer, Text, Move, Resize variants, etc.

**Why use it:**
- Standard interoperability type for GUI frameworks
- `winit` accepts `cursor-icon::CursorIcon` directly
- Consistent cursor behavior across platforms

---

### 14.5 Utility Crates

#### `smallvec` (v1.15.1)
**Source**: [crates.io/crates/smallvec](https://crates.io/crates/smallvec)

Stack-allocated Vec for small collections.

```toml
[dependencies]
smallvec = "1.15.1"
```

**Use in event system:**
- Hit test result collection (typically 1-4 widgets in path)
- Focus chain traversal
- Avoids heap allocation for common cases

---

#### `bitflags` (v2.10.0)
**Source**: [crates.io/crates/bitflags](https://crates.io/crates/bitflags)

Macro for efficient bitflag types.

```toml
[dependencies]
bitflags = "2.10.0"
```

**Use in event system:**
- Modifier key state (Shift, Ctrl, Alt, Meta)
- Pointer button state (multi-button tracking)
- Component status flags

---

### 14.6 Summary: Cargo.toml Additions

Add these to `crates/rvue/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
rudo-gc = "0.1"
vello = "0.7"
wgpu = "27"
winit = "0.30"
taffy = "0.9"
parley = "0.7.0"
# ... other existing deps

# NEW: Event System Dependencies
ui-events = { version = "0.3.0", default-features = false, features = ["kurbo"] }
ui-events-winit = { version = "0.3.0", default-features = false }
accesskit = "0.21.1"
accesskit_winit = { version = "0.29.2", default-features = false, features = [
    "accesskit_unix",
    "tokio",
    "rwh_06",
] }
cursor-icon = "1.2.0"
dpi = "0.1.2"
smallvec = "1.15.1"
bitflags = "2.10.0"
```

### 14.7 Version Alignment with Xilem

For maximum compatibility with Xilem/Masonry patterns, align versions with `learn-projects/xilem/Cargo.toml`:

| Crate | Xilem Version | Rvue Target |
|-------|---------------|-------------|
| `ui-events` | 0.3.0 | 0.3.0 |
| `ui-events-winit` | 0.3.0 | 0.3.0 |
| `accesskit` | 0.21.1 | 0.21.1 |
| `cursor-icon` | 1.2.0 | 1.2.0 |
| `dpi` | 0.1.2 | 0.1.2 |
| `winit` | 0.30.12 | 0.30 ✓ |
| `vello` | 0.7.0 | 0.7 ✓ |
| `kurbo` | 0.13.0 | via vello |
| `parley` | 0.7.0 | 0.7.0 ✓ |

---

## 15. References

- [Xilem Masonry Event System](learn-projects/xilem/masonry_core/src/passes/event.rs)
- [Masonry Widget Trait](learn-projects/xilem/masonry_core/src/core/widget.rs)
- [Masonry Concepts Documentation](learn-projects/xilem/masonry_core/src/doc/masonry_concepts.md)
- [Rvue Core Design Document](docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md)
- [winit Event Documentation](https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html)
- [accesskit Actions](https://docs.rs/accesskit/latest/accesskit/enum.Action.html)
