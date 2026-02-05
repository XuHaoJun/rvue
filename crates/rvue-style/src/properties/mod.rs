//! Core property types for styling widgets.

pub mod background;
pub mod border;
pub mod color;
pub mod computed_styles;
pub mod font;
pub mod layout;
pub mod overflow;
pub mod sizing;
pub mod spacing;
pub mod visibility;

pub use background::BackgroundColor;
pub use border::{BorderColor, BorderRadius, BorderStyle, BorderWidth};
pub use color::{Color, TextColor};
pub use computed_styles::ComputedStyles;
pub use font::{FontFamily, FontSize, FontWeight};
pub use layout::{
    AlignItems, AlignSelf, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink, Gap,
    JustifyContent,
};
pub use overflow::Overflow;
pub use sizing::{Height, MaxHeight, MaxWidth, MinHeight, MinWidth, Size, Width};
pub use spacing::{Margin, Padding};
pub use visibility::{Cursor, Opacity, Visibility, ZIndex};

pub use crate::selectors::ElementState;
