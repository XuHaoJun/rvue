# Overflow & ScrollBar Implementation Plan

## Overview

Implement a Web-compatible overflow/scrollbar system for rvue, allowing widgets like Flex to natively support `overflow: auto`, `overflow: hidden`, `overflow: scroll`, and `overflow: visible`.

## Research Findings

### Xilem Portal/ScrollBar Analysis

Xilem (via Masonry) provides a complete reference implementation:

**Key Files:**
- `/learn-projects/xilem/masonry/src/widgets/portal.rs` (792 lines)
- `/learn-projects/xilem/masonry/src/widgets/scroll_bar.rs` (348 lines)

**Key Mechanisms:**

1. **ScrollBar Interaction:**
   - Mouse wheel → `viewport_pos` update
   - Drag → `cursor_progress` ↔ `viewport_pos` bidirectional binding
   - `grab_anchor` tracks click position relative to thumb

2. **Overflow Clipping:**
   ```rust
   ctx.set_clip_path(size.to_rect());  // Clip to container bounds
   ```

3. **Content Offset via Translation:**
   ```rust
   ctx.set_child_scroll_translation(&mut child, Vec2::new(-viewport.x, -viewport.y));
   ```

### Taffy Overflow Support Analysis

Taffy already provides comprehensive layout-level overflow support:

**Style Definition (`style/mod.rs:311-326`):**
```rust
pub enum Overflow {
    Visible,  // Default, content overflows, contributes to parent scroll region
    Clip,     // Clip content, don't contribute to parent scroll region, min-size = content
    Hidden,   // Clip content, min-size = 0
    Scroll,   // Clip content, min-size = 0, reserve space for scrollbar
}

pub struct Style {
    pub overflow: Point<Overflow>,      // Independent x/y control
    pub scrollbar_width: f32,           // Scrollbar width in points (default: 0.0)
}
```

**Layout Output (`tree/layout.rs:226-248, 325-346`):**
```rust
pub struct Layout {
    pub size: Size<f32>,           // Node size
    pub content_size: Size<f32>,   // Actual content size (feature = "content_size")
    pub scrollbar_size: Size<f32>, // Scrollbar dimensions
    pub border: Rect<f32>,         // Border sizes (used in scroll calculations)
}

impl Layout {
    /// scroll_width = max(0, content_size.width + min(scrollbar_size.width, size.width) 
    ///                     - size.width + border.right)
    pub fn scroll_width(&self) -> f32;
    pub fn scroll_height(&self) -> f32;
}
```

**Important Notes:**
1. Taffy lacks `Overflow::Auto` - rvue must implement dynamic visibility at the application layer
2. `content_size` is behind `#[cfg(feature = "content_size")]` - ensure this feature is enabled
3. `scroll_width()`/`scroll_height()` include border considerations

---

## Design Decision: Native Widget Overflow

Instead of a separate `ScrollView` wrapper, implement overflow natively on existing widgets (Flex, Grid, etc.).

**Rationale:**
- Web-compatible: `<div style="overflow: auto">` directly maps to `Flex { style: stylesheet! { overflow: auto } }`
- No additional wrapper layers
- Consistent developer experience for web developers

---

## CSS Behavior Mapping

| CSS Value | Behavior | Taffy Mapping | rvue Implementation |
|-----------|----------|---------------|---------------------|
| `overflow: visible` | Content overflows visibly | `Overflow::Visible` | No clipping, no scrollbar |
| `overflow: clip` | Content clipped, no scroll region | `Overflow::Clip` | Clip content, no scrollbar, no scroll interaction |
| `overflow: hidden` | Content clipped, no scrollbar | `Overflow::Hidden` | Clip content, hide scrollbar, min-size = 0 |
| `overflow: scroll` | Always show scrollbars | `Overflow::Scroll` | Clip content, always show scrollbars |
| `overflow: auto` | Show scrollbars only when needed | *rvue-only* | Clip content, show scrollbars when `scroll_width > 0` |

