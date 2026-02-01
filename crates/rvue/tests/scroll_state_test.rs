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
