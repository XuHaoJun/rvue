# Rvue Async Implementation Plan v1

> **Status:** Draft  
> **Created:** 2026-02-07  
> **Based on:** Technical review of `async-best-practices.md` and analysis of `crates/` implementation

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Design Goals](#2-design-goals)
3. [Architecture Overview](#3-architecture-overview)
4. [Phase 1: Foundation](#phase-1-foundation)
5. [Phase 2: Task Management](#phase-2-task-management)
6. [Phase 3: Reactive Integration](#phase-3-reactive-integration)
7. [Phase 4: Higher-Level APIs](#phase-4-higher-level-apis)
8. [Phase 5: Testing & Documentation](#phase-5-testing--documentation)
9. [API Reference](#api-reference)
10. [Migration Guide](#migration-guide)
11. [Open Questions](#open-questions)

---

## 1. Current State Analysis

### 1.1 What Exists in rudo-gc

| Component | File Location | Status |
|-----------|---------------|--------|
| `GcRootSet` | `rudo-gc/src/tokio/root.rs` | ✅ Complete |
| `GcRootGuard` | `rudo-gc/src/tokio/guard.rs` | ✅ Complete |
| `GcTokioExt` trait | `rudo-gc/src/tokio/mod.rs` | ✅ Complete |
| `AsyncHandleScope` | `rudo-gc/src/handles/async.rs` | ✅ Complete |
| `AsyncHandle<T>` | `rudo-gc/src/handles/async.rs` | ✅ Complete |
| `spawn_with_gc!` | `rudo-gc/src/handles/async.rs` | ✅ Complete |

### 1.2 What's Missing in Rvue

| Component | Priority | Complexity |
|-----------|----------|------------|
| `dispatch_to_ui()` | P0 - Critical | Medium |
| `spawn_task()` | P0 - Critical | Medium |
| `TaskHandle` | P0 - Critical | Low |
| `TaskRegistry` | P1 - High | Medium |
| `on_cleanup()` hook | P1 - High | Medium |
| `create_resource()` | P2 - Medium | High |
| `spawn_interval()` | P3 - Low | Low |
| `spawn_debounced()` | P3 - Low | Medium |
| `Suspense` component | P3 - Low | High |

### 1.3 Current Rvue Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      winit Event Loop                        │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ WindowEvent  │───▶│   AppState   │───▶│    Render    │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                               │
│         ▼                   ▼                               │
│  ┌──────────────┐    ┌──────────────┐                      │
│  │ Event Pass   │    │ Update Pass  │                      │
│  └──────────────┘    └──────────────┘                      │
│                            │                                │
│                            ▼                                │
│                     ┌──────────────┐                        │
│                     │  Component   │                        │
│                     │    Tree      │                        │
│                     └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

**Problem:** No async runtime integration. All operations are synchronous.

---

## 2. Design Goals

### 2.1 Primary Goals

1. **GC Safety**: All async operations must correctly interact with rudo-gc
2. **Thread Safety**: Clear Send/Sync boundaries
3. **Lifecycle Binding**: Tasks cancelled when component unmounts
4. **Ergonomic API**: Minimal boilerplate for common patterns
5. **Non-blocking UI**: Async work never blocks the UI thread

### 2.2 Non-Goals (v1)

1. ~~Custom executor~~ - Use tokio
2. ~~Streaming/SSR~~ - Focus on desktop GUI
3. ~~Hot module replacement~~ - Future work
4. ~~Wasm support~~ - tokio doesn't support wasm well

### 2.3 Design Principles

| Principle | Rationale |
|-----------|-----------|
| Explicit over implicit | Make async boundaries visible |
| Fail fast | Panic on misuse rather than silent bugs |
| Composable | Small primitives that combine well |
| Zero-cost when unused | No overhead if async not used |

---

## 3. Architecture Overview

### 3.1 Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      winit Event Loop                            │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ WindowEvent  │───▶│   AppState   │───▶│    Render    │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                                    │
│         │                   ▼                                    │
│         │           ┌──────────────┐                            │
│         │           │ UI Dispatch  │◀────────────┐              │
│         │           │    Queue     │             │              │
│         │           └──────────────┘             │              │
│         │                   │                    │              │
│         ▼                   ▼                    │              │
│  ┌──────────────┐    ┌──────────────┐           │              │
│  │ Event Pass   │    │ Update Pass  │           │              │
│  └──────────────┘    └──────────────┘           │              │
│                            │                     │              │
│                            ▼                     │              │
│                     ┌──────────────┐            │              │
│                     │  Component   │            │              │
│                     │    Tree      │            │              │
│                     └──────────────┘            │              │
└─────────────────────────────────────────────────│──────────────┘
                                                  │
                    ┌─────────────────────────────┴────────────┐
                    │              tokio Runtime                │
                    │                                           │
                    │  ┌─────────┐  ┌─────────┐  ┌─────────┐   │
                    │  │  Task   │  │  Task   │  │  Task   │   │
                    │  └─────────┘  └─────────┘  └─────────┘   │
                    │                                           │
                    │  ┌──────────────────────────────────────┐ │
                    │  │           Task Registry               │ │
                    │  │  (tracks task → component mapping)    │ │
                    │  └──────────────────────────────────────┘ │
                    └───────────────────────────────────────────┘
```

### 3.2 Key Components

| Component | Responsibility |
|-----------|----------------|
| `UiDispatchQueue` | Thread-safe queue for closures to run on UI thread |
| `TaskRegistry` | Maps tasks to components for lifecycle management |
| `TaskHandle` | Abort handle with metadata |
| `AsyncContext` | Per-component async state |

### 3.3 Threading Model

```
┌─────────────────┐     ┌─────────────────┐
│   UI Thread     │     │  tokio Workers   │
│                 │     │                  │
│ • Event loop    │     │ • spawn_task()   │
│ • Rendering     │     │ • I/O operations │
│ • Layout        │     │ • Computation    │
│ • GC (primary)  │     │                  │
│                 │     │                  │
│ ◀──────────────────── │ dispatch_to_ui() │
│  closures       │     │                  │
└─────────────────┘     └─────────────────┘

Thread boundary: mpsc channel
GC boundary: AsyncHandleScope / GcRootGuard
```

---

## Phase 1: Foundation

**Duration:** 1-2 weeks  
**Goal:** Core infrastructure for async ↔ UI communication

### 1.1 Add tokio Dependency

**File:** `crates/rvue/Cargo.toml`

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time"] }

[features]
default = []
async = ["tokio"]
```

**Rationale:** Feature-gated to avoid overhead for sync-only apps.

### 1.2 Create Async Module Structure

**Files to create:**

```
crates/rvue/src/
├── async_runtime/
│   ├── mod.rs           # Module root, re-exports
│   ├── dispatch.rs      # UI dispatch queue
│   ├── task.rs          # Task spawning and handles
│   ├── registry.rs      # Task-to-component mapping
│   └── context.rs       # Per-component async state
```

### 1.3 Implement UiDispatchQueue

**File:** `crates/rvue/src/async_runtime/dispatch.rs`

```rust
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::VecDeque;

/// A closure that can be sent to the UI thread
type UiCallback = Box<dyn FnOnce() + Send + 'static>;

/// Thread-safe queue for UI dispatch
pub struct UiDispatchQueue {
    queue: Arc<Mutex<VecDeque<UiCallback>>>,
}

impl UiDispatchQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Enqueue a closure to run on the UI thread.
    /// 
    /// This is safe to call from any thread.
    pub fn dispatch<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.lock().push_back(Box::new(f));
    }

    /// Drain and execute all pending callbacks.
    /// 
    /// Must be called on the UI thread.
    pub fn drain_and_execute(&self) {
        let callbacks: Vec<_> = {
            let mut queue = self.queue.lock();
            queue.drain(..).collect()
        };

        for callback in callbacks {
            callback();
        }
    }

    /// Returns number of pending callbacks.
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }
}

// Global singleton for simple API
static UI_DISPATCH_QUEUE: once_cell::sync::Lazy<UiDispatchQueue> =
    once_cell::sync::Lazy::new(UiDispatchQueue::new);

/// Dispatch a closure to run on the UI thread.
/// 
/// # Example
/// 
/// ```rust
/// use rvue::async_runtime::dispatch_to_ui;
/// 
/// tokio::spawn(async move {
///     let result = fetch_data().await;
///     dispatch_to_ui(move || {
///         signal.set(result);
///     });
/// });
/// ```
pub fn dispatch_to_ui<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    UI_DISPATCH_QUEUE.dispatch(f);
}

/// Drain and execute pending UI callbacks.
/// 
/// Called internally by the event loop.
pub(crate) fn drain_ui_callbacks() {
    UI_DISPATCH_QUEUE.drain_and_execute();
}
```

### 1.4 Integrate with AppState

**File:** `crates/rvue/src/app.rs` (modifications)

```rust
impl<'a> AppState<'a> {
    fn run_update_passes(&mut self) {
        // NEW: Execute pending UI callbacks first
        #[cfg(feature = "async")]
        crate::async_runtime::dispatch::drain_ui_callbacks();

        // Existing update passes...
        run_update_focus_pass(self);
        run_update_pointer_pass(self);
        update_cursor_blink_states(self);
        // ...
    }
}
```

### 1.5 Tests for Phase 1

**File:** `crates/rvue/src/async_runtime/tests/dispatch_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_dispatch_executes_on_drain() {
        let queue = UiDispatchQueue::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        let counter_clone = counter.clone();
        queue.dispatch(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 0);
        queue.drain_and_execute();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_dispatch_from_multiple_threads() {
        let queue = Arc::new(UiDispatchQueue::new());
        let counter = Arc::new(AtomicUsize::new(0));
        
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let q = queue.clone();
                let c = counter.clone();
                std::thread::spawn(move || {
                    q.dispatch(move || {
                        c.fetch_add(1, Ordering::SeqCst);
                    });
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        queue.drain_and_execute();
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_dispatch_from_tokio_task() {
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        dispatch_to_ui(move || {
            executed_clone.store(true, Ordering::SeqCst);
        });

        // Simulate UI thread processing
        drain_ui_callbacks();

        assert!(executed.load(Ordering::SeqCst));
    }
}
```

### 1.6 Phase 1 Deliverables

- [ ] `tokio` dependency added with feature flag
- [ ] `UiDispatchQueue` implemented
- [ ] `dispatch_to_ui()` function available
- [ ] Integration with `AppState::run_update_passes()`
- [ ] Unit tests passing
- [ ] Documentation for dispatch pattern

---

## Phase 2: Task Management

**Duration:** 2-3 weeks  
**Goal:** Task spawning, handles, and lifecycle management

### 2.1 TaskHandle Implementation

**File:** `crates/rvue/src/async_runtime/task.rs`

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::task::{AbortHandle, JoinHandle};

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TaskId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Handle for managing a spawned async task.
/// 
/// When dropped, the task continues running. Call `abort()` to cancel.
#[derive(Debug)]
pub struct TaskHandle {
    id: TaskId,
    abort_handle: AbortHandle,
    completed: Arc<AtomicBool>,
}

impl TaskHandle {
    fn new(abort_handle: AbortHandle) -> Self {
        Self {
            id: TaskId::new(),
            abort_handle,
            completed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cancel the task.
    /// 
    /// This is idempotent - calling multiple times is safe.
    pub fn abort(&self) {
        self.abort_handle.abort();
    }

    /// Check if the task has completed (successfully or with error).
    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::Acquire)
    }

    /// Check if the task is still running.
    pub fn is_running(&self) -> bool {
        !self.is_completed() && !self.abort_handle.is_finished()
    }

    /// Get the unique task ID.
    pub fn id(&self) -> TaskId {
        self.id
    }
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            abort_handle: self.abort_handle.clone(),
            completed: self.completed.clone(),
        }
    }
}
```

### 2.2 spawn_task Implementation

**File:** `crates/rvue/src/async_runtime/task.rs` (continued)

```rust
use rudo_gc::{Gc, Trace, AsyncHandleScope};

/// Spawn an async task with GC-aware root tracking.
/// 
/// The task runs on the tokio runtime and can safely access GC objects
/// through the provided `AsyncHandleScope`.
/// 
/// # Example
/// 
/// ```rust
/// use rvue::async_runtime::spawn_task;
/// 
/// let gc = Gc::new(MyData { value: 42 });
/// 
/// spawn_task(async move {
///     let result = fetch_data().await;
///     dispatch_to_ui(move || {
///         signal.set(result);
///     });
/// });
/// ```
/// 
/// # GC Safety
/// 
/// To use GC objects across `.await` points, use `rudo_gc::spawn_with_gc!`:
/// 
/// ```rust
/// rudo_gc::spawn_with_gc!(gc => |handle| async move {
///     tokio::time::sleep(Duration::from_secs(1)).await;
///     let value = unsafe { handle.get().value };
///     dispatch_to_ui(move || {
///         signal.set(value);
///     });
/// });
/// ```
pub fn spawn_task<F>(future: F) -> TaskHandle
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let join_handle = tokio::spawn(future);
    TaskHandle::new(join_handle.abort_handle())
}

/// Spawn a task that returns a value.
/// 
/// The result is delivered via `dispatch_to_ui` callback.
pub fn spawn_task_with_result<F, T, C>(future: F, on_complete: C) -> TaskHandle
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
    C: FnOnce(T) + Send + 'static,
{
    let completed = Arc::new(AtomicBool::new(false));
    let completed_clone = completed.clone();

    let join_handle = tokio::spawn(async move {
        let result = future.await;
        completed_clone.store(true, Ordering::Release);
        dispatch_to_ui(move || {
            on_complete(result);
        });
    });

    let mut handle = TaskHandle::new(join_handle.abort_handle());
    handle.completed = completed;
    handle
}
```

### 2.3 TaskRegistry for Lifecycle Management

**File:** `crates/rvue/src/async_runtime/registry.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use rudo_gc::Gc;
use crate::component::Component;
use super::task::{TaskHandle, TaskId};

/// Registry mapping components to their spawned tasks.
/// 
/// When a component is unmounted, all its tasks are cancelled.
pub struct TaskRegistry {
    /// Component ID -> Vec<TaskHandle>
    tasks: RwLock<HashMap<usize, Vec<TaskHandle>>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a task for a component.
    /// 
    /// The task will be cancelled when the component is unmounted.
    pub fn register(&self, component_id: usize, handle: TaskHandle) {
        let mut tasks = self.tasks.write();
        tasks.entry(component_id).or_default().push(handle);
    }

    /// Cancel all tasks for a component.
    /// 
    /// Called when component is unmounted.
    pub fn cancel_all(&self, component_id: usize) {
        let mut tasks = self.tasks.write();
        if let Some(handles) = tasks.remove(&component_id) {
            for handle in handles {
                handle.abort();
            }
        }
    }

    /// Remove completed tasks to prevent memory growth.
    pub fn cleanup_completed(&self) {
        let mut tasks = self.tasks.write();
        for (_, handles) in tasks.iter_mut() {
            handles.retain(|h| h.is_running());
        }
    }

    /// Get count of active tasks for a component.
    pub fn task_count(&self, component_id: usize) -> usize {
        self.tasks
            .read()
            .get(&component_id)
            .map(|v| v.iter().filter(|h| h.is_running()).count())
            .unwrap_or(0)
    }
}

// Global registry
static TASK_REGISTRY: once_cell::sync::Lazy<TaskRegistry> =
    once_cell::sync::Lazy::new(TaskRegistry::new);

/// Get the global task registry.
pub fn task_registry() -> &'static TaskRegistry {
    &*TASK_REGISTRY
}
```

### 2.4 Component Lifecycle Integration

**File:** `crates/rvue/src/component.rs` (modifications)

```rust
impl Component {
    /// Called when component is being unmounted.
    pub fn unmount(&self) {
        // Cancel all async tasks
        #[cfg(feature = "async")]
        crate::async_runtime::registry::task_registry()
            .cancel_all(self.id());

        // Run cleanup callbacks
        self.run_cleanup_callbacks();

        // Recursively unmount children
        for child in self.children.borrow().iter() {
            child.unmount();
        }
    }
}
```

### 2.5 on_cleanup Hook

**File:** `crates/rvue/src/async_runtime/context.rs`

```rust
use std::cell::RefCell;
use rudo_gc::Gc;
use crate::component::Component;

/// Cleanup callback type
type CleanupFn = Box<dyn FnOnce() + 'static>;

thread_local! {
    /// Stack of current component contexts
    static CONTEXT_STACK: RefCell<Vec<Gc<Component>>> = RefCell::new(Vec::new());
}

/// Register a cleanup callback for the current component.
/// 
/// The callback will be called when the component is unmounted.
/// 
/// # Example
/// 
/// ```rust
/// #[component]
/// fn SearchBox(cx: Scope) -> View {
///     let task_handle = use_ref(cx, || None);
///     
///     on_cleanup(|| {
///         if let Some(handle) = task_handle.take() {
///             handle.abort();
///         }
///     });
///     
///     // ...
/// }
/// ```
pub fn on_cleanup<F>(f: F)
where
    F: FnOnce() + 'static,
{
    CONTEXT_STACK.with(|stack| {
        let stack = stack.borrow();
        if let Some(component) = stack.last() {
            component.add_cleanup_callback(Box::new(f));
        } else {
            panic!("on_cleanup called outside of component context");
        }
    });
}

/// Push a component onto the context stack.
/// 
/// Used internally during component rendering.
pub(crate) fn push_context(component: Gc<Component>) {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(component);
    });
}

/// Pop a component from the context stack.
/// 
/// Used internally after component rendering.
pub(crate) fn pop_context() -> Option<Gc<Component>> {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().pop()
    })
}
```

### 2.6 Component Cleanup Callbacks

**File:** `crates/rvue/src/component.rs` (additions)

```rust
impl Component {
    // Add field to Component struct:
    // cleanup_callbacks: RefCell<Vec<CleanupFn>>,

    pub fn add_cleanup_callback(&self, callback: Box<dyn FnOnce() + 'static>) {
        self.cleanup_callbacks.borrow_mut().push(callback);
    }

    fn run_cleanup_callbacks(&self) {
        let callbacks: Vec<_> = self.cleanup_callbacks.borrow_mut().drain(..).collect();
        for callback in callbacks {
            callback();
        }
    }
}
```

### 2.7 Phase 2 Deliverables

- [ ] `TaskHandle` with abort, completion tracking
- [ ] `spawn_task()` and `spawn_task_with_result()`
- [ ] `TaskRegistry` for component → task mapping
- [ ] `on_cleanup()` hook
- [ ] Integration with component unmount lifecycle
- [ ] Tests for task cancellation on unmount
- [ ] Documentation

---

## Phase 3: Reactive Integration

**Duration:** 2-3 weeks  
**Goal:** Make signals work with async

### 3.1 Signal Thread Safety Analysis

**Current state in `crates/rvue/src/signal.rs`:**

Signals use `GcCell<T>` internally, which is NOT `Send + Sync`.

**Options:**

| Option | Pros | Cons |
|--------|------|------|
| A: Keep signals !Send | Simple, no changes | Verbose async code |
| B: Make signals Send+Sync | Ergonomic async | Complex, overhead |
| C: Create async signal proxy | Best of both | More API surface |

**Recommendation:** Option C - Create `SignalSender` type

### 3.2 SignalSender Design

**File:** `crates/rvue/src/async_runtime/signal_sender.rs`

```rust
use std::sync::Arc;
use parking_lot::Mutex;
use crate::signal::WriteSignal;

/// A thread-safe sender for updating signals from async tasks.
/// 
/// `SignalSender` captures a closure that updates the signal, allowing
/// the update to be dispatched to the UI thread.
/// 
/// # Example
/// 
/// ```rust
/// let (count, set_count) = create_signal(0);
/// let sender = set_count.sender();
/// 
/// spawn_task(async move {
///     let result = fetch_count().await;
///     sender.set(result);  // Automatically dispatches to UI
/// });
/// ```
#[derive(Clone)]
pub struct SignalSender<T: Clone + Send + 'static> {
    /// Queued value to be set
    pending: Arc<Mutex<Option<T>>>,
    /// Unique ID for deduplication
    signal_id: usize,
}

impl<T: Clone + Send + 'static> SignalSender<T> {
    pub fn new(signal_id: usize) -> Self {
        Self {
            pending: Arc::new(Mutex::new(None)),
            signal_id,
        }
    }

    /// Queue a value to be set on the signal.
    /// 
    /// The actual update happens on the next UI dispatch cycle.
    pub fn set(&self, value: T) {
        let pending = self.pending.clone();
        let signal_id = self.signal_id;

        // Store the latest value
        *pending.lock() = Some(value);

        // Dispatch to UI thread
        dispatch_to_ui(move || {
            if let Some(value) = pending.lock().take() {
                // Look up signal by ID and set value
                SIGNAL_REGISTRY.with(|reg| {
                    if let Some(setter) = reg.borrow().get(&signal_id) {
                        setter(Box::new(value));
                    }
                });
            }
        });
    }

    /// Queue an update function.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&T) -> T + Send + 'static,
    {
        // Implementation would need access to current value
        // This is more complex - consider if needed for v1
        todo!("update() requires additional infrastructure");
    }
}

// SAFETY: SignalSender only holds Arc<Mutex<_>> which is Send+Sync
unsafe impl<T: Clone + Send + 'static> Send for SignalSender<T> {}
unsafe impl<T: Clone + Send + 'static> Sync for SignalSender<T> {}
```

### 3.3 Integration with WriteSignal

**File:** `crates/rvue/src/signal.rs` (modifications)

```rust
impl<T: Clone + 'static> WriteSignal<T> {
    /// Create a thread-safe sender for this signal.
    /// 
    /// The sender can be moved into async tasks.
    #[cfg(feature = "async")]
    pub fn sender(&self) -> SignalSender<T>
    where
        T: Send,
    {
        SignalSender::new(self.id())
    }
}
```

### 3.4 create_resource Pattern

**File:** `crates/rvue/src/async_runtime/resource.rs`

```rust
use std::future::Future;
use std::sync::Arc;
use parking_lot::Mutex;

/// State of an async resource
#[derive(Clone)]
pub enum ResourceState<T> {
    /// Initial state, fetch not started
    Pending,
    /// Fetching data
    Loading,
    /// Data loaded successfully
    Ready(T),
    /// Fetch failed
    Error(String),
}

impl<T> ResourceState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, ResourceState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, ResourceState::Ready(_))
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            ResourceState::Ready(data) => Some(data),
            _ => None,
        }
    }
}

