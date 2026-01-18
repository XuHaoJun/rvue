//! For widget for list rendering

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::SignalRead;
use rudo_gc::{Gc, Trace};
use std::collections::HashMap;

/// For widget for rendering lists of items
pub struct For;

/// Key-based item tracking for efficient list updates
#[derive(Debug, Clone)]
pub struct ItemKey {
    pub key: String,
    pub index: usize,
}

impl For {
    /// Create a new For component with a static item count
    pub fn new(id: ComponentId, item_count: usize) -> Gc<Component> {
        Component::new(id, ComponentType::For, ComponentProps::For { item_count })
    }

    /// Create a new For component from a signal containing a collection
    /// For MVP, we'll track the length of the collection
    pub fn from_signal<T, U>(id: ComponentId, each_signal: T) -> Gc<Component>
    where
        T: SignalRead<Vec<U>> + Clone + 'static,
        U: Trace + Clone + 'static,
    {
        let items = each_signal.get();
        let initial_item_count = items.len();
        let component = Component::new(
            id,
            ComponentType::For,
            ComponentProps::For { item_count: initial_item_count },
        );

        // Setup reactive update for item count
        let comp = Gc::clone(&component);
        let sig = each_signal.clone();
        let effect = create_effect(move || {
            let items = sig.get();
            let new_item_count = items.len();
            *comp.props.borrow_mut() = ComponentProps::For { item_count: new_item_count };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }

    /// Perform key-based diffing to determine what items were added, removed, or updated
    /// Returns: (added_keys, removed_keys, updated_keys)
    pub fn diff_items<T, F>(
        old_items: &[T],
        new_items: &[T],
        key_fn: F,
    ) -> (Vec<String>, Vec<String>, Vec<String>)
    where
        F: Fn(&T) -> String,
    {
        let old_keys: HashMap<String, usize> =
            old_items.iter().enumerate().map(|(i, item)| (key_fn(item), i)).collect();

        let new_keys: HashMap<String, usize> =
            new_items.iter().enumerate().map(|(i, item)| (key_fn(item), i)).collect();

        // Find added items (in new but not in old)
        let added: Vec<String> =
            new_keys.keys().filter(|k| !old_keys.contains_key(*k)).cloned().collect();

        // Find removed items (in old but not in new)
        let removed: Vec<String> =
            old_keys.keys().filter(|k| !new_keys.contains_key(*k)).cloned().collect();

        // Find updated items (in both but position changed or content changed)
        let updated: Vec<String> = new_keys
            .iter()
            .filter(|(k, &new_idx)| {
                if let Some(&old_idx) = old_keys.get(*k) {
                    // Item exists in both, check if position changed
                    new_idx != old_idx
                } else {
                    false
                }
            })
            .map(|(k, _)| k.clone())
            .collect();

        (added, removed, updated)
    }

    /// Apply efficient add/remove/update operations to component children
    /// For MVP, this is a placeholder that will be expanded with full implementation
    pub fn update_children(
        component: &Component,
        _added_keys: &[String],
        _removed_keys: &[String],
        _updated_keys: &[String],
    ) {
        // Remove components for removed keys
        component.children.borrow_mut().retain(|_child| {
            // In a full implementation, we'd check the child's key
            // For MVP, we'll keep all children
            true
        });

        // Add components for added keys
        // In a full implementation, we'd create a new component for each key
        // For MVP, this is a placeholder

        // Update components for updated keys
        // In a full implementation, we'd update the existing component
        // For MVP, this is a placeholder
    }
}
