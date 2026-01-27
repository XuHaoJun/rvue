//! Element type for CSS selector matching.

use crate::selectors::ElementState;
use rudo_gc::Trace;

/// Represents a styled element for selector matching.
#[derive(Clone, Debug, Default)]
pub struct RvueElement {
    /// Element tag name
    pub tag_name: String,
    /// Element classes
    pub classes: Vec<String>,
    /// Element ID
    pub id: Option<String>,
    /// Element state for pseudo-class matching
    pub state: ElementState,
    /// Parent element reference
    pub parent: Option<Box<RvueElement>>,
    /// Child elements
    pub children: Vec<RvueElement>,
}

unsafe impl Trace for RvueElement {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl RvueElement {
    /// Creates a new element with the given tag name.
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
            classes: Vec::new(),
            id: None,
            state: ElementState::empty(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Adds a class to the element.
    pub fn with_class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Sets the element ID.
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Adds a child element.
    pub fn with_child(mut self, child: RvueElement) -> Self {
        self.children.push(child);
        self
    }

    /// Sets the parent element.
    pub fn with_parent(mut self, parent: RvueElement) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    /// Checks if element matches a given tag name.
    pub fn has_tag_name(&self, tag: &str) -> bool {
        self.tag_name.eq_ignore_ascii_case(tag)
    }

    /// Checks if element has a specific class.
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c.eq_ignore_ascii_case(class))
    }

    /// Checks if element has a specific ID.
    pub fn has_id(&self, id: &str) -> bool {
        self.id.as_ref().map_or(false, |i| i.eq_ignore_ascii_case(id))
    }

    /// Checks if element is in a specific state.
    pub fn is_in_state(&self, state: ElementState) -> bool {
        self.state.intersects(state)
    }
}
