//! Show widget for conditional rendering

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::{ReadSignal, SignalRead};
use rudo_gc::Gc;

/// Show widget for conditionally rendering content
pub struct Show;

impl Show {
    /// Create a new Show component with a boolean condition
    pub fn new(id: ComponentId, when: bool) -> Gc<Component> {
        Component::new(id, ComponentType::Show, ComponentProps::Show { when })
    }

    /// Create a new Show component with a reactive signal
    pub fn from_signal(id: ComponentId, when_signal: ReadSignal<bool>) -> Gc<Component> {
        // Get the current value from the signal
        let initial_when = when_signal.get();
        let component =
            Component::new(id, ComponentType::Show, ComponentProps::Show { when: initial_when });

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = when_signal.clone();
        let effect = create_effect(move || {
            let new_when = sig.get();
            *comp.props.borrow_mut() = ComponentProps::Show { when: new_when };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }

    /// Check if the Show component should display its children
    pub fn should_show(component: &Component) -> bool {
        match &*component.props.borrow() {
            ComponentProps::Show { when } => *when,
            _ => false,
        }
    }
}
