//! Checkbox widget component

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps, ComponentId};
use crate::signal::SignalRead;

/// Checkbox widget for boolean input
pub struct Checkbox;

impl Checkbox {
    /// Create a new Checkbox component with a static checked state
    pub fn new(id: ComponentId, checked: bool) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::Checkbox,
            ComponentProps::Checkbox { checked },
        )
    }

    /// Create a new Checkbox component with a reactive signal
    pub fn from_signal<T: SignalRead<bool> + Clone>(id: ComponentId, checked_signal: T) -> Gc<Component> {
        let checked = checked_signal.get();
        Component::new(
            id,
            ComponentType::Checkbox,
            ComponentProps::Checkbox { checked },
        )
    }
}
