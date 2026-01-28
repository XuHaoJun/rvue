//! GC-compatible style sharing for efficient memory usage.
//!
//! This module provides types and utilities for sharing styles across
//! multiple widgets using garbage-collected references.

use crate::properties::{
    AlignItems, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth, Color,
    ComputedStyles, Display, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize, FontWeight,
    Gap, Height, JustifyContent, Margin, Opacity, Padding, TextColor, Visibility, Width, ZIndex,
};
use rudo_gc::{Gc, Weak};

/// Shared computed styles using garbage-collected reference.
///
/// This type allows multiple widgets to share the same style definition
/// without copying, reducing memory usage when many widgets have identical styles.
#[derive(Debug, Clone)]
pub struct SharedComputedStyles(Gc<ComputedStyles>);

impl SharedComputedStyles {
    /// Creates a new shared style from computed styles.
    pub fn new(styles: ComputedStyles) -> Self {
        Self(Gc::new(styles))
    }

    /// Creates a shared style from a properties container.
    pub fn from_properties(props: &crate::Properties) -> Self {
        let mut computed = ComputedStyles::new();
        computed.merge(props);
        Self::new(computed)
    }

    /// Returns a reference to the underlying computed styles.
    pub fn as_ref(&self) -> &ComputedStyles {
        &self.0
    }

    /// Gets the background color, if set.
    pub fn background_color(&self) -> Option<BackgroundColor> {
        self.0.background_color
    }

    /// Gets the text color, if set.
    pub fn color(&self) -> Option<Color> {
        self.0.color
    }

    /// Gets the text color, if set.
    pub fn text_color(&self) -> Option<TextColor> {
        self.0.text_color
    }

    /// Gets the font size, if set.
    pub fn font_size(&self) -> Option<FontSize> {
        self.0.font_size
    }

    /// Gets the font family, if set.
    pub fn font_family(&self) -> Option<FontFamily> {
        self.0.font_family.clone()
    }

    /// Gets the font weight, if set.
    pub fn font_weight(&self) -> Option<FontWeight> {
        self.0.font_weight
    }

    /// Gets the padding, if set.
    pub fn padding(&self) -> Option<Padding> {
        self.0.padding
    }

    /// Gets the margin, if set.
    pub fn margin(&self) -> Option<Margin> {
        self.0.margin
    }

    /// Gets the width, if set.
    pub fn width(&self) -> Option<Width> {
        self.0.width.clone()
    }

    /// Gets the height, if set.
    pub fn height(&self) -> Option<Height> {
        self.0.height.clone()
    }

    /// Gets the display property, if set.
    pub fn display(&self) -> Option<Display> {
        self.0.display
    }

    /// Gets the flex direction, if set.
    pub fn flex_direction(&self) -> Option<FlexDirection> {
        self.0.flex_direction
    }

    /// Gets the justify content property, if set.
    pub fn justify_content(&self) -> Option<JustifyContent> {
        self.0.justify_content
    }

    /// Gets the align items property, if set.
    pub fn align_items(&self) -> Option<AlignItems> {
        self.0.align_items
    }

    /// Gets the flex grow property, if set.
    pub fn flex_grow(&self) -> Option<FlexGrow> {
        self.0.flex_grow
    }

    /// Gets the flex shrink property, if set.
    pub fn flex_shrink(&self) -> Option<FlexShrink> {
        self.0.flex_shrink
    }

    /// Gets the gap property, if set.
    pub fn gap(&self) -> Option<Gap> {
        self.0.gap
    }

    /// Gets the border color, if set.
    pub fn border_color(&self) -> Option<BorderColor> {
        self.0.border_color
    }

    /// Gets the border width, if set.
    pub fn border_width(&self) -> Option<BorderWidth> {
        self.0.border_width
    }

    /// Gets the border radius, if set.
    pub fn border_radius(&self) -> Option<BorderRadius> {
        self.0.border_radius
    }

    /// Gets the border style, if set.
    pub fn border_style(&self) -> Option<BorderStyle> {
        self.0.border_style
    }

    /// Gets the opacity, if set.
    pub fn opacity(&self) -> Option<Opacity> {
        self.0.opacity
    }

    /// Gets the visibility, if set.
    pub fn visibility(&self) -> Option<Visibility> {
        self.0.visibility
    }

