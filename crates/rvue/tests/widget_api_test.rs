//! Tests for the new Widget API and fine-grained updates

use rvue::{
    create_signal, text::TextContext, widget::*, widgets::*, Component, ComponentProps,
    ComponentType,
};
use rvue_style::{AlignItems, FlexDirection, Gap, JustifyContent};
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
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Text { content, .. } => {
                    assert_eq!(content, "Hello World");
                }
                _ => panic!("Expected Text props"),
            }
        }
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
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Text { content, .. } => {
                    assert_eq!(content, "Initial");
                }
                _ => panic!("Expected Text props"),
            }
        }
    });
}

#[test]
fn test_button_widget_builder() {
    with_build_context(|ctx| {
        let widget = Button::new("Click Me");
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Button);
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Button { label } => {
                    assert_eq!(label, "Click Me");
                }
                _ => panic!("Expected Button props"),
            }
        }
    });
}

#[test]
fn test_flex_widget_builder() {
    with_build_context(|ctx| {
        let widget = Flex::new()
            .direction(FlexDirection::Column)
            .gap(Gap(10.0))
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween);

        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Flex);
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Flex { direction, gap, .. } => {
                    assert_eq!(direction, "column");
                    assert_eq!(*gap, 10.0);
                }
                _ => panic!("Expected Flex props"),
            }
        }
    });
}

#[test]
fn test_checkbox_widget_builder() {
    with_build_context(|ctx| {
        let widget = Checkbox::new(true);
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::Checkbox);
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Checkbox { checked } => {
                    assert!(*checked);
                }
                _ => panic!("Expected Checkbox props"),
            }
        }
    });
}

#[test]
fn test_text_input_widget_builder() {
    with_build_context(|ctx| {
        let widget = TextInput::new("test input");
        let state = widget.build(ctx);

        assert_eq!(state.component().component_type, ComponentType::TextInput);
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::TextInput { value } => {
                    assert_eq!(value, "test input");
                }
                _ => panic!("Expected TextInput props"),
            }
        }
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
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Show { when } => {
                    assert!(*when);
                }
                _ => panic!("Expected Show props"),
            }
        }
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

        // Verify the content was updated
        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Text { content, .. } => {
                    assert_eq!(content, "Updated");
                }
                _ => panic!("Expected Text props"),
            }
        }
    });
}

#[test]
fn test_fine_grained_button_update() {
    with_build_context(|ctx| {
        let widget = Button::new("Initial Label");
        let mut state = widget.build(ctx);

        let updated_widget = Button::new("Updated Label");
        updated_widget.rebuild(&mut state);

        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Button { label } => {
                    assert_eq!(label, "Updated Label");
                }
                _ => panic!("Expected Button props"),
            }
        }
    });
}

#[test]
fn test_fine_grained_flex_update() {
    with_build_context(|ctx| {
        let widget = Flex::new().gap(Gap(5.0));
        let mut state = widget.build(ctx);

        let updated_widget = Flex::new().gap(Gap(20.0));
        updated_widget.rebuild(&mut state);

        {
            let props = state.component().props.borrow();
            match &*props {
                ComponentProps::Flex { gap, .. } => {
                    assert_eq!(*gap, 20.0);
                }
                _ => panic!("Expected Flex props"),
            }
        }
    });
}

#[test]
fn test_component_setters() {
    // Test the new property-specific setters
    let component = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: "Initial".to_string(), styles: None },
    );

    // Test set_text_content
    component.set_text_content("Updated".to_string());
    match &*component.props.borrow() {
        ComponentProps::Text { content, .. } => {
            assert_eq!(content, "Updated");
        }
        _ => panic!("Expected Text props"),
    };
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
