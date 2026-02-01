use crate::component::ContextValue;
use crate::runtime::current_owner;
use rudo_gc::{Gc, Trace};
use std::any::Any;

/// Provide a context value to the current component and its descendants.
/// Must be called during component setup.
pub fn provide_context<T: ContextValue + Trace + Clone>(value: T) {
    if let Some(owner) = current_owner() {
        owner.provide_context(value);
    } else {
        panic!("provide_context called outside of a component setup or effect");
    }
}

/// Injects a context value of type T from the current component or its ancestors.
pub fn inject<T: Any + Trace + Clone>() -> Option<Gc<T>> {
    current_owner().and_then(|owner| owner.find_context::<T>())
}

/// Injects a context value of type T, panicking if not found.
pub fn use_context<T: Any + Trace + Clone>() -> Gc<T> {
    inject::<T>().expect("context not found")
}

/// Alias for use_context, matches Leptos/Hooks naming.
pub fn expect_context<T: Any + Trace + Clone>() -> Gc<T> {
    use_context::<T>()
}
