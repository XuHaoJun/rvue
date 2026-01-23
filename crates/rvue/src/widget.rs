//! Widget trait system for fine-grained reactive updates
//!
//! This module provides the core traits for building widgets that support
//! compile-time reactivity optimization, following patterns from Leptos
//! and Svelte.

use crate::component::Component;
use crate::effect::Effect;
use crate::signal::ReadSignal;
use crate::text::TextContext;
use rudo_gc::{Gc, Trace};
use taffy::TaffyTree;

/// Reactive value that can be either static or derived from a signal
///
/// This enum allows widgets to accept both static values and reactive
/// signals, enabling fine-grained updates when signals change.
pub enum ReactiveValue<T: Trace + Clone + 'static> {
    /// Static value that never changes
    Static(T),
    /// Value derived from a signal
    Signal(ReadSignal<T>),
    /// Value derived from a closure (for computed values)
    /// Note: This variant is not Clone, so cloning ReactiveValue with Derived
    /// will panic. Use Static or Signal for cloneable reactive values.
    Derived(Box<dyn Fn() -> T + 'static>),
}

impl<T: Trace + Clone + 'static> Clone for ReactiveValue<T> {
    fn clone(&self) -> Self {
        match self {
            ReactiveValue::Static(value) => ReactiveValue::Static(value.clone()),
            ReactiveValue::Signal(signal) => ReactiveValue::Signal(signal.clone()),
            ReactiveValue::Derived(_) => {
                panic!("Cannot clone ReactiveValue::Derived - use Static or Signal instead")
            }
        }
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for ReactiveValue<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        match self {
            ReactiveValue::Static(_) => {
                // Static values don't contain GC pointers
            }
            ReactiveValue::Signal(signal) => {
                signal.trace(visitor);
            }
            ReactiveValue::Derived(_) => {
                // Closures may capture GC pointers, but we can't trace them
                // This is a limitation - derived values should be used carefully
            }
        }
    }
}

impl<T: Trace + Clone + 'static> ReactiveValue<T> {
    /// Get the current value
    pub fn get(&self) -> T {
        match self {
            ReactiveValue::Static(value) => value.clone(),
            ReactiveValue::Signal(signal) => {
                use crate::signal::SignalRead;
                SignalRead::get(signal)
            }
            ReactiveValue::Derived(f) => f(),
        }
    }

    /// Check if this value is reactive (signal or derived)
    pub fn is_reactive(&self) -> bool {
        matches!(self, ReactiveValue::Signal(_) | ReactiveValue::Derived(_))
    }
}

/// Trait for converting values into reactive values
///
/// Similar to Leptos's `IntoReactiveValue`, this allows flexible
/// property assignment in widgets.
pub trait IntoReactiveValue<T: Trace + Clone + 'static> {
    /// Convert this value into a reactive value
    fn into_reactive(self) -> ReactiveValue<T>;
}

// Implement for signals
impl<T: Trace + Clone + 'static> IntoReactiveValue<T> for ReadSignal<T> {
    fn into_reactive(self) -> ReactiveValue<T> {
        ReactiveValue::Signal(self)
    }
}

// Helper function for static values (avoids trait conflict)
pub fn static_value<T: Trace + Clone + 'static>(value: T) -> ReactiveValue<T> {
    ReactiveValue::Static(value)
}

// Helper function for closures
pub fn derived_value<T: Trace + Clone + 'static, F: Fn() -> T + 'static>(
    f: F,
) -> ReactiveValue<T> {
    ReactiveValue::Derived(Box::new(f))
}

// Implement for common primitive types to avoid needing explicit conversion
macro_rules! impl_into_reactive_for {
    ($($t:ty),*) => {
        $(
            impl IntoReactiveValue<$t> for $t {
                fn into_reactive(self) -> ReactiveValue<$t> {
                    ReactiveValue::Static(self)
                }
            }
        )*
    };
}

