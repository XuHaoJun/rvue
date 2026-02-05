use rvue::context::{inject, provide_context};
use rvue::prelude::*;
use rvue::{text::TextContext, widget::BuildContext, TaffyTree};
use rvue_macro::{component, view};

#[allow(non_snake_case)]
#[component]
fn Child() -> impl View {
    let val = inject::<i32>();
    assert_eq!(val.map(|v| *v), Some(42));
    view! { <Text content="child" /> }
}

#[test]
fn test_basic_context() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(42i32);

        let val = inject::<i32>();
        assert_eq!(val.map(|v| *v), Some(42));
    });
}

#[test]
fn test_nested_context() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(42i32);

        // This simulates what view! macro does for custom components
        let child_comp = ctx.create_component(
            ComponentType::Custom("Child".to_string()),
            rvue::properties::PropertyMap::new(),
        );
        child_comp.set_parent(Some(root.clone()));
        root.add_child(child_comp.clone());

        rvue::runtime::with_owner(child_comp.clone(), || {
            let _ = Child(ChildProps {});
        });
    });
}

#[allow(non_snake_case)]
#[component]
fn ShadowChild() -> impl View {
    let val = inject::<i32>();
    assert_eq!(val.map(|v| *v), Some(100)); // Should see the overridden value
    view! { <Text content="shadow" /> }
}

#[test]
fn test_context_shadowing() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(42i32);

        let sub_root =
            ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

        // CRITICAL: Set parent so search can traverse up
        sub_root.set_parent(Some(root.clone()));
        root.add_child(sub_root.clone());

        rvue::runtime::with_owner(sub_root.clone(), || {
            provide_context(100i32);

            let child_comp = ctx.create_component(
                ComponentType::Custom("ShadowChild".to_string()),
                rvue::properties::PropertyMap::new(),
            );
            child_comp.set_parent(Some(sub_root.clone()));
            sub_root.add_child(child_comp.clone());

            rvue::runtime::with_owner(child_comp.clone(), || {
                let _ = ShadowChild(ShadowChildProps {});
            });
        });
    });
}

#[test]
fn test_context_in_effect() {
    let mut taffy = TaffyTree::new();
    let mut text_context = TextContext::new();
    let mut id_counter = 0u64;
    let mut ctx = BuildContext::new(&mut taffy, &mut text_context, &mut id_counter);

    let root = ctx.create_component(ComponentType::Flex, rvue::properties::PropertyMap::new());

    rvue::runtime::with_owner(root.clone(), || {
        provide_context(42i32);

        let (s, set_s) = create_signal(0);

        create_effect(move || {
            let _ = s.get();
            let val = inject::<i32>();
            assert_eq!(val.map(|v| *v), Some(42));
        });

        set_s.set(1);
    });
}
