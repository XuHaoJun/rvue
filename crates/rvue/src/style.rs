//! Style resolution combining inline styles and stylesheet rules.
//!
//! This module provides CSS selector matching and style resolution for components,
//! supporting class selectors (`.class`), ID selectors (`#id`), and pseudo-classes
//! (`:hover`, `:focus`, `:disabled`).

use std::cell::RefCell;
use std::rc::Rc;

use crate::component::Component;
use rudo_gc::Gc;
use rvue_style::{
    BackgroundColor, Color, ComputedStyles, ElementState, Properties, RvueElement, StyleResolver,
};

#[derive(Debug)]
pub struct Stylesheet {
    inner: Rc<RefCell<rvue_style::Stylesheet>>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self { inner: Rc::new(RefCell::new(rvue_style::Stylesheet::new())) }
    }

    pub fn add_rule(&mut self, selector: &str, properties: Properties) {
        self.inner
            .borrow_mut()
            .add_rule(rvue_style::StyleRule::new(selector.to_string(), properties));
    }

    /// Add a background-color rule for a selector
    pub fn add_background_color(&mut self, selector: &str, color: Color) {
        let mut props = Properties::new();
        props.insert(BackgroundColor(color));
        self.add_rule(selector, props);
    }

    /// Add a background-color rule for a selector and its hover state
    pub fn add_background_color_with_hover(
        &mut self,
        selector: &str,
        normal_color: Color,
        hover_color: Color,
    ) {
        self.add_background_color(selector, normal_color);
        self.add_background_color(&format!("{}:hover", selector), hover_color);
    }

    /// Add a background-color rule for a selector with hover, active, and disabled states
    pub fn add_interactive_colors(
        &mut self,
        selector: &str,
        normal: Color,
        hover: Color,
        active: Color,
        disabled: Color,
    ) {
        self.add_background_color(selector, normal);
        self.add_background_color(&format!("{}:hover", selector), hover);
        self.add_background_color(&format!("{}:active", selector), active);
        self.add_background_color(&format!("{}:disabled", selector), disabled);
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}

impl Clone for Stylesheet {
    fn clone(&self) -> Self {
        Self { inner: Rc::clone(&self.inner) }
    }
}

impl Default for Stylesheet {
    fn default() -> Self {
        Self::new()
    }
}

pub fn component_to_element(component: &Gc<Component>) -> RvueElement {
    let tag_name = component_type_to_tag_name(&component.component_type);

    let mut element = RvueElement::new(&tag_name);

    for class in component.classes.borrow().iter() {
        element = element.with_class(class);
    }

    if let Some(id) = component.element_id.borrow().as_ref() {
        element = element.with_id(id);
    }

    if *component.is_hovered.borrow() {
        element.state.insert(ElementState::HOVER);
    }
    if *component.is_focused.borrow() {
        element.state.insert(ElementState::FOCUS);
    }
    if *component.is_active.borrow() {
        element.state.insert(ElementState::ACTIVE);
    }
    if component.is_disabled() {
        element.state.insert(ElementState::DISABLED);
    }

    element
}

fn component_type_to_tag_name(component_type: &crate::component::ComponentType) -> String {
    match component_type {
        crate::component::ComponentType::Text => "text",
        crate::component::ComponentType::Button => "button",
        crate::component::ComponentType::TextInput => "input",
        crate::component::ComponentType::NumberInput => "input",
        crate::component::ComponentType::Checkbox => "checkbox",
        crate::component::ComponentType::Radio => "radio",
        crate::component::ComponentType::Show => "show",
        crate::component::ComponentType::For => "for",
        crate::component::ComponentType::Flex => "flex",
        crate::component::ComponentType::Custom(name) => name,
    }
    .to_string()
}

pub fn resolve_styles(component: &Gc<Component>, stylesheet: &Stylesheet) -> ComputedStyles {
    let element = component_to_element(component);
    eprintln!("[DEBUG-RESOLVE] component_type={:?}, element_tag={}, element_classes={:?}, element_id={:?}, element_state={:?}",
        component.component_type,
        element.tag_name,
        element.classes,
        element.id,
        element.state);

    let resolver = StyleResolver::new();

    let inline_styles = get_inline_styles(component);

    let resolved = resolver.resolve_styles(&element, &stylesheet.inner.borrow());

    eprintln!(
        "[DEBUG-RESOLVE] resolved has_bg={}, bg={:?}",
        resolved.background_color.is_some(),
        resolved.background_color.as_ref().map(|bg| (bg.0 .0.r, bg.0 .0.g, bg.0 .0.b))
    );

    let mut merged = resolved;

    if let Some(inline) = inline_styles {
        merged.merge_with_computed(&inline);
    }

    merged
}

fn get_inline_styles(component: &Gc<Component>) -> Option<ComputedStyles> {
    use crate::component::ComponentProps;

    let props = component.props.borrow();

    match &*props {
        ComponentProps::Text { styles, .. } => styles.clone(),
        ComponentProps::Button { styles, .. } => styles.clone(),
        ComponentProps::TextInput { styles, .. } => styles.clone(),
        ComponentProps::NumberInput { styles, .. } => styles.clone(),
        ComponentProps::Checkbox { styles, .. } => styles.clone(),
        ComponentProps::Radio { styles, .. } => styles.clone(),
        ComponentProps::Flex { styles, .. } => styles.clone(),
        _ => None,
    }
}

pub trait StylesheetProvider {
    fn stylesheet(&self) -> Option<&Stylesheet>;
}

impl StylesheetProvider for () {
    fn stylesheet(&self) -> Option<&Stylesheet> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, ComponentProps, ComponentType};
    use rvue_style::properties::BackgroundColor;

    #[test]
    fn test_component_to_element() {
        let component = Component::new(
            1,
            ComponentType::Button,
            ComponentProps::Button { label: "Test".into(), styles: None },
        );

        let element = component_to_element(&component);

        assert_eq!(element.tag_name.as_ref(), "button");
        assert!(!element.state.intersects(ElementState::HOVER));
    }

    #[test]
    fn test_component_with_classes() {
        let component = Component::new(
            1,
            ComponentType::Button,
            ComponentProps::Button { label: "Test".into(), styles: None },
        );

        component.classes.borrow_mut().push("primary".into());
        component.classes.borrow_mut().push("large".into());

        let element = component_to_element(&component);

        assert!(element.has_class("primary"));
        assert!(element.has_class("large"));
        assert!(!element.has_class("small"));
    }

    #[test]
    fn test_component_with_state() {
        let component = Component::new(
            1,
            ComponentType::Button,
            ComponentProps::Button { label: "Test".into(), styles: None },
        );

        *component.is_hovered.borrow_mut() = true;
        *component.is_focused.borrow_mut() = true;

        let element = component_to_element(&component);

        assert!(element.state.intersects(ElementState::HOVER));
        assert!(element.state.intersects(ElementState::FOCUS));
        assert!(!element.state.intersects(ElementState::ACTIVE));
    }

    #[test]
    fn test_stylesheet_add_rule() {
        let mut stylesheet = Stylesheet::new();

        let mut props = rvue_style::Properties::new();
        props.insert(BackgroundColor(rvue_style::properties::Color::rgb(255, 0, 0)));

        stylesheet.add_rule("button", props);

        assert_eq!(stylesheet.len(), 1);
        assert!(!stylesheet.is_empty());
    }
}
