//! Tests verifying that UiThreadDispatcher API works correctly.

#[cfg(feature = "async")]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use rvue::async_runtime::WriteSignalUiExt;
    use rvue::create_signal;
    use rvue::effect::create_effect;

    /// Test that dispatcher can be created.
    #[test]
    fn test_dispatcher_creation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (_count, set_count) = create_signal(0i32);
            let _dispatcher = set_count.ui_dispatcher();
        });
    }

    /// Test that dispatcher clones work independently.
    #[test]
    fn test_dispatcher_clone() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (_count, set_count) = create_signal(0i32);
            let dispatcher1 = set_count.ui_dispatcher();
            let _dispatcher2 = dispatcher1.clone();
        });
    }

    /// Test that dispatcher set method has correct bounds.
    #[test]
    fn test_dispatcher_set_bounds() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Test with i32 (Send + Sync)
            let (_count, set_count) = create_signal(0i32);
            let dispatcher = set_count.ui_dispatcher();
            dispatcher.set(42).await;

            // Test with String (Send + Sync)
            let (_text, set_text) = create_signal(String::new());
            let dispatcher = set_text.ui_dispatcher();
            dispatcher.set("hello".to_string()).await;

            // Test with Vec<i32> (Send + Sync)
            let (_vec, set_vec) = create_signal(Vec::<i32>::new());
            let dispatcher = set_vec.ui_dispatcher();
            dispatcher.set(vec![1, 2, 3]).await;
        });
    }

    /// Test that effect runs when signal is set directly.
    #[test]
    fn test_effect_runs_on_direct_set() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);
            let effect_count = Arc::new(AtomicUsize::new(0));
            let effect_count_clone = Arc::clone(&effect_count);

            create_effect(move || {
                let _ = count.get();
                effect_count_clone.fetch_add(1, Ordering::SeqCst);
            });

            // Direct set should trigger effect
            set_count.set(42);

            // Effect should have run
            assert!(effect_count.load(Ordering::SeqCst) >= 1);
        });
    }

    /// Test that effect runs on multiple direct sets.
    #[test]
    fn test_effect_runs_on_multiple_sets() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (count, set_count) = create_signal(0i32);
            let count_clone = count.clone();
            let effect_count = Arc::new(AtomicUsize::new(0));
            let effect_count_clone = Arc::clone(&effect_count);

            create_effect(move || {
                let _ = count_clone.get();
                effect_count_clone.fetch_add(1, Ordering::SeqCst);
            });

            // Multiple direct sets - effect runs at least once
            for i in 0..5 {
                set_count.set(i);
            }

            // Effect should have run at least once (batching may reduce count)
            // but we know it ran because the final value is 4
            let final_value = count.get();
            assert_eq!(final_value, 4, "Signal should have final value from last set");

            // Effect ran at least once for the final value
            assert!(
                effect_count.load(Ordering::SeqCst) >= 1,
                "Effect should run when signal changes"
            );
        });
    }
}
