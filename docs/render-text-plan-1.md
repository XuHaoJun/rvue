# Rvue Text Rendering Technical Specification

This document outlines the technical plan for implementing high-quality text rendering in Rvue using `Vello` and `Parley`, integrated with the `Taffy` layout engine.

## 1. Overview

Rvue aims to provide a high-performance, GPU-accelerated UI framework. Text rendering is a critical component that requires both sophisticated layout (line breaking, font shaping) and efficient rendering (glyph caching, GPU drawing). Following the "Fine-Grained Reactivity" model, text updates should be localized and efficient.

## 2. Technology Stack

*   **Vello**: GPU Compute-based 2D renderer.
*   **Parley**: Rich text layout engine from the Linebender ecosystem.
*   **Taffy**: Flexbox/Grid layout engine.
*   **rudo-gc**: GC-managed component tree.

## 3. Architecture Changes

### 3.1 Dependencies

Add the following crates to `crates/rvue/Cargo.toml`:

```toml
[dependencies]
parley = "0.7.0"
peniko = "0.5.0"
```

*Note: Versions are aligned with `learn-projects/xilem` baseline.*

### 3.2 Text Context Management

A global `TextContext` will be initialized in `App` and shared via `Gc`. This context handles font loading and layout caching.

```rust
pub struct TextContext {
    pub font_ctx: parley::FontContext,
    pub layout_ctx: parley::LayoutContext<BrushIndex>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct BrushIndex(pub usize);
```

### 3.3 Component State Updates

Update `ComponentProps` in `crates/rvue/src/component.rs` to support basic styling:

```rust
pub enum ComponentProps {
    Text { 
        content: String,
        font_size: f32,
        color: vello::peniko::Color,
    },
    // ...
}
```

Text components will store their `parley::Layout` in `user_data` to avoid re-calculating layout during the paint phase.

### 3.4 Layout Integration (Taffy + Parley)

The `LayoutNode` should support `Taffy`'s measure functions. For `ComponentType::Text`, we will register a measure closure.

#### Measure Logic:
1.  Input: `KnownDimensions` (available width/height from Taffy).
2.  Execution:
    *   Construct a `parley::Layout` using `TextContext`.
    *   Perform line breaking based on `max_advance` (from Taffy's constraints).
    *   Store the result `Layout` in the component's `user_data`.
3.  Output: `taffy::Size` representing the text's bounding box.

### 3.5 Rendering Implementation

Update `VelloFragment::render_text` in `crates/rvue/src/render/widget.rs`:

```rust
fn render_text(
    component: &Component,
    scene: &mut vello::Scene,
    transform: vello::kurbo::Affine,
) {
    if let Some(layout) = component.user_data.borrow().downcast_ref::<parley::Layout<BrushIndex>>() {
        // Implement glyph run iteration and scene.draw_glyphs(...)
        // Refer to masonry_core/src/core/text.rs for detailed painting logic
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    let run = glyph_run.run();
                    scene.draw_glyphs(run.font())
                        .font_size(run.font_size())
                        .transform(transform)
                        .brush(&Color::BLACK) // Use color from props
                        .draw(Fill::NonZero, glyph_run.glyphs().map(|g| ...));
                }
            }
        }
    }
}
```

## 4. Fine-Grained Update Flow

1.  **Signal Update**: A signal governing `content` or `font_size` changes.
2.  **Effect Trigger**: The associated effect calls `component.mark_dirty()`.
3.  **Layout Pass**: `Taffy` detects the dirty node, re-runs the measure function using `Parley`.
4.  **Scene Update**: `VelloFragment` detects the dirty component, clears its `cached_scene`, and re-renders the text using the new `parley::Layout`.

## 5. Next Steps

1.  Initialize `FontContext` in `App::new()`.
2.  Implement `Taffy` measure function support in `LayoutNode`.
3.  Port Masonry's `render_text` glyph-loop to `VelloFragment`.
4.  Verify with a "Counter" demo showing dynamic text updates.
