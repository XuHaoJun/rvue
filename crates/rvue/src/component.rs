//! Component trait and lifecycle management

use crate::effect::Effect;
use crate::layout::LayoutNode;
use crate::text::TextContext;
use rudo_gc::{Gc, GcCell, Trace};
use std::sync::atomic::{AtomicBool, Ordering};
use taffy::TaffyTree;

/// Unique identifier for a component
pub type ComponentId = u64;

/// Component type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentType {
    Text,
    Button,
    TextInput,
    NumberInput,
    Checkbox,
    Radio,
    Show,
    For,
    Flex,
    Custom(String),
}

/// Component properties (variant type for different widget types)
/// For MVP, we'll use a simplified approach - props are stored as strings/values
/// and converted as needed by widget implementations
#[derive(Debug, Clone)]
pub enum ComponentProps {
    Text { content: String, font_size: Option<f32>, color: Option<vello::peniko::Color> },
    Button { label: String },
    TextInput { value: String },
    NumberInput { value: f64 },
    Checkbox { checked: bool },
    Radio { value: String, checked: bool },
    Show { when: bool },
    For { item_count: usize },
    Flex { direction: String, gap: f32, align_items: String, justify_content: String },
    Custom { data: String },
}

unsafe impl Trace for ComponentProps {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // ComponentProps contains only primitive types and strings, no GC pointers
        // For MVP, we don't need to trace anything
    }
}

/// Component structure representing a UI building block
pub struct Component {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub children: GcCell<Vec<Gc<Component>>>,
    pub parent: GcCell<Option<Gc<Component>>>,
    pub effects: GcCell<Vec<Gc<Effect>>>,
    pub props: GcCell<ComponentProps>,
    pub is_dirty: AtomicBool,
    pub user_data: GcCell<Option<Box<dyn std::any::Any>>>,
    pub layout_node: GcCell<Option<LayoutNode>>,
}

unsafe impl Trace for Component {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.children.trace(visitor);
        self.parent.trace(visitor);
        self.effects.trace(visitor);
        self.props.trace(visitor);
        self.layout_node.trace(visitor);
        // user_data doesn't contain GC pointers
    }
}

/// Component trait defining lifecycle methods
pub trait ComponentLifecycle {
    /// Mount the component to the component tree
    fn mount(&self, parent: Option<Gc<Component>>);

    /// Unmount the component from the component tree
    fn unmount(&self);

    /// Update the component when signals change
    fn update(&self);
}

impl Component {
    /// Create a new component with the given type and props
    /// Optimized: Pre-allocate with capacity hints for common component trees
    pub fn new(id: ComponentId, component_type: ComponentType, props: ComponentProps) -> Gc<Self> {
        // Pre-allocate children vector with small capacity for common case
        // This reduces reallocations during component tree construction
        let initial_children_capacity = match component_type {
            ComponentType::Flex | ComponentType::For => 8, // Containers typically have multiple children
            _ => 0, // Leaf components typically have no children
        };

        Gc::new(Self {
            id,
            component_type,
            children: GcCell::new(Vec::with_capacity(initial_children_capacity)),
            parent: GcCell::new(None),
            effects: GcCell::new(Vec::new()),
            props: GcCell::new(props),
            is_dirty: AtomicBool::new(true),
            user_data: GcCell::new(None),
            layout_node: GcCell::new(None),
        })
    }

    /// Mark the component as dirty (needs re-render)
    pub fn mark_dirty(&self) {
        self.is_dirty.store(true, Ordering::SeqCst);
        // Propagate dirty flag upwards so parents know they need to re-render
        if let Some(parent) = self.parent.borrow().as_ref() {
            parent.mark_dirty();
        }
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&self) {
        self.is_dirty.store(false, Ordering::SeqCst);
    }

