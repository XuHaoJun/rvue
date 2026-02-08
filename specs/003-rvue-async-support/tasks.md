# Tasks: Rvue Async Runtime Support

**Input**: Design documents from `/specs/003-rvue-async-support/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Test tasks are included in the Polish phase per spec success criteria and plan (unit tests for dispatch, task, registry; panic handling). No TDD test-first tasks—tests accompany implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US6)
- Include exact file paths in descriptions

## Path Conventions

- **Rvue crate**: `crates/rvue/` (src/, tests/, Cargo.toml)
- **Async module**: `crates/rvue/src/async_runtime/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and async feature wiring

- [X] T001 Add tokio and parking_lot dependencies with feature gate in crates/rvue/Cargo.toml (async = ["dep:tokio", "dep:parking_lot"], tokio features: rt-multi-thread, sync, time)
- [X] T002 Create async_runtime module stub with feature gate and re-exports in crates/rvue/src/async_runtime/mod.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: UI dispatch and event-loop wakeup that all user stories depend on. No user story work can begin until this phase is complete.

- [X] T003 Implement UiDispatchQueue, dispatch_to_ui, and drain_ui_callbacks (with parking_lot::Mutex, VecDeque) in crates/rvue/src/async_runtime/dispatch.rs
- [X] T004 Define RvueUserEvent enum and EventLoopProxy storage; add set_proxy to dispatch queue and user_event handler in crates/rvue/src/app.rs
- [X] T005 Call drain_ui_callbacks when RvueUserEvent::AsyncDispatchReady is received (and optionally in run_update_passes as fallback) in crates/rvue/src/app.rs
- [X] T006 Add panic handling (catch_unwind) in drain_ui_callbacks and log panics in crates/rvue/src/async_runtime/dispatch.rs

**Checkpoint**: Foundation ready — user story implementation can begin

---

## Phase 3: User Story 1 — Non-Blocking Async Operations (Priority: P1) — MVP

**Goal**: Spawn async tasks and deliver results to the UI without blocking; UI stays responsive.

**Independent Test**: Spawn an async task with a simulated delay; verify UI remains responsive and the result is applied on the next UI cycle via dispatch_to_ui.

- [X] T007 [P] [US1] Implement TaskId and TaskHandle (id, abort_handle, completed, abort, is_completed, is_running, id()) in crates/rvue/src/async_runtime/task.rs
- [X] T008 [US1] Implement lazy tokio runtime (OnceLock) and spawn_task in crates/rvue/src/async_runtime/task.rs
- [X] T009 [US1] Implement spawn_task_with_result in crates/rvue/src/async_runtime/task.rs
- [X] T010 [US1] Add async_runtime module to lib.rs and prelude (feature-gated) in crates/rvue/src/lib.rs and crates/rvue/src/prelude.rs

**Checkpoint**: User Story 1 is testable — spawn_task + dispatch_to_ui work end-to-end

---

## Phase 4: User Story 2 — Task Lifecycle Management (Priority: P1)

**Goal**: Tasks registered to a component are automatically cancelled when the component unmounts.

**Independent Test**: Spawn tasks in a component, unmount the component, verify tasks are cancelled (TaskHandle::is_running() false).

- [X] T011 [P] [US2] Implement TaskRegistry (register, cancel_all, cleanup_completed, task_count) in crates/rvue/src/async_runtime/registry.rs
- [X] T012 [US2] Register task with TaskRegistry when spawn_task/spawn_task_with_result is called and runtime::current_owner() is Some in crates/rvue/src/async_runtime/task.rs
- [X] T013 [US2] Call crate::async_runtime::registry::task_registry().cancel_all(self.id()) in Component::unmount in crates/rvue/src/component.rs (feature-gated)

**Checkpoint**: User Stories 1 and 2 work — tasks cancel on unmount

---

## Phase 5: User Story 4 — Thread-Safe Signal Updates (Priority: P2)

**Goal**: Update signals from async tasks via a Send+Sync proxy so UI receives updates on the UI thread.

**Independent Test**: Spawn an async task that calls SignalSender::set; verify signal value updates on the UI thread.

- [X] T014 [P] [US4] Implement SignalSender<T> (dispatch_fn, set, Send+Sync impl) in crates/rvue/src/async_runtime/signal_sender.rs
- [X] T015 [US4] Add WriteSignal::sender() method (feature-gated, T: Send) in crates/rvue/src/signal.rs
- [X] T016 [US4] Export SignalSender and dispatch_to_ui from async_runtime mod and prelude in crates/rvue/src/async_runtime/mod.rs and crates/rvue/src/prelude.rs

**Checkpoint**: User Story 4 works — SignalSender::set from async updates signal on UI thread

---

## Phase 6: User Story 3 — Reactive Async Data Patterns (Priority: P1)

**Goal**: create_resource with loading/ready/error states; UI reactively updates when data is ready or errors.

**Independent Test**: Create a resource that fetches data; observe Loading then Ready/Error and verify UI updates.

