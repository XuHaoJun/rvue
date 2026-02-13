//! Headless async refresh test
//!
//! This test simulates multiple refresh cycles of a resource to reproduce
//! the crash that occurs when clicking refresh multiple times.

#[cfg(feature = "async")]
#[cfg(test)]
mod tests {
    use rvue::async_runtime::create_resource;
    use rvue::create_signal;
    use rvue::headless::{advance, init_runtime, UiDispatchQueue};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    /// Simple wait function that runs tokio to let async complete
    fn wait_for_async() {
        // We can't use block_on here because we're not in an async context
        // Instead, just advance the queues multiple times
        for _ in 0..10 {
            advance();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn test_resource_refresh_cycle() {
        // Initialize the async runtime
        init_runtime();

        // Create a source signal
        let (refresh_counter, set_refresh) = create_signal(0u32);

        // Create a resource that fetches data - use Arc for shared state
        let fetch_count = Arc::new(AtomicU32::new(0));
        let fetch_count_for_check = Arc::clone(&fetch_count);

        let resource = create_resource(refresh_counter, move |_| {
            let count = Arc::clone(&fetch_count);
            async move {
                eprintln!("[TEST] Starting fetch");
                count.fetch_add(1, Ordering::SeqCst);
                // Simulate async fetch
                tokio::time::sleep(Duration::from_millis(10)).await;
                eprintln!("[TEST] Fetch complete");
                Ok(format!("data-{}", count.load(Ordering::SeqCst)))
            }
        });

        // Initial fetch should start
        advance();

        // Wait for async
        wait_for_async();

        // Check if ready
        let state = resource.get();
        eprintln!("[TEST] After waiting, state: is_ready={}", state.is_ready());

        assert!(state.is_ready(), "Resource should be ready after first fetch");

        // Now simulate refresh cycles
        for i in 1..=10 {
            println!("Refresh cycle {}", i);

            // Trigger refresh
            set_refresh.set(i);

            // Advance the system
            advance();

            // Wait for async
            wait_for_async();

            // Check if ready
            let state = resource.get();
            assert!(state.is_ready(), "Resource should be ready after refresh {}", i);

            // Check data is correct
            if let Some(data) = state.data() {
                println!("Got data: {}", data);
            }
        }

        println!("Total fetch count: {}", fetch_count_for_check.load(Ordering::SeqCst));
    }

    #[test]
    fn test_resource_refetch_method() {
        // Initialize the async runtime
        init_runtime();

        // Create a simple source
        let (source, _) = create_signal(0i32);

        let fetch_count = Arc::new(AtomicU32::new(0));
        let fetch_count_for_check = Arc::clone(&fetch_count);

        let resource = create_resource(source, move |_| {
            let count = Arc::clone(&fetch_count);
            async move {
                eprintln!("[TEST] Starting fetch");
                count.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(5)).await;
                eprintln!("[TEST] Fetch complete");
                Ok("result")
            }
        });

        // Initial fetch
        advance();
        wait_for_async();

        let state = resource.get();
        eprintln!("[TEST] After waiting, state: is_ready={}", state.is_ready());

        assert!(state.is_ready(), "Should be ready after initial fetch");
        assert_eq!(fetch_count_for_check.load(Ordering::SeqCst), 1);

        // Call refetch
        resource.refetch();
        advance();
        wait_for_async();

        let state = resource.get();
        assert!(state.is_ready(), "Should be ready after refetch");
        assert_eq!(fetch_count_for_check.load(Ordering::SeqCst), 2);

        // Refetch again
        resource.refetch();
        advance();
        wait_for_async();

        let state = resource.get();
        assert!(state.is_ready(), "Should be ready after second refetch");
        assert_eq!(fetch_count_for_check.load(Ordering::SeqCst), 3);
    }
}
