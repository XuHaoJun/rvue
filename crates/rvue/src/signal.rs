//! Reactive signal implementation for fine-grained reactivity

use crate::effect::{current_effect, Effect};
use rudo_gc::{Gc, GcCell, Trace, Weak};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

thread_local! {
    static LEAKED_EFFECTS: RefCell<Vec<Gc<Effect>>> = const { RefCell::new(Vec::new()) };
}

/// Internal signal data structure
pub struct SignalData<T: Trace + Clone + 'static> {
    pub(crate) value: GcCell<T>,
    pub(crate) version: AtomicU64,
    pub(crate) subscribers: GcCell<Vec<Weak<Effect>>>,
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for SignalData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalData").field("version", &self.version).finish()
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for SignalData<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        // Note: subscribers uses Weak<Effect>, which doesn't need tracing
        // Weak references don't keep objects alive, so GC handles them automatically
        // AtomicU64 is not GC-managed, so we don't trace it
    }
}

/// Read handle for a signal
pub struct ReadSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

impl<T: Trace + Clone + 'static> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data)) // Avoid requiring T: Debug
            .field("version", &self.data.version.load(Ordering::Relaxed))
            .finish()
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for ReadSignal<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.data.trace(visitor);
    }
}

/// Write handle for a signal
pub struct WriteSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

impl<T: Trace + Clone + 'static> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data)) // Avoid requiring T: Debug
            .field("version", &self.data.version.load(Ordering::Relaxed))
            .finish()
    }
}

/// Trait for reading signal values
pub trait SignalRead<T: Trace + Clone + 'static> {
    /// Read the signal value and subscribe the current effect
    fn get(&self) -> T;
    /// Read the signal value without subscribing the current effect
    fn get_untracked(&self) -> T;
}

/// Trait for writing signal values
pub trait SignalWrite<T: Trace + Clone + 'static> {
    fn set(&self, value: T);
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T);
}

impl<T: Trace + Clone + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        // Automatically register current effect as a subscriber if one is running
        if let Some(effect) = current_effect() {
            self.data.subscribe(effect);
        }
        self.data.value.borrow().clone()
    }

    fn get_untracked(&self) -> T {
        self.data.value.borrow().clone()
    }
}

impl<T: Trace + Clone + 'static> SignalWrite<T> for WriteSignal<T> {
    fn set(&self, value: T) {
        *self.data.value.borrow_mut() = value;
        self.data.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut *self.data.value.borrow_mut());
        self.data.version.fetch_add(1, Ordering::SeqCst);
        self.data.notify_subscribers();
    }
}

