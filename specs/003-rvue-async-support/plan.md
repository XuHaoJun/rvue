# Implementation Plan: Rvue Async Runtime Support

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07
**Updated**: 2026-02-08 (aligned with implementation)

## Summary

Add async runtime support to the Rvue GUI framework so that applications can perform non-blocking I/O, computation, and data fetching without freezing the UI. The approach integrates tokio as a feature-gated dependency, uses winit's `EventLoopProxy` to wake the UI thread from async tasks, and bridges the reactive signal system with rudo-gc's `AsyncHandleScope` for thread-safe GC access.

## Technical Context

**Language/Version**: Rust 2021 Edition, minimum 1.84+ (workspace `rust-version = "1.84"`)
**Primary Dependencies**: rudo-gc 0.8 (GC), vello 0.7 (rendering), taffy 0.9 (layout), winit 0.30 (event loop)
**New Dependencies**: tokio 1 (async runtime, feature-gated)
**Storage**: N/A (desktop GUI framework)
**Testing**: `cargo test -- --test-threads=1` (single-threaded for GC determinism)
**Target Platform**: Desktop (Linux, macOS, Windows) — WASM explicitly out of scope
**Performance Goals**: 60 FPS = 16ms frame budget; async dispatch overhead < 1ms per frame
**Constraints**: All signal/effect operations are !Send (use `Gc<T>`/`GcCell<T>`); UI thread is winit's main thread; rudo-gc is thread-local
**Scale/Scope**: Framework library; ~15k LOC in `crates/rvue/src/`

### Key Existing Infrastructure

| Asset | Location | Relevance |
|-------|----------|-----------|
| `on_cleanup()` hook | `effect.rs` | Already exists — registers cleanup with current effect/component |
| `Component.cleanups` field | `component.rs` | Already stores `Vec<Box<dyn FnOnce()>>`, runs on `unmount()` |
| `Component.unmount()` | `component.rs` | Already recurses children + runs cleanups |
| `OWNER_STACK` | `runtime.rs` | Thread-local component context stack |
| `EventLoop::with_user_event()` | `app.rs` | winit event loop already configured for user events |

### Key Technical Decisions

1. **tokio integration**: Lazy-init a multi-thread runtime on first `spawn_task()` call; feature-gated behind `async` feature
2. **UI wakeup**: Use winit `EventLoopProxy` to send user events that trigger `drain_ui_callbacks()` — avoids polling
3. **GC safety**: Use `rudo_gc::handles::AsyncHandleScope` for GC objects across `.await` points
4. **Signal bridging**: `AsyncSignalSender<T>` that holds `Gc<SignalDataInner<T>>` for thread-safe signal access
5. **Component tracking**: `ComponentScope` with `Rc<RefCell<GcScope>>` for dynamic component tracking in async contexts

---

## Project Structure

### Documentation (this feature)

```
specs/003-rvue-async-support/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── async-api.md     # Public API contract
└── spec.md             # Feature specification
```

### Source Code (repository root)

```
crates/rvue/src/
├── async_runtime/            # Async runtime module
│   ├── mod.rs                # Module root, re-exports, feature gate
│   ├── dispatch.rs           # UiDispatchQueue + dispatch_to_ui()
│   ├── task.rs               # TaskHandle + spawn_task() + watch_signal()
│   ├── registry.rs           # TaskRegistry (placeholder, not used)
│   ├── signal_sender.rs      # DELETED - merged into task.rs
│   ├── component_scope.rs    # ComponentScope for async component tracking
│   └── resource.rs           # create_resource() + ResourceState<T>
├── app.rs                    # MODIFIED: tokio init, EventLoopProxy, drain dispatch queue
├── component.rs              # Minimal changes
├── signal.rs                 # MODIFIED: SignalDataInner made public for async access
├── lib.rs                    # MODIFIED: add async_runtime module + prelude exports
└── ...existing files...

crates/rvue/Cargo.toml        # MODIFIED: add tokio feature flag
```

