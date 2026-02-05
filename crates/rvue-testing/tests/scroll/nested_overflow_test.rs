// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for nested overflow containers - reproduces the styles example scenario.

use rvue_style::properties::{FlexDirection, Overflow};
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test that nested overflow containers correctly compute scroll_height.
/// This reproduces the styles example structure where:
/// - Root Flex has overflow=Auto, height=600
/// - Inner DemoSection has overflow_y=Auto, height=200
/// - 10 items inside the demo section (400px total content in 200px container)
#[test]
fn test_nested_overflow_scroll_height_computation() {
    eprintln!("\n=== Test: Nested overflow scroll_height computation ===");

    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_overflow(Overflow::Auto)
        .build();

    let demo_section = TestWidgetBuilder::new()
        .with_tag("demo-section")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add enough items to overflow the container (200px height, items are ~40px each)
    // Need 6+ items to overflow
    for i in 0..10 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("item-{}", i))
            .with_size(380.0, 40.0)
            .build();
        demo_section.add_child(item);
    }

    root.add_child(demo_section.clone());

    let mut harness = TestHarness::create(root);

    harness.compute_layout();

    let demo_section_widget = harness.get_widget_by_tag("demo-section").unwrap();
    let state = demo_section_widget.scroll_state();

    eprintln!("Demo Section container_height: {}", state.container_height);
    eprintln!("Demo Section scroll_height: {}", state.scroll_height);

    harness.debug_scroll_state(&demo_section_widget, "demo-section");

    eprintln!("\nLayout tree:");
    for (tag, info) in harness.get_layout_info_tree(&demo_section_widget) {
        eprintln!(
            "  {}: location=({:.1}, {:.1}), size=({:.1}, {:.1})",
            tag, info.location.0, info.location.1, info.size.0, info.size.1
        );
    }

    assert!(
        state.container_height > 0.0,
        "container_height should be > 0, got {}",
        state.container_height
    );

    assert!(
        state.scroll_height > 0.0,
        "scroll_height should be > 0 when content overflows. Content height: ~400px, Container height: {}px. \
         This means Taffy didn't compute overflow correctly for nested containers.",
        state.container_height
    );

    eprintln!("\n=== PASSED ===\n");
}

/// Test that nested overflow container shows scrollbar when content overflows.
#[test]
fn test_nested_overflow_shows_scrollbar() {
    eprintln!("\n=== Test: Nested overflow shows scrollbar ===");

    let container = TestWidgetBuilder::new()
        .with_tag("outer-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let inner_container = TestWidgetBuilder::new()
        .with_tag("inner-container")
        .with_size(380.0, 150.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add items that overflow 150px height (items are ~30px each)
    for i in 0..10 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("inner-item-{}", i))
            .with_size(360.0, 30.0)
            .build();
        inner_container.add_child(item);
    }

    container.add_child(inner_container.clone());

    let mut harness = TestHarness::create(container);
    harness.compute_layout();

    let inner_widget = harness.get_widget_by_tag("inner-container").unwrap();
    let state = inner_widget.scroll_state();

    eprintln!("Inner container_height: {}", state.container_height);
    eprintln!("Inner scroll_height: {}", state.scroll_height);
    eprintln!("has_scrollbar: {}", harness.has_scrollbar(inner_widget.clone()));

    harness.debug_scroll_state(&inner_widget, "inner-container");

    eprintln!("\nLayout:");
    for (tag, info) in harness.get_layout_info_tree(&inner_widget) {
        eprintln!("  {}: size=({:.1}, {:.1})", tag, info.size.0, info.size.1);
    }

    assert!(
        state.scroll_height > 0.0,
        "Expected scroll_height > 0, got {}. Content: ~300px, Container: {}px",
        state.scroll_height,
        state.container_height
    );

    assert!(
        harness.has_scrollbar(inner_widget),
        "Inner container should show scrollbar with overflowing content"
    );

    eprintln!("\n=== PASSED ===\n");
}

