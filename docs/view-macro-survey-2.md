# View Macro Survey Part 2: Compile-Time Reactivity & Svelte Architecture Analysis

> **Status**: Draft
> **Focus**: Deep dive into Svelte 5's compile-time reactivity and its application to rvue.

## 1. Introduction

This document expands on the initial survey by providing a technical deep dive into Svelte 5's compiler architecture. The goal is to inform the design of rvue's `view!` macro, specifically focusing on how to achieve **compile-time reactivity optimization** similar to Svelte's Runes system, while adapting it for a Rust/Vello/Taffy environment.

## 2. Svelte 5 Compiler Architecture

Svelte 5 moves towards a "runes" based system (`$state`, `$derived`, `$effect`), which simplifies the compiler's reactivity model while making it more explicit. Our analysis of `packages/svelte/src/compiler` reveals a three-phase process:

### 2.1. Phase 1: Parsing
*(Standard AST generation using Acorn, not the primary focus for rvue which will use `syn`)*

### 2.2. Phase 2: Analysis (`scope.js` & `analyze/index.js`)

This is the "brain" of the compiler. It builds a detailed scope tree and tracks variable usage *before* code generation.

#### Key Data Structures (`scope.js`)
*   **`Scope` Class**: Maintains a map of declarations and references. It handles scope validation (preventing duplicate declarations) and tracks closures (`function_depth`).
*   **`Binding` Class**: Represents a variable declaration. Crucially, it tracks:
    *   `kind`: 'state', 'derived', 'prop', 'normal', etc.
    *   `references`: All usages of the variable in the code.
    *   `assignments`: Where the variable is mutated.
    *   `initial`: The initial value expression (used for constant folding/evaluation).
*   **`Evaluation` Class**: Attempts to statically evaluate expressions. If an expression depends only on constants, Svelte resolves it at compile time.

#### Runes Detection (`analyze/index.js`)
The analyzer walks the AST. When it encounters a `CallExpression`, it checks if it matches a known rune (e.g., `$state`).
*   If found, the variable binding is marked with `kind: 'state'`.
*   This "coloring" of variables propagates: variables derived from state become derived values.

**Relevance to rvue**:
The `view!` macro can scan the Rust tokens to identify "signals" (variables implementing `ReadSignal` or marked with a hypothetical attribute). By tracking these bindings, we can know *at compile time* which parts of the `view!` macro are dynamic and which are static.

### 2.3. Phase 3: Transformation (`transform-client.js`)

This phase generates the runtime code using the analysis data.

#### Client Transformation (`client_component`)
*   **Hoisting**: Static parts of the template are hoisted out of the render function.
*   **Reactive Statements**: converts `$:` (legacy) or `$effect` (runes) into runtime effect calls.
*   **State Proxies**: In Svelte 5, `$state` becomes a `source` (signal), and mutations (`count += 1`) are transformed into `set(count, count + 1)`.

#### Element Transformation (`RegularElement.js`)
This is highly relevant for rvue's widget tree generation.
*   **Static vs. Dynamic Attributes**: The compiler splits attributes into initialized-once (`init`) and updated-often (`update`).
*   **Optimization**:
    *   If an element has *no* dynamic bindings, it is created once (often via `cloneNode` or string injection).
    *   If it has dynamic bindings, fine-grained effect hooks are generated.
*   **Special Cases**: Fast paths for `class` toggling, `style` setting, and `input` value binding.

## 3. Designing rvue's Compiled Reactivity

Based on the Svelte analysis, here is the technical proposal for rvue's `view!` macro.

### 3.1. Static Analysis in a Proc Macro

Unlike Svelte (JS), rvue operates in Rust macros. We cannot fully "analyze" the entire crate, but we can analyze the `view!` block and the variables captured by it.

**Proposed Strategy**:
1.  **Scope Walking**: The macro should parse the inputs and identifying all identifiers.
2.  **Signal Detection**:
    *   *Implicit*: If a variable `count` is used as `move || count.get()`, it's dynamic.
    *   *Explicit*: If the user writes `{count}`, the macro checks if `count` typically behaves like a signal (this is hard in Rust macros without type info).
    *   *Hybrid*: We assume `{expr}` is dynamic *unless* it is a literal or const.
3.  **Inert Detection** (Leptos style):
    *   If a sub-tree of the view layout contains NO dynamic expressions, construct it once or mark it `const`.

### 3.2. Code Generation (The "Transform" Phase)

Instead of generating DOM nodes, we generate `rvue::Widget` builder chains.

#### Svelte Optimization Applied to rvue
Svelte separates `init` (creation) from `update` (reactivity). rvue should do the same in the generated code for a Widget.

**Example Input**:
```rust
view! {
    <div class="box" width={width_sig}>
        <span color="red">"Static"</span>
        <span>{text_sig}</span>
    </div>
}
```

