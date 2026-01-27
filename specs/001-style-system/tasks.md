---

description: "Task list for rvue-style library implementation"
---

# Tasks: Rvue Style System

**Input**: Design documents from `/specs/001-styleFeature-system/`
****: `001-style-system` | **Branch**: `001-style-system`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md, contracts/

**Tests**: NOT requested in spec - focus on implementation only

**Organization**: Tasks grouped by user story to enable independent implementation

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Initialize the rvue-style library crate and basic structure

- [x] T001 Create `crates/rvue-style/` directory structure per implementation plan
- [x] T002 Create `crates/rvue-style/Cargo.toml` with dependencies:
  - `selectors = "0.35"`
  - `cssparser = "0.36"`
  - `bitflags = "2"`
  - `rudo-gc` (from rvue workspace)
- [x] T003 [P] Create `crates/rvue-style/src/lib.rs` with module declarations
- [x] T004 [P] Create `crates/rvue-style/src/property.rs` module file
- [x] T005 [P] Create `crates/rvue-style/src/properties/` module files (mod.rs, color.rs, spacing.rs, layout.rs, sizing.rs, background.rs, border.rs, font.rs, visibility.rs, computed_styles.rs)
- [x] T006 [P] Create `crates/rvue-style/src/selectors/` module files (mod.rs, element.rs, state.rs)
- [x] T007 [P] Create `crates/rvue-style/src/stylesheet/` module files (mod.rs, parser.rs, rule.rs, stylesheet.rs)
- [x] T008 [P] Create `crates/rvue-style/src/css/` module files (mod.rs, value_parser.rs, properties.rs)
- [x] T009 [P] Create `crates/rvue-style/src/reactive/` module files (mod.rs, style_signal.rs)
- [x] T010 [P] Create `crates/rvue-style/src/widget/` module files (mod.rs, styled.rs)
- [x] T011 [P] Create `crates/rvue-style/tests/` directory with property_test.rs, selector_test.rs, stylesheet_test.rs
- [x] T012 [P] Create `crates/rvue-style/examples/` directory with basic_styling.rs, stylesheet.rs, reactive_styles.rs
- [x] T013 Create `crates/rvue-style/src/prelude.rs` with commonly used exports

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Property System Foundation

- [x] T014 Create `Property` trait in `crates/rvue-style/src/property.rs`:
  ```rust
  pub trait Property: Default + Send + Sync + 'static {
      fn static_default() -> &'static Self;
  }
  ```
- [x] T015 Implement `rudo_gc::Trace` for `Property` trait and `Properties` container
- [x] T016 Create `Properties` struct in `crates/rvue-style/src/property.rs` with `AnyMap` field
- [x] T017 Implement `Properties::new()`, `Properties::with()`, `Properties::get()`, `Properties::insert()`, `Properties::remove()` methods
- [x] T018 Create `ParseError` struct in `crates/rvue-style/src/` for error handling

### CSS Value Parsing Foundation

- [x] T019 Create `Color` struct in `crates/rvue-style/src/properties/color.rs` with `rgb()`, `rgba()`, `from_hex()` constructors
- [x] T020 Implement `Color::from_str()` for CSS named colors in `crates/rvue-style/src/properties/color.rs`
- [x] T021 Implement `Property` trait for `Color` in `crates/rvue-style/src/properties/color.rs`
- [x] T022 Create `Size` enum in `crates/rvue-style/src/properties/sizing.rs` (Auto, Pixels, Percent, MinContent, MaxContent, FitContent)
- [x] T023 Implement `Property` trait for `Size` in `crates/rvue-style/src/properties/sizing.rs`

### ElementState Foundation

- [x] T024 Create `ElementState` bitflags struct in `crates/rvue-style/src/selectors/state.rs`:
  - HOVER, FOCUS, ACTIVE, DISABLED, CHECKED, DRAGGING, DRAG_OVER, SELECTED, EXPANDED, COLLAPSED, VISITED, TARGET, FOCUS_WITHIN, FOCUS_VISIBLE
- [x] T025 Implement `ElementState` methods: `empty()`, `add()`, `remove()`, `contains()`, `toggle()`
- [x] T026 Implement `matches_pseudo_class()` method in `ElementState` for `:hover`, `:focus`, `:active`, `:disabled`, `:checked`

