use rudo_gc::{collect_full, Gc};
use rvue::context::{inject, provide_context};
use rvue::prelude::*;
use rvue::{text::TextContext, widget::BuildContext, TaffyTree};

fn super_aggressive_gc_cleanup() {
    for _ in 0..20 {
        collect_full();
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[test]
fn test_context_gc_on_unmount_with_i32() {
    super_aggressive_gc_cleanup();

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    let child_comp = ctx.create_component(
        ComponentType::Custom("Child".to_string()),
        rvue::properties::PropertyMap::new(),
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

    #[allow(clippy::drop_non_drop)]
    drop(injected_value);
    #[allow(clippy::drop_non_drop)]
    drop(root);
    #[allow(clippy::drop_non_drop)]
    drop(child_comp);
    #[allow(clippy::drop_non_drop)]
    drop(ctx);

    super_aggressive_gc_cleanup();
}

#[test]
#[ignore = "GC isolation issue with GcCell<Vec<Gc<T>>> pattern"]
fn test_context_nested_injection() {
    super_aggressive_gc_cleanup();

    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    let mid_comp = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());
    mid_comp.set_parent(Some(root.clone()));
    root.add_child(mid_comp.clone());

    let leaf_comp = ctx.create_component(
        ComponentType::Custom("Leaf".to_string()),
        rvue::properties::PropertyMap::new(),
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

    #[allow(clippy::drop_non_drop)]
    drop(leaf_comp);
    #[allow(clippy::drop_non_drop)]
    drop(mid_comp);
    #[allow(clippy::drop_non_drop)]
    drop(root);
    #[allow(clippy::drop_non_drop)]
    drop(ctx);

    super_aggressive_gc_cleanup();
}
