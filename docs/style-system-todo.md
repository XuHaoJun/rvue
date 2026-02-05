# Style System Issues - TODO

## Critical Issues

### Issue #1: Dual Property Systems

**Severity**: High
**Impact**: API confusion, maintenance burden
**Status**: CONFIRMED

**Problem**: The codebase has two separate property systems:
- `WidgetProperty` trait (`crates/rvue/src/properties/`) - widget state (TextContent, ShowCondition, etc.)
- `Property` trait (`crates/rvue-style/src/property.rs`) - CSS styles (BackgroundColor, Width, etc.)

**Xilem approach**: Single unified `Property` trait handles both.

**Fix**: Unify into a single `Property` trait.

---

### Issue #2: Context API Regression

**Severity**: High
**Impact**: Blocks common patterns (theme, i18n, dependency injection)
**Status**: CONFIRMED

**Problem**: Current `ContextValueEnum` is severely limited:

```rust
pub enum ContextValueEnum {
    I32(Gc<i32>),
    I64(Gc<i64>),
    F64(Gc<f64>),
    Bool(Gc<bool>),
    GcString(Gc<String>),
    GcVecString(Gc<Vec<String>>),
}
```

**Leptos approach** (truly generic):
```rust
fn provide_context<T: Send + Sync + 'static>(value: T)
fn use_context<T: Clone + 'static>() -> Option<T>
```

**Problems**:
- Only 6 hardcoded types supported
- Unsafe transmute operations in `to_gc()`
- No composability for custom types
- Memory leak risk with `std::mem::forget(gc)`

**Fix**: Adopt `Gc<Owner>`-based context with proper type erasure.

---

### Issue #3: Unsafe Type Casting Pattern

**Severity**: Critical
**Impact**: Memory safety risk
**Status**: CONFIRMED

**Location**: `crates/rvue/src/component.rs:53-85`

```rust
let ptr = Gc::internal_ptr(&gc);
std::mem::forget(gc);
if type_id == TypeId::of::<i32>() {
    let gc_i32: Gc<i32> = unsafe { Gc::from_raw(ptr) };
    // ...
}
```

This pattern is extremely fragile - any type or lifetime mismatch causes undefined behavior.

