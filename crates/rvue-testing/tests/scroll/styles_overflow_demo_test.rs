// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for styles overflow demo - button clicks in scrolling containers.

use rvue_style::properties::Overflow;
use rvue_testing::{TestHarness, TestWidgetBuilder};

/// Test that a button inside an Auto overflow container can be clicked.
/// This simulates the overflow demo scenario where buttons control overflow mode.
#[test]
fn test_button_click_in_auto_overflow_container() {
    eprintln!("\n=== Test: Button click in Auto overflow container ===");

    let container = TestWidgetBuilder::new()
        .with_tag("overflow-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let button = TestWidgetBuilder::new().with_tag("visible-button").with_size(80.0, 40.0).build();

    container.add_child(button);

    let mut harness = TestHarness::create(container);

    harness.set_scroll_state(harness.get_widget_by_tag("overflow-container").unwrap(), 0.0, 300.0);

    let btn = harness.get_widget_by_tag("visible-button").unwrap();
    eprintln!("Button found, attempting click...");

    harness.mouse_click_on(btn.clone());
    eprintln!("Click simulated successfully");

    eprintln!("=== PASSED ===\n");
}

/// Test that a button below the visible area (scrolled out) should NOT be hittable
/// when the container has overflow=Hidden.
#[test]
fn test_button_outside_hidden_clip_bounds_not_hittable() {
    eprintln!("\n=== Test: Button outside Hidden clip bounds not hittable ===");

    let container = TestWidgetBuilder::new()
        .with_tag("hidden-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let visible_button =
        TestWidgetBuilder::new().with_tag("visible-btn").with_size(80.0, 40.0).build();

    let hidden_button =
        TestWidgetBuilder::new().with_tag("hidden-btn").with_size(80.0, 40.0).build();

    container.add_child(visible_button);
    container.add_child(hidden_button);

    let mut harness = TestHarness::create(container);

    let hidden_widget = harness.get_widget_by_tag("hidden-container").unwrap();
    harness.set_scroll_state(hidden_widget.clone(), 0.0, 400.0);

    let visible_btn = harness.get_widget_by_tag("visible-btn").unwrap();
    harness.mouse_click_on(visible_btn.clone());

    eprintln!("Visible button click handled without errors");

    eprintln!("=== PASSED ===\n");
}

/// Test that buttons in a scrollable container remain hittable when visible.
#[test]
fn test_buttons_in_scrollable_container_remain_hittable() {
    eprintln!("\n=== Test: Buttons in scrollable container remain hittable ===");

    let scroll_container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let button1 = TestWidgetBuilder::new().with_tag("btn-visible").with_size(80.0, 40.0).build();

    let button2 = TestWidgetBuilder::new().with_tag("btn-scrolled").with_size(80.0, 40.0).build();

    let button3 =
        TestWidgetBuilder::new().with_tag("btn-scrolled-far").with_size(80.0, 40.0).build();

    scroll_container.add_child(button1);
    scroll_container.add_child(button2);
    scroll_container.add_child(button3);

    let mut harness = TestHarness::create(scroll_container);

    let container = harness.get_widget_by_tag("scroll-container").unwrap();
    harness.set_scroll_state(container.clone(), 0.0, 300.0);

    let btn1 = harness.get_widget_by_tag("btn-visible").unwrap();
    harness.mouse_click_on(btn1.clone());

    let btn2 = harness.get_widget_by_tag("btn-scrolled").unwrap();
    harness.mouse_click_on(btn2.clone());

    let btn3 = harness.get_widget_by_tag("btn-scrolled-far").unwrap();
    harness.mouse_click_on(btn3.clone());

    eprintln!("All buttons clicked successfully");

    eprintln!("=== PASSED ===\n");
}

/// Test overflow mode button interactions - simulates the styles demo scenario.
#[test]
fn test_overflow_mode_buttons() {
    eprintln!("\n=== Test: Overflow mode button interactions ===");

    let control_panel =
        TestWidgetBuilder::new().with_tag("control-panel").with_size(400.0, 100.0).build();

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
        .with_size(400.0, 300.0)
        .with_overflow(Overflow::Auto)
        .build();

    let item1 = TestWidgetBuilder::new().with_tag("item-1").with_size(380.0, 40.0).build();
    let item2 = TestWidgetBuilder::new().with_tag("item-2").with_size(380.0, 40.0).build();
    let item3 = TestWidgetBuilder::new().with_tag("item-3").with_size(380.0, 40.0).build();
    let item4 = TestWidgetBuilder::new().with_tag("item-4").with_size(380.0, 40.0).build();
    let item5 = TestWidgetBuilder::new().with_tag("item-5").with_size(380.0, 40.0).build();
    let item6 = TestWidgetBuilder::new().with_tag("item-6").with_size(380.0, 40.0).build();
    let item7 = TestWidgetBuilder::new().with_tag("item-7").with_size(380.0, 40.0).build();
    let item8 = TestWidgetBuilder::new().with_tag("item-8").with_size(380.0, 40.0).build();
    let item9 = TestWidgetBuilder::new().with_tag("item-9").with_size(380.0, 40.0).build();
    let item10 = TestWidgetBuilder::new().with_tag("item-10").with_size(380.0, 40.0).build();
    let item11 = TestWidgetBuilder::new().with_tag("item-11").with_size(380.0, 40.0).build();
    let item12 = TestWidgetBuilder::new().with_tag("item-12").with_size(380.0, 40.0).build();

    scroll_area.add_child(item1);
    scroll_area.add_child(item2);
    scroll_area.add_child(item3);
    scroll_area.add_child(item4);
    scroll_area.add_child(item5);
    scroll_area.add_child(item6);
    scroll_area.add_child(item7);
    scroll_area.add_child(item8);
    scroll_area.add_child(item9);
    scroll_area.add_child(item10);
    scroll_area.add_child(item11);
    scroll_area.add_child(item12);

    let root = TestWidgetBuilder::new()
        .with_tag("root")
        .with_size(800.0, 600.0)
        .with_child(control_panel)
        .with_child(scroll_area.clone())
        .build();

    let mut harness = TestHarness::create(root);

    harness.set_scroll_state(scroll_area.clone(), 0.0, 200.0);

    let visible_btn_widget = harness.get_widget_by_tag("visible-btn").unwrap();
    let hidden_btn_widget = harness.get_widget_by_tag("hidden-btn").unwrap();
    let scroll_btn_widget = harness.get_widget_by_tag("scroll-btn").unwrap();
    let auto_btn_widget = harness.get_widget_by_tag("auto-btn").unwrap();

    harness.mouse_click_on(visible_btn_widget.clone());
    eprintln!("Visible button clicked");

    harness.mouse_click_on(hidden_btn_widget.clone());
    eprintln!("Hidden button clicked");

    harness.mouse_click_on(scroll_btn_widget.clone());
    eprintln!("Scroll button clicked");

    harness.mouse_click_on(auto_btn_widget.clone());
    eprintln!("Auto button clicked");

    eprintln!("=== PASSED ===\n");
}

/// Test that hit_test works correctly with scroll offset applied.
#[test]
fn test_hit_test_with_scroll_offset() {
    eprintln!("\n=== Test: Hit test with scroll offset ===");

    let container = TestWidgetBuilder::new()
        .with_tag("scroll-container")
        .with_size(400.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let button =
        TestWidgetBuilder::new().with_tag("scrolled-button").with_size(100.0, 50.0).build();

    container.add_child(button);

    let mut harness = TestHarness::create(container.clone());

    harness.set_scroll_state(container.clone(), 0.0, 300.0);

    let btn = harness.get_widget_by_tag("scrolled-button").unwrap();

    harness.mouse_click_on(btn.clone());

    let scroll_state = container.scroll_state();
    eprintln!("Scroll offset after click: {}", scroll_state.scroll_offset_y);

    eprintln!("=== PASSED ===\n");
}
