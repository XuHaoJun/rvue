use rvue::{create_memo, create_signal};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_memo_basic() {
    let (read, write) = create_signal(10);
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();

    let memo = create_memo(move || {
        count_clone.set(count_clone.get() + 1);
        read.get() * 2
    });

    assert_eq!(memo.get(), 20);
    assert_eq!(count.get(), 2);

    write.set(20);
    assert_eq!(memo.get(), 40);
    assert_eq!(count.get(), 3);
}

#[test]
fn test_memo_with_loading_state() {
    let (state, set_state) = create_signal(0i32);

    let memo = create_memo(move || {
        let s = state.get();
        if s == 0 {
            vec![0; 0].len()
        } else {
            vec![1; 10].len()
        }
    });

    assert_eq!(memo.get(), 0);

    set_state.set(1);
    assert_eq!(memo.get(), 10);

    set_state.set(2);
    assert_eq!(memo.get(), 10);

    set_state.set(3);
    assert_eq!(memo.get(), 10);
}

#[test]
fn test_signal_notify_multiple_subscribers() {
    let (signal, set_signal) = create_signal(0i32);
    let signal_clone = signal.clone();

    let effect1_count = Rc::new(Cell::new(0));
    let effect1_count_clone = effect1_count.clone();
    let _effect1 = rvue::create_effect(move || {
        signal_clone.get();
        effect1_count_clone.set(effect1_count_clone.get() + 1);
    });

    let effect2_count = Rc::new(Cell::new(0));
    let effect2_count_clone = effect2_count.clone();
    let _effect2 = rvue::create_effect(move || {
        signal.get();
        effect2_count_clone.set(effect2_count_clone.get() + 1);
    });

    assert_eq!(effect1_count.get(), 1);
    assert_eq!(effect2_count.get(), 1);

    set_signal.set(10);

    assert_eq!(effect1_count.get(), 2);
    assert_eq!(effect2_count.get(), 2);
}

#[test]
fn test_memo_updates_on_dependency_change() {
    let (source, set_source) = create_signal(5i32);

    let memo = create_memo(move || {
        let val = source.get();
        vec![0; val as usize]
    });

    assert_eq!(memo.get().len(), 5);

    set_source.set(3);
    assert_eq!(memo.get().len(), 3);

    set_source.set(0);
    assert_eq!(memo.get().len(), 0);

    set_source.set(7);
    assert_eq!(memo.get().len(), 7);
}

#[test]
fn test_multiple_refresh_cycles() {
    // Simulates the hackernews refresh scenario
    let (counter, increment) = create_signal(0i32);

    // Memo that extracts data (like stories)
    let memo = create_memo(move || {
        let c = counter.get();
        if c == 0 {
            vec![0; 0].len() // Loading state - empty
        } else {
            vec![1; 10].len() // Ready state - 10 items
        }
    });

    // Initial state
    assert_eq!(memo.get(), 0);

    // First refresh cycle
    increment.set(1);
    assert_eq!(memo.get(), 10);

    // Second refresh cycle
    increment.set(2);
    assert_eq!(memo.get(), 10);

    // Third refresh cycle
    increment.set(3);
    assert_eq!(memo.get(), 10);

    // Fourth refresh cycle
    increment.set(4);
    assert_eq!(memo.get(), 10);

    // Fifth refresh cycle
    increment.set(5);
    assert_eq!(memo.get(), 10);
}

#[test]
fn test_rapid_state_changes() {
    // Test rapid state changes like in the bug
    let (state, set_state) = create_signal(0i32);

    let memo = create_memo(move || {
        let s = state.get();
        vec![0; s as usize]
    });

    // Rapid changes
    for i in 1..=20 {
        set_state.set(i);
        let len = memo.get().len();
        assert_eq!(len, i as usize, "Failed at iteration {}", i);
    }
}