### Specificity Foundation

- [x] T027 Create `Specificity` struct in `crates/rvue-style/src/stylesheet/rule.rs` (id, class, element fields)
- [x] T028 Implement `PartialEq`, `Eq`, `PartialOrd`, `Ord` for `Specificity`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Apply CSS-Based Styling to Widgets (Priority: P1) 🎯 MVP

**Goal**: Enable developers to style widgets using CSS selector syntax (id, class, element selectors)

**Independent Test**: Create a widget, define CSS rules, verify styles are correctly applied

### Core Components for US1

- [ ] T029 [P] [US1] Create `RvueElement<'a>` struct in `crates/rvue-style/src/selectors/element.rs`
- [ ] T030 [P] [US1] Implement `selectors::Element` trait for `RvueElement<'a>`:
  - `parent_element()`
  - `is_root()`
  - `has_local_name()`
  - `has_class()`
  - `has_id()`
  - `attr_matches()`
  - `match_pseudo_element()` (returns false for now)
  - `match_non_ts_pseudo_class()` (returns false for now - US2)
- [ ] T031 [US1] Define `RvueSelectorImpl` struct for selector implementation in `crates/rvue-style/src/selectors/mod.rs`
- [ ] T032 [US1] Implement `SelectorImpl` for `RvueSelectorImpl` in `crates/rvue-style/src/selectors/mod.rs`

### Stylesheet Components for US1

- [ ] T033 [P] [US1] Create `StyleRule` struct in `crates/rvue-style/src/stylesheet/rule.rs`
- [ ] T034 [P] [US1] Create `Stylesheet` struct in `crates/rvue-style/src/stylesheet/mod.rs`
- [ ] T035 [US1] Implement `Stylesheet::new()`, `Stylesheet::add_rule()` methods
- [ ] T036 [US1] Implement CSS selector parsing using `selectors` crate in `Stylesheet::add_rule()`
- [ ] T037 [US1] Implement CSS property parsing using `cssparser` in `Stylesheet::add_rule()`

### ComputedStyles for US1

- [ ] T038 [P] [US1] Create `ComputedStyles` struct in `crates/rvue-style/src/properties/mod.rs`
- [ ] T039 [US1] Implement `ComputedStyles::merge()` method for property cascade
- [ ] T040 [US1] Add property fields to `ComputedStyles`: `background_color`, `color`, `font_size`, `padding`, `margin`, `width`, `height`, `display`

### StyleResolver for US1

- [ ] T041 [US1] Create `StyleResolver` struct in `crates/rvue-style/src/stylesheet/mod.rs`
- [ ] T042 [US1] Implement `StyleResolver::new()` constructor
- [ ] T043 [US1] Implement `StyleResolver::resolve()` method:
  - Create `MatchingContext`
  - Sort rules by specificity
  - Match selectors using `matches_selector()`
  - Merge properties into `ComputedStyles`

### Basic Property Implementations for US1

- [ ] T044 [P] [US1] Implement `BackgroundColor` property in `crates/rvue-style/src/properties/background.rs`
- [ ] T045 [P] [US1] Implement `TextColor` property in `crates/rvue-style/src/properties/color.rs`
- [ ] T046 [P] [US1] Implement `FontSize` property in `crates/rvue-style/src/properties/font.rs`
- [ ] T047 [P] [US1] Implement `Padding` property in `crates/rvue-style/src/properties/spacing.rs`
- [ ] T048 [P] [US1] Implement `Margin` property in `crates/rvue-style/src/properties/spacing.rs`
- [ ] T049 [P] [US1] Implement `Width` and `Height` properties in `crates/rvue-style/src/properties/sizing.rs`
- [ ] T050 [P] [US1] Implement `Display`, `FlexDirection`, `JustifyContent`, `AlignItems` in `crates/rvue-style/src/properties/layout.rs`

### Tests for US1 (Implementation Verification)

