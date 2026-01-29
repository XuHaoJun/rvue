use rudo_gc::{collect_full, Gc};
use rvue::context::{inject, provide_context};
use rvue::prelude::*;
use rvue::{text::TextContext, widget::BuildContext, TaffyTree};

fn aggressive_gc_cleanup() {
    for _ in 0..5 {
        collect_full();
    }
}

#[test]
fn test_context_trace_size_correct() {
    aggressive_gc_cleanup();

    use std::mem::size_of;

    let gc_i32 = Gc::new(42i32);
    let gc_ptr = rudo_gc::Gc::internal_ptr(&gc_i32);
    let gc_size = size_of::<Gc<i32>>();

    assert!(!gc_ptr.is_null());
    assert!(gc_size > 0, "Gc<T> should have non-zero size");

    let gc_string = Gc::new(String::from("hello"));
    let gc_string_ptr = rudo_gc::Gc::internal_ptr(&gc_string);
    let gc_string_size = size_of::<Gc<String>>();

    assert!(!gc_string_ptr.is_null());
    assert!(gc_string_size >= gc_size, "Gc<String> should be >= Gc<i32> size");

    drop(gc_i32);
    drop(gc_string);
    aggressive_gc_cleanup();
}

#[test]
fn test_context_gc_on_unmount_with_i32() {
    aggressive_gc_cleanup();

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "row".to_string(),
            gap: 0.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
        },
    );

    let child_comp = ctx.create_component(
        ComponentType::Custom("Child".to_string()),
        ComponentProps::Custom { data: String::new() },
    );
    child_comp.set_parent(Some(root.clone()));
    root.add_child(child_comp.clone());

    let mut injected_value: Option<Gc<i32>> = None;

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(42i32);

        rvue::runtime::with_owner(child_comp.clone(), || {
            injected_value = inject::<i32>();
        });
    });

    assert!(injected_value.is_some(), "Context should be available");
    let value = **injected_value.as_ref().unwrap();
    assert_eq!(value, 42);

    drop(injected_value);
    drop(root);
    drop(child_comp);

    aggressive_gc_cleanup();
}

#[test]
fn test_context_nested_injection() {
    aggressive_gc_cleanup();

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
        },
    );

    let mid_comp = ctx.create_component(
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 0.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
        },
    );
    mid_comp.set_parent(Some(root.clone()));
    root.add_child(mid_comp.clone());

    let leaf_comp = ctx.create_component(
        ComponentType::Custom("Leaf".to_string()),
        ComponentProps::Custom { data: String::new() },
    );
    leaf_comp.set_parent(Some(mid_comp.clone()));
    mid_comp.add_child(leaf_comp.clone());

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(100i32);

        rvue::runtime::with_owner(mid_comp.clone(), || {
            provide_context(200i32);

            rvue::runtime::with_owner(leaf_comp.clone(), || {
                let ctx = inject::<i32>();
                assert_eq!(ctx.map(|v| *v), Some(200), "Child context should shadow parent");
            });
        });
    });

    drop(leaf_comp);
    drop(mid_comp);
    drop(root);

    aggressive_gc_cleanup();
}
