//! Slot mechanism for passing content to components
//!
//! Slots allow components to accept and render content from their parents.
//! This follows the pattern established by Leptos and Vue.
//!
//! ## Types
//!
//! - `Children` - One-time content, consumed on render (`FnOnce`)
//! - `ChildrenFn` - Reusable content, can be called multiple times (`Fn`)
//! - `SlotProps` - Trait for slot properties (auto-derived by #[slot] macro)
//!
//! ## Slot Props
//!
//! Slot structs can have additional fields marked with `#[prop(...)]`:
//! - `#[prop(into)]` - Call `.into()` on the value (handled by user)
//! - `#[prop(optional)]` - Field is wrapped in `Option<T>`

use crate::component::Component;
use crate::view::ViewStruct;
use rudo_gc::{Gc, Trace};

/// Marker trait for slot properties
///
/// This trait is automatically implemented by the `#[slot]` macro
/// for slot structs with prop fields. It enables proper GC tracing
/// of slot prop values.
pub trait SlotProps: Trace + Clone + 'static {}

pub(crate) struct LazyView {
    view: Option<Gc<ViewStruct>>,
}

impl LazyView {
    fn new(closure: Box<dyn Fn() -> ViewStruct>) -> Self {
        let view = Gc::new(closure());
        Self { view: Some(view) }
    }

    fn run(&self) -> ViewStruct {
        self.view.as_ref().unwrap().as_ref().clone()
    }
}

unsafe impl Trace for LazyView {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        if let Some(view) = &self.view {
            view.trace(visitor);
        }
    }
}

/// A function that renders slot content, can be called multiple times.
///
/// This uses `Gc` for shared ownership. Use `.run()` to render.
#[derive(Clone)]
pub struct ChildrenFn(pub(crate) Gc<LazyView>);

impl ChildrenFn {
    /// Render the slot content and return the view.
    pub fn run(&self) -> ViewStruct {
        self.0.run()
    }

    /// Returns true if both ChildrenFn point to the same underlying closure.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(&self.0, &other.0)
    }
}

unsafe impl Trace for ChildrenFn {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.0.trace(visitor);
    }
}

/// A function that renders slot content, can only be called once.
///
/// This consumes the closure on render (FnOnce semantics).
pub struct Children(pub Box<dyn FnOnce() -> ViewStruct>);

impl Children {
    /// Render the slot content and return the view.
    ///
    /// This consumes the closure - can only be called once.
    pub fn run(self) -> ViewStruct {
        (self.0)()
    }
}

impl<F> From<Box<F>> for Children
where
    F: FnOnce() -> ViewStruct + 'static,
{
    fn from(value: Box<F>) -> Self {
        Children(value)
    }
}

unsafe impl Trace for Children {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

/// Wrapper for optional children content.
#[derive(Clone, Default)]
pub struct MaybeChildren(pub Option<ChildrenFn>);

impl MaybeChildren {
    /// Render the children if they exist
    pub fn render(&self) -> Option<ViewStruct> {
        self.0.as_ref().map(|children| children.run())
    }

    /// Render the children, consuming self
    pub fn run(self) -> Option<ViewStruct> {
        self.0.map(|children| children.run())
    }

    /// Check if children exist
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    /// Check if children are empty
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl From<Option<ChildrenFn>> for MaybeChildren {
    fn from(value: Option<ChildrenFn>) -> Self {
        MaybeChildren(value)
    }
}

impl From<ChildrenFn> for MaybeChildren {
    fn from(value: ChildrenFn) -> Self {
        MaybeChildren(Some(value))
    }
}

/// Trait for converting values into children types
pub trait ToChildren<T> {
    fn to_children(self) -> T;
}

impl<F> ToChildren<ChildrenFn> for F
where
    F: Fn() -> ViewStruct + 'static,
{
    fn to_children(self) -> ChildrenFn {
        ChildrenFn(Gc::new(LazyView::new(Box::new(self))))
    }
}

impl<F> ToChildren<Children> for F
where
    F: FnOnce() -> ViewStruct + 'static,
{
    fn to_children(self) -> Children {
        Children(Box::new(self))
    }
}

impl ToChildren<ChildrenFn> for ViewStruct {
    /// Note: This creates a reusable closure that clones the ViewStruct on each call.
    /// While ChildrenFn supports multiple calls, this conversion may be inefficient
    /// for large view trees. Consider using Children (FnOnce) for one-time content.
    fn to_children(self) -> ChildrenFn {
        let view = self;
        ChildrenFn(Gc::new(LazyView::new(Box::new(move || view.clone()))))
    }
}

impl ToChildren<Children> for ViewStruct {
    fn to_children(self) -> Children {
        Children(Box::new(move || self.clone()))
    }
}

impl ToChildren<ChildrenFn> for MaybeChildren {
    fn to_children(self) -> ChildrenFn {
        let inner = self.0;
        ChildrenFn(Gc::new(LazyView::new(Box::new(move || {
            inner.as_ref().map_or_else(
                || {
                    ViewStruct::new(Component::new(
                        0,
                        crate::component::ComponentType::Text,
                        crate::component::ComponentProps::Text {
                            content: String::new(),
                            font_size: None,
                            color: None,
                        },
                    ))
                },
                |c| c.run(),
            )
        }))))
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
    fn test_children_fn_run() {
        let children: ChildrenFn = (|| ViewStruct::new(create_test_component())).to_children();

        let view = children.run();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_children_fn_multiple_calls() {
        let counter = std::rc::Rc::new(std::cell::RefCell::new(0));
        let counter_clone = counter.clone();
        let children: ChildrenFn = (move || {
            *counter_clone.borrow_mut() += 1;
            ViewStruct::new(create_test_component())
        })
        .to_children();

        let _view1 = children.run();
        let _view2 = children.run();
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_children_run() {
        let called = std::rc::Rc::new(std::cell::RefCell::new(false));
        let called_clone = called.clone();
        let children: Children = Children(Box::new(move || {
            *called_clone.borrow_mut() = true;
            ViewStruct::new(create_test_component())
        }));

        let view = children.run();
        assert!(*called.borrow());
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_maybe_children() {
        let none: MaybeChildren = MaybeChildren::default();
        assert!(none.is_none());
        assert!(none.render().is_none());

        let closure: ChildrenFn = (|| ViewStruct::new(create_test_component())).to_children();
        let some = MaybeChildren::from(closure);
        assert!(some.is_some());
        let view = some.run().unwrap();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_to_children_traits() {
        let from_closure: ChildrenFn = (|| ViewStruct::new(create_test_component())).to_children();
        assert!(from_closure.run().root_component.id == 0);

        let from_view: ChildrenFn = ViewStruct::new(create_test_component()).to_children();
        assert!(from_view.run().root_component.id == 0);
    }
}
