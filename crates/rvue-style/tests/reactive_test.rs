use rvue_style::{
    create_reactive_signal, AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius,
    BorderStyle, BorderWidth, Color, ComputedStyles, Cursor, Display, FlexBasis, FlexDirection,
    FlexGrow, FlexShrink, FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin,
    Opacity, Padding, ReactiveProperty, ReactiveSignal, ReactiveSignalWrite, ReactiveStyles, Size,
    TextColor, Visibility, Width, ZIndex,
};

#[test]
fn test_create_reactive_signal() {
    let (read, write) = create_reactive_signal(42);
    assert_eq!(read.get(), 42);
    write.set(100);
    assert_eq!(read.get(), 100);
}

#[test]
fn test_reactive_signal_untracked() {
    let (read, _write) = create_reactive_signal(42);
    assert_eq!(read.get_untracked(), 42);
}

#[test]
fn test_reactive_signal_write_update() {
    let (read, write) = create_reactive_signal(10);
    write.update(|v| *v *= 2);
    assert_eq!(read.get(), 20);
}

#[test]
fn test_reactive_property_static() {
    let prop = ReactiveProperty::<i32>::static_value(42);
    assert_eq!(prop.get(), 42);
    assert!(!prop.is_reactive());
}

#[test]
fn test_reactive_property_reactive() {
    let (read, _write) = create_reactive_signal(100);
    let prop = ReactiveProperty::<i32>::reactive(read);
    assert_eq!(prop.get(), 100);
    assert!(prop.is_reactive());
}

#[test]
fn test_reactive_property_from_static() {
    let prop: ReactiveProperty<i32> = 42.into();
    assert_eq!(prop.get(), 42);
}

#[test]
fn test_reactive_property_from_signal() {
    let (read, _write) = create_reactive_signal(42);
    let prop: ReactiveProperty<i32> = read.into();
    assert!(prop.is_reactive());
}

#[test]
fn test_reactive_styles_new() {
    let styles = ReactiveStyles::new();
    let computed = styles.compute();
    assert!(computed.background_color.is_some());
    assert!(computed.color.is_some());
}

#[test]
fn test_reactive_styles_setters() {
    let styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor::default())
        .set_color(Color::rgb(255, 0, 0))
        .set_width(Width(Size::pixels(100.0)))
        .set_height(Height(Size::pixels(200.0)))
        .set_display(Display::Flex)
        .set_flex_direction(FlexDirection::Column);

    let computed = styles.compute();
    assert_eq!(computed.width, Some(Width(Size::pixels(100.0))));
    assert_eq!(computed.height, Some(Height(Size::pixels(200.0))));
    assert_eq!(computed.display, Some(Display::Flex));
    assert_eq!(computed.flex_direction, Some(FlexDirection::Column));
    assert_eq!(computed.background_color, Some(BackgroundColor::default()));
    assert_eq!(computed.color, Some(Color::rgb(255, 0, 0)));
}

#[test]
fn test_reactive_styles_with_reactive_values() {
    let opacity_signal = create_reactive_signal(Opacity(0.5));

    let styles = ReactiveStyles::new().set_opacity(opacity_signal.0);

    let computed = styles.compute();
    assert_eq!(computed.opacity, Some(Opacity(0.5)));

    opacity_signal.1.set(Opacity(0.8));
    let computed2 = styles.compute();
    assert_eq!(computed2.opacity, Some(Opacity(0.8)));
}

