//! Basic styling example for rvue-style.

use rvue_style::{
    selectors::ElementState, stylesheet::parser::parse_stylesheet, Color, Margin, Padding,
    Properties,
};

fn main() {
    // Create properties manually using Property trait
    let mut props = Properties::new();
    props.insert(Color::rgb(255, 0, 0));
    props.insert(Padding(16.0));
    props.insert(Margin(8.0));

    // Access properties by type
    if let Some(color) = props.get::<Color>() {
        println!("Color: r={}, g={}, b={}", color.0.r, color.0.g, color.0.b);
    }
    if let Some(padding) = props.get::<Padding>() {
        println!("Padding: {:?}", padding.0);
    }

    // Parse a stylesheet from CSS
    let css = r#"
        button {
            background-color: red;
            color: white;
            padding: 10px;
            margin: 5px;
        }
    "#;

    match parse_stylesheet(css) {
        Some(stylesheet) => {
            println!("Parsed stylesheet with {} rules", stylesheet.len());
            for rule in stylesheet.rules() {
                println!("Selector: {}, Specificity: {:?}", rule.selector, rule.specificity);
            }
        }
        None => {
            println!("Failed to parse stylesheet");
        }
    }

    // Element state for pseudo-class matching
    let mut state = ElementState::empty();
    state.insert(ElementState::HOVER);
    state.insert(ElementState::FOCUS);

    println!("Element state: {:?}", state);
    println!("Has HOVER: {}", state.contains(ElementState::HOVER));
    println!("Has ACTIVE: {}", state.contains(ElementState::ACTIVE));

    println!("\nBasic styling example completed!");
}
