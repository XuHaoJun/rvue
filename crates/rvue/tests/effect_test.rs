//! Unit tests for Effect implementation

use rvue::{create_effect, create_signal, Effect};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_create_effect_runs_immediately() {
    let call_count = Rc::new(Cell::new(0));

    let effect = create_effect({
        let call_count = call_count.clone();
        move || {
            call_count.set(call_count.get() + 1);
        }
    });

    // Effect should have run once on creation
    assert_eq!(call_count.get(), 1);
    assert!(!effect.is_dirty());

    rudo_gc::collect_full();
}

#[test]
fn test_effect_reruns_on_signal_change() {
    let (read, write) = create_signal(0);
    let call_count = Rc::new(Cell::new(0));
    let last_value = Rc::new(Cell::new(0));

    let _effect = create_effect({
        let read = read.clone();
        let call_count = call_count.clone();
        let last_value = last_value.clone();
        move || {
            call_count.set(call_count.get() + 1);
            last_value.set(read.get());
        }
    });

    assert_eq!(call_count.get(), 1, "Effect should run once on creation");
    assert_eq!(last_value.get(), 0);

    // Update signal - effect should be marked dirty and re-run
    write.set(10);
    assert_eq!(call_count.get(), 2, "Effect should run again when signal changes");
    assert_eq!(last_value.get(), 10);

    write.set(20);
    assert_eq!(call_count.get(), 3);
    assert_eq!(last_value.get(), 20);
}

#[test]
fn test_effect_multiple_dependencies() {
    let (read1, write1) = create_signal(1);
    let (read2, write2) = create_signal(2);
    let call_count = Rc::new(Cell::new(0));

    let _effect = create_effect({
        let read1 = read1.clone();
        let read2 = read2.clone();
        let call_count = call_count.clone();
        move || {
            call_count.set(call_count.get() + 1);
            let _ = read1.get();
            let _ = read2.get();
        }
    });

    assert_eq!(call_count.get(), 1);

    // Update first signal
    write1.set(10);
    assert_eq!(call_count.get(), 2);

    // Update second signal
    write2.set(20);
    assert_eq!(call_count.get(), 3);
}

#[test]
fn test_effect_mark_dirty() {
    let effect = create_effect(|| {});

    assert!(!effect.is_dirty()); // Should be clean after creation

    effect.mark_dirty();
    assert!(effect.is_dirty());

    Effect::update_if_dirty(&effect);
    assert!(!effect.is_dirty()); // Should be clean after update
}

#[test]
fn test_effect_independent_signals() {
    let (read1, write1) = create_signal(1);
    let (_read2, write2) = create_signal(2);
    let call_count = Rc::new(Cell::new(0));

    let _effect = create_effect({
        let read1 = read1.clone();
        let call_count = call_count.clone();
        move || {
            call_count.set(call_count.get() + 1);
            let _ = read1.get(); // Only depends on read1
        }
    });

    assert_eq!(call_count.get(), 1);

    // Update read2 - effect should NOT re-run (not a dependency)
    write2.set(20);
    assert_eq!(call_count.get(), 1); // Unchanged

    // Update read1 - effect SHOULD re-run
    write1.set(10);
    assert_eq!(call_count.get(), 2);
}

/// Regression test for subscription cleanup on effect re-run
/// This tests the fix for the memory corruption bug that occurred when
/// effects were re-run multiple times (like clicking refresh multiple times).
/// The bug was caused by stale weak references remaining in signal subscriber lists.
#[test]
fn test_effect_subscription_cleanup_on_rerun() {
    let (counter, set_counter) = create_signal(0);
    let call_count = Rc::new(Cell::new(0));

    let _effect = create_effect({
        let counter = counter.clone();
        let call_count = call_count.clone();
        move || {
            call_count.set(call_count.get() + 1);
            let _ = counter.get();
        }
    });

    assert_eq!(call_count.get(), 1, "Effect should run once on creation");

    // Simulate multiple signal updates (like clicking refresh multiple times)
    // This is the scenario that caused the memory corruption bug
    for i in 1..=50 {
        set_counter.set(i);
        assert_eq!(call_count.get(), i + 1, "Effect should run on signal change {}", i);
    }

    // Force GC to trace subscriptions - this would crash with stale weak refs
    rudo_gc::collect_full();

    // Continue updating - should still work without crashes
    for i in 51..=100 {
        set_counter.set(i);
        assert_eq!(call_count.get(), i + 1, "Effect should continue working after GC {}", i);
    }

    // Final GC to verify no corruption
    rudo_gc::collect_full();
}
