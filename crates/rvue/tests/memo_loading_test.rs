use rvue::{create_memo, create_signal};

#[test]
fn test_memo_with_loading_state() {
    // Simulate the hackernews scenario:
    // - A resource with Loading -> Ready states
    // - A memo that extracts data
    // - Multiple state transitions

    let (state, set_state) = create_signal(0i32);

    // Memo that simulates extracting stories from resource
    let memo = create_memo(move || {
        let s = state.get();
        if s == 0 {
            vec![0; 0].len() // Empty array (Loading)
        } else {
            vec![1; 10].len() // 10 items (Ready)
        }
    });

    // Initial state
    assert_eq!(memo.get(), 0); // Should be empty

    // Set to Loading state (simulate refetch)
    set_state.set(1);
    assert_eq!(memo.get(), 10); // Should have 10 items

    // Set to Loading again
    set_state.set(2);
    assert_eq!(memo.get(), 10); // Should still have 10

    // Set back to some value
    set_state.set(3);
    assert_eq!(memo.get(), 10);
}

#[test]
fn test_signal_notify_multiple_subscribers() {
    use std::cell::Cell;
    use std::rc::Rc;

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

    // Both effects should have run once
    assert_eq!(effect1_count.get(), 1);
    assert_eq!(effect2_count.get(), 1);

    // Set signal value - both effects should run
    set_signal.set(10);

    // Both effects should have run again
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
    assert_eq!(memo.get().len(), 0); // Should be empty!

    set_source.set(7);
    assert_eq!(memo.get().len(), 7);
}
