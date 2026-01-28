# Implementation Plan: Rvue Style System

**Branch**: `001-style-system` | **Date**: 2026-01-27 | **Spec**: [link](spec.md)
**Input**: Feature specification from `/specs/001-style-system/spec.md`

## Summary

The rvue style system implements a hybrid approach combining the Stylo selectors crate for CSS selector parsing/matching with Xilem/Masonry-style type-safe property system. Key components include:
- **Property trait** (compile-time type safety)
- **Selectors integration** (CSS selector matching with pseudo-classes)
- **Signal integration** (reactive style updates)
- **rudo-gc integration** (memory management)

## Technical Context

**Language/Version**: Rust 2021 Edition (minimum 1.75+)
**Primary Dependencies**: `selectors = "0.35"` (Stylo), `cssparser = "0.36"`, `bitflags = "2"`, `rudo-gc`
**Storage**: N/A (in-memory style computation)
**Testing**: `cargo test -- --test-threads=1` per AGENTS.md
**Target Platform**: Desktop GUI (Vello rendering via Taffy layout)
**Project Type**: Rust library crate (`crates/rvue-style/`)
**Performance Goals**: 60 FPS with <16ms per frame for style matching on 1000 widgets
**Constraints**: GC-compatible, type-safe, minimal dependencies (selectors crate only, not full Stylo)
**Scale/Scope**: Core properties, selectors integration, CSS parsing, widget integration (4 phases)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | Will follow Rust standard conventions, use exhaustive pattern matching |
| II. Testing Standards | ✅ PASS | Will place unit tests in `tests/unit/`, integration tests in `tests/` root |
| III. User Experience Consistency | ✅ PASS | Follows Vue-like reactivity model, consistent naming (snake_case/PascalCase) |
| IV. Performance Requirements | ✅ PASS | Hot-path functions will use `#[inline(always)]`, fine-grained reactivity |
| V. Safety and Correctness | ✅ PASS | No unsafe code (except GC trait bounds), Result/Option for fallible ops |

**GATE RESULT**: ✅ PASS - Ready for implementation

## Project Structure

### Documentation (this feature)

```text
specs/001-style-system/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (research on Stylo/Xilem patterns)
├── data-model.md        # Phase 1 output (entities: Property, StyleRule, Stylesheet)
├── quickstart.md        # Phase 1 output (developer guide)
├── contracts/           # Phase 1 output (API specifications)
│   └── api.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/rvue-style/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── property.rs           # Property trait and Properties container
│   ├── properties/
│   │   ├── mod.rs
│   │   ├── background.rs
│   │   ├── border.rs
│   │   ├── color.rs
│   │   ├── font.rs
│   │   ├── layout.rs
│   │   ├── sizing.rs
│   │   ├── spacing.rs
│   │   └── visibility.rs
│   ├── selectors/
│   │   ├── mod.rs
│   │   ├── element.rs        # RvueElement implementation
│   │   └── state.rs          # ElementState tracking
│   ├── stylesheet/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── rule.rs
│   ├── css/
│   │   ├── mod.rs
│   │   ├── value_parser.rs
│   │   └── properties.rs
│   ├── reactive/
│   │   ├── mod.rs
│   │   └── style_signal.rs
│   └── widget/
│       ├── mod.rs
│       └── styled.rs
├── tests/
│   ├── property_test.rs
│   ├── selector_test.rs
│   └── stylesheet_test.rs
└── examples/
    ├── basic_styling.rs
    ├── stylesheet.rs
    └── reactive_styles.rs
```

**Structure Decision**: Rust library crate structure following Xilem/Masonry patterns. Modular organization by concern (properties, selectors, stylesheet, css, reactive, widget).

## Phase 0: Research - COMPLETE

### Research Outcomes

| Decision | Outcome |
|----------|---------|
| Property Trait Design | Adopt Xilem/Masonry `Property` trait with `static_default()` |
| Properties Container | Custom `Properties` with `rudo_gc::Trace` compatible `AnyMap` |
| Selectors Integration | Use Stylo `selectors` crate with custom `RvueElement` wrapper |
| ElementState Mapping | Map CSS pseudo-classes to bitflags-based `ElementState` |
| CSS Parsing | Use `cssparser` crate for value parsing |
| Reactive Integration | `ReactiveProperty<T>` enum supporting static and signal values |
| Cascade Resolution | CSS specificity ordering with computed styles merge |

### Reference Implementations

- Xilem/Masonry: `learn-projects/xilem/masonry_core/src/core/properties.rs`
- Xilem/Masonry: `learn-projects/xilem/masonry/src/properties/background.rs`
- Stylo Selectors: `learn-projects/stylo/selectors/lib.rs`
- Stylo Selectors: `learn-projects/stylo/selectors/matching.rs`

## Phase 1: Design - COMPLETE

### Data Model Entities

| Entity | Purpose | Key Attributes |
|--------|---------|----------------|
| `Property` (trait) | Type-safe styling marker | `static_default()` |
| `Properties` | Property container | `map: AnyMap` |
| `RvueElement<'a>` | Selector matching wrapper | `widget`, `parent` |
| `StyleRule` | CSS rule with selector | `selector`, `specificity`, `properties` |
| `Stylesheet` | Rule collection | `rules`, `media_queries`, `keyframes` |
| `ElementState` | Widget state flags | `HOVER`, `FOCUS`, `ACTIVE`, etc. |
| `ComputedStyles` | Resolved styles | All property values |
| `StyleResolver` | Rule matching engine | `stylesheet`, `caches` |

### API Contracts

Generated contracts in `contracts/api.md`:
- Property module API
- Properties module API (Color, Spacing, Layout, Sizing)
- Selectors module API (ElementState, RvueElement)
- Stylesheet module API (Stylesheet, StyleResolver)
- Reactive module API (ReactiveProperty)
- Widget module API (StyledWidgetExt)

### Quickstart Guide

Created `quickstart.md` with:
- Dependency setup
- Basic styling patterns
- CSS stylesheet creation
- State-based styling
- Color values
- Layout properties
- Reactive styles
- Widget integration examples

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

## Post-Design Constitution Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ PASS | All patterns follow Rust conventions |
| II. Testing Standards | ✅ PASS | Test structure defined in plan |
| III. User Experience Consistency | ✅ PASS | Vue-like API patterns maintained |
| IV. Performance Requirements | ✅ PASS | Performance goals specified (<16ms/frame) |
| V. Safety and Correctness | ✅ PASS | No unsafe code, proper error handling |

**RECHECK RESULT**: ✅ PASS - Design satisfies all principles

## Implementation Tasks

*To be generated by `/speckit.tasks` command*

Recommended task breakdown:
1. Phase 1: Core Property System (Property trait, Properties container, core properties)
2. Phase 2: Selectors Integration (RvueElement, ElementState, pseudo-class matching)
3. Phase 3: CSS Parsing (stylesheet parser, value parsers, specificity)
4. Phase 4: Widget Integration (StyledWidgetExt, style effects, examples)

## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Begin Phase 1: Core Property System implementation
3. Add unit tests for property system
4. Integrate with rvue widget system
