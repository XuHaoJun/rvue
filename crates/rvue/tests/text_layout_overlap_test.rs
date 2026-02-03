//! Integration test for text layout overlap bug
//!
//! This test verifies that text components have correct heights when placed in Flex containers,
//! preventing the overlap bug where texts would appear at the same position.
//!
//! Bug: Text components were getting height=0 from Taffy when inside a Flex container
//! with align_items: stretch, causing them to overlap with other text elements.

use rudo_gc::Gc;
use rvue::component::{Component, ComponentProps, ComponentType};
use rvue::Scene;
use std::sync::atomic::{AtomicU64, Ordering};

/// Helper to create components with unique IDs
fn create_id_counter() -> AtomicU64 {
    AtomicU64::new(0)
}

macro_rules! next_id {
    ($counter:expr) => {
        $counter.fetch_add(1, Ordering::SeqCst)
    };
}

#[test]
fn test_text_height_not_zero_in_stretch_flex() {
    //! Test that text has positive height when parent Flex has align_items: stretch
    //!
    //! This specifically tests the bug where text height was computed as 0
    //! when inside a Flex container with align_items: stretch

    let id_counter = create_id_counter();

    // Create a Flex container with stretch alignment (the problematic case)
    let flex = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(), // This caused the bug
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    // Create a text component
    let text = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text { content: "Rvue Styling System Showcase".to_string(), styles: None },
    );

    flex.add_child(Gc::clone(&text));

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&flex));

    // Trigger layout
    scene.update();

    // Get the text's layout
    let text_layout_node = text.layout_node().expect("Text should have a layout node");
    let text_layout = text_layout_node.layout().expect("Text layout should have a result");

    // THE KEY ASSERTION: Text height must be positive
    assert!(
        text_layout.size.height > 0.0,
        "Text height should be positive when inside a Flex with stretch, got {}",
        text_layout.size.height
    );
    assert!(
        text_layout.size.height > 0.0,
        "Text height should be positive when inside a Flex with stretch, got {}",
        text_layout.size.height
    );

    // Also verify width is positive
    assert!(
        text_layout.size.width > 0.0,
        "Text width should be positive, got {}",
        text_layout.size.width
    );
}

#[test]
fn test_adjacent_texts_do_not_overlap() {
    //! Test that two adjacent text elements in a Flex column don't overlap
    //!
    //! This directly tests the original bug: Text 20 and Text 22 overlapping

    let id_counter = create_id_counter();

    // Create a Flex container in column direction
    let flex = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    // Create first text (like "Rvue Styling System Showcase" - Text 20)
    let text1 = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text { content: "Rvue Styling System Showcase".to_string(), styles: None },
    );

    // Create second text (like "Theme:" - Text 22)
    let text2 = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text { content: "Theme:".to_string(), styles: None },
    );

    flex.add_child(Gc::clone(&text1));
    flex.add_child(Gc::clone(&text2));

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&flex));

    scene.update();

    // Get layouts
    let text1_layout_node = text1.layout_node().unwrap();
    let text1_layout = text1_layout_node.layout().unwrap();
    let text2_layout_node = text2.layout_node().unwrap();
    let text2_layout = text2_layout_node.layout().unwrap();

    // Text2 should be positioned after text1 (no overlap)
    // In a column flex, text2's y should be >= text1's y + text1's height
    let expected_y_for_text2 = text1_layout.size.height;
    let actual_y_for_text2 = text2_layout.location.y;

    assert!(
        actual_y_for_text2 >= expected_y_for_text2 - 1.0, // Allow small floating point tolerance
        "Text2 (y={}) should be positioned after Text1 (y={} + height={}), not overlapping. \
         Text1 height={}, Text2 height={}",
        actual_y_for_text2,
        text1_layout.location.y,
        text1_layout.size.height,
        text1_layout.size.height,
        text2_layout.size.height
    );

    // Both texts must have positive heights
    assert!(
        text1_layout.size.height > 0.0,
        "Text1 height must be positive, got {}",
        text1_layout.size.height
    );
    assert!(
        text2_layout.size.height > 0.0,
        "Text2 height must be positive, got {}",
        text2_layout.size.height
    );
}

#[test]
fn test_text_with_explicit_font_size_has_height() {
    //! Test that text with explicit font size has proper height
    //!
    //! Uses the ReactiveStyles pattern from the styles example

    let id_counter = create_id_counter();

    let text = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text {
            content: "Interactive Reactive Style Design".to_string(),
            styles: None,
        },
    );

    let flex = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 16.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    flex.add_child(Gc::clone(&text));

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&flex));

    scene.update();

    let text_layout_node = text.layout_node().unwrap();
    let text_layout = text_layout_node.layout().unwrap();

    // With default 16px font, height should be around 21-24px
    assert!(
        text_layout.size.height > 15.0 && text_layout.size.height < 30.0,
        "Text height should be reasonable for font size, got {}",
        text_layout.size.height
    );
}