#[test]
fn test_reactive_styles_all_properties() {
    let styles = ReactiveStyles::new()
        .set_background_color(BackgroundColor::default())
        .set_border_color(BorderColor::default())
        .set_border_radius(BorderRadius::default())
        .set_color(Color::default())
        .set_text_color(TextColor::default())
        .set_cursor(Cursor::Default)
        .set_display(Display::default())
        .set_opacity(Opacity::default())
        .set_visibility(Visibility::Visible)
        .set_width(Width::default())
        .set_height(Height::default())
        .set_font_family(FontFamily::default())
        .set_font_size(FontSize::default())
        .set_font_weight(FontWeight::default())
        .set_z_index(ZIndex::default())
        .set_align_items(AlignItems::default())
        .set_align_self(AlignSelf::default())
        .set_border_style(BorderStyle::default())
        .set_border_width(BorderWidth::default())
        .set_flex_basis(FlexBasis::default())
        .set_flex_direction(FlexDirection::default())
        .set_flex_grow(FlexGrow::default())
        .set_flex_shrink(FlexShrink::default())
        .set_gap(Gap::default())
        .set_justify_content(JustifyContent::default())
        .set_margin(Margin::default())
        .set_padding(Padding::default());

    let computed = styles.compute();
    assert!(computed.background_color.is_some());
    assert!(computed.border_color.is_some());
    assert!(computed.border_radius.is_some());
    assert!(computed.color.is_some());
    assert!(computed.text_color.is_some());
    assert!(computed.display.is_some());
    assert!(computed.opacity.is_some());
    assert!(computed.visibility.is_some());
    assert!(computed.width.is_some());
    assert!(computed.height.is_some());
    assert!(computed.font_family.is_some());
    assert!(computed.font_size.is_some());
    assert!(computed.font_weight.is_some());
    assert!(computed.z_index.is_some());
    assert!(computed.align_items.is_some());
    assert!(computed.flex_direction.is_some());
    assert!(computed.flex_grow.is_some());
    assert!(computed.flex_shrink.is_some());
    assert!(computed.gap.is_some());
    assert!(computed.justify_content.is_some());
    assert!(computed.border_style.is_some());
    assert!(computed.border_width.is_some());
    assert!(computed.margin.is_some());
    assert!(computed.padding.is_some());
}

#[test]
fn test_computed_styles_from_reactive_styles() {
    let styles = ReactiveStyles::new()
        .set_width(Width(Size::pixels(100.0)))
        .set_height(Height(Size::pixels(200.0)));

    let computed: ComputedStyles = styles.into();
    assert_eq!(computed.width, Some(Width(Size::pixels(100.0))));
    assert_eq!(computed.height, Some(Height(Size::pixels(200.0))));
}

#[test]
fn test_reactive_property_get_untracked() {
    let (read, _write) = create_reactive_signal(42);
    let prop = ReactiveProperty::<i32>::reactive(read);

    assert_eq!(prop.get(), 42);
    assert_eq!(prop.get_untracked(), 42);
}

#[test]
fn test_reactive_signal_trait() {
    let (read, _write) = create_reactive_signal(42);

    fn get_value<T: Clone + 'static, S: ReactiveSignal<T>>(signal: &S) -> T {
        signal.get()
    }

    assert_eq!(get_value(&read), 42);
}

#[test]
fn test_reactive_signal_write_trait() {
    let (read, write) = create_reactive_signal(10);

    fn set_value<T: Clone + 'static, S: ReactiveSignalWrite<T>>(signal: &S, value: T) {
        signal.set(value);
    }

    set_value(&write, 100);
    assert_eq!(read.get(), 100);
}

#[test]
fn test_reactive_styles_default() {
    let styles = ReactiveStyles::default();
    let computed = styles.compute();
    assert!(computed.background_color.is_some());
}

#[test]
fn test_color_types() {
    let color = Color::rgb(255, 0, 0);
    assert_eq!(color.0.r, 255);
    assert_eq!(color.0.g, 0);
    assert_eq!(color.0.b, 0);

    let bg = BackgroundColor(color);
    assert_eq!(bg.0 .0.r, 255);
}

#[test]
fn test_size_types() {
    let w = Width(Size::pixels(100.0));
    assert_eq!(w, Width(Size::Pixels(100.0)));

    let h = Height(Size::percent(50.0));
    assert_eq!(h, Height(Size::Percent(50.0)));

    let auto = Width(Size::auto());
    assert_eq!(auto, Width(Size::Auto));
}

#[test]
fn test_display_types() {
    assert_eq!(Display::Flex, Display::Flex);
    assert_eq!(Display::Grid, Display::Grid);
    assert_eq!(Display::Block, Display::Block);
    assert_eq!(Display::None, Display::None);
}

#[test]
fn test_flex_direction_types() {
    assert_eq!(FlexDirection::Row, FlexDirection::Row);
    assert_eq!(FlexDirection::Column, FlexDirection::Column);
    assert_eq!(FlexDirection::RowReverse, FlexDirection::RowReverse);
    assert_eq!(FlexDirection::ColumnReverse, FlexDirection::ColumnReverse);
}

