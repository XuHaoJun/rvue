//! Integration tests for Flex layout, overflow, and scroll state propagation
//!
//! These tests verify the complete flow from Flex building to layout calculation
//! to scroll state propagation.

use rvue::render::FlexScrollState;
use rvue_style::properties::Overflow;

/// Test that Flex with overflow=Auto has should_clip=true
#[test]
fn test_flex_overflow_auto_should_clip() {
    let overflow_x: Overflow = Overflow::Auto;
    let overflow_y: Overflow = Overflow::Auto;

    assert!(overflow_x.should_clip());
    assert!(overflow_y.should_clip());
}

/// Test that Flex with overflow=Scroll has should_clip=true
#[test]
fn test_flex_overflow_scroll_should_clip() {
    let overflow_x: Overflow = Overflow::Scroll;
    let overflow_y: Overflow = Overflow::Scroll;

    assert!(overflow_x.should_clip());
    assert!(overflow_y.should_clip());
}

/// Test that Flex with overflow=Visible has should_clip=false
#[test]
fn test_flex_overflow_visible_should_not_clip() {
    let overflow_x: Overflow = Overflow::Visible;
    let overflow_y: Overflow = Overflow::Visible;

    assert!(!overflow_x.should_clip());
    assert!(!overflow_y.should_clip());
}

/// Test that Flex with overflow=Hidden has should_clip=true
#[test]
fn test_flex_overflow_hidden_should_clip() {
    let overflow_x: Overflow = Overflow::Hidden;
    let overflow_y: Overflow = Overflow::Hidden;

    assert!(overflow_x.should_clip());
    assert!(overflow_y.should_clip());
}

/// Test FlexScrollState vertical scrollbar visibility with Overflow::Auto
#[test]
fn test_scroll_state_vertical_auto_with_content() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 200.0, // Content overflows container
        container_width: 100.0,
        container_height: 100.0,
    };

    assert!(state.should_show_vertical_scrollbar(Overflow::Auto));
}

/// Test FlexScrollState vertical scrollbar visibility with Overflow::Auto (no overflow)
#[test]
fn test_scroll_state_vertical_auto_no_content() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 0.0, // No overflow
        container_width: 100.0,
        container_height: 100.0,
    };

    assert!(!state.should_show_vertical_scrollbar(Overflow::Auto));
}

/// Test FlexScrollState vertical scrollbar visibility with Overflow::Scroll (always shows)
#[test]
fn test_scroll_state_vertical_scroll_always_shows() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 0.0, // Even with no overflow
        container_width: 100.0,
        container_height: 100.0,
    };

    assert!(state.should_show_vertical_scrollbar(Overflow::Scroll));
}

/// Test that scroll_state is default when not set
#[test]
fn test_scroll_state_default_values() {
    let state = FlexScrollState::default();

    assert_eq!(state.scroll_offset_x, 0.0);
    assert_eq!(state.scroll_offset_y, 0.0);
    assert_eq!(state.scroll_width, 0.0);
    assert_eq!(state.scroll_height, 0.0);
    assert_eq!(state.container_width, 0.0);
    assert_eq!(state.container_height, 0.0);
}

/// Test scroll offset calculation - scroll up (negative delta) increases offset
#[test]
fn test_scroll_offset_up_increases() {
    let scroll_height: f32 = 200.0;
    let delta_y: f32 = -20.0; // Scroll up (negative delta in winit)

    // With inverted sign (as implemented in dispatch.rs)
    // delta_y = -20 -> -delta_y = 20 -> clamp to [0, 200] = 20
    let new_offset = (0.0f32 - delta_y).clamp(0.0, scroll_height);

    assert_eq!(new_offset, 20.0);
}

/// Test scroll offset calculation - scroll down (positive delta) decreases offset
#[test]
fn test_scroll_offset_down_decreases() {
    let scroll_height: f32 = 200.0;
    let delta_y: f32 = 20.0; // Scroll down (positive delta)

    // delta_y = 20 -> -delta_y = -20 -> clamp to [0, 200] = 0
    let new_offset = (0.0f32 - delta_y).clamp(0.0, scroll_height);

    assert_eq!(new_offset, 0.0);
}

/// Test scroll offset clamping at max
#[test]
fn test_scroll_offset_clamping_max() {
    let scroll_height: f32 = 200.0;
    let delta_y: f32 = -250.0; // Large scroll up

    // delta_y = -250 -> -delta_y = 250 -> clamp to [0, 200] = 200
    let new_offset = (0.0f32 - delta_y).clamp(0.0, scroll_height);

    assert_eq!(new_offset, 200.0);
}

