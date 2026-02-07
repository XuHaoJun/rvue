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
    clip: ReactiveValue<bool>,
}

unsafe impl Trace for TextInput {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.styles.trace(visitor);
        self.clip.trace(visitor);
    }
}

impl TextInput {
    /// Create a new TextInput widget with a value
    pub fn new(value: impl crate::widget::IntoReactiveValue<String>) -> Self {
        Self { value: value.into_reactive(), styles: None, clip: ReactiveValue::Static(true) }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
    }

    /// Whether to clip the text to the input boundaries.
    ///
    /// When true (default), text that overflows the input box will be hidden.
    /// When false, overflowing text will be visible.
    pub fn clip(mut self, clip: impl Into<ReactiveValue<bool>>) -> Self {
        self.clip = clip.into();
        self
    }
}

/// State for a mounted TextInput widget
pub struct TextInputState {
    component: Gc<Component>,
    value_effect: Option<Gc<crate::effect::Effect>>,
    clip_effect: Option<Gc<crate::effect::Effect>>,
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
        if let Some(effect) = &self.clip_effect {
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

        // Initialize text editor state for TextInput
        component.init_text_editor(&initial_value);

        // Initialize WidgetStyles in PropertyMap for layout calculations
        let mut widget_styles = computed_styles.unwrap_or_default();
        if widget_styles.width.is_none() {
            widget_styles.width = Some(rvue_style::Width(rvue_style::Size::Pixels(200.0)));
        }
        if widget_styles.height.is_none() {
            widget_styles.height = Some(rvue_style::Height(rvue_style::Size::Pixels(30.0)));
        }
        component.set_widget_styles(widget_styles);

        // Set clip to true by default (text overflow will be hidden)
        component.set_clip(self.clip.get());

        let value_effect = if is_reactive {
            let comp = Gc::clone(&component);
            let value = self.value.clone();
            let effect = create_effect(move || {
                let new_value = value.get();
                // Update both the property and the text editor
                comp.set_text_input_value(new_value.clone());
                if let Some(editor) = comp.text_editor() {
                    editor.editor().set_content(new_value);
                }
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        TextInputState { component, value_effect, clip_effect: None }
    }

    fn rebuild(self, state: &mut Self::State) {
        let clip = self.clip.get();
        state.component.set_clip(clip);

        if self.clip.is_reactive() && state.clip_effect.is_none() {
            let comp = Gc::clone(&state.component);
            let clip = self.clip.clone();
            let effect = create_effect(move || {
                comp.set_clip(clip.get());
            });
            state.component.add_effect(Gc::clone(&effect));
            state.clip_effect = Some(effect);
        }

        if self.value.is_reactive() {
            if state.value_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let value = self.value.clone();
                let effect = create_effect(move || {
                    let new_value = value.get();
                    comp.set_text_input_value(new_value.clone());
                    if let Some(editor) = comp.text_editor() {
                        editor.editor().set_content(new_value);
                    }
                });
                state.component.add_effect(Gc::clone(&effect));
                state.value_effect = Some(effect);
            }
        } else {
            let new_value = self.value.get();
            state.component.set_text_input_value(new_value.clone());
            if let Some(editor) = state.component.text_editor() {
                editor.editor().set_content(new_value);
            }
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
        let mut widget_styles = computed_styles.unwrap_or_default();
        if widget_styles.width.is_none() {
            widget_styles.width = Some(rvue_style::Width(rvue_style::Size::Pixels(100.0)));
        }
        if widget_styles.height.is_none() {
            widget_styles.height = Some(rvue_style::Height(rvue_style::Size::Pixels(30.0)));
        }
        component.set_widget_styles(widget_styles);

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
