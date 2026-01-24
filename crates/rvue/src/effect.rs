//! Reactive effect implementation for automatic dependency tracking

use crate::component::Component;
use rudo_gc::{Gc, GcCell, Trace};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

// Thread-local storage for tracking the currently running effect
thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Gc<Effect>>> = const { RefCell::new(None) };
}

/// Effect structure for reactive computations
pub struct Effect {
    closure: Box<dyn Fn() + 'static>,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    owner: GcCell<Option<Gc<Component>>>,
    cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
}

// Effect contains a closure which is not Trace, but we can still make Effect Trace
// by not tracing the closure (it's not GC-managed)
unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Closure is not GC-managed, so we don't trace it
        // Cleanups are also not traced since they are closures
        self.owner.trace(visitor);
    }
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect")
            .field("is_dirty", &self.is_dirty)
            .field("is_running", &self.is_running)
            .finish()
    }
}

impl Effect {
    /// Create a new effect with a closure
    pub fn new<F>(closure: F) -> Gc<Self>
    where
        F: Fn() + 'static,
    {
        let owner = crate::runtime::current_owner();
        Gc::new(Self {
            closure: Box::new(closure),
            is_dirty: AtomicBool::new(true),
            is_running: AtomicBool::new(false),
            owner: GcCell::new(owner),
            cleanups: GcCell::new(Vec::new()),
        })
    }

    /// Run the effect closure with automatic dependency tracking
    /// This is an associated function that requires a Gc reference
    pub fn run(gc_effect: &Gc<Self>) {
        // Prevent recursive execution
        if gc_effect.is_running.swap(true, Ordering::SeqCst) {
            // Already running, skip to prevent infinite loop
            return;
        }

        // Run cleanups from previous run
        let cleanups = {
            let mut cleanups = gc_effect.cleanups.borrow_mut();
            std::mem::take(&mut *cleanups)
        };
        for cleanup in cleanups {
            cleanup();
        }

        // Mark as clean before running
        gc_effect.is_dirty.store(false, Ordering::SeqCst);

        // Set this effect as the current effect in thread-local storage
        CURRENT_EFFECT.with(|cell| {
            let previous = cell.borrow().clone();
            *cell.borrow_mut() = Some(Gc::clone(gc_effect));

            // Execute the closure (this may trigger signal.get() calls which will
            // automatically register this effect as a subscriber)
            if let Some(owner) = gc_effect.owner.borrow().as_ref() {
                crate::runtime::with_owner(Gc::clone(owner), || {
                    (gc_effect.closure)();
                });
            } else {
                (gc_effect.closure)();
            }

            // Restore previous effect (if any)
            *cell.borrow_mut() = previous;
        });

        // Mark as not running
        gc_effect.is_running.store(false, Ordering::SeqCst);
    }

    /// Mark the effect as dirty (needs to run)
    pub fn mark_dirty(&self) {
        self.is_dirty.store(true, Ordering::SeqCst);
    }

    /// Check if the effect is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(Ordering::SeqCst)
    }

    /// Manually trigger the effect to run if dirty
    pub fn update_if_dirty(gc_effect: &Gc<Self>) {
        if gc_effect.is_dirty() {
            Self::run(gc_effect);
        }
    }
}

/// Get the currently running effect (if any)
/// This is used by signals to automatically register dependencies
pub(crate) fn current_effect() -> Option<Gc<Effect>> {
    CURRENT_EFFECT.with(|cell| cell.borrow().clone())
}

/// Create a new effect that automatically runs when dependencies change
///
/// The effect runs immediately on creation, and automatically tracks
/// which signals are accessed during execution. When any tracked signal
/// changes, the effect will be marked dirty and can be re-run.
pub fn create_effect<F>(closure: F) -> Gc<Effect>
where
    F: Fn() + 'static,
{
    let effect = Effect::new(closure);
    Effect::run(&effect);
    effect
}

/// Run a closure without tracking any dependencies
///
/// This is useful for reading signals in effects or memos without
/// creating a dependency on them.
pub fn untracked<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    CURRENT_EFFECT.with(|cell| {
        let previous = cell.borrow_mut().take();
        let result = f();
        *cell.borrow_mut() = previous;
        result
    })
}

/// Register a cleanup function for the current reactive scope (Effect or Component)
pub fn on_cleanup<F>(f: F)
where
    F: FnOnce() + 'static,
{
    // Try Effect first
    if let Some(effect) = current_effect() {
        effect.cleanups.borrow_mut().push(Box::new(f));
        return;
    }

    // Try Component
    if let Some(owner) = crate::runtime::current_owner() {
        owner.cleanups.borrow_mut().push(Box::new(f));
    }
}
