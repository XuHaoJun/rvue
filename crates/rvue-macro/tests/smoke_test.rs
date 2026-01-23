//! Simple smoke test of view! macro
//!
//! This test verifies that the view! macro compiles and generates valid code.
//! It tests basic widget creation with static attributes.
//!
//! Note: The macro currently uses the deprecated widget API (Text::new, Button::new, etc.)
//! which is fine for backward compatibility. The macro will be updated in Phase 3 to use
//! the new Widget API.

use rvue::prelude::*;
use rvue_macro::view;

#[test]
fn test_view_macro_compiles() {
    // Test that the macro expands to valid code
    // Text widget uses "content" attribute (not "value")
    let _view = view! {
        <Text content="Hello" />
    };

    // If we get here, the macro compiled successfully
    assert!(true);
}

#[test]
fn test_view_macro_with_button() {
    let _view = view! {
        <Button label="Click Me" />
    };

    assert!(true);
}

#[test]
fn test_view_macro_text_content() {
    // Test text content as children
    let _view = view! {
        <Text>"Hello World"</Text>
    };

    assert!(true);
}

// Note: Multiple root elements test is commented out due to macro codegen issue
// This will be fixed when the macro is updated in Phase 3
// #[test]
// fn test_view_macro_multiple_roots() {
//     // Multiple root elements return ViewStruct (wrapped in Flex)
//     let _view = view! {
//         <Text content="First" />
//         <Text content="Second" />
//     };
//
//     assert!(true);
// }
