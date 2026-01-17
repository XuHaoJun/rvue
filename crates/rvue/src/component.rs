//! Component trait and lifecycle management

use rudo_gc::{Gc, Trace};
use crate::effect::Effect;

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
    Text { content: String },
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
    pub children: Vec<Gc<Component>>,
    pub parent: Option<Gc<Component>>,
    pub effects: Vec<Gc<Effect>>,
    pub props: ComponentProps,
}

unsafe impl Trace for Component {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        // Trace all GC-managed fields
        for child in &self.children {
            child.trace(visitor);
        }
        if let Some(parent) = &self.parent {
            parent.trace(visitor);
        }
        for effect in &self.effects {
            effect.trace(visitor);
        }
        self.props.trace(visitor);
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
    pub fn new(id: ComponentId, component_type: ComponentType, props: ComponentProps) -> Gc<Self> {
        Gc::new(Self {
            id,
            component_type,
            children: Vec::new(),
            parent: None,
            effects: Vec::new(),
            props,
        })
    }

    /// Add a child component
    pub fn add_child(&mut self, child: Gc<Component>) {
        self.children.push(child);
    }

    /// Set the parent component
    pub fn set_parent(&mut self, parent: Option<Gc<Component>>) {
        self.parent = parent;
    }

    /// Add an effect to this component
    pub fn add_effect(&mut self, effect: Gc<Effect>) {
        self.effects.push(effect);
    }
}

impl ComponentLifecycle for Component {
    fn mount(&self, _parent: Option<Gc<Component>>) {
        // Set parent reference
        // Note: We need mutable access, but Component is in Gc, so we'll need GcCell
        // For MVP, we'll handle this at a higher level
        
        // For Show components, mount/unmount children based on when condition
        // Note: We can't easily get the Gc wrapper from inside Component
        // For MVP, we'll mount children without parent reference
        if let ComponentType::Show = self.component_type {
            if let ComponentProps::Show { when } = &self.props {
                if *when {
                    // Mount children if visible
                    for child in &self.children {
                        child.mount(None);
                    }
                }
            }
        } else {
            // For other components, mount all children
            for child in &self.children {
                child.mount(None);
            }
        }
    }

    fn unmount(&self) {
        // Unmount all children
        for child in &self.children {
            child.unmount();
        }
        
        // Clean up effects
        // Effects are cleaned up by GC when component is dropped
    }

    fn update(&self) {
        // Run all effects that are dirty
        for effect in &self.effects {
            Effect::update_if_dirty(effect);
        }
        
        // For Show components, update children mounting based on when condition
        if let ComponentType::Show = self.component_type {
            if let ComponentProps::Show { when } = &self.props {
                if *when {
                    // Ensure children are mounted
                    for child in &self.children {
                        child.mount(None);
                    }
                } else {
                    // Unmount children if hidden
                    for child in &self.children {
                        child.unmount();
                    }
                }
            }
        }
        
        // Update all children
        for child in &self.children {
            child.update();
        }
    }
}
