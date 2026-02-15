//! Integration tests for view! macro
//!
//! These tests verify that the macro works correctly with the full rvue runtime,
//! including signals, effects, and component trees.

#![allow(unused_braces)]

use rvue::prelude::*;
use rvue_macro::view;

#[test]
fn test_signal_reactivity() {
    let (count, _set_count) = create_signal(0);
    let count_label = || format!("Count: {}", count.get());

    #[allow(unused_braces)]
    let _view = view! {
        <Text content={count_label()} />
    };

    let _ = _view;
}

#[test]
fn test_nested_with_signals() {
    let (text, _set_text) = create_signal("Hello".to_string());
    let text_value = text.get();

    #[allow(unused_braces)]
    let _view = view! {
        <Flex direction="column">
            <Text content={text_value} />
            <Text content="Static" />
        </Flex>
    };

    let _ = _view;
}

#[test]
fn test_show_widget_with_signal() {
    let (visible, _set_visible) = create_signal(true);
    let visible_value = visible.get();

    #[allow(unused_braces)]
    let _view = view! {
        <Show when={visible_value}>
            <Text content="Visible" />
        </Show>
    };

    let _ = _view;
}

#[test]
fn test_two_reactive_then_nested_flex() {
    let (count, _set_count) = create_signal(0);
    let (label, _set_label) = create_signal("Button".to_string());
    let count_label = || format!("Count: {}", count.get());
    let label_value = label.get();

    #[allow(unused_braces)]
    let _view = view! {
        <Flex direction="column" gap=10.0>
            <Text content={count_label()} />
            <Text content={label_value} />
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
    let name_label = || format!("Name: {}", name.get());
    let age_label = || format!("Age: {}", age.get());

    #[allow(unused_braces)]
    let _view = view! {
        <Flex direction="column">
            <Text content={name_label()} />
            <Text content={age_label()} />
        </Flex>
    };

    let _ = _view;
}

#[test]
fn test_event_handler_with_signal() {
    let (count, _set_count) = create_signal(0);
    let count_label = || format!("Count: {}", count.get());

    #[allow(unused_braces)]
    let _view = view! {
        <Text content={count_label()} />
    };

    let _ = _view;
}

#[test]
fn test_reactive_text_update() {
    let (count, set_count) = create_signal("0".to_string());

    let view = view! {
        <Text content={count.clone()} />
    };

    set_count.set("1".to_string());

    let content = {
        view.root_component
            .properties
            .borrow()
            .get::<rvue::properties::TextContent>()
            .map(|tc| tc.0.clone())
            .expect("Expected TextContent property")
    };
    assert_eq!(content, "1");

    rvue::signal::__test_clear_signal_subscriptions();
}

#[test]
fn test_static_vs_dynamic_effects() {
    let (count, set_count) = create_signal("0".to_string());

    let reactive_view = view! {
        <Text content={count.clone()} />
    };

    let static_view = view! {
        <Text content="Static" />
    };

    set_count.set("1".to_string());

    assert_eq!(reactive_view.root_component.effects.borrow().len(), 1);
    assert_eq!(static_view.root_component.effects.borrow().len(), 0);

    rvue::signal::__test_clear_signal_subscriptions();
}

#[test]
fn test_nested_reactive_update() {
    let (name, set_name) = create_signal("Alice".to_string());

    let view = view! {
        <Flex direction="column">
            <Text content={name.clone()} />
        </Flex>
    };

    set_name.set("Bob".to_string());

    let content = {
        let child = view.root_component.children.borrow().first().cloned().expect("child");
        let text_content =
            child.properties.borrow().get::<rvue::properties::TextContent>().map(|tc| tc.0.clone());
        text_content.expect("Expected TextContent property")
    };
    assert_eq!(content, "Bob");

    rvue::signal::__test_clear_signal_subscriptions();
}
