#![allow(unused_braces)]

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue::Scene;
use rvue_macro::view;

#[test]
fn test_nested_child_transform_update() {
    let (text_content, set_text_content) = create_signal("Initial".to_string());
    let set_text_content_clone = set_text_content.clone();
    let text_value = text_content.clone();

    let view = view! {
        <Flex direction="column" gap=10.0 align_items="center">
            <Text content={text_value} />
            <Button label="Click" on_click=move || {
                set_text_content_clone.set("Updated".to_string());
            } />
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    assert!(!root_component.is_dirty());

    set_text_content.set("Changed text".to_string());
    assert!(root_component.is_dirty());
    scene.update();
    assert!(!root_component.is_dirty());

    let text_component = {
        let children = root_component.children.borrow();
        children[0].clone()
    };

    let cache = text_component.vello_cache.borrow();
    assert!(cache.is_some());
}

#[test]
fn test_sibling_cache_preserved() {
    let (count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();
    let count_label = || format!("Count: {}", count.get());

    let view = view! {
        <Flex direction="column">
            <Text content={count_label()} />
            <Text content="Static sibling" />
            <Button label="+" on_click=move || set_count_clone.update(|c| *c += 1) />
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    let static_sibling = {
        let children = root_component.children.borrow();
        children[1].clone()
    };

    let static_sibling_cache_before = static_sibling.vello_cache.borrow().is_some();

    set_count.set(42);
    scene.update();

    let static_sibling_cache_after = static_sibling.vello_cache.borrow().is_some();

    assert_eq!(static_sibling_cache_before, static_sibling_cache_after);
}

#[test]
fn test_cache_not_double_appended() {
    let (count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();
    let count_label = || format!("Count: {}", count.get());

    let view = view! {
        <Flex direction="column">
            <Text content={count_label()} />
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    let text_component = {
        let children = root_component.children.borrow();
        children[0].clone()
    };

    {
        let cache = text_component.vello_cache.borrow();
        assert!(cache.is_some());
    }

    set_count_clone.set(1);
    scene.update();

    {
        let cache2 = text_component.vello_cache.borrow();
        assert!(cache2.is_some());
    }

    let flex_cache = root_component.vello_cache.borrow();
    assert!(flex_cache.is_some());
}

#[test]
fn test_parent_dirty_force_child_regen() {
    let (outer_count, set_outer_count) = create_signal(0);
    let set_outer_count_clone = set_outer_count.clone();
    let outer_label = || format!("Outer: {}", outer_count.get());

    let view = view! {
        <Flex direction="column">
            <Text content={outer_label()} />
            <Flex direction="row">
                <Text content="Inner static" />
            </Flex>
            <Button label="Update outer" on_click=move || {
                set_outer_count_clone.update(|c| *c += 1);
            } />
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    let inner_flex = {
        let children = root_component.children.borrow();
        children[1].clone()
    };

    let inner_text = {
        let children = inner_flex.children.borrow();
        children[0].clone()
    };

    assert!(!inner_text.vello_cache.borrow().is_none());

    set_outer_count.set(99);
    scene.update();

    let inner_text_cache = inner_text.vello_cache.borrow();
    assert!(inner_text_cache.is_some());
}

#[test]
fn test_cache_valid_after_multiple_updates() {
    let (count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();
    let count_label = || format!("Count: {}", count.get());

    let view = view! {
        <Flex direction="column">
            <Text content={count_label()} />
            <Text content="Static" />
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    let static_text = {
        let children = root_component.children.borrow();
        children[1].clone()
    };

    for i in 1..=5 {
        set_count_clone.set(i);
        scene.update();

        let cache = static_text.vello_cache.borrow();
        assert!(cache.is_some(), "Cache should exist after update {}", i);
    }
}

#[test]
fn test_clean_component_skips_recursion() {
    let (count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();
    let count_label = || format!("Count: {}", count.get());

    let view = view! {
        <Flex direction="column">
            <Text content={count_label()} />
            <Flex direction="row">
                <Text content="Nested child" />
                <Text content="Another child" />
            </Flex>
        </Flex>
    };

    let root_component = view.root_component;
    let mut scene = Scene::new();
    scene.add_fragment(root_component.clone());
    scene.update();

    let nested_flex = {
        let children = root_component.children.borrow();
        children[1].clone()
    };

    let nested_children: Vec<Gc<rvue::Component>> = {
        let children = nested_flex.children.borrow();
        children.iter().cloned().collect()
    };

    for child in &nested_children {
        assert!(child.vello_cache.borrow().is_some());
    }

    set_count_clone.set(100);
    scene.update();

    for child in &nested_children {
        assert!(child.vello_cache.borrow().is_some());
    }
}
