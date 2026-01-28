//! Text widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Text widget builder for displaying text content
#[derive(Clone)]
pub struct Text {
    content: ReactiveValue<String>,
    font_size: Option<f32>,
    color: Option<vello::peniko::Color>,
}

unsafe impl Trace for Text {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.content.trace(visitor);
    }
}

impl Text {
    /// Create a new Text widget with content
    pub fn new(content: impl crate::widget::IntoReactiveValue<String>) -> Self {
        Self { content: content.into_reactive(), font_size: None, color: None }
    }

    /// Set the font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set the text color
    pub fn color(mut self, color: vello::peniko::Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// State for a mounted Text widget
pub struct TextState {
    component: Gc<Component>,
    content_effect: Option<Gc<crate::effect::Effect>>,
}

impl TextState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for TextState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.content_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for TextState {
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

impl Widget for Text {
    type State = TextState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let initial_content = self.content.get();

        let component = Component::new(
            id,
            ComponentType::Text,
            ComponentProps::Text {
                content: initial_content.clone(),
                font_size: self.font_size,
                color: self.color,
            },
        );

        // Setup reactive update if content is reactive
        let content_effect = if self.content.is_reactive() {
            let comp = Gc::clone(&component);
            let content = self.content.clone();
            let effect = create_effect(move || {
                let new_content = content.get();
                comp.set_text_content(new_content);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        TextState { component, content_effect }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update content if it changed
        if self.content.is_reactive() {
            // Content is reactive, effect will handle updates
            // Just ensure the effect is still set up
            if state.content_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let content = self.content.clone();
                let effect = create_effect(move || {
                    let new_content = content.get();
                    comp.set_text_content(new_content);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.content_effect = Some(effect);
            }
        } else {
            // Static content - update directly
            let new_content = self.content.get();
            state.component.set_text_content(new_content);
        }

        // Update font_size if changed
        if let Some(font_size) = self.font_size {
            state.component.set_text_font_size(font_size);
        }

        // Update color if changed
        if let Some(color) = self.color {
            state.component.set_text_color(color);
        }
    }
}
