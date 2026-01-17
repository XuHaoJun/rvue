# Implementation Plan: Rvue MVP Framework

**Branch**: `002-rvue-mvp` | **Date**: 2026-01-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-rvue-mvp/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.cursor/commands/speckit.plan.md` for the execution workflow.

## Summary

Build a Rust GUI framework MVP that enables web developers to create native desktop applications with familiar Vue/React-like syntax. The framework combines rudo-gc (easy-oilpan garbage collection) for automatic memory management, Leptos-style view! macros for declarative UI, Vello for GPU-accelerated rendering, and Taffy for flexbox/grid layouts. The goal is to replace Electron/Tauri webview-based solutions with faster startup times, lower memory usage, and a more intuitive developer experience while maintaining web-like development patterns.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust (latest stable, minimum 1.75+)  
**Primary Dependencies**: 
- `rudo-gc` (from learn-projects/rudo): Garbage collection with Gc<T> and Trace derive
- `vello`: GPU-accelerated 2D rendering engine
- `taffy`: Flexbox and Grid layout engine
- `winit`: Cross-platform window management
- `wgpu` or `vello` backend: GPU rendering backend
- Procedural macros: Custom view! macro similar to Leptos

**Storage**: N/A (in-memory UI state managed by GC)  
**Testing**: `cargo test` with unit tests, integration tests, and example applications  
**Target Platform**: Desktop applications on Windows, macOS, and Linux  
**Project Type**: Single project (desktop framework library)  
**Performance Goals**: 
- 60fps UI updates (16ms frame budget)
- <2 second application startup time
- <100MB initial memory footprint for simple applications
- At least 50% faster startup than equivalent Electron/Tauri applications

**Constraints**: 
- Single-threaded UI operations (all UI work on main thread)
- GC-managed memory for UI components (no manual cleanup)
- Native compilation (not webview-based)
- MVP scope: single-window applications only

**Scale/Scope**: 
- MVP supports basic components (text, button, input, checkbox, radio)
- Conditional rendering (Show component)
- List rendering (For component)
- Flexbox/Grid layouts
- Reactive signals and effects
- Component composition

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I. Code Quality**: Design uses established patterns (Leptos macros, rudo-gc) and follows Rust best practices. Code will be modular with clear separation between reactivity, layout, and rendering layers.
- [x] **II. Testing**: Automated tests planned for: signal reactivity, component rendering, layout calculations, macro expansion, GC memory management. Integration tests via example applications (counter, list view).
- [x] **III. UX Consistency**: Framework provides consistent API following Vue Composition API patterns. Developers implement their own UI, but framework ensures consistent reactive behavior and layout semantics.
- [x] **IV. Performance**: Performance targets defined: 60fps (16ms), <2s startup, <100MB memory. GC incremental marking may be needed to prevent frame drops (deferred to Phase 2 optimization).
- [x] **V. Maintainability**: Solution is modular (reactivity, layout, rendering, macros as separate concerns). Full Spec-Plan-Task documentation workflow followed.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── rvue/                    # Main framework crate
│   ├── src/
│   │   ├── lib.rs           # Public API exports
│   │   ├── signal.rs        # Reactive signal implementation
│   │   ├── effect.rs        # Effect system for reactive updates
│   │   ├── component.rs      # Component trait and lifecycle
│   │   ├── view.rs          # View trait and rendering
│   │   ├── layout/          # Taffy integration layer
│   │   │   ├── mod.rs
│   │   │   └── node.rs      # Layout node wrapper
│   │   ├── render/          # Vello integration layer
│   │   │   ├── mod.rs
│   │   │   ├── scene.rs     # Scene graph management
│   │   │   └── widget.rs    # Widget-to-Vello mapping
│   │   ├── widgets/         # Built-in widget components
│   │   │   ├── mod.rs
│   │   │   ├── text.rs
│   │   │   ├── button.rs
│   │   │   ├── input.rs
│   │   │   ├── checkbox.rs
│   │   │   ├── radio.rs
│   │   │   ├── show.rs      # Conditional rendering
│   │   │   └── for_loop.rs  # List rendering
│   │   └── style.rs         # Styling system
│   └── tests/
│       ├── integration/
│       └── unit/
├── rvue-macro/              # Procedural macro crate
│   ├── src/
│   │   ├── lib.rs           # Macro entry points
│   │   ├── view.rs          # view! macro implementation
│   │   ├── component.rs     # component attribute macro
│   │   └── parse.rs         # DSL parsing
│   └── tests/
└── rvue-examples/           # Example applications
    ├── counter/             # Basic counter example
    ├── list/                # List rendering example
    └── layout/               # Layout example

tests/
├── integration/             # Integration tests
└── unit/                    # Unit tests for framework components
```

**Structure Decision**: Single project structure with multiple crates in a workspace. The main `rvue` crate provides the framework API, `rvue-macro` handles procedural macros, and `rvue-examples` contains demonstration applications. This separation allows independent versioning and compilation of macros vs runtime code.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. All Constitution principles are satisfied:
- Code quality: Uses established patterns, modular design
- Testing: Comprehensive test plan for all components
- UX consistency: Framework provides consistent API patterns
- Performance: Targets defined and achievable
- Maintainability: Full Spec-Plan-Task documentation

## Phase 0: Research Complete

**Status**: ✅ Complete  
**Output**: `research.md`

All technical unknowns resolved:
- GC integration strategy with rudo-gc
- Macro implementation approach (Leptos-style)
- Vello and Taffy integration patterns
- Signal/Effect system design
- Component architecture decisions
- Threading model
- Styling system
- Error handling approach

## Phase 1: Design Complete

**Status**: ✅ Complete  
**Outputs**:
- `data-model.md`: Core entities and relationships defined
- `contracts/api.md`: Public API contracts documented
- `quickstart.md`: Developer onboarding guide created
- Agent context updated: Cursor IDE context file created

**Key Design Artifacts**:
- Component lifecycle and state transitions
- Signal dependency graph model
- Layout and rendering integration points
- Styling system architecture
- Error handling patterns

## Next Steps

1. **Phase 2**: Run `/speckit.tasks` to break down implementation into tasks
2. **Phase 3**: Begin implementation following task breakdown
3. **Testing**: Implement tests alongside features (TDD approach)
4. **Documentation**: Keep documentation in sync with implementation

## Implementation Notes

- Start with core signal/effect system (foundation)
- Implement view! macro incrementally (start with static components)
- Add widget components one at a time (Text, Button, then inputs)
- Layout and rendering integration can be done in parallel
- Example applications serve as integration tests