impl<T: Trace + Clone + 'static> SignalData<T> {
    /// Register an effect as a subscriber to this signal
    pub(crate) fn subscribe(&self, effect: Gc<Effect>) {
        let weak_effect = Gc::downgrade(&effect);
        let effect_ptr = Gc::as_ptr(&effect) as *const ();
        let signal_ptr = self as *const _ as *const ();

        // Check if already subscribed first (using immutable borrow to avoid deadlock)
        let already_subscribed = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().any(|sub| {
                sub.upgrade().map(|e| (Gc::as_ptr(&e) as *const ()) == effect_ptr).unwrap_or(false)
            })
        };

        // Only add if not already subscribed
        if !already_subscribed {
            let mut subscribers = self.subscribers.borrow_mut();
            // Double-check after acquiring mutable borrow (in case it was added between checks)
            if !subscribers.iter().any(|sub| {
                sub.upgrade().map(|e| (Gc::as_ptr(&e) as *const ()) == effect_ptr).unwrap_or(false)
            }) {
                subscribers.push(weak_effect.clone());

                // Also register this subscription in the effect using raw pointer
                effect.add_subscription(signal_ptr, &weak_effect);
            }
        }
    }

    /// Notify all subscriber effects that this signal has changed
    pub(crate) fn notify_subscribers(&self) {
        // Collect effects to update (upgrade Weak refs and filter dead ones)
        let effects_to_update: Vec<Gc<Effect>> = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().filter_map(|weak| weak.upgrade()).collect()
        };

        // Mark all effects as dirty first (no borrow needed)
        for effect in effects_to_update.iter() {
            effect.mark_dirty();
        }

        // Release the borrow before running effects to avoid deadlock
        // Effects may call signal.get() which tries to subscribe, but since
        // we've already released the borrow, this is safe

        // Then update them (use the collected list, don't re-collect to avoid
        // running newly added effects that weren't dirty)
        // Note: We iterate over the collected list to avoid running effects
        // that were added during execution (they'll be notified on next update)
        // The Effect::run() method already has protection against recursive execution
        for effect in effects_to_update.iter() {
            // Only run if still dirty (may have been run by another signal update)
            if effect.is_dirty() {
                Effect::update_if_dirty(effect);
            }
        }
    }

    /// Remove an effect from the subscribers list using a raw pointer
    #[allow(dead_code)]
    pub(crate) fn unsubscribe_by_ptr(effect_ptr: *const (), weak_effect: &Weak<Effect>) {
        // Find the signal by pointer and remove the specific weak reference
        // This is called during effect cleanup
        unsafe {
            // The signal_ptr points to the SignalData, cast it back
            let signal = &*effect_ptr.cast::<SignalData<()>>();
            let mut subscribers = signal.subscribers.borrow_mut();
            subscribers.retain(|weak| !Weak::ptr_eq(weak, weak_effect));
        }
    }
}

/// Create a new signal with an initial value
pub fn create_signal<T: Trace + Clone + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let data = Gc::new(SignalData {
        value: GcCell::new(initial_value),
        version: AtomicU64::new(0),
        subscribers: GcCell::new(Vec::new()),
    });
    (ReadSignal { data: Gc::clone(&data) }, WriteSignal { data })
}

/// Create a new memo that automatically updates when its dependencies change
///
/// A memo is a derived signal that only recomputes its value when one of its
/// reactive dependencies changes. This is useful for caching expensive
/// computations.
pub fn create_memo<T: Trace + Clone + 'static, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
{
    let initial_value = crate::effect::untracked(&f);
    let (read, write) = create_signal(initial_value.clone());

    let f_shared = std::rc::Rc::new(f);
    let f_clone = f_shared.clone();

    let is_first = std::cell::Cell::new(true);
    let effect = crate::effect::create_effect(move || {
        let value = f_clone();
        if is_first.replace(false) {
            // First run: just set the initial value (already done by create_signal)
            // But we need to run f_clone() to track dependencies
        } else {
            // Subsequent runs: update the value
            write.set(value);
        }
    });

    // Store the effect in thread-local storage to keep it alive.
    LEAKED_EFFECTS.with(|cell| {
        let mut leaked = cell.borrow_mut();
        leaked.push(effect);
    });

    read
}

/// Create a new memo with a custom equality check to avoid redundant updates
pub fn create_memo_with_equality<T: Trace + Clone + PartialEq + 'static, F>(f: F) -> ReadSignal<T>
where
    F: Fn() -> T + 'static,
{
    let initial_value = crate::effect::untracked(&f);
    let (read, write) = create_signal(initial_value.clone());

    let last_value = GcCell::new(initial_value);
    let f_shared = std::rc::Rc::new(f);
    let f_clone = f_shared.clone();

    let is_first = std::cell::Cell::new(true);
    let effect = crate::effect::create_effect(move || {
        let new_value = f_clone();
        if is_first.replace(false) {
            // First run: just track dependencies
        } else if new_value != *last_value.borrow() {
            *last_value.borrow_mut() = new_value.clone();
            write.set(new_value);
        }
    });

    // Store the effect in thread-local storage to keep it alive.
    LEAKED_EFFECTS.with(|cell| {
        let mut leaked = cell.borrow_mut();
        leaked.push(effect);
    });

    read
}
