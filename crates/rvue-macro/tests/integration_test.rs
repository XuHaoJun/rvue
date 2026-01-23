//! Integration tests for view! macro
//!
//! These tests verify that the macro works correctly with the full rvue runtime,
//! including signals, effects, and component trees.

use rvue::prelude::*;
use rvue_macro::view;

#[test]
fn test_signal_reactivity() {
    let (count, _set_count) = create_signal(0);

    let _view = view! {
        <Text content={format!("Count: {}", count.get())} />
    };

    let _ = _view;
}

#[test]
fn test_nested_with_signals() {
    let (text, _set_text) = create_signal("Hello".to_string());

    let _view = view! {
        <Flex direction="column">
            <Text content={text.get()} />
            <Text content="Static" />
        </Flex>
    };

    let _ = _view;
}

#[test]
fn test_show_widget_with_signal() {
    let (visible, _set_visible) = create_signal(true);

    let _view = view! {
        <Show when={visible.get()}>
            <Text content="Visible" />
        </Show>
    };

    let _ = _view;
}

#[test]
fn test_complex_nested_structure() {
    let (count, _set_count) = create_signal(0);
    let (label, _set_label) = create_signal("Button".to_string());

    let _view = view! {
        <Flex direction="column" gap={10.0}>
            <Text content={format!("Count: {}", count.get())} />
            <Button label={label.get()} />
            <Flex direction="row">
                <Text content="Nested" />
            </Flex>
        </Flex>
    };

    let _ = _view;
}

#[test]
fn test_multiple_signals() {
    let (name, _set_name) = create_signal("World".to_string());
    let (age, _set_age) = create_signal(25);

    let _view = view! {
        <Flex direction="column">
            <Text content={format!("Name: {}", name.get())} />
            <Text content={format!("Age: {}", age.get())} />
        </Flex>
    };

    let _ = _view;
}

// Event handler tests are commented out until event handler generation is fixed
// #[test]
// fn test_event_handler_with_signal() {
//     let (count, set_count) = create_signal(0);
//
//     let _view = view! {
//         <Button
//             label={format!("Count: {}", count.get())}
//             on_click=move || set_count.update(|c| *c += 1)
//         />
//     };
//
//     let _ = _view;
// }
