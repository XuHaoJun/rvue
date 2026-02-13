//! Reactive effect implementation for automatic dependency tracking

use crate::component::Component;
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

// Thread-local storage for tracking the currently running effect
thread_local! {
    static CURRENT_EFFECT: RefCell<Option<Gc<Effect>>> = const { RefCell::new(None) };
    static EFFECTS_PENDING_RUN: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
    static DEFER_EFFECT_RUN: RefCell<bool> = const { RefCell::new(false) };
    static NOTIFY_DEPTH: RefCell<u32> = const { RefCell::new(0) };
    static DEFERRED_EFFECTS: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
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
    DEFER_EFFECT_RUN.with(|defer| {
        if *defer.borrow() {
            EFFECTS_PENDING_RUN.with(|pending| {
                pending.borrow_mut().push(Gc::clone(&effect));
            });
        } else {
            Effect::run(&effect);
        }
    });
    effect
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

/// Enter a notification scope (increments depth)
pub(crate) fn enter_notify() {
    NOTIFY_DEPTH.with(|d| *d.borrow_mut() += 1);
}

/// Exit a notification scope - just decrement depth
pub(crate) fn exit_notify() {
    NOTIFY_DEPTH.with(|d| {
        let new_depth = d.borrow().saturating_sub(1);
        *d.borrow_mut() = new_depth;
    });
}

/// Run all pending effects (called from event loop)
pub fn run_pending_effects() {
    let depth = NOTIFY_DEPTH.with(|d| *d.borrow());
    if depth > 0 {
        return;
    }

    let effects = DEFERRED_EFFECTS.with(|def| std::mem::take(&mut *def.borrow_mut()));
    for effect in effects {
        // Only run if dirty
        if !effect.is_dirty.load(Ordering::SeqCst) {
            continue;
        }
        Effect::run(&effect);
    }
}

/// Queue an effect to run after the current notification cycle completes
#[allow(dead_code)]
pub(crate) fn queue_effect(effect: Gc<Effect>) {
    // Don't queue if already running
    if effect.is_running.load(Ordering::SeqCst) {
        return;
    }
    DEFERRED_EFFECTS.with(|deferred| {
        deferred.borrow_mut().push(effect);
    });
}

/// Effect structure for reactive computations
pub struct Effect {
    closure: Box<dyn Fn() + 'static>,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    owner: GcCell<Option<Gc<Component>>>,
    cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
    sources: GcCell<Vec<(*const (), Weak<Effect>)>>, // (signal_ptr, weak_effect)
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
            sources: GcCell::new(Vec::new()),
        })
    }

    /// Register a source (signal) that this effect is subscribed to
    pub fn add_source(&self, signal_ptr: *const (), weak_effect: Weak<Effect>) {
        let mut sources = self.sources.borrow_mut_gen_only();

        let already_subscribed =
            sources.iter().any(|(ptr, w)| *ptr == signal_ptr && Weak::ptr_eq(w, &weak_effect));

        if !already_subscribed {
            sources.push((signal_ptr, weak_effect));
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

        // Clear sources (they'll be re-added during execution)
        // Signal cleanup handles invalid weak refs
        let _sources = std::mem::take(&mut *gc_effect.sources.borrow_mut_gen_only());

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
        CURRENT_EFFECT.with(|cell| {
            let previous = cell.borrow().clone();
            *cell.borrow_mut() = Some(Gc::clone(gc_effect));

            // Execute the closure (this may trigger signal.get() calls which will
            // automatically register this effect as a subscriber)
            let owner = gc_effect.owner.borrow();
            if owner.is_some() {
                crate::runtime::with_owner(Gc::clone(owner.as_ref().unwrap()), || {
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

    /// Unsubscribe from all sources this effect is subscribed to
    fn unsubscribe_all(&self) {
        let sources = std::mem::take(&mut *self.sources.borrow_mut_gen_only());

        for (signal_ptr, weak_effect) in sources {
            crate::signal::SignalDataInner::<()>::unsubscribe_by_ptr(signal_ptr, &weak_effect);
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
        effect.cleanups.borrow_mut_gen_only().push(Box::new(f));
        return;
    }

    // Try Component
    if let Some(owner) = crate::runtime::current_owner() {
        owner.cleanups.borrow_mut_gen_only().push(Box::new(f));
    }
}
