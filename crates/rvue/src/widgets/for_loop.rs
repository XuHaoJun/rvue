//! For widget for list rendering

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// For widget builder for rendering lists of items
#[derive(Clone)]
pub struct ForWidget {
    item_count: ReactiveValue<usize>,
}

unsafe impl Trace for ForWidget {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.item_count.trace(visitor);
    }
}

impl ForWidget {
    /// Create a new For widget with item count
    pub fn new(item_count: impl crate::widget::IntoReactiveValue<usize>) -> Self {
        Self { item_count: item_count.into_reactive() }
    }
}

/// State for a mounted For widget
pub struct ForState {
    component: Gc<Component>,
    item_count_effect: Option<Gc<crate::effect::Effect>>,
}

impl ForState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for ForState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.item_count_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for ForState {
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

impl Widget for ForWidget {
    type State = ForState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let initial_item_count = self.item_count.get();

        let component = Component::new(
            id,
            ComponentType::For,
            ComponentProps::For { item_count: initial_item_count },
        );

        // Setup reactive update if item_count is reactive
        let item_count_effect = if self.item_count.is_reactive() {
            let comp = Gc::clone(&component);
            let item_count = self.item_count.clone();
            let effect = create_effect(move || {
                let new_item_count = item_count.get();
                comp.set_for_item_count(new_item_count);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        ForState { component, item_count_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update item_count if it changed
        if self.item_count.is_reactive() {
            // Item count is reactive, effect will handle updates
            if state.item_count_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let item_count = self.item_count.clone();
                let effect = create_effect(move || {
                    let new_item_count = item_count.get();
                    comp.set_for_item_count(new_item_count);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.item_count_effect = Some(effect);
            }
        } else {
            // Static item count - update directly
            let new_item_count = self.item_count.get();
            state.component.set_for_item_count(new_item_count);
        }
    }
}
