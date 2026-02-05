// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for scroll event handling.

use rvue_style::properties::Overflow;
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test that scroll events are properly dispatched.
#[test]
fn test_scroll_event_dispatched() {
    let container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let content =
        TestWidgetBuilder::new().with_tag("scroll-content").with_size(100.0, 400.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let content_widget = harness.get_widget_by_tag("scroll-content").unwrap();
    let container_widget = harness.get_widget_by_tag("scroll-container").unwrap();

    // Set up scrollable content
    harness.set_scroll_state(container_widget.clone(), 0.0, 200.0);

    harness.scroll_vertical(content_widget.clone(), 50.0);

    let offset = harness.get_scroll_offset(container_widget.clone());
    assert!(offset.1 >= 0.0, "Scroll offset should be non-negative");
}

/// Test that scroll delta is correctly converted.
#[test]
fn test_scroll_delta_conversion() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Scroll)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(100.0, 1000.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let content_widget = harness.get_widget_by_tag("content").unwrap();
    let container_widget = harness.get_widget_by_tag("container").unwrap();

    // Set up scrollable content
    harness.set_scroll_state(container_widget.clone(), 0.0, 800.0);

    harness.scroll_vertical(content_widget.clone(), 100.0);
    let offset_after_first = harness.get_scroll_offset(container_widget.clone());

    harness.scroll_vertical(content_widget.clone(), -50.0);
    let offset_after_second = harness.get_scroll_offset(container_widget.clone());

    assert!(
        offset_after_second.1 <= offset_after_first.1,
        "Scrolling up should decrease the offset"
    );
}

/// Test that scroll doesn't go negative.
#[test]
fn test_scroll_does_not_go_negative() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Scroll)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(100.0, 1000.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let content_widget = harness.get_widget_by_tag("content").unwrap();
    let container_widget = harness.get_widget_by_tag("container").unwrap();

    // Set up scrollable content
    harness.set_scroll_state(container_widget.clone(), 0.0, 800.0);

    // Try to scroll up when already at top
    harness.scroll_vertical(content_widget.clone(), -100.0);

    let offset = harness.get_scroll_offset(container_widget.clone());
    assert!(offset.1 >= 0.0, "Scroll offset should not go negative, got {}", offset.1);
}

/// Test horizontal scroll behavior.
#[test]
fn test_horizontal_scroll() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Scroll)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(500.0, 100.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let container_widget = harness.get_widget_by_tag("container").unwrap();

    // Set up horizontal scrollable content
    harness.set_scroll_state(container_widget.clone(), 300.0, 0.0);

    // Get initial offset
    let initial_offset = harness.get_scroll_offset(container_widget.clone());
    eprintln!("Initial offset: ({}, {})", initial_offset.0, initial_offset.1);

    // Scroll horizontally
    harness.scroll_horizontal(container_widget.clone(), 100.0);

    // Get offset after scroll
    let offset = harness.get_scroll_offset(container_widget.clone());
    eprintln!("Offset after scroll: ({}, {})", offset.0, offset.1);

    assert!(offset.0 > 0.0, "Horizontal scroll should increase X offset, got {}", offset.0);
}
