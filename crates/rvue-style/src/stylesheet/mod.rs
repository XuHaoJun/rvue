//! CSS stylesheet parsing and rule management.

pub mod parser;
pub mod resolver;
pub mod rule;

pub use resolver::StyleResolver;
pub use rule::{StyleRule, Stylesheet};
