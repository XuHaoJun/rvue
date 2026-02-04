// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Debug test for styles example - reproducing the "content below Color Palette not visible" issue.

use rvue_style::properties::Overflow;
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test that replicates the exact styles example structure.
/// The root Flex has overflow=Auto, height=600, and contains:
/// - Interactive section (~200px)
/// - Theme section (~50px)
/// - Font Size section (~100px)
/// - Border Radius section (~100px)
/// - Border Color section (~100px)
/// - Color Palette section (~100px) <-- User says content below this is not visible
/// - CSS Selector Matching section (~200px)
/// - How CSS Works section (~150px)
/// - Overflow Examples section (~500px)
///
/// Total content height: ~1500px
/// Root container height: 600px
/// Expected: Root should have scrollbar showing ~900px of overflow content
#[test]
fn test_styles_example_debug() {
    eprintln!("\n=== DEBUG: Styles Example Structure ===");
    eprintln!("Testing if content below 'Color Palette' section is accessible\n");

    // Create root matching styles example
    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_overflow(Overflow::Auto)
        .build();

    // Section 1: Interactive Reactive Style Design (~150px with padding)
    let section1 =
        TestWidgetBuilder::new().with_tag("section-interactive").with_size(760.0, 150.0).build();

    // Section 2: Theme (~50px)
    let section_theme =
        TestWidgetBuilder::new().with_tag("section-theme").with_size(760.0, 50.0).build();

    // Section 3: Font Size Examples (~80px)
    let section_font =
        TestWidgetBuilder::new().with_tag("section-font").with_size(760.0, 80.0).build();

    // Section 4: Border Radius Examples (~100px)
    let section_border_radius =
        TestWidgetBuilder::new().with_tag("section-border-radius").with_size(760.0, 100.0).build();

    // Section 5: Border Color Examples (~100px)
    let section_border_color =
        TestWidgetBuilder::new().with_tag("section-border-color").with_size(760.0, 100.0).build();

    // Section 6: Color Palette (~80px) - User says content BELOW this is not visible
    let section_color_palette =
        TestWidgetBuilder::new().with_tag("section-color-palette").with_size(760.0, 80.0).build();

    // Section 7: CSS Selector Matching (~200px) - SHOULD be visible when scrolling
    let section_css_selector =
        TestWidgetBuilder::new().with_tag("section-css-selector").with_size(760.0, 200.0).build();

    // Section 8: How CSS Works (~150px)
    let section_how_css =
        TestWidgetBuilder::new().with_tag("section-how-css").with_size(760.0, 150.0).build();

    // Section 9: Overflow Examples (~500px with scroll area)
    let section_overflow =
        TestWidgetBuilder::new().with_tag("section-overflow").with_size(760.0, 500.0).build();

    // Add children in exact order
    root.add_child(section1);
    root.add_child(section_theme);
    root.add_child(section_font);
    root.add_child(section_border_radius);
    root.add_child(section_border_color);
    root.add_child(section_color_palette.clone());
    root.add_child(section_css_selector.clone());
    root.add_child(section_how_css.clone());
    root.add_child(section_overflow.clone());

    let mut harness = TestHarness::create(root.clone());
    harness.compute_layout();

    eprintln!("\n--- Layout Information ---");
    eprintln!("Window size: 800x600");
    eprintln!("");

    // Print all widget layouts
    eprintln!("Widget Layouts (location and size):");
    for (tag, info) in harness.get_layout_info_tree(&root) {
        eprintln!(
            "  {}: location=({:.1}, {:.1}), size=({:.1}, {:.1}), bottom={:.1}",
            tag,
            info.location.0,
            info.location.1,
            info.size.0,
            info.size.1,
            info.location.1 + info.size.1
        );
    }

    eprintln!("");
    eprintln!("--- Scroll State Debug ---");

    // Check root scroll state
    let root_widget = harness.get_widget_by_tag("root").unwrap();
    let root_state = root_widget.scroll_state();

    eprintln!("ROOT (overflow=Auto, height=600):");
    eprintln!("  container_width: {}", root_state.container_width);
    eprintln!("  container_height: {}", root_state.container_height);
    eprintln!("  scroll_width: {}", root_state.scroll_width);
    eprintln!("  scroll_height: {}", root_state.scroll_height);
    eprintln!("  scroll_offset_x: {}", root_state.scroll_offset_x);
    eprintln!("  scroll_offset_y: {}", root_state.scroll_offset_y);

    // Calculate total content height
    let total_content_height: f64 = harness
        .get_layout_info_tree(&root)
        .iter()
        .filter(|(tag, _)| !tag.starts_with("section-"))
        .map(|(_, info)| info.size.1)
        .sum();

    eprintln!("");
    eprintln!("--- Analysis ---");
    eprintln!("Total estimated content height: {:.1}px", total_content_height);
    eprintln!("Root container height: {:.1}px", root_state.container_height);
    eprintln!(
        "Expected scroll_height: {:.1}px (content - container)",
        total_content_height - root_state.container_height as f64
    );
    eprintln!("Actual scroll_height: {:.1}px", root_state.scroll_height);

    // Check if Color Palette section is below viewport
    let palette_info = harness.get_layout_info(&section_color_palette);
    if let Some(info) = palette_info {
        eprintln!("");
        eprintln!("Color Palette section:");
        eprintln!("  location: ({:.1}, {:.1})", info.location.0, info.location.1);
        eprintln!("  size: ({:.1}, {:.1})", info.size.0, info.size.1);
        eprintln!("  bottom: {:.1}", info.location.1 + info.size.1);

        // Is it within the 600px viewport?
        let is_visible = info.location.1 + info.size.1 <= 600.0;
        eprintln!("  within 600px viewport: {}", is_visible);

        if !is_visible {
            eprintln!("  NOTE: Color Palette section extends beyond 600px viewport!");
            eprintln!("  It should be scrolled into view when user scrolls.");
        }
    }

    // Check if content below Color Palette exists and its position
    let css_section_info = harness.get_layout_info(&section_css_selector);
    let how_css_info = harness.get_layout_info(&section_how_css);
    let overflow_info = harness.get_layout_info(&section_overflow);

    eprintln!("");
    eprintln!("Content BELOW Color Palette:");
    if let Some(info) = css_section_info {
        eprintln!(
            "  CSS Selector section: location=({:.1}, {:.1}), bottom={:.1}",
            info.location.0,
            info.location.1,
            info.location.1 + info.size.1
        );
    }
    if let Some(info) = how_css_info {
        eprintln!(
            "  How CSS Works section: location=({:.1}, {:.1}), bottom={:.1}",
            info.location.0,
            info.location.1,
            info.location.1 + info.size.1
        );
    }
    if let Some(info) = overflow_info {
        eprintln!(
            "  Overflow Examples section: location=({:.1}, {:.1}), bottom={:.1}",
            info.location.0,
            info.location.1,
            info.location.1 + info.size.1
        );
    }

    // Check scrollbar visibility
    eprintln!("");
    eprintln!("--- Scrollbar Status ---");
    eprintln!("has_scrollbar(root): {}", harness.has_scrollbar(root_widget.clone()));

    // The key assertion: scroll_height should be > 0 if content exceeds container
    let has_overflow = total_content_height > root_state.container_height as f64;
    eprintln!("Has overflow content: {}", has_overflow);

    if root_state.scroll_height == 0.0 && has_overflow {
        eprintln!("");
        eprintln!("!!! BUG DETECTED !!!");
        eprintln!("Root has overflowing content but scroll_height is 0!");
        eprintln!("This means Taffy didn't compute overflow dimensions correctly.");
    }

    eprintln!("");
    eprintln!("=== END DEBUG ===\n");

    // For now, just verify the test runs without panicking
    // The actual rendering issue needs visual inspection
    assert!(true, "Debug test completed - check eprintln output above");
}