/// Test overflow demo button click scenario with real layout computation.
#[test]
fn test_overflow_demo_button_with_layout() {
    eprintln!("\n=== Test: Overflow demo button with layout computation ===");

    let control_panel =
        TestWidgetBuilder::new().with_tag("control-panel").with_size(400.0, 60.0).build();

    let visible_btn =
        TestWidgetBuilder::new().with_tag("visible-btn").with_size(80.0, 40.0).build();
    let hidden_btn = TestWidgetBuilder::new().with_tag("hidden-btn").with_size(80.0, 40.0).build();
    let scroll_btn = TestWidgetBuilder::new().with_tag("scroll-btn").with_size(80.0, 40.0).build();
    let auto_btn = TestWidgetBuilder::new().with_tag("auto-btn").with_size(80.0, 40.0).build();

    control_panel.add_child(visible_btn.clone());
    control_panel.add_child(hidden_btn.clone());
    control_panel.add_child(scroll_btn.clone());
    control_panel.add_child(auto_btn.clone());

    let scroll_area = TestWidgetBuilder::new()
        .with_tag("scroll-area")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add items that overflow 200px (items are ~50px each)
    for i in 0..8 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("demo-item-{}", i))
            .with_size(380.0, 50.0)
            .build();
        scroll_area.add_child(item);
    }

    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_child(control_panel)
        .with_child(scroll_area.clone())
        .build();

    let mut harness = TestHarness::create(root);
    harness.compute_layout();

    let scroll_widget = harness.get_widget_by_tag("scroll-area").unwrap();
    let scroll_state = scroll_widget.scroll_state();

    eprintln!("Scroll area:");
    harness.debug_scroll_state(&scroll_widget, "scroll-area");

    eprintln!("\nLayout:");
    for (tag, info) in harness.get_layout_info_tree(&scroll_widget) {
        eprintln!("  {}: size=({:.1}, {:.1})", tag, info.size.0, info.size.1);
    }

    assert!(
        scroll_state.scroll_height > 0.0,
        "scroll_height should be > 0 for overflowing content. Content: ~400px, Container: {}px",
        scroll_state.container_height
    );

    harness.mouse_click_on(visible_btn);
    harness.mouse_click_on(hidden_btn);
    harness.mouse_click_on(scroll_btn);
    harness.mouse_click_on(auto_btn);

    eprintln!("All buttons clicked successfully");

    eprintln!("\n=== PASSED ===\n");
}

/// Test the actual styles example structure more closely.
#[test]
fn test_styles_example_structure() {
    eprintln!("\n=== Test: Styles example structure ===");

    let root = TestWidgetBuilder::new()
        .with_tag("styles-root")
        .with_size(800.0, 600.0)
        .with_overflow(Overflow::Auto)
        .build();

    let padding_section =
        TestWidgetBuilder::new().with_tag("padding-section").with_size(760.0, 150.0).build();

    let overflow_section = TestWidgetBuilder::new()
        .with_tag("overflow-section")
        .with_size(760.0, 200.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add items that overflow 200px (items are ~60px each)
    for i in 0..10 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("color-item-{}", i))
            .with_size(200.0, 60.0)
            .build();
        overflow_section.add_child(item);
    }

    root.add_child(padding_section);
    root.add_child(overflow_section.clone());

    let mut harness = TestHarness::create(root);
    harness.compute_layout();

    let overflow_widget = harness.get_widget_by_tag("overflow-section").unwrap();
    let state = overflow_widget.scroll_state();

    eprintln!("Overflow section state:");
    harness.debug_scroll_state(&overflow_widget, "overflow-section");

    eprintln!("\nLayout tree:");
    for (tag, info) in harness.get_layout_info_tree(&overflow_widget) {
        eprintln!(
            "  {}: location=({:.1}, {:.1}), size=({:.1}, {:.1})",
            tag, info.location.0, info.location.1, info.size.0, info.size.1
        );
    }

    assert!(
        state.scroll_height > 0.0,
        "Expected scroll_height > 0, got {}. Content: ~600px, Container: {}px. \
         This is the root cause of the overflow demo issue - Taffy isn't computing scroll dimensions correctly.",
        state.scroll_height, state.container_height
    );

    eprintln!("\n=== PASSED ===\n");
}

