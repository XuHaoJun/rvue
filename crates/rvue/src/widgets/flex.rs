//! Flex widget for flexbox layouts

use rudo_gc::Gc;
use crate::component::{Component, ComponentType, ComponentProps, ComponentId};
use crate::style::{FlexDirection, AlignItems, JustifyContent};

/// Flex widget for creating flexbox layouts
pub struct Flex;

impl Flex {
    /// Create a new Flex component with direction, gap, alignment, and justification
    pub fn new(
        id: ComponentId,
        direction: FlexDirection,
        gap: f32,
        align_items: AlignItems,
        justify_content: JustifyContent,
    ) -> Gc<Component> {
        Component::new(
            id,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: format!("{:?}", direction),
                gap,
                align_items: format!("{:?}", align_items),
                justify_content: format!("{:?}", justify_content),
            },
        )
    }

    /// Create a new Flex component with default values
    pub fn default(id: ComponentId) -> Gc<Component> {
        Self::new(
            id,
            FlexDirection::Row,
            0.0,
            AlignItems::Stretch,
            JustifyContent::Start,
        )
    }
}
