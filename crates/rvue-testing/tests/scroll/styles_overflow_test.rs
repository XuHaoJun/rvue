// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for overflow mode changes - focused debug tests.

use rvue_style::properties::Overflow;
use rvue_testing::TestHarness;

/// Test that harness.has_scrollbar() works correctly with Hidden overflow.
#[test]
fn test_hidden_overflow_no_scrollbar() {
    eprintln!("\n=== Test: Hidden overflow no scrollbar ===");

    let container = rvue_testing::TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Hidden)
        .build();

    let content =
        rvue_testing::TestWidgetBuilder::new().with_tag("content").with_size(100.0, 500.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container.clone());
    harness.set_scroll_state(container.clone(), 0.0, 300.0);

    eprintln!("Overflow: Hidden, scroll_height: 300");
    eprintln!("has_scrollbar: {:?}", harness.has_scrollbar(container.clone()));

    assert!(!harness.has_scrollbar(container.clone()), "Hidden overflow should NOT show scrollbar");

    eprintln!("=== PASSED ===\n");
}

/// Test that Auto overflow shows scrollbar when content overflows.
#[test]
fn test_auto_overflow_with_overflowing_content() {
    eprintln!("\n=== Test: Auto overflow with overflowing content ===");

    let container = rvue_testing::TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Auto)
        .build();

    let content =
        rvue_testing::TestWidgetBuilder::new().with_tag("content").with_size(100.0, 500.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container.clone());
    harness.set_scroll_state(container.clone(), 0.0, 300.0);

    eprintln!("Overflow: Auto, scroll_height: 300");
    eprintln!("has_scrollbar: {:?}", harness.has_scrollbar(container.clone()));

    assert!(
        harness.has_scrollbar(container.clone()),
        "Auto overflow with content should show scrollbar"
    );

    eprintln!("=== PASSED ===\n");
}

/// Test Scroll mode always shows scrollbar.
#[test]
fn test_scroll_mode_always_shows_scrollbar() {
    eprintln!("\n=== Test: Scroll mode always shows scrollbar ===");

    let container = rvue_testing::TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Scroll)
        .build();

    let content =
        rvue_testing::TestWidgetBuilder::new().with_tag("content").with_size(100.0, 100.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container.clone());
    harness.set_scroll_state(container.clone(), 0.0, 0.0);

    eprintln!("Overflow: Scroll, scroll_height: 0");
    eprintln!("has_scrollbar: {:?}", harness.has_scrollbar(container.clone()));

    assert!(
        harness.has_scrollbar(container.clone()),
        "Scroll overflow should ALWAYS show scrollbar"
    );

    eprintln!("=== PASSED ===\n");
}

/// Test Visible overflow never shows scrollbar.
#[test]
fn test_visible_overflow_no_scrollbar() {
    eprintln!("\n=== Test: Visible overflow no scrollbar ===");

    let container = rvue_testing::TestWidgetBuilder::new()
        .with_tag("container")
        .with_size(200.0, 200.0)
        .with_overflow(Overflow::Visible)
        .build();

    let content =
        rvue_testing::TestWidgetBuilder::new().with_tag("content").with_size(100.0, 500.0).build();

    container.add_child(content);

    let mut harness = TestHarness::create(container.clone());
    harness.set_scroll_state(container.clone(), 0.0, 300.0);

    eprintln!("Overflow: Visible, scroll_height: 300");
    eprintln!("has_scrollbar: {:?}", harness.has_scrollbar(container.clone()));

    assert!(
        !harness.has_scrollbar(container.clone()),
        "Visible overflow should NOT show scrollbar"
    );

    eprintln!("=== PASSED ===\n");
}

/// Test all overflow modes.
#[test]
fn test_all_overflow_modes() {
    eprintln!("\n=== Test: All overflow modes ===");

    let modes = [
        (Overflow::Visible, false, "Visible"),
        (Overflow::Hidden, false, "Hidden"),
        (Overflow::Auto, true, "Auto with overflow"),
        (Overflow::Scroll, true, "Scroll"),
        (Overflow::Clip, false, "Clip"),
    ];

    for (overflow, expected_scrollbar, name) in modes {
        let container = rvue_testing::TestWidgetBuilder::new()
            .with_tag(&format!("container-{}", name))
            .with_size(200.0, 200.0)
            .with_overflow(overflow)
            .build();

        let content = rvue_testing::TestWidgetBuilder::new()
            .with_tag(&format!("content-{}", name))
            .with_size(100.0, 500.0)
            .build();

        container.add_child(content);

        let mut harness = TestHarness::create(container.clone());
        harness.set_scroll_state(container.clone(), 0.0, 300.0);

        let has_scrollbar = harness.has_scrollbar(container.clone());
        eprintln!("{}: has_scrollbar={}", name, has_scrollbar);

        assert_eq!(
            has_scrollbar, expected_scrollbar,
            "{} should have_scrollbar={}",
            name, expected_scrollbar
        );
    }

    eprintln!("=== PASSED ===\n");
}