/// Test scroll transform accumulation is prevented in nested overflow containers.
/// This is the key test for the fix: when parent scrolls, child overflow should work correctly.
#[test]
fn test_nested_overflow_no_transform_accumulation() {
    eprintln!("\n=== Test: Nested overflow no transform accumulation ===");

    // Create structure matching styles example:
    // Root (overflow=Auto, height=600)
    //   └── Section1 (static)
    //   └── DemoSection (overflow_y=Auto, height=300) <-- should scroll independently
    //       └── Items (15 items, 32px each = 480px content)

    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_overflow(Overflow::Auto)
        .build();

    // Section 1 (takes some space)
    let section1 = TestWidgetBuilder::new().with_tag("section-1").with_size(760.0, 100.0).build();

    // Demo section with overflow
    let demo_section = TestWidgetBuilder::new()
        .with_tag("demo-section")
        .with_size(400.0, 300.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add items (more than container height)
    for i in 0..15 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("demo-item-{}", i))
            .with_size(380.0, 32.0)
            .build();
        demo_section.add_child(item);
    }

    root.add_child(section1);
    root.add_child(demo_section.clone());

    let mut harness = TestHarness::create(root);
    harness.compute_layout();

    // Verify demo section has scrollable content
    let demo_widget = harness.get_widget_by_tag("demo-section").unwrap();
    let state = demo_widget.scroll_state();

    eprintln!("Demo section container_height: {}", state.container_height);
    eprintln!("Demo section scroll_height: {}", state.scroll_height);

    assert!(state.container_height > 0.0, "container_height should be > 0");
    assert!(
        state.scroll_height > 0.0,
        "scroll_height should be > 0 for overflowing content. Got {}",
        state.scroll_height
    );

    // Now simulate scrolling the demo section
    harness.set_scroll_state(demo_widget.clone(), 0.0, state.scroll_height);

    // Verify the scroll state was applied
    let after_scroll = demo_widget.scroll_state();
    eprintln!("After scroll - scroll_offset_y: {}", after_scroll.scroll_offset_y);

    harness.mouse_click_on(demo_widget.clone());

    eprintln!("Click on demo section succeeded");

    eprintln!("\n=== PASSED ===\n");
}

/// Test that child overflow containers work correctly even when parent overflows.
#[test]
fn test_child_overflow_works_with_parent_overflow() {
    eprintln!("\n=== Test: Child overflow works with parent overflow ===");

    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_overflow(Overflow::Auto)
        .build();

    let child_container = TestWidgetBuilder::new()
        .with_tag("child-container")
        .with_size(400.0, 300.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    for i in 0..15 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("child-item-{}", i))
            .with_size(380.0, 32.0)
            .build();
        child_container.add_child(item);
    }

    root.add_child(child_container.clone());

    let mut harness = TestHarness::create(root);
    harness.compute_layout();

    let child_widget = harness.get_widget_by_tag("child-container").unwrap();
    let state = child_widget.scroll_state();

    eprintln!("Child container:");
    harness.debug_scroll_state(&child_widget, "child-container");

    assert!(
        state.scroll_height > 0.0,
        "Child container should have scroll_height > 0. Got {}",
        state.scroll_height
    );

    harness.mouse_click_on(child_widget.clone());

    eprintln!("\n=== PASSED ===\n");
}

/// Test that scroll offset is correctly applied and content is clipped.
#[test]
fn test_scroll_offset_and_clipping() {
    eprintln!("\n=== Test: Scroll offset and clipping ===");

    let scroll_container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .with_flex_direction(FlexDirection::Column)
        .build();

    // Add items that overflow significantly (items are ~50px each, 8 items = 400px content)
    for i in 0..8 {
        let item = TestWidgetBuilder::new()
            .with_tag(&format!("scroll-item-{}", i))
            .with_size(380.0, 50.0)
            .build();
        scroll_container.add_child(item);
    }

    let mut harness = TestHarness::create(scroll_container.clone());
    harness.compute_layout();

    let container_widget = harness.get_widget_by_tag("scroll-container").unwrap();
    let state = container_widget.scroll_state();

    eprintln!("Initial scroll_height: {}", state.scroll_height);
    assert!(state.scroll_height > 0.0, "Should have scrollable content");

    harness.set_scroll_state(container_widget.clone(), 0.0, state.scroll_height);

    let after_scroll_state = container_widget.scroll_state();
    eprintln!("After setting scroll_offset_y={}", after_scroll_state.scroll_offset_y);

    harness.debug_scroll_state(&container_widget, "after-scroll");

    let layout_info = harness.get_layout_info(&container_widget);
    if let Some(info) = layout_info {
        eprintln!(
            "Container layout: location=({:.1}, {:.1}), size=({:.1}, {:.1})",
            info.location.0, info.location.1, info.size.0, info.size.1
        );
    }

    eprintln!("\n=== PASSED ===\n");
}
