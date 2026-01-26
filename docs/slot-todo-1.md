# Slot Implementation - Post-MVP Roadmap

## Current State (MVP)

The slot system works for basic use cases:
- Default slot via `children: ChildrenFn`
- Optional slots via `MaybeChildren`
- Proper GC tracing via `ChildrenClosure`

## MVP Limitations

### 1. Slot Props (Medium Priority)

**Missing:** Parent-to-slot data flow (like Vue's scoped slots).

```rust
// Desired API (not yet supported)
#[slot(prop(count: i32))]
struct ListSlot {
    children: ChildrenFn,
}

// Usage:
<ListView items={items} slot:item=|item| {
    Text { content: format!("Item: {}", item) }
}>
```

**Implementation approach:**
- Extend `#[slot]` macro to parse `#[slot(prop(name: Type))]` attributes
- Generate prop struct, pass to closure at render time

### 2. TypedBuilder Derive (Low Priority)

**Missing:** Builder pattern for optional props.

**Current:**
```rust
CardBody { children: my_children }
```

**Desired:**
```rust
CardBody::builder().children(my_children).optional_prop(value).build()
```

**Implementation:** Derive `typed_builder` on slot structs.

### 3. Dynamic Slot Names (Low Priority)

**Missing:** Runtime-determined slot names.

**Current:** Only static slot names at compile time.

**Implementation:** Track slot names in component metadata, render by lookup.

## Priority Order

| Priority | Feature | Effort | Value |
|----------|---------|--------|-------|
| P1 | Slot props | Medium | High |
| P2 | TypedBuilder | Low | Medium |
| P3 | Dynamic slots | High | Low |

## P1: Slot Props - Implementation Sketch

```rust
// Current macro at rvue-macro/src/slot.rs

// Extend to detect #[slot(prop(...))] attributes
struct SlotProp {
    name: Ident,
    ty: Type,
}

// Generate:
// 1. Prop struct with fields
// 2. Pass props to closure: children(props)
// 3. Closure type: Box<dyn Fn(Props) -> ViewStruct>
```

## Testing Requirements

- Unit tests for prop passing
- Integration test with `#[component]` + `#[slot]` combination
- GC stress test with nested slot closures

## References

- Leptos slot implementation: `learn-projects/leptos/leptos_macro/src/slot.rs`
- Rvue slot runtime: `crates/rvue/src/slot.rs`
- Code generation: `crates/rvue-macro/src/codegen.rs`
