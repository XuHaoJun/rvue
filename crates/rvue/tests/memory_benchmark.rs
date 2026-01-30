//! Benchmark test for initial memory usage

use rvue::{Component, ComponentProps, ComponentType, ViewStruct};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Simple memory tracking allocator for testing
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

#[test]
#[ignore] // Ignore by default - run with `cargo test -- --ignored`
fn benchmark_initial_memory_footprint() {
    // Reset allocation counter
    ALLOCATED.store(0, Ordering::Relaxed);

    // Create a simple application
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

    // Add some components
    for i in 1..=10 {
        let _child = Component::new(
            i,
            ComponentType::Text,
            ComponentProps::Text { content: format!("Item {}", i), font_size: None, color: None },
        );
    }

    let _view = ViewStruct::new(root);

    // Get allocated memory (approximate)
    let allocated = ALLOCATED.load(Ordering::Relaxed);
    let allocated_mb = allocated as f64 / (1024.0 * 1024.0);

    // Target: < 100MB initial memory
    let target_mb = 100.0;
    assert!(
        allocated_mb < target_mb,
        "Initial memory footprint {}MB exceeds target of {}MB",
        allocated_mb,
        target_mb
    );

    println!("Initial memory footprint: {:.2}MB (target: <{}MB)", allocated_mb, target_mb);
}

#[test]
#[ignore]
fn benchmark_component_memory_usage() {
    // Measure memory usage for component creation
    ALLOCATED.store(0, Ordering::Relaxed);

    // Create 100 components
    let mut components = Vec::new();
    for i in 0..100 {
        let component = Component::new(
            i,
            ComponentType::Text,
            ComponentProps::Text {
                content: format!("Component {}", i),
                font_size: None,
                color: None,
            },
        );
        components.push(component);
    }

    let allocated = ALLOCATED.load(Ordering::Relaxed);
    let allocated_kb = allocated as f64 / 1024.0;
    let per_component = allocated_kb / 100.0;

    println!(
        "Memory for 100 components: {:.2}KB ({:.2}KB per component)",
        allocated_kb, per_component
    );

    // Target: < 1MB for 100 components
    let target_kb = 1024.0;
    assert!(
        allocated_kb < target_kb,
        "Component memory usage {}KB exceeds target of {}KB",
        allocated_kb,
        target_kb
    );
}

#[test]
#[ignore]
fn benchmark_signal_memory_usage() {
    // Measure memory usage for signal creation
    ALLOCATED.store(0, Ordering::Relaxed);

    // Create 100 signals
    let mut signals = Vec::new();
    for i in 0..100 {
        let (read, _write) = rvue::create_signal(i);
        signals.push(read);
    }

    let allocated = ALLOCATED.load(Ordering::Relaxed);
    let allocated_kb = allocated as f64 / 1024.0;
    let per_signal = allocated_kb / 100.0;

    println!("Memory for 100 signals: {:.2}KB ({:.2}KB per signal)", allocated_kb, per_signal);

    // Target: < 100KB for 100 signals
    let target_kb = 100.0;
    assert!(
        allocated_kb < target_kb,
        "Signal memory usage {}KB exceeds target of {}KB",
        allocated_kb,
        target_kb
    );
}