/// A reactive async resource.
/// 
/// `Resource` fetches data asynchronously and provides reactive state.
/// 
/// # Example
/// 
/// ```rust
/// #[component]
/// fn UserProfile(cx: Scope, user_id: i32) -> View {
///     let user = create_resource(
///         cx,
///         move || user_id,  // Source
///         |id| async move { fetch_user(id).await }  // Fetcher
///     );
///     
///     view! {
///         match user.get() {
///             ResourceState::Loading => view! { <Text value="Loading..." /> },
///             ResourceState::Ready(u) => view! { <Text value=u.name.clone() /> },
///             ResourceState::Error(e) => view! { <Text value=e.clone() /> },
///             _ => view! { <Text value="" /> },
///         }
///     }
/// }
/// ```
pub fn create_resource<S, T, Fu, Fetcher>(
    source: S,
    fetcher: Fetcher,
) -> Resource<T>
where
    S: Fn() -> T + 'static,
    T: Clone + Send + 'static,
    Fu: Future<Output = T> + Send + 'static,
    Fetcher: Fn(T) -> Fu + Send + Sync + 'static,
{
    let (state, set_state) = create_signal(ResourceState::Pending);
    let sender = set_state.sender();
    
    // Create the resource
    let resource = Resource {
        state,
        refetch: Arc::new(move || {
            let source_value = source();
            let future = fetcher(source_value);
            let sender = sender.clone();
            
            sender.set(ResourceState::Loading);
            
            spawn_task(async move {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    futures::executor::block_on(future)
                })) {
                    Ok(result) => sender.set(ResourceState::Ready(result)),
                    Err(_) => sender.set(ResourceState::Error("Fetch panicked".into())),
                }
            });
        }),
    };
    
    // Trigger initial fetch
    (resource.refetch)();
    
    resource
}

