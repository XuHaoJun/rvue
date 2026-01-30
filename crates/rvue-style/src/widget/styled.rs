//! Type-safe widget styling extension.

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Color, Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize,
    FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Padding, Size, TextColor, Visibility,
    Width, ZIndex,
};
use crate::property::{Properties, Property, StyleStore};
use rudo_gc::{Gc, Trace};

/// Extension trait for adding styles to widgets.
///
/// This trait provides type-safe builder methods for styling widgets.
/// All methods consume `self` and return a new instance with the property applied.
///
/// # Example
///
/// ```ignore
/// use rvue_style::widget::StyledWidgetExt;
///
/// let styled_button = Button::new("Click Me")
///     .style_background(Color::rgb(0, 120, 215))
///     .style_padding(Padding::uniform(12.0))
///     .style_color(Color::rgb(255, 255, 255));
/// ```
pub trait StyledWidgetExt: Sized {
    /// Returns a reference to the widget's properties.
    fn properties(&self) -> &Properties;

    /// Returns a mutable reference to the widget's properties.
    fn properties_mut(&mut self) -> &mut Properties;

    /// Creates a new instance with the given property applied.
    ///
    /// This is the generic method used by all specific style methods.
    fn with_style<P: Property>(mut self, property: P) -> Self {
        self.properties_mut().insert(property);
        self
    }

    /// Sets the background color.
    fn style_background<C: Into<Color>>(self, color: C) -> Self {
        self.with_style(BackgroundColor(color.into()))
    }

    /// Sets the text color.
    fn style_color<C: Into<Color>>(self, color: C) -> Self {
        self.with_style(TextColor(color.into()))
    }

    /// Sets the padding (uniform on all sides).
    fn style_padding(self, padding: f32) -> Self {
        self.with_style(Padding(padding))
    }

    /// Sets the margin (uniform on all sides).
    fn style_margin(self, margin: f32) -> Self {
        self.with_style(Margin(margin))
    }

    /// Sets the font size.
    fn style_font_size(self, size: f32) -> Self {
        self.with_style(FontSize(size))
    }

    /// Sets the width.
    fn style_width(self, width: Width) -> Self {
        self.with_style(width)
    }

    /// Sets the height.
    fn style_height(self, height: Height) -> Self {
        self.with_style(height)
    }

    /// Sets the display property.
    fn style_display(self, display: Display) -> Self {
        self.with_style(display)
    }

    /// Sets the flex direction.
    fn style_flex_direction(self, direction: FlexDirection) -> Self {
        self.with_style(direction)
    }

    /// Sets the justify content property.
    fn style_justify_content(self, justify: JustifyContent) -> Self {
        self.with_style(justify)
    }

    /// Sets the align items property.
    fn style_align_items(self, align: AlignItems) -> Self {
        self.with_style(align)
    }

    /// Sets the border color.
    fn style_border_color<C: Into<Color>>(self, color: C) -> Self {
        self.with_style(BorderColor(color.into()))
    }

    /// Sets the border width.
    fn style_border_width(self, width: f32) -> Self {
        self.with_style(BorderWidth(width))
    }

    /// Sets the border radius.
    fn style_border_radius(self, radius: f32) -> Self {
        self.with_style(BorderRadius(radius))
    }

    /// Sets the opacity (0.0 to 1.0).
    fn style_opacity(self, opacity: f32) -> Self {
        self.with_style(Opacity(opacity))
    }

    /// Sets the visibility.
    fn style_visibility(self, visibility: Visibility) -> Self {
        self.with_style(visibility)
    }

    /// Sets the cursor.
    fn style_cursor(self, cursor: Cursor) -> Self {
        self.with_style(cursor)
    }

    /// Sets the z-index.
    fn style_z_index(self, z_index: i32) -> Self {
        self.with_style(ZIndex(z_index))
    }

    /// Sets the gap between flex/grid items.
    fn style_gap(self, gap: f32) -> Self {
        self.with_style(Gap(gap))
    }
}

