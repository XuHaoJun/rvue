//! Tests for the slot mechanism

use rudo_gc::Gc;
use rvue::prelude::*;
use rvue_macro::slot;

#[slot]
struct TestSlot {
    children: ChildrenFn,
}

#[slot]
struct TestSlotWithProps {
    item: String,
    children: ChildrenFn,
}

#[slot]
struct TestSlotWithOptional {
    item: Option<String>,
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
    assert_eq!(*counter.borrow(), 1);
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
fn test_children_fn_single_run() {
    let children: Children = Children(Box::new(|| create_test_view(6)));
    let view = children.run();
    assert_eq!(view.root_component.id, 6);
}

#[test]
fn test_slot_with_props() {
    let slot = TestSlotWithProps::new((|| create_test_view(7)).to_children())
        .item("test_item".to_string());

    assert_eq!(slot.item, "test_item");
    let view = slot.children.run();
    assert_eq!(view.root_component.id, 7);
}

#[test]
fn test_slot_with_optional_props() {
    let slot = TestSlotWithOptional::new((|| create_test_view(8)).to_children())
        .item(Some("optional_item".to_string()));

    assert_eq!(slot.item.unwrap(), "optional_item");

    let slot_none = TestSlotWithOptional::new((|| create_test_view(9)).to_children());
    assert!(slot_none.item.is_none());
}

#[test]
fn test_slot_props_clone() {
    let slot1 =
        TestSlotWithProps::new((|| create_test_view(10)).to_children()).item("item1".to_string());
    let slot2 = slot1.clone();

    assert_eq!(slot1.item, slot2.item);
    assert!(slot1.children.ptr_eq(&slot2.children));
}

#[test]
fn test_slot_into_reactive_value() {
    use rvue::widget::IntoReactiveValue;

    let slot = TestSlot::new((|| create_test_view(11)).to_children());
    let reactive = slot.into_reactive();

    match reactive {
        ReactiveValue::Static(s) => {
            let view = s.children.run();
            assert_eq!(view.root_component.id, 11);
        }
        ReactiveValue::Signal(_) => {
            // Signal variant exists but not tested here
        }
    }
}

#[test]
fn test_slot_with_props_into_reactive_value() {
    use rvue::widget::IntoReactiveValue;

    let slot = TestSlotWithProps::new((|| create_test_view(12)).to_children())
        .item("reactive_item".to_string());
    let reactive = slot.into_reactive();

    match reactive {
        ReactiveValue::Static(s) => {
            assert_eq!(s.item, "reactive_item");
        }
        ReactiveValue::Signal(_) => {
            // Signal variant exists but not tested here
        }
    }
}

#[test]
fn test_slot_multiple_props() {
    let slot = TestSlotWithProps::new((|| create_test_view(13)).to_children())
        .item("multi_prop".to_string());

    let slot2 = slot.clone().item("updated".to_string());

    assert_eq!(slot.item, "multi_prop");
    assert_eq!(slot2.item, "updated");
}
