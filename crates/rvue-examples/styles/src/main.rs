//! Complex styling showcase demonstrating Rvue's styling system

use rvue::prelude::*;
use rvue_macro::view;
#[allow(unused_imports)]
use rvue_style::{
    BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth, Color, FontSize,
    ReactiveStyles, TextColor,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rudo_gc::test_util::reset();

    let styled_view = create_styled_view();
    rvue::run_app(|| styled_view)?;
    Ok(())
}

#[allow(dead_code)]
fn text_style(color: TextColor) -> ReactiveStyles {
    ReactiveStyles::new().set_text_color(color)
}

fn create_styled_view() -> ViewStruct {
    let _border_radius = 8.0;

    view! {
        <Flex
            direction="column"
            gap=0.0
            align_items="stretch"
            justify_content="start"
            styles=ReactiveStyles::new()
                .set_background_color(BackgroundColor(Color::rgb(250, 250, 250)))
        >
            <Text
                content="Rvue Styling System Showcase"
                style=text_style(TextColor(Color::rgb(33, 37, 41)))
            />

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="Theme:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Text content="[Light / Dark toggle buttons]" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
            </Flex>

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="Font Size Examples:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Flex direction="row" gap=8.0 align_items="center" justify_content="start">
                    <Text content="12px" style=ReactiveStyles::new().set_font_size(FontSize(12.0)) />
                    <Text content="Aa" style=text_style(TextColor(Color::rgb(33, 37, 41))) />
                    <Text content="24px" style=ReactiveStyles::new().set_font_size(FontSize(24.0)) />
                    <Text content="48px" style=ReactiveStyles::new().set_font_size(FontSize(48.0)) />
                </Flex>
            </Flex>

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="Border Radius Examples:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Flex direction="row" gap=16.0 align_items="center" justify_content="start">
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        .set_border_radius(BorderRadius(0.0))
                        width=Size::Pixels(40.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="0" style=text_style(TextColor(Color::rgb(255, 255, 255))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        .set_border_radius(BorderRadius(4.0))
                        width=Size::Pixels(40.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="4" style=text_style(TextColor(Color::rgb(255, 255, 255))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        .set_border_radius(BorderRadius(8.0))
                        width=Size::Pixels(40.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="8" style=text_style(TextColor(Color::rgb(255, 255, 255))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        .set_border_radius(BorderRadius(16.0))
                        width=Size::Pixels(40.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="16" style=text_style(TextColor(Color::rgb(255, 255, 255))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        .set_border_radius(BorderRadius(32.0))
                        width=Size::Pixels(40.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="32" style=text_style(TextColor(Color::rgb(255, 255, 255))) />
                    </Flex>
                </Flex>
            </Flex>

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="Border Color Examples:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Flex direction="row" gap=16.0 align_items="center" justify_content="start">
                    <Flex styles=ReactiveStyles::new()
                        .set_border_color(BorderColor(Color::rgb(200, 200, 200)))
                        .set_border_width(BorderWidth(2.0))
                        .set_border_style(BorderStyle::Solid)
                        .set_border_radius(BorderRadius(4.0))
                        width=Size::Pixels(60.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="Default" style=text_style(TextColor(Color::rgb(100, 100, 100))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_border_color(BorderColor(Color::rgb(0, 123, 255)))
                        .set_border_width(BorderWidth(2.0))
                        .set_border_style(BorderStyle::Solid)
                        .set_border_radius(BorderRadius(4.0))
                        width=Size::Pixels(60.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="Primary" style=text_style(TextColor(Color::rgb(0, 123, 255))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_border_color(BorderColor(Color::rgb(40, 167, 69)))
                        .set_border_width(BorderWidth(2.0))
                        .set_border_style(BorderStyle::Solid)
                        .set_border_radius(BorderRadius(4.0))
                        width=Size::Pixels(60.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="Success" style=text_style(TextColor(Color::rgb(40, 167, 69))) />
                    </Flex>
                    <Flex styles=ReactiveStyles::new()
                        .set_border_color(BorderColor(Color::rgb(220, 53, 69)))
                        .set_border_width(BorderWidth(2.0))
                        .set_border_style(BorderStyle::Solid)
                        .set_border_radius(BorderRadius(4.0))
                        width=Size::Pixels(60.0)
                        height=Size::Pixels(40.0)
                    >
                        <Text content="Danger" style=text_style(TextColor(Color::rgb(220, 53, 69))) />
                    </Flex>
                </Flex>
            </Flex>

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="Color Palette:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Flex direction="row" gap=12.0 align_items="center" justify_content="start">
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(0, 123, 255)))
                        width=Size::Pixels(48.0)
                        height=Size::Pixels(48.0)
                    />
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(40, 167, 69)))
                        width=Size::Pixels(48.0)
                        height=Size::Pixels(48.0)
                    />
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(255, 193, 7)))
                        width=Size::Pixels(48.0)
                        height=Size::Pixels(48.0)
                    />
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(220, 53, 69)))
                        width=Size::Pixels(48.0)
                        height=Size::Pixels(48.0)
                    />
                    <Flex styles=ReactiveStyles::new()
                        .set_background_color(BackgroundColor(Color::rgb(23, 162, 184)))
                        width=Size::Pixels(48.0)
                        height=Size::Pixels(48.0)
                    />
                </Flex>
            </Flex>
        </Flex>
    }
}
