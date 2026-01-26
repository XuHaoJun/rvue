//! Slot example demonstrating the slot mechanism
//!
//! This example shows how to define slots and use them in components.

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue_macro::{component, slot, view};

#[slot]
struct CardBody {
    children: ChildrenFn,
}

#[component]
fn Card(body: CardBody) -> impl View {
    let body_view = (body.children.0)();
    CardInner { view: body_view }
}

struct CardInner {
    view: ViewStruct,
}

impl View for CardInner {
    fn into_component(self) -> Gc<Component> {
        let root = Component::new(
            0,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "column".to_string(),
                gap: 10.0,
                align_items: "stretch".to_string(),
                justify_content: "start".to_string(),
            },
        );

        let header = Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text {
                content: "Card Header".to_string(),
                font_size: None,
                color: None,
            },
        );
        let footer = Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text {
                content: "Card Footer".to_string(),
                font_size: None,
                color: None,
            },
        );

        root.add_child(header);
        root.add_child(self.view.root_component.clone());
        root.add_child(footer);

        root
    }
}

#[component]
fn App() -> impl View {
    let body1: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Text,
            ComponentProps::Text {
                content: "This is the card body content.".to_string(),
                font_size: None,
                color: None,
            },
        ))
    })
    .to_children();

    let body2: ChildrenFn = (|| {
        ViewStruct::from_component(Component::new(
            0,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "row".to_string(),
                gap: 5.0,
                align_items: "start".to_string(),
                justify_content: "start".to_string(),
            },
        ))
    })
    .to_children();

    view! {
        <Flex direction="column" gap=20.0>
            <Text content="Slot Example" />

            <Card body=CardBody { children: body1 } />
            <Card body=CardBody { children: body2 } />
        </Flex>
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_view = App(Default::default());
    let app_component = app_view.into_component();
    let app_view_struct = ViewStruct::new(app_component);
    rvue::run_app(|| app_view_struct)?;
    Ok(())
}
