# Vello + WGPU Integration Plan

This document outlines the plan to integrate Vello and WGPU into `rvue`, replacing the current placeholder implementation in `app.rs`.

## Objectives

1.  **Implement `RenderContext`**: A struct to manage `wgpu::Instance`, `Adapter`, `Device`, and `Queue`.
2.  **Implement `RenderSurface`**: A struct to manage the `wgpu::Surface`, swapchain configuration, and the "Render-to-Texture + Blit" pipeline required by Vello.
3.  **Update `AppState`**: Store the rendering context and surface.
4.  **Implement Rendering Loop**: Real rendering logic in `WindowEvent::RedrawRequested`.
5.  **Lazy Initialization**: Maintain the current lazy initialization strategy.

## 1. Architecture

We will adopt the architecture used in `masonry_winit` (from Xilem), which splits responsibilities into `RenderContext` (global GPU state) and `RenderSurface` (window-specific state).

### RenderContext
- **Responsibility**: GPU handle management.
- **Fields**:
  - `instance: wgpu::Instance`
  - `device: wgpu::Device`
  - `queue: wgpu::Queue`
  - `adapter: wgpu::Adapter` (Optional, useful for capability checks)

### RenderSurface
- **Responsibility**: Swapchain and Blitting.
- **Fields**:
  - `surface: wgpu::Surface`
  - `config: wgpu::SurfaceConfiguration`
  - `target_texture: wgpu::Texture` (Intermediate texture for Vello)
  - `target_view: wgpu::TextureView`
  - `blitter: wgpu::util::TextureBlitter` (Or custom blitter if not available)

### "Render to Texture" Pipeline
Vello uses compute shaders which often require writing to a texture that supports `STORAGE_BINDING`. Usage of the swapchain texture directly is often restricted.
**Strategy**:
1.  Create an intermediate MSAA-less texture (`target_texture`) with `STORAGE_BINDING` usage.
2.  Vello renders the scene into `target_texture`.
3.  Use a `TextureBlitter` to copy `target_texture` to the actual `surface` texture (swapchain).

## 2. Implementation Steps

### Step 1: Dependencies

Update `crates/rvue/Cargo.toml` to include:
- `vello`
- `wgpu`
- `winit` (already there)
- `pollster` (for async device requesting)

### Step 2: Create `src/render.rs`

Create a new module `crates/rvue/src/render.rs` to house the rendering logic. This keeps `app.rs` clean.

**Key Functions**:
- `RenderContext::new() -> impl Future<Output = Result<Self>>`
- `RenderSurface::new(window, context) -> Self`
- `RenderSurface::resize(width, height)`
- `render_frame(context, surface, scene)`

### Step 3: Update `AppState` in `app.rs`

```rust
pub struct AppState {
    window: Option<Window>,
    // Rendering state wrapped in Option for lazy init
    render_cx: Option<RenderContext>,
    surface: Option<RenderSurface>,
    // ... view state
}
```

### Step 4: Implement the Event Loop

In `app.rs`:

**`resumed`**:
- Create `Window`.
- (Optional) Initialize `RenderContext` here if we want to preload, or keep it lazy.

**`window_event` -> `RedrawRequested`**:
1.  **Lazy Init**: If `render_cx` is None, initialize it (using `pollster::block_on`).
2.  **Surface Logic**:
    - If `surface` is None, create it.
    - If window resized, call `surface.resize()`.
3.  **Scene Generation**:
    - Create a test `vello::Scene` (e.g., a simple shape).
4.  **Render**:
    - Call `renderer.render_to_texture(...)`.
    - Blit to frame.
    - `surface.present()`.

## 3. Detailed Logic (Reference)

We will adapt `masonry_winit/src/vello_util.rs` for `RenderSurface`.

**Blitter Note**:
If `wgpu::util::TextureBlitter` is not available in the public `wgpu` crate (it might be a `vello` export), we will implement a simple render encoding copy:
```rust
encoder.copy_texture_to_texture(
    source.as_image_copy(),
    destination.as_image_copy(),
    size
);
```
*Note: `TextureBlitter` usually handles format conversion. Simple copy requires matching formats. If Vello's target format matches Surface format (e.g. BGRA8), direct copy is fine. If not, we might need a simple render pipeline.* -> **Decision**: Start with `create_texture` using the same format as the surface if possible, or use `vello`'s tools if accessible. Xilem uses `Rgba8` for target and blits to Surface (which handles conversion).

## 4. Verification

1.  **Compile**: Ensure all dependencies align.
2.  **Run**: Launch the app. It should open a window.
3.  **Visual**: It should display the test scene (e.g., a red circle) instead of a black/white window.
4.  **Resize**: Resizing the window should not crash and should update the viewport.
