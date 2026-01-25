//! Todo list example application

use rvue::prelude::*;
use rvue_macro::view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create todo list view
    let todo_view = create_todo_list_view();

    // Run the application
    rvue::run_app(|| todo_view)?;

    Ok(())
}

fn create_todo_list_view() -> ViewStruct {
    let initial_items = vec![
        "Learn Rvue".to_string(),
        "Build a counter app".to_string(),
        "Create a todo list".to_string(),
    ];

    let (todos, _set_todos) = create_signal(initial_items);

    view! {
        <Flex direction="column" gap=10.0 align_items="start" justify_content="start">
            <Text content="Todo List:" />
            <For each=todos.clone() key=|item: &String| item.clone() view={|item| view! {
                <Text content=format!("• {}", item) />
            }}/>
        </Flex>
    }
}
