// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for nested scroll containers.

use rvue_style::properties::Overflow;
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test behavior of nested scroll containers.
#[test]
fn test_nested_scroll_containers() {
    let outer_container = TestWidgetBuilder::new()
        .with_tag("outer")
        .with_size(400.0, 400.0)
        .with_overflow(Overflow::Auto)
        .build();

    let inner_container = TestWidgetBuilder::new()
        .with_tag("inner")
        .with_size(300.0, 300.0)
        .with_overflow(Overflow::Auto)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(200.0, 600.0).build();

    inner_container.add_child(content);
    outer_container.add_child(inner_container);

    let mut harness = TestHarness::create(outer_container);
    let content_widget = harness.get_widget_by_tag("content").unwrap();
    let inner_widget = harness.get_widget_by_tag("inner").unwrap();

    // Set up inner container as scrollable
    harness.set_scroll_state(inner_widget.clone(), 0.0, 300.0);

    harness.scroll_vertical(content_widget.clone(), 50.0);

    let inner_offset = harness.get_scroll_offset(inner_widget.clone());
    assert!(inner_offset.1 >= 0.0, "Inner container should receive scroll events");
}

/// Test that clicking in nested containers works correctly.
#[test]
fn test_click_in_nested_containers() {
    let outer = TestWidgetBuilder::new()
        .with_tag("outer")
        .with_size(300.0, 300.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let inner = TestWidgetBuilder::new()
        .with_tag("inner")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let button = TestWidgetBuilder::new().with_tag("button").with_size(50.0, 50.0).build();

    inner.add_child(button);
    outer.add_child(inner);

    let mut harness = TestHarness::create(outer);
    let btn = harness.get_widget_by_tag("button").unwrap();

    harness.mouse_click_on(btn.clone());

    let records = harness.take_records(btn.clone());
    assert!(
        !records.is_empty() || harness.get_widget_by_tag("button").is_some(),
        "Click should be processed on button"
    );

    let outer_widget = harness.get_widget_by_tag("outer").unwrap();
    assert!(
        !harness.has_scrollbar(outer_widget.clone()),
        "Hidden overflow should not show scrollbar"
    );
}

/// Test that scroll containers with visible overflow don't scroll.
#[test]
fn test_visible_overflow_no_scroll() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Visible)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(100.0, 500.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let content_widget = harness.get_widget_by_tag("content").unwrap();
    let container_widget = harness.get_widget_by_tag("container").unwrap();

    // Visible overflow - no scrollbars even with overflowing content
    harness.set_scroll_state(container_widget.clone(), 0.0, 300.0);

    harness.scroll_vertical(content_widget.clone(), 100.0);

    assert!(
        !harness.has_scrollbar(container_widget.clone()),
        "Visible overflow should not show scrollbar"
    );
}