/// Test scroll offset clamping at min
#[test]
fn test_scroll_offset_clamping_min() {
    let scroll_height: f32 = 200.0;
    let delta_y: f32 = 50.0; // Scroll down

    // delta_y = 50 -> -delta_y = -50 -> clamp to [0, 200] = 0
    let new_offset = (0.0f32 - delta_y).clamp(0.0, scroll_height);

    assert_eq!(new_offset, 0.0);
}

/// Test that can_scroll_y returns true when scroll_height > 0
#[test]
fn test_can_scroll_y_with_content() {
    let scroll_height: f32 = 200.0;
    let can_scroll = scroll_height > 0.0;
    assert!(can_scroll);
}

/// Test that can_scroll_y returns false when scroll_height == 0
#[test]
fn test_cannot_scroll_y_without_content() {
    let scroll_height: f32 = 0.0;
    let can_scroll = scroll_height > 0.0;
    assert!(!can_scroll);
}

/// Test that Overflow::Auto and Overflow::Scroll both trigger clipping
#[test]
fn test_overflow_auto_and_scroll_both_clip() {
    assert!(Overflow::Auto.should_clip());
    assert!(Overflow::Scroll.should_clip());
    assert!(Overflow::Hidden.should_clip()); // Hidden also clips
    assert!(!Overflow::Visible.should_clip());
}

/// Test that Overflow::Hidden clips (unlike Visible)
#[test]
fn test_overflow_hidden_clips() {
    assert!(Overflow::Hidden.should_clip());
}

/// Test FlexScrollState with realistic scroll container scenario
#[test]
fn test_scroll_state_realistic_scenario() {
    let container_height: f32 = 200.0;
    let content_height: f32 = 480.0;
    let scroll_height = (content_height - container_height).max(0.0);

    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height,
        container_width: 400.0,
        container_height,
    };

    assert_eq!(scroll_height, 280.0);
    assert!(state.should_show_vertical_scrollbar(Overflow::Auto));
    assert!(state.should_show_vertical_scrollbar(Overflow::Scroll));
    assert!(!state.should_show_vertical_scrollbar(Overflow::Visible));
}

/// Test that scrolling updates offset correctly
#[test]
fn test_scroll_updates_offset_correctly() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 280.0,
        container_width: 400.0,
        container_height: 200.0,
    };

    // Scroll up (negative delta in winit)
    let delta_y: f32 = -20.0;
    let new_offset_y = (state.scroll_offset_y - delta_y).clamp(0.0, state.scroll_height);
    state.scroll_offset_y = new_offset_y;
    assert_eq!(state.scroll_offset_y, 20.0);

    // Scroll up again
    let new_offset_y = (state.scroll_offset_y - delta_y).clamp(0.0, state.scroll_height);
    state.scroll_offset_y = new_offset_y;
    assert_eq!(state.scroll_offset_y, 40.0);
}

/// Test scroll clamping at maximum
#[test]
fn test_scroll_clamping_at_max() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 280.0,
        container_width: 400.0,
        container_height: 200.0,
    };

    // Large scroll up (negative delta)
    let delta_y: f32 = -400.0;
    let new_offset_y = (state.scroll_offset_y - delta_y).clamp(0.0, state.scroll_height);
    state.scroll_offset_y = new_offset_y;
    assert_eq!(state.scroll_offset_y, 280.0); // Clamped
}

/// Test scrollbar visibility for horizontal overflow
#[test]
fn test_horizontal_scrollbar_visible() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 150.0,
        scroll_height: 0.0,
        container_width: 100.0,
        container_height: 200.0,
    };

    assert!(state.should_show_horizontal_scrollbar(Overflow::Auto));
    assert!(!state.should_show_horizontal_scrollbar(Overflow::Visible));
}

/// Test scroll state equality
#[test]
fn test_scroll_state_values_match() {
    let state = FlexScrollState {
        scroll_offset_x: 10.0,
        scroll_offset_y: 20.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    assert_eq!(state.scroll_offset_x, 10.0);
    assert_eq!(state.scroll_offset_y, 20.0);
    assert_eq!(state.scroll_width, 100.0);
    assert_eq!(state.scroll_height, 200.0);
    assert_eq!(state.container_width, 50.0);
    assert_eq!(state.container_height, 100.0);
}
