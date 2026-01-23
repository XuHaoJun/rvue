//! Radio widget component

use crate::component::{Component, ComponentId, ComponentProps, ComponentType};
use crate::effect::create_effect;
use crate::widget::{BuildContext, Mountable, ReactiveValue, Widget};
use rudo_gc::{Gc, Trace};

/// Radio widget builder for single selection from multiple options
#[derive(Clone)]
pub struct RadioWidget {
    value: String,
    checked: ReactiveValue<bool>,
}

unsafe impl Trace for RadioWidget {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.checked.trace(visitor);
    }
}

impl RadioWidget {
    /// Create a new Radio widget with a value and checked state
    pub fn new(value: String, checked: impl crate::widget::IntoReactiveValue<bool>) -> Self {
        Self { value, checked: checked.into_reactive() }
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

impl Widget for RadioWidget {
    type State = RadioState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let id = ctx.next_id();
        let initial_checked = self.checked.get();

        let component = Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value: self.value.clone(), checked: initial_checked },
        );

        // Setup reactive update if checked is reactive
        let checked_effect = if self.checked.is_reactive() {
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
        // Update checked if it changed
        if self.checked.is_reactive() {
            // Checked is reactive, effect will handle updates
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
            // Static checked - update directly
            let new_checked = self.checked.get();
            state.component.set_radio_checked(new_checked);
        }
    }
}

// Keep old API for backward compatibility
#[deprecated(note = "Use RadioWidget::new() instead")]
pub struct Radio;

#[allow(deprecated)]
impl Radio {
    /// Create a new Radio component with a static value and checked state
    #[deprecated(note = "Use RadioWidget::new() instead")]
    pub fn new(id: ComponentId, value: String, checked: bool) -> Gc<Component> {
        Component::new(id, ComponentType::Radio, ComponentProps::Radio { value, checked })
    }

    /// Create a new Radio component with a reactive checked signal
    #[deprecated(note = "Use RadioWidget::new() instead")]
    pub fn from_signal<T: crate::signal::SignalRead<String> + Clone + 'static>(
        id: ComponentId,
        value: String,
        checked_signal: T,
    ) -> Gc<Component> {
        use crate::signal::SignalRead;
        // Check if this radio's value matches the signal value
        let initial_signal_value = checked_signal.get();
        let initial_checked = initial_signal_value == value;

        let component = Component::new(
            id,
            ComponentType::Radio,
            ComponentProps::Radio { value: value.clone(), checked: initial_checked },
        );

        // Setup reactive update
        let comp = Gc::clone(&component);
        let sig = checked_signal.clone();
        let radio_value = value;
        let effect = create_effect(move || {
            use crate::signal::SignalRead;
            let signal_value = sig.get();
            let checked = signal_value == radio_value;
            *comp.props.borrow_mut() =
                ComponentProps::Radio { value: radio_value.clone(), checked };
            comp.mark_dirty();
        });

        component.add_effect(effect);
        component
    }
}
