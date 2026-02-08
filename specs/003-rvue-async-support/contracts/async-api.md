# API Contract: Rvue Async Runtime

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-08
**Updated**: Aligned with actual implementation

This document defines the public API surface for the async runtime module. All items listed here are part of the public contract and must have tests.

---

## Module: `rvue::async_runtime`

**Feature gate**: `#[cfg(feature = "async")]`

### Re-exports (via `rvue::prelude`)

```rust
pub use async_runtime::{
    dispatch_to_ui,
    spawn_task,
    spawn_interval,
    spawn_debounced,
    watch_signal,
    create_resource,
    TaskHandle,
    TaskId,
    AsyncSignalSender,
    Resource,
    ResourceState,
    DebouncedTask,
    IntervalHandle,
    SignalWatcher,
    ComponentScope,
    WriteSignalExt,
};
```

---

## Core Functions

### `dispatch_to_ui`

```rust
/// Dispatch a closure to execute on the UI thread.
///
/// # Thread Safety
/// Safe to call from any thread. The closure will execute during
/// the next event loop cycle.
///
/// # Panics
/// Does not panic. If the event loop proxy is not yet initialized
/// (before `run_app()`), the callback is queued and will execute
/// once the event loop starts.
///
/// # Example
/// ```rust
/// use rvue::async_runtime::dispatch_to_ui;
///
/// std::thread::spawn(|| {
///     let result = expensive_computation();
///     dispatch_to_ui(move || {
///         set_data(result);
///     });
/// });
/// ```
pub fn dispatch_to_ui<F>(f: F)
where
    F: FnOnce() + Send + 'static;
```

**Contract**:
- MUST be callable from any thread (Send requirement on F)
- MUST execute `f` on the UI thread
- MUST execute callbacks in FIFO order
- MUST NOT block the calling thread
- MUST wake the event loop via `EventLoopProxy` if available
- MUST catch panics in `f` and log them (not crash UI)

---

### `spawn_task`

```rust
/// Spawn an async task on the tokio runtime.
///
/// The task runs concurrently on a background thread. To send results
/// back to the UI, use `dispatch_to_ui()` within the task.
///
/// # Returns
/// A `TaskHandle` that can be used to abort or query the task.
///
/// # Example
/// ```rust
/// use rvue::async_runtime::{spawn_task, dispatch_to_ui};
///
/// let handle = spawn_task(async move {
///     let data = fetch_api("/users").await;
///     dispatch_to_ui(move || {
///         set_users(data);
///     });
/// });
///
/// // Optionally cancel:
/// handle.abort();
/// ```
pub fn spawn_task<F>(future: F) -> TaskHandle
where
    F: std::future::Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST spawn the future on the tokio runtime
- MUST lazily initialize the runtime on first call
- MUST return a valid `TaskHandle`
- MUST NOT block the calling thread
- The future MUST be `Send + 'static`

---

### `spawn_interval`

```rust
/// Spawn a recurring async task that runs at a fixed interval.
///
/// The task starts immediately and repeats every `period`.
///
/// # GC Safety
/// **IMPORTANT**: The closure MUST NOT capture `Gc<T>` objects.
///
/// `Gc<T>` is `!Send + !Sync` and cannot be moved across thread boundaries
/// without explicit handling. The closure is spawned on a tokio worker thread,
/// so any captured `Gc<T>` will cause memory corruption.
///
/// **Correct Usage**:
/// ```ignore
/// let count = create_signal(0i32);
/// let current = *count.get();  // Extract value
/// spawn_interval(Duration::from_secs(1), move || {
///     println!("{}", current);  // Safe: current is owned value
/// });
/// ```
///
/// # Example
/// ```rust
/// use std::time::Duration;
/// use rvue::async_runtime::spawn_interval;
///
/// let handle = spawn_interval(Duration::from_secs(30), || async {
///     let status = check_server_status().await;
///     dispatch_to_ui(move || {
///         set_status(status);
///     });
/// });
///
/// on_cleanup(move || handle.stop());
/// ```
pub fn spawn_interval<F, Fut>(period: Duration, f: F) -> IntervalHandle
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST execute `f` immediately, then every `period`
- MUST stop when `IntervalHandle::stop()` is called
- **GC Safety**: The closure must not capture `Gc<T>` or any `!Send` types

