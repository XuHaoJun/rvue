use rvue::prelude::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_nested_signal_updates() {
    let (counter, set_counter) = create_signal(0);
    let update_count = Arc::new(Mutex::new(0));

    let set_counter_clone = set_counter.clone();
    let update_clone = Arc::clone(&update_count);
    let _effect = create_effect(move || {
        let _val = counter.get();
        let mut count = update_clone.lock().unwrap();
        *count += 1;

        // Simulate nested signal update (like resource does)
        if *count < 3 {
            set_counter_clone.set(*count);
        }
    });

    // Trigger the initial update
    set_counter.set(1);

    let final_count = *update_count.lock().unwrap();
    assert!(final_count >= 1, "Effect should have run at least once, got {}", final_count);
}

#[test]
fn test_rapid_signal_updates() {
    let (value, set_value) = create_signal(0);
    let values_seen = Arc::new(Mutex::new(Vec::new()));

    let values_clone = Arc::clone(&values_seen);
    let _effect = create_effect(move || {
        let val = value.get();
        let mut seen = values_clone.lock().unwrap();
        seen.push(val);
    });

    // Rapid updates
    for i in 0..100 {
        set_value.set(i);
    }

    let seen = values_seen.lock().unwrap();
    assert!(!seen.is_empty(), "Should have seen at least one update");
}

#[test]
fn test_async_like_refresh_cycle() {
    let (refresh_counter, inc_refresh) = create_signal(0u32);
    let (data_state, set_data_state) = create_signal(0u32);
    let render_count = Arc::new(Mutex::new(0));

    let render_clone = Arc::clone(&render_count);
    let _effect = create_effect(move || {
        let _refresh = refresh_counter.get();
        let _data = data_state.get();

        let mut count = render_clone.lock().unwrap();
        *count += 1;
    });

    // Simulate refresh cycle: increment refresh -> effect runs -> set data -> effect runs again
    let mut refresh_val = 0u32;
    let mut data_val = 0u32;
    for _ in 0..10 {
        inc_refresh.set(refresh_val);
        refresh_val = refresh_val.wrapping_add(1);
        // In real app, this would be async, but we simulate sync
        set_data_state.set(data_val);
        data_val = data_val.wrapping_add(1);
    }

    let count = *render_count.lock().unwrap();
    assert!(count >= 10, "Effect should have run at least 10 times, got {}", count);
}

#[test]
fn test_multiple_effects_chained() {
    let (source, set_source) = create_signal(0);
    let (middle, set_middle) = create_signal(0);
    let (end, set_end) = create_signal(0);

    let set_middle_clone = set_middle.clone();
    let _effect1 = create_effect(move || {
        let val = source.get();
        set_middle_clone.set(val * 2);
    });

    let set_end_clone = set_end.clone();
    let _effect2 = create_effect(move || {
        let val = middle.get();
        set_end_clone.set(val + 1);
    });

    let final_values = Arc::new(Mutex::new(Vec::new()));
    let final_clone = Arc::clone(&final_values);
    let _effect3 = create_effect(move || {
        let val = end.get();
        let mut values = final_clone.lock().unwrap();
        values.push(val);
    });

    // Trigger chain
    for i in 0..20 {
        set_source.set(i);
    }

    let values = final_values.lock().unwrap();
    assert!(!values.is_empty(), "Should have captured final values");
}

#[test]
fn test_deeply_nested_notifications() {
    let (a, set_a) = create_signal(0);
    let (_b, set_b) = create_signal(0);
    let (_c, set_c) = create_signal(0);
    let (_d, set_d) = create_signal(0);

    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    let set_b_clone = set_b.clone();
    let set_c_clone = set_c.clone();
    let set_d_clone = set_d.clone();

    let _effect = create_effect(move || {
        let a_val = a.get();

        // This will trigger nested notifications
        if a_val > 0 && a_val < 100 {
            set_b_clone.set(a_val * 2);
            set_c_clone.set(a_val * 3);
            set_d_clone.set(a_val * 4);
        }

        let mut cnt = count_clone.lock().unwrap();
        *cnt += 1;
    });

    for i in 0..50 {
        set_a.set(i);
    }

    let final_count = *count.lock().unwrap();
    assert!(final_count >= 50, "Effect should have run at least 50 times, got {}", final_count);
}
