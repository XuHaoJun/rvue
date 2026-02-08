# API Contract: Rvue Async Runtime

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07

This document defines the public API surface for the async runtime module. All items listed here are part of the public contract and must have tests.

---

## Module: `rvue::async_runtime`

**Feature gate**: `#[cfg(feature = "async")]`

### Re-exports (via `rvue::prelude`)

```rust
pub use async_runtime::{
    dispatch_to_ui,
    spawn_task,
    spawn_task_with_result,
    spawn_interval,
    spawn_debounced,
    spawn_watch_signal,
    create_resource,
    TaskHandle,
    TaskId,
    SignalSender,
    Resource,
    ResourceState,
    DebouncedTask,
    SignalWatcher,
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
/// If called within a component context, the task is automatically
/// registered and will be cancelled when the component unmounts.
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
    F: Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST spawn the future on the tokio runtime
- MUST lazily initialize the runtime on first call
- MUST return a valid `TaskHandle`
- MUST register the task with `TaskRegistry` if `current_owner()` exists
- MUST NOT block the calling thread
- The future MUST be `Send + 'static`

---

### `spawn_task_with_result`

```rust
/// Spawn an async task that delivers its result to the UI thread.
///
/// When the future completes, `on_complete` is called on the UI thread
/// with the result value.
///
/// # Example
/// ```rust
/// use rvue::async_runtime::spawn_task_with_result;
///
/// let handle = spawn_task_with_result(
///     async { fetch_user(42).await },
///     |user| {
///         set_user(user);
///     },
/// );
/// ```
pub fn spawn_task_with_result<F, T, C>(future: F, on_complete: C) -> TaskHandle
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
    C: FnOnce(T) + Send + 'static;
```

**Contract**:
- MUST deliver the result via `dispatch_to_ui()`
- MUST NOT call `on_complete` if the task is aborted before completion
- MUST mark the task as completed before dispatching `on_complete`

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
/// **For Signal Watching**: Use `spawn_watch_signal()` instead.
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
/// on_cleanup(move || handle.abort());
/// ```
pub fn spawn_interval<F, Fut>(period: Duration, f: F) -> TaskHandle
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST execute `f` immediately, then every `period`
- MUST stop when `TaskHandle::abort()` is called
- MUST be registered with `TaskRegistry` if in component context
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
/// **For Signal Watching**: Use `spawn_watch_signal()` instead.
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
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static;
```

**Contract**:
- MUST only execute `handler` after `delay` has elapsed since last `call()`
- MUST cancel previous pending execution when `call()` is invoked
- MUST use the value from the most recent `call()`
- `DebouncedTask::cancel()` MUST stop all pending and future executions
- **GC Safety**: `T` must be `Send`; handler closure must not capture `Gc<T>`

---

### `spawn_watch_signal`

```rust
/// Spawn a task that watches a signal and automatically dispatches updates to UI.
///
/// The signal is polled at the given interval. On each poll, the callback
/// receives the current signal value. The callback can return `Some(v)` to
/// update the signal, or `None` to just observe.
///
/// # GC Safety
/// **SAFE**: This function is designed for signal watching and handles GC
/// internally by extracting the signal value before async operations.
///
/// # Example
/// ```rust
/// use rvue::prelude::*;
/// use rvue::async_runtime::spawn_watch_signal;
/// use std::time::Duration;
///
/// #[component]
/// fn LiveStatus() -> View {
///     let (status, set_status) = create_signal(String::from("Checking..."));
///
///     // Watch signal, automatically dispatch to UI
///     let watcher = spawn_watch_signal(
///         status,
///         Duration::from_secs(10),
///         |current| {
///             println!("Status changed to: {}", current);
///             None  // Just watching, not updating
///         }
///     );
///
///     on_cleanup(move || watcher.stop());
///
///     // Changing status triggers the watcher
///     set_status(String::from("Online"));
///
///     view! {
///         <Text value=status.get() />
///     }
/// }
/// ```
///
/// # Arguments
/// - `signal`: The signal to watch
/// - `period`: How often to poll the signal
/// - `callback`: Called with current value; return `Some(v)` to update, `None` to just watch
///
/// # Returns
/// A `SignalWatcher<T>` handle for stopping the watcher.
pub fn spawn_watch_signal<T, F>(
    signal: ReadSignal<T>,
    period: Duration,
    callback: F,
) -> SignalWatcher<T>
where
    T: Trace + Clone + Send + 'static,
    F: FnMut(T) -> Option<T> + Send + 'static;