**Key Differences:**
- `clip` vs `hidden`: Both clip content, but `clip` keeps content-based minimum sizing while `hidden` sets min-size to 0
- `auto` requires **two-pass layout** (see below)

**Shorthand:**
```css
.div { overflow: auto; }      /* x and y both auto */
.div { overflow-x: scroll; }  /* only horizontal */
.div { overflow-y: hidden; }  /* only vertical */
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Flex Widget (or any container widget)                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 1. Style Reading                                           │  │
│  │    overflow_x, overflow_y from ComputedStyles              │  │
│  │                                                             │  │
│  │ 2. Taffy Layout                                            │  │
│  │    - Computes container_size vs content_size               │  │
│  │    - Reserves scrollbar_space if Overflow::Scroll          │  │
│  │    - Returns scroll_width, scroll_height                   │  │
│  │                                                             │  │
│  │ 3. Scrollbar Visibility Logic (Two-Pass for Auto)         │  │
│  │    - overflow == Visible: hidden                           │  │
│  │    - overflow == Clip: hidden                              │  │
│  │    - overflow == Hidden: hidden                            │  │
│  │    - overflow == Scroll: visible                           │  │
│  │    - overflow == Auto:                                     │  │
│  │        Pass 1: Layout without scrollbar space              │  │
│  │        Pass 2: If overflow detected, relayout with space   │  │
│  │    - **State Update**: Write scroll_width/height to        │  │
│  │        FlexScrollState (in user_data) after Layout         │  │
│  │                                                             │  │
│  │ 4. Rendering                                               │  │
│  │    - Read scroll state from user_data                      │  │
│  │    - set_clip_path(container_bounds) if !Visible           │  │
│  │    - Translate content by scroll_offset                    │  │
│  │    - Render scrollbars (horizontal + vertical)             │  │
│  │                                                             │  │
│  │ 5. Event Handling (Bubble Up)                              │  │
│  │    - PointerScroll (Input): Bubble up from target          │  │
│  │      → Container checks if it can scroll                   │  │
│  │      → If yes, consume event & update scroll_offset        │  │
│  │      → If no, continue bubbling                            │  │
│  │    - Scrollbar Drag: Direct manipulation                   │  │
│  │    - Dispatch ScrollEvent (State Change) to target only    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Style System (`rvue-style`)

**New Files:**
- `crates/rvue-style/src/properties/overflow.rs`

**Changes:**
- `crates/rvue-style/src/properties/mod.rs` - Export overflow module
- `crates/rvue-style/src/properties/computed_styles.rs` - Add `overflow_x`, `overflow_y`

**Code Structure:**
```rust
// crates/rvue-style/src/properties/overflow.rs

/// CSS overflow behavior - extended for web compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,    // Added for CSS compatibility (Taffy has this)
    Hidden,
    Scroll,
    Auto,    // rvue extension (Taffy doesn't have this)
}

impl Overflow {
    /// Whether this overflow mode creates a scroll container
    /// (i.e., content is clipped and scroll interaction is possible)
    pub fn is_scroll_container(self) -> bool {
        matches!(self, Self::Hidden | Self::Scroll | Self::Auto)
    }
    
    /// Whether content should be clipped
    pub fn should_clip(self) -> bool {
        !matches!(self, Self::Visible)
    }
    
    /// Convert to Taffy overflow for layout computation.
    /// Note: Auto is resolved by checking content vs container size,
    /// then using either Hidden (no visible scrollbar) or Scroll (with scrollbar)
    pub fn to_taffy_overflow(self, needs_scrollbar: bool) -> taffy::style::Overflow {
        match self {
            Self::Visible => taffy::style::Overflow::Visible,
            Self::Clip => taffy::style::Overflow::Clip,
            Self::Hidden => taffy::style::Overflow::Hidden,
            Self::Scroll => taffy::style::Overflow::Scroll,
            Self::Auto => {
                if needs_scrollbar {
                    taffy::style::Overflow::Scroll
                } else {
                    taffy::style::Overflow::Hidden
                }
            }
        }
    }
}