/// A widget with associated styles.
///
/// This is a marker trait that widgets can implement to indicate
/// they support the style system.
pub trait StyledWidget {
    /// Returns a reference to the properties.
    fn style(&self) -> &Properties;

    /// Returns a mutable reference to the properties.
    fn style_mut(&mut self) -> &mut Properties;

    /// Sets the properties.
    fn set_style(&mut self, properties: Properties);
}

/// Unified style storage with zero-cost property access.
///
/// This struct provides type-safe style storage without the runtime
/// overhead of type erasure. Each property is stored as an optional
/// direct field, enabling O(1) access without hashing or downcasting.
#[derive(Debug, Default, Clone)]
pub struct StyleData {
    pub background_color: Option<BackgroundColor>,
    pub color: Option<TextColor>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
    pub font_size: Option<FontSize>,
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
    pub width: Option<Width>,
    pub height: Option<Height>,
    pub display: Option<Display>,
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub flex_grow: Option<FlexGrow>,
    pub flex_shrink: Option<FlexShrink>,
    pub flex_basis: Option<FlexBasis>,
    pub border_color: Option<BorderColor>,
    pub border_width: Option<BorderWidth>,
    pub border_radius: Option<BorderRadius>,
    pub border_style: Option<BorderStyle>,
    pub opacity: Option<Opacity>,
    pub visibility: Option<Visibility>,
    pub cursor: Option<Cursor>,
    pub z_index: Option<ZIndex>,
    pub gap: Option<Gap>,
    pub size: Option<Size>,
}

impl StyleData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_background_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.background_color = Some(BackgroundColor(color.into()));
        self
    }

    pub fn with_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(TextColor(color.into()));
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Some(Padding(padding));
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = Some(Margin(margin));
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(FontSize(size));
        self
    }

    pub fn with_font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    pub fn with_font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn with_width(mut self, width: Width) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: Height) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.width = Some(Width(size.clone()));
        self.height = Some(Height(size.clone()));
        self.size = Some(size);
        self
    }

    pub fn with_display(mut self, display: Display) -> Self {
        self.display = Some(display);
        self
    }

    pub fn with_flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = Some(direction);
        self
    }

    pub fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = Some(justify);
        self
    }

    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.align_items = Some(align);
        self
    }

    pub fn with_align_self(mut self, align: AlignSelf) -> Self {
        self.align_self = Some(align);
        self
    }

    pub fn with_flex_grow(mut self, grow: FlexGrow) -> Self {
        self.flex_grow = Some(grow);
        self
    }

    pub fn with_flex_shrink(mut self, shrink: FlexShrink) -> Self {
        self.flex_shrink = Some(shrink);
        self
    }

    pub fn with_flex_basis(mut self, basis: FlexBasis) -> Self {
        self.flex_basis = Some(basis);
        self
    }

    pub fn with_border_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.border_color = Some(BorderColor(color.into()));
        self
    }

    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = Some(BorderWidth(width));
        self
    }

    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = Some(style);
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(BorderRadius(radius));
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(Opacity(opacity));
        self
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(ZIndex(z_index));
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Some(Gap(gap));
        self
    }

    pub fn to_properties(&self) -> Properties {
        self.clone().into()
    }

    pub fn shared(self) -> Gc<Self> {
        Gc::new(self)
    }
}

impl StyleStore for StyleData {
    fn background_color(&self) -> Option<&BackgroundColor> {
        self.background_color.as_ref()
    }

    fn color(&self) -> Option<&TextColor> {
        self.color.as_ref()
    }

    fn padding(&self) -> Option<&Padding> {
        self.padding.as_ref()
    }

    fn margin(&self) -> Option<&Margin> {
        self.margin.as_ref()
    }

    fn font_size(&self) -> Option<&FontSize> {
        self.font_size.as_ref()
    }

    fn font_family(&self) -> Option<&FontFamily> {
        self.font_family.as_ref()
    }

    fn font_weight(&self) -> Option<&FontWeight> {
        self.font_weight.as_ref()
    }

    fn width(&self) -> Option<&Width> {
        self.width.as_ref()
    }

