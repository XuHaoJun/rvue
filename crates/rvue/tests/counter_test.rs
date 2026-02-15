//! Integration test for counter component

use rvue::create_signal;

#[test]
fn test_counter_state_management() {
    // Test that state can be declared and updated
    let (count, set_count) = create_signal(0);
    assert_eq!(count.get(), 0);

    set_count.set(5);
    assert_eq!(count.get(), 5);

    set_count.update(|x| *x += 1);
    assert_eq!(count.get(), 6);
}

#[test]
fn test_counter_increment_decrement() {
    let (count, set_count) = create_signal(0);

    // Simulate increment
    set_count.update(|x| *x += 1);
    assert_eq!(count.get(), 1);

    set_count.update(|x| *x += 1);
    assert_eq!(count.get(), 2);

    // Simulate decrement
    set_count.update(|x| *x -= 1);
    assert_eq!(count.get(), 1);

    set_count.update(|x| *x -= 1);
    assert_eq!(count.get(), 0);
}

#[test]
fn test_counter_reactive_updates() {
    let (count, set_count) = create_signal(0);
    let call_count = std::rc::Rc::new(std::cell::Cell::new(0));
    let last_value = std::rc::Rc::new(std::cell::Cell::new(0));

    // Create an effect that tracks the count
    let _effect = rvue::create_effect({
        let count = count.clone();
        let call_count = call_count.clone();
        let last_value = last_value.clone();
        move || {
            let val = count.get(); // Access the signal
            last_value.set(val);
            call_count.set(call_count.get() + 1);
        }
    });

    // Effect should have run once on creation
    assert_eq!(call_count.get(), 1);
    assert_eq!(last_value.get(), 0);

    // Update count - effect should re-run
    set_count.set(10);
    // Note: Effects run synchronously when signals are updated
    assert_eq!(call_count.get(), 2, "Effect should re-run when signal changes");
    assert_eq!(last_value.get(), 10);
    assert_eq!(count.get(), 10);

    set_count.set(20);
    assert_eq!(call_count.get(), 3);
    assert_eq!(last_value.get(), 20);
    assert_eq!(count.get(), 20);

    // Clear subscriptions before test end to avoid TLS destroy-order: dropping
    // Gc<Effect> during thread_local teardown would trigger GC, which can access
    // already-destroyed rudo-gc thread locals.
    rvue::signal::__test_clear_signal_subscriptions();
}
