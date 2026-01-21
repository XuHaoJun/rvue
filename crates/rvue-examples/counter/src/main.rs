//! Counter example application with click event handling

use rvue::event::context::EventContext;
use rvue::event::types::PointerButtonEvent;
use rvue::prelude::*;
use rvue::{Component, ComponentProps, ComponentType, Show, ViewStruct};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create counter component
    let counter_view = create_counter_view();

    // Run the application
    rvue::run_app(|| counter_view)?;

    Ok(())
}

fn create_counter_view() -> ViewStruct {
    // Create a signal for the count
    let (count, set_count) = create_signal(0);

    // Create a signal for showing/hiding a message
    let (show_message, _set_show_message) = create_signal(true);

    // Create root component (Flex container)
    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 20.0,
            align_items: "center".to_string(),
            justify_content: "center".to_string(),
        },
    );

    // Create text component to display count
    let count_text = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text {
            content: format!("Count: {}", count.get()),
            font_size: None,
            color: None,
        },
    );

    // Add effect to update text when count changes
    let count_comp = count_text.clone();
    let count_read = count.clone();
    let _count_effect = create_effect(move || {
        let val = count_read.get();
        *count_comp.props.borrow_mut() = ComponentProps::Text {
            content: format!("Count: {}", val),
            font_size: None,
            color: None,
        };
        count_comp.mark_dirty();
    });

    // Create a message that can be shown/hidden
    let message_text = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text {
            content: "Counter is active!".to_string(),
            font_size: None,
            color: None,
        },
    );

    // Create Show component to conditionally display the message
    let show_component = Show::new(3, show_message.get());
    show_component.add_child(message_text);

    // Add effect to update Show component when show_message changes
    let show_comp = show_component.clone();
    let show_read = show_message.clone();
    let _show_effect = create_effect(move || {
        let when = show_read.get();
        *show_comp.props.borrow_mut() = ComponentProps::Show { when };
        show_comp.mark_dirty();
    });

    // Create increment button
    let inc_button =
        Component::new(4, ComponentType::Button, ComponentProps::Button { label: "+".to_string() });

    // Register click handler for increment button
    let set_count_inc = set_count.clone();
    inc_button.on_click(move |_event: &PointerButtonEvent, _ctx: &mut EventContext| {
        println!("Increment button clicked");
        set_count_inc.update(|x| *x += 1);
    });

    // Create decrement button
    let dec_button =
        Component::new(5, ComponentType::Button, ComponentProps::Button { label: "-".to_string() });

    // Register click handler for decrement button
    let set_count_dec = set_count;
    dec_button.on_click(move |_event: &PointerButtonEvent, _ctx: &mut EventContext| {
        println!("Decrement button clicked");
        set_count_dec.update(|x| *x -= 1);
    });

    // Add components to the root component's children
    root.add_child(count_text);
    root.add_child(show_component);
    root.add_child(inc_button);
    root.add_child(dec_button);

    // Create view
    let view = ViewStruct::new(root);

    view
}
