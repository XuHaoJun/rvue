//! Integration test for Flexbox layout

use rvue::layout::node::overflow_to_taffy;
use rvue::{Component, ComponentProps, ComponentType};
use rvue_style::properties::Overflow;

#[test]
fn test_flexbox_layout_creation() {
    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 10.0,
            align_items: "center".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    assert_eq!(flex.component_type, ComponentType::Flex);
    match &*flex.props.borrow() {
        ComponentProps::Flex { direction, gap, align_items, justify_content, .. } => {
            assert_eq!(direction, "row");
            assert_eq!(*gap, 10.0);
            assert_eq!(align_items, "center");
            assert_eq!(justify_content, "start");
        }
        _ => panic!("Expected Flex props"),
    };
}

#[test]
fn test_flexbox_nested_layouts() {
    let outer = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 20.0,
            align_items: "stretch".to_string(),
            justify_content: "center".to_string(),
            styles: None,
        },
    );

    let inner = Component::new(
        2,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 5.0,
            align_items: "center".to_string(),
            justify_content: "space-between".to_string(),
            styles: None,
        },
    );

    assert_eq!(outer.component_type, ComponentType::Flex);
    assert_eq!(inner.component_type, ComponentType::Flex);
}

#[test]
fn test_flexbox_spacing() {
    let flex = Component::new(
        1,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 15.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
            styles: None,
        },
    );

    match &*flex.props.borrow() {
        ComponentProps::Flex { gap, .. } => {
            assert_eq!(*gap, 15.0);
        }
        _ => panic!("Expected Flex props"),
    };
}

#[test]
fn test_flexbox_alignment() {
    let test_cases = vec![
        ("start", "center"),
        ("center", "end"),
        ("end", "start"),
        ("stretch", "space-between"),
    ];

    for (align_items, justify_content) in test_cases {
        let flex = Component::new(
            1,
            ComponentType::Flex,
            ComponentProps::Flex {
                direction: "row".to_string(),
                gap: 0.0,
                align_items: align_items.to_string(),
                justify_content: justify_content.to_string(),
                styles: None,
            },
        );

        match &*flex.props.borrow() {
            ComponentProps::Flex { align_items: ai, justify_content: jc, .. } => {
                assert_eq!(ai, align_items);
                assert_eq!(jc, justify_content);
            }
            _ => panic!("Expected Flex props"),
        };
    }
}

#[test]
fn test_show_conditional_rendering_with_flex_gap() {
    use rudo_gc::test_util::reset;
    use rvue::component::build_layout_tree;
    use rvue::prelude::*;
    use rvue::text::TextContext;
    use rvue_macro::view;
    use rvue_style::{Height, ReactiveStyles, Size, Width};
    use taffy::TaffyTree;

    reset();

    let (is_visible, set_visible) = create_signal(true);
    let is_visible_clone = is_visible.clone();

    let view: ViewStruct = view! {
        <Flex direction="column" gap=4.0 align_items="center">
            <Show when=is_visible>
                <Flex styles=ReactiveStyles::new()
                    .set_width(Width(Size::Pixels(100.0)))
                    .set_height(Height(Size::Pixels(100.0)))
                />
            </Show>
            <Show when=create_memo(move || !is_visible_clone.get())>
                <Flex styles=ReactiveStyles::new()
                    .set_width(Width(Size::Pixels(100.0)))
                    .set_height(Height(Size::Pixels(100.0)))
                />
            </Show>
        </Flex>
    };

    let root = view.into_component();

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();

    let children = root.children.borrow();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].component_type, ComponentType::Show);
    assert_eq!(children[1].component_type, ComponentType::Show);
    drop(children);

    let mut root_layout = build_layout_tree(&root, &mut taffy, &mut text_context, None);
    root_layout.calculate_layout(&mut taffy).unwrap();

    let children = root.children.borrow();
    let show1_when = if let ComponentProps::Show { when } = &*children[0].props.borrow() {
        *when
    } else {
        panic!("Expected Show props")
    };
    let show2_when = if let ComponentProps::Show { when } = &*children[1].props.borrow() {
        *when
    } else {
        panic!("Expected Show props")
    };
    assert!(show1_when, "Show1 should be visible initially");
    assert!(!show2_when, "Show2 should be hidden initially");

    {
        let show1_children = children[0].children.borrow();
        let show2_children = children[1].children.borrow();
        assert_eq!(show1_children.len(), 1, "Show1 should have 1 child");
        assert_eq!(show2_children.len(), 1, "Show2 should have 1 child");

        let blue_box_layout = show1_children[0].layout_node();
        let red_box_layout = show2_children[0].layout_node();
        assert!(blue_box_layout.is_some(), "Blue box should have layout node");
        assert!(red_box_layout.is_some(), "Red box should have layout node");
    }

    drop(children);

    set_visible.set(false);

    let children = root.children.borrow();
    let show1_when_after = children[0].show_when();
    let show2_when_after = children[1].show_when();
    assert!(!show1_when_after, "Show1 should be hidden after toggle");
    assert!(show2_when_after, "Show2 should be visible after toggle");
    drop(children);

    root.mark_dirty();
    let mut root_layout2 = build_layout_tree(&root, &mut taffy, &mut text_context, None);
    root_layout2.calculate_layout(&mut taffy).unwrap();

    let children = root.children.borrow();
    let show1_when_final = children[0].show_when();
    assert!(!show1_when_final, "Show1 should still be hidden after rebuild");
}

#[test]
fn test_overflow_to_taffy_visible() {
    let result = overflow_to_taffy(&Some(Overflow::Visible));
    assert_eq!(result.x, taffy::style::Overflow::Visible);
    assert_eq!(result.y, taffy::style::Overflow::Visible);
}

#[test]
fn test_overflow_to_taffy_auto() {
    let result = overflow_to_taffy(&Some(Overflow::Auto));
    assert_eq!(result.x, taffy::style::Overflow::Scroll);
    assert_eq!(result.y, taffy::style::Overflow::Scroll);
}

#[test]
fn test_overflow_to_taffy_scroll() {
    let result = overflow_to_taffy(&Some(Overflow::Scroll));
    assert_eq!(result.x, taffy::style::Overflow::Scroll);
    assert_eq!(result.y, taffy::style::Overflow::Scroll);
}

#[test]
fn test_overflow_to_taffy_none_returns_visible() {
    let result = overflow_to_taffy(&None);
    assert_eq!(result.x, taffy::style::Overflow::Visible);
    assert_eq!(result.y, taffy::style::Overflow::Visible);
}

#[test]
fn test_overflow_to_taffy_clip() {
    let result = overflow_to_taffy(&Some(Overflow::Clip));
    assert_eq!(result.x, taffy::style::Overflow::Clip);
    assert_eq!(result.y, taffy::style::Overflow::Clip);
}

#[test]
fn test_overflow_to_taffy_hidden() {
    let result = overflow_to_taffy(&Some(Overflow::Hidden));
    assert_eq!(result.x, taffy::style::Overflow::Hidden);
    assert_eq!(result.y, taffy::style::Overflow::Hidden);
}
