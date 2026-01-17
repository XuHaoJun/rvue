//! Layout node wrapper around Taffy

use rudo_gc::Gc;
use crate::component::Component;

/// Layout node wrapper around Taffy node
/// For MVP, this is a placeholder that will be expanded with Taffy integration
pub struct LayoutNode {
    pub component: Gc<Component>,
    pub is_dirty: bool,
}

impl LayoutNode {
    /// Create a new layout node for a component
    pub fn new(component: Gc<Component>) -> Self {
        Self {
            component,
            is_dirty: true,
        }
    }

    /// Mark the layout node as dirty (needs recalculation)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if the layout node is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}
