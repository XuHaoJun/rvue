//! Stylesheet parsing tests.

use rvue_style::stylesheet::parser::parse_stylesheet;

#[test]
fn test_parse_empty_stylesheet() {
    let result = parse_stylesheet("");
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert!(stylesheet.is_empty());
}

#[test]
fn test_parse_single_rule() {
    let css = "button { background-color: red; }";
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 1);
}

#[test]
fn test_parse_multiple_rules() {
    let css = r#"
        button { background-color: red; }
        .primary { color: blue; }
        #header { font-size: 16px; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 3);
}

#[test]
fn test_parse_named_colors() {
    let css = r#"
        .red { color: red; }
        .blue { color: blue; }
        .white { color: white; }
        .black { color: black; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 4);
}

#[test]
fn test_parse_length_values() {
    let css = r#"
        .p1 { padding: 10px; }
        .p2 { margin: 20; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
}

#[test]
fn test_parse_size_values() {
    let css = r#"
        .w1 { width: auto; }
        .w2 { width: 100px; }
        .w3 { width: 50%; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
}

#[test]
fn test_parse_comments() {
    let css = r#"
        /* This is a comment */
        button {
            background-color: red;
            /* Another comment */
        }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 1);
}

#[test]
fn test_parse_whitespace_variations() {
    let css = "button { background-color: red; }";
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 1);
}

#[test]
fn test_parse_multiple_properties() {
    let css = r#"
        button {
            background-color: red;
            color: white;
            padding: 10px;
            margin: 5px;
            width: 100px;
            height: 50px;
        }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 1);
}

#[test]
fn test_parse_selectors() {
    let css = r#"
        button { }
        .class { }
        #id { }
        button.class { }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 4);
}

#[test]
fn test_parse_optional_semicolon() {
    let css = r#"
        button {
            background-color: red;
            color: white
        }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
}

#[test]
fn test_parse_pseudo_class_selectors() {
    let css = r#"
        button:hover { background-color: red; }
        button:focus { outline: none; }
        button:active { transform: scale(0.95); }
        button:disabled { opacity: 0.5; }
        input:checked { border-color: blue; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 5);
}

#[test]
fn test_parse_complex_selectors() {
    let css = r#"
        .parent .child { background-color: blue; }
        button.primary { background-color: red; }
    "#;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
    let stylesheet = result.unwrap();
    assert_eq!(stylesheet.len(), 2);
}
