//! Reactive effect implementation for automatic dependency tracking

use crate::component::Component;
use crate::signal::SignalData;
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

// Thread-local storage for tracking the currently running effect
thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Gc<Effect>>> = const { RefCell::new(None) };
}

/// Effect structure for reactive computations
pub struct Effect {
    closure: Box<dyn Fn() + 'static>,
    closure_layout: std::alloc::Layout,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    owner: GcCell<Option<Gc<Component>>>,
    cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
    subscriptions: GcCell<Vec<(*const (), Weak<Effect>)>>, // (signal_ptr, weak_ref)
}

unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Trace known Gc fields
        self.owner.trace(visitor);
        // subscriptions contains raw pointers, not traced

        // Temporarily disable conservative scanning of closures to prevent stack overflow
        // in deep component trees.
        // TODO: Implement precise tracing of closure captures using `TraceClosure` trait
        // or iterate over tracked signals instead of scanning memory.

        // Also scan cleanup closures if any
        let cleanups_borrow = self.cleanups.borrow();
        for cleanup in cleanups_borrow.iter() {
            // Cast FnOnce to Fn for scanning (safe since we only read memory)
            // SAFETY: We cast &dyn FnOnce to &dyn Fn, which is valid for reading.
            // We scan the closure's data for pointers, we don't call it.
            let func_ptr: *const dyn Fn() = unsafe { std::mem::transmute(cleanup.as_ref()) };
            let data_ptr = func_ptr as *const u8;
            let layout = std::alloc::Layout::for_value(&**cleanup);

            if layout.size() > 0 && layout.align() >= std::mem::align_of::<usize>() {
                unsafe {
                    visitor.visit_region(data_ptr, layout.size());
                }
            }
        }
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
        let boxed = Box::new(closure);
        let closure_layout = std::alloc::Layout::for_value(&*boxed);
        Gc::new(Self {
            closure: boxed,
            closure_layout,
            is_dirty: AtomicBool::new(true),
            is_running: AtomicBool::new(false),
            owner: GcCell::new(owner),
            cleanups: GcCell::new(Vec::new()),
            subscriptions: GcCell::new(Vec::new()),
        })
    }

    /// Register a signal that this effect is subscribed to
    pub fn add_subscription(&self, signal_ptr: *const (), weak_effect: &Weak<Effect>) {
        let mut subscriptions = self.subscriptions.borrow_mut();
        let pair = (signal_ptr, weak_effect.clone());
        if !subscriptions
            .iter()
            .any(|(ptr, weak)| *ptr == signal_ptr && Weak::ptr_eq(weak, weak_effect))
        {
            subscriptions.push(pair);
        }
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

    /// Unsubscribe from all signals this effect is subscribed to
    fn unsubscribe_all(&self) {
        let subscriptions = std::mem::take(&mut *self.subscriptions.borrow_mut());
        for (signal_ptr, weak_effect) in subscriptions {
            SignalData::<()>::unsubscribe_by_ptr(signal_ptr, &weak_effect);
        }
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.unsubscribe_all();
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