**Fix**: Replace with safe generic context API (addresses Issue #2).

---

### Issue #4: GC Correctness Risk - No-Op Trace Implementations

**Severity**: Critical
**Impact**: Memory leaks, incorrect collection of GC-managed data
**Status**: CONFIRMED

**Location**: `crates/rvue/src/properties/types.rs:20-349`

Most widget properties have no-op Trace implementations:

```rust
unsafe impl Trace for TextContent {
    fn trace(&self, _visitor: &mut impl Visitor) {}  // Does nothing!
}

unsafe impl Trace for ShowCondition {
    fn trace(&self, _visitor: &mut impl Visitor) {}  // Does nothing!
}
```

These types contain data (like `TextContent(pub String)`) that may hold GC-managed pointers.

**Compare with rvue-style** (`crates/rvue-style/src/property.rs:131-133`):
```rust
unsafe impl Trace for DynProperty {
    fn trace(&self, _visitor: &mut impl Visitor) {}  // Also does nothing!
}
```

**Risk**: If any property eventually contains `Gc<T>`, it won't be properly traced during GC cycles.

**Fix**: Implement actual tracing for all property types that may contain GC pointers.

---

### Issue #5: Context API Returns Gc<T> Instead of Generic T

**Severity**: High
**Impact**: Forces all context values into GC system, prevents non-GC types
**Status**: CONFIRMED

**Location**: `crates/rvue/src/context.rs:17-24`

```rust
pub fn inject<T: Any + Trace + Clone>() -> Option<Gc<T>> {
    current_owner().and_then(|owner| owner.find_context::<T>())
}
```

**Problem**: The API promises generic `T` but returns `Gc<T>`. This:
- Forces all context values into the GC system
- Prevents storing non-GC types in context
- Contradicts Leptos's `use_context<T>` which returns `T` directly

**Contrast with Leptos**:
```rust
fn provide_context<T: Send + Sync + 'static>(value: T)
fn use_context<T: Clone + 'static>() -> Option<T>
```

**Fix**: Return generic `T` instead of `Gc<T>`.

---

## Medium Issues

### Issue #6: Widget Properties Use Strings Instead of Enums

**Severity**: Medium
**Impact**: Poor DX, runtime errors instead of compile-time
**Status**: CONFIRMED

**Location**: `crates/rvue/src/properties/types.rs:167-215`

```rust
pub struct FlexDirection(pub String);
pub struct FlexAlignItems(pub String);
pub struct FlexJustifyContent(pub String);
```

**Fix**: Use enum types:

```rust
pub enum FlexDirection { Row, Column, RowReverse, ColumnReverse }
pub enum FlexAlignItems { Start, End, Center, Stretch, Baseline }
pub enum FlexJustifyContent { Start, End, Center, SpaceBetween, SpaceAround }
```

---

### Issue #7: Dual DynProperty Implementations

**Severity**: High
**Impact**: Code duplication, maintenance burden, different memory strategies
**Status**: CONFIRMED

The codebase has two completely separate `DynProperty` implementations:

**rvue** (`crates/rvue/src/properties/map.rs:32-35`):
```rust
pub struct DynProperty {
    type_id: TypeId,
    value: Box<dyn DynClone>,  // Heap allocation per property
}
```

**rvue-style** (`crates/rvue-style/src/property.rs:27-33`):
```rust
struct DynProperty {
    type_id: TypeId,
    ptr: *mut u8,              // Inline byte storage
    layout: Layout,
    drop_fn: unsafe fn(*mut u8, Layout),
    clone_fn: unsafe fn(*const u8, Layout) -> *mut u8,
}
```

**Problems**:
- Duplicated code and maintenance burden
- Different memory strategies (heap allocation vs inline bytes)
- Both have no-op Trace implementations
- No shared abstraction

**Fix**: Unify into a single DynProperty implementation.

---

### Issue #8: Inconsistent Memory Allocation Strategies

**Severity**: Medium
**Impact**: Code complexity, potential for memory errors
**Status**: CONFIRMED

| Crate | Strategy | Location |
|-------|----------|----------|
| rvue/properties | `Box<dyn DynClone>` | `map.rs:34` |
| rvue-style/property | `alloc::alloc()` manual | `property.rs:43` |
| rvue-style/computed | `Vec<u8>` inline | `computed_styles.rs` |

**Problem**: No unified strategy. The `rvue-style` implementation uses raw pointer arithmetic with manual memory management, while `rvue` uses `Box<dyn>`. Both require `unsafe` code.

**Fix**: Adopt a consistent memory allocation strategy across all crates.

---

### Issue #9: Missing Trait Bounds on DynClone

**Severity**: Medium
**Impact**: Thread safety, API completeness
**Status**: CONFIRMED

**Location**: `crates/rvue/src/properties/map.rs:12-30`

```rust
trait DynClone: Any {
    fn clone_box(&self) -> Box<dyn DynClone>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

**Problem**: `DynClone` doesn't implement `Send + Sync`, but `DynProperty` is used in `Send + Sync` contexts (Component's PropertyMap).

**Fix**: Add `Send + Sync` bounds to `DynClone` and `DynProperty`.

---

### Issue #10: Global Mutable State in Stylesheet

**Severity**: Medium
**Impact**: Thread safety, multiple app instances
**Status**: CONFIRMED

**Location**: `crates/rvue/src/style.rs:21-28`

```rust
impl Stylesheet {
    pub fn with_defaults() -> Self {
        let defaults = default_stylesheet();  // Global state!
        Self { inner: Rc::new(RefCell::new(defaults)) }
    }
}
```

**Problem**: The default stylesheet is a global singleton. Multiple apps or threads would share/modify this state unsafely.

**Fix**: Remove global state, require explicit stylesheet configuration.

---

### Issue #11: String-Based Type Identification for Custom Components

**Severity**: Medium
**Impact**: Runtime errors, no validation, potential typos
**Status**: CONFIRMED

**Location**: `crates/rvue/src/component.rs:207-219`

```rust
pub enum ComponentType {
    // ...
    Custom(String),  // String-based identification
}
```

**Problem**: Custom components use string identifiers instead of type IDs, leading to:
- Runtime errors instead of compile-time
- No validation of custom type names
- Potential typos in custom widget names

**Fix**: Use type IDs instead of strings for component identification.

---

### Issue #14: Memory Leak - `create_memo` Effects Never Cleaned Up

**Severity**: Critical
**Impact**: Memory leak in long-running applications
**Status**: NEW

**Location**: `crates/rvue/src/signal.rs:234-237` and `crates/rvue/src/signal.rs:263-266`

```rust
LEAKED_EFFECTS.with(|cell| {
    let mut leaked = cell.borrow_mut();
    leaked.push(effect);  // Never removed!
});
```

**Problem**: Effects created by `create_memo` and `create_memo_with_equality` are stored in a thread-local `LEAKED_EFFECTS` vector and never cleaned up. This is a memory leak that grows with every memo created.

**Contrast with Leptos**: Leptos memos are properly cleaned up when their owner component is dropped.

**Fix**: Implement proper effect cleanup when memo's owner is dropped, or use a different mechanism for memo tracking.

---

### Issue #15: Incomplete DynProperty Tracing

**Severity**: High
**Impact**: GC correctness for properties
**Status**: NEW

**Location**: `crates/rvue/src/properties/map.rs:53-59`

```rust
fn trace_inner(&self, visitor: &mut impl Visitor) {
    if self.type_id == TypeId::of::<WidgetStyles>() {
        if let Some(styles) = self.downcast::<WidgetStyles>() {
            styles.trace(visitor);
        }
    }
    // All other property types are NOT traced!
}
```

**Problem**: Only `WidgetStyles` is traced. Any other property type containing GC pointers won't be properly traced during GC cycles.

**Note**: This is more specific than Issue #4 (no-op Trace implementations). The problem here is selective tracing - even if types implement `Trace`, the code doesn't call it.

**Fix**: Trace all property types that may contain GC pointers, not just `WidgetStyles`.

---

### Issue #16: Component Clone Creates Inconsistent State

**Severity**: Medium
**Impact**: Potential runtime issues with cloned components
**Status**: NEW

**Location**: `crates/rvue/src/component.rs:277-306`

```rust
impl Clone for Component {
    fn clone(&self) -> Self {
        Self {
            // ...
            parent: GcCell::new(None),  // Parent reset to None
            effects: GcCell::new(self.effects.borrow().clone()),  // Effects cloned
            // ...
            contexts: GcCell::new(Vec::new()),  // Contexts cleared
            // ...
        }
    }
}
```

**Problem**: Cloning a component:
- Resets `parent` to `None`
- Clones `effects` (which reference this component via `owner` field)
- Clears `contexts`

This creates inconsistent state where cloned effects still reference the original component's owner but have no parent.

**Fix**: Either prevent component cloning entirely, or properly rewire all references during clone.

---

### Issue #17: No Cleanup for Effects on Unmount

**Severity**: High
**Impact**: Memory leaks, continued effect execution after unmount
**Status**: NEW

**Location**: `crates/rvue/src/component.rs:1305-1322`

```rust
fn unmount(&self) {
    for child in self.children.borrow().iter() {
        child.unmount();
    }
    // Effects are NOT explicitly cleaned up here
    // Cleanups are run, but effects continue to exist
}
```

**Problem**: When a component is unmounted:
- Effects are not unsubscribed from signals
- Effects remain in GC graph
- Effects may continue running if signals change

**Contrast with Leptos**: Effects are automatically cleaned up when their scope is dropped.

**Fix**: Implement effect cleanup in `unmount()` to unsubscribe from all tracked signals.

---

### Issue #18: LazyView Captured Data Cannot Contain GC Types

**Severity**: High
**Impact**: Limits composition patterns
**Status**: NEW

**Location**: `crates/rvue/src/slot.rs:30-56`

```rust
/// Internal lazy view structure.
/// SAFETY: Closures stored here must NOT capture any GC-tracked types
/// (Gc, GcCell, ReadSignal, WriteSignal, etc.).
pub(crate) struct LazyView {
    closure: Box<dyn Fn(&mut BuildContext) -> ViewStruct>,
}
```

**Problem**: This is a fundamental limitation - slot closures cannot capture signals or other reactive state. This severely limits composition patterns that are common in Leptos/Vue.

**Example of Broken Pattern**:
```rust
// This pattern would NOT work in current rvue:
let count = create_signal(0);
view! { <Slot> { move || view! { <Text> { count.get() } } } }
```

**Fix**: Store closures with `Gc` support, or redesign to allow reactive capture.

---

### Issue #19: Event Handler API Bloat

**Severity**: Medium
**Impact**: API usability, code maintenance
**Status**: NEW

**Location**: `crates/rvue/src/component.rs:756-1131`

**Problem**: 90+ event handler methods with combinatorial explosion:
- `on_click_0arg`, `on_click_1arg`, `on_click`
- `on_pointer_down_0arg`, `on_pointer_down_1arg`, `on_pointer_down`
- etc. for every event type

**Contrast with Xilem**: Uses a single `on` method accepting event type enum.

**Fix**: Consolidate into a single `on<EventType>` method with event type enum.

---

### Issue #20: For Loop Missing Key Prop

**Severity**: Medium
**Impact**: Performance, unnecessary re-renders
**Status**: NEW

**Location**: `crates/rvue/src/widgets/for_loop.rs`

**Problem**: The `For` component doesn't support keyed rendering. This causes unnecessary re-renders when list items change order or are inserted/removed.

**Contrast with Leptos/Vue**: Both support keyed `For` components for efficient list reconciliation.

**Fix**: Add key support to `For` component for efficient list reconciliation.

---

### Issue #21: Missing `create_selector` / Derived Signal Pattern

**Severity**: Medium
**Impact**: Missing functionality
**Status**: NEW

**Location**: `crates/rvue/src/signal.rs`

**Problem**: Only `create_memo` exists for derived signals. Missing:
- `create_selector` (memo with equality check before update)
- Proper memoization with dependency tracking

**Contrast with Leptos**: Has both `create_memo` and `create_selector` with different update semantics.

**Fix**: Implement `create_selector` with equality check semantics.

---

### Issue #22: Thread-Local Owner Stack Not Thread-Safe

**Severity**: Medium
**Impact**: Cannot use async/parallel operations
**Status**: NEW

**Location**: `crates/rvue/src/runtime.rs:5-9`

```rust
thread_local! {
    static OWNER_STACK: RefCell<Vec<Gc<Component>>> = const { RefCell::new(Vec::new()) };
}
```

**Problem**: Components and effects cannot cross thread boundaries. Any async or parallel operation would break the owner/context system.

**Contrast with Leptos**: Supports `Spawn` trait for thread-safe concurrent operations.

**Fix**: Design thread-safe owner/context system for concurrent operations.

---

### Issue #23: Widget Trait Missing Key Prop

**Severity**: Low
**Impact**: Cannot implement keyed rendering at widget level
**Status**: NEW

**Location**: `crates/rvue/src/widget.rs:212-231`

**Problem**: The `Widget` trait doesn't include a `key` field, making it impossible to implement keyed rendering at the widget level.

**Fix**: Add `key` field to Widget trait for keyed rendering support.

---

## Low Issues

### Issue #24: Box<dyn> Allocation Overhead

**Severity**: Low
**Impact**: Performance concern
**Status**: PRESENT (not fixed)

**Current** (`crates/rvue/src/properties/map.rs:34`):
```rust
struct DynProperty {
    type_id: TypeId,
    value: Box<dyn DynClone>,  // Heap allocation per property
}
```

**Xilem approach** (inline bytes):
```rust
struct DynProperty {
    type_id: TypeId,
    value: Vec<u8>,  // Inline storage
    size: usize,
    align: usize,
}
```

**Fix**: Consider inline byte storage to reduce allocations.

---

### Issue #13: Missing PropertyMap::iter() Method

**Severity**: Low
**Impact**: API completeness
**Status**: FIXED

**Location**: `crates/rvue/src/properties/map.rs`

Cannot iterate over PropertyMap to:
- Debug/log all properties
- Apply transformations
- Merge property maps programmatically

**Fix**: Add `iter()` and `iter_mut()` methods.

---

## Priority Order

1. **P0 - Fix immediately**:
   - Issue #3 (unsafe casting) - blocks safe context usage
   - Issue #2 (context API) - depends on #3 fix
   - Issue #4 (GC correctness) - memory safety risk
   - Issue #14 (memo memory leak) - memory leak in long-running apps
   - Issue #15 (incomplete tracing) - GC correctness for properties
   - Issue #17 (no unmount cleanup) - memory leaks, continued effect execution

2. **P1 - High priority**:
   - Issue #1 (unify property systems)
   - Issue #7 (dual DynProperty)
   - Issue #5 (context API returns Gc<T>)
   - Issue #18 (LazyView GC limitation) - limits composition patterns

3. **P2 - Medium priority**:
   - Issue #6 (string to enums)
   - Issue #8 (inconsistent memory strategies)
   - Issue #9 (missing Send + Sync bounds)
   - Issue #10 (global state)
   - Issue #11 (string-based type IDs)
   - Issue #16 (clone inconsistency) - potential runtime issues
   - Issue #19 (API bloat) - code maintenance burden
   - Issue #20 (no keys) - performance with list changes
   - Issue #21 (missing selector) - missing functionality
   - Issue #22 (thread safety) - async/parallel limitations

4. **P3 - Nice to have**:
   - Issue #24 (allocation optimization)
   - Issue #13 (PropertyMap::iter) - already fixed
