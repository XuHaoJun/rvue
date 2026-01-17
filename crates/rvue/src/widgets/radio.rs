//! Radio widget component

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps, ComponentId};
use crate::signal::SignalRead;

/// Radio widget for single selection from multiple options
pub struct Radio;

impl Radio {
    /// Create a new Radio component with a static value and checked state
    pub fn new(id: ComponentId, value: String, checked: bool) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value, checked },
        )
    }

    /// Create a new Radio component with a reactive checked signal
    pub fn from_signal<T: SignalRead<String> + Clone>(
        id: ComponentId,
        value: String,
        checked_signal: T,
    ) -> Gc<Component> {
        // Check if this radio's value matches the signal value
        let signal_value = checked_signal.get();
        let checked = signal_value == value;
        
        Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value, checked },
        )
    }
}