pub struct Resource<T> {
    state: ReadSignal<ResourceState<T>>,
    refetch: Arc<dyn Fn() + Send + Sync>,
}

impl<T: Clone> Resource<T> {
    /// Get current state.
    pub fn get(&self) -> ResourceState<T> {
        self.state.get()
    }

    /// Trigger a refetch.
    pub fn refetch(&self) {
        (self.refetch)();
    }
}
```

### 3.5 Phase 3 Deliverables

- [ ] `SignalSender<T>` for thread-safe signal updates
- [ ] `WriteSignal::sender()` method
- [ ] Signal registry for ID → setter lookup
- [ ] `create_resource()` function
- [ ] `ResourceState` enum
- [ ] Tests for async signal updates
- [ ] Documentation

---

## Phase 4: Higher-Level APIs

**Duration:** 1-2 weeks  
**Goal:** Ergonomic helpers for common patterns

### 4.1 spawn_interval

```rust
/// Spawn a task that runs periodically.
/// 
/// Returns a handle that can be used to cancel the interval.
/// 
/// # Example
/// 
/// ```rust
/// let handle = spawn_interval(Duration::from_secs(30), || async {
///     let data = fetch_updates().await;
///     dispatch_to_ui(move || {
///         update_signal.set(data);
///     });
/// });
/// 
/// on_cleanup(move || handle.abort());
/// ```
pub fn spawn_interval<F, Fut>(
    period: Duration,
    mut f: F,
) -> TaskHandle
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    spawn_task(async move {
        let mut interval = tokio::time::interval(period);
        loop {
            interval.tick().await;
            f().await;
        }
    })
}
```

### 4.2 spawn_debounced

```rust
/// Spawn a debounced async operation.
/// 
/// If called again before the delay expires, the previous call is cancelled.
/// 
/// # Example
/// 
/// ```rust
/// let search = spawn_debounced(
///     Duration::from_millis(300),
///     move |query: String| async move {
///         let results = search_api(query).await;
///         dispatch_to_ui(move || {
///             suggestions.set(results);
///         });
///     }
/// );
/// 
/// // In event handler:
/// search.call(input_value.get());
/// ```
pub struct DebouncedTask<T: Send + 'static> {
    sender: tokio::sync::mpsc::UnboundedSender<T>,
    handle: TaskHandle,
}