    /// Gets the z-index, if set.
    pub fn z_index(&self) -> Option<ZIndex> {
        self.0.z_index
    }

    /// Creates a weak reference to these styles.
    pub fn downgrade(&self) -> WeakSharedComputedStyles {
        WeakSharedComputedStyles(Gc::downgrade(&self.0))
    }

    /// Gets the reference count.
    pub fn strong_count(&self) -> usize {
        Gc::ref_count(&self.0).get()
    }
}

impl AsRef<ComputedStyles> for SharedComputedStyles {
    fn as_ref(&self) -> &ComputedStyles {
        &self.0
    }
}

impl std::ops::Deref for SharedComputedStyles {
    type Target = ComputedStyles;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Weak reference to shared computed styles.
#[derive(Debug, Clone)]
pub struct WeakSharedComputedStyles(Weak<ComputedStyles>);

impl WeakSharedComputedStyles {
    /// Attempts to upgrade the weak reference to a strong one.
    pub fn upgrade(&self) -> Option<SharedComputedStyles> {
        self.0.upgrade().map(|gc| SharedComputedStyles(gc))
    }

    /// Returns true if the strong count is zero.
    pub fn is_dead(&self) -> bool {
        self.0.strong_count() == 0
    }
}

/// A builder for creating shared styles efficiently.
#[derive(Debug, Clone, Default)]
pub struct SharedStyleBuilder {
    background_color: Option<BackgroundColor>,
    color: Option<TextColor>,
    text_color: Option<TextColor>,
    font_size: Option<FontSize>,
    font_family: Option<FontFamily>,
    font_weight: Option<FontWeight>,
    padding: Option<Padding>,
    margin: Option<Margin>,
    width: Option<Width>,
    height: Option<Height>,
    display: Option<Display>,
    flex_direction: Option<FlexDirection>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    flex_grow: Option<FlexGrow>,
    flex_shrink: Option<FlexShrink>,
    gap: Option<Gap>,
    border_color: Option<BorderColor>,
    border_width: Option<BorderWidth>,
    border_radius: Option<BorderRadius>,
    border_style: Option<BorderStyle>,
    opacity: Option<Opacity>,
    visibility: Option<Visibility>,
    z_index: Option<ZIndex>,
}

impl SharedStyleBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the background color.
    pub fn with_background_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.background_color = Some(BackgroundColor(color.into()));
        self
    }

    /// Sets the text color.
    pub fn with_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(TextColor(color.into()));
        self
    }

    /// Sets the text color.
    pub fn with_text_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.text_color = Some(TextColor(color.into()));
        self
    }

    /// Sets the font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(FontSize(size));
        self
    }

    /// Sets the font family.
    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(FontFamily(family.into()));
        self
    }

    /// Sets the font weight.
    pub fn with_font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Sets the padding.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Some(Padding(padding));
        self
    }

    /// Sets the margin.
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = Some(Margin(margin));
        self
    }

    /// Sets the width.
    pub fn with_width(mut self, width: Width) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height.
    pub fn with_height(mut self, height: Height) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the display property.
    pub fn with_display(mut self, display: Display) -> Self {
        self.display = Some(display);
        self
    }

    /// Sets the flex direction.
    pub fn with_flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = Some(direction);
        self
    }

    /// Sets the justify content property.
    pub fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = Some(justify);
        self
    }

    /// Sets the align items property.
    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.align_items = Some(align);
        self
    }

    /// Sets the flex grow property.
    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = Some(FlexGrow(grow));
        self
    }

    /// Sets the flex shrink property.
    pub fn with_flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = Some(FlexShrink(shrink));
        self
    }

    /// Sets the gap property.
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Some(Gap(gap));
        self
    }

    /// Sets the border color.
    pub fn with_border_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.border_color = Some(BorderColor(color.into()));
        self
    }

    /// Sets the border width.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = Some(BorderWidth(width));
        self
    }

    /// Sets the border radius.
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(BorderRadius(radius));
        self
    }

    /// Sets the border style.
    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Sets the opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(Opacity(opacity));
        self
    }

    /// Sets the visibility.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Sets the z-index.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(ZIndex(z_index));
        self
    }

    /// Builds the shared computed styles.
    pub fn build(&self) -> SharedComputedStyles {
        let mut computed = ComputedStyles::new();

        if let Some(bg) = &self.background_color {
            computed.background_color = Some(*bg);
        }
        if let Some(c) = &self.color {
            computed.color = Some(c.0);
        }
        if let Some(c) = &self.text_color {
            computed.text_color = Some(*c);
        }
        if let Some(fs) = &self.font_size {
            computed.font_size = Some(*fs);
        }
        if let Some(ff) = &self.font_family {
            computed.font_family = Some(ff.clone());
        }
        if let Some(fw) = &self.font_weight {
            computed.font_weight = Some(*fw);
        }
        if let Some(p) = &self.padding {
            computed.padding = Some(*p);
        }
        if let Some(m) = &self.margin {
            computed.margin = Some(*m);
        }
        if let Some(w) = &self.width {
            computed.width = Some(w.clone());
        }
        if let Some(h) = &self.height {
            computed.height = Some(h.clone());
        }
        if let Some(d) = &self.display {
            computed.display = Some(*d);
        }
        if let Some(fd) = &self.flex_direction {
            computed.flex_direction = Some(*fd);
        }
        if let Some(jc) = &self.justify_content {
            computed.justify_content = Some(*jc);
        }
        if let Some(ai) = &self.align_items {
            computed.align_items = Some(*ai);
        }
        if let Some(fg) = &self.flex_grow {
            computed.flex_grow = Some(*fg);
        }
        if let Some(fs) = &self.flex_shrink {
            computed.flex_shrink = Some(*fs);
        }
        if let Some(g) = &self.gap {
            computed.gap = Some(*g);
        }
        if let Some(bc) = &self.border_color {
            computed.border_color = Some(*bc);
        }
        if let Some(bw) = &self.border_width {
            computed.border_width = Some(*bw);
        }
        if let Some(br) = &self.border_radius {
            computed.border_radius = Some(*br);
        }
        if let Some(bs) = &self.border_style {
            computed.border_style = Some(*bs);
        }
        if let Some(o) = &self.opacity {
            computed.opacity = Some(*o);
        }
        if let Some(v) = &self.visibility {
            computed.visibility = Some(*v);
        }
        if let Some(zi) = &self.z_index {
            computed.z_index = Some(*zi);
        }

        SharedComputedStyles::new(computed)
    }
}

