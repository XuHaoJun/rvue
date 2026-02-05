//! Input widget components (TextInput, NumberInput)

use crate::component::{Component, ComponentType};
use crate::effect::create_effect;
use crate::properties::{NumberInputValue, PropertyMap, TextInputValue};
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};
use rvue_style::ReactiveStyles;

/// TextInput widget builder for text input
#[derive(Clone)]
pub struct TextInput {
    value: ReactiveValue<String>,
    styles: Option<ReactiveStyles>,
}

unsafe impl Trace for TextInput {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.styles.trace(visitor);
    }
}

impl TextInput {
    /// Create a new TextInput widget with a value
    pub fn new(value: impl crate::widget::IntoReactiveValue<String>) -> Self {
        Self { value: value.into_reactive(), styles: None }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
    }
}

/// State for a mounted TextInput widget
pub struct TextInputState {
    component: Gc<Component>,
    value_effect: Option<Gc<crate::effect::Effect>>,
}

impl TextInputState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for TextInputState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.value_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for TextInputState {
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

impl Widget for TextInput {
    type State = TextInputState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_value = self.value.get();
        let is_reactive = self.value.is_reactive();
        let computed_styles = self.styles.as_ref().map(|s| s.compute());

        let properties = if is_reactive {
            PropertyMap::new()
        } else {
            PropertyMap::with(TextInputValue(initial_value.clone()))
        };

        let component = Component::with_properties(id, ComponentType::TextInput, properties);

        // Initialize WidgetStyles in PropertyMap for layout calculations
        if let Some(styles) = computed_styles {
            component.set_widget_styles(styles);
        }

        let value_effect = if is_reactive {
            let comp = Gc::clone(&component);
            let value = self.value.clone();
            let effect = create_effect(move || {
                let new_value = value.get();
                comp.set_text_input_value(new_value);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        TextInputState { component, value_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.value.is_reactive() {
            if state.value_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let value = self.value.clone();
                let effect = create_effect(move || {
                    let new_value = value.get();
                    comp.set_text_input_value(new_value);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.value_effect = Some(effect);
            }
        } else {
            let new_value = self.value.get();
            state.component.set_text_input_value(new_value);
        }
    }
}

/// NumberInput widget builder for numeric input
#[derive(Clone)]
pub struct NumberInput {
    value: ReactiveValue<f64>,
    styles: Option<ReactiveStyles>,
}

unsafe impl Trace for NumberInput {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.styles.trace(visitor);
    }
}

impl NumberInput {
    /// Create a new NumberInput widget with a value
    pub fn new(value: impl crate::widget::IntoReactiveValue<f64>) -> Self {
        Self { value: value.into_reactive(), styles: None }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
    }
}

/// State for a mounted NumberInput widget
pub struct NumberInputState {
    component: Gc<Component>,
    value_effect: Option<Gc<crate::effect::Effect>>,
}

impl NumberInputState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for NumberInputState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.value_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for NumberInputState {
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

impl Widget for NumberInput {
    type State = NumberInputState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_value = self.value.get();
        let is_reactive = self.value.is_reactive();
        let computed_styles = self.styles.as_ref().map(|s| s.compute());

        let properties = if is_reactive {
            PropertyMap::new()
        } else {
            PropertyMap::with(NumberInputValue(initial_value))
        };

        let component = Component::with_properties(id, ComponentType::NumberInput, properties);

        // Initialize WidgetStyles in PropertyMap for layout calculations
        if let Some(styles) = computed_styles {
            component.set_widget_styles(styles);
        }

        let value_effect = if is_reactive {
            let comp = Gc::clone(&component);
            let value = self.value.clone();
            let effect = create_effect(move || {
                let new_value = value.get();
                comp.set_number_input_value(new_value);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        NumberInputState { component, value_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        if self.value.is_reactive() {
            if state.value_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let value = self.value.clone();
                let effect = create_effect(move || {
                    let new_value = value.get();
                    comp.set_number_input_value(new_value);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.value_effect = Some(effect);
            }
        } else {
            let new_value = self.value.get();
            state.component.set_number_input_value(new_value);
        }
    }
}
