//! Counter example application with click event handling

use rvue::prelude::*;
use rvue_macro::view;

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

    let set_count_inc = set_count.clone();
    let set_count_dec = set_count;
    let count_inc = count.clone();
    let count_dec = count.clone();
    let count_label = create_memo(move || format!("Count: {}", count.get()));

    let view = view! {
        <Flex direction="column" gap=20.0 align_items="center" justify_content="center">
            <Text content=count_label />
            <Show when=show_message.get()>
                <Text content="Counter is active!" />
            </Show>
            <Button label="+" on_click={move || { println!("Increment clicked, current count: {}", count_inc.get()); set_count_inc.update(|x| *x += 1); }} />
            <Button label="-" on_click={move || { println!("Decrement clicked, current count: {}", count_dec.get()); set_count_dec.update(|x| *x -= 1); }} />
        </Flex>
    };

    view
}
