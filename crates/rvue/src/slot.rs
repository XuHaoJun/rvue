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
use crate::widget::BuildContext;
use rudo_gc::{Gc, Trace};

/// Marker trait for slot properties
///
/// This trait is automatically implemented by the `#[slot]` macro
/// for slot structs with prop fields. It enables proper GC tracing
/// of slot prop values.
pub trait SlotProps: Trace + Clone + 'static {}

/// Internal lazy view structure.
///
/// SAFETY: Closures stored here must NOT capture any GC-tracked types
/// (Gc, GcCell, ReadSignal, WriteSignal, etc.). The closure can only
/// capture non-GC types like primitives, Strings, or Arc. If a closure
/// needs to reference GC-tracked data, that data must be stored separately
/// (e.g., via GcCell) and the closure must not capture it directly.
///
/// Violating this constraint will cause use-after-free or memory corruption
/// during garbage collection because the Trace impl cannot trace dyn Fn captures.
pub(crate) struct LazyView {
    closure: Box<dyn Fn(&mut BuildContext) -> ViewStruct>,
}

impl LazyView {
    fn new(closure: Box<dyn Fn(&mut BuildContext) -> ViewStruct>) -> Self {
        Self { closure }
    }
}

/// SAFETY: LazyView requires closure to not capture GC-tracked types.
/// See struct-level documentation for safety requirements.
unsafe impl Trace for LazyView {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // Cannot trace dyn Fn captures - caller must ensure no GC types are captured
    }
}

/// A function that renders slot content, can be called multiple times.
///
/// This uses `Gc` for shared ownership. Use `.run()` to render.
#[derive(Clone)]
pub struct ChildrenFn(pub(crate) Gc<LazyView>);

impl ChildrenFn {
    /// Render the slot content and return the view.
    pub fn run(&self, ctx: &mut BuildContext) -> ViewStruct {
        (self.0.closure)(ctx)
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
pub struct Children(pub Box<dyn FnOnce(&mut BuildContext) -> ViewStruct>);

impl Children {
    /// Render the slot content and return the view.
    ///
    /// This consumes the closure - can only be called once.
    pub fn run(self, ctx: &mut BuildContext) -> ViewStruct {
        (self.0)(ctx)
    }
}

impl<F> From<Box<F>> for Children
where
    F: FnOnce(&mut BuildContext) -> ViewStruct + 'static,
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
    pub fn render(&self, ctx: &mut BuildContext) -> Option<ViewStruct> {
        self.0.as_ref().map(|children| children.run(ctx))
    }

    /// Render the children, consuming self
    pub fn run(self, ctx: &mut BuildContext) -> Option<ViewStruct> {
        self.0.map(|children| children.run(ctx))
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
    F: Fn(&mut BuildContext) -> ViewStruct + 'static,
{
    fn to_children(self) -> ChildrenFn {
        ChildrenFn(Gc::new(LazyView::new(Box::new(self))))
    }
}

impl<F> ToChildren<Children> for F
where
    F: FnOnce(&mut BuildContext) -> ViewStruct + 'static,
{
    fn to_children(self) -> Children {
        Children(Box::new(self))
    }
}

impl ToChildren<ChildrenFn> for ViewStruct {
    fn to_children(self) -> ChildrenFn {
        let view = self;
        ChildrenFn(Gc::new(LazyView::new(Box::new(move |_ctx: &mut BuildContext| view.clone()))))
    }
}

impl ToChildren<Children> for ViewStruct {
    fn to_children(self) -> Children {
        let view = self;
        Children(Box::new(move |_ctx: &mut BuildContext| view.clone()))
    }
}

impl ToChildren<ChildrenFn> for MaybeChildren {
    fn to_children(self) -> ChildrenFn {
        ChildrenFn(Gc::new(LazyView::new(Box::new(|_ctx: &mut BuildContext| {
            ViewStruct::new(Component::new(
                0,
                crate::component::ComponentType::Text,
                crate::component::ComponentProps::Text {
                    content: String::new(),
                    font_size: None,
                    styles: None,
                },
            ))
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
            ComponentProps::Text { content: "test".to_string(), font_size: None, styles: None },
        )
    }

    #[test]
    fn test_children_fn_run() {
        let mut taffy = taffy::TaffyTree::new();
        let mut text_context = crate::text::TextContext::new();
        let mut id_counter: u64 = 0;
        let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

        let children: ChildrenFn =
            (|_ctx: &mut BuildContext| ViewStruct::new(create_test_component())).to_children();

        let view = children.run(&mut ctx);
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_children_fn_multiple_calls() {
        let counter = std::rc::Rc::new(std::cell::RefCell::new(0));
        let counter_clone = counter.clone();

        let mut taffy = taffy::TaffyTree::new();
        let mut text_context = crate::text::TextContext::new();
        let mut id_counter: u64 = 0;
        let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

        let children: ChildrenFn = (move |_ctx: &mut BuildContext| {
            *counter_clone.borrow_mut() += 1;
            ViewStruct::new(create_test_component())
        })
        .to_children();

        let _view1 = children.run(&mut ctx);
        let _view2 = children.run(&mut ctx);
        assert_eq!(*counter.borrow(), 2);
    }

    #[test]
    fn test_children_run() {
        let called = std::rc::Rc::new(std::cell::RefCell::new(false));
        let called_clone = called.clone();

        let mut taffy = taffy::TaffyTree::new();
        let mut text_context = crate::text::TextContext::new();
        let mut id_counter: u64 = 0;
        let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

        let children = Children(Box::new(move |_ctx: &mut BuildContext| {
            *called_clone.borrow_mut() = true;
            ViewStruct::new(create_test_component())
        }));

        let view = children.run(&mut ctx);
        assert!(*called.borrow());
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_maybe_children() {
        let none: MaybeChildren = MaybeChildren::default();
        assert!(none.is_none());

        let mut taffy = taffy::TaffyTree::new();
        let mut text_context = crate::text::TextContext::new();
        let mut id_counter: u64 = 0;
        let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

        assert!(none.render(&mut ctx).is_none());

        let closure: ChildrenFn =
            (|_ctx: &mut BuildContext| ViewStruct::new(create_test_component())).to_children();
        let some = MaybeChildren::from(closure);
        assert!(some.is_some());
        let view = some.run(&mut ctx).unwrap();
        assert!(view.root_component.id == 0);
    }

    #[test]
    fn test_to_children_traits() {
        let mut taffy = taffy::TaffyTree::new();
        let mut text_context = crate::text::TextContext::new();
        let mut id_counter: u64 = 0;
        let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

        let from_closure: ChildrenFn =
            (|_ctx: &mut BuildContext| ViewStruct::new(create_test_component())).to_children();
        assert!(from_closure.run(&mut ctx).root_component.id == 0);

        let from_view: ChildrenFn = ViewStruct::new(create_test_component()).to_children();
        assert!(from_view.run(&mut ctx).root_component.id == 0);
    }
}
