//! Checkbox widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};
use rvue_style::ReactiveStyles;

/// Checkbox widget builder for boolean input
#[derive(Clone)]
pub struct Checkbox {
    checked: ReactiveValue<bool>,
    styles: Option<ReactiveStyles>,
}

unsafe impl Trace for Checkbox {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.checked.trace(visitor);
        self.styles.trace(visitor);
    }
}

impl Checkbox {
    /// Create a new Checkbox widget with checked state
    pub fn new(checked: impl crate::widget::IntoReactiveValue<bool>) -> Self {
        Self { checked: checked.into_reactive(), styles: None }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
    }
}

/// State for a mounted Checkbox widget
pub struct CheckboxState {
    component: Gc<Component>,
    checked_effect: Option<Gc<crate::effect::Effect>>,
}

impl CheckboxState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for CheckboxState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.checked_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for CheckboxState {
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

impl Widget for Checkbox {
    type State = CheckboxState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_checked = self.checked.get();
        let computed_styles = self.styles.as_ref().map(|s| s.compute());

        let component = Component::new(
            id,
            ComponentType::Checkbox,
            ComponentProps::Checkbox { checked: initial_checked, styles: computed_styles },
        );

        let checked_effect = if self.checked.is_reactive() {
            let comp = Gc::clone(&component);
            let checked = self.checked.clone();
            let effect = create_effect(move || {
                let new_checked = checked.get();
                comp.set_checkbox_checked(new_checked);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        CheckboxState { component, checked_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.checked.is_reactive() {
            if state.checked_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let checked = self.checked.clone();
                let effect = create_effect(move || {
                    let new_checked = checked.get();
                    comp.set_checkbox_checked(new_checked);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.checked_effect = Some(effect);
            }
        } else {
            let new_checked = self.checked.get();
            state.component.set_checkbox_checked(new_checked);
        }
    }
}