// Note: Separate OverflowX/OverflowY types not needed since Overflow is used
// directly with Point<Overflow> for independent axis control (matching Taffy)
```

### Phase 2: Event System (`rvue`)

**Existing Infrastructure (Already Present):**
- `PointerScrollEvent` already exists in `event/types.rs` (lines 58-63)
- `PointerEvent::Scroll(PointerScrollEvent)` variant exists (line 84)
- `ScrollDelta` enum supports both `Line(f64)` and `Pixel(f64, f64)` (lines 52-56)

**Changes Needed:**
- `crates/rvue/src/event/handler.rs` - Add `on_scroll` handler using existing `DynHandler` pattern
- `crates/rvue/src/event/dispatch.rs` - **Logic: Bubble scroll events up from target to nearest scrollable container**

**Event Handler Structure (Following Existing Pattern):**
```rust
// crates/rvue/src/event/handler.rs

// Add to DynHandler enum:
enum DynHandler {
    // ... existing variants ...
    OneArgPointerScroll(Box<dyn Fn(&PointerScrollEvent)>),
    TwoArgPointerScroll(Box<dyn Fn(&PointerScrollEvent, &mut EventContext)>),
}

// Add to AnyEventHandler enum:
pub enum AnyEventHandler {
    // ... existing variants ...
    PointerScroll(EventHandler<PointerScrollEvent>),
}

// Add to EventHandlers struct:
#[derive(Default, Clone)]
pub struct EventHandlers {
    // ... existing handlers (on_pointer_down, on_click, etc.) ...
    pub on_scroll: Option<EventHandler<PointerScrollEvent>>,
}

// Add impl for EventHandler<PointerScrollEvent> following existing pattern
impl EventHandler<PointerScrollEvent> {
    pub fn new_0arg<F>(handler: F) -> Self
    where F: Fn() + 'static { /* ... */ }
    
    pub fn new_1arg<F>(handler: F) -> Self
    where F: Fn(&PointerScrollEvent) + 'static { /* ... */ }
    
    pub fn new<F>(handler: F) -> Self
    where F: Fn(&PointerScrollEvent, &mut EventContext) + 'static { /* ... */ }
    
    pub fn call(&self, event: &PointerScrollEvent, ctx: &mut EventContext) { /* ... */ }
}
```

**Dispatch Logic Update:**
1. `PointerScrollEvent` is an **Input Event**. It starts at the global `hovered_component` (or focused).
2. Dispatcher walks up the tree (bubble).
3. At each node, check:
   - Does this node have `on_scroll` handler? (Application level logic)
   - Is this node a **Scroll Container**? (System level logic - `overflow != Visible/Clip` AND `scroll_width > 0`)
4. If Scroll Container:
   - Consume event (stop propagation unless explicitly passed).
   - Update `scroll_offset` in `FlexScrollState` (user_data).
   - Component marks dirty -> Re-render.
5. If not, continue bubbling.

---

### Phase 3: Flex Widget Enhancement (`rvue`)

**Current Flex Architecture (from `widgets/flex.rs`):**
- `Flex` struct: Builder with `ReactiveValue<T>` fields (direction, gap, align_items, etc.)
- `FlexState` struct: Runtime state with `Gc<Component>` + effect trackers
- Uses `ComponentProps::Flex { direction, gap, ... }` for prop storage
- Styles passed via `ReactiveStyles` and stored in `ComponentProps::Flex { styles }`

**Changes:**
- `crates/rvue/src/widgets/flex.rs` - Add overflow and scroll state
- `crates/rvue/src/layout/node.rs` - Support Taffy overflow property

**Extended Flex Builder (Following Existing Pattern):**
```rust
// crates/rvue/src/widgets/flex.rs