/// Creates a new shared style with the given background color.
pub fn shared_background_color<C: Into<Color>>(color: C) -> SharedComputedStyles {
    SharedStyleBuilder::new().with_background_color(color).build()
}

/// Creates a new shared style with the given text color.
pub fn shared_text_color<C: Into<Color>>(color: C) -> SharedComputedStyles {
    SharedStyleBuilder::new().with_text_color(color).build()
}

/// Creates a new shared style with padding.
pub fn shared_padding(padding: f32) -> SharedComputedStyles {
    SharedStyleBuilder::new().with_padding(padding).build()
}

/// Creates a new shared style with margin.
pub fn shared_margin(margin: f32) -> SharedComputedStyles {
    SharedStyleBuilder::new().with_margin(margin).build()
}

/// Creates a new shared style for a flex container.
pub fn shared_flex_container() -> SharedComputedStyles {
    SharedStyleBuilder::new().with_display(Display::Flex).build()
}

/// Creates a new shared style for a centered flex container.
pub fn shared_centered_flex() -> SharedComputedStyles {
    SharedStyleBuilder::new()
        .with_display(Display::Flex)
        .with_justify_content(JustifyContent::Center)
        .with_align_items(AlignItems::Center)
        .build()
}

/// Converts a WidgetStyles into SharedComputedStyles.
impl From<&crate::widget::WidgetStyles> for SharedComputedStyles {
    fn from(widget_styles: &crate::widget::WidgetStyles) -> Self {
        let props = widget_styles.to_properties();
        Self::from_properties(&props)
    }
}

/// A widget that supports shared style references.
pub trait StyledWidgetExtShared: Sized {
    /// Returns a reference to the shared styles, if set.
    fn shared_styles(&self) -> Option<&SharedComputedStyles>;

    /// Returns a mutable reference to the shared styles, if set.
    fn shared_styles_mut(&mut self) -> Option<&mut SharedComputedStyles>;

    /// Creates a new instance with the given shared styles.
    fn with_shared_styles(mut self, styles: SharedComputedStyles) -> Self {
        self.set_shared_styles(styles);
        self
    }

    /// Sets the shared styles.
    fn set_shared_styles(&mut self, styles: SharedComputedStyles);
}
