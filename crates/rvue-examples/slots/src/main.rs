//! Slot example demonstrating the slot mechanism
//!
//! This example shows how to define slots and use them in components.

use rvue::prelude::*;
use rvue_macro::{component, slot, view};
use std::sync::Arc;

#[slot]
struct CardBody {
    children: ChildrenFn,
}

#[component]
fn Card(body: CardBody) -> impl View {
    let body_view = (body.children)();
    view! {
        <Flex direction="column" gap=10.0>
            <Text content="Card Header" />
            { body_view }
            <Text content="Card Footer" />
        </Flex>
    }
}

#[component]
fn App() -> impl View {
    let body_content1: ChildrenFn = Arc::new(|| {
        ViewStruct::new(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text {
                content: "This is the card body content.".to_string(),
                font_size: None,
                color: None,
            },
        ))
    });

    let body_content2: ChildrenFn = Arc::new(|| {
        ViewStruct::new(Component::new(
            0,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "row".to_string(),
                gap: 5.0,
                align_items: "start".to_string(),
                justify_content: "start".to_string(),
            },
        ))
    });

    view! {
        <Flex direction="column" gap=20.0>
            <Text content="Slot Example" />

            <Card
                body=CardBody { children: body_content1 }
            />

            <Card
                body=CardBody { children: body_content2 }
            />
        </Flex>
    }
}

fn create_app_view() -> ViewStruct {
    App()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_view = create_app_view();
    rvue::run_app(|| app_view)?;
    Ok(())
}
