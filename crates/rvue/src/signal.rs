//! Reactive signal implementation for fine-grained reactivity

use crate::effect::{current_effect, Effect};
use rudo_gc::{Gc, GcCell, Trace};
use std::sync::atomic::{AtomicU64, Ordering};

/// Internal signal data structure
pub struct SignalData<T: Trace + Clone + 'static> {
    pub(crate) value: GcCell<T>,
    pub(crate) version: AtomicU64,
    pub(crate) subscribers: GcCell<Vec<Gc<Effect>>>,
}

impl<T: Trace + Clone + 'static> std::fmt::Debug for SignalData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalData").field("version", &self.version).finish()
    }
}

unsafe impl<T: Trace + Clone + 'static> Trace for SignalData<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.subscribers.trace(visitor);
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
        // Check if already subscribed first (using immutable borrow to avoid deadlock)
        let already_subscribed = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().any(|sub| Gc::ptr_eq(sub, &effect))
        };

        // Only add if not already subscribed
        if !already_subscribed {
            let mut subscribers = self.subscribers.borrow_mut();
            // Double-check after acquiring mutable borrow (in case it was added between checks)
            if !subscribers.iter().any(|sub| Gc::ptr_eq(sub, &effect)) {
                subscribers.push(effect);
            }
        }
    }

    /// Notify all subscriber effects that this signal has changed
    pub(crate) fn notify_subscribers(&self) {
        // Collect effects to update (clone to avoid borrow conflicts)
        let effects_to_update: Vec<Gc<Effect>> = {
            let subscribers = self.subscribers.borrow();
            subscribers.iter().map(Gc::clone).collect()
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
    // Initialize with a dummy or uninitialized state is hard in Rust without Option
    // But we can use f() once and then use an effect.
    // To avoid calling f() twice (once for create_signal and once for initial effect run),
    // we can use a manual effect construction or untracked read.

    let (read, write) = create_signal(crate::effect::untracked(&f));

    // Create an effect that updates the signal when dependencies change.
    // We use a SkipFirst runner or just let it run.
    // If we use create_effect, it runs immediately.
    let f_shared = std::rc::Rc::new(f);
    let f_clone = f_shared.clone();

    // We want the effect to run on changes, but we already have the initial value.
    // However, create_effect RUNS once to track dependencies.
    // So if we want to avoid double-calling f() at the very start:
    crate::effect::create_effect(move || {
        write.set(f_clone());
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

    crate::effect::create_effect(move || {
        let new_value = f();
        if new_value != *last_value.borrow() {
            *last_value.borrow_mut() = new_value.clone();
            write.set(new_value);
        }
    });

    read
}
