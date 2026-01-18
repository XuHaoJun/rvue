//! Radio widget component

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::SignalRead;
use rudo_gc::Gc;

/// Radio widget for single selection from multiple options
pub struct Radio;

impl Radio {
    /// Create a new Radio component with a static value and checked state
    pub fn new(id: ComponentId, value: String, checked: bool) -> Gc<Component> {
        Component::new(id, ComponentType::Radio, ComponentProps::Radio { value, checked })
    }

    /// Create a new Radio component with a reactive checked signal
    pub fn from_signal<T: SignalRead<String> + Clone + 'static>(
        id: ComponentId,
        value: String,
        checked_signal: T,
    ) -> Gc<Component> {
        // Check if this radio's value matches the signal value
        let initial_signal_value = checked_signal.get();
        let initial_checked = initial_signal_value == value;

        let component = Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value: value.clone(), checked: initial_checked },
        );

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = checked_signal.clone();
        let radio_value = value;
        let effect = create_effect(move || {
            let signal_value = sig.get();
            let checked = signal_value == radio_value;
            *comp.props.borrow_mut() =
                ComponentProps::Radio { value: radio_value.clone(), checked };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}
