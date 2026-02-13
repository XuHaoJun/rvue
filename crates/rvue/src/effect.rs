//! Reactive effect implementation for automatic dependency tracking

use crate::component::Component;
use rudo_gc::handles::HandleScope;
use rudo_gc::heap::current_thread_control_block;
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

// Thread-local storage for tracking the currently running effect
thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Gc<Effect>>> = const { RefCell::new(None) };
    static EFFECTS_PENDING_RUN: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
    static DEFER_EFFECT_RUN: RefCell<bool> = const { RefCell::new(false) };
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
    let tcb = current_thread_control_block().expect("GC not initialized");
    let scope = HandleScope::new(&tcb);

    let effect = Effect::new(closure);
    let handle = scope.handle(&effect);
    let escaped_gc = handle.to_gc();

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

/// Effect structure for reactive computations
pub struct Effect {
    closure: Box<dyn Fn() + 'static>,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    owner: GcCell<Option<Gc<Component>>>,
    cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
    subscriptions: GcCell<Vec<(usize, Weak<Effect>)>>, // (signal_ptr, weak_ref) pairs
}

unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
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
        let boxed = Box::new(closure);
        Gc::new(Self {
            closure: boxed,
            is_dirty: AtomicBool::new(true),
            is_running: AtomicBool::new(false),
            owner: GcCell::new(owner),
            cleanups: GcCell::new(Vec::new()),
            subscriptions: GcCell::new(Vec::new()),
        })
    }

    /// Register a signal that this effect is subscribed to
    pub fn add_subscription(&self, signal_ptr: usize, weak_effect: &Weak<Effect>) {
        let mut subscriptions = self.subscriptions.borrow_mut_gen_only();

        // Avoid duplicates
        if !subscriptions.iter().any(|(ptr, w)| *ptr == signal_ptr && Weak::ptr_eq(w, weak_effect))
        {
            subscriptions.push((signal_ptr, weak_effect.clone()));
        }
    }

    /// Run the effect closure with automatic dependency tracking
    /// This is an associated function that requires a Gc reference
    pub fn run(gc_effect: &Gc<Self>) {
        log::debug!("Effect::run: starting effect");

        // Prevent recursive execution
        if gc_effect.is_running.swap(true, Ordering::SeqCst) {
            log::debug!("Effect::run: already running, skipping");
            // Already running, skip to prevent infinite loop
            return;
        }

        // Properly unsubscribe from all signals BEFORE clearing internal subscriptions
        // This ensures the signal's subscriber list is cleaned up (Leptos-style)
        {
            let subscriptions: Vec<(usize, Weak<Effect>)> = {
                let mut subs = gc_effect.subscriptions.borrow_mut_gen_only();
                std::mem::take(&mut subs)
            };
            for (signal_ptr, weak_effect) in subscriptions {
                let signal_ptr_void = signal_ptr as *const ();
                crate::signal::SignalDataInner::<()>::unsubscribe_by_ptr(
                    signal_ptr_void,
                    &weak_effect,
                );
            }
        }

        // Run cleanups from previous run
        let cleanups = {
            let mut cleanups = gc_effect.cleanups.borrow_mut_gen_only();
            std::mem::take(&mut *cleanups)
        };
        for cleanup in cleanups {
            cleanup();
        }

        // Mark as clean before running
        gc_effect.is_dirty.store(false, Ordering::SeqCst);

        // Set this effect as the current effect in thread-local storage
        let effect_ptr = Gc::as_ptr(gc_effect) as *const ();
        log::debug!("Effect::run: START effect ptr={:?}", effect_ptr);

        let previous = CURRENT_EFFECT.with(|cell| {
            let prev = cell.borrow().clone();
            *cell.borrow_mut() = Some(Gc::clone(gc_effect));
            prev
        });

        log::debug!("Effect::run: Setting current effect, previous = {:?}", previous.is_some());

        // Execute the closure (this may trigger signal.get() calls which will
        // automatically register this effect as a subscriber)
        let owner_opt = {
            let owner_ref = gc_effect.owner.borrow();
            owner_ref.clone()
        };

        if let Some(owner) = owner_opt {
            crate::runtime::with_owner(owner, || {
                (gc_effect.closure)();
            });
        } else {
            (gc_effect.closure)();
        }

        // Restore previous effect (if any)
        CURRENT_EFFECT.with(|cell| {
            *cell.borrow_mut() = previous;
        });

        // Mark as not running
        gc_effect.is_running.store(false, Ordering::SeqCst);
        log::debug!("Effect::run: completed effect");
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
        let subscriptions: Vec<(usize, Weak<Effect>)> = {
            let mut subs = self.subscriptions.borrow_mut_gen_only();
            std::mem::take(&mut subs)
        };

        for (signal_ptr, weak_effect) in subscriptions {
            let signal_ptr_void = signal_ptr as *const ();
            crate::signal::SignalDataInner::<()>::unsubscribe_by_ptr(signal_ptr_void, &weak_effect);
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

/// Register a cleanup function for the current reactive scope (Effect or Component)
pub fn on_cleanup<F>(f: F)
where
    F: FnOnce() + 'static,
{
    // Try Effect first
    if let Some(effect) = current_effect() {
        effect.cleanups.borrow_mut_gen_only().push(Box::new(f));
        return;
    }

    // Try Component
    if let Some(owner) = crate::runtime::current_owner() {
        owner.cleanups.borrow_mut_gen_only().push(Box::new(f));
    }
}
