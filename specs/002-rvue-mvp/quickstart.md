# Quickstart Guide: Rvue MVP Framework

**Date**: 2026-01-17  
**Purpose**: Get developers started with Rvue framework quickly

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rvue = "0.1.0"
```

## Hello World

```rust
use rvue::prelude::*;

fn main() {
    run_app(|| {
        view! {
            <Text>"Hello, Rvue!"</Text>
        }
    }).unwrap();
}
```

## Counter Example

```rust
use rvue::prelude::*;

#[component]
fn Counter() -> impl View {
    let (count, set_count) = create_signal(0);
    
    let increment = move |_| {
        set_count.update(|c| *c += 1);
    };
    
    let decrement = move |_| {
        set_count.update(|c| *c -= 1);
    };
    
    view! {
        <Flex direction=FlexDirection::Column gap=16.0 align_items=AlignItems::Center>
            <Text font_size=48.0>
                {move || count.get().to_string()}
            </Text>
            <Flex direction=FlexDirection::Row gap=10.0>
                <Button on_click=decrement>
                    <Text>"-"</Text>
                </Button>
                <Button on_click=increment>
                    <Text>"+"</Text>
                </Button>
            </Flex>
        </Flex>
    }
}

fn main() {
    run_app(|| {
        view! {
            <Counter />
        }
    }).unwrap();
}
```

## Conditional Rendering

```rust
use rvue::prelude::*;

#[component]
fn ToggleExample() -> impl View {
    let (is_visible, set_visible) = create_signal(true);
    
    let toggle = move |_| {
        set_visible.update(|v| *v = !*v);
    };
    
    view! {
        <Flex direction=FlexDirection::Column gap=10.0>
            <Button on_click=toggle>
                <Text>"Toggle"</Text>
            </Button>
            <Show when=is_visible>
                <Text>"This text is conditionally rendered!"</Text>
            </Show>
        </Flex>
    }
}
```

## List Rendering

```rust
use rvue::prelude::*;

#[derive(Clone, Trace)]
struct TodoItem {
    id: u32,
    text: String,
    completed: bool,
}

#[component]
fn TodoList() -> impl View {
    let (todos, set_todos) = create_signal(vec![
        TodoItem { id: 1, text: "Learn Rvue".to_string(), completed: false },
        TodoItem { id: 2, text: "Build app".to_string(), completed: false },
    ]);
    
    let add_todo = move |_| {
        set_todos.update(|todos| {
            let new_id = todos.len() as u32 + 1;
            todos.push(TodoItem {
                id: new_id,
                text: "New todo".to_string(),
                completed: false,
            });
        });
    };
    
    view! {
        <Flex direction=FlexDirection::Column gap=10.0>
            <Button on_click=add_todo>
                <Text>"Add Todo"</Text>
            </Button>
            <For each=todos key=|item| item.id.to_string()>
                |item| view! {
                    <Flex direction=FlexDirection::Row gap=10.0>
                        <Checkbox checked=move || item.completed />
                        <Text>{move || item.text.clone()}</Text>
                    </Flex>
                }
            </For>
        </Flex>
    }
}
```

## Form Input

```rust
use rvue::prelude::*;

#[component]
fn LoginForm() -> impl View {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    
    let handle_submit = move |_| {
        println!("Username: {}, Password: {}", username.get(), password.get());
        // Handle login...
    };
    
    view! {
        <Flex direction=FlexDirection::Column gap=10.0>
            <TextInput 
                value=username 
                on_input=move |val| set_username.set(val)
                placeholder="Username"
            />
            <TextInput 
                value=password 
                on_input=move |val| set_password.set(val)
                placeholder="Password"
                input_type=InputType::Password
            />
            <Button on_click=handle_submit>
                <Text>"Login"</Text>
            </Button>
        </Flex>
    }
}
```

## Styling

```rust
use rvue::prelude::*;

