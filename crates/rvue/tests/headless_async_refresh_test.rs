//! Headless async refresh test - reproduces the hackernews crash

#[cfg(feature = "async")]
#[cfg(test)]
mod tests {
    use rvue::async_runtime::create_resource;
    use rvue::create_signal;
    use rvue::headless::{advance, init_runtime};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    fn wait_for_async() {
        for _ in 0..20 {
            advance();
            std::thread::sleep(Duration::from_millis(10));
        }
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
            advance();

            // Wait for async to complete
            let mut waited = 0;
            while !resource.get().is_ready() && waited < 100 {
                advance();
                std::thread::sleep(Duration::from_millis(20));
                waited += 1;
                println!("Waited {}, state = {:?}", waited, resource.get());
            }

            let state = resource.get();
            println!("Cycle {}: state = {:?}", i, state);
            assert!(state.is_ready(), "Should be ready at cycle {}", i);
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
            advance();
        }

        wait_for_async();

        println!("Rapid fetch count: {}", fetch_count_for_check.load(Ordering::SeqCst));
    }
}
