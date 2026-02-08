//! GC Safety Tests for Async Runtime
//!
//! These tests verify the GC safety patterns for async operations.

#[cfg(test)]
mod gc_safety_tests {
    use rudo_gc::Trace;

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    unsafe impl Trace for TestData {
        fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
    }

    #[test]
    fn test_gc_safety_extraction_pattern() {
        use rudo_gc::{Gc, Trace};

        #[derive(Debug, Clone, Trace)]
        struct GcData {
            value: i32,
        }

        let gc_data = Gc::new(GcData { value: 42 });
        let value = gc_data.value;

        assert_eq!(value, 42);
    }
}

#[cfg(feature = "async")]
#[cfg(test)]
mod async_gc_safety_tests {
    use rvue::signal::create_signal;

    #[tokio::test]
    async fn test_signal_sender_works() {
        use rvue::prelude::WriteSignalExt;

        let (signal, setter) = create_signal(42i32);
        let sender = setter.sender();

        sender.set(100).await;
        assert_eq!(signal.get(), 100);

        setter.set(50);
        assert_eq!(signal.get(), 50);
    }
}
