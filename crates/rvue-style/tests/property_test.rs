//! Property system tests.

use rvue_style::{
    BackgroundColor, Color, Display, FontFamily, FontSize, Height, Margin, Padding, Properties,
    Size, TextColor, Width,
};
use rvue_style::{StyledWidget, StyledWidgetExt, WidgetStyles};

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
    assert!(color.is_some());
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

#[derive(Default)]
struct MockWidget {
    properties: Properties,
}

impl StyledWidget for MockWidget {
    fn style(&self) -> &Properties {
        &self.properties
    }

    fn style_mut(&mut self) -> &mut Properties {
        &mut self.properties
    }

    fn set_style(&mut self, properties: Properties) {
        self.properties = properties;
    }
}

impl StyledWidgetExt for MockWidget {
    fn properties(&self) -> &Properties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut Properties {
        &mut self.properties
    }
}

#[test]
fn test_styled_widget_ext_basic() {
    let widget = MockWidget::default()
        .style_background(Color::rgb(255, 0, 0))
        .style_color(Color::rgb(255, 255, 255))
        .style_padding(12.0)
        .style_margin(8.0);

    assert!(widget.properties.contains::<BackgroundColor>());
    assert!(widget.properties.contains::<TextColor>());
    assert!(widget.properties.contains::<Padding>());
    assert!(widget.properties.contains::<Margin>());
}

#[test]
fn test_styled_widget_ext_builder_chain() {
    let widget = MockWidget::default()
        .style_background(Color::rgb(0, 120, 215))
        .style_color(Color::rgb(255, 255, 255))
        .style_padding(12.0)
        .style_margin(8.0)
        .style_font_size(16.0)
        .style_display(Display::Flex);

    assert_eq!(widget.properties.len(), 6);
}

#[test]
fn test_styled_widget_ext_width_height() {
    let widget = MockWidget::default()
        .style_width(Width(Size::pixels(200.0)))
        .style_height(Height(Size::pixels(100.0)));

    assert!(widget.properties.contains::<Width>());
    assert!(widget.properties.contains::<Height>());
}

#[test]
fn test_widget_styles_new() {
    let styles = WidgetStyles::new();
    assert!(styles.to_properties().is_empty());
}

#[test]
fn test_widget_styles_with() {
    let styles =
        WidgetStyles::new().with_background_color(Color::rgb(255, 0, 0)).with_padding(10.0);

    let props = styles.to_properties();
    assert_eq!(props.len(), 2);
    assert!(props.contains::<BackgroundColor>());
    assert!(props.contains::<Padding>());
}

#[test]
fn test_widget_styles_fluent_api() {
    let styles = WidgetStyles::new()
        .with_background_color(Color::rgb(0, 120, 215))
        .with_color(Color::rgb(255, 255, 255))
        .with_padding(12.0)
        .with_margin(8.0)
        .with_font_size(16.0)
        .with_display(Display::Flex);

    let props = styles.to_properties();
    assert_eq!(props.len(), 6);
}

#[test]
fn test_widget_styles_width_height() {
    let styles = WidgetStyles::new()
        .with_width(Width(Size::pixels(200.0)))
        .with_height(Height(Size::pixels(100.0)));

    let props = styles.to_properties();
    assert!(props.contains::<Width>());
    assert!(props.contains::<Height>());
}

#[test]
fn test_font_family_drop() {
    // Test that FontFamily (which contains a String) is properly dropped
    // This verifies that DynProperty correctly calls the destructor
    let mut props = Properties::new();
    props.insert(FontFamily("Arial".to_string()));

    // Verify it was inserted
    assert!(props.contains::<FontFamily>());
    let font = props.get::<FontFamily>().unwrap();
    assert_eq!(font.0, "Arial");

    // Remove it - this should trigger the Drop implementation
    props.remove::<FontFamily>();
    assert!(!props.contains::<FontFamily>());

    // The String inside FontFamily should have been properly dropped
    // If there was a memory leak, we'd see it in valgrind or similar tools
}

#[test]
fn test_properties_clone_with_heap_allocated() {
    // Test that cloning Properties with heap-allocated types works correctly
    let mut props1 = Properties::new();
    props1.insert(FontFamily("Helvetica".to_string()));

    let props2 = props1.clone();

    // Both should contain the same value
    assert_eq!(props1.get::<FontFamily>().unwrap().0, "Helvetica");
    assert_eq!(props2.get::<FontFamily>().unwrap().0, "Helvetica");

    // Modifying one shouldn't affect the other (deep copy)
    props1.insert(FontFamily("Arial".to_string()));
    assert_eq!(props1.get::<FontFamily>().unwrap().0, "Arial");
    assert_eq!(props2.get::<FontFamily>().unwrap().0, "Helvetica");

    // Both should be properly dropped when they go out of scope
}
