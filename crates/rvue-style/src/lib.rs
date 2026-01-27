//! rvue-style - GPU-accelerated GUI styling system
//!
//! A simple CSS-compatible styling system for rvue.

#![warn(missing_docs)]

pub mod properties;
pub mod property;
pub mod selectors;
pub mod stylesheet;

pub use properties::{Color, ElementState, Margin, Padding, Size};
pub use property::{Properties, Property};
