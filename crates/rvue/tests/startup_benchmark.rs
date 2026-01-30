//! Benchmark test for application startup time

use rvue::{Component, ComponentProps, ComponentType, ViewStruct};
use std::time::{Duration, Instant};

#[test]
#[ignore] // Ignore by default - run with `cargo test -- --ignored`
fn benchmark_startup_time() {
    // Measure startup time for a simple application
    let start = Instant::now();

    // Simulate application initialization
    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "center".to_string(),
            styles: None,
        },
    );

    let _view = ViewStruct::new(root);

    let elapsed = start.elapsed();

    // Target: < 2 seconds for startup
    let target = Duration::from_secs(2);
    assert!(
        elapsed < target,
        "Startup time {}ms exceeds target of {}ms",
        elapsed.as_millis(),
        target.as_millis()
    );

    println!("Startup time: {}ms (target: <{}ms)", elapsed.as_millis(), target.as_millis());
}

#[test]
#[ignore]
fn benchmark_component_tree_creation() {
    // Measure time to create a component tree
    let start = Instant::now();

    // Create a tree with 100 components
    let _root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 5.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    // Add 100 child components
    for i in 1..=100 {
        let _child = Component::new(
            i,
            ComponentType::Text,
            ComponentProps::Text { content: format!("Item {}", i), styles: None },
        );
        // Note: In a full implementation, we'd add this to root's children
    }

    let elapsed = start.elapsed();

    // Target: < 100ms for 100 components
    let target = Duration::from_millis(100);
    assert!(
        elapsed < target,
        "Component tree creation time {}ms exceeds target of {}ms",
        elapsed.as_millis(),
        target.as_millis()
    );

    println!(
        "Component tree creation (100 components): {}ms (target: <{}ms)",
        elapsed.as_millis(),
        target.as_millis()
    );
}

#[test]
#[ignore]
fn benchmark_signal_creation() {
    // Measure time to create signals
    let start = Instant::now();

    // Create 100 signals
    for _ in 0..100 {
        let (_read, _write) = rvue::create_signal(0);
    }

    let elapsed = start.elapsed();

    // Target: < 10ms for 100 signals
    let target = Duration::from_millis(10);
    assert!(
        elapsed < target,
        "Signal creation time {}ms exceeds target of {}ms",
        elapsed.as_millis(),
        target.as_millis()
    );

    println!(
        "Signal creation (100 signals): {}ms (target: <{}ms)",
        elapsed.as_millis(),
        target.as_millis()
    );
}
