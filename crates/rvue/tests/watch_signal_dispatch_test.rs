//! Tests for watch_signal.
//! Verifies that watch_signal creates and stops correctly.

#[cfg(feature = "async")]
mod tests {
    use std::time::Duration;

    use rvue::async_runtime::watch_signal;
    use rvue::create_signal;

    #[test]
    fn test_watch_signal_creation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);

            let watcher =
                watch_signal(count, set_count, Duration::from_millis(10), |_current| Some(1)).await;

            // Just verify it was created without panic
            watcher.stop();
        });
    }

    #[test]
    fn test_watch_signal_with_closure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);

            let _watcher = watch_signal(count, set_count, Duration::from_millis(100), |current| {
                // Simple callback that doubles the value
                Some(current * 2)
            })
            .await;
        });
    }
}
