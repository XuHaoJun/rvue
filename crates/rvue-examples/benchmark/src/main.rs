//! Performance benchmark example - Counter app

use rvue::prelude::*;
use rvue::{Component, ComponentProps, ComponentType, ViewStruct};
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
    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 20.0,
            align_items: "center".to_string(),
            justify_content: "center".to_string(),
            styles: None,
        },
    );

    // Create text component to display count
    let _count_text = Component::new(
        1,
        ComponentType::Text,
        ComponentProps::Text { content: format!("Count: {}", count.get()), styles: None },
    );

    // Create increment button
    let _inc_button =
        Component::new(2, ComponentType::Button, ComponentProps::Button { styles: None });

    // Create decrement button
    let _dec_button =
        Component::new(3, ComponentType::Button, ComponentProps::Button { styles: None });

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