#[test]
fn test_font_weight_types() {
    assert_eq!(FontWeight::from_numeric(400), FontWeight::Normal);
    assert_eq!(FontWeight::from_numeric(700), FontWeight::Bold);
}

#[test]
fn test_opacity_types() {
    let op1 = Opacity(0.5);
    let op2 = Opacity(1.0);
    assert_ne!(op1, op2);
}

#[test]
fn test_visibility_types() {
    assert_eq!(Visibility::Visible, Visibility::Visible);
    assert_eq!(Visibility::Hidden, Visibility::Hidden);
    assert_eq!(Visibility::Collapse, Visibility::Collapse);
}

#[test]
fn test_cursor_types() {
    assert_eq!(Cursor::Default, Cursor::Default);
    assert_eq!(Cursor::Pointer, Cursor::Pointer);
    assert_eq!(Cursor::Text, Cursor::Text);
}

#[test]
fn test_z_index_types() {
    let z1 = ZIndex(0);
    let z2 = ZIndex(10);
    assert_ne!(z1, z2);
}

#[test]
fn test_reactive_property_clone() {
    let prop1 = ReactiveProperty::<i32>::static_value(42);
    let prop2 = prop1.clone();
    assert_eq!(prop2.get(), 42);
}

#[test]
fn test_reactive_styles_clone() {
    let styles1 = ReactiveStyles::new().set_width(Width(Size::pixels(100.0)));
    let styles2 = styles1.clone();
    assert_eq!(styles2.compute().width, Some(Width(Size::pixels(100.0))));
}

#[test]
fn test_border_style_types() {
    assert_eq!(BorderStyle::None, BorderStyle::None);
    assert_eq!(BorderStyle::Solid, BorderStyle::Solid);
    assert_eq!(BorderStyle::Dashed, BorderStyle::Dashed);
    assert_eq!(BorderStyle::Dotted, BorderStyle::Dotted);
}

#[test]
fn test_align_items_types() {
    assert_eq!(AlignItems::FlexStart, AlignItems::FlexStart);
    assert_eq!(AlignItems::FlexEnd, AlignItems::FlexEnd);
    assert_eq!(AlignItems::Center, AlignItems::Center);
    assert_eq!(AlignItems::Stretch, AlignItems::Stretch);
}

#[test]
fn test_justify_content_types() {
    assert_eq!(JustifyContent::FlexStart, JustifyContent::FlexStart);
    assert_eq!(JustifyContent::FlexEnd, JustifyContent::FlexEnd);
    assert_eq!(JustifyContent::Center, JustifyContent::Center);
    assert_eq!(JustifyContent::SpaceBetween, JustifyContent::SpaceBetween);
}

#[test]
fn test_spacing_types() {
    let margin = Margin(10.0);
    let padding = Padding(5.0);
    assert_eq!(margin.0, 10.0);
    assert_eq!(padding.0, 5.0);
}

#[test]
fn test_reactive_styles_compute_returns_initial_for_unset() {
    // Verify that compute() returns initial values (not None) for unset properties
    // This is important for the style system to have complete computed styles
    let styles = ReactiveStyles::new();
    let computed = styles.compute();

    // All properties should return Some, never None
    assert!(computed.background_color.is_some(), "background_color should not be None");
    assert!(computed.color.is_some(), "color should not be None");
    assert!(computed.text_color.is_some(), "text_color should not be None");
    assert!(computed.font_size.is_some(), "font_size should not be None");
    assert!(computed.font_family.is_some(), "font_family should not be None");
    assert!(computed.font_weight.is_some(), "font_weight should not be None");
    assert!(computed.padding.is_some(), "padding should not be None");
    assert!(computed.margin.is_some(), "margin should not be None");
    assert!(computed.width.is_some(), "width should not be None");
    assert!(computed.height.is_some(), "height should not be None");
    assert!(computed.display.is_some(), "display should not be None");
    assert!(computed.flex_direction.is_some(), "flex_direction should not be None");
    assert!(computed.justify_content.is_some(), "justify_content should not be None");
    assert!(computed.align_items.is_some(), "align_items should not be None");
    assert!(computed.align_self.is_some(), "align_self should not be None");
    assert!(computed.flex_grow.is_some(), "flex_grow should not be None");
    assert!(computed.flex_shrink.is_some(), "flex_shrink should not be None");
    assert!(computed.flex_basis.is_some(), "flex_basis should not be None");
    assert!(computed.gap.is_some(), "gap should not be None");
    assert!(computed.border_color.is_some(), "border_color should not be None");
    assert!(computed.border_width.is_some(), "border_width should not be None");
    assert!(computed.border_radius.is_some(), "border_radius should not be None");
    assert!(computed.border_style.is_some(), "border_style should not be None");
    assert!(computed.opacity.is_some(), "opacity should not be None");
    assert!(computed.visibility.is_some(), "visibility should not be None");
    assert!(computed.z_index.is_some(), "z_index should not be None");
    assert!(computed.cursor.is_some(), "cursor should not be None");
}