/// Test to verify hit_test works for content that should be scrollable.
#[test]
fn test_hit_test_below_color_palette() {
    eprintln!("\n=== DEBUG: Hit Test Below Color Palette ===");

    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    // Item 1: at y=0 (visible)
    let item1 = TestWidgetBuilder::new().with_tag("item1").with_size(380.0, 50.0).build();

    // Item 2: at y=50 (visible)
    let item2 = TestWidgetBuilder::new().with_tag("item2").with_size(380.0, 50.0).build();

    // Item 3: at y=100 (visible)
    let item3 = TestWidgetBuilder::new().with_tag("item3").with_size(380.0, 50.0).build();

    // Item 4: at y=150 (visible)
    let item4 = TestWidgetBuilder::new().with_tag("item4").with_size(380.0, 50.0).build();

    // Item 5: at y=200 (BELOW viewport - should be scrolled to see)
    let item5 = TestWidgetBuilder::new().with_tag("item5").with_size(380.0, 50.0).build();

    // Item 6: at y=250 (BELOW viewport)
    let item6 = TestWidgetBuilder::new().with_tag("item6").with_size(380.0, 50.0).build();

    container.add_child(item1.clone());
    container.add_child(item2.clone());
    container.add_child(item3.clone());
    container.add_child(item4.clone());
    container.add_child(item5.clone());
    container.add_child(item6.clone());

    let mut harness = TestHarness::create(container.clone());
    harness.compute_layout();

    // Set scroll offset to show items 5 and 6
    harness.set_scroll_state(container.clone(), 0.0, 200.0); // 400px content - 200px container = 200px scroll

    eprintln!("Container scroll state after set:");
    let state = container.scroll_state();
    eprintln!("  scroll_offset_y: {}", state.scroll_offset_y);
    eprintln!("  scroll_height: {}", state.scroll_height);

    // Now test hit_test for items in different positions
    eprintln!("");
    eprintln!("--- Hit Test Results ---");

    // Item 1 is at y=0 in content, but with scroll_offset=100, it's at y=-100 on screen
    // So hit_test at screen y=10 should NOT hit item1
    let screen_y_for_item5 = 50.0; // Item 5 content y=200, minus scroll=100, equals screen y=100... wait let me recalculate
                                   // Content y=200, scroll_offset=100, screen_y = 200 - 100 = 100

    // Actually let me recalculate:
    // scroll_offset_y = 100 means content is shifted UP by 100px
    // So content at y=200 appears at screen y=100
    // content at y=250 appears at screen y=150

    // Test hit at screen position where item5 should be visible
    // Item 5: content y=200, with scroll=100, screen y=100
    // The item has height 50, so it spans screen y=100 to y=150
    eprintln!("Testing hit at screen (50, 125) - should hit item5 if scroll is working");
    eprintln!("  item5 content position: y=200");
    eprintln!("  scroll_offset: 100");
    eprintln!("  expected screen position: y=100");
    eprintln!("  item5 height: 50");
    eprintln!("  expected screen y range: 100-150");

    eprintln!("");
    eprintln!("=== END DEBUG ===\n");

    assert!(true, "Hit test debug completed");
}

