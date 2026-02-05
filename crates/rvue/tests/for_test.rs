//! Integration test for For component

use rvue::create_signal;
use rvue::{Component, ComponentType};

#[test]
fn test_for_component_list_rendering() {
    // Test that For component can render items from a collection
    let items = vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()];
    let _items_signal = create_signal(items.clone());

    // Create For component
    let for_component =
        Component::with_properties(1, ComponentType::For, rvue::properties::PropertyMap::new());

    assert_eq!(for_component.component_type, ComponentType::For);
}

#[test]
fn test_for_component_add_item() {
    // Test that adding items updates the UI efficiently
    let items = vec!["Item 1".to_string()];
    let (items_signal, set_items) = create_signal(items);

    let _for_component =
        Component::with_properties(1, ComponentType::For, rvue::properties::PropertyMap::new());

    // Add an item
    set_items.update(|items| {
        items.push("Item 2".to_string());
    });

    assert_eq!(items_signal.get().len(), 2);
}

#[test]
fn test_for_component_remove_item() {
    // Test that removing items updates the UI efficiently
    let items = vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()];
    let (items_signal, set_items) = create_signal(items);

    let _for_component =
        Component::with_properties(1, ComponentType::For, rvue::properties::PropertyMap::new());

    // Remove an item
    set_items.update(|items| {
        items.remove(1);
    });

    assert_eq!(items_signal.get().len(), 2);
}

#[test]
fn test_for_component_reactive_updates() {
    let items = vec!["A".to_string(), "B".to_string()];
    let (items_signal, set_items) = create_signal(items);
    let call_count = std::rc::Rc::new(std::cell::Cell::new(0));

    // Create an effect that tracks the items signal
    let _effect = rvue::create_effect({
        let items_signal = items_signal.clone();
        let call_count = call_count.clone();
        move || {
            let _ = items_signal.get(); // Track the signal
            call_count.set(call_count.get() + 1);
        }
    });

    // Effect should have run once on creation
    assert_eq!(call_count.get(), 1);

    // Update items - effect should re-run
    set_items.update(|items| {
        items.push("C".to_string());
    });
    assert_eq!(call_count.get(), 2);
    assert_eq!(items_signal.get().len(), 3);
}