#[derive(Clone)]
pub struct Flex {
    direction: ReactiveValue<FlexDirection>,
    gap: ReactiveValue<f32>,
    align_items: ReactiveValue<AlignItems>,
    justify_content: ReactiveValue<JustifyContent>,
    styles: Option<ReactiveStyles>,
    // NEW: Overflow properties
    overflow_x: ReactiveValue<Overflow>,
    overflow_y: ReactiveValue<Overflow>,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            // ... existing defaults ...
            overflow_x: ReactiveValue::Static(Overflow::Visible),
            overflow_y: ReactiveValue::Static(Overflow::Visible),
        }
    }
    
    /// Set x-axis overflow behavior
    pub fn overflow_x(mut self, overflow: impl IntoReactiveValue<Overflow>) -> Self {
        self.overflow_x = overflow.into_reactive();
        self
    }
    
    /// Set y-axis overflow behavior  
    pub fn overflow_y(mut self, overflow: impl IntoReactiveValue<Overflow>) -> Self {
        self.overflow_y = overflow.into_reactive();
        self
    }
    
    /// Set both axes to same overflow behavior
    pub fn overflow(self, overflow: impl IntoReactiveValue<Overflow> + Clone) -> Self {
        self.overflow_x(overflow.clone()).overflow_y(overflow)
    }
}
```

**Extended FlexState (Runtime Scroll State):**
```rust
// Stored in Component via user_data (CONFIRMED: Box<dyn Any>)
// This keeps runtime state separate from declarative props
pub struct FlexScrollState {
    pub scroll_offset_x: f32,
    pub scroll_offset_y: f32,
    // Cached from layout computation (Updated Post-Layout)
    pub scroll_width: f32,       // From layout.scroll_width()
    pub scroll_height: f32,      // From layout.scroll_height()
    pub container_width: f32,
    pub container_height: f32,
    pub show_horizontal_scrollbar: bool,
    pub show_vertical_scrollbar: bool,
}

impl FlexScrollState {
    pub fn scroll_by(&mut self, delta_x: f32, delta_y: f32) {
        self.scroll_offset_x = (self.scroll_offset_x + delta_x)
            .clamp(0.0, self.scroll_width.max(0.0));
        self.scroll_offset_y = (self.scroll_offset_y + delta_y)
            .clamp(0.0, self.scroll_height.max(0.0));
    }
}
```

**Layout Integration (in `layout/node.rs`):**
```rust
// In calculate_layout() or build_in_tree()

// 1. Compute Taffy Layout (including 2-pass for Auto)

// 2. Post-Layout Update (CONFIRMED TIMING)
// Immediately after valid layout is obtained, update FlexScrollState in user_data
if supports_scroll(component) {
    let mut user_data = component.user_data.borrow_mut();
    // Create or update FlexScrollState
    let state = user_data.get_or_insert_with(|| Box::new(FlexScrollState::default()))
        .downcast_mut::<FlexScrollState>()
        .unwrap();
        
    state.scroll_width = layout.scroll_width();
    state.scroll_height = layout.scroll_height();
    state.container_width = layout.size.width;
    state.container_height = layout.size.height;
    
    // Clamp offset in case content shrank
    state.scroll_offset_x = state.scroll_offset_x.clamp(0.0, state.scroll_width);
    state.scroll_offset_y = state.scroll_offset_y.clamp(0.0, state.scroll_height);
}
```

**Layout Integration (in `layout/node.rs`):**
```rust
// In component_to_taffy_style(), add to Flex case:
ComponentType::Flex => {
    // ... existing direction, gap, align_items, justify_content ...
    
    // Add overflow support
    if let Some(computed) = computed {
        if let Some(ref overflow_x) = computed.overflow_x {
            style.overflow.x = overflow_to_taffy(overflow_x);
        }
        if let Some(ref overflow_y) = computed.overflow_y {
            style.overflow.y = overflow_to_taffy(overflow_y);
        }
        // Reserve scrollbar space for Scroll mode
        if style.overflow.x == taffy::style::Overflow::Scroll 
           || style.overflow.y == taffy::style::Overflow::Scroll {
            style.scrollbar_width = 10.0;
        }
    }
}

