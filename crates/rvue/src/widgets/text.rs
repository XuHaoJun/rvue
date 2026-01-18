//! Text widget component

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::signal::{ReadSignal, SignalRead};
use rudo_gc::Gc;

/// Text widget for displaying text content
pub struct Text;

impl Text {
    /// Create a new Text component with static content
    pub fn new(id: ComponentId, content: String) -> Gc<Component> {
        Component::new(id, ComponentType::Text, ComponentProps::Text { content })
    }

    /// Create a new Text component with reactive content from a signal
    pub fn from_signal(id: ComponentId, signal: ReadSignal<String>) -> Gc<Component> {
        let initial_content = signal.get();
        let component = Component::new(
            id,
            ComponentType::Text,
            ComponentProps::Text { content: initial_content },
        );

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = signal.clone();
        let effect = create_effect(move || {
            let new_content = sig.get();
            *comp.props.borrow_mut() = ComponentProps::Text { content: new_content };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}
