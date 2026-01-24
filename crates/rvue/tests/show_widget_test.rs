//! Unit tests for Show widget component

use rvue::{Component, ComponentLifecycle, ComponentProps, ComponentType};

#[test]
fn test_show_widget_creation() {
    let show = Component::new(1, ComponentType::Show, ComponentProps::Show { when: true });

    assert_eq!(show.component_type, ComponentType::Show);
    match &*show.props.borrow() {
        ComponentProps::Show { when } => {
            assert!(*when);
        }
        _ => panic!("Expected Show props"),
    };
}

#[test]
fn test_show_widget_when_false() {
    let show = Component::new(1, ComponentType::Show, ComponentProps::Show { when: false });

    match &*show.props.borrow() {
        ComponentProps::Show { when } => {
            assert!(!*when);
        }
        _ => panic!("Expected Show props"),
    };
}

#[test]
fn test_show_widget_mounting() {
    let show = Component::new(1, ComponentType::Show, ComponentProps::Show { when: true });

    // Mount should not panic
    show.mount(None);
    show.mount(Some(rudo_gc::Gc::clone(&show)));
}

#[test]
fn test_show_widget_unmounting() {
    let show = Component::new(1, ComponentType::Show, ComponentProps::Show { when: true });

    // Unmount should not panic
    show.unmount();
}

#[test]
fn test_show_widget_lifecycle() {
    let show = Component::new(1, ComponentType::Show, ComponentProps::Show { when: true });

    // Test full lifecycle
    show.mount(None);
    show.update();
    show.unmount();
}