---

### `spawn_debounced`

```rust
/// Create a debounced async operation.
///
/// When `call()` is invoked, the operation is delayed by `delay`.
/// If `call()` is invoked again before the delay expires, the
/// previous pending execution is cancelled and the timer resets.
///
/// # GC Safety
/// **IMPORTANT**: The handler closure and the value type `T` MUST NOT contain `Gc<T>`.
///
/// `Gc<T>` is `!Send + !Sync`. Values are sent through an mpsc channel to the
/// debounce task, which runs on a tokio worker thread.
///
/// **Correct Usage**:
/// ```ignore
/// let gc_data = Gc::new(MyData { value: 42 });
/// let value = gc_data.value.clone();  // Extract value
///
/// let search = spawn_debounced(Duration::from_millis(300), move |query: String| {
///     let data = value.clone();  // Clone from extracted value
///     async move {
///         let result = search_api(&query, &data).await;
///         dispatch_to_ui(move || { /* update UI */ });
///     }
/// });
/// ```
///
/// # Example
/// ```rust
/// use std::time::Duration;
/// use rvue::async_runtime::spawn_debounced;
///
/// let search = spawn_debounced(Duration::from_millis(300), |query: String| async move {
///     let results = search_api(&query).await;
///     dispatch_to_ui(move || {
///         set_suggestions(results);
///     });
/// });
///
/// // In event handler:
/// search.call(input.get());
/// ```
pub fn spawn_debounced<T, F, Fut>(delay: Duration, handler: F) -> DebouncedTask<T>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST only execute `handler` after `delay` has elapsed since last `call()`
- MUST cancel previous pending execution when `call()` is invoked
- MUST use the value from the most recent `call()`
- `DebouncedTask::cancel()` MUST stop all pending and future executions
- **GC Safety**: `T` must be `Send`; handler closure must not capture `Gc<T>`

---

### `watch_signal`

```rust
/// Watch a signal and invoke callback on changes.
///
/// Uses AsyncHandle for safe GC-managed access in async context.
/// The watcher runs on a tokio task and polls the signal at the given period.
///
/// # Arguments
/// - `read_signal`: The signal to watch
/// - `write_signal`: The signal to update (can be the same as read_signal)
/// - `period`: How often to poll the signal
/// - `callback`: Called with current value; return `Some(v)` to update, `None` to just watch
///
/// # Returns
/// A `SignalWatcher` handle for stopping the watcher.
///
/// # GC Safety
/// **SAFE**: This function uses `AsyncHandleScope` for safe GC-managed
/// access to signals across async await points.
///
/// # Example
/// ```rust
/// use rvue::prelude::*;
/// use rvue::async_runtime::watch_signal;
/// use std::time::Duration;
///
/// #[component]
/// fn LiveCounter() -> View {
///     let (count, set_count) = create_signal(0i32);
///
///     let watcher = watch_signal(
///         count,
///         set_count,
///         Duration::from_millis(100),
///         |current| {
///             println!("Count: {}", current);
///             None  // Just watch, don't update
///         }
///     );
///
///     on_cleanup(move || watcher.stop());
///
///     view! {
///         <Text value=format!("Count: {}", count.get()) />
///     }
/// }
/// ```
pub async fn watch_signal<T>(
    read_signal: ReadSignal<T>,
    write_signal: WriteSignal<T>,
    period: Duration,
    mut callback: impl FnMut(T) -> Option<T> + Send + Sync + 'static,
) -> SignalWatcher
where
    T: Trace + Clone + Send + Sync + 'static;
