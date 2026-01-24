//! Complex nested flexbox layout example

use rvue::prelude::*;
use rvue_macro::view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create layout view
    let layout_view = create_layout_view();

    // Run the application
    rvue::run_app(|| layout_view)?;

    Ok(())
}

fn create_layout_view() -> ViewStruct {
    view! {
        <Flex direction="column" gap=20.0 align_items="center" justify_content="start">
            <Flex direction="row" gap=10.0 align_items="center" justify_content="space-between">
                <Text content="Rvue Layout Example" />
            </Flex>

            <Flex direction="row" gap=15.0 align_items="stretch" justify_content="start">
                <Flex direction="column" gap=5.0 align_items="start" justify_content="start">
                    <Text content="Sidebar Item 1" />
                    <Text content="Sidebar Item 2" />
                    <Text content="Sidebar Item 3" />
                    <Text content="Sidebar Item 4" />
                    <Text content="Sidebar Item 5" />
                </Flex>

                <Flex direction="column" gap=10.0 align_items="start" justify_content="start">
                    <Text content="Content Section 1" />
                    <Text content="Content Section 2" />
                    <Text content="Content Section 3" />
                </Flex>
            </Flex>

            <Flex direction="row" gap=5.0 align_items="center" justify_content="center">
                <Text content="Footer" />
            </Flex>
        </Flex>
    }
}
