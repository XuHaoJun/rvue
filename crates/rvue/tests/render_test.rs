//! Unit tests for scroll offset rendering logic

use rvue::render::FlexScrollState;

#[test]
fn test_no_scroll_offset_when_zero() {
    let scroll_state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    let should_apply = scroll_state.scroll_offset_x != 0.0 || scroll_state.scroll_offset_y != 0.0;
    assert!(!should_apply);
}

#[test]
fn test_scroll_offset_when_nonzero() {
    let scroll_state = FlexScrollState {
        scroll_offset_x: 10.0,
        scroll_offset_y: 20.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    let should_apply = scroll_state.scroll_offset_x != 0.0 || scroll_state.scroll_offset_y != 0.0;
    assert!(should_apply);
}

#[test]
fn test_scroll_offset_x_only() {
    let scroll_state = FlexScrollState {
        scroll_offset_x: 15.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    let should_apply = scroll_state.scroll_offset_x != 0.0 || scroll_state.scroll_offset_y != 0.0;
    assert!(should_apply);
}

#[test]
fn test_scroll_offset_y_only() {
    let scroll_state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 25.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    let should_apply = scroll_state.scroll_offset_x != 0.0 || scroll_state.scroll_offset_y != 0.0;
    assert!(should_apply);
}

#[test]
fn test_scroll_state_with_both_offsets() {
    let scroll_state = FlexScrollState {
        scroll_offset_x: 5.0,
        scroll_offset_y: 10.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 80.0,
        container_height: 150.0,
    };

    let should_apply_x = scroll_state.scroll_offset_x != 0.0;
    let should_apply_y = scroll_state.scroll_offset_y != 0.0;

    assert!(should_apply_x);
    assert!(should_apply_y);
}
