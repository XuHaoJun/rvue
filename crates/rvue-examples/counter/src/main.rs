//! Counter example application with click event handling
#![allow(unused_braces)]

use rvue::prelude::*;
use rvue_macro::view;
#[allow(unused_imports)]
use rvue_style::{AlignItems, BackgroundColor, Color, JustifyContent, ReactiveStyles};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create counter component
    let counter_view = create_counter_view();

    // Run the application
    rvue::run_app(|| counter_view)?;

    Ok(())
}

fn increment_button_styles() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_align_items(AlignItems::Center)
        .set_justify_content(JustifyContent::Center)
        .set_background_color(BackgroundColor(Color::rgb(76, 175, 80)))
}

fn decrement_button_styles() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_align_items(AlignItems::Center)
        .set_justify_content(JustifyContent::Center)
        .set_background_color(BackgroundColor(Color::rgb(244, 67, 54)))
}

fn create_counter_view() -> ViewStruct {
    // Create a signal for the count
    let (count, set_count) = create_signal(0);

    // Create a signal for showing/hiding a message
    let (show_message, _set_show_message) = create_signal(true);

    let set_count_inc = set_count.clone();
    let set_count_dec = set_count;
    let count_inc = count.clone();
    let count_dec = count.clone();
    let count_label = create_memo(move || format!("Count: {}", count.get()));

    let view = view! {
        <Flex direction="column" gap=20.0 align_items="center" justify_content="center">
            <Text content=count_label />
            <Show when=show_message>
                <Text content="Counter is active!" />
            </Show>
            <Button styles=increment_button_styles() on_click={move || { println!("Increment clicked, current count: {}", count_inc.get()); set_count_inc.update(|x| *x += 1); }}>
                <Text content="+" />
            </Button>
            <Button styles=decrement_button_styles() on_click={move || { println!("Decrement clicked, current count: {}", count_dec.get()); set_count_dec.update(|x| *x -= 1); }}>
                <Text content="-" />
            </Button>
        </Flex>
    };

    view
}