```

**Contract**:
- MUST poll the signal at the specified interval
- MUST execute callback on each poll with current signal value
- MUST dispatch `Some(v)` to update signal
- MUST return `SignalWatcher` that can stop the watcher
- **GC Safety**: Uses `AsyncHandleScope` for thread-safe signal access

---

### `create_resource`

```rust
/// Create a reactive async resource.
///
/// The resource fetches data using `fetcher` and exposes the result
/// as a reactive `ResourceState<T>` signal.
///
/// # Type Parameters
/// - `S`: Source function that provides the input to the fetcher
/// - `T`: The data type returned by the fetcher
/// - `Fu`: The future returned by the fetcher
/// - `Fetcher`: Function that creates a fetch future from the source value
///
/// # Example
/// ```rust
/// use rvue::async_runtime::create_resource;
///
/// let user = create_resource(
///     move || user_id.get(),     // source (reactive)
///     |id| async move {          // fetcher
///         fetch_user(id).await
///     },
/// );
///
/// // In view:
/// match user.get() {
///     ResourceState::Ready(u) => { /* show user */ }
///     ResourceState::Loading => { /* show spinner */ }
///     ResourceState::Error(e) => { /* show error */ }
///     ResourceState::Pending => { /* initial state */ }
/// }
/// ```
pub fn create_resource<S, T, Fu, Fetcher>(
    source: S,
    fetcher: Fetcher,
) -> Resource<T>
where
    S: Fn() -> T + 'static,
    T: Trace + Clone + Send + 'static,
    Fu: std::future::Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(T) -> Fu + Send + Sync + 'static;
```

**Contract**:
- MUST trigger initial fetch on creation
- MUST transition through `Pending → Loading → Ready/Error`
- `Resource::get()` MUST be reactive (tracked by effects)
- `Resource::refetch()` MUST trigger a new fetch cycle (`Loading → Ready/Error`)
- MUST cancel in-flight fetch on refetch (only latest result applies)

---

## Types

### `TaskHandle`

```rust
#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: TaskId,
    pub abort_handle: tokio::task::AbortHandle,
    pub completed: Arc<AtomicBool>,
}

impl TaskHandle {
    /// Cancel the task. Idempotent — safe to call multiple times.
    pub fn abort(&self);

    /// Check if the task has completed (success or error).
    pub fn is_completed(&self) -> bool;

    /// Check if the task is still running.
    pub fn is_running(&self) -> bool;

    /// Get the unique task identifier.
    pub fn id(&self) -> TaskId;
}
```

### `TaskId`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);
```

### `AsyncSignalSender<T>`

```rust
#[derive(Clone)]
pub struct AsyncSignalSender<T: Trace + Clone + 'static> {
    data: Gc<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> AsyncSignalSender<T> {
    /// Create a new AsyncSignalSender.
    pub fn new(write_signal: WriteSignal<T>) -> Self;

    /// Set the signal value asynchronously.
    pub async fn set(&self, value: T);
}
```

**Contract**:
- MUST set the signal value on the GC thread
- MUST notify subscribers after setting
- **GC Safety**: Uses `Gc<T>` for thread-safe signal access

---

### `WriteSignalExt<T>` Trait

```rust
/// Extension trait for creating async signal senders.
pub trait WriteSignalExt<T: Trace + Clone + 'static> {
    /// Creates an AsyncSignalSender for thread-safe signal updates from async contexts.
    fn sender(&self) -> AsyncSignalSender<T>;
}

impl<T: Trace + Clone + 'static> WriteSignalExt<T> for WriteSignal<T> {
    fn sender(&self) -> AsyncSignalSender<T> {
        AsyncSignalSender::new(self.clone())
    }
}
```

**Usage**:
```rust
use rvue::prelude::WriteSignalExt;

let (count, set_count) = create_signal(0i32);
let sender = set_count.sender();

tokio::spawn(async move {
    sender.set(100).await;
});
```

---

### `ComponentScope`

```rust
#[derive(Default, Clone)]
pub struct ComponentScope {
    scope: std::rc::Rc<std::cell::RefCell<rudo_gc::handles::GcScope>>,
}

impl ComponentScope {
    /// Create a new empty component scope.
    pub fn new() -> Self;

    /// Track a single component.
    pub fn track(&mut self, component: &Gc<Component>);

    /// Track multiple components from a slice.
    pub fn track_all(&mut self, components: &[Gc<Component>]);

    /// Track components from another scope.
    pub fn extend(&mut self, other: &ComponentScope);

    /// Get the number of tracked components.
    pub fn len(&self) -> usize;

    /// Check if the scope is empty.
    pub fn is_empty(&self) -> bool;
}
```

**Purpose**: Dynamic tracking of GC-managed components for async operations.

---

### `Resource<T>`

