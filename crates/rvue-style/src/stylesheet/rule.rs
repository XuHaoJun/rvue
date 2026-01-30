//! Style rules and stylesheet structures.

use crate::property::Properties;
use std::cmp::Ordering;

/// A style rule consisting of a selector and properties.
#[derive(Debug)]
pub struct StyleRule {
    pub selector: String,
    pub specificity: Specificity,
    pub properties: Properties,
}

impl StyleRule {
    #[inline]
    pub fn new(selector: String, properties: Properties) -> Self {
        let specificity = Specificity::from_selector(&selector);
        Self { selector, specificity, properties }
    }

    #[inline]
    pub fn parse(selector: &str, properties: Properties) -> Self {
        Self::new(selector.to_string(), properties)
    }
}

/// Specificity of a CSS selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Specificity {
    pub id: u32,
    pub class: u32,
    pub element: u32,
}

impl Specificity {
    #[inline]
    pub fn new(id: u32, class: u32, element: u32) -> Self {
        Self { id, class, element }
    }

    pub fn from_selector(selector: &str) -> Self {
        let mut id = 0;
        let mut class = 0;
        let mut element = 0;

        let mut chars = selector.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    id += 1;
                    while chars
                        .peek()
                        .map(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                }
                '.' => {
                    class += 1;
                    while chars
                        .peek()
                        .map(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                }
                '[' => {
                    class += 1;
                    while chars.next() != Some(']') {}
                }
                ':' => {
                    class += 1;
                    if chars.peek() == Some(&':') {
                        chars.next();
                    }
                    while chars.peek().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                        chars.next();
                    }
                    if chars.peek() == Some(&'(') {
                        while chars.next() != Some(')') {}
                    }
                }
                _ if c.is_alphabetic() || c == '_' || c == '*' => {
                    element += 1;
                    while chars
                        .peek()
                        .map(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                }
                _ => {}
            }
        }

        Self { id, class, element }
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.class.cmp(&other.class))
            .then_with(|| self.element.cmp(&other.element))
    }
}

/// A collection of style rules.
#[derive(Default, Debug)]
pub struct Stylesheet {
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    #[inline]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    #[inline]
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    #[inline]
    pub fn rules(&self) -> impl Iterator<Item = &StyleRule> {
        self.rules.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

unsafe impl rudo_gc::Trace for Stylesheet {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}