- [ ] T051 [P] [US1] Add unit tests for `Properties` container in `crates/rvue-style/tests/property_test.rs`
- [ ] T052 [P] [US1] Add unit tests for `Color` parsing in `crates/rvue-style/tests/property_test.rs`
- [ ] T053 [P] [US1] Add unit tests for `ElementState` in `crates/rvue-style/tests/selector_test.rs`
- [ ] T054 [P] [US1] Add integration test for CSS parsing in `crates/rvue-style/tests/stylesheet_test.rs`
- [ ] T055 [P] [US1] Add integration test for `StyleResolver` in `crates/rvue-style/tests/stylesheet_test.rs`

**Checkpoint**: User Story 1 complete - CSS-based styling works independently

---

## Phase 4: User Story 2 - Apply State-Based Styling (Priority: P1)

**Goal**: Enable widgets to change appearance based on interaction states (hover, focus, active)

**Independent Test**: Create interactive widgets, verify style changes on hover, focus, and other states

### State-Based Pseudo-Classes

- [ ] T056 [P] [US2] Extend `RvueElement::match_non_ts_pseudo_class()` in `crates/rvue-style/src/selectors/element.rs`:
  - Support `:hover`, `:focus`, `:active`, `:disabled`, `:checked`
  - Call `ElementState::matches_pseudo_class()` for state matching
- [ ] T057 [US2] Connect widget state to `ElementState` tracking in `RvueElement`

### Extended Property Support for US2

- [ ] T058 [P] [US2] Implement `Cursor` property in `crates/rvue-style/src/properties/mod.rs`
- [ ] T059 [P] [US2] Implement `Opacity` property in `crates/rvue-style/src/properties/visibility.rs`
- [ ] T060 [P] [US2] Implement `Visibility` enum in `crates/rvue-style/src/properties/visibility.rs`
- [ ] T061 [P] [US2] Implement `BorderColor`, `BorderWidth`, `BorderRadius`, `BorderStyle` in `crates/rvue-style/src/properties/border.rs`

### CSS Value Parsing for US2

- [ ] T062 [US2] Implement CSS length parsing in `crates/rvue-style/src/css/value_parser.rs` (px, %, em, rem)
- [ ] T063 [US2] Implement CSS color value parsing (rgba, hsla, named colors) in `crates/rvue-style/src/css/value_parser.rs`
- [ ] T064 [US2] Implement CSS border value parsing in `crates/rvue-style/src/css/properties.rs`

### Tests for US2

- [ ] T065 [P] [US2] Add unit tests for pseudo-class matching in `crates/rvue-style/tests/selector_test.rs`
- [ ] T066 [P] [US2] Add unit tests for border properties in `crates/rvue-style/tests/property_test.rs`
- [ ] T067 [P] [US2] Add integration test for state-based styling in `crates/rvue-style/tests/stylesheet_test.rs`

**Checkpoint**: User Story 2 complete - state-based styling works alongside US1

---

## Phase 5: User Story 3 - Type-Safe Property Styling (Priority: P2)

**Goal**: Enable compile-time type checking for styles through Rust's type system

**Independent Test**: Attempt to use invalid property types, verify compilation failures

### Type Safety Infrastructure

- [ ] T068 [P] [US3] Create `StyledWidgetExt` trait in `crates/rvue-style/src/widget/styled.rs`
- [ ] T069 [P] [US3] Implement `with_style<P: Property>()` method in `StyledWidgetExt`
- [ ] T070 [P] [US3] Implement builder methods in `StyledWidgetExt`:
  - `style_background()`
  - `style_color()`
  - `style_padding()`
  - `style_margin()`
  - `style_font_size()`
  - `style_width()`
  - `style_height()`

### Property Documentation

- [ ] T071 [US3] Add comprehensive doc comments for all property types in their respective files
- [ ] T072 [US3] Add examples in property doc comments showing proper usage
- [ ] T073 [US3] Document error cases and what happens with invalid types

### Tests for US3 (Type Safety Verification)

- [ ] T074 [P] [US3] Verify compilation failures for invalid property combinations (manual test, not automated)
- [ ] T075 [P] [US3] Add unit tests for `StyledWidgetExt` API in `crates/rvue-style/tests/property_test.rs`

**Checkpoint**: User Story 3 complete - type-safe API works alongside US1-US2

---

## Phase 6: User Story 4 - Reactive Style Updates (Priority: P2)

**Goal**: Enable styles to update automatically when bound signals change

**Independent Test**: Create widget with signal-based styles, verify automatic updates when signals change

### Reactive Property System

