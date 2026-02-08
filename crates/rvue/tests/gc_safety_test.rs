//! GC Safety Tests for Async Runtime
//!
//! These tests verify the GC safety patterns for async operations.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[cfg(test)]
mod gc_safety_tests {
    use rudo_gc::{Gc, Trace};

    use crate::signal::create_signal;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    impl Trace for TestData {}

    #[test]
    fn test_gc_safety_extraction_pattern() {
        #[derive(Debug, Clone, Trace)]
        struct GcData {
            value: i32,
        }

        let gc_data = Gc::new(GcData { value: 42 });
        let value = gc_data.value.clone();

        assert_eq!(value, 42);
    }

    #[test]
    fn test_signal_sender_error() {
        use crate::prelude::WriteSignalExt;

        let (signal, _setter) = create_signal(42i32);
        let result = signal.sender();

        assert!(result.is_err());
    }
}
