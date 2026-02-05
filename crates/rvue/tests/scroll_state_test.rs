//! Unit tests for FlexScrollState

use rvue::render::FlexScrollState;
use rvue_style::properties::Overflow;

#[test]
fn test_scroll_state_horizontal_bar_visibility() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 150.0,
    };

    assert!(state.should_show_horizontal_scrollbar(Overflow::Auto));
}

#[test]
fn test_scroll_state_vertical_bar_visibility() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 300.0,
        container_width: 80.0,
        container_height: 200.0,
    };

    assert!(state.should_show_vertical_scrollbar(Overflow::Auto));
}

#[test]
fn test_scroll_state_no_bar_for_visible() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    assert!(!state.should_show_vertical_scrollbar(Overflow::Visible));
    assert!(!state.should_show_horizontal_scrollbar(Overflow::Visible));
}

#[test]
fn test_scroll_state_no_scroll_width() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    assert!(!state.should_show_horizontal_scrollbar(Overflow::Auto));
}

#[test]
fn test_scroll_state_no_scroll_height() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 0.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    assert!(!state.should_show_vertical_scrollbar(Overflow::Auto));
}

#[test]
fn test_scroll_state_scroll_overflow_always_shows() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 0.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    assert!(state.should_show_horizontal_scrollbar(Overflow::Scroll));
    assert!(state.should_show_vertical_scrollbar(Overflow::Scroll));
}

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

#[test]
fn test_scroll_offset_clamping_positive_delta() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let delta_y = 50.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 50.0);
}

#[test]
fn test_scroll_offset_clamping_negative_delta() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 100.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let delta_y = -50.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 50.0);
}

#[test]
fn test_scroll_offset_clamping_exceeds_max() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 150.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let delta_y = 100.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 200.0);
}

#[test]
fn test_scroll_offset_clamping_negative_exceeds_min() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let delta_y = -50.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 0.0);
}

#[test]
fn test_can_scroll_with_positive_scroll_height() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let can_scroll_y = state.scroll_height > 0.0;
    assert!(can_scroll_y);
}

#[test]
fn test_cannot_scroll_with_zero_scroll_height() {
    let state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 0.0,
        scroll_height: 0.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    let can_scroll_y = state.scroll_height > 0.0;
    assert!(!can_scroll_y);
}
