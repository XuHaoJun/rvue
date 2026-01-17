# Data Model: Rvue MVP Framework

**Date**: 2026-01-17  
**Purpose**: Define core entities, relationships, and data structures for Rvue framework

## Core Entities

### Signal<T>

**Purpose**: Reactive data container that notifies dependents when value changes.

**Fields**:
- `value: T` - Current value
- `subscribers: Vec<Gc<Effect>>` - List of effects that depend on this signal
- `version: u64` - Version counter for change detection

**Relationships**:
- One-to-many with `Effect` (signals can have multiple subscribers)
- Many-to-one with `Component` (components can own multiple signals)

**Lifecycle**:
- Created via `create_signal(initial_value)`
- Automatically cleaned up by GC when no longer referenced
- Version increments on each `.set()` or `.update()` call

**Constraints**:
- `T` must implement `Trace` if it contains `Gc` pointers
- Signals are `Copy` for ergonomic usage in closures
- Signal updates are synchronous (immediate propagation)

### Effect

**Purpose**: Reactive computation that re-runs when dependencies change.

**Fields**:
- `closure: Box<dyn Fn()>` - The effect function to execute
- `dependencies: Vec<Gc<Signal<dyn Any>>>` - Signals this effect depends on
- `is_dirty: bool` - Whether effect needs to run
- `component: Option<Gc<Component>>` - Associated component (if any)

**Relationships**:
- Many-to-many with `Signal` (effects can depend on multiple signals)
- Many-to-one with `Component` (components can have multiple effects)

**Lifecycle**:
- Created via `create_effect(closure)`
- Automatically tracks dependencies on first run
- Re-runs when any dependency changes
- Cleaned up by GC when component unmounts or effect is dropped

**Constraints**:
- Effects must not have side effects that break reactivity (no circular dependencies)
- Effects run synchronously in dependency order

### Component

**Purpose**: Reusable UI building block that encapsulates state, logic, and presentation.

**Fields**:
- `id: ComponentId` - Unique identifier
- `component_type: ComponentType` - Type of component (Text, Button, Custom, etc.)
- `children: Vec<Gc<Component>>` - Child components
- `parent: Option<Gc<Component>>` - Parent component (bidirectional reference)
- `signals: Vec<Gc<Signal<dyn Any>>>` - Signals owned by this component
- `effects: Vec<Gc<Effect>>` - Effects owned by this component
- `layout_node: Option<TaffyNodeId>` - Associated Taffy layout node
- `vello_fragment: Option<VelloFragmentId>` - Associated Vello rendering fragment
- `style: Style` - Component styling
- `props: ComponentProps` - Component properties (type depends on component_type)

**Relationships**:
- One-to-many with `Component` (parent-child tree structure)
- One-to-many with `Signal` (components own signals)
- One-to-many with `Effect` (components own effects)
- One-to-one with `TaffyNode` (layout node)
- One-to-one with `VelloFragment` (rendering fragment)

**Lifecycle**:
- Created when component function is called
- Mounted when added to component tree
- Updated when signals change (via effects)
- Unmounted when removed from tree
- GC automatically cleans up when no longer referenced

**Constraints**:
- Component tree must be acyclic (enforced by type system)
- All `Gc` references form valid object graph for GC tracing

### View

**Purpose**: Declarative UI structure returned from component functions.

**Fields**:
- `root_component: Gc<Component>` - Root component of the view tree
- `signals: Vec<Gc<Signal<dyn Any>>>` - Top-level signals
- `effects: Vec<Gc<Effect>>` - Top-level effects

**Relationships**:
- One-to-one with `Component` (root component)
- One-to-many with `Signal` (view-level signals)
- One-to-many with `Effect` (view-level effects)

**Lifecycle**:
- Created when component function returns
- Mounted to application root
- Updated reactively via signal changes
- Unmounted when application closes

### LayoutNode

**Purpose**: Wrapper around Taffy node for layout calculations.

**Fields**:
- `taffy_id: TaffyNodeId` - Taffy's internal node ID
- `component: Gc<Component>` - Associated component
- `style: taffy::Style` - Taffy style properties
- `layout: Option<taffy::Layout>` - Calculated layout (None if dirty)
- `is_dirty: bool` - Whether layout needs recalculation

**Relationships**:
- One-to-one with `Component` (each component has one layout node)
- Many-to-one with `LayoutNode` (parent-child layout tree)

**Lifecycle**:
- Created when component is mounted
- Updated when style or content size changes
- Deleted when component is unmounted

**Constraints**:
- Layout nodes form tree structure matching component tree
- Dirty marking propagates up to root

### VelloFragment

**Purpose**: Rendering fragment in Vello scene graph.

**Fields**:
- `fragment_id: VelloFragmentId` - Unique fragment identifier
- `component: Gc<Component>` - Associated component
- `scene_items: Vec<vello::SceneItem>` - Vello scene items for this fragment
- `transform: Transform` - Position and transformation
- `z_index: i32` - Rendering order

**Relationships**:
- One-to-one with `Component` (each component has one fragment)
- Many-to-one with `VelloFragment` (parent-child in scene graph)

**Lifecycle**:
- Created when component is mounted
- Updated when component properties change
- Deleted when component is unmounted

**Constraints**:
- Fragments form tree structure matching component tree
- Transform updates trigger scene graph rebuild

### Style

**Purpose**: Component styling properties.

