# Research: Rvue Async Runtime Support

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07

This document resolves all technical unknowns identified during the planning phase.

---

## R1: tokio Runtime Lifecycle Management

**Question**: How should the tokio runtime be created and managed alongside winit's event loop?

**Context**: winit owns the main thread via `EventLoop::run_app()`. tokio's multi-thread runtime spawns its own worker threads. Both cannot own the main thread simultaneously.

### Decision: Lazy initialization with `OnceLock`

Create the tokio runtime lazily on the first call to `spawn_task()`. Store it in a `static OnceLock<tokio::runtime::Runtime>`. The runtime runs worker threads in the background; the main thread remains under winit's control.

```rust
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_or_init_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("rvue-async")
            .build()
            .expect("Failed to create tokio runtime")
    })
}
```

### Rationale

- **Zero cost when unused**: If the app never calls `spawn_task()`, no runtime is created.
- **No user ceremony**: No need for `#[tokio::main]` or manual runtime setup.
- **Thread safety**: `OnceLock` is `Send + Sync` and initializes exactly once.
- **Worker count**: 2 workers is sufficient for GUI async tasks (I/O-bound, not CPU-bound). Users can configure via environment variable `TOKIO_WORKER_THREADS` if needed.

### Alternatives Rejected

| Alternative | Reason Rejected |
|-------------|-----------------|
| User creates runtime | Adds ceremony; most users want "it just works" |
| Runtime in `run_app()` | Creates runtime even if async never used; wastes resources |
| Single-threaded runtime | Cannot run blocking I/O without starving the executor |
| `current_thread` runtime | Would need integration with winit's event loop; complex and fragile |

### Shutdown

The runtime is leaked intentionally (static lifetime). On process exit, the OS reclaims all resources. For graceful shutdown during tests, `spawn_task` uses `AbortHandle` which works without runtime shutdown.

---

## R2: UI Thread Wakeup Mechanism

**Question**: How do async tasks signal the UI thread to process dispatched callbacks?

**Context**: winit's event loop blocks waiting for OS events. Without an explicit wakeup, dispatched callbacks would only run when the next user interaction (mouse move, key press) triggers a frame.

### Decision: winit `EventLoopProxy` with custom user event

Use `EventLoopProxy::send_event()` to wake the event loop from any thread. This triggers `ApplicationHandler::user_event()` where we drain the dispatch queue.

```rust
/// User events sent to the winit event loop
#[derive(Debug, Clone)]
pub enum RvueUserEvent {
    /// Async task dispatched a callback to the UI thread
    AsyncDispatchReady,
}
```

The `EventLoopProxy` is stored in a global `OnceLock` initialized during `run_app()`, before the event loop starts. `dispatch_to_ui()` calls `proxy.send_event(RvueUserEvent::AsyncDispatchReady)` after enqueuing the callback.

### Rationale

- **Immediate response**: UI processes callbacks within the same frame as the wakeup, not on next user interaction.
- **winit-native**: Uses the intended mechanism for cross-thread wakeup.
- **No polling**: No periodic timer needed to check the queue.
- **Low overhead**: `send_event()` is a single syscall (eventfd on Linux, mach port on macOS).

### Alternatives Rejected

| Alternative | Reason Rejected |
|-------------|-----------------|
| Polling in `run_update_passes()` | Only runs when there are already events; adds latency |
| `window.request_redraw()` from async | Requires `Arc<Window>` to be Send; couples rendering to dispatch |
| Separate OS-level pipe/channel | Reinvents what `EventLoopProxy` already provides |
| Timer-based polling (16ms interval) | Adds 0-16ms latency; wastes CPU when idle |

### Integration Point

```rust
// In app.rs, handle the user event:
fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: RvueUserEvent) {
    match event {
        RvueUserEvent::AsyncDispatchReady => {
            crate::async_runtime::dispatch::drain_ui_callbacks();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}
```

---

## R3: Signal Thread Safety — SignalSender Design

**Question**: How do async tasks update signals that are `!Send` (because they use `Gc<T>`)?

**Context**: `ReadSignal<T>` and `WriteSignal<T>` wrap `Gc<SignalDataInner<T>>`. `Gc<T>` is `!Send + !Sync` because it participates in thread-local GC. Async tasks run on tokio worker threads and cannot hold signal references.

