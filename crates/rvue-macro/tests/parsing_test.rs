//! RSX parsing tests for view! macro
//!
//! These tests verify that the macro correctly parses various RSX syntax patterns.

use rvue_macro::view;

#[test]
fn test_self_closing_tag() {
    let _view = view! {
        <Text content="Hello" />
    };
    let _ = _view;
}

#[test]
fn test_open_close_tag() {
    let _view = view! {
        <Text>"Content"</Text>
    };
    let _ = _view;
}

#[test]
fn test_nested_elements() {
    let _view = view! {
        <Flex direction="column">
            <Text content="Child 1" />
            <Text content="Child 2" />
        </Flex>
    };
    let _ = _view;
}

#[test]
fn test_multiple_attributes() {
    let _view = view! {
        <Flex direction="column" gap=10.0 align_items="center" />
    };
    let _ = _view;
}

#[test]
fn test_text_content_vs_attribute() {
    // Text can be specified as attribute or children
    let _view1 = view! {
        <Text content="Attribute" />
    };
    let _view2 = view! {
        <Text>"Children"</Text>
    };
    let _ = (_view1, _view2);
}

#[test]
fn test_empty_element() {
    let _view = view! {
        <Flex direction="column" />
    };
    let _ = _view;
}

#[test]
fn test_deeply_nested() {
    let _view = view! {
        <Flex direction="column">
            <Flex direction="row">
                <Text content="Nested" />
            </Flex>
        </Flex>
    };
    let _ = _view;
}

#[test]
fn test_block_expressions() {
    let value = "dynamic";
    let _view = view! {
        <Text content={value} />
    };
    let _ = _view;
}

#[test]
fn test_mixed_static_dynamic() {
    let dynamic = "dynamic";
    let _view = view! {
        <Flex direction="column" gap=10.0>
            <Text content="static" />
            <Text content={dynamic} />
        </Flex>
    };
    let _ = _view;
}
