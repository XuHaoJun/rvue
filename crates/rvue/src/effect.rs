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
    closure_layout: std::alloc::Layout,
    is_dirty: AtomicBool,
    is_running: AtomicBool, // Prevent recursive execution
    owner: GcCell<Option<Gc<Component>>>,
    cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
}

unsafe impl Trace for Effect {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Trace known Gc fields
        self.owner.trace(visitor);

        // Conservatively scan the main closure's captured environment
        let closure_ptr: *const dyn Fn() = &*self.closure;
        let (data_ptr, vtable_ptr) =
            unsafe { std::mem::transmute::<*const dyn Fn(), (*const u8, *const ())>(closure_ptr) };

        // Use a minimum scan size to ensure we capture any Gc pointers.
        // The actual closure capture might be optimized to ZST by the compiler,
        // but the fat pointer itself (data + vtable) is always at least 16 bytes.
        let min_scan_size = std::mem::size_of::<usize>() * 2;
        let size = std::cmp::max(self.closure_layout.size(), min_scan_size);

        // Safety: Only scan if it looks like a valid heap pointer and size is reasonable
        if !data_ptr.is_null()
            && (data_ptr as usize).is_multiple_of(8)
            && (data_ptr as usize) >= 0x10000
            && !vtable_ptr.is_null()
            && size > 0
            && size < 1024 * 1024
        {
            unsafe {
                visitor.visit_region(data_ptr, size);
            }
        }

        // Also scan cleanup closures if any
        let cleanups_borrow = self.cleanups.borrow();
        for cleanup in cleanups_borrow.iter() {
            let func_ptr: *const dyn FnOnce() = &**cleanup;
            let (data_ptr, vtable_ptr) = unsafe {
                std::mem::transmute::<*const dyn FnOnce(), (*const u8, *const ())>(func_ptr)
            };
            let layout = std::alloc::Layout::for_value(&**cleanup);
            let size = std::cmp::max(layout.size(), min_scan_size);

            if !data_ptr.is_null()
                && (data_ptr as usize).is_multiple_of(8)
                && (data_ptr as usize) >= 0x10000
                && !vtable_ptr.is_null()
                && size > 0
                && size < 1024 * 1024
            {
                unsafe {
                    visitor.visit_region(data_ptr, size);
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