#[component]
fn StyledComponent() -> impl View {
    view! {
        <Flex 
            direction=FlexDirection::Column 
            gap=20.0
            style=Style {
                padding: Spacing::all(20.0),
                background_color: Some(Color::rgb(0.95, 0.95, 0.95)),
                border_radius: Some(10.0),
                ..Default::default()
            }
        >
            <Text 
                style=Style {
                    font_size: Some(24.0),
                    font_weight: Some(FontWeight::Bold),
                    color: Some(Color::rgb(0.2, 0.2, 0.2)),
                    ..Default::default()
                }
            >
                "Styled Text"
            </Text>
            <Button 
                style=Style {
                    padding: Spacing::all(10.0),
                    background_color: Some(Color::rgb(0.0, 0.5, 1.0)),
                    border_radius: Some(5.0),
                    ..Default::default()
                }
            >
                <Text color=Color::White>"Styled Button"</Text>
            </Button>
        </Flex>
    }
}
```

## Computed Values

```rust
use rvue::prelude::*;

#[component]
fn ComputedExample() -> impl View {
    let (count, set_count) = create_signal(0);
    
    // Computed value (derived signal)
    let double_count = move || count.get() * 2;
    let is_high = move || count.get() > 10;
    
    view! {
        <Flex direction=FlexDirection::Column gap=10.0>
            <Text>Count: {move || count.get().to_string()}</Text>
            <Text>Double: {move || double_count().to_string()}</Text>
            <Show when=is_high>
                <Text color=Color::Red>"Count is high!"</Text>
            </Show>
            <Button on_click=move |_| set_count.update(|c| *c += 1)>
                <Text>"Increment"</Text>
            </Button>
        </Flex>
    }
}
```

## Component Composition

```rust
use rvue::prelude::*;

#[component]
fn Card(title: String, children: impl View) -> impl View {
    view! {
        <Flex 
            direction=FlexDirection::Column 
            gap=10.0
            style=Style {
                padding: Spacing::all(15.0),
                border: Some(Border {
                    width: 1.0,
                    color: Color::rgb(0.8, 0.8, 0.8),
                    ..Default::default()
                }),
                border_radius: Some(8.0),
                ..Default::default()
            }
        >
            <Text font_size=18.0 font_weight=FontWeight::Bold>
                {title}
            </Text>
            {children}
        </Flex>
    }
}

#[component]
fn App() -> impl View {
    view! {
        <Flex direction=FlexDirection::Column gap=20.0>
            <Card title="First Card">
                <Text>"Content of first card"</Text>
            </Card>
            <Card title="Second Card">
                <Text>"Content of second card"</Text>
            </Card>
        </Flex>
    }
}
```

## Next Steps

1. **Explore Examples**: Check `rvue-examples` crate for more complex examples
2. **Read Documentation**: See full API documentation at [docs.rs/rvue](https://docs.rs/rvue)
3. **Join Community**: [GitHub Discussions](https://github.com/your-org/rvue/discussions)
4. **Report Issues**: [GitHub Issues](https://github.com/your-org/rvue/issues)

## Common Patterns

### Two-way Binding

```rust
let (value, set_value) = create_signal(String::new());

<TextInput 
    value=value 
    on_input=move |val| set_value.set(val)
/>
```

### Conditional Classes

```rust
let (is_active, set_active) = create_signal(false);

<Button 
    style=move || Style {
        background_color: if is_active.get() { 
            Some(Color::Green) 
        } else { 
            Some(Color::Gray) 
        },
        ..Default::default()
    }
>
    <Text>"Toggle"</Text>
</Button>
```

### Event Handling with Data

```rust
let (items, set_items) = create_signal(vec![1, 2, 3]);

<For each=items key=|item| item.to_string()>
    |item| {
        let remove_item = move |_| {
            set_items.update(|items| {
                items.retain(|&x| x != item);
            });
        };
        view! {
            <Flex direction=FlexDirection::Row>
                <Text>{move || item.to_string()}</Text>
                <Button on_click=remove_item>
                    <Text>"Remove"</Text>
                </Button>
            </Flex>
        }
    }
</For>
```

## Troubleshooting

### Signal not updating UI

- Ensure you're using `move ||` closures in view bindings
- Check that signal is actually being updated (use `println!` for debugging)
- Verify effect dependencies are being tracked correctly

### Component not rendering

- Ensure component function returns `impl View`
- Check that `#[component]` macro is applied
- Verify view macro syntax is correct

### Layout issues

- Check flex direction and alignment properties
- Ensure parent container has appropriate size constraints
- Use `gap` property for spacing instead of margins where possible

### Performance issues

- Avoid creating signals inside render functions
- Use computed values instead of recalculating in effects
- Minimize signal updates in tight loops
