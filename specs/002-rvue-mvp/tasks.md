# Tasks: Rvue MVP Framework

**Input**: Design documents from `/specs/002-rvue-mvp/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included per NFR-002 requirement (automated unit and integration tests with min. 80% coverage).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace structure**: `crates/rvue/`, `crates/rvue-macro/`, `crates/rvue-examples/`
- Paths follow the structure defined in plan.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create Cargo workspace with root Cargo.toml in repository root
- [x] T002 [P] Create rvue crate structure in crates/rvue/ with Cargo.toml
- [x] T003 [P] Create rvue-macro crate structure in crates/rvue-macro/ with Cargo.toml
- [x] T004 [P] Create rvue-examples crate structure in crates/rvue-examples/ with Cargo.toml
- [x] T005 [P] Configure rustfmt.toml and clippy.toml in repository root
- [x] T006 [P] Add rudo-gc dependency to rvue crate (crates.io version 0.1)
- [x] T007 [P] Add vello dependency to rvue crate in crates/rvue/Cargo.toml
- [x] T008 [P] Add taffy dependency to rvue crate in crates/rvue/Cargo.toml
- [x] T009 [P] Add winit dependency to rvue crate in crates/rvue/Cargo.toml
- [x] T010 [P] Add proc_macro dependencies to rvue-macro crate in crates/rvue-macro/Cargo.toml (syn, quote, proc_macro2)
- [x] T011 [P] Configure rvue-macro as proc-macro crate in crates/rvue-macro/Cargo.toml
- [x] T012 Add rvue-macro as dependency to rvue crate in crates/rvue/Cargo.toml

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T013 Create Signal trait and ReadSignal/WriteSignal traits in crates/rvue/src/signal.rs
- [x] T014 [P] Implement SignalData<T> struct with value, subscribers, and version fields in crates/rvue/src/signal.rs
- [x] T015 [P] Implement create_signal function returning (ReadSignal, WriteSignal) tuple in crates/rvue/src/signal.rs
- [x] T016 Implement ReadSignal<T> trait with get() method in crates/rvue/src/signal.rs
- [x] T017 Implement WriteSignal<T> trait with set() and update() methods in crates/rvue/src/signal.rs
- [x] T018 [P] Create Effect struct with closure, dependencies, and is_dirty fields in crates/rvue/src/effect.rs
- [x] T019 [P] Implement create_effect function with automatic dependency tracking in crates/rvue/src/effect.rs
- [x] T020 Implement effect execution and re-run logic when dependencies change in crates/rvue/src/effect.rs
- [x] T021 [P] Create Component struct with id, component_type, children, parent, signals, effects fields in crates/rvue/src/component.rs
- [x] T022 [P] Implement Component trait with lifecycle methods (mount, unmount, update) in crates/rvue/src/component.rs
- [x] T023 [P] Create View trait with into_component method in crates/rvue/src/view.rs
- [x] T024 Implement View struct with root_component, signals, and effects fields in crates/rvue/src/view.rs
- [x] T025 [P] Create Style struct with color, font_size, padding, margin, flex properties in crates/rvue/src/style.rs
- [x] T026 [P] Create Color enum and related types in crates/rvue/src/style.rs
- [x] T027 [P] Create Spacing and Size types for layout properties in crates/rvue/src/style.rs
- [x] T028 Create ComponentType enum (Text, Button, Custom, etc.) in crates/rvue/src/component.rs
- [x] T029 Create ComponentProps enum for different widget types in crates/rvue/src/component.rs
- [x] T030 [P] Create lib.rs with public API exports (prelude module) in crates/rvue/src/lib.rs
- [x] T031 [P] Create prelude module re-exporting common types in crates/rvue/src/prelude.rs
- [x] T032 [P] Add unit tests for Signal creation and updates in crates/rvue/tests/signal_test.rs
- [x] T033 [P] Add unit tests for Effect dependency tracking in crates/rvue/tests/effect_test.rs
- [x] T034 [P] Add unit tests for Component lifecycle in crates/rvue/tests/component_test.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Desktop Application Developer Creates Simple Counter App (Priority: P1) 🎯 MVP

**Goal**: Enable developers to create a counter component with reactive state management and UI updates using familiar web-like syntax.

**Independent Test**: Create a counter component that increments/decrements a value displayed in the UI. Test validates: (1) state can be declared and updated, (2) UI automatically reflects state changes, (3) event handlers work correctly, (4) no manual memory management is required.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T035 [P] [US1] Create integration test for counter component in crates/rvue/tests/counter_test.rs
- [x] T036 [P] [US1] Create unit test for Text widget rendering in crates/rvue/tests/text_widget_test.rs
- [x] T037 [P] [US1] Create unit test for Button widget event handling in crates/rvue/tests/button_widget_test.rs

### Implementation for User Story 1

- [x] T038 [P] [US1] Create widgets module structure in crates/rvue/src/widgets/mod.rs
- [x] T039 [P] [US1] Implement Text widget struct and ComponentType::Text in crates/rvue/src/widgets/text.rs
- [x] T040 [P] [US1] Implement Button widget struct and ComponentType::Button in crates/rvue/src/widgets/button.rs
- [x] T041 [P] [US1] Create basic view! macro entry point in crates/rvue-macro/src/lib.rs
- [x] T042 [US1] Implement view! macro parsing for static components in crates/rvue-macro/src/lib.rs (basic placeholder)
- [x] T043 [US1] Implement view! macro code generation for Text and Button components in crates/rvue-macro/src/lib.rs (basic placeholder)
- [ ] T044 [US1] Implement signal binding detection in view! macro (move || closures) in crates/rvue-macro/src/lib.rs (deferred - needs full parser)
- [x] T045 [US1] Create component attribute macro entry point in crates/rvue-macro/src/lib.rs
- [x] T046 [US1] Implement #[component] macro for GC allocation and lifecycle in crates/rvue-macro/src/lib.rs (basic placeholder)
- [x] T047 [US1] Create layout module structure in crates/rvue/src/layout/mod.rs
- [x] T048 [US1] Create LayoutNode wrapper around Taffy node in crates/rvue/src/layout/node.rs (basic structure)
- [x] T049 [US1] Create render module structure in crates/rvue/src/render/mod.rs
- [x] T050 [US1] Create VelloFragment wrapper for scene graph in crates/rvue/src/render/widget.rs (basic structure)
- [x] T051 [US1] Implement basic Vello scene generation for Text widget in crates/rvue/src/render/widget.rs
- [x] T052 [US1] Implement basic Vello scene generation for Button widget in crates/rvue/src/render/widget.rs
- [x] T053 [US1] Create application runner with winit event loop in crates/rvue/src/app.rs
- [x] T054 [US1] Implement run_app function that initializes window and starts event loop in crates/rvue/src/app.rs
- [x] T055 [US1] Implement window creation and event handling in crates/rvue/src/app.rs
- [x] T056 [US1] Implement Vello renderer initialization and scene rendering in crates/rvue/src/render/scene.rs
- [x] T057 [US1] Connect signal updates to Vello scene updates via effects in crates/rvue/src/render/scene.rs (basic structure)
- [x] T058 [US1] Create counter example application in crates/rvue-examples/counter/src/main.rs
- [x] T059 [US1] Implement counter component with increment/decrement buttons in crates/rvue-examples/counter/src/main.rs
- [x] T060 [US1] Verify counter example compiles and runs successfully

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Counter app should display, buttons should work, and count should update reactively.

---

## Phase 4: User Story 2 - Developer Builds Application with Conditional Rendering (Priority: P1)

**Goal**: Enable developers to show/hide UI elements based on application state using familiar conditional rendering patterns.

**Independent Test**: Create a component that conditionally shows/hides content based on a boolean signal. Test validates: (1) conditional components can be declared declaratively, (2) UI updates when conditions change, (3) hidden components don't consume rendering resources unnecessarily.

### Tests for User Story 2

- [ ] T061 [P] [US2] Create integration test for Show component in crates/rvue/tests/integration/show_test.rs
- [ ] T062 [P] [US2] Create unit test for Show component mounting/unmounting in crates/rvue/tests/unit/widgets/show_test.rs

### Implementation for User Story 2

- [ ] T063 [P] [US2] Implement Show widget struct and ComponentType::Show in crates/rvue/src/widgets/show.rs
- [ ] T064 [US2] Implement Show component props with when signal and children in crates/rvue/src/widgets/show.rs
- [ ] T065 [US2] Implement Show component mounting/unmounting logic based on when signal in crates/rvue/src/widgets/show.rs
- [ ] T066 [US2] Add Show component support to view! macro parsing in crates/rvue-macro/src/view.rs
- [ ] T067 [US2] Generate effect for Show component that watches when signal in crates/rvue-macro/src/view.rs
- [ ] T068 [US2] Implement conditional rendering in Vello scene (skip hidden components) in crates/rvue/src/render/widget.rs
- [ ] T069 [US2] Create example demonstrating Show component in crates/rvue-examples/counter/src/main.rs (extend counter)
- [ ] T070 [US2] Verify Show component example works correctly

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Conditional rendering should work with signal-based conditions.

---

## Phase 5: User Story 3 - Developer Creates List View with Dynamic Items (Priority: P1)

**Goal**: Enable developers to render lists of items that can be added, removed, or modified dynamically with efficient updates.

**Independent Test**: Create a component that renders a list from a reactive data source. Test validates: (1) items can be rendered from a collection, (2) adding items updates the UI efficiently, (3) removing items updates the UI efficiently, (4) modifying items updates only affected UI elements.

### Tests for User Story 3

- [ ] T071 [P] [US3] Create integration test for For component in crates/rvue/tests/integration/for_test.rs
- [ ] T072 [P] [US3] Create unit test for For component key-based diffing in crates/rvue/tests/unit/widgets/for_loop_test.rs

### Implementation for User Story 3

- [ ] T073 [P] [US3] Implement For widget struct and ComponentType::For in crates/rvue/src/widgets/for_loop.rs
- [ ] T074 [US3] Implement For component props with each signal, key function, and children closure in crates/rvue/src/widgets/for_loop.rs
- [ ] T075 [US3] Implement key-based diffing algorithm for list updates in crates/rvue/src/widgets/for_loop.rs
- [ ] T076 [US3] Implement efficient add/remove/update operations for list items in crates/rvue/src/widgets/for_loop.rs
- [ ] T077 [US3] Add For component support to view! macro parsing in crates/rvue-macro/src/view.rs
- [ ] T078 [US3] Generate effect for For component that watches each signal in crates/rvue-macro/src/view.rs
- [ ] T079 [US3] Implement list rendering in Vello scene with efficient updates in crates/rvue/src/render/widget.rs
- [ ] T080 [US3] Create list example application in crates/rvue-examples/list/src/main.rs
- [ ] T081 [US3] Implement todo list component with add/remove functionality in crates/rvue-examples/list/src/main.rs
- [ ] T082 [US3] Verify list example works correctly with efficient updates

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently. List rendering should efficiently handle add/remove/update operations.

---

## Phase 6: User Story 4 - Application Startup Performance (Priority: P2)

**Goal**: Ensure desktop applications start quickly with low memory overhead compared to webview-based solutions.

**Independent Test**: Measure startup time and memory usage of a simple application. Test validates: (1) application starts within acceptable time, (2) initial memory footprint is reasonable, (3) startup is faster than equivalent Electron/Tauri applications.

### Tests for User Story 4

- [ ] T083 [P] [US4] Create benchmark test for application startup time in crates/rvue/tests/integration/startup_benchmark.rs
- [ ] T084 [P] [US4] Create benchmark test for initial memory usage in crates/rvue/tests/integration/memory_benchmark.rs

### Implementation for User Story 4

- [ ] T085 [US4] Optimize application initialization to reduce startup time in crates/rvue/src/app.rs
- [ ] T086 [US4] Implement lazy loading for Vello renderer initialization in crates/rvue/src/render/scene.rs
- [ ] T087 [US4] Optimize GC initialization to reduce startup overhead in crates/rvue/src/lib.rs
- [ ] T088 [US4] Profile and optimize component tree creation in crates/rvue/src/component.rs
- [ ] T089 [US4] Create performance comparison example (counter app) in crates/rvue-examples/benchmark/src/main.rs
- [ ] T090 [US4] Document startup time and memory usage benchmarks in docs/performance.md
- [ ] T091 [US4] Verify startup time < 2 seconds and memory < 100MB for simple application

**Checkpoint**: Application startup should meet performance targets. Benchmarks should demonstrate improvement over Electron/Tauri.

---

## Phase 7: User Story 5 - Developer Builds Complex Layout with Flexbox/Grid (Priority: P2)

**Goal**: Enable developers to create complex layouts with flexible positioning, alignment, and spacing using familiar CSS-like concepts.

**Independent Test**: Create a component with complex nested layouts. Test validates: (1) flexbox layouts work as expected, (2) spacing and alignment behave correctly, (3) layouts adapt to content changes, (4) layout code is intuitive for web developers.

### Tests for User Story 5

- [ ] T092 [P] [US5] Create integration test for Flexbox layout in crates/rvue/tests/integration/layout_test.rs
- [ ] T093 [P] [US5] Create unit test for Taffy layout integration in crates/rvue/tests/unit/layout/node_test.rs

### Implementation for User Story 5

- [ ] T094 [P] [US5] Implement Flex widget struct and ComponentType::Flex in crates/rvue/src/widgets/flex.rs
- [ ] T095 [US5] Implement Flex component props (direction, gap, align_items, justify_content) in crates/rvue/src/widgets/flex.rs
- [ ] T096 [US5] Implement Taffy style mapping from Flex props in crates/rvue/src/layout/node.rs
- [ ] T097 [US5] Implement layout calculation trigger on style/content changes in crates/rvue/src/layout/node.rs
- [ ] T098 [US5] Implement layout result application to Vello scene positions in crates/rvue/src/render/widget.rs
- [ ] T099 [US5] Add Flex component support to view! macro parsing in crates/rvue-macro/src/view.rs
- [ ] T100 [US5] Implement FlexDirection, AlignItems, JustifyContent enums in crates/rvue/src/style.rs
- [ ] T101 [US5] Create layout example application in crates/rvue-examples/layout/src/main.rs
- [ ] T102 [US5] Implement complex nested flexbox layout example in crates/rvue-examples/layout/src/main.rs
- [ ] T103 [US5] Verify layout example works correctly with responsive behavior

**Checkpoint**: Complex layouts should work correctly. Layout code should be intuitive for web developers familiar with CSS flexbox.

---

## Phase 8: Additional Input Components (FR-013)

**Purpose**: Implement remaining input components (text input, number input, checkbox, radio) required by functional requirements.

- [ ] T104 [P] Implement TextInput widget struct and ComponentType::TextInput in crates/rvue/src/widgets/input.rs
- [ ] T105 [P] Implement NumberInput widget struct and ComponentType::NumberInput in crates/rvue/src/widgets/input.rs
- [ ] T106 [P] Implement Checkbox widget struct and ComponentType::Checkbox in crates/rvue/src/widgets/checkbox.rs
- [ ] T107 [P] Implement Radio widget struct and ComponentType::Radio in crates/rvue/src/widgets/radio.rs
- [ ] T108 Add input component support to view! macro in crates/rvue-macro/src/view.rs
- [ ] T109 Implement event handling for input components (on_input, on_change) in crates/rvue/src/widgets/input.rs
- [ ] T110 Create form example demonstrating all input types in crates/rvue-examples/form/src/main.rs
- [ ] T111 [P] Add unit tests for input components in crates/rvue/tests/unit/widgets/input_test.rs

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and Constitution compliance

- [ ] T112 [P] Update framework documentation in crates/rvue/README.md
- [ ] T113 [P] Code cleanup and refactoring per Principle I in all crates
- [ ] T114 [P] Performance optimization audit and benchmarking per Principle IV
- [ ] T115 [P] UX Consistency audit - verify API follows Vue Composition API patterns per Principle III
- [ ] T116 [P] Final test coverage verification - ensure 80% coverage per Principle II in crates/rvue/tests/
- [ ] T117 [P] Security hardening - input validation for all user inputs per NFR-005
- [ ] T118 [P] Error handling implementation - error types and validation results per NFR-008
- [ ] T119 [P] Run quickstart.md validation - ensure all examples work
- [ ] T120 [P] Platform compatibility testing on Windows, macOS, and Linux per NFR-007
- [ ] T121 [P] Create comprehensive API documentation with examples in docs/api.md
- [ ] T122 [P] Verify all functional requirements (FR-001 through FR-013) are implemented
- [ ] T123 [P] Verify all non-functional requirements (NFR-001 through NFR-009) are met

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (US1 → US2 → US3 → US4 → US5)
- **Additional Components (Phase 8)**: Can proceed after US1-3 are complete
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories. This is the MVP.
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - Uses signals and components from US1 but independently testable
- **User Story 3 (P1)**: Can start after Foundational (Phase 2) - Uses signals and components from US1 but independently testable
- **User Story 4 (P2)**: Can start after US1 is complete - Requires working application to benchmark
- **User Story 5 (P2)**: Can start after US1 is complete - Requires basic components and rendering

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Core types before widgets
- Widgets before macro support
- Macro support before examples
- Examples serve as integration tests
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, user stories US1, US2, US3 can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Widget implementations within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: T035 [P] [US1] Create integration test for counter component
Task: T036 [P] [US1] Create unit test for Text widget rendering
Task: T037 [P] [US1] Create unit test for Button widget event handling

# Launch all widget implementations together:
Task: T039 [P] [US1] Implement Text widget struct
Task: T040 [P] [US1] Implement Button widget struct

# Launch layout and render setup together:
Task: T047 [P] [US1] Create layout module structure
Task: T049 [P] [US1] Create render module structure
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Counter App)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo (Performance)
6. Add User Story 5 → Test independently → Deploy/Demo (Layout)
7. Add Input Components → Test independently → Deploy/Demo
8. Polish → Final release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Counter - MVP)
   - Developer B: User Story 2 (Conditional Rendering)
   - Developer C: User Story 3 (List View)
3. After US1-3 complete:
   - Developer A: User Story 4 (Performance)
   - Developer B: User Story 5 (Layout)
   - Developer C: Input Components
4. All team: Polish phase

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- MVP scope: Complete through User Story 1 for initial release
- All tasks include specific file paths for clarity