#[test]
fn test_reactive_styles_compute_returns_explicit_for_set() {
    // Verify that compute() returns explicit values for set properties
    let styles = ReactiveStyles::new()
        .set_width(Width(Size::Pixels(100.0)))
        .set_height(Height(Size::Pixels(50.0)))
        .set_background_color(BackgroundColor(Color::rgb(255, 0, 0)))
        .set_display(Display::Flex);

    let computed = styles.compute();

    // Explicit values should be returned
    assert_eq!(computed.width, Some(Width(Size::Pixels(100.0))));
    assert_eq!(computed.height, Some(Height(Size::Pixels(50.0))));
    assert_eq!(computed.background_color, Some(BackgroundColor(Color::rgb(255, 0, 0))));
    assert_eq!(computed.display, Some(Display::Flex));
}

#[test]
fn test_reactive_styles_flags_track_explicitly_set() {
    // Verify that ReactiveStyles correctly tracks which properties are set
    // We test this indirectly by checking compute() results
    let styles_with_width = ReactiveStyles::new().set_width(Width(Size::Pixels(100.0)));
    let styles_without_width = ReactiveStyles::new();

    let computed_with = styles_with_width.compute();
    let computed_without = styles_without_width.compute();

    // Width should be explicitly set in one but not the other
    // Both have initial values, but we can verify the styles work differently
    assert_eq!(computed_with.width, Some(Width(Size::Pixels(100.0))));
    assert!(computed_without.width.is_some()); // Has initial value
}

#[test]
fn test_reactive_styles_clone_preserves_explicit_values() {
    let styles1 = ReactiveStyles::new()
        .set_width(Width(Size::Pixels(100.0)))
        .set_background_color(BackgroundColor(Color::rgb(255, 0, 0)));

    let styles2 = styles1.clone();

    let computed1 = styles1.compute();
    let computed2 = styles2.compute();

    // Both should have the same explicit values
    assert_eq!(computed1.width, computed2.width);
    assert_eq!(computed1.background_color, computed2.background_color);
}

#[test]
fn test_reactive_styles_counter_button_example() {
    // Simulate the counter example's increment button styles
    use rvue_style::properties::AlignItems;
    use rvue_style::properties::JustifyContent;

    let increment_styles = ReactiveStyles::new()
        .set_align_items(AlignItems::Center)
        .set_justify_content(JustifyContent::Center)
        .set_background_color(BackgroundColor(Color::rgb(76, 175, 80))); // Green

    let computed = increment_styles.compute();

    // Verify all explicitly set values
    assert_eq!(computed.align_items, Some(AlignItems::Center));
    assert_eq!(computed.justify_content, Some(JustifyContent::Center));
    assert_eq!(computed.background_color, Some(BackgroundColor(Color::rgb(76, 175, 80))));

    // Verify width/height have initial values (not None, not explicit)
    assert!(computed.width.is_some());
    assert!(computed.height.is_some());
}

#[test]
fn test_reactive_styles_to_computed() {
    // Test the From<ReactiveStyles> for ComputedStyles implementation
    let styles = ReactiveStyles::new()
        .set_width(Width(Size::Pixels(100.0)))
        .set_height(Height(Size::Pixels(50.0)));

    let computed: ComputedStyles = styles.into();

    assert_eq!(computed.width, Some(Width(Size::Pixels(100.0))));
    assert_eq!(computed.height, Some(Height(Size::Pixels(50.0))));
}
