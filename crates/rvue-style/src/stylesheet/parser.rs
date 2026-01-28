//! CSS stylesheet parsing.

use crate::properties::{Color, Height, Margin, Padding, Size, Width};
use crate::property::Properties;
use crate::stylesheet::rule::{StyleRule, Stylesheet};

/// Parses a CSS stylesheet from a string.
pub fn parse_stylesheet(css: &str) -> Option<Stylesheet> {
    let mut stylesheet = Stylesheet::new();
    let mut position = 0;

    while position < css.len() {
        while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
            position += 1;
        }

        if position >= css.len() {
            break;
        }

        if css[position..].starts_with("/*") {
            if let Some(end) = css[position..].find("*/") {
                position += end + 2;
                continue;
            } else {
                return None;
            }
        }

        let selector_start = position;
        while position < css.len() && !css[position..].starts_with('{') {
            position += 1;
        }
        let selector = css[selector_start..position].trim();

        if selector.is_empty() {
            return None;
        }

        if position >= css.len() || !css[position..].starts_with('{') {
            return None;
        }
        position += 1;

        let mut properties = Properties::new();

        while position < css.len() {
            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            if position >= css.len() {
                return None;
            }

            if css[position..].starts_with('}') {
                position += 1;
                break;
            }

            if css[position..].starts_with("/*") {
                if let Some(end) = css[position..].find("*/") {
                    position += end + 2;
                    continue;
                } else {
                    return None;
                }
            }

            let name_start = position;
            while position < css.len()
                && !css[position..].starts_with(|c: char| c.is_whitespace() || c == ':')
            {
                position += 1;
            }
            let name = css[name_start..position].trim();

            if name.is_empty() {
                return None;
            }

            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            if position >= css.len() || !css[position..].starts_with(':') {
                return None;
            }
            position += 1;

            while position < css.len() && css[position..].starts_with(|c: char| c.is_whitespace()) {
                position += 1;
            }

            let value_start = position;
            while position < css.len()
                && !css[position..].starts_with(|c: char| c.is_whitespace() || c == ';' || c == '}')
            {
                position += 1;
            }
            let value = css[value_start..position].trim();

            if position < css.len() && css[position..].starts_with(';') {
                position += 1;
            }

            add_property(&mut properties, name, value);
        }

        stylesheet.add_rule(StyleRule::new(selector.to_string(), properties));
    }

    Some(stylesheet)
}

fn add_property(properties: &mut Properties, name: &str, value: &str) {
    match name {
        "background-color" | "bg" => {
            if let Some(color) = parse_color(value) {
                properties.insert(color);
            }
        }
        "color" => {
            if let Some(color) = parse_color(value) {
                properties.insert(color);
            }
        }
        "padding" => {
            if let Some(p) = parse_length(value) {
                properties.insert(Padding(p));
            }
        }
        "margin" => {
            if let Some(m) = parse_length(value) {
                properties.insert(Margin(m));
            }
        }
        "width" => {
            if let Some(s) = parse_size(value) {
                properties.insert(Width(s));
            }
        }
        "height" => {
            if let Some(s) = parse_size(value) {
                properties.insert(Height(s));
            }
        }
        _ => {}
    }
}

fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();

    if value.starts_with('#') {
        Color::from_hex(value)
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
            "transparent" => Some(Color::rgb(0, 0, 0)),
            _ => Some(Color::rgb(0, 0, 0)),
        }
    }
}

fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "auto" {
        return Some(0.0);
    }
    let num: f32 = value.parse().ok()?;
    Some(num)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_stylesheet() {
        let css = r#"
            button {
                background-color: #ff0000;
                color: white;
            }
        "#;

        let stylesheet = parse_stylesheet(css).unwrap();
        assert!(!stylesheet.is_empty());
    }

    #[test]
    fn test_parse_multiple_rules() {
        let css = r#"
            button {
                background-color: blue;
            }

            .primary {
                background-color: green;
            }

            #submit-btn {
                background-color: orange;
            }
        "#;

        let stylesheet = parse_stylesheet(css).unwrap();
        assert_eq!(stylesheet.len(), 3);
    }

    #[test]
    fn test_parse_with_comments() {
        let css = r#"
            /* This is a comment */
            button {
                /* Another comment */
                background-color: red;
            }
        "#;

        let stylesheet = parse_stylesheet(css).unwrap();
        assert!(!stylesheet.is_empty());
    }
}
