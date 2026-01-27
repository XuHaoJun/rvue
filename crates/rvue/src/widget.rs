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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use taffy::TaffyTree;

type CtxMap = Arc<Mutex<HashMap<std::thread::ThreadId, usize>>>;

pub(crate) static CURRENT_CTX: std::sync::LazyLock<CtxMap, fn() -> CtxMap> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[doc(hidden)]
pub fn with_current_ctx<F, R>(ctx: &mut u64, f: F) -> R
where
    F: FnOnce() -> R,
{
    let thread_id = std::thread::current().id();
    let ctx_addr = ctx as *mut u64 as usize;

    CURRENT_CTX.lock().unwrap().insert(thread_id, ctx_addr);
    let result = f();
    CURRENT_CTX.lock().unwrap().remove(&thread_id);
    result
}

#[doc(hidden)]
pub fn get_current_ctx() -> Option<*mut u64> {
    let thread_id = std::thread::current().id();
    CURRENT_CTX.lock().unwrap().get(&thread_id).map(|&addr| addr as *mut u64)
}

/// Reactive value that can be either static or derived from a signal
///
/// This enum allows widgets to accept both static values and reactive
/// signals, enabling fine-grained updates when signals change.
pub enum ReactiveValue<T: Trace + Clone + 'static> {
    /// Static value that never changes
    Static(T),
    /// Value derived from a signal
    Signal(ReadSignal<T>),
}

impl<T: Trace + Clone + 'static> Clone for ReactiveValue<T> {
    fn clone(&self) -> Self {
        match self {
            ReactiveValue::Static(value) => ReactiveValue::Static(value.clone()),
            ReactiveValue::Signal(signal) => ReactiveValue::Signal(signal.clone()),
        }
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for ReactiveValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReactiveValue::Static(_) => f.debug_tuple("Static").field(&"<value>").finish(),
            ReactiveValue::Signal(sig) => f.debug_tuple("Signal").field(sig).finish(),
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
        }
    }

    /// Check if this value is reactive (signal)
    pub fn is_reactive(&self) -> bool {
        matches!(self, ReactiveValue::Signal(_))
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

impl crate::widget::IntoReactiveValue<crate::style::JustifyContent>
    for crate::style::JustifyContent
{
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

    /// Create a new component with a unique ID
    pub fn create_component(
        &mut self,
        component_type: crate::component::ComponentType,
        props: crate::component::ComponentProps,
    ) -> Gc<Component> {
        Component::with_global_id(component_type, props)
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
