//! CSS stylesheet parsing.

use crate::properties::{Color, Height, Margin, Padding, Size, Width};
use crate::property::{Properties, Property};
use crate::stylesheet::rule::{StyleRule, Stylesheet};

/// Parses a CSS stylesheet from a string.
pub fn parse_stylesheet(css: &str) -> Result<Stylesheet, ()> {
    let mut stylesheet = Stylesheet::new();
    let mut position = 0;

    while position < css.len() {
        // Skip whitespace
        while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
            position += 1;
        }

        if position >= css.len() {
            break;
        }

        // Skip comments
        if css[position..].starts_with("/*") {
            if let Some(end) = css[position..].find("*/") {
                position += end + 2;
                continue;
            } else {
                return Err(());
            }
        }

        // Parse selector
        let selector_start = position;
        while position < css.len() && !css[position..].starts_with('{') {
            position += 1;
        }
        let selector = css[selector_start..position].trim();

        if selector.is_empty() {
            return Err(());
        }

        // Expect opening brace
        if position >= css.len() || !css[position..].starts_with('{') {
            return Err(());
        }
        position += 1;

        // Parse properties
        let mut properties = Properties::new();

        while position < css.len() {
            // Skip whitespace
            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            if position >= css.len() {
                return Err(());
            }

            // Check for closing brace
            if css[position..].starts_with('}') {
                position += 1;
                break;
            }

            // Skip comments
            if css[position..].starts_with("/*") {
                if let Some(end) = css[position..].find("*/") {
                    position += end + 2;
                    continue;
                } else {
                    return Err(());
                }
            }

            // Parse property name
            let name_start = position;
            while position < css.len()
                && !css[position..].starts_with(|c: char| c.is_whitespace() || c == ':')
            {
                position += 1;
            }
            let name = css[name_start..position].trim();

            if name.is_empty() {
                return Err(());
            }

            // Skip whitespace
            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            // Expect colon
            if position >= css.len() || !css[position..].starts_with(':') {
                return Err(());
            }
            position += 1;

            // Skip whitespace
            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            // Parse value
            let value_start = position;
            while position < css.len()
                && !css[position..].starts_with(|c: char| c.is_whitespace() || c == ';' || c == '}')
            {
                position += 1;
            }
            let value = css[value_start..position].trim();

            // Skip optional semicolon
            if position < css.len() && css[position..].starts_with(';') {
                position += 1;
            }

            // Create property
            if let Some(prop) = create_property(name, value) {
                properties.insert(prop);
            }
        }

        stylesheet.add_rule(StyleRule::new(selector.to_string(), properties));
    }

    Ok(stylesheet)
}

/// Creates a property from a name-value pair.
fn create_property(name: &str, value: &str) -> Option<Property> {
    match name {
        "background-color" | "bg" | "color" => parse_color(value).map(Property::Color),
        "padding" => parse_length(value).map(|p| Property::Padding(Padding(p))),
        "margin" => parse_length(value).map(|m| Property::Margin(Margin(m))),
        "width" => parse_size(value).map(|s| Property::Width(Width(s))),
        "height" => parse_size(value).map(|s| Property::Height(Height(s))),
        _ => None,
    }
}

/// Parses a color value.
fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();

    if value.starts_with('#') {
        Color::from_hex(value).ok()
    } else if value.starts_with("rgb") {
        let value =
            value.trim_start_matches("rgb(").trim_start_matches("rgba(").trim_end_matches(')');
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse().ok()?;
            let g = parts[1].trim().parse().ok()?;
            let b = parts[2].trim().parse().ok()?;
            Some(Color::rgb(r, g, b))
        } else {
            None
        }
    } else {
        match value.to_lowercase().as_str() {
            "red" => Some(Color::rgb(255, 0, 0)),
            "green" => Some(Color::rgb(0, 128, 0)),
            "blue" => Some(Color::rgb(0, 0, 255)),
            "white" => Some(Color::rgb(255, 255, 255)),
            "black" => Some(Color::rgb(0, 0, 0)),
            "orange" => Some(Color::rgb(255, 165, 0)),
            "darkblue" | "dark" => Some(Color::rgb(0, 0, 139)),
            "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
            "yellow" => Some(Color::rgb(255, 255, 0)),
            "purple" => Some(Color::rgb(128, 0, 128)),
            "pink" => Some(Color::rgb(255, 192, 203)),
            "brown" => Some(Color::rgb(165, 42, 42)),
            "transparent" => Some(Color::rgb(0, 0, 0)),
            _ => Some(Color::rgb(0, 0, 0)),
        }
    }
}

/// Parses a length value.
fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "auto" {
        return Some(0.0);
    }
    let num: f32 = value.parse().ok()?;
    Some(num)
}

/// Parses a size value.
fn parse_size(value: &str) -> Option<Size> {
    let value = value.trim();
    match value {
        "auto" => Some(Size::Auto),
        "min-content" => Some(Size::MinContent),
        "max-content" => Some(Size::MaxContent),
        v if v.ends_with("px") => {
            let num: f32 = v.trim_end_matches("px").parse().ok()?;
            Some(Size::Pixels(num))
        }
        v if v.ends_with('%') => {
            let num: f32 = v.trim_end_matches('%').parse().ok()?;
            Some(Size::Percent(num))
        }
        _ => Some(Size::Auto),
    }
}
