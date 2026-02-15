//! Test For with nested structure - 5 items

use rudo_gc::{Trace, Visitor};
use rvue::impl_gc_capture;
use rvue::prelude::*;
use rvue_macro::view;

#[derive(Clone)]
struct Story {
    id: i64,
    title: String,
}

unsafe impl Trace for Story {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

impl_gc_capture!(Story);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_view = create_test_view();
    rvue::run_app(|| app_view)?;
    Ok(())
}

fn create_test_view() -> ViewStruct {
    let stories = vec![
        Story { id: 1, title: "First Story".to_string() },
        Story { id: 2, title: "Second Story".to_string() },
        Story { id: 3, title: "Third Story".to_string() },
        Story { id: 4, title: "Fourth Story".to_string() },
        Story { id: 5, title: "Fifth Story".to_string() },
    ];

    let (todos, _) = create_signal(stories);

    view! {
        <Flex direction="column" gap=12.0 padding=16.0>
            <Text content="Test" font_size=24.0 />
            <For each=todos key=|s: &Story| s.id view={|s| view! {
                <Flex direction="row" gap=4.0 padding=8.0 background_color="#f8f9fa" border_radius=8.0>
                    <Flex>
                        <Text content=s.title.clone() font_size=15.0 />
                    </Flex>
                    <Flex>
                        <Text content=format!("ID: {}", s.id) font_size=12.0 color="#666" />
                    </Flex>
                </Flex>
            }}/>
        </Flex>
    }
}
