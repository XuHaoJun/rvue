//! Core signal types for Rvue's fine-grained reactivity system.
//!
//! This crate provides the foundational signal types used by both `rvue` and `rvue-style`.
//! It handles the core value storage and versioning without subscriber tracking,
//! which is handled by the parent crates.

use rudo_gc::{Gc, GcThreadSafeCell, Trace};
use std::sync::atomic::{AtomicU64, Ordering};

/// Internal signal data structure containing the value and version tracking.
///
/// This is the core storage type shared between rvue and rvue-style.
/// Version tracking allows consumers to detect when values change.
///
/// Design: Uses GcThreadSafeCell for thread-safe cross-thread access.
/// This allows mutations from tokio worker threads without needing
/// to dispatch back to the main thread.
pub struct SignalData<T: Clone + 'static> {
    /// The stored value (protected by GcThreadSafeCell for thread-safe access)
    pub value: GcThreadSafeCell<T>,
    /// Monotonically increasing version counter
    pub version: AtomicU64,
}

impl<T: Clone + Trace + 'static> SignalData<T> {
    /// Create a new signal data structure
    pub fn new(value: T) -> Self {
        Self { value: GcThreadSafeCell::new(value), version: AtomicU64::new(0) }
    }

    /// Get the current value
    #[inline(always)]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.borrow().clone()
    }

    /// Set a new value and increment version
    #[inline(always)]
    pub fn set(&self, value: T) {
        let _old = {
            let mut guard = self.value.borrow_mut_simple();
            std::mem::replace(&mut *guard, value)
        };
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    /// Modify the value and increment version
    #[inline(always)]
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let _old = {
            let mut guard = self.value.borrow_mut_simple();
            let old = (*guard).clone();
            f(&mut *guard);
            old
        };
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    /// Get the current version
    #[inline(always)]
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }
}

impl<T: Clone + 'static> std::fmt::Debug for SignalData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalData")
            .field("version", &self.version.load(Ordering::Relaxed))
            .finish()
    }
}

unsafe impl<T: Clone + Trace + 'static> Trace for SignalData<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
    }
}

/// Read handle for a signal.
///
/// Cloneable reference to a signal's value.
#[derive(Clone)]
pub struct ReadSignal<T: Clone + Trace + 'static> {
    data: Gc<SignalData<T>>,
}

impl<T: Clone + Trace + 'static> ReadSignal<T> {
    /// Get the current value
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.data.get()
    }

    /// Get the current value without any effect tracking
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.data.get()
    }

    /// Get a reference to the inner data (for advanced use cases)
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Gc<SignalData<T>> {
        &self.data
    }
}

impl<T: Clone + Trace + 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data))
            .field("version", &self.data.version.load(Ordering::Relaxed))
            .finish()
    }
}

unsafe impl<T: Clone + Trace + 'static> Trace for ReadSignal<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Trace the Gc<SignalData<T>> field to ensure the signal data is marked as reachable
        self.data.trace(visitor);
    }
}

/// Write handle for a signal.
///
/// Allows modifying the signal's value and triggers version increment.
#[derive(Clone)]
pub struct WriteSignal<T: Clone + Trace + 'static> {
    data: Gc<SignalData<T>>,
}

impl<T: Clone + Trace + 'static> WriteSignal<T> {
    /// Set a new value
    pub fn set(&self, value: T) {
        self.data.set(value);
    }

    /// Modify the value with a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.data.update(f);
    }

    /// Get a reference to the inner data (for advanced use cases)
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Gc<SignalData<T>> {
        &self.data
    }
}

impl<T: Clone + Trace + 'static> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSignal")
            .field("data_ptr", &Gc::as_ptr(&self.data))
            .field("version", &self.data.version.load(Ordering::Relaxed))
            .finish()
    }
}

unsafe impl<T: Clone + Trace + 'static> Trace for WriteSignal<T> {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Trace the Gc<SignalData<T>> field to ensure the signal data is marked as reachable
        self.data.trace(visitor);
    }
}

/// Trait for reading signal values.
///
/// Provides common interface for reading from signal handles.
pub trait SignalRead<T: Clone + Trace + 'static> {
    /// Read the signal value
    fn get(&self) -> T;
    /// Read the signal value without effect tracking
    fn get_untracked(&self) -> T;
}

/// Trait for writing signal values.
///
/// Provides common interface for writing to signal handles.
pub trait SignalWrite<T: Clone + Trace + 'static> {
    /// Set a new value
    fn set(&self, value: T);
    /// Modify the value with a function
    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T);
}

impl<T: Clone + Trace + 'static> SignalRead<T> for ReadSignal<T> {
    fn get(&self) -> T {
        self.data.get()
    }

    fn get_untracked(&self) -> T {
        self.data.get()
    }
}

impl<T: Clone + Trace + 'static> SignalWrite<T> for WriteSignal<T> {
    fn set(&self, value: T) {
        self.data.set(value)
    }

    fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.data.update(f);
    }
}

/// Create a new signal with an initial value.
///
/// Returns a tuple of (read_handle, write_handle).
pub fn create_signal<T: Clone + Trace + 'static>(
    initial_value: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let data = Gc::new(SignalData::new(initial_value));
    (ReadSignal { data: Gc::clone(&data) }, WriteSignal { data })
}