```rust
pub struct Resource<T: Trace + Clone + 'static> { /* private fields */ }

impl<T: Trace + Clone + 'static> Resource<T> {
    /// Get the current resource state. Reactive — tracked by effects.
    pub fn get(&self) -> ResourceState<T>;

    /// Manually trigger a refetch.
    pub fn refetch(&self);
}
```

### `ResourceState<T>`

```rust
#[derive(Clone, Debug)]
pub enum ResourceState<T> {
    /// Initial state before any fetch
    Pending,
    /// Fetch in progress
    Loading,
    /// Data loaded successfully
    Ready(T),
    /// Fetch failed
    Error(String),
}

impl<T> ResourceState<T> {
    pub fn is_loading(&self) -> bool;
    pub fn is_ready(&self) -> bool;
    pub fn is_error(&self) -> bool;
    pub fn data(&self) -> Option<&T>;
    pub fn error(&self) -> Option<&str>;
}
```

### `DebouncedTask<T>`

```rust
#[derive(Clone)]
pub struct DebouncedTask<T: Send + 'static> {
    sender: tokio::sync::mpsc::UnboundedSender<T>,
    stopped: Arc<AtomicBool>,
}

impl<T: Send + 'static> DebouncedTask<T> {
    /// Submit a value. Resets the debounce timer.
    pub fn call(&self, value: T);

    /// Cancel the debounce task entirely.
    pub fn cancel(&self);
}
```

---

### `SignalWatcher`

```rust
/// Handle for stopping a signal watcher.
#[derive(Clone)]
pub struct SignalWatcher {
    stopped: Arc<AtomicBool>,
}

impl SignalWatcher {
    /// Create a new stopped signal watcher.
    pub fn new() -> Self;

    /// Stop watching (drops the scope, stopping the task).
    pub fn stop(self);
}
```

### `IntervalHandle`

```rust
/// A handle for stopping interval tasks.
#[derive(Clone)]
pub struct IntervalHandle {
    stopped: Arc<AtomicBool>,
}

impl IntervalHandle {
    /// Create a new stopped interval handle.
    pub fn new() -> Self;

    /// Stop the interval task.
    pub fn stop(&self);
}
```

---

## Cargo Feature

```toml
# crates/rvue/Cargo.toml
[features]
default = []
async = ["dep:tokio"]

[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time"], optional = true }
```

**Contract**:
- Without `async` feature: no tokio dependency, no async_runtime module compiled
- All async types and functions gated behind `#[cfg(feature = "async")]`
- `WriteSignalExt::sender()` gated behind `#[cfg(feature = "async")]`

---

## Error Behavior

| Scenario | Behavior |
|----------|----------|
| `dispatch_to_ui` callback panics | Caught by `catch_unwind`, logged, other callbacks continue |
| Async task panics | tokio catches it; task marked as completed |
| `spawn_task` called before `run_app` | Runtime initializes; callbacks queue until event loop starts |
| `AsyncSignalSender::set()` after signal GC'd | No-op (Gc<T> handles this safely) |
| `create_resource` fetcher returns `Err` | State transitions to `ResourceState::Error(msg)` |
| `spawn_debounced` with zero delay | Executes immediately (same as no debounce) |
| Task completes after component unmount | `dispatch_to_ui` callback is still executed (signal may be gone — no-op) |

---

## Implementation Notes

### GC Safety Strategy

The async runtime uses rudo-gc's `AsyncHandleScope` for thread-safe GC access:

1. **`watch_signal`**: Creates an `AsyncHandleScope` on the GC thread, extracts handles for read/write signals, and polls in a tokio task
2. **`AsyncSignalSender`**: Holds a `Gc<SignalDataInner<T>>` clone that remains valid across awaits
3. **`ComponentScope`**: Uses `Rc<RefCell<GcScope>>` for dynamic component tracking

### Threading Model

- **UI Thread**: Main winit event loop thread where all signal/effect operations occur
- **Background Threads**: tokio worker threads for async computation
- **GC Thread**: Same as UI thread (rudo-gc is thread-local)

### Removed APIs (from original spec)

- `spawn_task_with_result` - Removed due to `Send` bound issues with `SignalDataInner`
- `SignalSender` - Renamed to `AsyncSignalSender` for clarity
- `spawn_watch_signal` - Renamed to `watch_signal` (async function)
- `TaskRegistry` - Lifecycle binding via `on_cleanup` instead of separate registry
