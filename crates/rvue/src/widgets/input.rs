//! Input widget components (TextInput, NumberInput)

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps, ComponentId};
use crate::signal::{ReadSignal, SignalRead};

/// TextInput widget for text input
pub struct TextInput;

impl TextInput {
    /// Create a new TextInput component with a static value
    pub fn new(id: ComponentId, value: String) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::TextInput,
            ComponentProps::TextInput { value },
        )
    }

    /// Create a new TextInput component with a reactive signal
    pub fn from_signal(id: ComponentId, value_signal: ReadSignal<String>) -> Gc<Component> {
        let value = value_signal.get();
        Component::new(
            id,
            ComponentType::TextInput,
            ComponentProps::TextInput { value },
        )
    }
}

/// NumberInput widget for numeric input
pub struct NumberInput;

impl NumberInput {
    /// Create a new NumberInput component with a static value
    pub fn new(id: ComponentId, value: f64) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::NumberInput,
            ComponentProps::NumberInput { value },
        )
    }

    /// Create a new NumberInput component with a reactive signal
    pub fn from_signal<T: SignalRead<f64> + Clone>(id: ComponentId, value_signal: T) -> Gc<Component> {
        let value = value_signal.get();
        Component::new(
            id,
            ComponentType::NumberInput,
            ComponentProps::NumberInput { value },
        )
    }
}
