use rvue::prelude::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_effect_cleanup() {
    let (count, set_count) = create_signal(0);
    let cleanup_called = Arc::new(Mutex::new(0));

    let cleanup_clone = Arc::clone(&cleanup_called);

    // Store the effect to prevent it from being dropped
    let _effect = create_effect(move || {
        let _value = count.get();
        let cleanup_inner = Arc::clone(&cleanup_clone);
        on_cleanup(move || {
            let mut count = cleanup_inner.lock().unwrap();
            *count += 1;
        });
    });

    // On creation, cleanup is NOT called
    assert_eq!(*cleanup_called.lock().unwrap(), 0);

    // Update signal, triggers re-run of effect
    set_count.set(1);

    // Cleanup from previous run should be called
    assert_eq!(*cleanup_called.lock().unwrap(), 1);

    // Update signal again
    set_count.set(2);
    assert_eq!(*cleanup_called.lock().unwrap(), 2);
}

#[test]
fn test_component_cleanup() {
    let cleanup_called = Arc::new(Mutex::new(0));
    let cleanup_inner = Arc::clone(&cleanup_called);

    let component = Component::new(
        1,
        ComponentType::Custom("Test".to_string()),
        ComponentProps::Custom { data: "".to_string() },
    );

    rvue::runtime::with_owner(component.clone(), move || {
        on_cleanup(move || {
            let mut count = cleanup_inner.lock().unwrap();
            *count += 1;
        });
    });

    // Unmount the component
    component.unmount();

    // Component cleanup should be called
    assert_eq!(*cleanup_called.lock().unwrap(), 1);
}
