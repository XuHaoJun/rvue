# Rvue

A high-performance, GPU-accelerated Rust GUI framework inspired by **Vue**'s developer experience and **SolidJS**'s fine-grained reactivity.

## 🚀 Vision

Rvue aims to be the "Holy Grail" of Rust GUI development:
- **Write like TypeScript/Solid**: No lifetime headaches, thanks to a hybrid GC ([rudo-gc](https://github.com/XuHaoJun/rudo)).
- **Run like C++**: No Virtual DOM. Direct GPU-accelerated rendering via **Vello**.
- **Layout like CSS**: Flexible and powerful layouts powered by **Taffy**.

For more details on the vision and technical roadmap, see the [Vision Document](docs/2026-01-17_17-30-40_Gemini_Google_Gemini.md).

## ✨ Features

- **Fine-Grained Reactivity**: Signals and effects for efficient state management without full-tree re-renders.
- **GPU-Powered Rendering**: Utilizes **Vello** for high-quality, high-performance 2D vector graphics.
- **Flexible Layouts**: Integrated **Taffy** for Flexbox and Grid layout systems.
- **Component-Based**: Familiar component architecture for building reusable UI building blocks.
- **Native Performance**: Built from the ground up in Rust for maximum performance and efficiency.

## 🛠️ Basic Example

Stop worrying about lifetimes. Start building beautiful, native UI with a familiar Vue-like developer experience.

```rust
use rvue::prelude::*;
use rvue_macro::{view, component};
use rvue_style::{BackgroundColor, Color, BorderRadius, Padding, ReactiveStyles, TextColor, FontWeight, FontSize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rvue::run_app(|| view! {
        <Flex direction="column" gap=30.0 align_items="center" justify_content="center">
            <Text content="Welcome to Rvue" styles=header_style() />
            <InteractiveCard />
        </Flex>
    })?;
    Ok(())
}

#[component]
fn InteractiveCard() -> ViewStruct {
    let (count, set_count) = create_signal(0);
    let (items, set_items) = create_signal(vec!["Initial state".to_string()]);
    
    // Derived state - automatic dependency tracking
    let is_active = create_memo(move || count.get() > 0);

    view! {
        <Flex direction="column" gap=20.0 styles=card_style()>
            <Text content={move || format!("Updates: {}", count.get())} />
            
            // Conditional Rendering - One-line logic
            <Show when=is_active>
                <Text content="System Active 🚀" styles=text_style(Color::rgb(40, 167, 69)) />
            </Show>

            <Button on_click=move || {
                set_count.update(|n| *n += 1);
                set_items.update(|list| list.push(format!("Update #{}", count.get_untracked())));
            } styles=button_style()>
                <Text content="Push Update" styles=text_style(Color::rgb(255, 255, 255)) />
            </Button>

            // List Rendering - High-performance For loop
            <Flex direction="column" gap=8.0>
                <Text content="History Log:" styles=label_style() />
                <For each=items key=|item: &String| item.clone() view=|item| {
                    view! { <Text content=format!("• {}", item) /> }
                } />
            </Flex>
        </Flex>
    }
}

// Clean, reusable styling
fn card_style() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
        .set_padding(Padding(20.0))
        .set_border_radius(BorderRadius(12.0))
}

fn header_style() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_font_size(FontSize(32.0))
        .set_font_weight(FontWeight::Bold)
        .set_text_color(TextColor(Color::rgb(33, 37, 41)))
}

fn label_style() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_font_weight(FontWeight::Bold)
        .set_text_color(TextColor(Color::rgb(108, 117, 125)))
}

fn button_style() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
        .set_padding(Padding(12.0))
        .set_border_radius(BorderRadius(8.0))
}

fn text_style(color: Color) -> ReactiveStyles {
    ReactiveStyles::new().set_text_color(TextColor(color))
}
```



## 📦 Project Structure

- `crates/rvue`: The core framework library.
- `crates/rvue-macro`: Procedural macros for the declarative `view!` syntax.
- `crates/rvue-examples`: A collection of example applications demonstrating various features.
    - `counter`: Basic counter with reactive state.
    - `list`: Dynamic list rendering.
    - `layout`: Complex nested Flexbox layouts.
    - `form`: Form inputs and state management.
    - `benchmark`: Performance benchmarks for startup and memory.

## 🏁 Getting Started

### Prerequisites

- Rust 1.75+
- A GPU with support for compute shaders (required by Vello)

### Running Examples

You can run the examples from the root directory using Cargo:

```bash
cargo run --bin counter
cargo run --bin list
cargo run --bin layout
cargo run --bin form
```



## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.

---

*Note: Rvue is currently in early development (MVP stage).*
