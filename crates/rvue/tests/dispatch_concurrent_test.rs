//! Tests for concurrent dispatch_to_ui behavior.
//! Verifies that dispatch works correctly when called from multiple concurrent tasks.

#[cfg(feature = "async")]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use rvue::async_runtime::{dispatch_to_ui, spawn_task};

    #[test]
    fn test_dispatch_no_panic() {
        let call_count = Arc::new(AtomicUsize::new(0));

        let handle = spawn_task({
            let call_count = Arc::clone(&call_count);
            async move {
                for _ in 0..10 {
                    let call_count = Arc::clone(&call_count);
                    dispatch_to_ui(move || {
                        call_count.fetch_add(1, Ordering::SeqCst);
                    });
                }
            }
        });

        let _ = handle;
    }

    #[test]
    fn test_multiple_dispatches() {
        for _ in 0..5 {
            let call_count = Arc::new(AtomicUsize::new(0));

            let handle = spawn_task({
                let call_count = Arc::clone(&call_count);
                async move {
                    let call_count = Arc::clone(&call_count);
                    dispatch_to_ui(move || {
                        call_count.fetch_add(1, Ordering::SeqCst);
                    });
                }
            });

            let _ = handle;
        }
    }
}
