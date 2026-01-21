# View Macro Survey for rvue

> **Parallel World Collaboration**: Alex Crichton, Leptos Team, 尤雨溪 (Evan You), Ryan Carniato
>
> A comprehensive analysis of `view!` macro design patterns from Leptos and Svelte,
> with integration recommendations for rvue's hybrid GC + Vello + Taffy + Solid.js-like reactivity architecture.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Leptos View Macro Analysis](#leptos-view-macro-analysis)
3. [Svelte Compiler Analysis](#svelte-compiler-analysis)
4. [rvue Current State](#rvue-current-state)
5. [Design Recommendations](#design-recommendations)
6. [Implementation Plan](#implementation-plan)
7. [API Examples](#api-examples)

---

## Executive Summary

### Core Design Goals for rvue `view!` Macro

| Goal | Description |
|------|-------------|
| **Fine-Grained Reactivity** | SolidJS-style compile-time dependency extraction |
| **GC Integration** | Seamless integration with `rudo-gc` (easy-oilpan) |
| **Desktop-First** | Optimized for Vello + Taffy, not DOM/HTML |
| **Ergonomic DX** | Vue/Leptos-inspired JSX-like syntax |
| **Type Safety** | Full Rust type inference and IDE support |

### Key Insights from Research

1. **Leptos** uses `rstml` for RSX parsing, generates builder-pattern code with runtime reactivity
2. **Svelte** performs multi-phase compilation (parse → analyze → transform) with static scope analysis
3. **rvue** should combine compile-time analysis (Svelte-style) with Rust's type system (Leptos-style)

---

## Leptos View Macro Analysis

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     leptos_macro                                 │
│  ┌─────────────┐    ┌──────────────┐    ┌────────────────────┐ │
│  │   rstml     │───▶│  view/mod.rs │───▶│  TokenStream       │ │
│  │  (Parser)   │    │  (Transform) │    │  (Builder Pattern) │ │
│  └─────────────┘    └──────────────┘    └────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        tachys                                    │
│  ┌────────────┐    ┌────────────┐    ┌────────────────────────┐ │
│  │   view/    │    │   html/    │    │   reactive_graph/      │ │
│  │  (Traits)  │    │ (Elements) │    │   (Signal Runtime)     │ │
│  └────────────┘    └────────────┘    └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

#### 1. RSX Parsing (`rstml`)

Leptos uses the `rstml` crate for parsing RSX syntax:

```rust
// Input
view! {
    <div class="container">
        <Button on:click=move |_| count.update(|n| *n += 1)>
            "Click: " {count}
        </Button>
    </div>
}
```

#### 2. Node Types (from `leptos_macro/src/view/mod.rs`)

```rust
pub(crate) enum TagType {
    Unknown,
    Html,
    Svg,
    Math,
}

enum Item<'a, T> {
    Node(&'a Node<T>, bool),
    ClosingTag(String),
}
```

#### 3. Inert Element Optimization

Leptos identifies "inert" (purely static) elements at compile time and optimizes them:

```rust
fn is_inert_element(orig_node: &Node<impl CustomNode>) -> bool {
    // Check if all attributes are static string literals
    // Check if all children are static text
    // If so, can be compiled to a static string at compile time
}
```

**Key Insight**: Static parts are compiled to `&'static str`, dynamic parts generate `create_effect()` calls.

#### 4. Render Trait System (`tachys/src/view/mod.rs`)

```rust
/// The `Render` trait allows rendering something as part of the user interface.
pub trait Render: Sized {
    /// The "view state" for this type, which can be retained between updates.
    type State: Mountable;

    /// Creates the view for the first time, without hydrating from existing HTML.
    fn build(self) -> Self::State;

    /// Updates the view with new data.
    fn rebuild(self, state: &mut Self::State);
}

/// The `RenderHtml` trait allows rendering something to HTML, and transforming
/// that HTML into an interactive interface (hydration).
pub trait RenderHtml: Render + AddAnyAttr + Send {
    type AsyncOutput: RenderHtml;
    type Owned: RenderHtml + 'static;
    const MIN_LENGTH: usize;
    const EXISTS: bool = true;
    // ...
}

/// Allows a type to be mounted to the DOM.
pub trait Mountable {
    fn unmount(&mut self);
    fn mount(&mut self, parent: &Element, marker: Option<&Node>);
    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool;
    fn elements(&self) -> Vec<Element>;
}
```

#### 5. Attribute System (`tachys/src/html/attribute/mod.rs`)

```rust
/// Defines an attribute: anything that can modify an element.
pub trait Attribute: NextAttribute + Send {
    const MIN_LENGTH: usize;
    type State;
    type AsyncOutput: Attribute;
    type Cloneable: Attribute + Clone;
    type CloneableOwned: Attribute + Clone + 'static;

    fn html_len(&self) -> usize;
    fn to_html(self, buf: &mut String, class: &mut String, style: &mut String, inner_html: &mut String);
    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State;
    fn build(self, el: &Element) -> Self::State;
    fn rebuild(self, state: &mut Self::State);
    // ...
}

/// An attribute with a key and value.
pub struct Attr<K, V>(pub K, pub V)
where
    K: AttributeKey,
    V: AttributeValue;
```

### Leptos Reactive Graph (`reactive_graph`)

Key design principles:

1. **Signals as atomic state** - `ArcRwSignal<T>` with Copy + 'static identifiers
2. **Memos as derived values** - `ArcMemo<T>` for computed values
3. **Effects as side effects** - Run when dependencies change
4. **Automatic dependency tracking** - Runtime subscription model
5. **Asynchronous effect scheduling** - Effects are async tasks

```rust
use reactive_graph::{
    computed::ArcMemo,
    effect::Effect,
    prelude::{Read, Set},
    signal::ArcRwSignal,
};

let count = ArcRwSignal::new(1);
let double_count = ArcMemo::new({
    let count = count.clone();
    move |_| *count.read() * 2
});

Effect::new(move |_| {
    println!("double_count = {}", *double_count.read());
});

count.set(2); // Triggers effect
```

---

## Svelte Compiler Analysis

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Svelte Compiler                               │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  1-parse     │─▶│  2-analyze   │─▶│  3-transform         │  │
│  │  (AST)       │  │  (Scopes)    │  │  (JS Output)         │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 1: Parsing

Svelte parses `.svelte` files into an AST representation. The parser handles:
- HTML template syntax
- `{expression}` interpolation
- Directives (`on:`, `bind:`, `class:`, `style:`)
- Control flow (`{#if}`, `{#each}`, `{#await}`)

### Phase 2: Analysis (Scope Analysis)

The critical phase for compile-time reactivity. Key structures from `scope.js`:

```javascript
export class Binding {
    scope;           // Reference to containing scope
    node;            // AST Identifier node
    kind;            // 'prop' | 'state' | 'derived' | etc.
    declaration_kind;// 'var' | 'let' | 'const' | 'import'
    initial = null;  // Initial value expression
    references = []; // All references to this binding
    assignments = []; // All (re)assignments
    legacy_dependencies = []; // Reactive dependencies
    mutated = false;
    reassigned = false;
}

export class Scope {
    root;           // ScopeRoot reference
    parent;         // Parent scope (or null)
    declarations;   // Map<string, Binding>
    references;     // Map<string, reference[]>
    function_depth; // Depth for closure analysis
    
    // Key methods:
    declare(node, kind, declaration_kind, initial);
    get(name);      // Look up binding by name
    evaluate(expression); // Partial evaluation
}
```

#### Runes Detection

Svelte 5 uses "runes" (`$state`, `$derived`, `$effect`) detected at analysis time:

```javascript
case '$state':
case '$state.raw':
case '$derived':
    if (arg) {
        scope.evaluate(arg, this.values);
    } else {
        this.values.add(undefined);
    }
    break;
```

### Phase 3: Transform

Generates optimized JavaScript with fine-grained updates:

```javascript
// Input
let count = $state(0);
let doubled = $derived(count * 2);

// Output (simplified)
let count = source(0);
let doubled = derived(() => get(count) * 2);
```

### Svelte Reactivity Runtime (`internal/client/reactivity/sources.js`)

```javascript
export function source(v, stack) {
    var signal = {
        f: 0,           // flags
        v,              // value
        reactions: null,// subscriber list
        equals,         // equality function
        rv: 0,          // read version
        wv: 0           // write version
    };
    return signal;
}

export function set(source, value, should_proxy = false) {
    // Validation: prevent mutation inside deriveds/effects
    if (active_reaction !== null && is_runes() && ...) {
        e.state_unsafe_mutation();
    }
    
    let new_value = should_proxy ? proxy(value) : value;
    return internal_set(source, new_value);
}

export function internal_set(source, value) {
    if (!source.equals(value)) {
        var old_value = source.v;
        old_values.set(source, old_value);
        source.v = value;
        
        var batch = Batch.ensure();
        batch.capture(source, old_value);
        
        source.wv = increment_write_version();
        mark_reactions(source, DIRTY);
        
        if (!batch.is_fork && eager_effects.size > 0) {
            flush_eager_effects();
        }
    }
    return value;
}
```

### Key Svelte Insights for rvue

1. **Static Analysis First**: Svelte knows at compile time which variables are reactive
2. **Binding Classification**: Different handling for props, state, derived, etc.
3. **Scope Walking**: Full scope chain for dependency resolution
4. **Batched Updates**: Changes are batched before effects run
5. **Version Tracking**: Read/write versions for dirty checking

---

## rvue Current State

### Existing Component Structure (`component.rs`)

```rust
pub struct Component {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub children: GcCell<Vec<Gc<Component>>>,
    pub parent: GcCell<Option<Gc<Component>>>,
    pub effects: GcCell<Vec<Gc<Effect>>>,
    pub props: GcCell<ComponentProps>,
    pub is_dirty: AtomicBool,
    pub layout_node: GcCell<Option<LayoutNode>>,
    pub event_handlers: GcCell<EventHandlers>,
    // ...
}

pub enum ComponentType {
    Text, Button, TextInput, NumberInput,
    Checkbox, Radio, Show, For, Flex, Custom(String),
}

pub enum ComponentProps {
    Text { content: String, font_size: Option<f32>, color: Option<Color> },
    Button { label: String },
    Flex { direction: String, gap: f32, align_items: String, justify_content: String },
    // ...
}
```

### Existing Signal System (`signal.rs`)

```rust
pub struct SignalData<T: Trace + Clone + 'static> {
    value: GcCell<T>,
    version: AtomicU64,
    subscribers: GcCell<Vec<Gc<Effect>>>,
}

pub struct ReadSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

pub struct WriteSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

pub fn create_signal<T: Trace + Clone + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let data = Gc::new(SignalData {
        value: GcCell::new(initial_value),
        version: AtomicU64::new(0),
        subscribers: GcCell::new(Vec::new()),
    });
    (ReadSignal { data: Gc::clone(&data) }, WriteSignal { data })
}
```

### Current View System (`view.rs`)

```rust
pub trait View {
    fn into_component(self) -> Gc<Component>;
}

pub struct ViewStruct {
    pub root_component: Gc<Component>,
    pub effects: Vec<Gc<Effect>>,
}
```

### Gap Analysis

| Feature | Leptos | Svelte | rvue Current | Needed |
|---------|--------|--------|--------------|--------|
| DSL Parsing | rstml (RSX) | Custom parser | None | `syn`-based or custom |
| Static Analysis | Partial (inert detection) | Full scope analysis | None | Macro-time analysis |
| Render Target | DOM | DOM | Vello/Taffy | Widget trait system |
| GC Integration | None (arenas) | None (JS GC) | rudo-gc | ✓ Already integrated |
| Event System | DOM events | DOM events | Custom | ✓ Already implemented |

---

## Design Recommendations

### 1. Macro Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     rvue_macro                                   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Parser      │─▶│  Analyzer    │─▶│  Code Generator      │  │
│  │  (syn/rstml) │  │  (Static)    │  │  (TokenStream)       │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        rvue                                      │
│  ┌────────────┐    ┌────────────┐    ┌────────────────────────┐ │
│  │  Widget    │    │  Signal    │    │   Component            │ │
│  │  Traits    │    │  System    │    │   (Gc-managed)         │ │
│  └────────────┘    └────────────┘    └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Widget Trait System (Leptos-Inspired)

```rust
/// Core trait for rvue widgets
pub trait Widget: Trace {
    /// State retained between updates
    type State: WidgetState;
    
    /// Build initial widget state
    fn build(self) -> Gc<Self::State>;
    
    /// Rebuild with new data (fine-grained updates)
    fn rebuild(self, state: &mut Self::State);
}

/// State that can be mounted to the render tree
pub trait WidgetState: Trace {
    /// Layout node for Taffy
    fn layout_node(&self) -> Option<LayoutNode>;
    
    /// Render to Vello scene
    fn render(&self, scene: &mut Scene, layout: &Layout);
    
    /// Cleanup on unmount
    fn unmount(&mut self);
}

/// Widget that can have children
pub trait ContainerWidget: Widget {
    type Children: WidgetChildren;
    
    fn children(&self) -> &Self::Children;
}
```

### 3. Static Analysis Strategy (Svelte-Inspired)

The macro should classify expressions at compile time:

```rust
enum ExpressionKind {
    /// Purely static, can be inlined
    Static(LitExpr),
    
    /// Signal read - generates effect binding
    SignalRead { signal: Ident },
    
    /// Derived computation - generates memo
    Derived { deps: Vec<Ident>, expr: Expr },
    
    /// Dynamic but non-reactive (e.g., props)
    Dynamic(Expr),
}
```

### 4. Attribute Handling

```rust
/// Attribute types for rvue widgets
pub trait WidgetAttribute: Send {
    type State;
    
    /// Apply attribute to widget during build
    fn apply(self, widget: &mut impl Widget) -> Self::State;
    
    /// Update attribute value
    fn update(self, state: &mut Self::State);
}

/// Reactive attribute wrapper
pub struct ReactiveAttr<T, F>
where
    T: WidgetAttribute,
    F: Fn() -> T + 'static,
{
    getter: F,
    effect: Option<Gc<Effect>>,
}
```

### 5. Event Handler Design

Leverage existing event system with macro syntax:

```rust
// Macro syntax
view! {
    <Button on_click=move |_| count.update(|n| *n + 1)>
        "Increment"
    </Button>
}

// Expands to
{
    let button = Component::new(id, ComponentType::Button, ComponentProps::Button {
        label: "Increment".to_string(),
    });
    button.on_click(move |event, ctx| {
        count.update(|n| *n + 1);
    });
    button
}
```

---

## Implementation Plan

### Phase 1: Core Infrastructure

1. Create `rvue_macro` crate with basic setup
2. Implement RSX parser using `syn` + custom DSL
3. Define core `Widget` traits

### Phase 2: Static Analysis

1. Implement expression classifier
2. Build scope analysis for reactive bindings
3. Generate optimized code for static elements

### Phase 3: Widget Library

1. `Text` widget with reactive content
2. `Flex` container with Taffy integration
3. `Button` with event handling

### Phase 4: Advanced Features

1. Control flow (`Show`, `For`)
2. Component composition
3. Hot reload support (optional)

---

## API Examples

### Counter Example

```rust
use rvue::prelude::*;

#[component]
fn Counter() -> impl Widget {
    let (count, set_count) = create_signal(0);
    
    view! {
        <Flex direction="column" gap=10.0>
            <Text font_size=24.0>
                {move || format!("Count: {}", count.get())}
            </Text>
            <Flex direction="row" gap=8.0>
                <Button on_click=move |_| set_count.update(|n| *n -= 1)>
                    "-"
                </Button>
                <Button on_click=move |_| set_count.update(|n| *n += 1)>
                    "+"
                </Button>
            </Flex>
        </Flex>
    }
}

fn main() {
    App::new()
        .title("Counter")
        .size(400.0, 300.0)
        .run(Counter);
}
```

### Conditional Rendering

```rust
view! {
    <Show when=move || count.get() > 0>
        <Text color=Color::GREEN>
            "Positive!"
        </Text>
    </Show>
}
```

### List Rendering

```rust
view! {
    <For each=items key=|item| item.id>
        {|item| view! {
            <Text>{item.name}</Text>
        }}
    </For>
}
```

### Nested Components

```rust
#[component]
fn TodoItem(text: String, completed: ReadSignal<bool>) -> impl Widget {
    view! {
        <Flex direction="row" align_items="center" gap=8.0>
            <Checkbox checked=completed />
            <Text
                style:text_decoration=move || {
                    if completed.get() { "line-through" } else { "none" }
                }
            >
                {text}
            </Text>
        </Flex>
    }
}
```

---

## Comparison Matrix

| Feature | Leptos | Svelte | rvue (Proposed) |
|---------|--------|--------|-----------------|
| **Syntax** | RSX (JSX-like) | HTML + braces | RSX (JSX-like) |
| **Parsing** | `rstml` crate | Custom parser | `syn` + custom |
| **Static Analysis** | Limited (inert detection) | Full (scopes) | Full (Svelte-style) |
| **Reactivity** | Runtime (signals) | Compile-time (runes) | Hybrid |
| **Memory Model** | Arenas + RAII | JS GC | `rudo-gc` (hybrid GC) |
| **Render Target** | DOM | DOM | Vello + Taffy |
| **Type Safety** | Full Rust | Limited (JS) | Full Rust |
| **Build State** | `Mountable` trait | Generated JS | `WidgetState` trait |
| **Updates** | `rebuild()` method | Compiler-generated | Fine-grained effects |

---

## Conclusion

The rvue `view!` macro should:

1. **Adopt Leptos's RSX syntax** - Familiar, ergonomic, well-supported by tooling
2. **Implement Svelte's static analysis** - Compile-time dependency detection for optimal updates
3. **Leverage Rust's type system** - Full inference, IDE support, compile-time safety
4. **Integrate with rudo-gc** - No lifetime annotations, automatic memory management
5. **Target Vello/Taffy** - Desktop-first with GPU-accelerated rendering

This hybrid approach combines the best of both frameworks while leveraging rvue's unique
advantages: a hybrid GC for memory management and Vello/Taffy for high-performance desktop rendering.

---

## References

- Leptos Source: `learn-projects/leptos/`
  - `leptos_macro/src/view/mod.rs` - View macro implementation
  - `tachys/src/view/mod.rs` - Core render traits
  - `reactive_graph/src/lib.rs` - Reactive primitives
- Svelte Source: `learn-projects/svelte/`
  - `packages/svelte/src/compiler/phases/scope.js` - Scope analysis
  - `packages/svelte/src/internal/client/reactivity/sources.js` - Signal runtime
- rvue Source: `crates/rvue/src/`
  - `component.rs` - Current component system
  - `signal.rs` - Current signal implementation
  - `view.rs` - Current view traits
- Core Design: `docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md`