impl<T: Send + 'static> DebouncedTask<T> {
    pub fn call(&self, value: T) {
        let _ = self.sender.send(value);
    }

    pub fn cancel(&self) {
        self.handle.abort();
    }
}

pub fn spawn_debounced<T, F, Fut>(
    delay: Duration,
    handler: F,
) -> DebouncedTask<T>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();

    let handle = spawn_task(async move {
        let mut pending: Option<T> = None;
        let sleep = tokio::time::sleep(delay);
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                Some(value) = rx.recv() => {
                    pending = Some(value);
                    sleep.as_mut().reset(tokio::time::Instant::now() + delay);
                }
                _ = &mut sleep, if pending.is_some() => {
                    if let Some(value) = pending.take() {
                        handler(value).await;
                    }
                }
            }
        }
    });

    DebouncedTask { sender: tx, handle }
}
```

### 4.3 spawn_throttled

```rust
/// Spawn a throttled async operation.
/// 
/// Ensures the operation runs at most once per period.
pub fn spawn_throttled<T, F, Fut>(
    period: Duration,
    handler: F,
) -> ThrottledTask<T>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Similar to debounced but with different timing logic
    todo!()
}
```

### 4.4 Phase 4 Deliverables

- [ ] `spawn_interval()`
- [ ] `spawn_debounced()` with `DebouncedTask`
- [ ] `spawn_throttled()` with `ThrottledTask`
- [ ] Examples in docs
- [ ] Integration tests

---

## Phase 5: Testing & Documentation

**Duration:** 1-2 weeks  
**Goal:** Comprehensive testing and documentation

### 5.1 Test Matrix

| Test Category | Tests |
|---------------|-------|
| Unit: dispatch | Single-thread, multi-thread, ordering |
| Unit: task | Spawn, abort, completion |
| Unit: registry | Register, cancel, cleanup |
| Unit: signal sender | Set, dedup, ordering |
| Integration: lifecycle | Mount → spawn → unmount → verify cancelled |
| Integration: stress | Many tasks, concurrent updates |
| GC: safety | Tasks with Gc objects, collection timing |

### 5.2 Example Applications

1. **async-counter** - Simple fetch + update
2. **search-suggestions** - Debounced search input
3. **live-dashboard** - Interval polling
4. **file-browser** - Nested async loading

### 5.3 Documentation Structure

```
docs/
├── async-guide.md          # User guide
├── async-best-practices.md # Updated with real examples
├── async-api-reference.md  # API docs
└── async-migration.md      # Upgrading from sync code
```

### 5.4 Phase 5 Deliverables

- [ ] Full test suite passing
- [ ] 4 example applications
- [ ] Updated docs with accurate content
- [ ] Migration guide
- [ ] Performance benchmarks

---

## API Reference

### Core Functions

```rust
// Dispatch to UI thread
pub fn dispatch_to_ui<F: FnOnce() + Send + 'static>(f: F);

