//! CSS Stylesheet parsing and selector matching example.
//!
//! This example demonstrates how to parse CSS stylesheets, define style rules
//! with various selectors, and use the StyleResolver to match elements.

use rvue_style::{
    selectors::{ElementState, RvueElement},
    stylesheet::{parser::parse_stylesheet, resolver::StyleResolver, StyleRule, Stylesheet},
    BackgroundColor, Color, Height, Padding, Properties, Property, Width,
};

fn main() {
    println!("=== CSS Stylesheet Example ===\n");

    // Parse a CSS stylesheet
    let css = r#"
        /* Button styles */
        button {
            background-color: #3498db;
            color: white;
            padding: 12px;
        }

        button:hover {
            background-color: #2980b9;
        }

        button:active {
            background-color: #1a5276;
        }

        button:disabled {
            background-color: #95a5a6;
        }

        /* Card styles */
        .card {
            background-color: white;
        }

        .card.highlighted {
            background-color: #f0f0f0;
        }

        /* ID selector */
        #header {
            background-color: #2c3e50;
            color: white;
            height: 60px;
        }

        /* Combined selectors */
        button.primary {
            background-color: #e74c3c;
        }
    "#;

    match parse_stylesheet(css) {
        Ok(stylesheet) => {
            println!("Parsed {} style rules:\n", stylesheet.len());

            // Display all parsed rules (rules() returns an iterator directly)
            for (index, rule) in stylesheet.rules().enumerate() {
                println!("Rule {}: {:?}", index + 1, rule.selector);
                println!("  Specificity: {:?}", rule.specificity);
                println!("  Properties count: {}", rule.properties.len());
                println!();
            }
        }
        Err(()) => {
            println!("Failed to parse stylesheet");
            return;
        }
    }

    // Create a stylesheet and manually add rules
    println!("=== Manual Stylesheet Creation ===\n");

    let mut stylesheet = Stylesheet::new();

    // Build Properties and create StyleRule
    let mut button_props = Properties::new();
    button_props.insert(BackgroundColor(Color::rgb(52, 152, 219)));
    button_props.insert(Padding(12.0));
    button_props.insert(Width(Size::pixels(120.0)));
    stylesheet.add_rule(StyleRule::parse("button", button_props));

    // Add class selector rule
    let mut card_props = Properties::new();
    card_props.insert(BackgroundColor(Color::rgb(255, 255, 255)));
    card_props.insert(Height(Size::percent(100.0)));
    stylesheet.add_rule(StyleRule::parse(".card", card_props));

    println!("Stylesheet has {} rules\n", stylesheet.len());

    // Create element and test selector matching
    println!("=== Selector Matching ===\n");

    let resolver = StyleResolver::new();

    // Build a stylesheet with test rules
    let mut test_sheet = Stylesheet::new();

    let mut props1 = Properties::new();
    props1.insert(BackgroundColor(Color::rgb(0, 0, 255)));
    test_sheet.add_rule(StyleRule::parse("button", props1));

    let mut props2 = Properties::new();
    props2.insert(BackgroundColor(Color::rgb(255, 0, 0)));
    test_sheet.add_rule(StyleRule::parse(".primary", props2));

    let mut props3 = Properties::new();
    props3.insert(BackgroundColor(Color::rgb(0, 255, 0)));
    test_sheet.add_rule(StyleRule::parse("button.primary", props3));

    let mut props4 = Properties::new();
    props4.insert(BackgroundColor(Color::rgb(128, 0, 128)));
    test_sheet.add_rule(StyleRule::parse("button:hover", props4));

    // Test button element without classes
    let button = RvueElement::new("button");

    let resolved = resolver.resolve_styles(&button, &test_sheet);
    println!("Button (no classes, no state):");
    if let Some(bg) = resolved.background_color {
        println!("  Resolved background: {:?}", bg.0);
    }
    println!();

    // Test button with primary class (using builder pattern)
    let button_with_class = RvueElement::new("button").with_class("primary");

    let resolved = resolver.resolve_styles(&button_with_class, &test_sheet);
    println!("Button.primary:");
    if let Some(bg) = resolved.background_color {
        println!("  Resolved background: {:?}", bg.0);
    }
    println!();

    // Test button with hover state
    let button_hover = RvueElement::new("button");
    let mut state = ElementState::empty();
    state.insert(ElementState::HOVER);

    let resolved = resolver.resolve_styles(&button_hover, &test_sheet);
    println!("Button:hover:");
    if let Some(bg) = resolved.background_color {
        println!("  Resolved background: {:?}", bg.0);
    }
    println!();

    // Demonstrate specificity ordering
    println!("=== Specificity Demonstration ===\n");
    println!("When multiple rules match, higher specificity wins:");
    println!("  - Element selector: (0, 0, 1)");
    println!("  - Class selector: (0, 1, 0)");
    println!("  - ID selector: (1, 0, 0)");
    println!();

    println!("Stylesheet example completed!");
}

use rvue_style::Size;
