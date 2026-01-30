//! Complex styling showcase demonstrating Rvue's styling system
//!
//! This example demonstrates both:
//! 1. Inline styling with ReactiveStyles (typed, reactive properties)
//! 2. CSS selector matching with Stylesheet (class/ID selectors, pseudo-classes)

use rvue::prelude::*;
use rvue::Stylesheet;
use rvue_macro::view;
use rvue_style::{
    BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth, Color, FontSize, Margin,
    Properties, ReactiveStyles, TextColor,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rudo_gc::test_util::reset();

    // Create a stylesheet with CSS rules for selector-based styling
    let mut stylesheet = Stylesheet::new();

    // Primary button styles (class selector)
    stylesheet.add_interactive_colors(
        "button.primary",
        Color::rgb(0, 123, 255),   // Normal: Blue
        Color::rgb(0, 86, 179),    // Hover: Darker blue
        Color::rgb(0, 65, 130),    // Active: Even darker
        Color::rgb(200, 200, 200), // Disabled: Gray
    );
    let mut props = Properties::new();
    props.insert(BorderRadius(8.0));
    stylesheet.add_rule("button.primary", props);

    // Secondary button styles
    stylesheet.add_interactive_colors(
        "button.secondary",
        Color::rgb(108, 117, 125), // Normal: Gray
        Color::rgb(90, 98, 104),   // Hover: Darker gray
        Color::rgb(70, 78, 84),    // Active: Even darker
        Color::rgb(200, 200, 200), // Disabled: Gray
    );
    let mut props = Properties::new();
    props.insert(BorderRadius(8.0));
    stylesheet.add_rule("button.secondary", props);

    // Danger button styles
    stylesheet.add_interactive_colors(
        "button.danger",
        Color::rgb(220, 53, 69),   // Normal: Red
        Color::rgb(189, 37, 50),   // Hover: Darker red
        Color::rgb(158, 28, 40),   // Active: Even darker
        Color::rgb(200, 200, 200), // Disabled: Gray
    );
    let mut props = Properties::new();
    props.insert(BorderRadius(8.0));
    stylesheet.add_rule("button.danger", props);

    // Success button styles
    stylesheet.add_interactive_colors(
        "button.success",
        Color::rgb(40, 167, 69),   // Normal: Green
        Color::rgb(30, 126, 52),   // Hover: Darker green
        Color::rgb(25, 105, 43),   // Active: Even darker
        Color::rgb(200, 200, 200), // Disabled: Gray
    );
    let mut props = Properties::new();
    props.insert(BorderRadius(8.0));
    stylesheet.add_rule("button.success", props);

    // Special ID button styles
    stylesheet.add_background_color_with_hover(
        "#special-button",
        Color::rgb(255, 193, 7), // Normal: Gold
        Color::rgb(255, 160, 0), // Hover: Orange
    );
    let mut props = Properties::new();
    props.insert(BorderRadius(12.0));
    stylesheet.add_rule("#special-button", props);

    // Large button variant
    stylesheet.add_background_color("button.large", Color::rgb(23, 162, 184)); // Cyan
    let mut props = Properties::new();
    props.insert(BorderRadius(12.0));
    stylesheet.add_rule("button.large", props);

    let styled_view = create_styled_view();

    // Run with stylesheet for CSS selector matching
    rvue::run_app_with_stylesheet(|| styled_view, Some(stylesheet))?;
    Ok(())
}

#[allow(dead_code)]
fn text_style(color: TextColor) -> ReactiveStyles {
    ReactiveStyles::new().set_text_color(color)
}

fn create_styled_view() -> ViewStruct {
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

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="CSS Selector Matching (hover buttons below):" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Text content="These buttons demonstrate class and ID selectors with pseudo-classes" style=text_style(TextColor(Color::rgb(134, 142, 150))) />

                <Flex direction="row" gap=16.0 align_items="center" justify_content="start">
                    <Button label="Primary" class="primary" on_click=move || { println!("Primary clicked!"); } />
                    <Button label="Secondary" class="secondary" on_click=move || { println!("Secondary clicked!"); } />
                    <Button label="Danger" class="danger" on_click=move || { println!("Danger clicked!"); } />
                    <Button label="Success" class="success" on_click=move || { println!("Success clicked!"); } />
                </Flex>

                <Flex direction="row" gap=16.0 align_items="center" justify_content="start" styles=ReactiveStyles::new().set_margin(Margin(8.0))>
                    <Text content="ID Selector:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                    <Button label="Special Gold Button" id="special-button" on_click=move || { println!("Special button clicked!"); } />
                </Flex>

                <Flex direction="row" gap=16.0 align_items="center" justify_content="start" styles=ReactiveStyles::new().set_margin(Margin(8.0))>
                    <Button label="Disabled" class="primary" disabled=true on_click=move || {} />
                    <Button label="Large" class="primary large" on_click=move || {} />
                </Flex>
            </Flex>

            <Flex direction="column" gap=16.0 align_items="stretch" justify_content="start">
                <Text content="How CSS Selector Matching Works:" style=text_style(TextColor(Color::rgb(73, 80, 87))) />
                <Text content="1. Create a Stylesheet and add CSS rules with selectors" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
                <Text content="2. Use class=\"classname\" on components for class selectors" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
                <Text content="3. Use id=\"idname\" for ID selectors (#idname)" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
                <Text content="4. Pseudo-classes (:hover, :focus, :active, :disabled) are tracked automatically" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
                <Text content="5. Run app with run_app_with_stylesheet(view, Some(stylesheet))" style=text_style(TextColor(Color::rgb(134, 142, 150))) />
            </Flex>
        </Flex>
    }
}
