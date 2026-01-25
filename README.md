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

## 🛠️ Basic Usage (Experimental)

```rust
use rvue::prelude::*;
use rvue_macro::{view, component};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rvue::run_app(|| App())?;
    Ok(())
}

fn App() -> ViewStruct {
    view! {
        <Flex direction="column" gap=20.0 align_items="center">
            <Text content="My First Rvue App" />
            <Counter />
        </Flex>
    }
}

#[component]
fn Counter() -> ViewStruct {
    let (count, set_count) = create_signal(0);
    let set_count_inc = set_count.clone();

    view! {
        <Flex direction="row" gap=10.0 align_items="center">
            <Text content={format!("Count: {}", count.get())} />
            <Button label="Increment" 
                on_click={move || set_count_inc.update(|x| *x += 1)} 
            />
        </Flex>
    }
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