    /// Check if the component is dirty
    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(Ordering::SeqCst)
    }

    /// Get user data
    pub fn user_data(&self) -> &GcCell<Option<Box<dyn std::any::Any>>> {
        &self.user_data
    }

    /// Get layout node (cloned)
    pub fn layout_node(&self) -> Option<LayoutNode> {
        self.layout_node.borrow().clone()
    }

    /// Add a child component
    pub fn add_child(&self, child: Gc<Component>) {
        self.children.borrow_mut().push(Gc::clone(&child));
    }

    /// Set layout node
    pub fn set_layout_node(&self, layout_node: LayoutNode) {
        *self.layout_node.borrow_mut() = Some(layout_node);
    }

    /// Clean up layout node from Taffy tree when component is unmounted
    pub fn cleanup_layout(&self, taffy: &mut TaffyTree<()>) {
        if let Some(layout_node) = self.layout_node() {
            if let Some(node_id) = layout_node.taffy_node() {
                let _ = taffy.remove(node_id);
            }
        }
        for child in self.children.borrow().iter() {
            child.cleanup_layout(taffy);
        }
    }

    /// Set the parent component
    pub fn set_parent(&self, parent: Option<Gc<Component>>) {
        *self.parent.borrow_mut() = parent;
    }

    /// Add an effect to this component
    pub fn add_effect(&self, effect: Gc<Effect>) {
        self.effects.borrow_mut().push(effect);
    }
}

/// Build layout tree recursively in a shared TaffyTree and return the root layout node
pub fn build_layout_tree(
    component: &Gc<Component>,
    taffy: &mut TaffyTree<()>,
    text_context: &mut TextContext,
) -> LayoutNode {
    // Build child layout nodes first in the same tree
    let child_layouts: Vec<LayoutNode> = component
        .children
        .borrow()
        .iter()
        .map(|child| build_layout_tree(child, taffy, text_context))
        .collect();

    // Get Taffy node IDs from child layouts
    let child_node_ids: Vec<taffy::NodeId> =
        child_layouts.iter().filter_map(|ln| ln.taffy_node()).collect();

    // Build this node with children in the shared tree
    let node = LayoutNode::build_in_tree(taffy, component, &child_node_ids, text_context);

    // Store child layouts in their dedicated field for later retrieval
    for (child, child_layout) in component.children.borrow().iter().zip(child_layouts.iter()) {
        child.set_layout_node(child_layout.clone());
        child.set_parent(Some(Gc::clone(component)));
    }

    node
}

impl ComponentLifecycle for Component {
    fn mount(&self, _parent: Option<Gc<Component>>) {
        // For Show components, mount/unmount children based on when condition
        if let ComponentType::Show = self.component_type {
            if let ComponentProps::Show { when } = &*self.props.borrow() {
                if *when {
                    // Mount children if visible
                    for child in self.children.borrow().iter() {
                        child.mount(None);
                    }
                }
            }
        } else {
            // For other components, mount all children
            for child in self.children.borrow().iter() {
                child.mount(None);
            }
        }
    }

    fn unmount(&self) {
        // Unmount all children
        for child in self.children.borrow().iter() {
            child.unmount();
        }

        // Clean up effects
        // Effects are cleaned up by GC when component is dropped
    }

    fn update(&self) {
        // Run all effects that are dirty
        for effect in self.effects.borrow().iter() {
            Effect::update_if_dirty(effect);
        }

        // For Show components, update children mounting based on when condition
        if let ComponentType::Show = self.component_type {
            if let ComponentProps::Show { when } = &*self.props.borrow() {
                if *when {
                    // Ensure children are mounted
                    for child in self.children.borrow().iter() {
                        child.mount(None);
                    }
                } else {
                    // Unmount children if hidden
                    for child in self.children.borrow().iter() {
                        child.unmount();
                    }
                }
            }
        }

        // Update all children
        for child in self.children.borrow().iter() {
            child.update();
        }
    }
}

/// Propagate layout results from TaffyTree back to components
pub fn propagate_layout_results(component: &Gc<Component>, taffy: &TaffyTree<()>) {
    // Update this component's layout node with result from Taffy
    if let Some(mut layout_node) = component.layout_node() {
        if let Some(node_id) = layout_node.taffy_node() {
            if let Ok(layout) = taffy.layout(node_id) {
                layout_node.layout_result = Some(*layout);
                layout_node.is_dirty = false;
                component.set_layout_node(layout_node);
            }
        }
    }

    // Recurse to children
    for child in component.children.borrow().iter() {
        propagate_layout_results(child, taffy);
    }
}