fn overflow_to_taffy(overflow: &Overflow) -> taffy::style::Overflow {
    match overflow {
        Overflow::Visible => taffy::style::Overflow::Visible,
        Overflow::Clip => taffy::style::Overflow::Clip,
        Overflow::Hidden => taffy::style::Overflow::Hidden,
        Overflow::Scroll | Overflow::Auto => taffy::style::Overflow::Scroll,
    }
}
```

---

### Phase 4: ScrollBar Widget (`rvue`)

**New File:**
- `crates/rvue/src/widgets/scroll_bar.rs`

**Based on Xilem's implementation:**

```rust
pub struct ScrollBar {
    axis: ScrollAxis,  // Horizontal or Vertical
    portal_size: f32,       // Container size
    content_size: f32,      // Content size
    scroll_offset: f32,     // Current scroll position
    cursor_progress: f64,   // 0.0 to 1.0
    grab_anchor: Option<f64>,
    moved: bool,
}

impl ScrollBar {
    /// Calculate thumb (cursor) size based on portal/content ratio
    pub fn compute_thumb_length(&self, track_length: f32) -> f32 {
        let ratio = (self.portal_size / self.content_size).clamp(0.0, 1.0);
        (ratio * track_length).max(MIN_THUMB_LENGTH)
    }
}
```

**Default ScrollBar Style:**
- Width: 10px
- Track: Dark gray semi-transparent
- Thumb: White with rounded corners
- Hover: Slightly darker

---

### Phase 5: Rendering Integration (`rvue`)

**Existing Render Architecture (from `render/widget.rs`):**
- `render_component()` dispatches by `ComponentType`
- Each widget type has dedicated render function (e.g., `render_flex_background()`)
- Uses `vello::Scene` for rendering
- Styles resolved via `get_styles()` helper
- Layout accessed via `component.layout_node().layout()`

**Changes:**
- `crates/rvue/src/render/widget.rs` - Extend `render_flex_background()` for clipping

**Clipping Implementation (Following Existing Pattern):**
```rust
// In render/widget.rs, modify render_flex_background:

fn render_flex_background(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    stylesheet: Option<&Stylesheet>,
) {
    if let ComponentProps::Flex { styles: _, .. } = &*component.props.borrow() {
        let styles = get_styles(component, stylesheet);
        let layout_node = component.layout_node();

        if let Some(layout) = layout_node {
            if let Some(flex_layout) = layout.layout() {
                let x = flex_layout.location.x as f64;
                let y = flex_layout.location.y as f64;
                let width = flex_layout.size.width as f64;
                let height = flex_layout.size.height as f64;

                // Get overflow settings
                let should_clip = styles.overflow_x
                    .map(|o| o.should_clip())
                    .unwrap_or(false)
                    || styles.overflow_y
                    .map(|o| o.should_clip())
                    .unwrap_or(false);
                
                // Render background (existing code)
                let rect = Rect::new(x, y, x + width, y + height);
                if let Some(bg) = styles.background_color.as_ref() {
                    let rgb = bg.0.0;
                    let bg_color = Color::from_rgb8(rgb.r, rgb.g, rgb.b);
                    scene.fill(Fill::NonZero, Affine::IDENTITY, bg_color, None, &rect);
                }

                // Render border (existing code)
                let border_radius = styles.border_radius.as_ref().map(|r| r.0 as f64).unwrap_or(0.0);
                render_border(scene, Affine::IDENTITY, &Some(styles), x, y, width, height, border_radius);
            }
        }
    }
}