```

**Contract**:
- MUST poll the signal at the specified interval
- MUST execute callback on each poll with current signal value
- MUST dispatch `Some(v)` to UI thread to update signal
- MUST return `SignalWatcher<T>` that can stop the watcher
- **GC Safety**: Signal value is extracted and cloned, no `Gc<T>` crosses thread boundary

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
    Fu: Future<Output = Result<T, String>> + Send + 'static,
    Fetcher: Fn(T) -> Fu + Send + Sync + 'static;
```

**Contract**:
- MUST trigger initial fetch on creation
- MUST transition through `Pending → Loading → Ready/Error`
- `Resource::get()` MUST be reactive (tracked by effects)
- `Resource::refetch()` MUST trigger a new fetch cycle (`Loading → Ready/Error`)
- MUST cancel in-flight fetch on refetch (only latest result applies)
- If component unmounts, in-flight fetches MUST be cancelled

---

## Types

### `TaskHandle`

```rust
#[derive(Debug, Clone)]
pub struct TaskHandle { /* private fields */ }

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
pub struct TaskId(u64);
```

### `SignalSender<T>`

```rust
#[derive(Clone)]
pub struct SignalSender<T: Clone + Send + 'static> { /* private fields */ }

impl<T: Clone + Send + 'static> SignalSender<T> {
    /// Queue a value to be set on the signal via dispatch_to_ui.
    ///
    /// Safe to call from any thread.
    pub fn set(&self, value: T);
}

// SAFETY: Only holds Arc<dyn Fn + Send + Sync>
unsafe impl<T: Clone + Send + 'static> Send for SignalSender<T> {}
unsafe impl<T: Clone + Send + 'static> Sync for SignalSender<T> {}
```

### `WriteSignal<T>` Extension

```rust
impl<T: Trace + Clone + 'static> WriteSignal<T> {
    /// Create a thread-safe sender for updating this signal from async tasks.
    ///
    /// Must be called on the UI thread.
    #[cfg(feature = "async")]
    pub fn sender(&self) -> SignalSender<T>
    where
        T: Send;
}
```

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
pub struct DebouncedTask<T: Send + 'static> { /* private fields */ }

impl<T: Send + 'static> DebouncedTask<T> {
    /// Submit a value. Resets the debounce timer.
    pub fn call(&self, value: T);

    /// Cancel the debounce task entirely.
    pub fn cancel(&self);
}
```

---

### `SignalWatcher<T>`

```rust
/// Handle for a signal watching task.
///
/// Created by `spawn_watch_signal()`.
#[derive(Clone)]
pub struct SignalWatcher<T: Send + 'static> { /* private fields */ }

impl<T: Send + 'static> SignalWatcher<T> {
    /// Stop watching the signal.
    ///
    /// The watcher will stop polling after this is called.
    /// It is safe to call multiple times.
    pub fn stop(&self);
}
```

---

## Cargo Feature

```toml
# crates/rvue/Cargo.toml
[features]
default = []
async = ["dep:tokio", "dep:parking_lot"]

[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time"], optional = true }
parking_lot = { version = "0.12", optional = true }
```

**Contract**:
- Without `async` feature: no tokio dependency, no async_runtime module compiled
- All async types and functions gated behind `#[cfg(feature = "async")]`
- `WriteSignal::sender()` gated behind `#[cfg(feature = "async")]`

---

## Error Behavior

| Scenario | Behavior |
|----------|----------|
| `dispatch_to_ui` callback panics | Caught by `catch_unwind`, logged, other callbacks continue |
| Async task panics | tokio catches it; task marked as completed |
| `spawn_task` called before `run_app` | Runtime initializes; callbacks queue until event loop starts |
| `set()` on sender after signal dropped | No-op (closure captures signal; if signal GC'd, dispatch is harmless) |
| `create_resource` fetcher returns `Err` | State transitions to `ResourceState::Error(msg)` |
| `spawn_debounced` with zero delay | Executes immediately (same as no debounce) |
| Task completes after component unmount | `dispatch_to_ui` callback is still executed (signal may be gone — no-op) |
