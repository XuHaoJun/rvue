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
