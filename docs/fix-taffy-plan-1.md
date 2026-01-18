# Rvue Fix Technical Specification - Taffy & WGPU Lifecycle

This document outlines the technical strategy for fixing the Taffy SlotMap panic, the WGPU resource cleanup segmentation fault, and improving the rendering update path.

## 1. Taffy Tree Persistence & Synchronization

### Issue
The `taffy.clear()` call in `Scene::update` invalidates all `NodeId`s stored in `Component`s. When `build_layout_tree` is called, it might inadvertently rely on old state or fail to properly reset the layout node references in the component tree, leading to `invalid SlotMap key used`.

### Proposed Fix
*   **Persistent Nodes**: Modify `build_layout_tree` to reuse existing Taffy `NodeId`s if they exist in the component.
*   **Explicit Removal**: Implement a `Component::cleanup_layout` method that recursively removes nodes from the Taffy tree when a component is unmounted or removed from the tree, instead of clearing the entire tree every frame.
*   **Dirty Tracking**: Only call `taffy.compute_layout` if the component or its children are actually marked as dirty.

### Implementation Details
1.  Update `Scene::update` to remove `self.taffy.clear()`.
2.  In `component.rs`, modify `build_layout_tree` to:
    *   Check if `component.layout_node()` already has a valid `taffy_node`.
    *   If yes, update the node's style instead of creating a new one.
    *   If no, create a new node.
3.  Ensure `propagate_layout_results` is only called on branches that were actually recalculated.

## 2. WGPU/Vello Resource Lifecycle Management

### Issue
The segmentation fault and "SurfaceAcquireSemaphores in use" panic occur because `wgpu` resources are being dropped in an incorrect order or while the GPU still has pending work on the surface texture.

### Proposed Fix
*   **Explicit Cleanup**: Implement an explicit `cleanup` method for `AppState` (or a `Drop` implementation) that ensures resources are released in the correct order:
    1.  `renderer` (Vello Renderer)
    2.  `surface` (RenderSurface/wgpu::Surface)
    3.  `render_cx` (RenderContext/wgpu::Device/Queue)
*   **Wait for GPU**: Ensure `device.poll(wgpu::PollType::Wait)` is called before dropping the surface, ensuring all frames are presented and semaphores are released.
*   **SurfaceTexture Scope**: Ensure `SurfaceTexture` acquired in `render_frame` is dropped *before* any potentially destructive operations on the surface.

### Implementation Details
1.  Modify `AppState` to have a `ManualDrop` wrapper or carefully ordered fields. Rust drops fields in the order they are declared.
2.  Order fields as:
    ```rust
    struct AppState {
        // ...
        renderer: Option<Renderer>,      // Drop first
        surface: Option<RenderSurface>,  // Drop second
        render_cx: Option<RenderContext>, // Drop last
        window: Option<Arc<Window>>,     // Window must outlive surface
    }
    ```
3.  In `ApplicationHandler::exiting` (or similar cleanup hook), call `device.poll(wgpu::PollType::Wait)`.

## 3. Fine-grained Update Path (Vello Scene)

### Issue
Currently, `scene.reset()` is called every frame if anything is dirty, discarding all Vello encoding work.

### Proposed Fix
*   **Fragment Caching**: `VelloFragment` should store its own `vello::Scene` (as a sub-scene).
*   **Partial Re-encoding**:
    *   If a `Component` is dirty, only its corresponding `VelloFragment`'s internal scene is re-encoded.
    *   The main `Scene` then simply appends these cached sub-scenes with appropriate transforms.
*   **Scene Composition**: Use `vello::Scene::append` to compose the final scene from fragments.

### Implementation Details
1.  Update `VelloFragment` to hold a `vello::Scene`.
2.  Add a `VelloFragment::update(&self)` method that only re-encodes if `component.is_dirty()`.
3.  In `Scene::update`, iterate through fragments, call `fragment.update()`, and then `main_scene.append(&fragment.scene, Some(transform))`.

## 4. Immediate Action Plan

1.  **Phase 1**: Fix Taffy Panic by ensuring `NodeId` consistency (Stop `taffy.clear()` every frame).
2.  **Phase 2**: Fix WGPU Crash by reordering `AppState` fields and adding explicit synchronization.
3.  **Phase 3**: Optimize Vello path with Fragment-level scene caching.
