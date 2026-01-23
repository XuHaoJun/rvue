//! Button widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Button widget builder for user interaction
#[derive(Clone)]
pub struct ButtonWidget {
    label: ReactiveValue<String>,
}

unsafe impl Trace for ButtonWidget {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.label.trace(visitor);
    }
}

impl ButtonWidget {
    /// Create a new Button widget with a label
    pub fn new(label: impl crate::widget::IntoReactiveValue<String>) -> Self {
        Self { label: label.into_reactive() }
    }
}

/// State for a mounted Button widget
pub struct ButtonState {
    component: Gc<Component>,
    label_effect: Option<Gc<crate::effect::Effect>>,
}

impl ButtonState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for ButtonState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.label_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for ButtonState {
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

impl Widget for ButtonWidget {
    type State = ButtonState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let initial_label = self.label.get();

        let component = Component::new(
            id,
            ComponentType::Button,
            ComponentProps::Button { label: initial_label },
        );

        // Setup reactive update if label is reactive
        let label_effect = if self.label.is_reactive() {
            let comp = Gc::clone(&component);
            let label = self.label.clone();
            let effect = crate::effect::create_effect(move || {
                let new_label = label.get();
                comp.set_button_label(new_label);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        ButtonState { component, label_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update label if it changed
        if self.label.is_reactive() {
            // Label is reactive, effect will handle updates
            if state.label_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let label = self.label.clone();
                let effect = crate::effect::create_effect(move || {
                    let new_label = label.get();
                    comp.set_button_label(new_label);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.label_effect = Some(effect);
            }
        } else {
            // Static label - update directly
            let new_label = self.label.get();
            state.component.set_button_label(new_label);
        }
    }
}
