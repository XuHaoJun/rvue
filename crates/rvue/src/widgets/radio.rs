//! Radio widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::properties::{CheckboxChecked, PropertyMap, RadioValue};
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};
use rvue_style::ReactiveStyles;

/// Radio widget builder for single selection from multiple options
#[derive(Clone)]
pub struct Radio {
    value: String,
    checked: ReactiveValue<bool>,
    styles: Option<ReactiveStyles>,
}

unsafe impl Trace for Radio {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.checked.trace(visitor);
        self.styles.trace(visitor);
    }
}

impl Radio {
    /// Create a new Radio widget with a value and checked state
    pub fn new(value: String, checked: impl crate::widget::IntoReactiveValue<bool>) -> Self {
        Self { value, checked: checked.into_reactive(), styles: None }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
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
        let is_reactive = self.checked.is_reactive();
        let computed_styles = self.styles.as_ref().map(|s| s.compute());

        let properties = PropertyMap::new()
            .and(RadioValue(self.value.clone()))
            .and(CheckboxChecked(initial_checked));

        let component = Component::with_properties(
            id,
            ComponentType::Radio,
            ComponentProps::Radio {
                value: self.value.clone(),
                checked: initial_checked,
                styles: computed_styles.clone(),
            },
            properties,
        );

        // Initialize WidgetStyles in PropertyMap for layout calculations
        if let Some(styles) = computed_styles {
            component.set_widget_styles(styles);
        }

        let checked_effect = if is_reactive {
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
        if self.checked.is_reactive() {
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
            let new_checked = self.checked.get();
            state.component.set_radio_checked(new_checked);
        }
    }
}
