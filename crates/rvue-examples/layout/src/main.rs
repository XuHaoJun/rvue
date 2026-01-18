//! Complex nested flexbox layout example

use rvue::{
    AlignItems, Component, ComponentProps, ComponentType, Flex, FlexDirection, JustifyContent,
    ViewStruct,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create layout view
    let layout_view = create_layout_view();

    // Run the application
    rvue::run_app(|| layout_view)?;

    Ok(())
}

fn create_layout_view() -> ViewStruct {
    // Create root component with column layout
    let root = Flex::new(
        0,
        FlexDirection::Column,
        20.0, // gap
        AlignItems::Center,
        JustifyContent::Start,
    );

    // Create header section
    let header =
        Flex::new(1, FlexDirection::Row, 10.0, AlignItems::Center, JustifyContent::SpaceBetween);

    // Create header text
    let header_text = Component::new(
        10,
        ComponentType::Text,
        ComponentProps::Text {
            content: "Rvue Layout Example".to_string(),
            font_size: None,
            color: None,
        },
    );
    header.add_child(header_text);

    root.add_child(header);

    // Create main content area with nested layouts
    let main_content =
        Flex::new(2, FlexDirection::Row, 15.0, AlignItems::Stretch, JustifyContent::Start);

    // Create sidebar
    let sidebar =
        Flex::new(3, FlexDirection::Column, 5.0, AlignItems::Start, JustifyContent::Start);

    // Create sidebar items
    for i in 1..=5 {
        let sidebar_item = Component::new(
            30 + i,
            ComponentType::Text,
            ComponentProps::Text {
                content: format!("Sidebar Item {}", i),
                font_size: None,
                color: None,
            },
        );
        sidebar.add_child(sidebar_item);
    }

    main_content.add_child(sidebar);

    // Create content area
    let content =
        Flex::new(4, FlexDirection::Column, 10.0, AlignItems::Start, JustifyContent::Start);

    // Create content items
    for i in 1..=3 {
        let content_item = Component::new(
            40 + i,
            ComponentType::Text,
            ComponentProps::Text {
                content: format!("Content Section {}", i),
                font_size: None,
                color: None,
            },
        );
        content.add_child(content_item);
    }

    main_content.add_child(content);

    root.add_child(main_content);

    // Create footer
    let footer = Flex::new(5, FlexDirection::Row, 5.0, AlignItems::Center, JustifyContent::Center);

    let footer_text = Component::new(
        50,
        ComponentType::Text,
        ComponentProps::Text { content: "Footer".to_string(), font_size: None, color: None },
    );
    footer.add_child(footer_text);

    root.add_child(footer);

    // Create view
    let view = ViewStruct::new(root);

    view
}
