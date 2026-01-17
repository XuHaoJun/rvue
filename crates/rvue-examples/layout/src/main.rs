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
    let _header =
        Flex::new(1, FlexDirection::Row, 10.0, AlignItems::Center, JustifyContent::SpaceBetween);

    // Create header text
    let _header_text = Component::new(
        10,
        ComponentType::Text,
        ComponentProps::Text { content: "Rvue Layout Example".to_string() },
    );

    // Create main content area with nested layouts
    let _main_content =
        Flex::new(2, FlexDirection::Row, 15.0, AlignItems::Stretch, JustifyContent::Start);

    // Create sidebar
    let _sidebar =
        Flex::new(3, FlexDirection::Column, 5.0, AlignItems::Start, JustifyContent::Start);

    // Create sidebar items
    for i in 1..=5 {
        let _sidebar_item = Component::new(
            30 + i,
            ComponentType::Text,
            ComponentProps::Text { content: format!("Sidebar Item {}", i) },
        );
    }

    // Create content area
    let _content =
        Flex::new(4, FlexDirection::Column, 10.0, AlignItems::Start, JustifyContent::Start);

    // Create content items
    for i in 1..=3 {
        let _content_item = Component::new(
            40 + i,
            ComponentType::Text,
            ComponentProps::Text { content: format!("Content Section {}", i) },
        );
    }

    // Create footer
    let _footer = Flex::new(5, FlexDirection::Row, 5.0, AlignItems::Center, JustifyContent::Center);

    let _footer_text = Component::new(
        50,
        ComponentType::Text,
        ComponentProps::Text { content: "Footer".to_string() },
    );

    // Note: In a full implementation, we would:
    // 1. Use the view! macro to create components declaratively
    // 2. Add children to parent components
    // 3. Use effects to update layout when content changes
    // 4. Apply Taffy layout results to Vello scene positions
    // For MVP, this demonstrates the basic structure

    // Create view
    let view = ViewStruct::new(root);

    view
}
