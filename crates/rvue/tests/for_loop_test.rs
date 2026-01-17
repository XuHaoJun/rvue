//! Unit tests for For component key-based diffing

use rvue::{Component, ComponentId, ComponentLifecycle, ComponentProps, ComponentType};

#[test]
fn test_for_component_creation() {
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 5 });

    assert_eq!(for_component.component_type, ComponentType::For);
    match &for_component.props {
        ComponentProps::For { item_count } => {
            assert_eq!(*item_count, 5);
        }
        _ => panic!("Expected For props"),
    }
}

#[test]
fn test_for_component_empty_list() {
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 0 });

    match &for_component.props {
        ComponentProps::For { item_count } => {
            assert_eq!(*item_count, 0);
        }
        _ => panic!("Expected For props"),
    }
}

#[test]
fn test_for_component_lifecycle() {
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 3 });

    // Test full lifecycle
    for_component.mount(None);
    for_component.update();
    for_component.unmount();
}

#[test]
fn test_for_component_mounting() {
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 2 });

    // Mount should not panic
    for_component.mount(None);
    for_component.mount(Some(rudo_gc::Gc::clone(&for_component)));
}

#[test]
fn test_for_component_unmounting() {
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 1 });

    // Unmount should not panic
    for_component.unmount();
}

#[test]
fn test_for_component_key_based_diffing() {
    // Test that For component can track items by key
    // For MVP, we'll test the basic structure
    // Full diffing algorithm will be tested in integration tests
    let for_component =
        Component::new(1, ComponentType::For, ComponentProps::For { item_count: 3 });

    // Verify component structure
    assert_eq!(for_component.component_type, ComponentType::For);
    assert_eq!(for_component.children.len(), 0); // Initially no children
}
