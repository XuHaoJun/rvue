//! Reactive styling example for rvue-style.
//!
//! This example demonstrates how to create reactive signals for styling
//! and build reactive style objects that can be computed on demand.

use rvue_style::{
    reactive::{create_reactive_signal, ReactiveStyles},
    BackgroundColor, Color, ComputedStyles, Padding,
};

fn main() {
    println!("=== Reactive Styles Example ===\n");

    // Example 1: Basic reactive signal creation
    println!("1. Creating Reactive Signals\n");

    let (theme_color, set_theme_color) = create_reactive_signal(Color::rgb(66, 133, 244));
    let (is_dark_mode, set_dark_mode) = create_reactive_signal(false);
    let (counter, set_counter) = create_reactive_signal(0);

    println!("Initial theme color: {:?}", theme_color.get().0);
    println!("Dark mode: {}", is_dark_mode.get());
    println!("Counter: {}", counter.get());
    println!();

    // Update signals using .set() method on write signal
    set_theme_color.set(Color::rgb(231, 76, 60));
    set_dark_mode.set(true);
    set_counter.set(5);

    println!("Updated theme color: {:?}", theme_color.get().0);
    println!("Dark mode: {}", is_dark_mode.get());
    println!("Counter: {}", counter.get());
    println!();

    // Example 2: ReactiveStyles builder
    println!("2. Building Reactive Styles\n");

    // Static values work directly
    let static_styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor(Color::rgb(66, 133, 244)))
        .set_padding(Padding(16.0))
        .compute();

    println!("Static styles computed:");
    println!("  Background: {:?}", static_styles.background_color.map(|c| c.0));
    println!("  Padding: {:?}", static_styles.padding);
    println!();

    // Example 3: Pattern - Dynamic styling based on signals
    println!("3. Dynamic Styling Pattern\n");

    // Create signals for dynamic styling
    let (bg_signal, set_bg_signal) =
        create_reactive_signal(BackgroundColor(Color::rgb(66, 133, 244)));
    let (text_signal, set_text_signal) = create_reactive_signal(Color::rgb(255, 255, 255));

    // Build styles - when signals change, rebuild to get updated values
    let button_styles = ReactiveStyles::new()
        .set_background_color(bg_signal.get())
        .set_color(text_signal.get())
        .compute();

    println!("Button styles (initial):");
    println!("  Background: {:?}", button_styles.background_color.map(|c| c.0));
    println!();

    // Update signals
    set_bg_signal.set(BackgroundColor(Color::rgb(46, 204, 113)));
    set_text_signal.set(Color::rgb(0, 0, 0));

    // Rebuild styles to get updated values
    let updated_button_styles = ReactiveStyles::new()
        .set_background_color(bg_signal.get())
        .set_color(text_signal.get())
        .compute();

    println!("Button styles (after signal update):");
    println!("  Background: {:?}", updated_button_styles.background_color.map(|c| c.0));
    println!("  Color: {:?}", updated_button_styles.color.map(|c| c.0));
    println!();

    // Example 4: Value-based styling (e.g., progress indicator)
    println!("4. Value-Based Styling\n");

    let (_progress, set_progress) = create_reactive_signal(0.0);

    for p in [0.0, 0.25, 0.5, 0.75, 1.0] {
        set_progress.set(p);
        let progress_color = if p > 0.5 { "green" } else { "yellow" };
        println!("Progress {:.0}%: color = {}", p * 100.0, progress_color);
    }
    println!();

    // Example 5: Style effects are available for internal use
    println!("5. Style Effects\n");
    println!("  (create_style_effect is available for internal styling)\n");

    // Example 6: Computed styles with signals
    println!("6. Computed Styles Pattern\n");

    let (dynamic_color, set_dynamic_color) = create_reactive_signal(Color::rgb(66, 133, 244));

    fn create_button_styles(bg_color: &Color) -> ComputedStyles {
        let mut computed = ComputedStyles::new();
        computed.background_color = Some(BackgroundColor(*bg_color));
        computed.color = Some(Color::rgb(255, 255, 255));
        computed.padding = Some(Padding(12.0));
        computed
    }

    let initial = create_button_styles(&dynamic_color.get());
    println!("Button computed styles:");
    println!("  Background: {:?}", initial.background_color.map(|c| c.0));
    println!();

    set_dynamic_color.set(Color::rgb(231, 76, 60));
    let updated = create_button_styles(&dynamic_color.get());
    println!("Button computed styles (updated):");
    println!("  Background: {:?}", updated.background_color.map(|c| c.0));
    println!();

    println!("Reactive styles example completed!");
}