**Structure Decision**: Single crate extension. The async runtime is a new module within `crates/rvue/src/` rather than a separate crate, because it deeply integrates with existing component lifecycle, signals, and the app event loop. Feature-gated via `#[cfg(feature = "async")]`.

---

## API Summary

### Core Functions

| Function | Description |
|----------|-------------|
| `dispatch_to_ui<F>` | Execute closure on UI thread from any thread |
| `spawn_task<F>` | Spawn async task on tokio, returns `TaskHandle` |
| `spawn_interval<F>` | Recurring task at fixed interval, returns `IntervalHandle` |
| `spawn_debounced<T>` | Debounced async operation, returns `DebouncedTask<T>` |
| `watch_signal<T>` | Async signal watching with callbacks |
| `create_resource<S,T>` | Reactive async data fetching |

### Types

| Type | Description |
|------|-------------|
| `TaskHandle` | Manage spawned tasks (abort, query status) |
| `TaskId` | Unique task identifier |
| `AsyncSignalSender<T>` | Thread-safe signal update sender |
| `SignalWatcher` | Handle for stopping signal watcher |
| `IntervalHandle` | Handle for stopping interval tasks |
| `DebouncedTask<T>` | Handle for debounced operations |
| `ComponentScope` | Dynamic component tracking for async |
| `Resource<T>` | Reactive async resource |
| `ResourceState<T>` | State enum (Pending, Loading, Ready, Error) |

### Extension Traits

| Trait | Method | Description |
|-------|--------|-------------|
| `WriteSignalExt<T>` | `sender()` | Create `AsyncSignalSender<T>` |

---

## Differences from Original Plan

### Removed APIs

1. **`spawn_task_with_result`** - Removed due to `Send` bound issues with `SignalDataInner` (contains non-Send types like `GcRwLock`)
2. **`SignalSender`** - Renamed to `AsyncSignalSender` for clarity
3. **`spawn_watch_signal`** - Renamed to `watch_signal` (async function)
4. **`TaskRegistry`** - Lifecycle binding via `on_cleanup` instead of separate registry

### Modified APIs

1. **`WriteSignal::sender()`** - Now via `WriteSignalExt` trait for feature-gating
2. **`watch_signal`** - Now async function taking both read_signal and write_signal

### Added APIs

1. **`ComponentScope`** - Dynamic component tracking using `GcScope` for async operations

---

## GC Safety Implementation

### `watch_signal` Pattern

```rust
pub async fn watch_signal<T>(
    read_signal: ReadSignal<T>,
    write_signal: WriteSignal<T>,
    period: Duration,
    mut callback: impl FnMut(T) -> Option<T> + Send + Sync + 'static,
) -> SignalWatcher
where
    T: Trace + Clone + Send + Sync + 'static,
{
    let tcb = rudo_gc::heap::current_thread_control_block()
        .expect("watch_signal requires GC thread");

    let scope = Arc::new(AsyncHandleScope::new(&tcb));
    let read_handle = scope.handle(&read_signal.data);
    let write_handle = scope.handle(&write_signal.data);

    // ... spawn tokio task with handles
}
```

### `AsyncSignalSender` Pattern

```rust
pub struct AsyncSignalSender<T: Trace + Clone + 'static> {
    data: Gc<SignalDataInner<T>>,
}

impl<T: Trace + Clone + 'static> AsyncSignalSender<T> {
    pub async fn set(&self, value: T) {
        *self.data.inner.value.write() = value;
        self.data.inner.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }
}
```

---

## Build & Test Commands

```bash
# Build with async feature
cargo build --features async

# Run tests (single-threaded for GC determinism)
cargo test --features async -- --test-threads=1

# Run linting
cargo clippy --features async
cargo fmt
```

---

## Status

| Phase | Status | Notes |
|-------|--------|-------|
| Implementation | ✅ Complete | All core APIs implemented |
| Build | ✅ Passing | `cargo build --features async` |
| Tests | ✅ Passing | 453 tests including async GC safety test |
| Spec Alignment | ✅ Updated | async-api.md updated to match implementation |