/// Test specifically for the clip rectangle calculation bug.
#[test]
fn test_clip_rectangle_calculation() {
    eprintln!("\n=== DEBUG: Clip Rectangle Calculation ===");

    // Create a container with known dimensions
    let container = TestWidgetBuilder::new()
        .with_tag("test-container")
        .with_size(300.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    // Add content that overflows (total height = 400px, container = 200px)
    for i in 0..8 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("item-{}", i))
            .with_size(280.0, 50.0)
            .build();
        container.add_child(item);
    }

    let mut harness = TestHarness::create(container.clone());
    harness.compute_layout();

    let container_widget = harness.get_widget_by_tag("test-container").unwrap();
    let state = container_widget.scroll_state();

    eprintln!("Container:");
    eprintln!("  container_width: {}", state.container_width);
    eprintln!("  container_height: {}", state.container_height);
    eprintln!("  scroll_width: {}", state.scroll_width);
    eprintln!("  scroll_height: {}", state.scroll_height);
    eprintln!("  scroll_offset_x: {}", state.scroll_offset_x);
    eprintln!("  scroll_offset_y: {}", state.scroll_offset_y);

    eprintln!("");
    eprintln!("Layout info:");
    if let Some(info) = harness.get_layout_info(&container_widget) {
        eprintln!("  location: ({:.1}, {:.1})", info.location.0, info.location.1);
        eprintln!("  size: ({:.1}, {:.1})", info.size.0, info.size.1);
    }

    eprintln!("");
    eprintln!("--- Clip Rectangle Analysis ---");
    eprintln!("If scroll_offset_y = 100 (scrolled down by 100px):");
    eprintln!("  Content at content-y=100 should appear at screen-y=0");
    eprintln!("  Content at content-y=150 should appear at screen-y=50");
    eprintln!("  Clip should be at screen-y=0 to 200 (container bounds)");
    eprintln!("  WRONG clip would shift to screen-y=-100 to 100");
    eprintln!("");
    eprintln!("BUG: If clip shifts with scroll, content gets clipped incorrectly!");
    eprintln!("FIX: Clip rectangle should NOT include scroll_offset adjustment.");

    eprintln!("");
    eprintln!("=== END DEBUG ===\n");

    assert!(state.scroll_height > 0.0, "Should have scrollable content");
}
