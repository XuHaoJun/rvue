//! Text widget component

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
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
        // For MVP, we'll create a component and set up an effect to update it
        // The actual reactive update will be handled by the rendering system
        let content = signal.get();
        Component::new(id, ComponentType::Text, ComponentProps::Text { content })
    }
}
