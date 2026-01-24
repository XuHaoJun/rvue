//! Show widget for conditional rendering

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Show widget builder for conditionally rendering content
#[derive(Clone)]
pub struct Show {
    when: ReactiveValue<bool>,
}

unsafe impl Trace for Show {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.when.trace(visitor);
    }
}

impl Show {
    /// Create a new Show widget with a boolean condition
    pub fn new(when: impl crate::widget::IntoReactiveValue<bool>) -> Self {
        Self { when: when.into_reactive() }
    }
}

/// State for a mounted Show widget
pub struct ShowState {
    component: Gc<Component>,
    when_effect: Option<Gc<crate::effect::Effect>>,
}

impl ShowState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for ShowState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.when_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for ShowState {
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

impl Widget for Show {
    type State = ShowState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let initial_when = self.when.get();

        let component =
            Component::new(id, ComponentType::Show, ComponentProps::Show { when: initial_when });

        // Setup reactive update if when is reactive
        let when_effect = if self.when.is_reactive() {
            let comp = Gc::clone(&component);
            let when = self.when.clone();
            let effect = create_effect(move || {
                let new_when = when.get();
                comp.set_show_when(new_when);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        ShowState { component, when_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update when if it changed
        if self.when.is_reactive() {
            // When is reactive, effect will handle updates
            if state.when_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let when = self.when.clone();
                let effect = create_effect(move || {
                    let new_when = when.get();
                    comp.set_show_when(new_when);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.when_effect = Some(effect);
            }
        } else {
            // Static when - update directly
            let new_when = self.when.get();
            state.component.set_show_when(new_when);
        }
    }
}
