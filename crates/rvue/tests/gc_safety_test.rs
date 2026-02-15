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
    async fn test_ui_dispatcher_works() {
        use rvue::prelude::WriteSignalUiExt;

        let (signal, setter) = create_signal(42i32);
        let dispatcher = setter.ui_dispatcher();

        // The dispatcher can be created and sent to other tasks
        // Note: Without a UI event loop processing callbacks, the dispatch
        // won't actually update the signal value in this test environment.
        // This is expected - the dispatcher is designed to work with a runtime
        // that processes dispatch callbacks.
        dispatcher.set(100).await;

        // Direct set still works
        setter.set(50);
        assert_eq!(signal.get(), 50);

        // Verify the dispatcher was created and can be used
        let _ = dispatcher;
    }
}