// Spawn tasks
pub fn spawn_task<F: Future<Output = ()> + Send + 'static>(f: F) -> TaskHandle;
pub fn spawn_task_with_result<F, T, C>(future: F, on_complete: C) -> TaskHandle;

// Cleanup
pub fn on_cleanup<F: FnOnce() + 'static>(f: F);

// Helpers
pub fn spawn_interval<F, Fut>(period: Duration, f: F) -> TaskHandle;
pub fn spawn_debounced<T, F, Fut>(delay: Duration, handler: F) -> DebouncedTask<T>;
```

### Types

```rust
pub struct TaskHandle { /* ... */ }
impl TaskHandle {
    pub fn abort(&self);
    pub fn is_completed(&self) -> bool;
    pub fn is_running(&self) -> bool;
    pub fn id(&self) -> TaskId;
}

pub struct SignalSender<T> { /* ... */ }
impl<T: Clone + Send> SignalSender<T> {
    pub fn set(&self, value: T);
}

pub struct Resource<T> { /* ... */ }
impl<T: Clone> Resource<T> {
    pub fn get(&self) -> ResourceState<T>;
    pub fn refetch(&self);
}

pub enum ResourceState<T> {
    Pending,
    Loading,
    Ready(T),
    Error(String),
}
```

---

## Migration Guide

### From Sync to Async

**Before (blocking):**
```rust
fn fetch_data() -> Data {
    reqwest::blocking::get(url).unwrap().json().unwrap()
}

