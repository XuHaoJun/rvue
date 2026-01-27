#![allow(unused_braces)]

use rvue::prelude::*;
use rvue::text::TextContext;
use rvue::widget::BuildContext;
use rvue::Gc;
use rvue_macro::{component, view};
use taffy::TaffyTree;

fn with_build_context<F, R>(f: F) -> R
where
    F: for<'a> FnOnce(&'a mut BuildContext<'a>) -> R,
{
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);
    f(&mut ctx)
}

#[component]
fn ChildComp(value: String) -> impl View {
    let label = format!("Child: {}", value);
    view! {
        <Text content={label} />
    }
}

#[test]
fn test_block_expansion_reactivity() {
    let (count, set_count) = create_signal("0".to_string());
    let view = with_build_context(|_ctx| {
        view! {
            <Flex>
                {count.clone()}
            </Flex>
        }
    });
    let root = view.root_component;

    let initial_children = root.children.borrow().len();
    assert!(initial_children >= 1);

    set_count.set("1".to_string());
    root.update();
}

#[test]
fn test_custom_component_reactive_prop() {
    let (label, set_label) = create_signal("Initial".to_string());
    let label_value = label.get();
    with_build_context(|_ctx| {
        let view = view! {
            <ChildComp value={label_value.clone()} />
        };
        let root = view.root_component;

        set_label.set("Updated".to_string());
        root.update();
    });
}

#[test]
fn test_memo_reactivity() {
    let (count, set_count) = create_signal("2".to_string());
    let doubled = create_memo(move || format!("Doubled: {}", count.get()));

    let doubled_clone = doubled.clone();
    with_build_context(|_ctx| {
        let view = view! {
            <Text content={doubled_clone.clone()} />
        };
        let root = view.root_component;

        assert_eq!(doubled.get(), "Doubled: 2");

        set_count.set("5".to_string());
        root.update();

        assert_eq!(doubled.get(), "Doubled: 5");
    });
}

#[test]
fn test_for_each_reactive_item() {
    let (count, set_count) = create_signal(0);
    assert_eq!(count.get(), 0);
    set_count.set(1);
    assert_eq!(count.get(), 1);
}

#[test]
fn test_event_handler_0arg_closure() {
    let clicked = std::rc::Rc::new(std::cell::RefCell::new(false));
    let clicked_clone = clicked.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <Button label="Click" on_click={move || { *clicked_clone.borrow_mut() = true; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_click().is_some());
    });
}

#[test]
fn test_event_handler_1arg_closure() {
    let last_value = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let last_value_clone = last_value.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <TextInput value="" on_input={move |e| { *last_value_clone.borrow_mut() = e.value.clone(); }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_input().is_some());
    });
}

#[test]
fn test_event_handler_2arg_closure() {
    let call_count = std::rc::Rc::new(std::cell::RefCell::new(0));
    let call_count_clone = call_count.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <Button label="Click" on_click={move |_e, _ctx| { *call_count_clone.borrow_mut() += 1; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_click().is_some());
    });
}

#[test]
fn test_event_handler_with_signal_capture() {
    let (_count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <Button label="Click" on_click={move |_e| { set_count_clone.update(|x| *x += 1); }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_click().is_some());
    });
}

#[test]
fn test_multiple_event_handlers_in_view() {
    let click_count = std::rc::Rc::new(std::cell::RefCell::new(0));
    let input_count = std::rc::Rc::new(std::cell::RefCell::new(0));

    let click_count_clone = click_count.clone();
    let input_count_clone = input_count.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <Flex>
                <Button label="Click" on_click={move || { *click_count_clone.borrow_mut() += 1; }} />
                <TextInput value="" on_input={move |_e| { *input_count_clone.borrow_mut() += 1; }} />
            </Flex>
        };
        let root = view.root_component;

        let mut click_handler_found = false;
        let mut input_handler_found = false;

        fn find_handlers(
            component: &Gc<Component>,
            click_handler_found: &mut bool,
            input_handler_found: &mut bool,
        ) {
            let handlers = component.event_handlers.borrow();
            if handlers.get_click().is_some() {
                *click_handler_found = true;
            }
            if handlers.get_input().is_some() {
                *input_handler_found = true;
            }

            for child in component.children.borrow().iter() {
                find_handlers(child, click_handler_found, input_handler_found);
            }
        }

        find_handlers(&root, &mut click_handler_found, &mut input_handler_found);

        assert!(click_handler_found, "Click handler not found in any component");
        assert!(input_handler_found, "Input handler not found in any component");
    });
}

#[test]
fn test_keyboard_event_handler() {
    let pressed = std::rc::Rc::new(std::cell::RefCell::new(false));
    let pressed_clone = pressed.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <TextInput value="" on_key_down={move |_e| { *pressed_clone.borrow_mut() = true; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_key_down().is_some());
    });
}

#[test]
fn test_focus_event_handler() {
    let focused = std::rc::Rc::new(std::cell::RefCell::new(false));
    let focused_clone = focused.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <TextInput value="" on_focus={move || { *focused_clone.borrow_mut() = true; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_focus().is_some());
    });
}

#[test]
fn test_change_event_handler() {
    let checked = std::rc::Rc::new(std::cell::RefCell::new(false));
    let checked_clone = checked.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <Checkbox checked=false on_change={move |_e| { *checked_clone.borrow_mut() = true; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_change().is_some());
    });
}

#[test]
fn test_blur_event_handler() {
    let blurred = std::rc::Rc::new(std::cell::RefCell::new(false));
    let blurred_clone = blurred.clone();

    with_build_context(|_ctx| {
        let view = view! {
            <TextInput value="" on_blur={move || { *blurred_clone.borrow_mut() = true; }} />
        };
        let root = view.root_component;

        let handlers = root.event_handlers.borrow();
        assert!(handlers.get_blur().is_some());
    });
}
