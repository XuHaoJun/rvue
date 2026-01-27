//! Selector and ElementState tests.

use rvue_style::selectors::ElementState;

#[test]
fn test_element_state_empty() {
    let state = ElementState::empty();
    assert!(state.is_empty());
}

#[test]
fn test_element_state_insert() {
    let mut state = ElementState::empty();
    state.insert(ElementState::HOVER);
    assert!(!state.is_empty());
    assert!(state.contains(ElementState::HOVER));
}

#[test]
fn test_element_state_remove() {
    let mut state = ElementState::empty();
    state.insert(ElementState::HOVER);
    state.insert(ElementState::FOCUS);

    assert!(state.contains(ElementState::HOVER));
    assert!(state.contains(ElementState::FOCUS));

    state.remove(ElementState::HOVER);
    assert!(!state.contains(ElementState::HOVER));
    assert!(state.contains(ElementState::FOCUS));
}

#[test]
fn test_element_state_contains() {
    let mut state = ElementState::empty();

    state.insert(ElementState::HOVER);
    state.insert(ElementState::FOCUS);
    state.insert(ElementState::ACTIVE);

    assert!(state.contains(ElementState::HOVER));
    assert!(state.contains(ElementState::FOCUS));
    assert!(state.contains(ElementState::ACTIVE));
    assert!(!state.contains(ElementState::CHECKED));
}

#[test]
fn test_element_state_toggle() {
    let mut state = ElementState::empty();

    state.insert(ElementState::HOVER);
    assert!(state.contains(ElementState::HOVER));

    state.toggle(ElementState::HOVER);
    assert!(!state.contains(ElementState::HOVER));

    state.toggle(ElementState::HOVER);
    assert!(state.contains(ElementState::HOVER));
}

#[test]
fn test_element_state_all_flags() {
    let mut state = ElementState::empty();

    state.insert(ElementState::HOVER);
    state.insert(ElementState::FOCUS);
    state.insert(ElementState::ACTIVE);
    state.insert(ElementState::DISABLED);
    state.insert(ElementState::CHECKED);

    assert!(state.contains(ElementState::HOVER));
    assert!(state.contains(ElementState::FOCUS));
    assert!(state.contains(ElementState::ACTIVE));
    assert!(state.contains(ElementState::DISABLED));
    assert!(state.contains(ElementState::CHECKED));
}

#[test]
fn test_pseudo_class_matching() {
    let mut state = ElementState::empty();

    assert!(!state.matches_pseudo_class("hover"));
    assert!(!state.matches_pseudo_class("focus"));
    assert!(!state.matches_pseudo_class("active"));

    state.insert(ElementState::HOVER);
    assert!(state.matches_pseudo_class("hover"));
    assert!(!state.matches_pseudo_class("focus"));

    state.insert(ElementState::FOCUS);
    assert!(state.matches_pseudo_class("hover"));
    assert!(state.matches_pseudo_class("focus"));

    state.insert(ElementState::ACTIVE);
    assert!(state.matches_pseudo_class("active"));
}

#[test]
fn test_extended_state_matching() {
    let mut state = ElementState::empty();

    state.insert(ElementState::DRAGGING);
    assert!(state.matches_pseudo_class("dragging"));

    state.insert(ElementState::DRAG_OVER);
    assert!(state.matches_pseudo_class("drag-over"));
    assert!(state.matches_pseudo_class("drag_over"));

    state.insert(ElementState::SELECTED);
    assert!(state.matches_pseudo_class("selected"));

    state.insert(ElementState::EXPANDED);
    assert!(state.matches_pseudo_class("expanded"));

    state.insert(ElementState::COLLAPSED);
    assert!(state.matches_pseudo_class("collapsed"));

    state.insert(ElementState::FOCUS_WITHIN);
    assert!(state.matches_pseudo_class("focus-within"));

    state.insert(ElementState::FOCUS_VISIBLE);
    assert!(state.matches_pseudo_class("focus-visible"));
}

#[test]
fn test_unknown_pseudo_class() {
    let mut state = ElementState::empty();
    state.insert(ElementState::HOVER);

    assert!(!state.matches_pseudo_class("unknown"));
    assert!(!state.matches_pseudo_class(""));
}
