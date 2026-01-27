//! Radio widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Radio widget builder for single selection from multiple options
#[derive(Clone)]
pub struct Radio {
    value: String,
    checked: ReactiveValue<bool>,
}

unsafe impl Trace for Radio {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.checked.trace(visitor);
    }
}

impl Radio {
    /// Create a new Radio widget with a value and checked state
    pub fn new(value: String, checked: impl crate::widget::IntoReactiveValue<bool>) -> Self {
        Self { value, checked: checked.into_reactive() }
    }
}

/// State for a mounted Radio widget
pub struct RadioState {
    component: Gc<Component>,
    checked_effect: Option<Gc<crate::effect::Effect>>,
}

impl RadioState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for RadioState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.checked_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for RadioState {
    fn mount(&self, parent: Option<Gc<Component>>) {
        self.component.set_parent(parent.clone());
        if let Some(parent) = parent {
            parent.add_child(Gc::clone(&self.component));
        }
    }

    fn unmount(&self) {
        self.component.set_parent(None);
    }
}

impl Widget for Radio {
    type State = RadioState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_checked = self.checked.get();

        let component = Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value: self.value.clone(), checked: initial_checked },
        );

        // Setup reactive update if checked is reactive
        let checked_effect = if self.checked.is_reactive() {
            let comp = Gc::clone(&component);
            let checked = self.checked.clone();
            let effect = create_effect(move || {
                let new_checked = checked.get();
                comp.set_radio_checked(new_checked);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        RadioState { component, checked_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update checked if it changed
        if self.checked.is_reactive() {
            // Checked is reactive, effect will handle updates
            if state.checked_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let checked = self.checked.clone();
                let effect = create_effect(move || {
                    let new_checked = checked.get();
                    comp.set_radio_checked(new_checked);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.checked_effect = Some(effect);
            }
        } else {
            // Static checked - update directly
            let new_checked = self.checked.get();
            state.component.set_radio_checked(new_checked);
        }
    }
}
