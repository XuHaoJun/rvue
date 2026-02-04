//! Button widget component

use crate::component::{Component, ComponentProps, ComponentType};
use crate::widget::{BuildContext, Mountable, Widget};
use rudo_gc::{Gc, Trace};
use rvue_style::ReactiveStyles;

/// Button widget builder for user interaction
#[derive(Clone)]
pub struct Button {
    styles: Option<ReactiveStyles>,
    class: Option<String>,
    id: Option<String>,
}

unsafe impl Trace for Button {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        self.styles.trace(_visitor);
        // class and id are String, no GC pointers to trace
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    /// Create a new Button widget
    pub fn new() -> Self {
        Self { styles: None, class: None, id: None }
    }

    /// Set the styles directly
    pub fn styles(mut self, styles: ReactiveStyles) -> Self {
        self.styles = Some(styles);
        self
    }

    /// Set the CSS class for this button
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the element ID for this button
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
}

/// State for a mounted Button widget
pub struct ButtonState {
    component: Gc<Component>,
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

impl Widget for Button {
    type State = ButtonState;

    fn build(self, _ctx: &mut BuildContext) -> Self::State {
        let id = crate::component::next_component_id();
        let computed_styles = self.styles.as_ref().map(|s| s.compute());
        let class = self.class.clone();
        let element_id = self.id.clone();

        let component = Component::new(
            id,
            ComponentType::Button,
            ComponentProps::Button { styles: computed_styles.clone() },
        );

        // Initialize WidgetStyles in PropertyMap for layout calculations
        if let Some(styles) = computed_styles {
            component.set_widget_styles(styles);
        }

        if let Some(ref cls) = class {
            // Split class string by whitespace to support multiple classes (e.g., "primary large")
            for class_part in cls.split_whitespace() {
                if !class_part.is_empty() {
                    component.classes.borrow_mut().push(class_part.to_string());
                }
            }
        }

        if let Some(ref eid) = element_id {
            *component.element_id.borrow_mut() = Some(eid.clone());
        }

        ButtonState { component }
    }

    fn rebuild(self, _state: &mut Self::State) {
        // Button has no reactive props to update, children are handled by the framework
    }
}
