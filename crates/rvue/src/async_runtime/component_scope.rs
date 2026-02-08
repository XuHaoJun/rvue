//! Dynamic tracking of GC-managed components for async operations.
//!
//! This module provides `ComponentScope` for tracking multiple components
//! that can be accessed safely from async contexts using rudo-gc's GcScope.

use std::cell::RefCell;
use std::rc::Rc;

use rudo_gc::handles::GcScope;
use rudo_gc::Gc;

use crate::component::Component;

#[derive(Default)]
pub struct ComponentScope {
    scope: Rc<RefCell<GcScope>>,
}

impl Clone for ComponentScope {
    fn clone(&self) -> Self {
        Self { scope: Rc::clone(&self.scope) }
    }
}

impl ComponentScope {
    #[inline]
    pub fn new() -> Self {
        Self { scope: Rc::new(RefCell::new(GcScope::new())) }
    }

    #[inline]
    pub fn track(&mut self, component: &Gc<Component>) {
        self.scope.borrow_mut().track(component);
    }

    #[inline]
    pub fn track_all(&mut self, components: &[Gc<Component>]) {
        self.scope.borrow_mut().track_slice(components);
    }

    #[inline]
    pub fn extend(&mut self, other: &ComponentScope) {
        self.scope.borrow_mut().track_from(&other.scope.borrow());
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.scope.borrow().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scope.borrow().is_empty()
    }
}

pub trait TrackComponents {
    fn track_components(&self, _scope: &mut ComponentScope) {}
}

impl TrackComponents for Vec<Gc<Component>> {
    fn track_components(&self, scope: &mut ComponentScope) {
        scope.track_all(self);
    }
}

impl TrackComponents for Option<Gc<Component>> {
    fn track_components(&self, scope: &mut ComponentScope) {
        if let Some(ref comp) = self {
            scope.track(comp);
        }
    }
}
