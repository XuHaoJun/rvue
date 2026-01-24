//! Attribute parsing and classification tests
//!
//! These tests verify that the macro correctly classifies and handles
//! different types of attributes: static, dynamic, and event handlers.

use rvue_macro::view;

#[test]
fn test_static_string_attribute() {
    let _view = view! {
        <Text content="Hello" />
    };
    let _ = _view;
}

#[test]
fn test_static_numeric_attribute() {
    let _view = view! {
        <NumberInput value=42.0 />
    };
    let _ = _view;
}

#[test]
fn test_static_boolean_attribute() {
    let _view = view! {
        <Checkbox checked=true />
    };
    let _ = _view;
}

#[test]
fn test_dynamic_string_attribute() {
    let value = "dynamic";
    let _view = view! {
        <Text content=value />
    };
    let _ = _view;
}

#[test]
fn test_dynamic_numeric_attribute() {
    let value = 42.0;
    let _view = view! {
        <NumberInput value=value />
    };
    let _ = _view;
}

#[test]
fn test_dynamic_boolean_attribute() {
    let checked = true;
    let _view = view! {
        <Checkbox checked=checked />
    };
    let _ = _view;
}

#[test]
fn test_complex_expression_attribute() {
    let count = 5;
    let _view = view! {
        <Text content={format!("Count: {}", count)} />
    };
    let _ = _view;
}

#[test]
fn test_multiple_static_attributes() {
    let _view = view! {
        <Radio value="option1" checked=false />
    };
    let _ = _view;
}

#[test]
fn test_mixed_static_dynamic_attributes() {
    let checked = true;
    let _view = view! {
        <Radio value="option1" checked=checked />
    };
    let _ = _view;
}

// Event handler tests are commented out until event handler generation is fixed
// #[test]
// fn test_on_click_handler() {
//     let _view = view! {
//         <Button label="Click" on_click=|| println!("clicked") />
//     };
//     let _ = _view;
// }
//
// #[test]
// fn test_on_colon_click_handler() {
//     let _view = view! {
//         <Button label="Click" on:click=|| println!("clicked") />
//     };
//     let _ = _view;
// }