- [ ] T076 [P] [US4] Create `ReactiveProperty<T>` enum in `crates/rvue-style/src/reactive/mod.rs`:
  ```rust
  pub enum ReactiveProperty<T: Clone + 'static> {
      Static(T),
      Signal(ReadSignal<T>),
  }
  ```
- [ ] T077 [P] [US4] Implement `ReactiveProperty::get()` and `ReactiveProperty::is_reactive()` methods
- [ ] T078 [P] [US4] Implement `From<T>` and `From<ReadSignal<T>>` for `ReactiveProperty<T>`
- [ ] T079 [US4] Create `create_style_effect()` function in `crates/rvue-style/src/reactive/style_signal.rs`
- [ ] T080 [US4] Create `derive_style()` function in `crates/rvue-style/src/reactive/style_signal.rs`

### Signal Integration

- [ ] T081 [US4] Integrate `ReactiveProperty` with `Properties` container in `crates/rvue-style/src/property.rs`
- [ ] T082 [US4] Update `ComputedStyles` to handle reactive properties in `crates/rvue-style/src/properties/mod.rs`

### Tests for US4

- [ ] T083 [P] [US4] Add unit tests for `ReactiveProperty` in `crates/rvue-style/tests/property_test.rs`
- [ ] T084 [P] [US4] Add integration test for reactive style updates in `crates/rvue-style/tests/stylesheet_test.rs`

**Checkpoint**: User Story 4 complete - reactive styling works alongside US1-US3

---

## Phase 7: User Story 5 - GC-Compatible Style Sharing (Priority: P3)

**Goal**: Enable shared style definitions across multiple widgets for memory efficiency

**Independent Test**: Create many widgets sharing same style, verify memory usage is reasonable

### GC Integration

- [ ] T085 [P] [US5] Implement `rudo_gc::Trace` for all property types in their respective files
- [ ] T086 [P] [US5] Implement `rudo_gc::Trace` for `Properties`, `ComputedStyles`, `Stylesheet`
- [ ] T087 [P] [US5] Implement `Gc<ComputedStyles>` for shared styles in `crates/rvue-style/src/properties/mod.rs`
- [ ] T088 [US5] Create `SharedStyles` type alias in `crates/rvue-style/src/properties/mod.rs`

### Shared Style Implementation

- [ ] T089 [US5] Create `WidgetStyles` struct in `crates/rvue-style/src/widget/styled.rs`:
  ```rust
  pub struct WidgetStyles {
      base: SharedStyles,
      overrides: Properties,
  }
  ```
- [ ] T090 [US5] Implement `WidgetStyles::new()`, `WidgetStyles::with_override()`, `WidgetStyles::computed()`
- [ ] T091 [US5] Implement `ComputedStyles::shared()` method for creating `Gc<ComputedStyles>`

### Tests for US5

- [ ] T092 [P] [US5] Add unit tests for `Gc` compatibility in `crates/rvue-style/tests/property_test.rs`
- [ ] T093 [P] [US5] Add integration test for shared styles in `crates/rvue-style/tests/stylesheet_test.rs`

**Checkpoint**: User Story 5 complete - GC-compatible sharing works alongside all stories

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Documentation

- [ ] T094 [P] Add module-level documentation in all `mod.rs` files
- [ ] T095 [P] Add API documentation to all public types and functions
- [ ] T096 Update `crates/rvue-style/src/lib.rs` with crate-level documentation

### Examples

- [ ] T097 [P] Complete `crates/rvue-style/examples/basic_styling.rs` demonstrating basic property usage
- [ ] T098 [P] Complete `crates/rvue-style/examples/stylesheet.rs` demonstrating CSS styling
- [ ] T099 [P] Complete `crates/rvue-style/examples/reactive_styles.rs` demonstrating reactive styling

### Performance & Validation

- [ ] T100 [P] Run `cargo fmt` on all generated code
- [ ] T101 [P] Run `cargo clippy` and fix any warnings
- [ ] T102 [P] Run all tests with `cargo test -- --test-threads=1`
- [ ] T103 Validate quickstart.md examples work correctly

### Cleanup

- [ ] T104 Remove any TODO comments that are now implemented
- [ ] T105 Ensure all public APIs have proper error handling with `Result` types

