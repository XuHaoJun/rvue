//! Checkbox widget component

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::SignalRead;
use rudo_gc::Gc;

/// Checkbox widget for boolean input
pub struct Checkbox;

impl Checkbox {
    /// Create a new Checkbox component with a static checked state
    pub fn new(id: ComponentId, checked: bool) -> Gc<Component> {
        Component::new(id, ComponentType::Checkbox, ComponentProps::Checkbox { checked })
    }

    /// Create a new Checkbox component with a reactive signal
    pub fn from_signal<T: SignalRead<bool> + Clone + 'static>(
        id: ComponentId,
        checked_signal: T,
    ) -> Gc<Component> {
        let checked = checked_signal.get();
        let component =
            Component::new(id, ComponentType::Checkbox, ComponentProps::Checkbox { checked });

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = checked_signal.clone();
        let effect = create_effect(move || {
            let new_checked = sig.get();
            *comp.props.borrow_mut() = ComponentProps::Checkbox { checked: new_checked };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}
