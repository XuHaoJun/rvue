//! Todo list example application

use rvue::prelude::*;
use rvue::{Component, ComponentType, ComponentProps, ViewStruct, For};

// Note: TodoItem struct is defined but not used in MVP
// In a full implementation, this would be used with Trace derive
// #[derive(Clone, Debug, Trace)]
// struct TodoItem {
//     id: u32,
//     text: String,
//     completed: bool,
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create todo list view
    let todo_view = create_todo_list_view();
    
    // Run the application
    rvue::run_app(|| todo_view)?;
    
    Ok(())
}

fn create_todo_list_view() -> ViewStruct {
    // Note: For MVP, we'll use a simple Vec<String> since TodoItem doesn't implement Trace yet
    let initial_items = vec![
        "Learn Rvue".to_string(),
        "Build a counter app".to_string(),
        "Create a todo list".to_string(),
    ];
    
    let (todos, _set_todos) = create_signal(initial_items);
    
    // Create root component (Flex container)
    let root = Component::new(
        0,
        ComponentType::Flex,
        ComponentProps::Flex {
            direction: "column".to_string(),
            gap: 10.0,
            align_items: "start".to_string(),
            justify_content: "start".to_string(),
        },
    );
    
    // Create For component to render the list
    let _for_component = For::new(1, todos.get().len());
    
    // Create text components for each todo item
    // Note: In a full implementation, this would be done reactively via effects
    let mut item_id = 2;
    for item in todos.get() {
        let _item_component = Component::new(
            item_id,
            ComponentType::Text,
            ComponentProps::Text {
                content: format!("• {}", item),
            },
        );
        // In a full implementation, we'd add this to the For component's children
        item_id += 1;
    }
    
    // Create view
    let view = ViewStruct::new(root);
    
    // Add effect to update list when todos change
    let _effect = create_effect({
        let todos = todos.clone();
        move || {
            let _ = todos.get(); // Track the signal
            println!("Todo list changed, item count: {}", todos.get().len());
        }
    });
    
    view
}