#[component]
fn MyComponent(cx: Scope) -> View {
    let data = fetch_data(); // Blocks UI!
    view! { <Text value=data.name /> }
}
```

**After (async):**
```rust
async fn fetch_data() -> Data {
    reqwest::get(url).await?.json().await?
}

#[component]
fn MyComponent(cx: Scope) -> View {
    let data = create_resource(|| (), |_| fetch_data());
    
    view! {
        match data.get() {
            ResourceState::Ready(d) => view! { <Text value=d.name.clone() /> },
            ResourceState::Loading => view! { <Text value="Loading..." /> },
            _ => view! { <Text value="" /> },
        }
    }
}
```

---

## Open Questions

### Q1: tokio Runtime Management

**Options:**
1. User creates runtime manually
2. Rvue creates runtime internally
3. Lazy initialization on first `spawn_task`

**Recommendation:** Option 3 - Lazy initialization with sensible defaults

### Q2: Error Handling

**Options:**
1. Propagate via `Result` in `ResourceState`
2. Global error boundary pattern
3. Per-component error handling

**Recommendation:** Start with Option 1, add error boundaries later

### Q3: Cancellation Tokens

**Should we expose `CancellationToken` for cooperative cancellation?**

**Recommendation:** Yes, but in Phase 4 as an optional feature

### Q4: Suspense Component

**Should v1 include `Suspense`?**

**Recommendation:** No - complex, save for v2

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1: Foundation | 1-2 weeks | `dispatch_to_ui`, tokio integration |
| Phase 2: Task Management | 2-3 weeks | `spawn_task`, `TaskHandle`, lifecycle |
| Phase 3: Reactive Integration | 2-3 weeks | `SignalSender`, `create_resource` |
| Phase 4: Higher-Level APIs | 1-2 weeks | interval, debounce, throttle |
| Phase 5: Testing & Docs | 1-2 weeks | Tests, examples, documentation |
| **Total** | **7-12 weeks** | Full async support |

---

## Appendix: rudo-gc Integration Notes

### Using AsyncHandleScope

```rust
use rudo_gc::{Gc, Trace, AsyncHandleScope};

async fn with_gc_objects() {
    let tcb = rudo_gc::heap::current_thread_control_block()
        .expect("must be in GC thread");
    
    let scope = AsyncHandleScope::new(&tcb);
    let gc = Gc::new(Data { value: 42 });
    let handle = scope.handle(&gc);
    
    // Safe to use handle across await
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // SAFETY: scope is still alive
    let value = unsafe { handle.get().value };
}
```

### Using spawn_with_gc!

```rust
use rudo_gc::{Gc, spawn_with_gc};

fn example() {
    let gc = Gc::new(Data { value: 42 });
    
    // Macro handles scope creation and cleanup
    spawn_with_gc!(gc => |handle| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let value = unsafe { handle.get().value };
        dispatch_to_ui(move || {
            signal.set(value);
        });
    });
}
```

### GcRootGuard (Simple Case)

```rust
use rudo_gc::{Gc, GcTokioExt};

fn simple_case() {
    let gc = Gc::new(Data { value: 42 });
    let _guard = gc.root_guard();  // Keeps gc alive
    
    tokio::spawn(async move {
        // gc is protected for task duration
        println!("{}", gc.value);
    });
}
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-07  
**Status:** Ready for Review