// NEW: Add clip layer support in render_children
fn render_children_with_scroll(
    component: &Gc<Component>,
    scene: &mut vello::Scene,
    transform: Affine,
    already_appended: &mut FxHashSet<u64>,
    stylesheet: Option<&Stylesheet>,
) {
    let styles = get_styles(component, stylesheet);
    
    // Check if we need clipping
    let should_clip = styles.overflow_x
        .map(|o| o.should_clip())
        .unwrap_or(false)
        || styles.overflow_y
        .map(|o| o.should_clip())
        .unwrap_or(false);
    
    // Get scroll state from user_data
    let scroll_state = component.user_data.borrow()
        .as_ref()
        .and_then(|d| d.downcast_ref::<FlexScrollState>())
        .cloned();
    
    if should_clip {
        if let Some(layout_node) = component.layout_node() {
            if let Some(layout) = layout_node.layout() {
                let clip_rect = Rect::new(
                    0.0, 0.0,
                    layout.size.width as f64,
                    layout.size.height as f64,
                );
                
                // Push clip layer
                scene.push_layer(
                    vello::peniko::BlendMode::default(),
                    1.0,
                    transform,
                    &clip_rect,
                );
                
                // Translate children by scroll offset
                let scroll_transform = if let Some(ref state) = scroll_state {
                    Affine::translate((-state.scroll_offset_x as f64, -state.scroll_offset_y as f64))
                } else {
                    Affine::IDENTITY
                };
                
                // Render clipped children
                for child in component.children.borrow().iter() {
                    let child_transform = get_child_transform(child);
                    render_component(
                        child,
                        scene,
                        transform * scroll_transform * child_transform,
                        already_appended,
                        stylesheet,
                    );
                }
                
                // Pop clip layer
                scene.pop_layer();
                
                // Render scrollbars on top (outside clip)
                if let Some(ref state) = scroll_state {
                    if state.show_vertical_scrollbar {
                        render_vertical_scrollbar(scene, transform, layout, state);
                    }
                    if state.show_horizontal_scrollbar {
                        render_horizontal_scrollbar(scene, transform, layout, state);
                    }
                }
                
                return;
            }
        }
    }
    
    // Fallback: render without clipping (existing behavior)
    for child in component.children.borrow().iter() {
        let child_transform = get_child_transform(child);
        render_component(
            child,
            scene,
            transform * child_transform,
            already_appended,
            stylesheet,
        );
    }
}

fn render_vertical_scrollbar(
    scene: &mut vello::Scene,
    transform: Affine,
    layout: &taffy::Layout,
    state: &FlexScrollState,
) {
    const SCROLLBAR_WIDTH: f64 = 10.0;
    const THUMB_MIN_LENGTH: f64 = 20.0;
    
    let container_height = layout.size.height as f64;
    let content_height = container_height + state.scroll_height as f64;
    
    // Track
    let track_rect = Rect::new(
        layout.size.width as f64 - SCROLLBAR_WIDTH,
        0.0,
        layout.size.width as f64,
        container_height,
    );
    scene.fill(Fill::NonZero, transform, Color::from_rgba8(0, 0, 0, 77), None, &track_rect);
    
    // Thumb
    let thumb_ratio = container_height / content_height;
    let thumb_height = (thumb_ratio * container_height).max(THUMB_MIN_LENGTH);
    let thumb_offset = (state.scroll_offset_y as f64 / state.scroll_height as f64) 
        * (container_height - thumb_height);
    
    let thumb_rect = RoundedRect::new(
        layout.size.width as f64 - SCROLLBAR_WIDTH + 2.0,
        thumb_offset + 2.0,
        layout.size.width as f64 - 2.0,
        thumb_offset + thumb_height - 2.0,
        4.0,
    );
    scene.fill(Fill::NonZero, transform, Color::from_rgba8(255, 255, 255, 200), None, &thumb_rect);
}

