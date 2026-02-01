//! Tests for slot props functionality

#![allow(unused)]

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue::text::TextContext;
use rvue::widget::BuildContext;
use rvue::TaffyTree;
use rvue_macro::slot;

#[slot]
struct ItemSlot {
    item: String,
    children: ChildrenFn,
}

fn create_test_component(id: u64) -> Gc<Component> {
    Component::new(
        id,
        ComponentType::Text,
        ComponentProps::Text { content: "test".to_string(), styles: None },
    )
}

fn create_test_view(id: u64) -> ViewStruct {
    let comp = create_test_component(id);
    ViewStruct::new(comp)
}

#[test]
fn test_slot_with_prop() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(1)).to_children();

    let slot = ItemSlot::new(children).item("test_item".to_string());

    assert_eq!(slot.item, "test_item");
}

#[test]
fn test_slot_clone() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(3)).to_children();

    let slot1 = ItemSlot::new(children.clone()).item("item1".to_string());
    let slot2 = slot1.clone();

    assert_eq!(slot1.item, slot2.item);
    assert!(slot1.children.ptr_eq(&slot2.children));
}

#[test]
fn test_slot_into_vec() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(4)).to_children();

    let slot = ItemSlot::new(children).item("item".to_string());

    let vec: Vec<ItemSlot> = slot.into();
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_slot_children_run() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(5)).to_children();

    let slot = ItemSlot::new(children).item("test".to_string());

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter: u64 = 0;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let view = slot.children.run(&mut ctx);
    assert_eq!(view.root_component.id, 5);
}

#[slot]
struct OptionalSlot {
    label: Option<String>,
    children: ChildrenFn,
}

#[test]
fn test_optional_slot_with_value() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(7)).to_children();

    let slot = OptionalSlot::new(children).label(Some("My Label".to_string()));

    assert!(slot.label.is_some());
    assert_eq!(slot.label.unwrap(), "My Label");
}

#[test]
fn test_optional_slot_without_value() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(8)).to_children();

    let slot = OptionalSlot::new(children);

    assert!(slot.label.is_none());
}

#[slot]
struct MultiPropSlot {
    name: String,
    description: Option<String>,
    count: i32,
    children: ChildrenFn,
}

#[test]
fn test_multi_prop_slot() {
    let children: ChildrenFn = (|_: &mut BuildContext| create_test_view(9)).to_children();

    let slot = MultiPropSlot::new(children)
        .name("Test Name".to_string())
        .description(Some("Description".to_string()))
        .count(42);

    assert_eq!(slot.name, "Test Name");
    assert_eq!(slot.description.unwrap(), "Description");
    assert_eq!(slot.count, 42);
}
