# Data Model: Rvue Async Runtime Support

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07

---

## Entity Relationship Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                     UI Thread (main)                         │
│                                                              │
│  ┌──────────────┐     ┌──────────────────┐                  │
│  │  Component    │────▶│  TaskRegistry    │                  │
│  │ (ComponentId) │     │ (ComponentId →   │                  │
│  └──────┬───────┘     │  Vec<TaskHandle>)│                  │
│         │              └────────┬─────────┘                  │
│         │ unmount()             │ cancel_all()               │
│         │                      │                             │
│         ▼                      ▼                             │
│  ┌──────────────┐     ┌──────────────┐                      │
│  │ WriteSignal  │     │  TaskHandle  │                      │
│  │ .sender()    │     │  (TaskId,    │                      │
│  └──────┬───────┘     │   AbortHandle│                      │
│         │              │   completed) │                      │
│         │              └──────────────┘                      │
│         ▼                      ▲                             │
│  ┌──────────────┐              │                             │
│  │SignalSender  │──────────────┘ (dispatch_to_ui)            │
│  │ (Send+Sync)  │                                           │
│  └──────┬───────┘                                           │
│         │                                                    │
│  ┌──────┴───────┐     ┌──────────────┐                      │
│  │UiDispatch    │     │  Resource<T> │                      │
│  │   Queue      │     │ (state,      │                      │
│  │(VecDeque<Cb>)│     │  refetch)    │                      │
│  └──────────────┘     └──────┬───────┘                      │
│         ▲                    │                               │
│         │                    ▼                               │
│         │             ┌──────────────┐                      │
│         │             │ResourceState │                      │
│         │             │{Pending,     │                      │
│         │             │ Loading,     │                      │
│         │             │ Ready(T),    │                      │
│         │             │ Error(String)│                      │
│         │             └──────────────┘                      │
└─────────│──────────────────────────────────────────────────┘
          │
          │ dispatch_to_ui()
          │
┌─────────┴──────────────────────────────────────────────────┐
│                   tokio Worker Threads                       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ spawn_task() │  │ DebouncedTask│  │ spawn_interval() │  │
│  │ (Future→())  │  │ (mpsc + delay│  │ (tokio::interval)│  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## Entities

### 1. TaskId

**Location**: `crates/rvue/src/async_runtime/task.rs`

| Field | Type | Description |
|-------|------|-------------|
| `0` | `u64` | Monotonically increasing unique ID |

**Constraints**:
- Generated via `AtomicU64::fetch_add(1, Ordering::Relaxed)`
- Unique across the process lifetime
- Wraps at `u64::MAX` (not a practical concern)

**Derives**: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

---

### 2. TaskHandle

**Location**: `crates/rvue/src/async_runtime/task.rs`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `TaskId` | Unique task identifier |
| `abort_handle` | `tokio::task::AbortHandle` | Handle to abort the tokio task |
| `completed` | `Arc<AtomicBool>` | Whether the task has finished |

**Derives**: `Debug`, `Clone`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `abort` | `&self → ()` | Cancel the task (idempotent) |
| `is_completed` | `&self → bool` | Check if task finished |
| `is_running` | `&self → bool` | Check if task is still active |
| `id` | `&self → TaskId` | Get the task's unique ID |

**State Transitions**:
```text
Created ──▶ Running ──▶ Completed
                │
                └──▶ Aborted (via abort())
```

**Invariants**:
- `abort()` is safe to call multiple times
- `is_running()` returns `false` after `abort()` or completion
- Dropping `TaskHandle` does NOT cancel the task (fire-and-forget by default)

---

### 3. TaskRegistry

**Location**: `crates/rvue/src/async_runtime/registry.rs`

| Field | Type | Description |
|-------|------|-------------|
| `tasks` | `RwLock<HashMap<ComponentId, Vec<TaskHandle>>>` | Component → tasks mapping |

**Singleton**: Global `static` via `OnceLock`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `register` | `(ComponentId, TaskHandle) → ()` | Associate task with component |
| `cancel_all` | `(ComponentId) → ()` | Abort + remove all tasks for component |
| `cleanup_completed` | `() → ()` | Remove handles for finished tasks |
| `task_count` | `(ComponentId) → usize` | Count active tasks for component |

**Invariants**:
- `cancel_all()` removes the entire entry for the component (no stale data)
- `cleanup_completed()` retains only `is_running()` handles
- Write lock is held only during mutation; read lock for queries

---

### 4. UiDispatchQueue

**Location**: `crates/rvue/src/async_runtime/dispatch.rs`

| Field | Type | Description |
|-------|------|-------------|
| `queue` | `Mutex<VecDeque<UiCallback>>` | FIFO queue of closures |
| `proxy` | `OnceLock<EventLoopProxy<RvueUserEvent>>` | Wakeup handle |

**Type Alias**: `type UiCallback = Box<dyn FnOnce() + Send + 'static>`

**Singleton**: Global `static`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `dispatch` | `(impl FnOnce() + Send + 'static) → ()` | Enqueue callback + wake event loop |
| `drain_and_execute` | `() → ()` | Pop all callbacks and run them (UI thread only) |
| `set_proxy` | `(EventLoopProxy<RvueUserEvent>) → ()` | Initialize the wakeup proxy |
| `len` | `() → usize` | Pending callback count |
| `is_empty` | `() → bool` | Check if queue is empty |

**Invariants**:
- `dispatch()` is safe to call from any thread
- `drain_and_execute()` MUST only be called on the UI thread
- Callbacks execute in FIFO order
- Panicking callbacks are caught and logged (don't crash UI)

---

### 5. SignalSender\<T\>

**Location**: `crates/rvue/src/async_runtime/signal_sender.rs`

| Field | Type | Description |
|-------|------|-------------|
| `dispatch_fn` | `Arc<dyn Fn(T) + Send + Sync>` | Closure that dispatches set() to UI thread |

**Bounds**: `T: Clone + Send + 'static` (note: does NOT require `T: Trace` since the value is cloned across the thread boundary, not the signal itself)

**Derives**: `Clone`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `set` | `(T) → ()` | Queue a signal update on the UI thread |

**Invariants**:
- `set()` is safe to call from any thread
- The actual `WriteSignal::set()` runs on the UI thread via `dispatch_to_ui()`
- If the component/signal has been dropped, the dispatch is a no-op (closure captures weak ref or signal checks validity)

**Creation**: Via `WriteSignal::sender()` — must be called on the UI thread

---

### 6. ResourceState\<T\>

**Location**: `crates/rvue/src/async_runtime/resource.rs`

| Variant | Data | Description |
|---------|------|-------------|
| `Pending` | — | Initial state, fetch not yet triggered |
| `Loading` | — | Fetch in progress |
| `Ready` | `T` | Data successfully loaded |
| `Error` | `String` | Fetch failed with error message |

**Derives**: `Clone`, `Debug` (where `T: Debug`)

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_loading` | `&self → bool` | Check if in Loading state |
| `is_ready` | `&self → bool` | Check if in Ready state |
| `is_error` | `&self → bool` | Check if in Error state |
| `data` | `&self → Option<&T>` | Extract data if Ready |
| `error` | `&self → Option<&str>` | Extract error message if Error |

**State Transitions**:
```text
Pending ──▶ Loading ──▶ Ready(T)
                │           │
                │           ▼ (refetch)
                │       Loading ──▶ Ready(T')
                │                       │
                └──▶ Error(String)      └──▶ Error(String)
                        │
                        ▼ (refetch)
                    Loading ──▶ ...
```

---

### 7. Resource\<T\>

**Location**: `crates/rvue/src/async_runtime/resource.rs`

| Field | Type | Description |
|-------|------|-------------|
| `state` | `ReadSignal<ResourceState<T>>` | Reactive state observable by UI |
| `refetch` | `Arc<dyn Fn() + Send + Sync>` | Trigger to re-run the fetcher |

**Bounds**: `T: Trace + Clone + Send + 'static`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `get` | `() → ResourceState<T>` | Read current state (reactive — tracks dependencies) |
| `refetch` | `() → ()` | Manually trigger a new fetch |

**Creation**: Via `create_resource(source, fetcher)` — must be called on the UI thread within component context

---

### 8. DebouncedTask\<T\>

**Location**: `crates/rvue/src/async_runtime/task.rs`

| Field | Type | Description |
|-------|------|-------------|
| `sender` | `mpsc::UnboundedSender<T>` | Channel to submit new values |
| `handle` | `TaskHandle` | Handle to the background debounce task |

**Bounds**: `T: Send + 'static`

**Methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `call` | `(T) → ()` | Submit a value (resets debounce timer) |
| `cancel` | `() → ()` | Stop the debounce task entirely |

---

### 9. RvueUserEvent

**Location**: `crates/rvue/src/app.rs`

| Variant | Description |
|---------|-------------|
| `AsyncDispatchReady` | Async callback enqueued, wake event loop to process (cfg `async`) |

**Derives**: `Debug`, `Clone`

---

## Relationship Summary

| Source | Relationship | Target | Cardinality |
|--------|-------------|--------|-------------|
| Component | registers tasks via | TaskRegistry | 1:N |
| Component | cancels tasks on | unmount() | 1:N |
| TaskRegistry | stores | TaskHandle | N:M |
| WriteSignal | creates | SignalSender | 1:1 |
| SignalSender | dispatches via | UiDispatchQueue | N:1 |
| Resource | owns | ReadSignal\<ResourceState\> | 1:1 |
| Resource | spawns via | spawn_task() | 1:N |
| spawn_task() | returns | TaskHandle | 1:1 |
| DebouncedTask | wraps | TaskHandle | 1:1 |
| UiDispatchQueue | wakes via | EventLoopProxy | 1:1 |

---

## Validation Rules

| Entity | Rule | Enforcement |
|--------|------|-------------|
| TaskId | Unique across process | AtomicU64 counter |
| UiDispatchQueue | drain_and_execute only on UI thread | Documentation + debug_assert |
| SignalSender | Created on UI thread only | WriteSignal::sender() requires UI context |
| Resource | Created within component context | Panics if no current_owner() |
| TaskRegistry | cancel_all idempotent | HashMap::remove returns None on second call |
| DebouncedTask | Delay must be positive | Checked at creation time |
