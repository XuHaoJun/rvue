//! Type-safe widget styling extension.

use crate::properties::{
    AlignItems, BackgroundColor, BorderColor, BorderRadius, BorderWidth, Color, Cursor, Display,
    FlexDirection, FontSize, Gap, Height, JustifyContent, Margin, Opacity, Padding, TextColor,
    Visibility, Width, ZIndex,
};
use crate::property::{Properties, Property};
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

/// Shared styles that can be efficiently shared across multiple widgets.
///
/// Using `Gc<WidgetStyles>` allows multiple widgets to reference the same
/// style definition without copying.
#[derive(Debug, Default)]
pub struct WidgetStyles {
    background_color: Option<BackgroundColor>,
    color: Option<TextColor>,
    padding: Option<Padding>,
    margin: Option<Margin>,
    font_size: Option<FontSize>,
    width: Option<Width>,
    height: Option<Height>,
    display: Option<Display>,
    flex_direction: Option<FlexDirection>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    border_color: Option<BorderColor>,
    border_width: Option<BorderWidth>,
    border_radius: Option<BorderRadius>,
    opacity: Option<Opacity>,
    visibility: Option<Visibility>,
    cursor: Option<Cursor>,
    z_index: Option<ZIndex>,
    gap: Option<Gap>,
}

impl WidgetStyles {
    /// Creates a new empty style.
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

    /// Sets the font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(FontSize(size));
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

    /// Sets the cursor.
    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Sets the z-index.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(ZIndex(z_index));
        self
    }

    /// Sets the gap.
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Some(Gap(gap));
        self
    }

    /// Converts to a Properties container.
    pub fn to_properties(&self) -> Properties {
        let mut props = Properties::new();
        if let Some(ref p) = self.background_color {
            props.insert(*p);
        }
        if let Some(ref p) = self.color {
            props.insert(*p);
        }
        if let Some(ref p) = self.padding {
            props.insert(*p);
        }
        if let Some(ref p) = self.margin {
            props.insert(*p);
        }
        if let Some(ref p) = self.font_size {
            props.insert(*p);
        }
        if let Some(ref p) = self.width {
            props.insert(p.clone());
        }
        if let Some(ref p) = self.height {
            props.insert(p.clone());
        }
        if let Some(ref p) = self.display {
            props.insert(*p);
        }
        if let Some(ref p) = self.flex_direction {
            props.insert(*p);
        }
        if let Some(ref p) = self.justify_content {
            props.insert(*p);
        }
        if let Some(ref p) = self.align_items {
            props.insert(*p);
        }
        if let Some(ref p) = self.border_color {
            props.insert(*p);
        }
        if let Some(ref p) = self.border_width {
            props.insert(*p);
        }
        if let Some(ref p) = self.border_radius {
            props.insert(*p);
        }
        if let Some(ref p) = self.opacity {
            props.insert(*p);
        }
        if let Some(ref p) = self.visibility {
            props.insert(*p);
        }
        if let Some(ref p) = self.cursor {
            props.insert(p.clone());
        }
        if let Some(ref p) = self.z_index {
            props.insert(*p);
        }
        if let Some(ref p) = self.gap {
            props.insert(*p);
        }
        props
    }

    /// Creates a garbage-collected reference to these styles.
    pub fn shared(self) -> Gc<Self> {
        Gc::new(self)
    }
}

unsafe impl Trace for WidgetStyles {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}
