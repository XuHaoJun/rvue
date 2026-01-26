use rvue::prelude::*;
use rvue::text::TextContext;
use rvue::widget::BuildContext;
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
    view! {
        <Text content={format!("Child: {}", value)} />
    }
}

#[test]
fn test_block_expansion_reactivity() {
    let (count, set_count) = create_signal("0".to_string());
    with_build_context(|_ctx| {
        let view = view! {
            <Flex>
                {count.clone()}
            </Flex>
        };
        let root = view.root_component;

        // Initial build should have created a Text child
        assert_eq!(root.children.borrow().len(), 1);
        let child = root.children.borrow()[0].clone();

        {
            let props = child.props.borrow();
            if let ComponentProps::Text { content, .. } = &*props {
                assert_eq!(content, "0");
            }
        }

        // Update signal
        set_count.set("1".to_string());
        root.update(); // This should trigger effects recursively

        {
            let props = child.props.borrow();
            if let ComponentProps::Text { content, .. } = &*props {
                assert_eq!(content, "1");
            }
        }
    });
}

#[test]
fn test_custom_component_reactive_prop() {
    let (label, set_label) = create_signal("Initial".to_string());
    with_build_context(|_ctx| {
        // We need to use ChildComp inside a view! to trigger the codegen that wraps in memo
        let view = view! {
            <ChildComp value={label.get()} />
        };
        let root = view.root_component;

        // Initial check
        {
            let props = root.props.borrow();
            if let ComponentProps::Text { content, .. } = &*props {
                assert_eq!(content, "Child: Initial");
            }
        }

        // Update signal in parent
        set_label.set("Updated".to_string());
        root.update();

        {
            let props = root.props.borrow();
            if let ComponentProps::Text { content, .. } = &*props {
                assert_eq!(content, "Child: Updated");
            }
        }
    });
}

#[test]
#[ignore]
fn test_memo_reactivity() {
    let (count, set_count) = create_signal("2".to_string());
    let doubled = create_memo(move || format!("Doubled: {}", count.get()));

    let doubled_clone = doubled.clone();
    with_build_context(|_ctx| {
        let view = view! {
            <Text content={doubled_clone} />
        };
        let root = view.root_component;

        assert_eq!(doubled.get(), "Doubled: 2");

        set_count.set("5".to_string());
        root.update();

        assert_eq!(doubled.get(), "Doubled: 5");

        {
            let props = root.props.borrow();
            if let ComponentProps::Text { content, .. } = &*props {
                assert_eq!(content, "Doubled: 5");
            }
        }
    });
}
