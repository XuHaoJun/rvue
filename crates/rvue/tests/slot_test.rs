//! Tests for the slot mechanism

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue_macro::slot;
use std::sync::Arc;

#[slot]
struct TestSlot {
    children: ChildrenFn,
}

fn create_test_component(id: u64) -> Gc<Component> {
    Component::new(
        id,
        ComponentType::Text,
        ComponentProps::Text { content: "test".to_string(), font_size: None, color: None },
    )
}

fn create_test_view(id: u64) -> ViewStruct {
    let comp = create_test_component(id);
    ViewStruct::new(comp)
}

#[test]
fn test_slot_new() {
    let slot = TestSlot::new(Arc::new(|| create_test_view(1)));

    let view = (slot.children)();
    assert_eq!(view.root_component.id, 1);
}

#[test]
fn test_slot_into_vec() {
    let slot = TestSlot::new(Arc::new(|| create_test_view(1)));

    let vec: Vec<TestSlot> = slot.into();
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_to_children_fn() {
    let closure: ChildrenFn = (|| create_test_view(2)).to_children();

    let view = closure();
    assert_eq!(view.root_component.id, 2);
}

#[test]
fn test_maybe_slot() {
    use rvue::slot::MaybeSlot;

    let none_slot: MaybeSlot = MaybeSlot::default();
    assert!(none_slot.is_none());
    assert!(none_slot.render().is_none());

    let some_slot = MaybeSlot::from(Some(rvue::slot::Slot::new(Arc::new(|| create_test_view(3)))));

    assert!(some_slot.is_some());
    let view = some_slot.render().unwrap();
    assert_eq!(view.root_component.id, 3);
}

#[test]
fn test_slot_to_children_trait() {
    let view = create_test_view(4);

    let children_fn: ChildrenFn = view.to_children();
    let rendered = children_fn();
    assert_eq!(rendered.root_component.id, 4);
}

#[test]
fn test_slot_clone() {
    let slot = TestSlot::new(Arc::new(|| create_test_view(5)));
    let cloned = slot.clone();

    let view = (cloned.children)();
    assert_eq!(view.root_component.id, 5);
}

#[test]
fn test_slot_render() {
    use rvue::slot::Slot;

    let slot = Slot::new(Arc::new(|| create_test_view(6)));

    let view = slot.render();
    assert_eq!(view.root_component.id, 6);
}
