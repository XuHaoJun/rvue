//! Flex widget for flexbox layouts

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::style::{AlignItems, FlexDirection, JustifyContent};
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Flex widget builder for creating flexbox layouts
#[derive(Clone)]
pub struct FlexWidget {
    direction: ReactiveValue<FlexDirection>,
    gap: ReactiveValue<f32>,
    align_items: ReactiveValue<AlignItems>,
    justify_content: ReactiveValue<JustifyContent>,
}

unsafe impl Trace for FlexWidget {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.direction.trace(visitor);
        self.gap.trace(visitor);
        self.align_items.trace(visitor);
        self.justify_content.trace(visitor);
    }
}

impl FlexWidget {
    /// Create a new Flex widget with default values
    pub fn new() -> Self {
        Self {
            direction: ReactiveValue::Static(FlexDirection::Row),
            gap: ReactiveValue::Static(0.0),
            align_items: ReactiveValue::Static(AlignItems::Stretch),
            justify_content: ReactiveValue::Static(JustifyContent::Start),
        }
    }

    /// Set the flex direction
    pub fn direction(mut self, direction: impl crate::widget::IntoReactiveValue<FlexDirection>) -> Self {
        self.direction = direction.into_reactive();
        self
    }

    /// Set the gap between items
    pub fn gap(mut self, gap: impl crate::widget::IntoReactiveValue<f32>) -> Self {
        self.gap = gap.into_reactive();
        self
    }

    /// Set the align items (cross-axis alignment)
    pub fn align_items(mut self, align_items: impl crate::widget::IntoReactiveValue<AlignItems>) -> Self {
        self.align_items = align_items.into_reactive();
        self
    }

    /// Set the justify content (main-axis alignment)
    pub fn justify_content(
        mut self,
        justify_content: impl crate::widget::IntoReactiveValue<JustifyContent>,
    ) -> Self {
        self.justify_content = justify_content.into_reactive();
        self
    }
}

impl Default for FlexWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// State for a mounted Flex widget
pub struct FlexState {
    component: Gc<Component>,
    direction_effect: Option<Gc<crate::effect::Effect>>,
    gap_effect: Option<Gc<crate::effect::Effect>>,
    align_items_effect: Option<Gc<crate::effect::Effect>>,
    justify_content_effect: Option<Gc<crate::effect::Effect>>,
}

impl FlexState {
    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }
}

unsafe impl Trace for FlexState {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        if let Some(effect) = &self.direction_effect {
            effect.trace(visitor);
        }
        if let Some(effect) = &self.gap_effect {
            effect.trace(visitor);
        }
        if let Some(effect) = &self.align_items_effect {
            effect.trace(visitor);
        }
        if let Some(effect) = &self.justify_content_effect {
            effect.trace(visitor);
        }
    }
}

impl Mountable for FlexState {
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

impl Widget for FlexWidget {
    type State = FlexState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let direction = self.direction.get();
        let gap = self.gap.get();
        let align_items = self.align_items.get();
        let justify_content = self.justify_content.get();

        let component = Component::new(
            id,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: format!("{:?}", direction),
                gap,
                align_items: format!("{:?}", align_items),
                justify_content: format!("{:?}", justify_content),
            },
        );

        // Setup reactive updates for each property
        let direction_effect = if self.direction.is_reactive() {
            let comp = Gc::clone(&component);
            let direction = self.direction.clone();
            let effect = crate::effect::create_effect(move || {
                let new_direction = direction.get();
                comp.set_flex_direction(format!("{:?}", new_direction));
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        let gap_effect = if self.gap.is_reactive() {
            let comp = Gc::clone(&component);
            let gap = self.gap.clone();
            let effect = crate::effect::create_effect(move || {
                let new_gap = gap.get();
                comp.set_flex_gap(new_gap);
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        let align_items_effect = if self.align_items.is_reactive() {
            let comp = Gc::clone(&component);
            let align_items = self.align_items.clone();
            let effect = crate::effect::create_effect(move || {
                let new_align_items = align_items.get();
                comp.set_flex_align_items(format!("{:?}", new_align_items));
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        let justify_content_effect = if self.justify_content.is_reactive() {
            let comp = Gc::clone(&component);
            let justify_content = self.justify_content.clone();
            let effect = crate::effect::create_effect(move || {
                let new_justify_content = justify_content.get();
                comp.set_flex_justify_content(format!("{:?}", new_justify_content));
            });
            component.add_effect(Gc::clone(&effect));
            Some(effect)
        } else {
            None
        };

        FlexState {
            component,
            direction_effect,
            gap_effect,
            align_items_effect,
            justify_content_effect,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        // Update direction if reactive
        if self.direction.is_reactive() {
            if state.direction_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let direction = self.direction.clone();
                let effect = crate::effect::create_effect(move || {
                    let new_direction = direction.get();
                    comp.set_flex_direction(format!("{:?}", new_direction));
                });
                state.component.add_effect(Gc::clone(&effect));
                state.direction_effect = Some(effect);
            }
        } else {
            let new_direction = self.direction.get();
            state.component.set_flex_direction(format!("{:?}", new_direction));
        }

        // Update gap if reactive
        if self.gap.is_reactive() {
            if state.gap_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let gap = self.gap.clone();
                let effect = crate::effect::create_effect(move || {
                    let new_gap = gap.get();
                    comp.set_flex_gap(new_gap);
                });
                state.component.add_effect(Gc::clone(&effect));
                state.gap_effect = Some(effect);
            }
        } else {
            let new_gap = self.gap.get();
            state.component.set_flex_gap(new_gap);
        }

        // Update align_items if reactive
        if self.align_items.is_reactive() {
            if state.align_items_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let align_items = self.align_items.clone();
                let effect = crate::effect::create_effect(move || {
                    let new_align_items = align_items.get();
                    comp.set_flex_align_items(format!("{:?}", new_align_items));
                });
                state.component.add_effect(Gc::clone(&effect));
                state.align_items_effect = Some(effect);
            }
        } else {
            let new_align_items = self.align_items.get();
            state.component.set_flex_align_items(format!("{:?}", new_align_items));
        }

        // Update justify_content if reactive
        if self.justify_content.is_reactive() {
            if state.justify_content_effect.is_none() {
                let comp = Gc::clone(&state.component);
                let justify_content = self.justify_content.clone();
                let effect = crate::effect::create_effect(move || {
                    let new_justify_content = justify_content.get();
                    comp.set_flex_justify_content(format!("{:?}", new_justify_content));
                });
                state.component.add_effect(Gc::clone(&effect));
                state.justify_content_effect = Some(effect);
            }
        } else {
            let new_justify_content = self.justify_content.get();
            state.component.set_flex_justify_content(format!("{:?}", new_justify_content));
        }
    }
}

// Keep old API for backward compatibility
#[deprecated(note = "Use FlexWidget::new() instead")]
pub struct Flex;

#[allow(deprecated)]
impl Flex {
    /// Create a new Flex component with direction, gap, alignment, and justification
    #[deprecated(note = "Use FlexWidget::new() instead")]
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
    #[deprecated(note = "Use FlexWidget::new() instead")]
    pub fn default(id: ComponentId) -> Gc<Component> {
        Self::new(id, FlexDirection::Row, 0.0, AlignItems::Stretch, JustifyContent::Start)
    }
}