**Generated Code Conceptual Model**:

```rust
// 1. Static Setup (Optimized)
// These properties are known to never change.
let static_span = Text::new("Static").color(Color::Red);

// 2. Dynamic Setup (Reactive)
// We create the widget with initial values but attach effects.
let dynamic_span = Text::new(text_sig.get_untracked());
Effect::new(move |_| {
    // Fine-grained update: only changes the text content
    dynamic_span.set_text(text_sig.get());
});

// 3. Container Assembly
let div = Flex::new()
    .class("box") // Static prop
    .width(width_sig.get_untracked()) // Initial value
    .child(static_span)
    .child(dynamic_span);

// 4. Reactive Bindings for Props
Effect::new(move |_| {
    // Only updates width, "class" is never touched again
    div.set_width(width_sig.get());
});
```

### 3.3. Key Differences from Svelte

1.  **No Virtual DOM / DOM**: We target Vello scene graphs and Taffy layouts. "Patching" means updating a `Gc<Widget>` state which marks the tree dirty for the next frame.
2.  **Rust Type System**: We don't need runtime proxies/interceptors (`source` in Svelte). Rust's explicit `.get()` and `.set()` signals (Solid-like) mean the compiler doesn't need to rewrite variable access, only wrap them in `Effect` closures.

## 4. Implementation Roadmap for rvue

This roadmap is based on the current state of `crates/rvue` and `crates/rvue-macro`.

### Phase 1: Infrastructure & Parsing (`crates/rvue-macro`)

The current `view!` macro in `crates/rvue-macro/src/lib.rs` is a placeholder. We need to implement a real RSX parser.

1.  **Dependencies**: Add `syn-rsx` (or custom `syn` parsing) to `rvue-macro/Cargo.toml`.
2.  **AST Definition**: Create `src/rsx.rs` in `rvue-macro` to define the RSX AST (Tags, Attributes, Expressions).
3.  **Parser Implementation**: Implement `Parse` for the AST to handle:
    *   Elements: `<Tag ...>`
    *   Attributes: `key="value"` (static) vs `key={expr}` (dynamic)
    *   Children: Nested tags and text.
    *   Interpolation: `{ expr }` in text positions.

### Phase 2: Widget System Refactor (`crates/rvue`)

The current `ComponentProps` enum in `crates/rvue/src/component.rs` is a monolithic bottleneck that prevents fine-grained updates.

1.  **Define `Widget` Trait**: In `src/widget.rs`, define a trait that replaces `ComponentProps`.
    ```rust
    pub trait Widget: Trace {
        fn build(self, ctx: &mut BuildContext) -> Gc<Component>;
        fn update(&self, component: &Gc<Component>);
    }
    ```
2.  **Refactor Widgets**: Convert `src/widgets/*.rs` (e.g., `Text`, `Button`) from simple wrappers around `ComponentProps` to struct builders that implement `Widget`.
    *   *Example*: `Text` widget should store `Signal<String>` or `String` directly, not wrap it in a generic enum.
3.  **Deprecate `ComponentProps`**: Gradually remove variantes from `ComponentProps` as they are ported to the new `Widget` trait system.

### Phase 3: Macro Expansion & Reactivity (`crates/rvue-macro`)

Implement the code generation logic to transform the RSX AST into the optimized builder pattern proposed in Section 3.2.

1.  **Expression Classification**: Implement a visitor to scan `syn::Expr` blocks within the RSX.
    *   Identify variables that match known signal patterns (e.g., `.get()`).
2.  **Builder Generation**:
    *   Transform `<Text value="Hello" />` -> `Text::new().value("Hello").build(...)`
    *   Transform `<Div class="box">` -> `Div::new().class("box")...`
3.  **Effect Injection**:
    *   For dynamic attributes (`<Text value={signal} />`), generate:
        ```rust
        let widget = Text::new();
        Effect::new(move |_| {
            widget.set_value(signal.get());
        });
        ```
    *   This requires adding specific, efficient setters to the Widget implementations in `crates/rvue`.

### Phase 4: Integration & Cleanup

1.  **Update `View` Trait**: Ensure `crates/rvue/src/view.rs` works seamlessly with the new Widget builders.
2.  **Migration**: Update `crates/rvue-examples` to use the new macro syntax and verify fine-grained updates (using logging or debugger).
3.  **Performance Check**: Verify that `mark_dirty` is called only for the specific leaf nodes changing, not the entire component tree.

## 5. Summary

Svelte 5 validates the concept that **static analysis is king**. By knowing what changes and what stays the same, we avoid the runtime overhead of diffing. For rvue, this confirms that our `view!` macro should not just be syntactic sugar for function calls, but an optimizing compiler that separates static layout (layout engine) from dynamic updates (reactivity graph).