    fn height(&self) -> Option<&Height> {
        self.height.as_ref()
    }

    fn display(&self) -> Option<&Display> {
        self.display.as_ref()
    }

    fn flex_direction(&self) -> Option<&FlexDirection> {
        self.flex_direction.as_ref()
    }

    fn justify_content(&self) -> Option<&JustifyContent> {
        self.justify_content.as_ref()
    }

    fn align_items(&self) -> Option<&AlignItems> {
        self.align_items.as_ref()
    }

    fn align_self(&self) -> Option<&AlignSelf> {
        self.align_self.as_ref()
    }

    fn flex_grow(&self) -> Option<&FlexGrow> {
        self.flex_grow.as_ref()
    }

    fn flex_shrink(&self) -> Option<&FlexShrink> {
        self.flex_shrink.as_ref()
    }

    fn flex_basis(&self) -> Option<&FlexBasis> {
        self.flex_basis.as_ref()
    }

    fn border_color(&self) -> Option<&BorderColor> {
        self.border_color.as_ref()
    }

    fn border_width(&self) -> Option<&BorderWidth> {
        self.border_width.as_ref()
    }

    fn border_radius(&self) -> Option<&BorderRadius> {
        self.border_radius.as_ref()
    }

    fn border_style(&self) -> Option<&BorderStyle> {
        self.border_style.as_ref()
    }

    fn opacity(&self) -> Option<&Opacity> {
        self.opacity.as_ref()
    }

    fn visibility(&self) -> Option<&Visibility> {
        self.visibility.as_ref()
    }

    fn cursor(&self) -> Option<&Cursor> {
        self.cursor.as_ref()
    }

    fn z_index(&self) -> Option<&ZIndex> {
        self.z_index.as_ref()
    }

    fn gap(&self) -> Option<&Gap> {
        self.gap.as_ref()
    }

    fn size(&self) -> Option<&Size> {
        self.size.as_ref()
    }
}

impl From<StyleData> for Properties {
    fn from(style: StyleData) -> Self {
        let mut props = Properties::new();
        if let Some(p) = style.background_color {
            props.insert(p);
        }
        if let Some(p) = style.color {
            props.insert(p);
        }
        if let Some(p) = style.padding {
            props.insert(p);
        }
        if let Some(p) = style.margin {
            props.insert(p);
        }
        if let Some(p) = style.font_size {
            props.insert(p);
        }
        if let Some(p) = style.font_family {
            props.insert(p);
        }
        if let Some(p) = style.font_weight {
            props.insert(p);
        }
        if let Some(p) = style.width {
            props.insert(p);
        }
        if let Some(p) = style.height {
            props.insert(p);
        }
        if let Some(p) = style.display {
            props.insert(p);
        }
        if let Some(p) = style.flex_direction {
            props.insert(p);
        }
        if let Some(p) = style.justify_content {
            props.insert(p);
        }
        if let Some(p) = style.align_items {
            props.insert(p);
        }
        if let Some(p) = style.align_self {
            props.insert(p);
        }
        if let Some(p) = style.flex_grow {
            props.insert(p);
        }
        if let Some(p) = style.flex_shrink {
            props.insert(p);
        }
        if let Some(p) = style.flex_basis {
            props.insert(p);
        }
        if let Some(p) = style.border_color {
            props.insert(p);
        }
        if let Some(p) = style.border_width {
            props.insert(p);
        }
        if let Some(p) = style.border_radius {
            props.insert(p);
        }
        if let Some(p) = style.border_style {
            props.insert(p);
        }
        if let Some(p) = style.opacity {
            props.insert(p);
        }
        if let Some(p) = style.visibility {
            props.insert(p);
        }
        if let Some(p) = style.cursor {
            props.insert(p);
        }
        if let Some(p) = style.z_index {
            props.insert(p);
        }
        if let Some(p) = style.gap {
            props.insert(p);
        }
        if let Some(p) = style.size {
            props.insert(p);
        }
        props
    }
}

unsafe impl Trace for StyleData {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

#[doc(hidden)]
pub type WidgetStyles = StyleData;
