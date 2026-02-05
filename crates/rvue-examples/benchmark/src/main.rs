//! Performance benchmark example - Counter app

use rvue::prelude::*;
use rvue::{Component, ComponentType, ViewStruct};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Measure startup time
    let start = Instant::now();

    // Create counter view
    let counter_view = create_counter_view();

    let init_time = start.elapsed();
    println!("Application initialization time: {}ms", init_time.as_millis());

    // Run the application
    rvue::run_app(|| counter_view)?;

    Ok(())
}

fn create_counter_view() -> ViewStruct {
    // Create a signal for the count
    let (count, _set_count) = create_signal(0);

    // Create root component (Flex container)
    let root =
        Component::with_properties(0, ComponentType::Flex, rvue::properties::PropertyMap::new());

    // Create text component to display count
    let _count_text = Component::with_properties(
        1,
        ComponentType::Text,
        rvue::properties::PropertyMap::with(rvue::properties::TextContent(format!(
            "Count: {}",
            count.get()
        ))),
    );

    // Create increment button
    let _inc_button =
        Component::with_properties(2, ComponentType::Button, rvue::properties::PropertyMap::new());

    // Create decrement button
    let _dec_button =
        Component::with_properties(3, ComponentType::Button, rvue::properties::PropertyMap::new());

    // Create view
    let view = ViewStruct::new(root);

    // Add effect to track count changes
    let _effect = create_effect({
        let count = count.clone();
        move || {
            let _ = count.get();
        }
    });

    view
}
