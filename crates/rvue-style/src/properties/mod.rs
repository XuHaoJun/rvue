//! Core property types for styling widgets.

pub mod color;
pub mod sizing;
pub mod spacing;

pub use color::Color;
pub use sizing::{Height, Size, Width};
pub use spacing::{Margin, Padding};

pub use crate::selectors::ElementState;
