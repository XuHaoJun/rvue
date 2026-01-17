//! Styling system for components

/// Color representation
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Named(String),
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
    Medium,
    SemiBold,
    ExtraBold,
}

/// Spacing values (top, right, bottom, left)
#[derive(Debug, Clone, PartialEq)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Spacing {
    /// Create uniform spacing
    pub fn uniform(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    /// Create spacing with different horizontal and vertical values
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }

    /// Create spacing with all four values
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
}

/// Size constraint
#[derive(Debug, Clone, PartialEq)]
pub enum Size {
    /// Fixed size in pixels
    Pixels(f32),
    /// Percentage of parent (0.0 to 1.0)
    Percent(f32),
    /// Auto size (fit content)
    Auto,
    /// Fill available space
    Fill,
}

/// Border properties
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    pub style: BorderStyle,
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Alignment items (cross-axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

/// Justify content (main-axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Style structure for component styling
#[derive(Debug, Clone)]
pub struct Style {
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeight>,
    pub font_family: Option<String>,
    pub padding: Option<Spacing>,
    pub margin: Option<Spacing>,
    pub border: Option<Border>,
    pub border_radius: Option<f32>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Option<f32>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: None,
            background_color: None,
            font_size: None,
            font_weight: None,
            font_family: None,
            padding: None,
            margin: None,
            border: None,
            border_radius: None,
            width: None,
            height: None,
            flex_direction: None,
            flex_grow: None,
            flex_shrink: None,
            align_items: None,
            justify_content: None,
            gap: None,
        }
    }
}
