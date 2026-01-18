//! Input widget components (TextInput, NumberInput)

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::{ReadSignal, SignalRead};
use rudo_gc::Gc;

/// TextInput widget for text input
pub struct TextInput;

impl TextInput {
    /// Create a new TextInput component with a static value
    pub fn new(id: ComponentId, value: String) -> Gc<Component> {
        Component::new(id, ComponentType::TextInput, ComponentProps::TextInput { value })
    }

    /// Create a new TextInput component with a reactive signal
    pub fn from_signal(id: ComponentId, value_signal: ReadSignal<String>) -> Gc<Component> {
        let initial_value = value_signal.get();
        let component = Component::new(
            id,
            ComponentType::TextInput,
            ComponentProps::TextInput { value: initial_value },
        );

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = value_signal.clone();
        let effect = create_effect(move || {
            let new_value = sig.get();
            *comp.props.borrow_mut() = ComponentProps::TextInput { value: new_value };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}

/// NumberInput widget for numeric input
pub struct NumberInput;

impl NumberInput {
    /// Create a new NumberInput component with a static value
    pub fn new(id: ComponentId, value: f64) -> Gc<Component> {
        Component::new(id, ComponentType::NumberInput, ComponentProps::NumberInput { value })
    }

    /// Create a new NumberInput component with a reactive signal
    pub fn from_signal<T: SignalRead<f64> + Clone + 'static>(
        id: ComponentId,
        value_signal: T,
    ) -> Gc<Component> {
        let initial_value = value_signal.get();
        let component = Component::new(
            id,
            ComponentType::NumberInput,
            ComponentProps::NumberInput { value: initial_value },
        );

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = value_signal.clone();
        let effect = create_effect(move || {
            let new_value = sig.get();
            *comp.props.borrow_mut() = ComponentProps::NumberInput { value: new_value };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}
