//! Unit tests for create_resource and Resource types.
//! These tests verify the async resource fetching behavior.

#[cfg(feature = "async")]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use rudo_gc::Gc;
    use rvue::async_runtime::{create_resource, Resource, ResourceState};
    use rvue::create_signal;

    #[test]
    fn test_create_resource_fetches_on_creation() {
        let (counter, _) = create_signal(0i32);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _resource = create_resource(counter, move |_| {
            let call_count = Arc::clone(&call_count_clone);
            async move {
                call_count.fetch_add(1, Ordering::SeqCst);
                Ok("result".to_string())
            }
        });

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_create_resource_handles_error() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Err("network error".to_string()) });

        match resource.get().as_ref() {
            ResourceState::Error(msg) => assert_eq!(msg, "network error"),
            _ => panic!("Expected Error state"),
        }
    }

    #[test]
    fn test_create_resource_state_transitions() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Ok("data".to_string()) });

        match resource.get().as_ref() {
            ResourceState::Ready(data) => assert_eq!(data, "data"),
            _ => panic!("Expected Ready state"),
        }
    }

    #[test]
    fn test_resource_get_returns_gc() {
        let (counter, _) = create_signal(0i32);

        let resource: Resource<String, _> =
            create_resource(counter, |_| async move { Ok("test".to_string()) });

        let state = resource.get();
        let state2 = resource.get();
        assert!(Gc::ptr_eq(&state, &state2));
    }
}
