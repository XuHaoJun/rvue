//! Todo list example application

use rudo_gc::Trace;
use rvue::impl_gc_capture;
use rvue::prelude::*;
use rvue_macro::view;

#[derive(Clone)]
struct TodoItem(String);

unsafe impl Trace for TodoItem {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl_gc_capture!(TodoItem);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create todo list view
    let todo_view = create_todo_list_view();

    // Run the application
    rvue::run_app(|| todo_view)?;

    Ok(())
}

fn create_todo_list_view() -> ViewStruct {
    let initial_items = vec![
        TodoItem("Learn Rvue".to_string()),
        TodoItem("Build a counter app".to_string()),
        TodoItem("Create a todo list".to_string()),
    ];

    let (todos, _set_todos) = create_signal(initial_items);

    view! {
        <Flex direction="column" gap=10.0 align_items="start" justify_content="start">
            <Text content="Todo List:" />
            <For each=todos.clone() key=|item: &TodoItem| item.0.clone() view={|item| view! {
                <Text content=format!("• {}", item.0) />
            }}/>
        </Flex>
    }
}
