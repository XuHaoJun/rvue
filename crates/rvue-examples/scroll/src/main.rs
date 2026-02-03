//! Rvue Scroll Example
//!
//! This example demonstrates the scroll/overflow functionality in Rvue.
//! It shows how to create scrollable containers with different overflow modes.

use rvue::prelude::*;
use rvue_macro::view;
use rvue_style::properties::Overflow;
use rvue_style::{BorderWidth, Height, ReactiveStyles, Size, Width};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rudo_gc::test_util::reset();

    rvue::run_app(|| create_scroll_view())?;
    Ok(())
}

fn create_scroll_view() -> ViewStruct {
    let (overflow_mode, set_overflow_mode) = create_signal(Overflow::Auto);
    let overflow_mode_for_memo = overflow_mode.clone();
    let overflow_mode_for_flex = overflow_mode.clone();

    let set_visible = set_overflow_mode.clone();
    let set_hidden = set_overflow_mode.clone();
    let set_auto = set_overflow_mode.clone();
    let set_scroll = set_overflow_mode.clone();

    view! {
        <Flex
            direction="column"
            gap=16.0
            align_items="stretch"
            justify_content="start"
            styles=ReactiveStyles::new()
                .set_background_color(BackgroundColor(Color::rgb(245, 245, 245)))
                .set_padding(Padding(24.0))
        >
            <Text
                content="Rvue Scroll Demo"
                styles=ReactiveStyles::new()
                    .set_font_size(24.0)
                    .set_font_weight(700)
                    .set_text_color(TextColor(Color::rgb(33, 37, 41)))
            />

            <Text
                content="Overflow Mode:"
                styles=ReactiveStyles::new()
                    .set_font_size(16.0)
                    .set_text_color(TextColor(Color::rgb(73, 80, 87)))
            />

            <Flex direction="row" gap=8.0 align_items="center" justify_content="start">
                <Button on_click=move || set_visible.set(Overflow::Visible)>
                    <Text content="Visible" />
                </Button>
                <Button on_click=move || set_hidden.set(Overflow::Hidden)>
                    <Text content="Hidden" />
                </Button>
                <Button on_click=move || set_auto.set(Overflow::Auto)>
                    <Text content="Auto" />
                </Button>
                <Button on_click=move || set_scroll.set(Overflow::Scroll)>
                    <Text content="Scroll" />
                </Button>
            </Flex>

            <Text
                content=create_memo(move || format!("Current Mode: {:?}", overflow_mode_for_memo.get()))
                styles=ReactiveStyles::new()
                    .set_font_size(14.0)
                    .set_text_color(TextColor(Color::rgb(0, 123, 255)))
            />

            <Text
                content="Scroll the container below:"
                styles=ReactiveStyles::new()
                    .set_font_size(14.0)
                    .set_text_color(TextColor(Color::rgb(108, 117, 125)))
            />

            <Flex
                direction="column"
                gap=0.0
                align_items="stretch"
                justify_content="start"
                overflow_y=overflow_mode_for_flex
                styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(300.0)))
                    .set_width(Width(Size::Pixels(400.0)))
                    .set_background_color(BackgroundColor(Color::rgb(255, 182, 193)))
                    .set_border_radius(BorderRadius(8.0))
                    .set_border_width(BorderWidth(1.0))
                    .set_border_style(BorderStyle::Solid)
                    .set_border_color(BorderColor(Color::rgb(200, 200, 200)))
            >
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 1 - Pink background is scrollable container" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 2 - Try scrolling with mouse wheel" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 3 - Click and drag scrollbar when visible" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 4 - Hidden: content clipped, no scrollbar" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 5 - Auto: scrollbar when needed" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 6 - Scroll: always shows scrollbar" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 7 - Scroll behavior matches CSS" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 8 - Fine-grained reactivity" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 9 - GPU-accelerated rendering" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 10 - Rust + Vello rendering" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 11 - Component-based architecture" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 12 - Declarative UI with view! macro" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 13 - Inspired by Vue and SolidJS" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 14 - Taffy for CSS-like layouts" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 15 - Hybrid GC with rudo-gc" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 16 - Type-safe styling system" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 17 - Reactive signal updates" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 18 - Effect tracking for reactivity" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(255, 255, 255)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 19 - End of scrollable content" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
                <Flex styles=ReactiveStyles::new()
                    .set_height(Height(Size::Pixels(40.0)))
                    .set_padding(Padding(8.0))
                    .set_background_color(BackgroundColor(Color::rgb(240, 240, 240)))
                    .set_width(Width(Size::Percent(100.0)))
                >
                    <Text content="Item 20 - Thank you for trying Rvue!" styles=ReactiveStyles::new().set_text_color(TextColor(Color::rgb(50, 50, 50))) />
                </Flex>
            </Flex>

            <Text
                content="Instructions: Click buttons to change overflow mode. Scroll with mouse wheel or drag scrollbar."
                styles=ReactiveStyles::new()
                    .set_font_size(12.0)
                    .set_text_color(TextColor(Color::rgb(134, 142, 150)))
            />
        </Flex>
    }
}
