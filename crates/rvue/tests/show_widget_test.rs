//! Unit tests for Show widget component

use rvue::{Component, ComponentLifecycle, ComponentType};

#[test]
fn test_show_widget_creation() {
    let show = Component::with_properties(
        1,
        ComponentType::Show,
        rvue::properties::PropertyMap::with(rvue::properties::ShowCondition(true)),
    );

    assert_eq!(show.component_type, ComponentType::Show);
}

#[test]
fn test_show_widget_when_false() {
    let show = Component::with_properties(
        1,
        ComponentType::Show,
        rvue::properties::PropertyMap::with(rvue::properties::ShowCondition(false)),
    );

    assert_eq!(show.component_type, ComponentType::Show);
}

#[test]
fn test_show_widget_mounting() {
    let show = Component::with_properties(
        1,
        ComponentType::Show,
        rvue::properties::PropertyMap::with(rvue::properties::ShowCondition(true)),
    );

    // Mount should not panic
    show.mount(None);
    show.mount(Some(rudo_gc::Gc::clone(&show)));
}

#[test]
fn test_show_widget_unmounting() {
    let show = Component::with_properties(
        1,
        ComponentType::Show,
        rvue::properties::PropertyMap::with(rvue::properties::ShowCondition(true)),
    );

    // Unmount should not panic
    show.unmount();
}

#[test]
fn test_show_widget_lifecycle() {
    let show = Component::with_properties(
        1,
        ComponentType::Show,
        rvue::properties::PropertyMap::with(rvue::properties::ShowCondition(true)),
    );

    // Test full lifecycle
    show.mount(None);
    show.update();
    show.unmount();
}
