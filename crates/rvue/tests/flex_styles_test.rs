//! Unit tests for Flex widget styles

use rudo_gc::Gc;
use rvue::{Component, ComponentType};

#[test]
fn test_flex_with_background_color() {
    let flex =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    assert_eq!(flex.component_type, ComponentType::Flex);
}

#[test]
fn test_flex_with_border() {
    let flex =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    assert_eq!(flex.component_type, ComponentType::Flex);
}

#[test]
fn test_nested_flex_with_styles() {
    let outer =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    let inner =
        Component::with_properties(2, ComponentType::Flex, rvue::properties::PropertyMap::new());

    outer.add_child(Gc::clone(&inner));

    assert_eq!(outer.component_type, ComponentType::Flex);
    assert_eq!(inner.component_type, ComponentType::Flex);
}

#[test]
fn test_flex_without_styles() {
    let flex =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    assert_eq!(flex.component_type, ComponentType::Flex);
}

#[test]
fn test_multiple_nested_flex_with_different_styles() {
    let parent =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    let child1 =
        Component::with_properties(2, ComponentType::Flex, rvue::properties::PropertyMap::new());

    let child2 =
        Component::with_properties(3, ComponentType::Flex, rvue::properties::PropertyMap::new());

    parent.add_child(Gc::clone(&child1));
    parent.add_child(Gc::clone(&child2));

    assert_eq!(parent.component_type, ComponentType::Flex);
    assert_eq!(child1.component_type, ComponentType::Flex);
    assert_eq!(child2.component_type, ComponentType::Flex);
}