impl_into_reactive_for!(String, bool, i32, i64, u32, u64, f32, f64, usize);

// Implement for &str -> String
impl IntoReactiveValue<String> for &str {
    fn into_reactive(self) -> ReactiveValue<String> {
        ReactiveValue::Static(self.to_string())
    }
}

// Implement for style types
impl crate::widget::IntoReactiveValue<crate::style::FlexDirection> for crate::style::FlexDirection {
    fn into_reactive(self) -> ReactiveValue<crate::style::FlexDirection> {
        ReactiveValue::Static(self)
    }
}

impl crate::widget::IntoReactiveValue<crate::style::AlignItems> for crate::style::AlignItems {
    fn into_reactive(self) -> ReactiveValue<crate::style::AlignItems> {
        ReactiveValue::Static(self)
    }
}

impl crate::widget::IntoReactiveValue<crate::style::JustifyContent> for crate::style::JustifyContent {
    fn into_reactive(self) -> ReactiveValue<crate::style::JustifyContent> {
        ReactiveValue::Static(self)
    }
}

/// State that can be mounted in the UI tree
pub trait Mountable: Trace {
    /// Mount this state to the component tree
    fn mount(&self, parent: Option<Gc<Component>>);

    /// Unmount this state from the component tree
    fn unmount(&self);
}

/// Build context provided during widget construction
pub struct BuildContext<'a> {
    pub taffy: &'a mut TaffyTree<()>,
    pub text_context: &'a mut TextContext,
    pub id_counter: &'a mut u64,
}

impl<'a> BuildContext<'a> {
    /// Create a new build context
    pub fn new(
        taffy: &'a mut TaffyTree<()>,
        text_context: &'a mut TextContext,
        id_counter: &'a mut u64,
    ) -> Self {
        Self { taffy, text_context, id_counter }
    }

    /// Allocate a new component ID
    pub fn next_id(&mut self) -> crate::component::ComponentId {
        let id = *self.id_counter;
        *self.id_counter += 1;
        id
    }
}

/// Core widget trait for building and updating UI
///
/// This trait follows the pattern from Leptos's `Render` trait, enabling
/// fine-grained updates by separating initial build from updates.
pub trait Widget: Sized + Trace {
    /// The state type that holds the mounted widget instance
    type State: Mountable;

    /// Build initial widget state (called once during initial render)
    ///
    /// This creates the component and sets up initial reactive bindings.
    fn build(self, ctx: &mut BuildContext) -> Self::State;

    /// Update existing state with new values (fine-grained update)
    ///
    /// This is called when the widget's properties change, allowing
    /// only the changed properties to be updated without recreating
    /// the entire component.
    fn rebuild(self, state: &mut Self::State);
}

/// Conversion trait for types that can be converted into widgets
///
/// Similar to Leptos's `IntoView`, this allows flexible composition
/// of widgets and other types.
pub trait IntoWidget: Sized {
    /// The widget type this converts into
    type Widget: Widget;

    /// Convert this type into a widget
    fn into_widget(self) -> Self::Widget;
}

/// Wrapper for type-erased widgets
///
/// This allows storing different widget types in collections
/// while maintaining type safety during construction.
pub struct WidgetWrapper {
    component: Gc<Component>,
    effects: Vec<Gc<Effect>>,
}

unsafe impl Trace for WidgetWrapper {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.component.trace(visitor);
        for effect in &self.effects {
            effect.trace(visitor);
        }
    }
}

impl WidgetWrapper {
    /// Create a new widget wrapper
    pub fn new(component: Gc<Component>, effects: Vec<Gc<Effect>>) -> Self {
        Self { component, effects }
    }

    /// Get the underlying component
    pub fn component(&self) -> &Gc<Component> {
        &self.component
    }

    /// Get the effects associated with this widget
    pub fn effects(&self) -> &[Gc<Effect>] {
        &self.effects
    }
}

impl Mountable for WidgetWrapper {
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
