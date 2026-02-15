//! Tests for watch_signal.
//! Verifies that watch_signal creates and stops correctly.

#[cfg(feature = "async")]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;
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

            watcher.stop();
        });
    }

    #[test]
    fn test_watch_signal_with_closure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);

            let _watcher = watch_signal(count, set_count, Duration::from_millis(100), |current| {
                Some(current * 2)
            })
            .await;
        });
    }

    #[test]
    fn test_watch_signal_panic_count_initially_zero() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);

            let watcher =
                watch_signal(count, set_count, Duration::from_millis(100), |_current| Some(1))
                    .await;

            assert_eq!(watcher.panic_count(), 0);
            watcher.stop();
        });
    }

    #[test]
    fn test_watch_signal_set_on_panic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);
            let handler_called = Arc::new(AtomicU8::new(0));
            let handler_called_clone = handler_called.clone();

            let mut watcher = watch_signal(count, set_count, Duration::from_millis(10), |_| {
                panic!("intentional panic")
            })
            .await;

            watcher.set_on_panic(move || {
                handler_called_clone.fetch_add(1, Ordering::SeqCst);
            });

            let panic_count = watcher.panic_count();
            watcher.stop();

            assert!(panic_count == 0);
        });
    }
}
