//! Effect system for reactive computations
//!
//! This module provides the `Effect` type and related utilities for
//! automatic dependency tracking and reactive updates.

use crate::component::Component;
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    pub(crate) static CURRENT_EFFECT: RefCell<Option<Weak<Effect>>> = const { RefCell::new(None) };
    static DEFER_EFFECT_RUN: RefCell<bool> = const { RefCell::new(false) };
    static EFFECTS_PENDING_RUN: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
}

/// Create a new effect that runs a closure and automatically tracks its dependencies
pub fn create_effect<F>(f: F) -> Gc<Effect>
where
    F: Fn() + 'static,
{
    let escaped_gc = Effect::new(f);

    DEFER_EFFECT_RUN.with(|defer| {
        if *defer.borrow() {
            EFFECTS_PENDING_RUN.with(|pending| {
                pending.borrow_mut().push(Gc::clone(&escaped_gc));
            });
        } else {
            Effect::run(&escaped_gc);
        }
    });
    escaped_gc
}

/// Flush any pending effects that were deferred during layout tree building
pub fn flush_pending_effects() {
    let pending = EFFECTS_PENDING_RUN.with(|p| std::mem::take(&mut *p.borrow_mut()));
    for effect in pending {
        Effect::run(&effect);
    }
}

/// Set whether to defer effect execution (used during layout tree building)
pub fn set_defer_effect_run(defer: bool) {
    DEFER_EFFECT_RUN.with(|d| *d.borrow_mut() = defer);
}

/// Register a cleanup function for the current effect or component
///
/// When running inside an effect, registers with that effect (runs when effect re-runs).
/// When running inside a component scope (with_owner) but no effect, registers with the
/// component (runs when component unmounts).
pub fn on_cleanup<F: FnOnce() + 'static>(cleanup: F) {
    if let Some(effect) = current_effect() {
        effect.cleanups.borrow_mut_gen_only().push(Box::new(cleanup));
    } else if let Some(owner) = crate::runtime::current_owner() {
        owner.cleanups.borrow_mut_gen_only().push(Box::new(cleanup));
    }
}

/// Effect structure for reactive computations
pub struct Effect {
    closure: Box<dyn Fn() + 'static>,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    is_valid: AtomicBool,   // False when effect is being cleaned up / unsubscribed
    owner: GcCell<Option<Gc<Component>>>,
    pub(crate) cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
    pub(crate) subscriptions: GcCell<Vec<(usize, Weak<()>, Weak<()>)>>, // (signal_ptr, signal_weak, effect_weak)
}

unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.owner.trace(visitor);
        // Note: cleanups and subscriptions are not traced because:
        // 1. Weak refs in subscriptions don't need marking
        // 2. cleanups are Boxed closures which aren't easily traceable
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
        let effect = Gc::new(Self {
            closure: Box::new(closure),
            is_dirty: AtomicBool::new(false),
            is_running: AtomicBool::new(false),
            is_valid: AtomicBool::new(true),
            owner: GcCell::new(owner.clone()),
            cleanups: GcCell::new(Vec::new()),
            subscriptions: GcCell::new(Vec::new()),
        });

        if let Some(owner) = owner {
            owner.add_effect(Gc::clone(&effect));
        } else {
            // Global effect - keep it alive via global root
            crate::signal::leak_effect(Gc::clone(&effect));
        }

        effect
    }

    /// Register a signal that this effect is subscribed to
    pub fn add_subscription(
        &self,
        signal_ptr: usize,
        signal_weak: &Weak<()>,
        weak_opaque: &Weak<()>,
    ) {
        let mut subscriptions = self.subscriptions.borrow_mut_gen_only();

        // Avoid duplicates
        if !subscriptions.iter().any(|(ptr, s, w)| {
            *ptr == signal_ptr && Weak::ptr_eq(s, signal_weak) && Weak::ptr_eq(w, weak_opaque)
        }) {
            subscriptions.push((signal_ptr, signal_weak.clone(), weak_opaque.clone()));
        }
    }

    /// Check if this effect is still valid (not being cleaned up)
    pub fn is_valid(&self) -> bool {
        self.is_valid.load(Ordering::SeqCst)
    }

    /// Run the effect closure with automatic dependency tracking
    /// This is an associated function that requires a Gc reference
    pub fn run(gc_effect: &Gc<Self>) {
        log::debug!("Effect::run: starting effect");

        // Prevent recursive execution
        if gc_effect.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        // Step 1: Run cleanups
        let cleanups = {
            let mut cleanups = gc_effect.cleanups.borrow_mut_gen_only();
            std::mem::take(&mut *cleanups)
        };
        for cleanup in cleanups {
            cleanup();
        }

        // Step 2: Drop old subscription records.
        // We intentionally avoid raw-pointer signal mutation here because stale
        // pointers can be reclaimed by GC; signals self-clean dead weak refs on notify.
        {
            let mut subs = gc_effect.subscriptions.borrow_mut_gen_only();
            subs.clear();
        }

        // Step 3: Run the closure - is_running prevents recursive execution
        gc_effect.is_dirty.store(false, Ordering::SeqCst);

        let previous = CURRENT_EFFECT.with(|cell| {
            let prev = (*cell.borrow()).clone();
            *cell.borrow_mut() = Some(Gc::downgrade(gc_effect));
            prev
        });

        (gc_effect.closure)();

        CURRENT_EFFECT.with(|cell| {
            *cell.borrow_mut() = previous;
        });

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
    ///
    /// This properly removes the weak ref from each signal's subscriber list.
    fn unsubscribe_all(&self) {
        self.subscriptions.borrow_mut_gen_only().clear();
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        self.is_valid.store(false, Ordering::SeqCst);
        self.unsubscribe_all();
    }
}

/// Get the currently running effect (if any)
/// This is used by signals to automatically register dependencies
pub(crate) fn current_effect() -> Option<Gc<Effect>> {
    CURRENT_EFFECT.with(|cell| cell.borrow().as_ref().and_then(|w| w.try_upgrade()))
}

/// Run a closure without tracking any dependencies
///
/// This is useful for reading signals in effects or memos without
/// creating a dependency on them.
pub fn untracked<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let previous = CURRENT_EFFECT.with(|cell| cell.borrow_mut().take());
    let result = f();
    CURRENT_EFFECT.with(|cell| {
        *cell.borrow_mut() = previous;
    });
    result
}
