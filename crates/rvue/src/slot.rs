//! Slot mechanism for passing content to components
//!
//! Slots allow components to accept and render content from their parents.
//! This follows the pattern established by Leptos and Vue.

use crate::view::ViewStruct;
use rudo_gc::{Gc, Trace};

/// Wrapper for closures that ensures GC tracing works correctly.
///
/// This wrapper calls the closure during GC trace to trace any `Gc<T>`
/// values in the returned ViewStruct, which also traces any `Gc<T>`
/// captured in the closure's environment.
pub struct ChildrenClosure(pub Box<dyn Fn() -> ViewStruct>);

unsafe impl Trace for ChildrenClosure {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        let view = (self.0)();
        view.trace(visitor);
    }
}

/// A function that renders slot content, can be called multiple times
pub type ChildrenFn = Gc<ChildrenClosure>;

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
        (self.children.0)()
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
        self.children.trace(visitor);
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
        Gc::new(ChildrenClosure(Box::new(self)))
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
    fn to_children(self) -> ChildrenFn {
        let view = self;
        Gc::new(ChildrenClosure(Box::new(move || view.clone())))
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
        let slot = Slot::new((|| ViewStruct::new(create_test_component())).to_children());

        let view = slot.render();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_to_children_fn() {
        let closure: ChildrenFn = (|| ViewStruct::new(create_test_component())).to_children();

        let view = (closure.0)();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_maybe_slot() {
        let none_slot: MaybeSlot = MaybeSlot::default();
        assert!(none_slot.is_none());
        assert!(none_slot.render().is_none());

        let some_slot = MaybeSlot::from(Some(Slot::new(
            (|| ViewStruct::new(create_test_component())).to_children(),
        )));

        assert!(some_slot.is_some());
        let view = some_slot.render().unwrap();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_slot_with_gc_capture() {
        let gc_component = create_test_component();
        let gc_clone = gc_component.clone();

        let closure: ChildrenFn =
            Gc::new(ChildrenClosure(Box::new(move || ViewStruct::new(gc_clone.clone()))));

        let view = (closure.0)();
        assert!(view.root_component.id == 0);
    }
}
