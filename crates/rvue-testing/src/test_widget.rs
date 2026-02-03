// Copyright 2025 the Rvue Authors
// SPDX-License-Identifier: Apache-2.0

//! Test widget builder utilities.

use rudo_gc::Gc;
use rvue::component::{Component, ComponentProps, ComponentType};
use rvue_style::properties::Overflow;

/// Builder for creating test widgets.
pub struct TestWidgetBuilder {
    tag: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
    overflow_x: Option<Overflow>,
    overflow_y: Option<Overflow>,
    scroll_offset_x: f32,
    scroll_offset_y: f32,
    visible: bool,
    children: Vec<Gc<Component>>,
    styles: Option<rvue_style::ComputedStyles>,
}

impl Default for TestWidgetBuilder {
    fn default() -> Self {
        Self {
            tag: None,
            width: None,
            height: None,
            overflow_x: None,
            overflow_y: None,
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
            visible: true,
            children: Vec::new(),
            styles: None,
        }
    }
}

impl TestWidgetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.to_string());
        self
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_overflow(mut self, overflow: Overflow) -> Self {
        self.overflow_x = Some(overflow);
        self.overflow_y = Some(overflow);
        self
    }

    pub fn with_overflow_x(mut self, overflow: Overflow) -> Self {
        self.overflow_x = Some(overflow);
        self
    }

    pub fn with_overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow_y = Some(overflow);
        self
    }

    pub fn with_scroll_offset(mut self, x: f64, y: f64) -> Self {
        self.scroll_offset_x = x as f32;
        self.scroll_offset_y = y as f32;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_child(mut self, child: Gc<Component>) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<Gc<Component>>) -> Self {
        self.children = children;
        self
    }

    pub fn with_styles(mut self, styles: rvue_style::ComputedStyles) -> Self {
        self.styles = Some(styles);
        self
    }

    fn create_styles(&self) -> Option<rvue_style::ComputedStyles> {
        if self.overflow_x.is_none() && self.overflow_y.is_none() && self.styles.is_none() {
            return None;
        }

        let mut styles = self.styles.clone().unwrap_or_default();

        if let Some(overflow) = self.overflow_x {
            styles.overflow_x = Some(overflow);
        }
        if let Some(overflow) = self.overflow_y {
            styles.overflow_y = Some(overflow);
        }

        Some(styles)
    }

    /// Build a Flex container widget.
    pub fn build_flex(self, direction: &str) -> Gc<Component> {
        let styles = self.create_styles();
        let component = Component::new(
            rand::random(),
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: direction.to_string(),
                gap: 0.0,
                align_items: "start".to_string(),
                justify_content: "start".to_string(),
                styles,
            },
        );

        // Set element_id if tag is provided
        if let Some(tag) = self.tag {
            *component.element_id.borrow_mut() = Some(tag);
        }

        for child in self.children {
            component.add_child(Gc::clone(&child));
        }

        component
    }

    /// Build a custom widget (for containers without specific type).
    /// Uses ComponentProps::Flex internally to support styles like overflow.
    pub fn build_custom(self, data: &str) -> Gc<Component> {
        let styles = self.create_styles();
        let component = Component::new(
            rand::random(),
            ComponentType::Custom(data.to_string()),
            ComponentProps::Flex {
                direction: "column".to_string(),
                gap: 0.0,
                align_items: "start".to_string(),
                justify_content: "start".to_string(),
                styles,
            },
        );

        // Set element_id if tag is provided
        if let Some(tag) = self.tag {
            *component.element_id.borrow_mut() = Some(tag);
        }

        for child in self.children {
            component.add_child(Gc::clone(&child));
        }

        component
    }

    /// Build using the default custom type.
    pub fn build(self) -> Gc<Component> {
        self.build_custom("test-widget")
    }
}

/// Helper function to create a large content widget for testing scroll behavior.
pub fn create_large_content() -> Gc<Component> {
    TestWidgetBuilder::new().with_size(400.0, 800.0).build_custom("large-content")
}

/// Helper function to create a small content widget.
pub fn create_small_content() -> Gc<Component> {
    TestWidgetBuilder::new().with_size(100.0, 100.0).build_custom("small-content")
}

/// Helper function to create a button widget for testing.
pub fn create_button(tag: &str, _text: &str) -> Gc<Component> {
    let component = Component::new(
        rand::random(),
        ComponentType::Button,
        ComponentProps::Button { styles: None },
    );

    *component.element_id.borrow_mut() = Some(tag.to_string());

    component
}
