# Xilem + Vello Integration Survey

This document outlines the findings from investigating how Xilem (specifically the `masonry_winit` crate) integrates Vello and WGPU. The goal is to inform the implementation of the rendering loop in `rvue`.

## Overview

Xilem delegates its windowing and rendering logic to `masonry_winit`. The core abstraction is `MasonryState`, which manages the application lifecycle, windows, and the Vello rendering context.

## Key Components

### 1. Render Context (`RenderContext`)
Located in `masonry_winit/src/vello_util.rs`.
- **Responsibility**: Manages the `wgpu::Instance` and a list of `DeviceHandle`s (which bundle `wgpu::Adapter`, `wgpu::Device`, and `wgpu::Queue`).
- **Initialization**: Created once at app startup.
- **Surface Creation**: Provides methods to create `RenderSurface`s for winit windows.

### 2. Render Surface (`RenderSurface`)
Located in `masonry_winit/src/vello_util.rs`.
- **Responsibility**: Wraps `wgpu::Surface` and holds the configuration (`SurfaceConfiguration`), target texture, and a `TextureBlitter`.
- **Blitting Strategy**: Xilem uses an intermediate texture for Vello's rendering.
  > "Vello uses a compute shader to render to the provided texture, which means that it can't bind the surface texture in most cases."
- **Resize Handling**: Recreates the target texture and view upon resize.

### 3. Event Loop Runner (`MasonryState`)
Located in `masonry_winit/src/event_loop_runner.rs`.
- **Lifecycle**: Handles `resumed`, `window_event`, etc.
- **Redraw Logic**:
  1.  **Preparation**: Checks if the surface needs resizing.
  2.  **Animation**: Handles animation timers.
  3.  **Scene Generation**: Calls into the UI tree (`RenderRoot`) to produce a `vello::Scene`.
  4.  **Rendering**:
      - Transforms the scene for HiDPI if necessary.
      - Acquires the surface texture.
      - Uses `vello::Renderer` to render the scene to the *intermediate target texture*.
      - Blits the target texture to the actual surface texture using `surface.blitter`.
      - Presents the surface texture.
      - Polls the device (`device.poll(wgpu::PollType::wait_indefinitely())`) to ensure completion.

## Recommendations for Rvue

1.  **Adopt the Blitting Strategy**: Direct rendering to the swapchain might not work consistently with Vello's compute shaders. `rvue` should implement the "render to texture -> blit to surface" pipeline.
2.  **RenderContext Abstraction**: Create a struct similar to `RenderContext` to manage the WGPU instance and devices. This allows for cleaner separation of concerns.
3.  **Lazy Initialization**: `rvue`'s current plan for lazy renderer initialization fits well. The `RenderContext` can be initialized when the first window is created or first redraw is requested.
4.  **Handling HiDPI**: Ensure the scene is scaled according to the window's scale factor, as Xilem does.

## Implementation Plan

1.  **Update `AppState`**: Add fields for `RenderContext` (or similar) and a map of `WindowId` to `RenderSurface`.
2.  **Implement `RenderContext`**: adapted from `masonry_winit`.
3.  **Implement Redraw Logic**: In `WindowEvent::RedrawRequested`:
    - Ensure `RenderContext` and `RenderSurface` exist.
    - Generate a simple test `Scene` (e.g., a colored rectangle).
    - Render to texture using Vello.
    - Blit to surface.
