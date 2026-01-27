//! Property system tests.

use rvue_style::{
    BackgroundColor, Color, Display, FontSize, Height, Margin, Padding, Properties, Property, Size,
    Width,
};

#[test]
fn test_properties_new() {
    let props = Properties::new();
    assert!(props.is_empty());
    assert_eq!(props.len(), 0);
}

#[test]
fn test_properties_with() {
    let props = Properties::with(Color::rgb(255, 0, 0));
    assert_eq!(props.len(), 1);
}

#[test]
fn test_properties_insert_and_get() {
    let mut props = Properties::new();

    props.insert(Color::rgb(255, 0, 0));
    assert_eq!(props.len(), 1);

    let color = props.get::<Color>();
    assert!(color.is_some());
    assert_eq!(color.unwrap().0.r, 255);
}

#[test]
fn test_properties_multiple_inserts() {
    let mut props = Properties::new();

    props.insert(Color::rgb(255, 0, 0));
    props.insert(Padding(10.0));
    props.insert(Margin(5.0));

    assert_eq!(props.len(), 3);

    assert!(props.get::<Color>().is_some());
    assert!(props.get::<Padding>().is_some());
    assert!(props.get::<Margin>().is_some());
}

#[test]
fn test_properties_remove() {
    let mut props = Properties::new();
    props.insert(Color::rgb(255, 0, 0));
    assert_eq!(props.len(), 1);

    props.remove::<Color>();
    assert!(props.is_empty());
}

#[test]
fn test_properties_contains() {
    let mut props = Properties::new();
    props.insert(Color::rgb(255, 0, 0));

    assert!(props.contains::<Color>());
    assert!(!props.contains::<Padding>());
}

#[test]
fn test_color_from_hex() {
    let color = Color::from_hex("#FF0000");
    assert!(color.is_ok());
    let c = color.unwrap();
    assert_eq!(c.0.r, 255);
    assert_eq!(c.0.g, 0);
    assert_eq!(c.0.b, 0);
}

#[test]
fn test_size_variants() {
    let auto = Size::auto();
    let pixels = Size::pixels(100.0);
    let percent = Size::percent(50.0);

    assert_eq!(auto, Size::Auto);
    assert_eq!(pixels, Size::Pixels(100.0));
    assert_eq!(percent, Size::Percent(50.0));
}

#[test]
fn test_property_traits() {
    // Verify all property types implement Property
    let _ = Color::rgb(0, 0, 0);
    let _ = Padding(0.0);
    let _ = Margin(0.0);
    let _ = Width(Size::Auto);
    let _ = Height(Size::Auto);
    let _ = BackgroundColor(Color::rgb(0, 0, 0));
    let _ = Display::Flex;
    let _ = FontSize(16.0);
}
