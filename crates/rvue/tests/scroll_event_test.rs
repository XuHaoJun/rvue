//! Unit tests for scroll event handling

use rudo_gc::Gc;
use rvue::component::{Component, ComponentProps, ComponentType};
use rvue::event::dispatch::find_scroll_container;
use rvue::event::types::{map_scroll_delta, ScrollDelta};
use rvue::render::FlexScrollState;
use rvue_style::properties::Overflow;
use winit::event::MouseScrollDelta;

fn create_test_flex_with_overflow(overflow_x: Overflow, overflow_y: Overflow) -> Gc<Component> {
    use rvue_style::ComputedStyles;

    let mut styles = ComputedStyles::default();
    styles.overflow_x = Some(overflow_x);
    styles.overflow_y = Some(overflow_y);

    let component = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
            styles: Some(styles),
        },
    );

    component
}

fn create_test_child() -> Gc<Component> {
    Component::new(
        2,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    )
}

#[test]
fn test_find_scroll_container_with_auto_overflow() {
    let parent = create_test_flex_with_overflow(Overflow::Auto, Overflow::Auto);
    let child = create_test_child();

    // Set child's parent to the scroll container
    child.set_parent(Some(Gc::clone(&parent)));

    let result = find_scroll_container(&child);

    assert!(result.is_some(), "Should find scroll container with Auto overflow");
    assert_eq!(result.unwrap().id, parent.id);
}

#[test]
fn test_find_scroll_container_with_scroll_overflow() {
    let parent = create_test_flex_with_overflow(Overflow::Scroll, Overflow::Scroll);
    let child = create_test_child();

    child.set_parent(Some(Gc::clone(&parent)));

    let result = find_scroll_container(&child);

    assert!(result.is_some(), "Should find scroll container with Scroll overflow");
}

#[test]
fn test_find_scroll_container_with_hidden_overflow() {
    let parent = create_test_flex_with_overflow(Overflow::Hidden, Overflow::Hidden);
    let child = create_test_child();

    child.set_parent(Some(Gc::clone(&parent)));

    let result = find_scroll_container(&child);

    assert!(result.is_some(), "Should find scroll container with Hidden overflow");
}

#[test]
fn test_find_scroll_container_with_visible_overflow() {
    let parent = create_test_flex_with_overflow(Overflow::Visible, Overflow::Visible);
    let child = create_test_child();

    child.set_parent(Some(Gc::clone(&parent)));

    let result = find_scroll_container(&child);

    assert!(result.is_none(), "Should NOT find scroll container with Visible overflow");
}

#[test]
fn test_scroll_state_updates_correctly() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    // Simulate scroll delta
    let delta_y = 50.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 50.0);
}

#[test]
fn test_scroll_state_clamping_negative() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 0.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    // Simulate negative scroll delta
    let delta_y = -50.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 0.0);
}

#[test]
fn test_scroll_state_clamping_exceeds_max() {
    let mut state = FlexScrollState {
        scroll_offset_x: 0.0,
        scroll_offset_y: 150.0,
        scroll_width: 100.0,
        scroll_height: 200.0,
        container_width: 50.0,
        container_height: 100.0,
    };

    // Simulate scroll delta that exceeds max
    let delta_y = 100.0;
    let max_offset = state.scroll_height.max(0.0);
    state.scroll_offset_y = (state.scroll_offset_y + delta_y).clamp(0.0, max_offset);

    assert_eq!(state.scroll_offset_y, 200.0);
}

#[test]
fn test_scroll_delta_line_conversion() {
    use rvue::event::types::map_scroll_delta;
    use winit::event::MouseScrollDelta;

    // Test line delta conversion
    let line_delta = MouseScrollDelta::LineDelta(0.0, 3.0);
    let result = map_scroll_delta(line_delta);

    match result {
        ScrollDelta::Line(lines) => {
            assert!((lines - 3.0).abs() < f64::EPSILON, "Line delta should be 3.0");
        }
        _ => panic!("Expected Line scroll delta"),
    }
}

#[test]
fn test_scroll_delta_pixel_conversion() {
    use rvue::event::types::map_scroll_delta;
    use winit::event::MouseScrollDelta;

    // Test pixel delta conversion
    let pixel_delta =
        MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition { x: 10.0, y: 20.0 });
    let result = map_scroll_delta(pixel_delta);

    match result {
        ScrollDelta::Pixel(dx, dy) => {
            assert!((dx - 10.0).abs() < f64::EPSILON, "Pixel dx should be 10.0");
            assert!((dy - 20.0).abs() < f64::EPSILON, "Pixel dy should be 20.0");
        }
        _ => panic!("Expected Pixel scroll delta"),
    }
}

#[test]
fn test_nested_scroll_containers() {
    let grandparent = create_test_flex_with_overflow(Overflow::Visible, Overflow::Visible);
    let parent = create_test_flex_with_overflow(Overflow::Auto, Overflow::Auto);
    let child = create_test_child();

    // Set up hierarchy: child -> parent -> grandparent
    child.set_parent(Some(Gc::clone(&parent)));
    parent.set_parent(Some(Gc::clone(&grandparent)));

    // Child should find parent as scroll container (not grandparent)
    let result = find_scroll_container(&child);

    assert!(result.is_some(), "Should find a scroll container");
    assert_eq!(result.unwrap().id, parent.id, "Should find nearest scroll container");
}
