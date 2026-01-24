# Rvue Macro Expansion Examples

This document explains how the `view!` macro transforms HTML-like syntax into Rust code.

## Core Expansion Logic

The `view!` macro follows these steps:
1. **Parsing**: Uses `rstml` to parse the input tokens into an HTML-like AST.
2. **Analysis**: Classifies expressions as static or reactive (signal-based).
3. **Codegen**: Generates code that:
   - Initializes a `BuildContext`.
   - Creates widgets using their builder patterns.
   - Registers `create_effect` blocks for reactive attributes.
   - Establishes parent-child relationships in the component tree.

## Static vs. Dynamic Attributes

### 1. Static Attribute
```rust
view! { <Text value="Hello" /> }
```
**Expands roughly to:**
```rust
{
    let widget = rvue::widgets::Text::new("Hello".to_string());
    let state = widget.build(&mut ctx);
    let component = state.component().clone();
    component
}
```

### 2. Dynamic Attribute (Signal)
```rust
view! { <Text value={count.get()} /> }
```
**Expands roughly to:**
```rust
{
    // 1. Initial build with current value
    let widget = rvue::widgets::Text::new(count.get().to_string());
    let state = widget.build(&mut ctx);
    let component = state.component().clone();

    // 2. Register effect for fine-grained updates
    {
        let comp = component.clone();
        let effect = rvue::prelude::create_effect(move || {
            let new_value = (count.get()).to_string();
            comp.set_text_content(new_value);
        });
        component.add_effect(effect);
    }

    component
}
```

## Event Handlers

### 3. Button with Click Handler
```rust
view! { <Button on_click={move || println!("clicked")} /> }
```
**Expands roughly to:**
```rust
{
    let widget = rvue::widgets::Button::new("".to_string());
    let state = widget.build(&mut ctx);
    let component = state.component().clone();

    // Event handler registration
    component.add_event_listener("click", move |_| {
        (move || println!("clicked"))();
    });

    component
}
```

## Nested Structures

### 4. Flex with Children
```rust
view! {
    <Flex direction="column">
        <Text value="Child 1" />
    </Flex>
}
```
**Expands roughly to:**
```rust
{
    let widget = rvue::widgets::Flex::new().direction(FlexDirection::Column);
    let state = widget.build(&mut ctx);
    let component = state.component().clone();

    // Child creation and attachment
    {
        let child = {
            let widget = rvue::widgets::Text::new("Child 1".to_string());
            let state = widget.build(&mut ctx);
            state.component().clone()
        };
        component.add_child(child);
    }

    component
}
```

## Important Notes

- **One-time execution**: The setup code in the expansion runs only once during the initial component creation.
- **Fine-grained updates**: Only the specific `create_effect` blocks run when signals change, targeting the exact `set_*` methods on the components.
- **GC Managed**: All components and effects are stored in `Gc<T>` pointers, allowing for cycles and shared ownership without manual lifetime management.
