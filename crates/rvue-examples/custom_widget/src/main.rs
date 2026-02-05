//! Custom Widget Example
//!
//! This example demonstrates how to create custom widgets in Rvue.
//!
//! Key Patterns:
//! 1. Zero-arg components - simple widgets with no props
//! 2. Internal state with signals - widgets that manage their own state
//! 3. Compose widgets together - building larger UIs from smaller pieces

#![recursion_limit = "256"]
#![allow(unused_imports, unused_variables, dead_code)]

use rvue::prelude::*;
use rvue_macro::{component, view};
use rvue_style::{
    properties::{Height, Padding, Width},
    ReactiveStyles,
};

// ============================================================================
// Pattern 1: Zero-Arg Component
// ============================================================================

/// A simple card component with no props
///
/// Zero-arg components work perfectly with #[component]
/// They're great for reusable UI pieces that don't need external data.
#[component]
fn SimpleCard() -> impl View {
    view! {
        <Flex direction="column" gap=10.0 padding=10.0>
            <Text content="Simple Card" />
            <Text content="No props needed!" />
        </Flex>
    }
}

// ============================================================================
// Pattern 2: Component with Internal State (Signals)
// ============================================================================

/// A counter component that uses signals internally
///
/// Components can create their own signals to manage state.
/// The view automatically updates when signals change.
#[component]
fn Counter() -> impl View {
    let (count, set_count) = create_signal(0);
    let set_count_clone = set_count.clone();

    view! {
        <Flex direction="column" gap=10.0 padding=10.0>
            <Text content="Counter Example" />
            <Text content=format!("Count: {}", count.get()) />
            <Flex gap=10.0>
                <Button on_click=move || set_count.update(|c| *c -= 1)>
                    <Text content="-" />
                </Button>
                <Button on_click=move || set_count_clone.update(|c| *c += 1)>
                    <Text content="+" />
                </Button>
            </Flex>
        </Flex>
    }
}

// ============================================================================
// Pattern 3: Component with Multiple Signals
// ============================================================================

/// A toggle component demonstrating multiple signals
#[component]
fn Toggle() -> impl View {
    let (is_on, set_is_on) = create_signal(false);
    let is_on_clone = is_on.clone();

    let status_text = create_memo(move || if is_on.get() { "ON" } else { "OFF" });

    view! {
        <Flex direction="column" gap=10.0 padding=10.0>
            <Text content="Toggle Switch" />
            <Text content=status_text.get() />
            <Button on_click=move || set_is_on.update(|b| *b = !*b)>
                <Text content="Toggle" />
            </Button>
        </Flex>
    }
}

// ============================================================================
// Pattern 4: Compose Widgets Together
// ============================================================================

/// A dashboard that composes multiple custom widgets
///
/// Components can freely compose other components.
/// This is how you build larger UIs from smaller pieces.
#[component]
fn Dashboard() -> impl View {
    view! {
        <Flex direction="column" gap=30.0 align_items="center" justify_content="center" style=container_styles()>
            <Text content="Custom Widget Composition" style=title_styles() />

            // Pattern 1: Simple zero-arg component
            <SimpleCard />

            // Pattern 2: Component with internal state
            <Counter />

            // Pattern 3: Another stateful component
            <Toggle />
        </Flex>
    }
}

// ============================================================================
// Styles
// ============================================================================

fn container_styles() -> ReactiveStyles {
    ReactiveStyles::new()
        .set_height(Height(rvue_style::Size::Pixels(600.0)))
        .set_width(Width(rvue_style::Size::Pixels(500.0)))
        .set_padding(Padding(20.0))
}

fn title_styles() -> ReactiveStyles {
    ReactiveStyles::new().set_height(Height(rvue_style::Size::Pixels(40.0)))
}

// ============================================================================
// Main App
// ============================================================================

#[component]
fn App() -> impl View {
    view! { <Dashboard /> }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_view = App(Default::default());
    let app_component = app_view.into_component();
    let app_view_struct = ViewStruct::new(app_component);
    rvue::run_app(|| app_view_struct)?;
    Ok(())
}

// ============================================================================
// Summary: Custom Component Patterns in Rvue
// ============================================================================
//
// PATTERN 1: Zero-Arg Components
// ----------------------------
// #[component]
// fn MyCard() -> impl View {
//     view! { <Text content="Hello" /> }
// }
//
// Usage: <MyCard />
//
// PATTERN 2: Internal State with Signals
// ----------------------------------
// #[component]
// fn Counter() -> impl View {
//     let (count, set_count) = create_signal(0);
//     let set_count = set_count.clone();  // Clone for multiple closures
//     view! { ... }
// }
//
// PATTERN 3: Memoized Computations
// ------------------------------
// let value = create_memo(move || { compute() });
// view! { <Text content=value.get() /> }
//
// PATTERN 4: Compose Existing Widgets
// --------------------------------
// #[component]
// fn Dashboard() -> impl View {
//     view! {
//         <SimpleCard />
//         <Counter />
//     }
// }
//
// KEY INSIGHTS:
// 1. #[component] generates a Props struct from function parameters
// 2. Zero-arg components are the simplest and most common pattern
// 3. Clone signals before using in multiple closures
// 4. Use create_memo for derived/computed values
// 5. Components freely compose other components
