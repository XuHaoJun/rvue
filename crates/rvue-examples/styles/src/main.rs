//! Minimal styling example to debug stack overflow

use rvue::prelude::*;
use rvue_macro::view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let styled_view = create_styled_view();
    rvue::run_app(|| styled_view)?;
    Ok(())
}

fn create_styled_view() -> ViewStruct {
    view! {
        <Flex direction="column" gap=20.0 align_items="center" justify_content="start">
            <Text content="Simple Text" font_size=24.0 />
        </Flex>
    }
}