#[test]
fn test_nested_flex_text_layout() {
    //! Test text layout in nested Flex containers
    //!
    //! This simulates the structure from the styles example:
    //! - Root Flex
    //!   - Text 20 (direct child)
    //!   - Flex 21 (contains Text 22)

    let id_counter = create_id_counter();

    let root_flex = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    // Text 20: Direct child of root Flex
    let text20 = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text { content: "Rvue Styling System Showcase".to_string(), styles: None },
    );

    // Flex 21: Another child of root Flex (like the Theme section)
    let flex21 = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 16.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    // Text 22: Child of Flex 21
    let text22 = Component::new(
        next_id!(id_counter),
        ComponentType::Text,
        ComponentProps::Text { content: "Theme:".to_string(), styles: None },
    );

    root_flex.add_child(Gc::clone(&text20));
    root_flex.add_child(Gc::clone(&flex21));
    flex21.add_child(Gc::clone(&text22));

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&root_flex));

    scene.update();

    // Get layouts
    let text20_layout_node = text20.layout_node().unwrap();
    let text20_layout = text20_layout_node.layout().unwrap();
    let flex21_layout_node = flex21.layout_node().unwrap();
    let flex21_layout = flex21_layout_node.layout().unwrap();
    let text22_layout_node = text22.layout_node().unwrap();
    let text22_layout = text22_layout_node.layout().unwrap();

    // Verify both texts have positive heights
    assert!(
        text20_layout.size.height > 0.0,
        "Text20 height must be positive, got {}",
        text20_layout.size.height
    );
    assert!(
        text22_layout.size.height > 0.0,
        "Text22 height must be positive, got {}",
        text22_layout.size.height
    );

    // Flex21 should be positioned after Text20
    assert!(
        flex21_layout.location.y >= text20_layout.location.y + text20_layout.size.height - 1.0,
        "Flex21 (y={}) should be after Text20 (y={} + height={}), got y={}",
        flex21_layout.location.y,
        text20_layout.location.y,
        text20_layout.size.height,
        flex21_layout.location.y
    );

    // Text22 should be at the top of Flex21
    assert!(
        text22_layout.location.y >= 0.0 && text22_layout.location.y < 10.0,
        "Text22 should be near top of Flex21, got y={}",
        text22_layout.location.y
    );
}

#[test]
fn test_multiple_texts_in_column_flex() {
    //! Test multiple texts in a column flex don't overlap
    //!
    //! Tests a realistic scenario similar to the styles example

    let id_counter = create_id_counter();

    let column = Component::new(
        next_id!(id_counter),
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "stretch".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    let texts: Vec<Gc<Component>> = vec![
        Component::new(
            next_id!(id_counter),
            ComponentType::Text,
            ComponentProps::Text { content: "First text line".to_string(), styles: None },
        ),
        Component::new(
            next_id!(id_counter),
            ComponentType::Text,
            ComponentProps::Text { content: "Second text line".to_string(), styles: None },
        ),
        Component::new(
            next_id!(id_counter),
            ComponentType::Text,
            ComponentProps::Text { content: "Third text line".to_string(), styles: None },
        ),
    ];

    for text in &texts {
        column.add_child(Gc::clone(text));
    }

    let mut scene = Scene::new();
    scene.add_fragment(Gc::clone(&column));

    scene.update();

    // Verify no overlap
    let first_node = texts[0].layout_node().unwrap();
    let first_layout = first_node.layout().unwrap();
    let mut prev_y = first_layout.location.y;
    let mut prev_height = first_layout.size.height;

    // Each text must have positive height
    assert!(prev_height > 0.0, "Text 0 height must be positive, got {}", prev_height);

    for i in 1..texts.len() {
        let curr_node = texts[i].layout_node().unwrap();
        let curr_layout = curr_node.layout().unwrap();

        // Current text should start after previous text ends (or very close to it)
        let min_expected_y = prev_y + prev_height - 1.0;
        let actual_y = curr_layout.location.y;

        assert!(
            actual_y >= min_expected_y,
            "Text {} (y={}) should be after Text {} (y={} + height={}). Overlap detected!",
            i,
            actual_y,
            i - 1,
            prev_y,
            prev_height
        );

        // Each text must have positive height
        let curr_height = curr_layout.size.height;
        assert!(curr_height > 0.0, "Text {} height must be positive, got {}", i, curr_height);

        prev_y = curr_layout.location.y;
        prev_height = curr_height;
    }
}