### Decision: `SignalSender<T>` — a closure-capture proxy

`SignalSender<T>` captures the signal's identity (an opaque ID) and uses `dispatch_to_ui()` to schedule the actual `set()` call on the UI thread where the signal lives.

```rust
#[derive(Clone)]
pub struct SignalSender<T: Clone + Send + 'static> {
    dispatch_fn: Arc<dyn Fn(T) + Send + Sync + 'static>,
}
```

The `dispatch_fn` is created at `.sender()` call time on the UI thread, closing over the `WriteSignal`. When called from an async task, it enqueues a `dispatch_to_ui()` closure.

### Rationale

- **No unsafe**: Signal stays on UI thread; only the value `T` crosses thread boundary.
- **No registry lookup**: The closure already captures the signal — no ID-based lookup needed at dispatch time.
- **Simple API**: `let sender = set_count.sender(); sender.set(42);`
- **Deduplication**: Latest-value-wins semantics via `Arc<Mutex<Option<T>>>` pending slot.

### Key Constraint: `T: Trace + Clone + Send + 'static`

Existing signals require `T: Trace + Clone + 'static`. `SignalSender` adds `Send` to allow the value to cross thread boundaries. This means not all signal types can create senders — only those with `Send` values. This is an acceptable limitation since most data types (primitives, String, Vec, etc.) are `Send`.

### Alternatives Rejected

| Alternative | Reason Rejected |
|-------------|-----------------|
| Make signals `Send+Sync` | Would require replacing `Gc<T>` with `Arc<T>`; massive refactor; GC integration lost |
| Global signal registry by ID | Runtime overhead for every signal; adds complexity; type erasure problems |
| Channel-per-signal | Memory overhead; hard to manage lifecycle |

---

## R4: GC Safety Across `.await` Points

**Question**: How should async tasks safely interact with garbage-collected objects?

**Context**: rudo-gc's `Gc<T>` pointers can be collected if not reachable from a root set. Across `.await` points, the GC may run on the UI thread while the async task is suspended.

### Decision: Three-tier safety model

| Tier | When to Use | Mechanism |
|------|-------------|-----------|
| **Tier 1: No GC objects** | Most common — fetch data, return value | Plain `spawn_task()` with `dispatch_to_ui()` |
| **Tier 2: GC objects owned** | Need to read GC data before spawning | Clone/extract data before spawn, send plain values |
| **Tier 3: GC objects across await** | Rare — need GC access during async work | `rudo_gc::spawn_with_gc!` or `AsyncHandleScope` |

### Rationale

- **Tier 1 covers 90%+ of use cases**: Fetch data → dispatch result to UI. No GC objects needed in the async task.
- **Tier 2 is natural Rust**: Clone the data you need before moving into the async block.
- **Tier 3 uses battle-tested rudo-gc primitives**: `AsyncHandleScope` and `spawn_with_gc!` are already implemented and tested in rudo-gc.

### Documentation Strategy

The quickstart guide will lead with Tier 1 (simplest). Tier 2 and 3 will be in an "Advanced" section. This prevents users from reaching for complex patterns when simple ones suffice.

---

## R5: parking_lot vs std::sync::Mutex

**Question**: Should we add `parking_lot` as a new dependency for the dispatch queue mutex?

**Context**: The dispatch queue mutex is locked briefly (enqueue/drain) but from multiple threads. `parking_lot::Mutex` is faster for uncontended locks and doesn't support poisoning.

### Decision: Use `parking_lot::Mutex`

Add `parking_lot = "0.12"` as an optional dependency gated behind the `async` feature.

### Rationale

- **Performance**: parking_lot is ~2x faster for uncontended lock/unlock (no syscall in fast path).
- **No poisoning**: `dispatch_to_ui()` closures should not panic; if they do, poisoning would break the queue permanently. parking_lot's no-poison behavior is more resilient.
- **Ecosystem standard**: parking_lot is used by rudo-gc internally and is a near-universal Rust dependency.
- **Feature-gated**: Only pulled in when `async` feature is enabled, so sync-only apps are unaffected.

### Alternatives Rejected

