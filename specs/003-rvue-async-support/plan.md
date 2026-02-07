# Implementation Plan: Rvue Async Runtime Support

**Branch**: `003-rvue-async-support` | **Date**: 2026-02-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-rvue-async-support/spec.md` and design from `/docs/async-plan-v1.md`

## Summary

Add async runtime support to the Rvue GUI framework so that applications can perform non-blocking I/O, computation, and data fetching without freezing the UI. The approach integrates tokio as a feature-gated dependency, uses winit's `EventLoopProxy` to wake the UI thread from async tasks, and bridges the reactive signal system with a thread-safe `SignalSender` proxy. Task lifecycle is bound to component mount/unmount via an existing cleanup infrastructure.

## Technical Context

**Language/Version**: Rust 2021 Edition, minimum 1.84+ (workspace `rust-version = "1.84"`)
**Primary Dependencies**: rudo-gc 0.7 (GC), vello 0.7 (rendering), taffy 0.9 (layout), winit 0.30 (event loop), once_cell 1 (lazy statics)
**New Dependencies**: tokio 1 (async runtime, feature-gated), parking_lot 0.12 (efficient Mutex for dispatch queue)
**Storage**: N/A (desktop GUI framework)
**Testing**: `cargo test -- --test-threads=1` (single-threaded for GC determinism)
**Target Platform**: Desktop (Linux, macOS, Windows) — WASM explicitly out of scope
**Project Type**: Cargo workspace with multiple crates (`rvue`, `rvue-signals`, `rvue-style`, `rvue-macro`, etc.)
**Performance Goals**: 60 FPS = 16ms frame budget; async dispatch overhead < 1ms per frame; GC pauses within budget
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
| `once_cell` | `Cargo.toml` | Already a dependency |

### Key Technical Decisions

1. **tokio integration**: Lazy-init a multi-thread runtime on first `spawn_task()` call; feature-gated behind `async` feature
2. **UI wakeup**: Use winit `EventLoopProxy` to send user events that trigger `drain_ui_callbacks()` — avoids polling
3. **Signal bridging**: `SignalSender<T>` proxy that captures a closure over `dispatch_to_ui()` — signals remain !Send
4. **GC safety**: Use `rudo_gc::AsyncHandleScope` / `spawn_with_gc!` for GC objects across `.await` points
5. **Mutex choice**: `parking_lot::Mutex` for dispatch queue hot path (no poisoning, faster uncontended)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality | PASS | Follows Rust conventions; public APIs documented; max 100 char lines; Result/Option for fallible ops |
| II. Testing Standards | PASS | Unit tests in `tests/` per crate; integration tests planned for lifecycle; `--test-threads=1` respected |
| III. User Experience Consistency | PASS | API follows Vue patterns (`create_signal` → `create_resource`); snake_case functions, PascalCase types; prelude exports planned |
| IV. Performance Requirements | PASS | Feature-gated (zero cost when unused); dispatch queue lock held < 1μs; hot paths use `borrow_mut_gen_only()` |
| V. Safety and Correctness | PASS | No new unsafe except documented GC bridge patterns; all public APIs return Result/Option; GC safety via established rudo-gc async primitives |

**Additional constraints check:**
- Technology Stack: Rust 2021, rudo-gc — PASS
- Dependency Management: tokio is well-maintained, widely used; parking_lot is established — PASS
- Quality Gates: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt` — PASS (planned)

**Gate result: PASS** — No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/003-rvue-async-support/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── async-api.md     # Public API contract
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/rvue/src/
├── async_runtime/            # NEW: Async runtime module
│   ├── mod.rs                # Module root, re-exports, feature gate
│   ├── dispatch.rs           # UiDispatchQueue + dispatch_to_ui()
│   ├── task.rs               # TaskHandle + spawn_task() + spawn_task_with_result()
│   ├── registry.rs           # TaskRegistry (component → task mapping)
│   ├── signal_sender.rs      # SignalSender<T> for thread-safe signal updates
│   └── resource.rs           # create_resource() + ResourceState<T>
├── app.rs                    # MODIFIED: tokio init, EventLoopProxy, drain dispatch queue
├── component.rs              # MODIFIED: cancel tasks on unmount (via TaskRegistry)
├── signal.rs                 # MODIFIED: WriteSignal::sender() method
├── lib.rs                    # MODIFIED: add async_runtime module + prelude exports
└── ...existing files...

crates/rvue/Cargo.toml        # MODIFIED: add tokio + parking_lot + feature flag
```

**Structure Decision**: Single crate extension. The async runtime is a new module within `crates/rvue/src/` rather than a separate crate, because it deeply integrates with existing component lifecycle, signals, and the app event loop. Feature-gated via `#[cfg(feature = "async")]`.

## Constitution Re-Check (Post-Design)

*Re-evaluated after Phase 1 design artifacts were generated.*

| Principle | Status | Post-Design Evidence |
|-----------|--------|----------------------|
| I. Code Quality | PASS | All public API types have doc comments (see contracts/async-api.md); naming follows conventions; Result in ResourceState::Error |
| II. Testing Standards | PASS | Test matrix defined in spec (dispatch, task, registry, signal sender, lifecycle, stress, GC safety); single-threaded tests respected |
| III. User Experience Consistency | PASS | API mirrors SolidJS patterns (`create_resource`); consistent `spawn_*` naming; prelude exports; quickstart guide provided |
| IV. Performance Requirements | PASS | Feature-gated; parking_lot for fast locks; EventLoopProxy for immediate wakeup (no polling); hot-path signal updates use `borrow_mut_gen_only()` |
| V. Safety and Correctness | PASS | One justified `unsafe impl Send/Sync` for SignalSender (only holds `Arc<dyn Fn + Send + Sync>`); GC safety via established rudo-gc primitives; `catch_unwind` at dispatch boundary |

**Post-design gate result: PASS** — No new violations introduced by the design.

## Complexity Tracking

No constitution violations to justify.
