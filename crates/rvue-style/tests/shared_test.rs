use rvue_style::{
    shared_background_color, shared_centered_flex, shared_flex_container, shared_margin,
    shared_padding, shared_text_color, AlignItems, BackgroundColor, BorderColor, BorderRadius,
    BorderStyle, BorderWidth, Color, ComputedStyles, Display, FlexDirection, FlexGrow, FlexShrink,
    FontFamily, FontSize, FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Padding,
    SharedComputedStyles, SharedStyleBuilder, Size, TextColor, Visibility, Width, ZIndex,
};

#[test]
fn test_shared_computed_styles_new() {
    let styles = ComputedStyles::new();
    let shared = SharedComputedStyles::new(styles);
    assert!(shared.background_color().is_none());
}

#[test]
fn test_shared_computed_styles_from_properties() {
    let mut props = rvue_style::Properties::new();
    props.insert(BackgroundColor(Color::rgb(255, 0, 0)));
    props.insert(Width(Size::pixels(100.0)));

    let shared = SharedComputedStyles::from_properties(&props);
    assert_eq!(shared.background_color(), Some(BackgroundColor(Color::rgb(255, 0, 0))));
    assert_eq!(shared.width(), Some(Width(Size::pixels(100.0))));
}

#[test]
fn test_shared_computed_styles_deref() {
    let mut computed = ComputedStyles::new();
    computed.background_color = Some(BackgroundColor(Color::rgb(100, 100, 100)));
    let shared = SharedComputedStyles::new(computed);

    assert_eq!(shared.background_color, Some(BackgroundColor(Color::rgb(100, 100, 100))));
}

#[test]
fn test_shared_computed_styles_as_ref() {
    let computed = ComputedStyles::new();
    let shared = SharedComputedStyles::new(computed);
    let _ = shared.as_ref();
}

#[test]
fn test_shared_computed_styles_downgrade() {
    let shared = shared_background_color(Color::rgb(255, 0, 0));
    let weak = shared.downgrade();

    assert!(!weak.is_dead());
    assert!(weak.upgrade().is_some());
}

#[test]
fn test_shared_computed_styles_strong_count() {
    let shared = shared_background_color(Color::rgb(255, 0, 0));
    assert_eq!(shared.strong_count(), 1);

    let shared2 = shared.clone();
    assert_eq!(shared.strong_count(), 2);
    assert_eq!(shared2.strong_count(), 2);
}

#[test]
fn test_weak_shared_computed_styles() {
    let shared = shared_background_color(Color::rgb(255, 0, 0));
    let weak = shared.downgrade();

    assert!(!weak.is_dead());
    assert!(weak.upgrade().is_some());

    drop(shared);
    assert!(weak.is_dead());
    assert!(weak.upgrade().is_none());
}

#[test]
fn test_shared_style_builder_new() {
    let builder = SharedStyleBuilder::new();
    let shared = builder.build();
    assert!(shared.background_color().is_none());
}

#[test]
fn test_shared_style_builder_background_color() {
    let shared = SharedStyleBuilder::new().with_background_color(Color::rgb(255, 0, 0)).build();

    assert_eq!(shared.background_color(), Some(BackgroundColor(Color::rgb(255, 0, 0))));
}

#[test]
fn test_shared_style_builder_text_color() {
    let shared = SharedStyleBuilder::new().with_text_color(Color::rgb(0, 0, 255)).build();

    assert_eq!(shared.text_color(), Some(TextColor(Color::rgb(0, 0, 255))));
}

#[test]
fn test_shared_style_builder_font_size() {
    let shared = SharedStyleBuilder::new().with_font_size(16.0).build();

    assert_eq!(shared.font_size(), Some(FontSize(16.0)));
}

#[test]
fn test_shared_style_builder_font_family() {
    let shared = SharedStyleBuilder::new().with_font_family("Arial").build();

    assert_eq!(shared.font_family(), Some(FontFamily("Arial".to_string())));
}

#[test]
fn test_shared_style_builder_font_weight() {
    let shared = SharedStyleBuilder::new().with_font_weight(FontWeight::Bold).build();

    assert_eq!(shared.font_weight(), Some(FontWeight::Bold));
}

#[test]
fn test_shared_style_builder_padding() {
    let shared = SharedStyleBuilder::new().with_padding(10.0).build();

    assert_eq!(shared.padding(), Some(Padding(10.0)));
}

#[test]
fn test_shared_style_builder_margin() {
    let shared = SharedStyleBuilder::new().with_margin(20.0).build();

    assert_eq!(shared.margin(), Some(Margin(20.0)));
}

#[test]
fn test_shared_style_builder_width_height() {
    let shared = SharedStyleBuilder::new()
        .with_width(Width(Size::pixels(100.0)))
        .with_height(Height(Size::pixels(200.0)))
        .build();

    assert_eq!(shared.width(), Some(Width(Size::pixels(100.0))));
    assert_eq!(shared.height(), Some(Height(Size::pixels(200.0))));
}

#[test]
fn test_shared_style_builder_display_flex() {
    let shared = SharedStyleBuilder::new()
        .with_display(Display::Flex)
        .with_flex_direction(FlexDirection::Column)
        .build();

    assert_eq!(shared.display(), Some(Display::Flex));
    assert_eq!(shared.flex_direction(), Some(FlexDirection::Column));
}