| Alternative | Reason Rejected |
|-------------|-----------------|
| `std::sync::Mutex` | Slightly slower; poisoning semantics add complexity for no benefit |
| Lock-free queue (crossbeam) | Over-engineered for this use case; adds another dependency |
| `tokio::sync::Mutex` | Async mutex is unnecessary — lock is held for microseconds |

---

## R6: Error Handling in Async Tasks

**Question**: How should panics and errors in async tasks be handled without crashing the UI?

**Context**: FR-012 requires that errors from async tasks don't crash the UI thread. tokio tasks that panic are caught by the runtime (task is cancelled, `JoinError` returned). But `dispatch_to_ui()` closures run on the UI thread.

### Decision: Catch panics at the dispatch boundary

```rust
pub(crate) fn drain_ui_callbacks() {
    let callbacks: Vec<_> = {
        let mut queue = DISPATCH_QUEUE.lock();
        queue.drain(..).collect()
    };

    for callback in callbacks {
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)) {
            log::error!("Panic in dispatch_to_ui callback: {:?}", panic);
            // Continue processing remaining callbacks
        }
    }
}
```

### Rationale

- **Resilience**: One bad callback doesn't kill the entire application.
- **Logging**: Errors are logged for debugging.
- **Isolation**: Each callback is isolated; panics don't propagate to other callbacks.
- **ResourceState::Error**: For `create_resource()`, fetch errors are captured in `ResourceState::Error(String)` — no panics needed for expected failures.

### Task-level errors

For `spawn_task_with_result()`, the async task wraps its future in `catch_unwind` so that panics produce an error result rather than crashing the tokio worker.

---

## R7: EventLoop Type Parameter

**Question**: winit 0.30's `EventLoop::with_user_event()` requires a type parameter. How do we define the user event type?

**Context**: Currently `EventLoop::with_user_event()` is called but no user event type is explicitly used (it's `()` by default in winit 0.30). We need to introduce `RvueUserEvent` as the user event type.

### Decision: Define `RvueUserEvent` enum in `app.rs`

```rust
#[derive(Debug, Clone)]
pub enum RvueUserEvent {
    /// Wake the event loop to process async dispatch callbacks
    #[cfg(feature = "async")]
    AsyncDispatchReady,
}
```

The event loop becomes `EventLoop::<RvueUserEvent>::with_user_event()` and `AppState` stores an `Option<EventLoopProxy<RvueUserEvent>>` for async dispatch.

### Rationale

- **Extensible**: Enum can grow for future user event types (timers, hot reload, etc.).
- **Feature-gated variant**: The `AsyncDispatchReady` variant only exists with the `async` feature.
- **Backward compatible**: Without the `async` feature, the enum is empty or has a placeholder variant.

---

## R8: Component ID for Task Registry

**Question**: How does the `TaskRegistry` identify components? What is a component's "ID"?

**Context**: The plan proposes `TaskRegistry` mapping component IDs to task handles. Components have a `ComponentId` type.

### Decision: Use existing `ComponentId`

```rust
// Already exists in component.rs:
pub struct ComponentId(usize);
```

`ComponentId` is a unique, monotonically increasing integer assigned at component creation. It's already used throughout the codebase for identity. The `TaskRegistry` maps `ComponentId → Vec<TaskHandle>`.

### Integration

When `spawn_task()` is called within a component context (detected via `runtime::current_owner()`), the task is automatically registered with that component's ID. When the component unmounts, `TaskRegistry::cancel_all(component_id)` is called.

---

## Summary of Decisions

| # | Topic | Decision | Key Rationale |
|---|-------|----------|---------------|
| R1 | Runtime lifecycle | Lazy `OnceLock<Runtime>` on first spawn | Zero cost when unused |
| R2 | UI wakeup | `EventLoopProxy` + `RvueUserEvent` | Immediate, no polling |
| R3 | Signal bridging | `SignalSender<T>` closure proxy | No unsafe, no registry |
| R4 | GC safety | Three-tier model (plain → clone → AsyncHandleScope) | Progressive complexity |
| R5 | Mutex choice | `parking_lot::Mutex` (feature-gated) | Faster, no poisoning |
| R6 | Error handling | `catch_unwind` at dispatch boundary | Resilient UI thread |
| R7 | Event type | `RvueUserEvent` enum | Extensible, feature-gated |
| R8 | Component ID | Existing `ComponentId` | Already unique and used |