- [X] T017 [P] [US3] Implement ResourceState<T> (Pending, Loading, Ready, Error) and Resource<T> (state, refetch, get) in crates/rvue/src/async_runtime/resource.rs
- [X] T018 [US3] Implement create_resource (source, fetcher, initial fetch, spawn_task + SignalSender for state) in crates/rvue/src/async_runtime/resource.rs
- [X] T019 [US3] Export create_resource, Resource, ResourceState from async_runtime mod and prelude in crates/rvue/src/async_runtime/mod.rs and crates/rvue/src/prelude.rs

**Checkpoint**: User Story 3 works — create_resource drives reactive UI

---

## Resolution: Resource API Re-Enabled via GcRwLock Migration

**Problem Solved**: The `create_resource`, `Resource<T>`, and `ResourceState<T>` APIs have been **re-enabled** by migrating from `GcCell<T>` to `GcRwLock<T>` in the reactivity system.

**Solution**: Changed `GcCell<Vec<Weak<Effect>>>` to `GcRwLock<Vec<Weak<Effect>>>` in `signal.rs` and `GcCell<T>` to `GcRwLock<T>` in `rvue-signals/src/lib.rs`. This makes the reactivity system thread-safe (Send + Sync) when contained types are Send + Sync.

**Key Changes**:
- `GcRwLock` wraps `parking_lot::RwLock` and implements `Send + Sync` when `T: Trace + Send + Sync`
- `.borrow()` → `.read()`, `.borrow_mut_gen_only()` → `.write()` for compatibility
- Resource API now uses `tokio::runtime::Handle::current().block_on()` to execute fetcher on UI thread

**Files Modified**:
- `crates/rvue/src/signal.rs` - Effect's GcCell → GcRwLock
- `crates/rvue-signals/src/lib.rs` - SignalData's GcCell → GcRwLock
- `crates/rvue/src/async_runtime/resource.rs` - create_resource implementation with fix for closure move error
- `crates/rvue/src/async_runtime/mod.rs` - Module export
- `crates/rvue/src/prelude.rs` - Public exports

---

## Phase 7: User Story 5 — Rate-Limited Async Operations (Priority: P2)

**Goal**: spawn_interval and spawn_debounced for periodic and debounced async work.

**Independent Test**: spawn_debounced with rapid triggers executes only after delay; spawn_interval runs at period.

- [X] T020 [P] [US5] Implement spawn_interval (period, FnMut -> Fut) returning TaskHandle in crates/rvue/src/async_runtime/task.rs
- [X] T021 [US5] Implement spawn_debounced and DebouncedTask (call, cancel) in crates/rvue/src/async_runtime/task.rs
- [X] T022 [US5] Export spawn_interval, spawn_debounced, DebouncedTask from async_runtime mod and prelude in crates/rvue/src/async_runtime/mod.rs and crates/rvue/src/prelude.rs

**Checkpoint**: User Stories 5 works — interval and debounce available

---

## Phase 8: User Story 6 — Component Cleanup Hooks (Priority: P2)

**Goal**: on_cleanup is available and documented for async cleanup (e.g. aborting handles on unmount).

**Independent Test**: Register on_cleanup in a component that spawns a task; unmount and verify cleanup runs (e.g. handle.abort() called).

- [X] T023 [US6] Ensure on_cleanup is documented for async cleanup (abort handles, release resources) in specs/003-rvue-async-support/quickstart.md and contracts/async-api.md

**Checkpoint**: User Story 6 satisfied — on_cleanup documented and usable with async

---

## Phase X: GC Safety (Priority: P1)

**Goal**: Fix GC safety issues in spawn_interval and spawn_debounced; provide spawn_watch_signal for safe signal watching.

**Background**: Gc<T> is !Send + !Sync and cannot be captured in async closures without explicit handling. spawn_with_gc! moves Gc<T>, requiring users to clone before use.

**Independent Test**: spawn_watch_signal polls signal correctly and dispatches to UI automatically.

### GC Safety Tasks

- [ ] TG001 [P] Document GC safety requirements for spawn_interval in crates/rvue/src/async_runtime/task.rs
      Content: "IMPORTANT: The closure MUST NOT capture Gc<T>. Extract values or clone before spawning."

- [ ] TG002 Document GC safety requirements for spawn_debounced in crates/rvue/src/async_runtime/task.rs
      Content: Same restrictions as spawn_interval

- [ ] TG003 [P] Implement spawn_watch_signal() helper in crates/rvue/src/async_runtime/task.rs
      Purpose: Automatic signal polling with proper GC handling
      Signature: spawn_watch_signal(signal, period, callback) -> SignalWatcher<T>
      Behavior: Polls signal at interval, callback returns Some(v) to update, None to just watch

- [ ] TG004 Implement SignalWatcher<T> type in crates/rvue/src/async_runtime/task.rs
      Fields: sender (mpsc::UnboundedSender<()>)
      Methods: stop() -> ()

- [ ] TG005 Export SignalWatcher from async_runtime mod in crates/rvue/src/async_runtime/mod.rs

- [ ] TG006 Export SignalWatcher from prelude in crates/rvue/src/prelude.rs

- [ ] TG007 [P] Update quickstart.md with GC safety patterns
      Content: Before/after examples, common mistakes, spawn_watch_signal usage