**Fields**:
- `color: Option<Color>` - Text/foreground color
- `background_color: Option<Color>` - Background color
- `font_size: Option<f32>` - Font size in pixels
- `font_weight: Option<FontWeight>` - Font weight
- `font_family: Option<String>` - Font family name
- `padding: Option<Spacing>` - Padding (top, right, bottom, left)
- `margin: Option<Spacing>` - Margin
- `border: Option<Border>` - Border properties
- `border_radius: Option<f32>` - Border radius
- `width: Option<Size>` - Width constraint
- `height: Option<Size>` - Height constraint
- `flex_direction: Option<FlexDirection>` - Flexbox direction
- `flex_grow: Option<f32>` - Flex grow factor
- `flex_shrink: Option<f32>` - Flex shrink factor
- `align_items: Option<AlignItems>` - Flexbox alignment
- `justify_content: Option<JustifyContent>` - Flexbox justification
- `gap: Option<f32>` - Gap between flex items

**Relationships**:
- One-to-one with `Component` (each component has one style)

**Constraints**:
- Style properties can be static values or signal-based (reactive)
- Layout properties (flex, width, height) map to Taffy `Style`

### ComponentProps

**Purpose**: Component-specific properties (variant type).

**Variants**:
- `TextProps { content: Signal<String> }`
- `ButtonProps { label: String, on_click: Box<dyn Fn()> }`
- `TextInputProps { value: Signal<String>, on_input: Box<dyn Fn(String)> }`
- `CheckboxProps { checked: Signal<bool>, on_change: Box<dyn Fn(bool)> }`
- `RadioProps { value: String, checked: Signal<String>, on_change: Box<dyn Fn(String)> }`
- `ShowProps { when: Signal<bool>, children: Gc<Component> }`
- `ForProps<T> { each: Signal<Vec<T>>, key: Box<dyn Fn(&T) -> String>, children: Box<dyn Fn(&T) -> Gc<Component>> }`
- `CustomProps { data: Box<dyn Any> }` - For user-defined components

**Relationships**:
- One-to-one with `Component` (each component has props matching its type)

**Constraints**:
- Props are validated at component creation time
- Signal-based props automatically create effects for updates

## Entity Relationships Diagram

```
Application
  └── View
       └── Component (root)
            ├── signals: Vec<Signal<T>>
            ├── effects: Vec<Effect>
            ├── children: Vec<Component>
            ├── layout_node: LayoutNode
            └── vello_fragment: VelloFragment

Signal<T>
  └── subscribers: Vec<Effect>

Effect
  ├── dependencies: Vec<Signal>
  └── component: Option<Component>

Component
  ├── parent: Option<Component>
  ├── children: Vec<Component>
  ├── signals: Vec<Signal>
  ├── effects: Vec<Effect>
  ├── layout_node: LayoutNode
  └── vello_fragment: VelloFragment
```

## State Transitions

### Component Lifecycle

```
[Uninitialized]
    │
    ├─ create_component() ──→ [Created]
    │                            │
    │                            ├─ mount() ──→ [Mounted]
    │                            │                 │
    │                            │                 ├─ signal.update() ──→ [Dirty]
    │                            │                 │                        │
    │                            │                 │                        └─ update() ──→ [Mounted]
    │                            │                 │
    │                            │                 └─ unmount() ──→ [Unmounted]
    │                            │
    │                            └─ drop() ──→ [Dropped] ──→ [GC Collected]
```

### Signal Update Flow

```
[Signal.set(value)]
    │
    ├─ update internal value
    ├─ increment version
    │
    └─ notify subscribers
         │
         ├─ for each Effect in subscribers:
         │     │
         │     ├─ mark Effect as dirty
         │     └─ schedule Effect.run()
         │
         └─ Effect.run()
              │
              ├─ execute closure
              ├─ track new dependencies
              └─ update Vello scene / layout
```

## Validation Rules

### Component Tree
- Must be acyclic (enforced by Rust type system)
- Root component must be mounted before children
- Unmounting parent automatically unmounts children

### Signals
- Signal values must be `Trace` if they contain `Gc` pointers
- Signal updates must not create circular dependencies in effects
- Signal version must increment on every `.set()` or `.update()`

### Effects
- Effects must not mutate their dependencies directly (use `.update()` instead)
- Effects must be deterministic (same inputs → same outputs)
- Effects must complete within frame budget (16ms)

### Layout
- Layout nodes must form valid tree (matching component tree)
- Dirty marking must propagate upward to root
- Layout calculation must complete before rendering

### Rendering
- Vello fragments must match component tree structure
- Transform updates must be applied before scene encoding
- Scene encoding must complete within frame budget

## Data Volume Assumptions

**MVP Scope**:
- Typical application: 100-1000 components
- Typical signal count: 10-100 signals per application
- Typical effect count: 20-200 effects per application
- Layout nodes: 1 per component
- Vello fragments: 1 per component

**Performance Targets**:
- Component tree creation: <100ms for 1000 components
- Signal update propagation: <1ms for typical dependency graph
- Layout calculation: <5ms for 1000 nodes
- Scene encoding: <10ms for 1000 fragments
- Total frame time: <16ms (60fps)

## Memory Management

**GC-Managed Objects**:
- All `Gc<T>` allocations (components, signals, effects)
- Component tree structure
- Signal dependency graph

**Non-GC Objects**:
- Taffy nodes (managed by Taffy internally)
- Vello scene items (plain Rust data, `Send`)
- Style structs (can be `Gc`-wrapped if needed for reactivity)

**Memory Budget**:
- Initial allocation: <100MB for simple application
- GC heap growth: Managed by generational GC
- Peak memory: Application-dependent, but GC prevents unbounded growth
