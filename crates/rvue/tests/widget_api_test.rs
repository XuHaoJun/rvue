//! Tests for the new Widget API and fine-grained updates

use rvue::{create_signal, text::TextContext, widget::*, widgets::*, Component, ComponentType};
use rvue_style::{AlignItems, FlexDirection, JustifyContent};
use taffy::TaffyTree;

/// Helper to create a BuildContext for testing
fn with_build_context<F, R>(f: F) -> R
where
    F: for<'a> FnOnce(&'a mut BuildContext<'a>) -> R,
{
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);
    f(&mut ctx)
}

#[test]
fn test_text_widget_builder() {
    with_build_context(|ctx| {
        // Test static text
        let widget = Text::new("Hello World");
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Text);
        // Use the new text_content getter
        assert_eq!(state.component().text_content(), "Hello World");
    });
}

#[test]
fn test_text_widget_with_signal() {
    let (text_signal, _set_text) = create_signal("Initial".to_string());

    with_build_context(|ctx| {
        // Test reactive text
        let widget = Text::new(text_signal);
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Text);
        // Initial value should be "Initial"
        assert_eq!(state.component().text_content(), "Initial");
    });
}

#[test]
fn test_button_widget_builder() {
    with_build_context(|ctx| {
        let widget = Button::new();
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Button);
        // Button has no text_content, verify via component_type
    });
}

#[test]
fn test_flex_widget_builder() {
    with_build_context(|ctx| {
        let widget = Flex::new()
            .direction(FlexDirection::Column)
            .gap(10.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween);

        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Flex);
    });
}

#[test]
fn test_checkbox_widget_builder() {
    with_build_context(|ctx| {
        let widget = Checkbox::new(true);
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Checkbox);
    });
}

#[test]
fn test_text_input_widget_builder() {
    with_build_context(|ctx| {
        let widget = TextInput::new("test input");
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::TextInput);
    });
}

#[test]
fn test_show_widget_builder() {
    with_build_context(|ctx| {
        let widget = Show::new(true, |_ctx| {
            let flex = Flex::new();
            let state = flex.build(_ctx);
            state.component().clone()
        });
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Show);
    });
}

#[test]
fn test_fine_grained_text_update() {
    with_build_context(|ctx| {
        // Create initial widget
        let widget = Text::new("Initial");
        let mut state = widget.build(ctx);

        // Update with new content
        let updated_widget = Text::new("Updated");
        updated_widget.rebuild(&mut state);

        // Verify the content was updated using new PropertyMap API
        let content = state.component().text_content();
        assert_eq!(content, "Updated");
    });
}

#[test]
fn test_fine_grained_button_update() {
    with_build_context(|ctx| {
        let widget = Button::new();
        let mut state = widget.build(ctx);

        let updated_widget = Button::new();
        updated_widget.rebuild(&mut state);

        assert_eq!(state.component().component_type, ComponentType::Button);
    });
}

#[test]
fn test_fine_grained_flex_update() {
    with_build_context(|ctx| {
        let widget = Flex::new().gap(5.0);
        let mut state = widget.build(ctx);

        let updated_widget = Flex::new().gap(20.0);
        updated_widget.rebuild(&mut state);

        // Verify the gap was updated using new PropertyMap API
        let gap = state.component().flex_gap();
        assert_eq!(gap, 20.0);
    });
}

#[test]
fn test_component_setters() {
    // Test the new property-specific setters
    let component =
        Component::with_properties(1, ComponentType::Text, rvue::properties::PropertyMap::new());

    // Test set_text_content using new PropertyMap API
    component.set_text_content("Updated".to_string());
    let content = component.text_content();
    assert_eq!(content, "Updated");
}

#[test]
fn test_reactive_value_static() {
    let value = ReactiveValue::Static("Hello".to_string());
    assert_eq!(value.get(), "Hello");
    assert!(!value.is_reactive());
}

#[test]
fn test_reactive_value_signal() {
    let (read, _write) = create_signal("Initial".to_string());
    let value = ReactiveValue::Signal(read.clone());

    assert_eq!(value.get(), "Initial");
    assert!(value.is_reactive());
}

#[test]
fn test_into_reactive_value_trait() {
    // Test that IntoReactiveValue works for String
    let value: ReactiveValue<String> = "Hello".to_string().into_reactive();
    assert_eq!(value.get(), "Hello");

    // Test for ReadSignal
    let (read, _write) = create_signal(42);
    let value: ReactiveValue<i32> = read.into_reactive();
    assert_eq!(value.get(), 42);
    assert!(value.is_reactive());
}
