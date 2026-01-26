//! Tests for the slot mechanism

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue_macro::slot;

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
    let slot = TestSlot::new((|| create_test_view(1)).to_children());

    let view = slot.children.run();
    assert_eq!(view.root_component.id, 1);
}

#[test]
fn test_slot_into_vec() {
    let slot = TestSlot::new((|| create_test_view(1)).to_children());

    let vec: Vec<TestSlot> = slot.into();
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_children_fn_run() {
    let closure: ChildrenFn = (|| create_test_view(2)).to_children();

    let view = closure.run();
    assert_eq!(view.root_component.id, 2);
}

#[test]
fn test_children_fn_multiple_calls() {
    let counter = std::rc::Rc::new(std::cell::RefCell::new(0));
    let counter_clone = counter.clone();
    let closure: ChildrenFn = (move || {
        *counter_clone.borrow_mut() += 1;
        create_test_view(2)
    })
    .to_children();

    closure.run();
    closure.run();
    assert_eq!(*counter.borrow(), 2);
}

#[test]
fn test_maybe_children() {
    use rvue::slot::MaybeChildren;

    let none: MaybeChildren = MaybeChildren::default();
    assert!(none.is_none());
    assert!(none.render().is_none());

    let closure: ChildrenFn = (|| create_test_view(3)).to_children();
    let some = MaybeChildren::from(closure);
    assert!(some.is_some());
    let view = some.run().unwrap();
    assert_eq!(view.root_component.id, 3);
}

#[test]
fn test_to_children_traits() {
    let view = create_test_view(4);

    let children_fn: ChildrenFn = view.to_children();
    let rendered = children_fn.run();
    assert_eq!(rendered.root_component.id, 4);
}

#[test]
fn test_slot_clone() {
    let slot = TestSlot::new((|| create_test_view(5)).to_children());
    let cloned = slot.clone();

    let view = cloned.children.run();
    assert_eq!(view.root_component.id, 5);
}

#[test]
fn test_children_run() {
    let children: Children = Children(Box::new(|| create_test_view(6)));
    let view = children.run();
    assert_eq!(view.root_component.id, 6);
}
