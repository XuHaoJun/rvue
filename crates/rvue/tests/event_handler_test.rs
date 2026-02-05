//! Unit tests for event handler type inference

use std::cell::RefCell;

use rvue::{
    component::Component,
    event::{status::FocusEvent, types::PointerButton},
    ComponentType,
};

#[test]
fn test_on_click_0arg_handler() {
    let clicked = std::rc::Rc::new(RefCell::new(false));
    let clicked_clone = clicked.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click_0arg(move || {
        *clicked_clone.borrow_mut() = true;
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_click().is_some());
}

#[test]
fn test_on_click_1arg_handler() {
    let last_clicked_button = std::rc::Rc::new(RefCell::new(None::<PointerButton>));
    let last_clicked_clone = last_clicked_button.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click_1arg(move |event| {
        *last_clicked_clone.borrow_mut() = Some(event.button.clone());
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_click().is_some());
}

#[test]
fn test_on_click_2arg_handler() {
    let call_count = std::rc::Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click(move |_event, _ctx| {
        *call_count_clone.borrow_mut() += 1;
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_click().is_some());
}

#[test]
fn test_on_input_0arg_handler() {
    let triggered = std::rc::Rc::new(RefCell::new(false));
    let triggered_clone = triggered.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_input_0arg(move || {
        *triggered_clone.borrow_mut() = true;
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_input().is_some());
}

#[test]
fn test_on_input_1arg_handler() {
    let last_value = std::rc::Rc::new(RefCell::new(String::new()));
    let last_value_clone = last_value.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_input_1arg(move |event| {
        *last_value_clone.borrow_mut() = event.value.clone();
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_input().is_some());
}

#[test]
fn test_on_input_2arg_handler() {
    let input_count = std::rc::Rc::new(RefCell::new(0));
    let input_count_clone = input_count.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_input(move |_event, _ctx| {
        *input_count_clone.borrow_mut() += 1;
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_input().is_some());
}

#[test]
fn test_on_key_down_0arg_handler() {
    let triggered = std::rc::Rc::new(RefCell::new(false));
    let triggered_clone = triggered.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_key_down_0arg(move || {
        *triggered_clone.borrow_mut() = true;
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_key_down().is_some());
}

#[test]
fn test_on_key_down_1arg_handler() {
    let last_key = std::rc::Rc::new(RefCell::new(None::<winit::keyboard::Key>));
    let last_key_clone = last_key.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_key_down_1arg(move |event| {
        *last_key_clone.borrow_mut() = Some(event.key.clone());
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_key_down().is_some());
}

#[test]
fn test_on_focus_0arg_handler() {
    let focused = std::rc::Rc::new(RefCell::new(false));
    let focused_clone = focused.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_focus_0arg(move || {
        *focused_clone.borrow_mut() = true;
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_focus().is_some());
}

#[test]
fn test_on_focus_1arg_handler() {
    let last_event = std::rc::Rc::new(RefCell::new(None::<FocusEvent>));
    let last_event_clone = last_event.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_focus_1arg(move |event| {
        *last_event_clone.borrow_mut() = Some(event.clone());
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_focus().is_some());
}

#[test]
fn test_on_pointer_move_0arg_handler() {
    let moved = std::rc::Rc::new(RefCell::new(false));
    let moved_clone = moved.clone();

    let flex =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    flex.on_pointer_move_0arg(move || {
        *moved_clone.borrow_mut() = true;
    });

    let handlers = flex.event_handlers.borrow();
    assert!(handlers.get_pointer_move().is_some());
}

#[test]
fn test_on_pointer_move_1arg_handler() {
    let last_position = std::rc::Rc::new(RefCell::new(None::<vello::kurbo::Point>));
    let last_position_clone = last_position.clone();

    let flex =
        Component::with_properties(1, ComponentType::Flex, rvue::properties::PropertyMap::new());

    flex.on_pointer_move_1arg(move |event| {
        *last_position_clone.borrow_mut() = Some(event.position);
    });

    let handlers = flex.event_handlers.borrow();
    assert!(handlers.get_pointer_move().is_some());
}

#[test]
fn test_multiple_event_handlers_same_component() {
    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click_0arg(|| {});

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::with(rvue::properties::TextInputValue(String::new())),
    );

    text_input.on_input_0arg(|| {});

    let button_handlers = button.event_handlers.borrow();
    let input_handlers = text_input.event_handlers.borrow();

    assert!(button_handlers.get_click().is_some());
    assert!(input_handlers.get_input().is_some());
}

#[test]
fn test_event_handler_with_captured_signals() {
    use rvue::create_signal;

    let (_count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click_1arg(move |_event| {
        set_count_clone.update(|x| *x += 1);
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_click().is_some());
}

#[test]
fn test_on_change_handler() {
    let last_value = std::rc::Rc::new(RefCell::new(String::new()));
    let last_value_clone = last_value.clone();

    let checkbox = Component::with_properties(
        1,
        ComponentType::Checkbox,
        rvue::properties::PropertyMap::with(rvue::properties::CheckboxChecked(false)),
    );

    checkbox.on_change_1arg(move |event| {
        *last_value_clone.borrow_mut() = format!("checked:{}", event.checked);
    });

    let handlers = checkbox.event_handlers.borrow();
    assert!(handlers.get_change().is_some());
}

#[test]
fn test_on_blur_handler() {
    let blurred = std::rc::Rc::new(RefCell::new(false));
    let blurred_clone = blurred.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_blur_0arg(move || {
        *blurred_clone.borrow_mut() = true;
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_blur().is_some());
}

#[test]
fn test_on_pointer_down_handler() {
    let pressed = std::rc::Rc::new(RefCell::new(false));
    let pressed_clone = pressed.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_pointer_down_1arg(move |_event| {
        *pressed_clone.borrow_mut() = true;
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_pointer_down().is_some());
}

#[test]
fn test_on_pointer_up_handler() {
    let released = std::rc::Rc::new(RefCell::new(false));
    let released_clone = released.clone();

    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_pointer_up_0arg(move || {
        *released_clone.borrow_mut() = true;
    });

    let handlers = button.event_handlers.borrow();
    assert!(handlers.get_pointer_up().is_some());
}

#[test]
fn test_on_key_up_handler() {
    let released = std::rc::Rc::new(RefCell::new(false));
    let released_clone = released.clone();

    let text_input = Component::with_properties(
        1,
        ComponentType::TextInput,
        rvue::properties::PropertyMap::new(),
    );

    text_input.on_key_up_1arg(move |event| {
        if event.state == rvue::event::types::KeyState::Up {
            *released_clone.borrow_mut() = true;
        }
    });

    let handlers = text_input.event_handlers.borrow();
    assert!(handlers.get_key_up().is_some());
}

#[test]
fn test_all_event_types_have_handlers() {
    let button =
        Component::with_properties(1, ComponentType::Button, rvue::properties::PropertyMap::new());

    button.on_click_0arg(|| {});
    button.on_pointer_down_0arg(|| {});
    button.on_pointer_up_0arg(|| {});
    button.on_pointer_move_0arg(|| {});
    button.on_key_down_0arg(|| {});
    button.on_key_up_0arg(|| {});

    let handlers = button.event_handlers.borrow();

    assert!(handlers.get_click().is_some());
    assert!(handlers.get_pointer_down().is_some());
    assert!(handlers.get_pointer_up().is_some());
    assert!(handlers.get_pointer_move().is_some());
    assert!(handlers.get_key_down().is_some());
    assert!(handlers.get_key_up().is_some());
}
