//! Button widget component

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps, ComponentId};

/// Button widget for user interaction
pub struct Button;

impl Button {
    /// Create a new Button component with a label
    pub fn new(id: ComponentId, label: String) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::Button,
            ComponentProps::Button { label },
        )
    }

    /// Create a new Button component with an on_click handler
    /// For MVP, event handlers will be stored separately and connected during rendering
    pub fn with_handler(id: ComponentId, label: String, _on_click: Box<dyn Fn()>) -> Gc<Component> {
        // Event handler will be connected during rendering/event system setup
        Component::new(
            id,
            ComponentType::Button,
            ComponentProps::Button { label },
        )
    }
}
