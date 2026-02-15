//! Headless async refresh test - reproduces the hackernews crash

#[cfg(feature = "async")]
#[cfg(test)]
mod tests {
    use rvue::async_runtime::create_resource;
    use rvue::{create_memo, create_signal};
    use rvue::headless::{advance, advance_tokio, init_runtime};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    fn wait_for_async() {
        for _ in 0..20 {
            advance_tokio();
            advance();
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn pump_until(mut predicate: impl FnMut() -> bool, max_ticks: usize) -> bool {
        for _ in 0..max_ticks {
            advance_tokio();
            advance();
            if predicate() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    #[test]
    fn test_sequential_refresh_cycles() {
        init_runtime();

        let (source, set_source) = create_signal(0i32);
        let fetch_count = Arc::new(AtomicU32::new(0));
        let fetch_for_check = Arc::clone(&fetch_count);

        let resource = create_resource(source.clone(), move |s| {
            let c = Arc::clone(&fetch_count);
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(10)).await;
                let result = format!("data-{}-{}", s, c.load(Ordering::SeqCst));
                println!("Async task completed: {}", result);
                Ok(result)
            }
        });

        // Trigger refresh cycles
        for i in 0..10 {
            println!("=== Setting source to {}", i);
            set_source.set(i);
            advance_tokio();
            advance();

            // Wait for async to complete
            let mut waited = 0;
            while !resource.get().is_ready() && waited < 100 {
                advance_tokio();
                advance();
                std::thread::sleep(Duration::from_millis(20));
                waited += 1;
                println!("Waited {}", waited);
            }

            let state = resource.get();
            assert!(state.is_ready(), "Should be ready at cycle {}", i);
            let payload = state
                .data()
                .expect("Resource should contain data in Ready state");
            assert!(
                payload.starts_with(&format!("data-{i}-")),
                "Unexpected payload at cycle {i}: {}",
                payload
            );
        }

        println!("Fetch count: {}", fetch_for_check.load(Ordering::SeqCst));
    }

    #[test]
    fn test_rapid_refresh_cycles() {
        init_runtime();

        let (source, set_source) = create_signal(0i32);
        let fetch_count = Arc::new(AtomicU32::new(0));
        let fetch_count_for_check = Arc::clone(&fetch_count);

        let _resource = create_resource(source, move |_| {
            let c = Arc::clone(&fetch_count);
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(5)).await;
                Ok("ok")
            }
        });

        // Rapid updates - this might trigger the crash
        for i in 0..20 {
            set_source.set(i);
            advance_tokio();
            advance();
        }

        wait_for_async();

        println!("Rapid fetch count: {}", fetch_count_for_check.load(Ordering::SeqCst));
    }

    #[test]
    fn test_resource_memo_survives_many_refresh_cycles() {
        init_runtime();

        let (source, set_source) = create_signal(0usize);

        let resource = create_resource(source.clone(), move |s| async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok(format!("value-{s}"))
        });

        let resource_for_memo = resource.clone();
        let memo = create_memo(move || {
            resource_for_memo.get().data().cloned().unwrap_or_else(|| "pending".to_string())
        });

        // Warm up initial fetch.
        let initial_ready = pump_until(|| memo.get() == "value-0", 200);
        assert!(initial_ready, "Initial memo value should become value-0");

        // Regression guard for this bug:
        // repeated refresh should keep memo reactive and never stall permanently.
        for i in 1..=80usize {
            set_source.set(i);

            let expected = format!("value-{i}");
            let updated = pump_until(|| memo.get() == expected, 250);

            assert!(
                updated,
                "Memo did not update for cycle {i}. Current state: {:?}",
                resource.get()
            );
        }
    }
}
