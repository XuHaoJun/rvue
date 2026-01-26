//! Smoke tests for view! macro
//!
//! This test suite verifies that the view! macro compiles and generates valid code
//! for all built-in widgets and common patterns.

#![allow(unused_braces)]

use rvue_macro::view;

#[test]
fn test_text_widget() {
    // Test Text widget with content attribute
    let _view = view! {
        <Text content="Hello" />
    };
    // Just verify it compiles
    let _ = _view;
}

#[test]
fn test_text_widget_children() {
    // Test text content as children
    let _view = view! {
        <Text>"Hello World"</Text>
    };
}

#[test]
fn test_button_widget() {
    let _view = view! {
        <Button label="Click Me" />
    };
}

#[test]
fn test_flex_widget() {
    let _view = view! {
        <Flex direction="column" gap=10.0 />
    };
}

#[test]
fn test_flex_widget_with_children() {
    let _view = view! {
        <Flex direction="column" gap=20.0>
            <Text content="Child 1" />
            <Text content="Child 2" />
        </Flex>
    };
}

#[test]
fn test_text_input_widget() {
    let _view = view! {
        <TextInput value="test" />
    };
}

#[test]
fn test_number_input_widget() {
    let _view = view! {
        <NumberInput value=42.0 />
    };
}

#[test]
fn test_checkbox_widget() {
    let _view = view! {
        <Checkbox checked=true />
    };
}

#[test]
fn test_radio_widget() {
    let _view = view! {
        <Radio value="option1" checked=false />
    };
}

#[test]
fn test_show_widget() {
    let _view = view! {
        <Show when=true>
            <Text content="Visible" />
        </Show>
    };
}

#[test]
fn test_for_widget() {
    use rvue::prelude::*;
    let (items, _set_items) = create_signal(vec!["Item 1", "Item 2"]);
    let _view = view! {
        <For each=items key=|s| s.to_string() view={|s| view! {
            <Text content={s.to_string()} />
        }}/>
    };
}

#[test]
fn test_nested_elements() {
    let _build = || {
        view! {
            <Flex direction="column">
                <Text content="First" />
                <Button label="Click" />
                <Flex direction="row">
                    <Text content="Nested" />
                </Flex>
            </Flex>
        }
    };
}

#[test]
fn test_multiple_roots() {
    // Multiple root elements should be wrapped in a Flex container
    let _view = view! {
        <Text content="First" />
        <Text content="Second" />
    };
}

#[test]
fn test_dynamic_attributes() {
    let value = "dynamic";
    let _view = view! {
        <Text content={value} />
    };
}

// TODO: Fix event handler generation
// #[test]
// fn test_event_handlers() {
//     let _view = view! {
//         <Button label="Click" on_click=|| println!("clicked") />
//     };
// // }

#[test]
fn test_mixed_static_dynamic() {
    let dynamic_label = "Dynamic Label";
    let _view = view! {
        <Flex direction="column" gap=10.0>
            <Text content="Static" />
            <Text content={dynamic_label} />
            <Button label="Static Button" />
        </Flex>
    };
}

#[test]
fn test_empty_view() {
    let _view = view! {};
}
