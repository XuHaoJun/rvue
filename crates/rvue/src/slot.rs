//! Slot mechanism for passing content to components
//!
//! Slots allow components to accept and render content from their parents.
//! This follows the pattern established by Leptos and Vue.

use crate::view::ViewStruct;
use rudo_gc::Trace;
use std::sync::Arc;

/// A function that renders slot content, can be called multiple times
///
/// Note: ChildrenFn uses `Arc<dyn Fn()>` which is not `Send + Sync` by default.
/// This is acceptable because UI rendering in rvue happens on a single-threaded
/// event loop (winit). The closure will be invoked only on the main thread.
pub type ChildrenFn = Arc<dyn Fn() -> ViewStruct>;

/// A function that renders slot content, can only be called once
pub type Children = Box<dyn FnOnce() -> ViewStruct>;

/// A slot that can be passed to a component
///
/// Slots are not components themselves - they are containers for content
/// that gets rendered directly by the parent component.
#[derive(Clone)]
pub struct Slot {
    /// The function that renders this slot's content
    pub children: ChildrenFn,
}

impl Slot {
    /// Create a new slot with the given children function
    pub fn new(children: ChildrenFn) -> Self {
        Self { children }
    }

    /// Render the slot content and return the view
    pub fn render(&self) -> ViewStruct {
        (self.children)()
    }
}

impl From<Slot> for Vec<Slot> {
    fn from(value: Slot) -> Self {
        vec![value]
    }
}

impl From<ChildrenFn> for Slot {
    fn from(children: ChildrenFn) -> Self {
        Slot::new(children)
    }
}

unsafe impl Trace for Slot {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        let view = (self.children)();
        view.trace(visitor);
    }
}

/// Trait for converting values into children types
///
/// This allows flexible composition of children content in the view! macro.
pub trait ToChildren<T> {
    /// Convert the given value into children
    fn to_children(self) -> T;
}

impl<F> ToChildren<ChildrenFn> for F
where
    F: Fn() -> ViewStruct + 'static,
{
    fn to_children(self) -> ChildrenFn {
        Arc::new(self)
    }
}

impl<F> ToChildren<Children> for F
where
    F: FnOnce() -> ViewStruct + 'static,
{
    fn to_children(self) -> Children {
        Box::new(self)
    }
}

impl ToChildren<ChildrenFn> for ViewStruct {
    /// Converts a ViewStruct into a ChildrenFn by wrapping it in an Arc.
    ///
    /// Note: Uses `#[allow(clippy::arc_with_non_send_sync)]` because UI rendering
    /// happens on a single-threaded event loop (winit). If multi-threaded rendering
    /// is added in the future, this should be reviewed.
    #[allow(clippy::arc_with_non_send_sync)]
    fn to_children(self) -> ChildrenFn {
        Arc::new(move || self.clone())
    }
}

/// Wrapper for optional slot content
#[derive(Clone, Default)]
pub struct MaybeSlot(pub Option<Slot>);

impl From<Option<Slot>> for MaybeSlot {
    fn from(value: Option<Slot>) -> Self {
        MaybeSlot(value)
    }
}

impl MaybeSlot {
    /// Render the slot if it exists
    pub fn render(&self) -> Option<ViewStruct> {
        self.0.as_ref().map(|slot| slot.render())
    }

    /// Check if the slot has content
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the slot is empty
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use rudo_gc::Gc;

    fn create_test_component() -> Gc<Component> {
        Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text { content: "test".to_string(), font_size: None, color: None },
        )
    }

    #[test]
    fn test_slot_new() {
        let slot = Slot::new(Arc::new(|| ViewStruct::new(create_test_component())));

        let view = slot.render();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_to_children_fn() {
        let closure: ChildrenFn = (|| ViewStruct::new(create_test_component())).to_children();

        let view = closure();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_maybe_slot() {
        let none_slot: MaybeSlot = MaybeSlot::default();
        assert!(none_slot.is_none());
        assert!(none_slot.render().is_none());

        let some_slot =
            MaybeSlot::from(Some(Slot::new(Arc::new(|| ViewStruct::new(create_test_component())))));

        assert!(some_slot.is_some());
        let view = some_slot.render().unwrap();
        assert!(view.root_component.id == 0);
    }
}
