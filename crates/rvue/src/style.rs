//! Style resolution combining inline styles and stylesheet rules.
//!
//! This module provides CSS selector matching and style resolution for components,
//! supporting class selectors (`.class`), ID selectors (`#id`), and pseudo-classes
//! (`:hover`, `:focus`, `:disabled`).

use std::cell::RefCell;
use std::rc::Rc;

use crate::component::Component;
use rvue_style::{
    default_stylesheet, BackgroundColor, Color, ComputedStyles, ElementState, Properties,
    RvueElement, StyleResolver,
};

#[derive(Debug)]
pub struct Stylesheet {
    inner: Rc<RefCell<rvue_style::Stylesheet>>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self { inner: Rc::new(RefCell::new(rvue_style::Stylesheet::new())) }
    }

    pub fn with_defaults() -> Self {
        let defaults = default_stylesheet();
        Self { inner: Rc::new(RefCell::new(defaults)) }
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

    pub fn merge(&mut self, other: &Stylesheet) {
        for rule in other.inner.borrow().rules() {
            self.inner.borrow_mut().add_rule(rule.clone());
        }
    }

    pub fn inner(&self) -> impl std::ops::Deref<Target = rvue_style::Stylesheet> + '_ {
        self.inner.borrow()
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

pub fn component_to_element(component: &Component) -> RvueElement {
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

/// Resolve styles for a component (accepts &Component instead of &Gc<Component>)
/// This provides a unified style resolution path for both layout and rendering systems.
pub fn resolve_styles_for_component(
    component: &Component,
    stylesheet: &Stylesheet,
) -> ComputedStyles {
    let element = component_to_element(component);

    let resolver = StyleResolver::new();

    let inline_styles = get_inline_styles(component);

    let inner_sheet = stylesheet.inner.borrow();

    let resolved = resolver.resolve_styles(&element, &inner_sheet);

    let mut merged = resolved;

    if let Some(inline) = inline_styles {
        // Only merge inline properties that differ from defaults
        // This prevents default values (like Width(Auto)) from overriding stylesheet values
        if let Some(w) = inline.width.as_ref() {
            if !matches!(w.0, rvue_style::Size::Auto) {
                merged.width = Some(w.clone());
            }
        }
        if let Some(h) = inline.height.as_ref() {
            if !matches!(h.0, rvue_style::Size::Auto) {
                merged.height = Some(h.clone());
            }
        }
        if inline.background_color.is_some() {
            merged.background_color = inline.background_color;
        }
        if inline.color.is_some() {
            merged.color = inline.color;
        }
        if inline.text_color.is_some() {
            merged.text_color = inline.text_color;
        }
        if inline.font_size.is_some() {
            merged.font_size = inline.font_size;
        }
        if inline.font_family.is_some() {
            merged.font_family = inline.font_family;
        }
        if inline.font_weight.is_some() {
            merged.font_weight = inline.font_weight;
        }
        if inline.padding.is_some() {
            merged.padding = inline.padding;
        }
        if inline.margin.is_some() {
            merged.margin = inline.margin;
        }
        if inline.display.is_some() {
            merged.display = inline.display;
        }
        if inline.flex_direction.is_some() {
            merged.flex_direction = inline.flex_direction;
        }
        if inline.justify_content.is_some() {
            merged.justify_content = inline.justify_content;
        }
        if inline.align_items.is_some() {
            merged.align_items = inline.align_items;
        }
        if inline.align_self.is_some() {
            merged.align_self = inline.align_self;
        }
        if inline.flex_grow.is_some() {
            merged.flex_grow = inline.flex_grow;
        }
        if inline.flex_shrink.is_some() {
            merged.flex_shrink = inline.flex_shrink;
        }
        if inline.flex_basis.is_some() {
            merged.flex_basis = inline.flex_basis;
        }
        if inline.gap.is_some() {
            merged.gap = inline.gap;
        }
        if inline.border_color.is_some() {
            merged.border_color = inline.border_color;
        }
        if inline.border_width.is_some() {
            merged.border_width = inline.border_width;
        }
        if inline.border_radius.is_some() {
            merged.border_radius = inline.border_radius;
        }
        if inline.border_style.is_some() {
            merged.border_style = inline.border_style;
        }
        if inline.opacity.is_some() {
            merged.opacity = inline.opacity;
        }
        if inline.visibility.is_some() {
            merged.visibility = inline.visibility;
        }
        if inline.z_index.is_some() {
            merged.z_index = inline.z_index;
        }
        if inline.cursor.is_some() {
            merged.cursor = inline.cursor;
        }
    }

    merged
}

fn get_inline_styles(component: &Component) -> Option<ComputedStyles> {
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
        let component =
            Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

        let element = component_to_element(&component);

        assert_eq!(element.tag_name.as_ref(), "button");
        assert!(!element.state.intersects(ElementState::HOVER));
    }

    #[test]
    fn test_component_with_classes() {
        let component =
            Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

        component.classes.borrow_mut().push("primary".into());
        component.classes.borrow_mut().push("large".into());

        let element = component_to_element(&component);

        assert!(element.has_class("primary"));
        assert!(element.has_class("large"));
        assert!(!element.has_class("small"));
    }

    #[test]
    fn test_component_with_state() {
        let component =
            Component::new(1, ComponentType::Button, ComponentProps::Button { styles: None });

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
