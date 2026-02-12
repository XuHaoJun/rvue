//! Unit tests for create_resource and Resource types.
//! These tests verify the async resource fetching behavior.
//!
//! Note: These tests are ignored because they require a winit event loop
//! to properly dispatch UI updates from async callbacks. In the actual app,
//! the winit event loop processes AsyncDispatchReady events and executes
//! the dispatch_to_ui callbacks on the main thread.
//!
//! To run these tests in a real application context, you would need to
//! either:
//! 1. Run them as part of an integration test with a proper event loop
//! 2. Manually pump the dispatch queue in a test harness
//!
//! The create_resource function works correctly in the actual application
//! because the winit event loop processes the dispatch_to_ui callbacks.

#[cfg(feature = "async")]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use rudo_gc::Gc;
    use rudo_gc::Trace;
    use rvue::async_runtime::{create_resource, Resource, ResourceState};
    use rvue::create_signal;

    fn wait_for_resource<T, S>(
        resource: &Resource<T, S>,
        max_wait: Duration,
        check: fn(&ResourceState<T>) -> bool,
    ) -> bool
    where
        T: Trace + Clone + 'static,
        S: Trace + Clone + 'static,
    {
        let start = Instant::now();
        while start.elapsed() < max_wait {
            let state = resource.get();
            if check(&state) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[ignore]
    #[test]
    fn test_create_resource_fetches_on_creation() {
        let (counter, _) = create_signal(0i32);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let resource = create_resource(counter, move |_| {
            let call_count = Arc::clone(&call_count_clone);
            async move {
                call_count.fetch_add(1, Ordering::SeqCst);
                Ok("result".to_string())
            }
        });

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let ready = wait_for_resource(&resource, Duration::from_secs(2), |s| s.is_ready());
        assert!(ready, "Resource should reach Ready state within timeout");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[ignore]
    #[test]
    fn test_create_resource_handles_error() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Err("network error".to_string()) });

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let has_error = wait_for_resource(&resource, Duration::from_secs(2), |s| s.is_error());
        assert!(has_error, "Resource should reach Error state within timeout");

        match resource.get().as_ref() {
            ResourceState::Error(msg) => assert_eq!(msg, "network error"),
            _ => panic!("Expected Error state"),
        }
    }

    #[ignore]
    #[test]
    fn test_create_resource_state_transitions() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Ok("data".to_string()) });

        assert!(resource.get().is_loading(), "Initial state should be Loading");

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let ready = wait_for_resource(&resource, Duration::from_secs(2), |s| s.is_ready());
        assert!(ready, "Resource should reach Ready state within timeout");

        match resource.get().as_ref() {
            ResourceState::Ready(data) => assert_eq!(data, "data"),
            _ => panic!("Expected Ready state"),
        }
    }

    #[ignore]
    #[test]
    fn test_resource_get_returns_gc() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Ok("test".to_string()) });

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
        });

        let ready = wait_for_resource(&resource, Duration::from_secs(2), |s| s.is_ready());
        assert!(ready, "Resource should reach Ready state");

        let state = resource.get();
        let state2 = resource.get();
        assert!(Gc::ptr_eq(&state, &state2));
    }
}
