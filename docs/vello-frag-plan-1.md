# Vello Fragment Optimization Plan

**Status**: Draft
**Date**: 2026-01-25
**Context**: Optimization of `Scene::update` mechanism to avoid full tree rebuilding on every frame.

---

## 1. Executive Summary

Current implementation of `Scene::update` in Rvue performs a partial optimization (fine-grained layout updates) but forces a full **Render Reset** (`scene.reset()`) every frame if any component is dirty.

This plan details the strategy to implement **Fragment Caching** and **Incremental Composition**. Based on expert review (Alex Crichton, Ryan Carniato, Evan You), we have decided to reject "Byte-Offset Modification" in favor of a safer, architectural approach: **"Skip Clean Components"** initially, followed by **"Layered Composition"**.

---

## 2. Decision Record (Q&A)

### Q1: Byte-offset Tracking vs. Skip Clean Components?

**Decision: "Skip Clean Components" (Fragment Caching)**

We explicitly **REJECT** the idea of tracking byte-offsets in the Vello encoding stream to perform in-place surgical edits.

*   **Safety (Alex Crichton)**: Vello's `Encoding` stream is an internal implementation detail without a stable ABI. Directly manipulating the command stream requires `unsafe` code and creates tight coupling with Vello versions. It is high risk and high maintenance.
*   **Performance (Ryan Carniato)**: The goal of fine-grained reactivity is to minimize *user code execution* and *expensive resource generation* (like text layout and path tessellation). "Skip Clean" achieves 99% of this benefit. The remaining cost is just traversing the component tree and appending pre-built fragments (`scene.append`), which is effectively a memory copy operation and very fast.
*   **Pragmatism (Evan You)**: "Premature optimization is the root of all evil." We will implement the stable "Skip Clean" approach first.

### Q2: Vello API Constraints on Fragment Removal?

**Constraint: Append-Only Architecture**

*   **Immutability**: `vello::Scene` is fundamentally an append-only command list. You cannot remove an instruction from the middle of the stream.
*   **Removal = Rebuild**: Unlike DOM nodes which can be `removeChild()`, removing a Vello fragment physically requires rebuilding the parent scene without that fragment's instructions.
*   **Conclusion**: We must accept that `Root Scene::reset()` is necessary. The optimization lies in **how fast we can refill it**.

---

## 3. Implementation Strategy

### Phase 1: Fragment Caching (Current Goal)

The objective is to ensure that if a component is not dirty, we reuse its previously encoded `vello::Scene` (Fragment) instead of asking it to draw again.

#### Component Architecture Changes

```rust
pub struct Component {
    // ... existing fields ...
    
    /// Cached Vello scene fragment for this component (and its subtree)
    pub vello_cache: RefCell<Option<SceneWrapper>>,
    
    /// Dirty flag specifically for rendering (distinct from layout dirty?)
    /// Currently we share `is_dirty`, might need separation if layout is clean but color changes.
    pub is_dirty: AtomicBool, 
}
```

#### Render Loop Logic (`Scene::update`)

1.  **Iterate Root Components**:
    Start traversing the top-level component tree.
    
2.  **Check Dirty State**:
    For each component:
    *   **If Dirty**: 
        1. Create a new `vello::Scene` (Local Fragment).
        2. Execute `render_component(..., &mut local_scene)`.
        3. Store `local_scene` in `component.vello_cache`.
        4. Append `local_scene` to the Root Scene.
    *   **If Clean**:
        1. Retrieve `cached_scene` from `component.vello_cache`.
        2. Append `cached_scene` to the Root Scene (`scene.append(&cached_scene, transform)`).
        3. **CRITICAL**: Do NOT traverse children. The `cached_scene` already contains the encoded drawing commands for the entire subtree.

#### Edge Cases
*   **Transform Updates**: If a component moved (layout changed) but didn't change appearance, we still use the cached fragment but apply a different `Affine` transform during append.
*   **Context Propagation**: If a parent context changes (e.g., Theme), the entire subtree needs to be marked dirty to regenerate fragments with new colors.

---

## 4. Phase 2: Layered Composition (Future Optimization)

To further reduce the cost of `scene.append` for large static backgrounds, we will introduce **Layered Composition**.

#### Concept
Separate the application into multiple `vello::Scene` instances based on update frequency.

```rust
pub struct Scene {
    pub static_layer: vello::Scene,   // Backgrounds, Sidebars (Updates rarely)
    pub dynamic_layer: vello::Scene,  // Cursors, Hover effects (Updates frequently)
}
```

#### Logic
1.  Components are tagged (implicitly or explicitly) with a Layer ID.
2.  When `dynamic_layer` components update:
    *   We **only reset and rebuild** `dynamic_layer`.
    *   `static_layer` is preserved as-is.
3.  During final rendering to `RenderSurface`, we composite both scenes.

*Note: This phase will only be implemented after Phase 1 is stable and profiled.*