#[test]
fn test_shared_style_builder_flex_alignment() {
    let shared = SharedStyleBuilder::new()
        .with_justify_content(JustifyContent::Center)
        .with_align_items(AlignItems::FlexEnd)
        .with_flex_grow(1.0)
        .with_flex_shrink(0.5)
        .with_gap(8.0)
        .build();

    assert_eq!(shared.justify_content(), Some(JustifyContent::Center));
    assert_eq!(shared.align_items(), Some(AlignItems::FlexEnd));
    assert_eq!(shared.flex_grow(), Some(FlexGrow(1.0)));
    assert_eq!(shared.flex_shrink(), Some(FlexShrink(0.5)));
    assert_eq!(shared.gap(), Some(Gap(8.0)));
}

#[test]
fn test_shared_style_builder_border() {
    let shared = SharedStyleBuilder::new()
        .with_border_color(Color::rgb(0, 0, 0))
        .with_border_width(2.0)
        .with_border_radius(4.0)
        .with_border_style(BorderStyle::Solid)
        .build();

    assert_eq!(shared.border_color(), Some(BorderColor(Color::rgb(0, 0, 0))));
    assert_eq!(shared.border_width(), Some(BorderWidth(2.0)));
    assert_eq!(shared.border_radius(), Some(BorderRadius(4.0)));
    assert_eq!(shared.border_style(), Some(BorderStyle::Solid));
}

#[test]
fn test_shared_style_builder_opacity_visibility_zindex() {
    let shared = SharedStyleBuilder::new()
        .with_opacity(0.5)
        .with_visibility(Visibility::Hidden)
        .with_z_index(100)
        .build();

    assert_eq!(shared.opacity(), Some(Opacity(0.5)));
    assert_eq!(shared.visibility(), Some(Visibility::Hidden));
    assert_eq!(shared.z_index(), Some(ZIndex(100)));
}

#[test]
fn test_shared_background_color_function() {
    let shared = shared_background_color(Color::rgb(255, 0, 0));
    assert_eq!(shared.background_color(), Some(BackgroundColor(Color::rgb(255, 0, 0))));
}

#[test]
fn test_shared_text_color_function() {
    let shared = shared_text_color(Color::rgb(0, 0, 255));
    assert_eq!(shared.text_color(), Some(TextColor(Color::rgb(0, 0, 255))));
}

#[test]
fn test_shared_padding_function() {
    let shared = shared_padding(12.0);
    assert_eq!(shared.padding(), Some(Padding(12.0)));
}

#[test]
fn test_shared_margin_function() {
    let shared = shared_margin(8.0);
    assert_eq!(shared.margin(), Some(Margin(8.0)));
}

#[test]
fn test_shared_flex_container_function() {
    let shared = shared_flex_container();
    assert_eq!(shared.display(), Some(Display::Flex));
}

#[test]
fn test_shared_centered_flex_function() {
    let shared = shared_centered_flex();
    assert_eq!(shared.display(), Some(Display::Flex));
    assert_eq!(shared.justify_content(), Some(JustifyContent::Center));
    assert_eq!(shared.align_items(), Some(AlignItems::Center));
}

#[test]
fn test_shared_style_builder_all_properties() {
    let shared = SharedStyleBuilder::new()
        .with_background_color(Color::rgb(255, 255, 255))
        .with_color(Color::rgb(0, 0, 0))
        .with_text_color(Color::rgb(0, 0, 0))
        .with_font_size(14.0)
        .with_font_family("Helvetica")
        .with_font_weight(FontWeight::Normal)
        .with_padding(16.0)
        .with_margin(8.0)
        .with_width(Width(Size::auto()))
        .with_height(Height(Size::pixels(100.0)))
        .with_display(Display::Flex)
        .with_flex_direction(FlexDirection::Row)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_align_items(AlignItems::Center)
        .with_flex_grow(1.0)
        .with_flex_shrink(0.0)
        .with_gap(4.0)
        .with_border_color(Color::rgb(200, 200, 200))
        .with_border_width(1.0)
        .with_border_radius(2.0)
        .with_border_style(BorderStyle::Dashed)
        .with_opacity(1.0)
        .with_visibility(Visibility::Visible)
        .with_z_index(0)
        .build();

    assert!(shared.background_color().is_some());
    assert!(shared.color().is_some());
    assert!(shared.text_color().is_some());
    assert!(shared.font_size().is_some());
    assert!(shared.font_family().is_some());
    assert!(shared.font_weight().is_some());
    assert!(shared.padding().is_some());
    assert!(shared.margin().is_some());
    assert!(shared.width().is_some());
    assert!(shared.height().is_some());
    assert!(shared.display().is_some());
    assert!(shared.flex_direction().is_some());
    assert!(shared.justify_content().is_some());
    assert!(shared.align_items().is_some());
    assert!(shared.flex_grow().is_some());
    assert!(shared.flex_shrink().is_some());
    assert!(shared.gap().is_some());
    assert!(shared.border_color().is_some());
    assert!(shared.border_width().is_some());
    assert!(shared.border_radius().is_some());
    assert!(shared.border_style().is_some());
    assert!(shared.opacity().is_some());
    assert!(shared.visibility().is_some());
    assert!(shared.z_index().is_some());
}

#[test]
fn test_shared_computed_styles_clone() {
    let shared1 = shared_background_color(Color::rgb(255, 0, 0));
    let shared2 = shared1.clone();

    assert_eq!(shared1.strong_count(), 2);
    assert_eq!(shared2.strong_count(), 2);
    assert_eq!(shared1.background_color(), shared2.background_color());
}

#[test]
fn test_shared_style_builder_clone() {
    let builder1 = SharedStyleBuilder::new().with_padding(10.0);
    let builder2 = builder1.clone();

    let shared1 = builder1.build();
    let shared2 = builder2.build();

    assert_eq!(shared1.padding(), shared2.padding());
}
