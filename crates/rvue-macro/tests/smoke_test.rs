// Simple test of view! macro
// This is just for basic smoke testing of the macro

use rvue::prelude::*;
use rvue_macro::view;

fn main() {
    // This will be expanded by the macro
    // For now, we expect it to compile successfully
    let _view = view! {
        <Text value="Hello" />
    };
}
