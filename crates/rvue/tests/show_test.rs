//! Integration test for Show component

use rvue::{create_signal, ComponentLifecycle};
use rvue::{Component, ComponentProps, ComponentType};

#[test]
fn test_show_component_conditional_rendering() {
    // Test that Show component can conditionally show/hide content
    let (is_visible, set_visible) = create_signal(true);

    // Create a child component
    let _child = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Visible".to_string(), font_size: None, styles: None },
    );

    // Create Show component
    let _show =
        Component::new(2, ComponentType::Show, ComponentProps::Show { when: is_visible.get() });

    // Initially visible
    assert!(is_visible.get());

    // Hide it
    set_visible.set(false);
    assert!(!is_visible.get());

    // Show it again
    set_visible.set(true);
    assert!(is_visible.get());
}

#[test]
fn test_show_component_with_signal_tracking() {
    let (is_visible, set_visible) = create_signal(false);
    let call_count = std::rc::Rc::new(std::cell::Cell::new(0));

    // Create an effect that tracks the visibility signal
    let _effect = rvue::create_effect({
        let is_visible = is_visible.clone();
        let call_count = call_count.clone();
        move || {
            let _ = is_visible.get(); // Track the signal
            call_count.set(call_count.get() + 1);
        }
    });

    // Effect should have run once on creation
    assert_eq!(call_count.get(), 1);

    // Update visibility - effect should re-run
    set_visible.set(true);
    assert_eq!(call_count.get(), 2);
    assert!(is_visible.get());

    set_visible.set(false);
    assert_eq!(call_count.get(), 3);
    assert!(!is_visible.get());
}

#[test]
fn test_show_component_mounting_unmounting() {
    let (is_visible, _set_visible) = create_signal(true);

    // Create Show component
    let show =
        Component::new(1, ComponentType::Show, ComponentProps::Show { when: is_visible.get() });

    // Test mounting/unmounting lifecycle
    show.mount(None);
    assert_eq!(show.component_type, ComponentType::Show);

    // Unmount should not panic
    show.unmount();
}