fn render_horizontal_scrollbar(
    scene: &mut vello::Scene,
    transform: Affine,
    layout: &taffy::Layout,
    state: &FlexScrollState,
) {
    // Similar to vertical, but horizontal orientation
    // ... (implementation follows same pattern)
}
```

---

## File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/rvue-style/src/properties/overflow.rs` | New | Overflow enum and types |
| `crates/rvue-style/src/properties/mod.rs` | Modify | Export overflow module |
| `crates/rvue-style/src/properties/computed_styles.rs` | Modify | Add overflow_x, overflow_y |
| `crates/rvue/src/event/handler.rs` | Modify | Add on_scroll handler |
| `crates/rvue/src/event/dispatch.rs` | Modify | Implement Scroll dispatch |
| `crates/rvue/src/event/types.rs` | Modify | Ensure PointerScrollEvent |
| `crates/rvue/src/component.rs` | Modify | Add on_scroll() method |
| `crates/rvue/src/widgets/flex.rs` | Modify | Add scroll state and rendering |
| `crates/rvue/src/widgets/scroll_bar.rs` | New | ScrollBar widget |
| `crates/rvue/src/widgets/mod.rs` | Modify | Export ScrollBar |
| `crates/rvue/src/layout/node.rs` | Modify | Use Taffy overflow |
| `crates/rvue/src/render/widget.rs` | Modify | Implement clipping |
| `crates/rvue/src/render/scene.rs` | Modify | Integrate scroll state |

---

## Usage Examples

**Option A: Builder Pattern (Recommended Initially):**
```rust
// Basic overflow auto with fixed height
Flex::new()
    .overflow_y(Overflow::Auto)
    .styles(stylesheet! {
        height: 400.px;
        background_color: #1a1a2e;
    })

// Independent axis control
Flex::new()
    .overflow_x(Overflow::Hidden)
    .overflow_y(Overflow::Scroll)
    .direction(FlexDirection::Column)
    
// With reactive overflow
let overflow_mode = create_signal(Overflow::Auto);
Flex::new()
    .overflow_y(overflow_mode.get())
```

**Option B: Via ReactiveStyles (Future Enhancement):**
```rust
// Using stylesheet! macro (requires macro extension)
Flex::new()
    .styles(stylesheet! {
        overflow: auto;
        overflow_x: hidden;
        overflow_y: scroll;
        height: 400.px;
    })
```

**Scroll Event Handling:**
```rust
// Using existing event handler pattern
let scroll_handler = EventHandler::<PointerScrollEvent>::new_1arg(|event| {
    match event.delta {
        ScrollDelta::Line(y) => info!("Scrolled {} lines", y),
        ScrollDelta::Pixel(x, y) => info!("Scrolled ({}, {}) pixels", x, y),
    }
});

// Attach to component (mechanism TBD based on current event dispatch)
```

---

## Default ScrollBar Appearance

```
┌─────────────────────────────────────────────────────────────┐
│ ┌─────────────────────────────────────────────────────────┐ │
│ │                                                         │ │
│ │                    Content Area                         │ │
│ │                                                         │ │
│ │                                                         │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│ │
│ └─────────────────────────────────────────────────────────┘ │
│                               ┌───────────────────────────┐ │
│                               │████████████████████████████│ │
│                               └───────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

Properties:
- Width: 10px
- Track: rgba(0, 0, 0, 0.3)
- Thumb: white with 4px border radius
- Hover: darker thumb
```

---

## Limitations

1. **No RTL Support** - Scrollbars always on right/bottom
2. **No Virtual Scrolling** - Only basic scrollbar functionality
3. **No Custom Scrollbar Styling** - Fixed appearance initially
4. **No `scrollbar-gutter: stable`** - Future enhancement for reserving scrollbar space
5. **No Touch/Swipe Gestures** - Mouse wheel and drag only initially
6. **No Smooth Scrolling Animation** - Instant scroll position updates

---

## References

- [Xilem Portal Implementation](/learn-projects/xilem/masonry/src/widgets/portal.rs)
- [Xilem ScrollBar Implementation](/learn-projects/xilem/masonry/src/widgets/scroll_bar.rs)
- [Taffy Style Module](/learn-projects/taffy/src/style/mod.rs)
- [Taffy Layout Output](/learn-projects/taffy/src/tree/layout.rs)
- [MDN overflow Property](https://developer.mozilla.org/en-US/docs/Web/CSS/overflow)

---

## Document Info

- Created: 2026-02-01
- Updated: 2026-02-01 (Corrected Taffy API references, added two-pass layout)
- Updated: 2026-02-01 (Arch refinements: Event Bubble, State Location, Update Timing)
- Based on: Xilem and Taffy research
- Status: In Progress
