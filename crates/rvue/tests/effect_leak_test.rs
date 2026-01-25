use rudo_gc::{Gc, Trace};
use rvue::{create_effect, create_signal, on_cleanup, SignalRead, SignalWrite};
use std::cell::Cell;

// A simple structure to track if it has been dropped
struct DropTracker {
    is_dropped: Gc<Cell<bool>>,
}

unsafe impl Trace for DropTracker {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.is_dropped.trace(visitor);
    }
}

impl Drop for DropTracker {
    fn drop(&mut self) {
        self.is_dropped.set(true);
    }
}

#[test]
fn test_effect_preserves_captured_gc() {
    eprintln!("TEST: Starting test_effect_preserves_captured_gc");
    let dropped = Gc::new(Cell::new(false));
    eprintln!("TEST: dropped ptr = {:?}", Gc::as_ptr(&dropped));
    let tracker = Gc::new(DropTracker { is_dropped: dropped.clone() });
    eprintln!("TEST: tracker ptr = {:?}", Gc::as_ptr(&tracker));
    eprintln!("TEST: Created tracker");

    let (read, _write) = create_signal(0i32);
    eprintln!("TEST: Created signal");

    // Create an effect that captures the tracker
    let effect = {
        let captured_tracker = tracker.clone();
        eprintln!(
            "TEST: Creating effect, captured_tracker ptr = {:?}",
            Gc::as_ptr(&captured_tracker)
        );
        create_effect(move || {
            let _val = read.get();
            let _ = &captured_tracker;
        })
    };
    eprintln!("TEST: Created effect, effect = {:?}", effect);

    eprintln!(
        "TEST: About to drop tracker, tracker ref_count before = {}",
        Gc::ref_count(&tracker)
    );
    eprintln!("TEST: Dropping tracker");
    drop(tracker);
    eprintln!("TEST: tracker dropped, dropped ptr still = {:?}", Gc::as_ptr(&dropped));

    eprintln!("TEST: About to call collect_full");
    // Trigger full GC
    rudo_gc::collect_full();
    eprintln!("TEST: collect_full returned");

    // The tracker should NOT have been dropped
    eprintln!("TEST: About to check assertion");
    let dropped_value = dropped.get();
    eprintln!("TEST: dropped.get() = {}", dropped_value);
    assert!(!dropped_value, "Tracker was incorrectly collected!");
    eprintln!("TEST: Assertion passed");

    eprintln!("TEST: About to drop effect");
    drop(effect);
    eprintln!("TEST: Test completed");
}

#[test]
fn test_effect_cleanup_preserves_captured_gc() {
    let dropped = Gc::new(Cell::new(false));
    let tracker = Gc::new(DropTracker { is_dropped: dropped.clone() });

    let (read, write) = create_signal(0i32);

    // Create an effect with a cleanup that captures the tracker
    let effect = {
        let captured_tracker = tracker.clone();
        create_effect(move || {
            let _val = read.get();
            let inner_tracker = captured_tracker.clone();
            on_cleanup(move || {
                let _ = &inner_tracker;
            });
        })
    };

    drop(tracker);
    write.set(1);

    // Trigger full GC
    rudo_gc::collect_full();

    // The tracker should NOT have been dropped (it's in the cleanup vec now)
    assert!(!dropped.get(), "Tracker in cleanup was incorrectly collected!");

    drop(effect);
}
