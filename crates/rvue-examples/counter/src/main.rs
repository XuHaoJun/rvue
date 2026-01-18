//! Counter example application with conditional rendering

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

    // Create increment handler
    // Note: Event handlers will be connected when full event system is implemented
    let _increment = {
        let set_count = set_count.clone();
        move || {
            set_count.update(|x| *x += 1);
        }
    };

    // Create decrement handler
    let _decrement = {
        let set_count = set_count.clone();
        move || {
            set_count.update(|x| *x -= 1);
        }
    };

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
    // For MVP, we'll create a simple text component
    // In a full implementation, this would be reactive via effects
    let count_text = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: format!("Count: {}", count.get()) },
    );

    // Create a message that can be shown/hidden
    let message_text = Component::new(
        2,
        ComponentType::Text,
        ComponentProps::Text { content: "Counter is active!".to_string() },
    );

    // Create Show component to conditionally display the message
    // Note: In a full implementation, this would be connected to show_message signal
    let show_component = Show::new(3, show_message.get());
    show_component.add_child(message_text);

    // Create increment button
    let inc_button =
        Component::new(4, ComponentType::Button, ComponentProps::Button { label: "+".to_string() });

    // Create decrement button
    let dec_button =
        Component::new(5, ComponentType::Button, ComponentProps::Button { label: "-".to_string() });

    // Add components to the root component's children
    root.add_child(count_text);
    root.add_child(show_component);
    root.add_child(inc_button);
    root.add_child(dec_button);

    // Create view
    let view = ViewStruct::new(root);

    // Add effect to update text when count changes
    let _effect = create_effect({
        let count = count.clone();
        move || {
            let _ = count.get(); // Track the signal
                                 // In a full implementation, this would update the text component
            println!("Count changed to: {}", count.get());
        }
    });

    // Add effect to update Show component when show_message changes
    let _show_effect = create_effect({
        let show_message = show_message.clone();
        move || {
            let _ = show_message.get(); // Track the signal
            println!("Show message changed to: {}", show_message.get());
        }
    });

    view
}
