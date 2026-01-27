//! Show widget for conditional rendering

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::view::View;
use crate::widget::{with_current_ctx, BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Show widget builder for conditionally rendering content
#[derive(Clone)]
pub struct Show<CF> {
    when: ReactiveValue<bool>,
    children_fn: CF,
}

unsafe impl<CF> Trace for Show<CF>
where
    CF: Trace,
{
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.when.trace(visitor);
        self.children_fn.trace(visitor);
    }
}

impl<CF> Show<CF>
where
    CF: Clone + 'static,
{
    /// Create a new Show widget with a boolean condition and children builder
    pub fn new(when: impl crate::widget::IntoReactiveValue<bool>, children_fn: CF) -> Self {
        Self { when: when.into_reactive(), children_fn }
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

impl<CF> Widget for Show<CF>
where
    CF: Fn() -> crate::ViewStruct + Clone + 'static,
{
    type State = ShowState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let initial_when = self.when.get();

        let component =
            Component::new(id, ComponentType::Show, ComponentProps::Show { when: initial_when });

        // Build children using the same context
        let view = with_current_ctx(ctx.id_counter, || (self.children_fn)());
        let child_component = view.into_component();
        child_component.set_parent(Some(Gc::clone(&component)));
        component.add_child(child_component);

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
            let new_when = self.when.get();
            state.component.set_show_when(new_when);
        }
    }
}