---

## Dependencies & Execution Order

### Phase Dependencies

| Phase | Depends On | Blocks |
|-------|------------|--------|
| Phase 1: Setup | None | Foundational |
| Phase 2: Foundational | Setup | All user stories |
| Phase 3: US1 (P1) | Foundational | Phase 8 (Polish) |
| Phase 4: US2 (P1) | Foundational | Phase 8 (Polish) |
| Phase 5: US3 (P2) | Foundational | Phase 8 (Polish) |
| Phase 6: US4 (P2) | Foundational | Phase 8 (Polish) |
| Phase 7: US5 (P3) | Foundational | Phase 8 (Polish) |
| Phase 8: Polish | All user stories | Done |

### User Story Dependencies

- **User Story 1 (P1)**: Requires Phase 2 completion - No other story dependencies
- **User Story 2 (P1)**: Requires Phase 2 completion - Extends US1 selector functionality
- **User Story 3 (P2)**: Requires Phase 2 completion - Independent of US1-US2
- **User Story 4 (P2)**: Requires Phase 2 completion - Independent of US1-US3
- **User Story 5 (P3)**: Requires Phase 2 completion - Independent of US1-US4

### Within Each User Story

- Foundational tasks (T014-T027) must complete first
- Core data structures before implementations
- Properties before stylesheet
- Stylesheet before resolver
- Tests before implementation (verification)

---

## Parallel Execution Examples

### Phase 1: Setup (All Tasks Can Run in Parallel)

```bash
# Create all module files in parallel:
Task: "Create properties/ module files"
Task: "Create selectors/ module files"
Task: "Create stylesheet/ module files"
Task: "Create css/ module files"
Task: "Create reactive/ module files"
Task: "Create widget/ module files"
```

### Phase 2: Foundational (Parallel Within Categories)

```bash
# Property system tasks:
Task: "Create Property trait"
Task: "Implement Properties struct"
Task: "Implement rudo_gc::Trace"

# CSS value parsing tasks:
Task: "Create Color struct"
Task: "Implement Size enum"
Task: "Implement Color parsing"
```

### User Story 1 (Parallel Opportunities)

```bash
# All US1 property implementations can run in parallel:
Task: "Implement BackgroundColor property"
Task: "Implement TextColor property"
Task: "Implement FontSize property"
Task: "Implement Padding property"
Task: "Implement Margin property"
Task: "Implement Width/Height properties"
Task: "Implement layout properties"
```

### User Stories Can Run in Parallel

Once Phase 2 is complete:
- Developer A: User Story 1
- Developer B: User Story 2
- Developer C: User Story 3
- Developer D: User Story 4
- Developer E: User Story 5

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test CSS-based styling independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add User Story 5 → Test independently → Deploy/Demo
7. Add Polish → Final release

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3 + 4
   - Developer D: User Story 5 + Polish
3. Stories complete and integrate independently

---

## Summary

| Metric | Value |
|--------|-------|
| **Total Tasks** | 105 |
| **Phase 1: Setup** | 13 tasks |
| **Phase 2: Foundational** | 14 tasks |
| **Phase 3: US1 (P1) - MVP** | 26 tasks |
| **Phase 4: US2 (P1)** | 14 tasks |
| **Phase 5: US3 (P2)** | 8 tasks |
| **Phase 6: US4 (P2)** | 9 tasks |
| **Phase 7: US5 (P3)** | 9 tasks |
| **Phase 8: Polish** | 12 tasks |

### Parallel Opportunities

- All setup tasks (T001-T013) marked [P]
- Foundational tasks within categories marked [P]
- Property implementations within each story marked [P]
- Tests for each story marked [P]

### MVP Scope (User Story 1 Only)

To deliver minimum viable product, complete:
- Phase 1: T001-T013
- Phase 2: T014-T027
- Phase 3: T029-T055

**Total: 55 tasks** for MVP

### Independent Test Criteria

Each user story can be verified independently:
- **US1**: Widget with CSS ID/class selectors renders with correct styles
- **US2**: Widget changes appearance on hover/focus/active states
- **US3**: Type system catches invalid property types at compile time
- **US4**: Signal-bound styles update automatically when values change
- **US5**: Shared styles reduce memory usage for many widgets
