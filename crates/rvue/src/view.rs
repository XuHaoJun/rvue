//! View trait and structure for declarative UI

use crate::component::Component;
use crate::effect::Effect;
use rudo_gc::{Gc, Trace};

/// View trait for converting types into components
pub trait View {
    /// Convert this type into a component
    fn into_component(self) -> Gc<Component>;
}

/// View structure representing a declarative UI tree
#[derive(Clone)]
pub struct ViewStruct {
    pub root_component: Gc<Component>,
    pub effects: Vec<Gc<Effect>>,
}

unsafe impl Trace for ViewStruct {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.root_component.trace(visitor);
        for effect in &self.effects {
            effect.trace(visitor);
        }
    }
}

impl ViewStruct {
    /// Create a new view with a root component
    pub fn new(root_component: Gc<Component>) -> Self {
        Self { root_component, effects: Vec::new() }
    }

    /// Add a top-level effect to the view
    pub fn add_effect(&mut self, effect: Gc<Effect>) {
        self.effects.push(effect);
    }
}

/// Implement View for ViewStruct
impl View for ViewStruct {
    fn into_component(self) -> Gc<Component> {
        self.root_component
    }
}

/// Implement View for Component (components are views)
impl View for Gc<Component> {
    fn into_component(self) -> Gc<Component> {
        self
    }
}
