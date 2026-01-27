<!--
Sync Impact Report
==================
Version Change: N/A → 1.0.0 (Initial constitution creation)

Modified Principles:
- N/A (first constitution)

Added Sections:
- Core Principles (5 principles)
  - I. Code Quality
  - II. Testing Standards
  - III. User Experience Consistency
  - IV. Performance Requirements
  - V. Safety and Correctness
- Additional Constraints
  - Technology Stack
  - Dependency Management
- Development Workflow
  - Code Review Requirements
  - Quality Gates
  - Commit Practices
- Governance
  - Amendment Procedure
  - Compliance Review
  - Runtime Guidance

Removed Sections:
- N/A (first constitution)

Templates Requiring Updates:
- .specify/templates/plan-template.md ✅ No changes needed (Constitution Check section already flexible)
- .specify/templates/spec-template.md ✅ No changes needed (User scenarios and testing already covered)
- .specify/templates/tasks-template.md ✅ No changes needed (Testing organization already flexible)

Follow-up TODOs:
- N/A (all placeholders filled)
-->

# Rvue Constitution

## Core Principles

### I. Code Quality

All code MUST adhere to Rust best practices and produce clean, maintainable codebases.

- Follow Rust standard conventions (RFC 1437) for naming, formatting, and structure.
- Write self-documenting code with clear intent and descriptive names.
- Use exhaustive pattern matching for enums to catch missing cases at compile time.
- Prefer Result/Option over unwrap in public APIs; include context in error messages.
- Max line width: 100 characters; indentation: 4 spaces (no tabs).
- All public APIs MUST have documentation comments explaining behavior and edge cases.

Rationale: A high-quality codebase reduces bugs, lowers maintenance cost, and improves developer experience. Rust's type system and compiler enforce safety when used correctly.

### II. Testing Standards

All features MUST be tested comprehensively to ensure correctness and prevent regressions.

- Unit tests MUST be placed in `tests/unit/` directory with naming convention `test_<feature>_<behavior>`.
- Integration tests MUST be placed in `tests/` root directory.
- All public APIs MUST have corresponding tests verifying behavior.
- Tests MUST use appropriate assertions (`assert_eq!`, `assert!`, `assert_ne!`).
- Tests run single-threaded via `cargo test -- --test-threads=1` to ensure deterministic behavior.
- A feature is NOT complete until all associated tests pass.

Rationale: Comprehensive testing catches bugs early, enables safe refactoring, and provides confidence in code correctness. Tests serve as living documentation of expected behavior.

### III. User Experience Consistency

The framework MUST provide a consistent, Vue-like developer experience across all APIs.

- Follow Vue's reactivity model: `create_signal`, `create_effect`, and declarative `view!` syntax.
- API naming MUST be consistent: snake_case for functions/variables, PascalCase for types/traits.
- Provide a prelude module (`rvue::prelude::*`) for commonly used types and traits.
- Component props MUST follow a predictable pattern: derive from simple data types.
- Error messages MUST be actionable and include context: `"Failed to X: {details}"`.

Rationale: Consistency reduces cognitive load for developers. Vue-inspired patterns are familiar to many developers, lowering the learning curve for Rvue.

### IV. Performance Requirements

The framework MUST deliver native performance through GPU acceleration and minimal overhead.

- Hot-path functions MUST use `#[inline(always)]` to eliminate function call overhead.
- Use `AtomicU64` with `Ordering::SeqCst` for version tracking in reactive primitives.
- Fine-grained reactivity MUST avoid full-tree re-renders; only changed elements update.
- Memory management MUST use `Gc<T>` and `GcCell<T>` appropriately for garbage-collected types.
- Release all borrows before running effects to prevent borrow checker conflicts.
- Benchmark applications (`cargo run --bin benchmark`) MUST validate startup and memory performance.

Rationale: Performance is a core value proposition of Rvue. GPU-accelerated rendering and fine-grained reactivity must not be undermined by inefficient code patterns.

### V. Safety and Correctness

The framework MUST leverage Rust's safety guarantees and prevent undefined behavior.

- NO unsafe code allowed except in FFI boundaries or when absolutely required for performance.
- All public APIs MUST return `Result` or `Option` for fallible operations.
- Use `thiserror` or manual `impl Error` for custom error types.
- Shared state MUST use proper synchronization or garbage-collected types.
- Memory leaks MUST be minimized; GC cycles should complete in reasonable time.

Rationale: Rust's primary advantage is memory safety without garbage collection. The framework must honor this contract while providing ergonomic APIs.

## Additional Constraints

### Technology Stack

- Language: Rust 2021 Edition (minimum 1.75+)
- Rendering: Vello for GPU-accelerated 2D vector graphics
- Layout: Taffy for CSS-like Flexbox and Grid layouts
- Garbage Collection: rudo-gc for hybrid GC implementation
- Edition: 2021 for async/await and other modern Rust features

### Dependency Management

- External dependencies MUST be reviewed for security, maintenance status, and performance impact.
- Prefer well-maintained crates with good documentation and test coverage.
- New dependencies require justification in pull request description.

## Development Workflow

### Code Review Requirements

- All changes MUST be reviewed before merging to main branch.
- Reviewers MUST verify compliance with constitution principles.
- Clippy warnings MUST be addressed (use `cargo clippy --fix`).
- Code MUST be formatted with `cargo fmt` before submission.

### Quality Gates

- `cargo build` MUST succeed without errors.
- `cargo test -- --test-threads=1` MUST pass all tests.
- `cargo clippy` MUST report no warnings (or be justified in PR).
- `cargo fmt --check` MUST show no formatting deviations.

### Commit Practices

- Commits SHOULD be atomic and describe a complete change.
- Commit messages SHOULD follow conventional commit format.
- Each commit SHOULD leave the codebase in a working state.

## Governance

This constitution supersedes all other development practices and guidelines. All team members and contributors MUST adhere to these principles.

### Amendment Procedure

1. Proposed amendments MUST be documented with rationale and impact analysis.
2. Amendments MUST be reviewed and approved by core maintainers.
3. Breaking changes to existing principles require MAJOR version bump.
4. New principles or expanded guidance require MINOR version bump.
5. Clarifications, wording fixes, or non-semantic refinements require PATCH version bump.

### Compliance Review

- All pull requests MUST include a self-review against constitution principles.
- Reviewers MUST verify each applicable principle is satisfied.
- Violations MUST be justified in the PR with complexity tracking documentation.

### Runtime Guidance

For day-to-day development guidance, refer to `AGENTS.md` which contains:
- Build commands and testing procedures
- Code style guidelines and naming conventions
- Common patterns for signals, effects, and components
- Project structure documentation

**Version**: 1.0.0 | **Ratified**: 2026-01-27 | **Last Amended**: 2026-01-27