- [ ] TG008 Update async-api.md with GC safety contracts
      Content: Add GC Safety section to spawn_interval and spawn_debounced contracts

- [ ] TG009 Update data-model.md with SignalWatcher entity documentation

- [ ] TG010 [P] Add GC safety tests in crates/rvue/tests/gc_safety_test.rs
      Test: spawn_watch_signal polls signal at correct interval
      Test: callback receives current signal value
      Test: Some(v) updates signal, None just watches
      Test: stop() cancels the watcher

**Checkpoint**: GC safety patterns documented; spawn_watch_signal provides safe alternative

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Tests, validation, and docs

- [ ] T024 [P] Add unit tests for UiDispatchQueue and dispatch_to_ui (single-thread drain, multi-thread dispatch) in crates/rvue/tests/ or crates/rvue/src/async_runtime/dispatch.rs
- [ ] T025 [P] Add unit tests for TaskHandle and spawn_task (abort, is_completed, is_running) in crates/rvue/tests/
- [ ] T026 [P] Add unit tests for TaskRegistry (register, cancel_all, task_count) in crates/rvue/tests/
- [ ] T027 Run cargo build --features async and cargo test --features async -- --test-threads=1; fix any failures
- [ ] T028 Update docs (async-api reference, quickstart) if any API drift from contracts/async-api.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1 — blocks all user stories.
- **Phase 3 (US1)**: Depends on Phase 2 — MVP.
- **Phase 4 (US2)**: Depends on Phase 3 (needs spawn_task and TaskHandle).
- **Phase 5 (US4)**: Depends on Phase 2 (needs dispatch_to_ui).
- **Phase 6 (US3)**: Depends on Phase 5 (create_resource uses SignalSender).
- **Phase 7 (US5)**: Depends on Phase 3 (uses spawn_task).
- **Phase 8 (US6)**: Depends on Phase 2 (on_cleanup already exists; documentation only).
- **Phase 9 (Polish)**: Depends on Phases 3–8.

### User Story Completion Order

- **US1 (P1)**: After Foundational → first deliverable (spawn_task + dispatch_to_ui).
- **US2 (P1)**: After US1 → task lifecycle (TaskRegistry + unmount).
- **US4 (P2)**: After Foundational → SignalSender (can be parallel to US1/US2 if desired).
- **US3 (P1)**: After US4 → create_resource.
- **US5 (P2)**: After US1 → spawn_interval, spawn_debounced.
- **US6 (P2)**: After Foundational → documentation only.

### Within Each User Story

- Implement types before functions that use them (e.g. TaskId/TaskHandle before spawn_task).
- Core implementation before exports/prelude.
- Story is complete when checkpoint criteria hold.

### Parallel Opportunities

- T007 (TaskId/TaskHandle) can run in parallel with any other file in a different phase if deps allow.
- T011 (TaskRegistry), T014 (SignalSender), T017 (ResourceState/Resource), T020 (spawn_interval) are [P] within their phase.
- T024–T026 (tests) can run in parallel.
- After Foundational, US1, US2, US4 can be advanced in parallel by different owners (US3 after US4, US5 after US1).

---

## Parallel Example: User Story 1

```text
# After Phase 2 complete, US1 can proceed as:
T007: Implement TaskId and TaskHandle in crates/rvue/src/async_runtime/task.rs
T008: Implement lazy runtime + spawn_task (same file, after T007)
T009: Implement spawn_task_with_result (same file, after T008)
T010: Wire async_runtime in lib.rs and prelude
```

---

## Parallel Example: Foundational + US4 Prep

```text
# After T003 (dispatch), T004–T006 (app + panic handling) must run in order.
# US4 (SignalSender) only needs dispatch_to_ui, so after T003+T005+T006:
T014: Implement SignalSender in crates/rvue/src/async_runtime/signal_sender.rs
T015: Add WriteSignal::sender() in crates/rvue/src/signal.rs
T016: Export from mod and prelude
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002).
2. Complete Phase 2: Foundational (T003–T006).
3. Complete Phase 3: User Story 1 (T007–T010).
4. **STOP and VALIDATE**: Build with `--features async`, run tests, verify spawn_task + dispatch_to_ui.
5. Demo: async task updates UI without blocking.

### Incremental Delivery

1. Setup + Foundational → dispatch and wakeup working.
2. Add US1 → Non-blocking async (MVP).
3. Add US2 → Task lifecycle (cancel on unmount).
4. Add US4 → SignalSender → US3 (create_resource).
5. Add US5 → spawn_interval, spawn_debounced.
6. Add US6 doc + Polish → tests and docs.

### Suggested MVP Scope

- **MVP**: Phase 1 + Phase 2 + Phase 3 (T001–T010). Delivers: dispatch_to_ui, spawn_task, spawn_task_with_result, TaskHandle, wired into app and prelude.

---

## Notes

- [P] = different files or no ordering dependency; safe to run in parallel.
- [USn] maps task to spec user story for traceability.
- Each user story phase is independently testable at its checkpoint.
- Use `cargo test --features async -- --test-threads=1` for tests.
- Commit after each task or logical group; stop at checkpoints to validate.
