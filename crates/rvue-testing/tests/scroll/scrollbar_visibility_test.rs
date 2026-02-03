// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for scrollbar visibility behavior.

use rvue::ComponentProps;
use rvue_style::properties::Overflow;
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test that Hidden overflow does not show scrollbar even with overflowing content.
#[test]
fn test_hidden_overflow_no_scrollbar() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let button = TestWidgetBuilder::new().with_tag("hidden-button").with_size(100.0, 400.0).build();

    container.add_child(button);

    let mut harness = TestHarness::create(container);
    let btn = harness.get_widget_by_tag("hidden-button").unwrap();

    harness.mouse_click_on(btn.clone());

    let container_widget = harness.get_widget_by_tag("container").unwrap();
    assert!(
        !harness.has_scrollbar(container_widget.clone()),
        "Hidden overflow should not show scrollbar"
    );
}

/// Test that Auto overflow shows scrollbar only when content overflows.
#[test]
fn test_auto_overflow_shows_scrollbar_when_overflowing() {
    let container = TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let content = TestWidgetBuilder::new().with_tag("content").with_size(100.0, 400.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);

    // Set up scroll state - content overflows container
    let container_widget = harness.get_widget_by_tag("container").unwrap();
    harness.set_scroll_state(container_widget.clone(), 0.0, 200.0); // scroll_height > 0

    // Debug output
    let scroll_state = container_widget.scroll_state();
    eprintln!("Container ID: {}", container_widget.id);
    eprintln!(
        "Container scroll state: width={}, height={}",
        scroll_state.scroll_width, scroll_state.scroll_height
    );

    let overflow = if let ComponentProps::Flex { styles, .. } = &*container_widget.props.borrow() {
        styles.as_ref().and_then(|s| s.overflow_y).unwrap_or(Overflow::Visible)
    } else {
        Overflow::Visible
    };
    eprintln!("Container overflow: {:?}", overflow);

    eprintln!("Has scrollbar: {}", harness.has_scrollbar(container_widget.clone()));

    let content_widget = harness.get_widget_by_tag("content").unwrap();
    harness.scroll_vertical(content_widget.clone(), 50.0);

    // Check scroll state after scroll
    let scroll_state_after = container_widget.scroll_state();
    eprintln!("After scroll - scroll_offset_y: {}", scroll_state_after.scroll_offset_y);
    eprintln!("After scroll - Has scrollbar: {}", harness.has_scrollbar(container_widget.clone()));

    assert!(
        harness.has_scrollbar(container_widget.clone()),
        "Auto overflow should show scrollbar when content overflows"
    );
}

/// Test that clicking a button inside a Hidden container doesn't affect scrollbar.
#[test]
fn test_click_button_in_hidden_container() {
    let container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(300.0, 200.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let button = TestWidgetBuilder::new().with_tag("action-button").with_size(100.0, 50.0).build();

    let tall_content =
        TestWidgetBuilder::new().with_tag("tall-content").with_size(100.0, 500.0).build();

    container.add_child(button);
    container.add_child(tall_content);

    let mut harness = TestHarness::create(container);
    let btn = harness.get_widget_by_tag("action-button").unwrap();

    for _ in 0..3 {
        harness.mouse_click_on(btn.clone());
    }

    let scroll_container = harness.get_widget_by_tag("scroll-container").unwrap();
    assert!(
        !harness.has_scrollbar(scroll_container.clone()),
        "Clicking button should not show scrollbar in Hidden container"
    );
}

/// Test scroll state updates correctly.
#[test]
fn test_scroll_state_updates() {
    let container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Scroll)
        .build();

    let content =
        TestWidgetBuilder::new().with_tag("scroll-content").with_size(100.0, 500.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container);
    let content_widget = harness.get_widget_by_tag("scroll-content").unwrap();
    let container_widget = harness.get_widget_by_tag("scroll-container").unwrap();

    // Set up scrollable content
    harness.set_scroll_state(container_widget.clone(), 0.0, 300.0); // 500px content in 200px container = 300px scrollable

    // Debug output
    let initial_offset = harness.get_scroll_offset(container_widget.clone());
    eprintln!("Initial scroll offset: ({}, {})", initial_offset.0, initial_offset.1);

    harness.scroll_vertical(content_widget.clone(), 100.0);

    let new_offset = harness.get_scroll_offset(container_widget.clone());
    eprintln!("Scroll offset after scroll_vertical: ({}, {})", new_offset.0, new_offset.1);

    assert_eq!(initial_offset, (0.0, 0.0), "Initial scroll offset should be (0, 0)");

    assert!(
        new_offset.1 > 0.0,
        "Scroll offset Y should increase after scrolling down, got {}",
        new_offset.1
    );
}
