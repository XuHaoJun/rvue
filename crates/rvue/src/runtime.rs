use crate::component::Component;
use rudo_gc::Gc;
use std::cell::RefCell;

thread_local! {
    /// Stack of components currently being built or executing effects.
    /// This is used to provide context to child components and effects.
    static OWNER_STACK: RefCell<Vec<Gc<Component>>> = const { RefCell::new(Vec::new()) };
}

/// Execute a closure with a specific component as the current owner.
pub fn with_owner<R>(owner: Gc<Component>, f: impl FnOnce() -> R) -> R {
    OWNER_STACK.with(|stack| {
        stack.borrow_mut().push(owner);
        let result = f();
        stack.borrow_mut().pop();
        result
    })
}

/// Get the current owner (the top component on the owner stack).
pub fn current_owner() -> Option<Gc<Component>> {
    OWNER_STACK.with(|stack| stack.borrow().last().cloned())
}

/// Execute a closure within a component scope.
/// This is used by the #[component] macro and BuildContext.
pub fn with_component_scope<R>(component: Gc<Component>, f: impl FnOnce() -> R) -> R {
    with_owner(component, f)
}
