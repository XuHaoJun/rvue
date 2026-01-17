//! Reactive signal implementation for fine-grained reactivity

use rudo_gc::{Gc, GcCell, Trace};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::effect::{Effect, current_effect};

/// Internal signal data structure
pub struct SignalData<T: Trace + Clone + 'static> {
    value: GcCell<T>,
    version: AtomicU64,
    subscribers: GcCell<Vec<Gc<Effect>>>,
}

unsafe impl<T: Trace + Clone + 'static> Trace for SignalData<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
        self.subscribers.trace(visitor);
        // AtomicU64 is not GC-managed, so we don't trace it
    }
}

/// Read handle for a signal
#[derive(Clone)]
pub struct ReadSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

/// Write handle for a signal
#[derive(Clone)]
pub struct WriteSignal<T: Trace + Clone + 'static> {
    data: Gc<SignalData<T>>,
}

/// Trait for reading signal values
pub trait SignalRead<T: Trace + Clone + 'static> {
    fn get(&self) -> T;
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
            subscribers.iter().map(|e| Gc::clone(e)).collect()
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
pub fn create_signal<T: Trace + Clone + 'static>(initial_value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let data = Gc::new(SignalData {
        value: GcCell::new(initial_value),
        version: AtomicU64::new(0),
        subscribers: GcCell::new(Vec::new()),
    });
    (
        ReadSignal { data: Gc::clone(&data) },
        WriteSignal { data },
    )
}
