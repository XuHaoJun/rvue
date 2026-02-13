//! Reactive traits for Leptos-style subscription management
//!
//! This module defines the core traits for the reactive system.
//! Note: Due to rudo-gc limitations, we use concrete types for storage
//! but implement bidirectional cleanup logic similar to Leptos.

use rudo_gc::{Gc, Trace};

pub fn gc_ptr_eq<T: Trace + 'static>(a: &Gc<T>, b: &Gc<T>) -> bool {
    Gc::as_ptr(a) == Gc::as_ptr(b)
}
