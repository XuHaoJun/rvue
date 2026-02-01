//! CSS stylesheet parsing and rule management.

pub mod defaults;
pub mod parser;
pub mod resolver;
pub mod rule;

pub use defaults::default_stylesheet;
pub use resolver::StyleResolver;
pub use rule::{StyleRule, Stylesheet};
