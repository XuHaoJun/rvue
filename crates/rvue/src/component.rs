//! Component trait and lifecycle management

use crate::effect::Effect;
use crate::event::handler::EventHandlers;
use crate::event::status::{ComponentFlags, StatusUpdate};
use crate::layout::LayoutNode;
use crate::text::TextContext;
use rudo_gc::{Gc, GcCell, Trace};
use std::any::{Any, TypeId};
use std::sync::atomic::AtomicBool;
use taffy::TaffyTree;
use vello::Scene;

/// Unique identifier for a component
use std::sync::atomic::{AtomicU64, Ordering};

pub static NEXT_COMPONENT_ID: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn next_component_id() -> ComponentId {
    NEXT_COMPONENT_ID.fetch_add(1, Ordering::SeqCst)
}

pub type ComponentId = u64;

pub struct ContextEntry {
    pub type_id: TypeId,
    pub value: ContextValueEnum,
}

unsafe impl Trace for ContextEntry {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.value.trace(visitor);
    }
}

#[derive(Clone)]
pub enum ContextValueEnum {
    I32(Gc<i32>),
    I64(Gc<i64>),
    F64(Gc<f64>),
    Bool(Gc<bool>),
    GcString(Gc<String>),
    GcVecString(Gc<Vec<String>>),
}

impl ContextValueEnum {
    pub fn from_value<T>(value: T) -> Self
    where
        T: Clone + 'static,
        T: Trace,
    {
        let type_id = TypeId::of::<T>();
        let gc = Gc::new(value);
        let ptr = Gc::internal_ptr(&gc);
        std::mem::forget(gc);
        if type_id == TypeId::of::<i32>() {
            let gc_i32: Gc<i32> = unsafe { Gc::from_raw(ptr) };
            return Self::I32(gc_i32);
        }
        if type_id == TypeId::of::<i64>() {
            let gc_i64: Gc<i64> = unsafe { Gc::from_raw(ptr) };
            return Self::I64(gc_i64);
        }
        if type_id == TypeId::of::<f64>() {
            let gc_f64: Gc<f64> = unsafe { Gc::from_raw(ptr) };
            return Self::F64(gc_f64);
        }
        if type_id == TypeId::of::<bool>() {
            let gc_bool: Gc<bool> = unsafe { Gc::from_raw(ptr) };
            return Self::Bool(gc_bool);
        }
        if type_id == TypeId::of::<String>() {
            let gc_string: Gc<String> = unsafe { Gc::from_raw(ptr) };
            return Self::GcString(gc_string);
        }
        if type_id == TypeId::of::<Vec<String>>() {
            let gc_vec: Gc<Vec<String>> = unsafe { Gc::from_raw(ptr) };
            return Self::GcVecString(gc_vec);
        }
        panic!("Unsupported context type");
    }

    pub fn to_gc<T>(&self) -> Option<Gc<T>>
    where
        T: 'static,
        T: Trace,
    {
        match self {
            ContextValueEnum::I32(gc) => {
                if TypeId::of::<T>() == TypeId::of::<i32>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<i32> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
            ContextValueEnum::I64(gc) => {
                if TypeId::of::<T>() == TypeId::of::<i64>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<i64> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
            ContextValueEnum::F64(gc) => {
                if TypeId::of::<T>() == TypeId::of::<f64>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<f64> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
            ContextValueEnum::Bool(gc) => {
                if TypeId::of::<T>() == TypeId::of::<bool>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<bool> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
            ContextValueEnum::GcString(gc) => {
                if TypeId::of::<T>() == TypeId::of::<String>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<String> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
            ContextValueEnum::GcVecString(gc) => {
                if TypeId::of::<T>() == TypeId::of::<Vec<String>>() {
                    let ptr = Gc::internal_ptr(gc);
                    let cloned = Gc::clone(gc);
                    let from_raw: Gc<Vec<String>> = unsafe { Gc::from_raw(ptr) };
                    std::mem::forget(from_raw);
                    let result: Gc<T> = unsafe { std::mem::transmute(cloned) };
                    Some(result)
                } else {
                    None
                }
            }
        }
    }
}

unsafe impl Trace for ContextValueEnum {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        match self {
            ContextValueEnum::I32(gc) => gc.trace(visitor),
            ContextValueEnum::I64(gc) => gc.trace(visitor),
            ContextValueEnum::F64(gc) => gc.trace(visitor),
            ContextValueEnum::Bool(gc) => gc.trace(visitor),
            ContextValueEnum::GcString(gc) => gc.trace(visitor),
            ContextValueEnum::GcVecString(gc) => gc.trace(visitor),
        }
    }
}

/// Wrapper for vello::Scene to implement Trace
#[derive(Default, Clone)]
pub struct SceneWrapper(pub Scene);

unsafe impl Trace for SceneWrapper {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {
        // vello::Scene does not contain any GC pointers
    }
}

/// Trait for values that can be stored in context
pub trait ContextValue: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> ContextValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

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
#[derive(Debug, Clone)]
pub enum ComponentProps {
    Text {
        content: String,
        styles: Option<rvue_style::ComputedStyles>,
    },
    Button {
        label: String,
        styles: Option<rvue_style::ComputedStyles>,
    },
    TextInput {
        value: String,
        styles: Option<rvue_style::ComputedStyles>,
    },
    NumberInput {
        value: f64,
        styles: Option<rvue_style::ComputedStyles>,
    },
    Checkbox {
        checked: bool,
        styles: Option<rvue_style::ComputedStyles>,
    },
    Radio {
        value: String,
        checked: bool,
        styles: Option<rvue_style::ComputedStyles>,
    },
    Show {
        when: bool,
    },
    For {
        item_count: usize,
    },
    KeyedFor {
        item_count: usize,
    },
    Flex {
        direction: String,
        gap: f32,
        align_items: String,
        justify_content: String,
        styles: Option<rvue_style::ComputedStyles>,
    },
    Custom {
        data: String,
    },
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
    pub is_updating: AtomicBool,
    pub user_data: GcCell<Option<Box<dyn std::any::Any>>>,
    pub layout_node: GcCell<Option<LayoutNode>>,
    pub flags: GcCell<ComponentFlags>,
    pub is_hovered: GcCell<bool>,
    pub has_hovered: GcCell<bool>,
    pub is_active: GcCell<bool>,
    pub has_active: GcCell<bool>,
    pub is_focused: GcCell<bool>,
    pub has_focus_target: GcCell<bool>,
    pub event_handlers: GcCell<EventHandlers>,
    pub vello_cache: GcCell<Option<SceneWrapper>>,
    pub contexts: GcCell<Vec<ContextEntry>>,
    pub cleanups: GcCell<Vec<Box<dyn FnOnce() + 'static>>>,
}

unsafe impl Trace for Component {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.children.trace(visitor);
        // self.parent.trace(visitor); // REMOVED: Tracing parent creates Component <-> Component cycles
        self.effects.trace(visitor);
        self.props.trace(visitor);
        self.layout_node.trace(visitor);
        self.flags.trace(visitor);
        self.is_hovered.trace(visitor);
        self.has_hovered.trace(visitor);
        self.is_active.trace(visitor);
        self.has_active.trace(visitor);
        self.is_focused.trace(visitor);
        self.has_focus_target.trace(visitor);
        // Note: vello::Scene is not GC-managed, but GcCell needs tracing if it could contain GC pointers.
        // vello::Scene itself doesn't contain GC pointers, so we just trace the cell.
        self.vello_cache.trace(visitor);
        // Cleanups are not traced since they are closures
        // Trace context values by directly visiting Gc pointers
        self.contexts.trace(visitor);
    }
}

impl Clone for Component {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            component_type: self.component_type.clone(),
            children: GcCell::new(self.children.borrow().clone()),
            parent: GcCell::new(None),
            effects: GcCell::new(self.effects.borrow().clone()),
            props: GcCell::new(self.props.borrow().clone()),
            is_dirty: AtomicBool::new(self.is_dirty.load(Ordering::SeqCst)),
            is_updating: AtomicBool::new(false),
            user_data: GcCell::new(None),
            layout_node: GcCell::new(self.layout_node.borrow().clone()),
            flags: GcCell::new(*self.flags.borrow()),
            is_hovered: GcCell::new(*self.is_hovered.borrow()),
            has_hovered: GcCell::new(*self.has_hovered.borrow()),
            is_active: GcCell::new(*self.is_active.borrow()),
            has_active: GcCell::new(*self.has_active.borrow()),
            is_focused: GcCell::new(*self.is_focused.borrow()),
            has_focus_target: GcCell::new(*self.has_focus_target.borrow()),
            event_handlers: GcCell::new(self.event_handlers.borrow().clone()),
            vello_cache: GcCell::new(self.vello_cache.borrow().clone()),
            contexts: GcCell::new(Vec::new()),
            cleanups: GcCell::new(Vec::new()),
        }
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
            ComponentType::Flex => 8, // Only real containers have multiple children
            _ => 0,                   // Leaf components typically have no children
        };

        let mut flags = ComponentFlags::empty();
        match component_type {
            ComponentType::Button
            | ComponentType::TextInput
            | ComponentType::NumberInput
            | ComponentType::Checkbox
            | ComponentType::Radio => {
                flags.insert(ComponentFlags::ACCEPTS_POINTER);
            }
            _ => {}
        }

        Gc::new(Self {
            id,
            component_type,
            children: GcCell::new(Vec::with_capacity(initial_children_capacity)),
            parent: GcCell::new(None),
            effects: GcCell::new(Vec::new()),
            props: GcCell::new(props),
            is_dirty: AtomicBool::new(true),
            is_updating: AtomicBool::new(false),
            user_data: GcCell::new(None),
            layout_node: GcCell::new(None),
            flags: GcCell::new(flags),
            is_hovered: GcCell::new(false),
            has_hovered: GcCell::new(false),
            is_active: GcCell::new(false),
            has_active: GcCell::new(false),
            is_focused: GcCell::new(false),
            has_focus_target: GcCell::new(false),
            event_handlers: GcCell::new(EventHandlers::default()),
            vello_cache: GcCell::new(None),
            contexts: GcCell::new(Vec::new()),
            cleanups: GcCell::new(Vec::new()),
        })
    }

    /// Create a new component with a globally unique ID (for use in slots)
    pub fn with_global_id(component_type: ComponentType, props: ComponentProps) -> Gc<Self> {
        let id = next_component_id();
        Self::new(id, component_type, props)
    }

    /// Mark the component as dirty (needs re-render)
    pub fn mark_dirty(&self) {
        self.is_dirty.store(true, Ordering::SeqCst);
        // Clear vello cache when dirty
        *self.vello_cache.borrow_mut() = None;
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
        // Prevent infinite recursion from component adding itself
        // This can happen if Component::clone() is used instead of Gc::clone()
        if std::ptr::eq(&*child, self) {
            // Silently ignore self-addition to prevent infinite recursion
            return;
        }
        self.children.borrow_mut().push(Gc::clone(&child));
    }

    /// Remove a child component
    pub fn remove_child(&self, child: &Gc<Component>) {
        let mut children = self.children.borrow_mut();
        children.retain(|c| !Gc::ptr_eq(c, child));
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

    pub fn accepts_pointer_interaction(&self) -> bool {
        self.flags.borrow().contains(ComponentFlags::ACCEPTS_POINTER)
    }

    pub fn accepts_focus(&self) -> bool {
        self.flags.borrow().contains(ComponentFlags::ACCEPTS_FOCUS)
    }

    pub fn is_disabled(&self) -> bool {
        self.flags.borrow().contains(ComponentFlags::IS_DISABLED)
    }

    pub fn is_stashed(&self) -> bool {
        self.flags.borrow().contains(ComponentFlags::IS_STASHED)
    }

    pub fn on_status_update(&self, update: &StatusUpdate) {
        match update {
            StatusUpdate::Mounted => {
                self.mount(self.parent.borrow().clone());
            }
            StatusUpdate::Unmounting => {
                self.unmount();
            }
            StatusUpdate::HoveredChanged(hovered) => {
                *self.is_hovered.borrow_mut() = *hovered;
                self.mark_dirty();
            }
            StatusUpdate::ActiveChanged(active) => {
                *self.is_active.borrow_mut() = *active;
                self.mark_dirty();
            }
            StatusUpdate::FocusChanged(focused) => {
                *self.is_focused.borrow_mut() = *focused;
                self.mark_dirty();
            }
            StatusUpdate::DisabledChanged(disabled) => {
                let mut flags = self.flags.borrow_mut();
                if *disabled {
                    flags.insert(ComponentFlags::IS_DISABLED);
                } else {
                    flags.remove(ComponentFlags::IS_DISABLED);
                }
                self.mark_dirty();
            }
            _ => {}
        }
    }

    // Property-specific setters for fine-grained updates

    /// Set text content (for Text components)
    pub fn set_text_content(&self, content: String) {
        let styles = {
            if let ComponentProps::Text { styles, .. } = &*self.props.borrow() {
                styles.clone()
            } else {
                return;
            }
        };
        *self.props.borrow_mut() = ComponentProps::Text { content, styles };
        self.mark_dirty();
    }

    /// Set button label (for Button components)
    pub fn set_button_label(&self, label: String) {
        if matches!(self.component_type, ComponentType::Button) {
            let styles = {
                if let ComponentProps::Button { styles, .. } = &*self.props.borrow() {
                    styles.clone()
                } else {
                    return;
                }
            };
            *self.props.borrow_mut() = ComponentProps::Button { label, styles };
            self.mark_dirty();
        }
    }

    /// Set flex direction (for Flex components)
    pub fn set_flex_direction(&self, direction: String) {
        let (gap, align_items, justify_content, styles) = {
            if let ComponentProps::Flex { gap, align_items, justify_content, styles, .. } =
                &*self.props.borrow()
            {
                (*gap, align_items.clone(), justify_content.clone(), styles.clone())
            } else {
                return;
            }
        };
        *self.props.borrow_mut() =
            ComponentProps::Flex { direction, gap, align_items, justify_content, styles };
        self.mark_dirty();
    }

    /// Set flex gap (for Flex components)
    pub fn set_flex_gap(&self, gap: f32) {
        let (direction, align_items, justify_content, styles) = {
            if let ComponentProps::Flex {
                direction, align_items, justify_content, styles, ..
            } = &*self.props.borrow()
            {
                (direction.clone(), align_items.clone(), justify_content.clone(), styles.clone())
            } else {
                return;
            }
        };
        *self.props.borrow_mut() =
            ComponentProps::Flex { direction, gap, align_items, justify_content, styles };
        self.mark_dirty();
    }

    /// Set flex align items (for Flex components)
    pub fn set_flex_align_items(&self, align_items: String) {
        let (direction, gap, justify_content, styles) = {
            if let ComponentProps::Flex { direction, gap, justify_content, styles, .. } =
                &*self.props.borrow()
            {
                (direction.clone(), *gap, justify_content.clone(), styles.clone())
            } else {
                return;
            }
        };
        *self.props.borrow_mut() =
            ComponentProps::Flex { direction, gap, align_items, justify_content, styles };
        self.mark_dirty();
    }

    /// Set flex justify content (for Flex components)
    pub fn set_flex_justify_content(&self, justify_content: String) {
        let (direction, gap, align_items, styles) = {
            if let ComponentProps::Flex { direction, gap, align_items, styles, .. } =
                &*self.props.borrow()
            {
                (direction.clone(), *gap, align_items.clone(), styles.clone())
            } else {
                return;
            }
        };
        *self.props.borrow_mut() =
            ComponentProps::Flex { direction, gap, align_items, justify_content, styles };
        self.mark_dirty();
    }

    /// Set checkbox checked state (for Checkbox components)
    pub fn set_checkbox_checked(&self, checked: bool) {
        if matches!(self.component_type, ComponentType::Checkbox) {
            let styles = {
                if let ComponentProps::Checkbox { styles, .. } = &*self.props.borrow() {
                    styles.clone()
                } else {
                    return;
                }
            };
            *self.props.borrow_mut() = ComponentProps::Checkbox { checked, styles };
            self.mark_dirty();
        }
    }

    /// Set radio checked state (for Radio components)
    pub fn set_radio_checked(&self, checked: bool) {
        if let ComponentProps::Radio { value, styles, .. } = &*self.props.borrow() {
            *self.props.borrow_mut() =
                ComponentProps::Radio { value: value.clone(), checked, styles: styles.clone() };
            self.mark_dirty();
        }
    }

    /// Set radio value (for Radio components)
    pub fn set_radio_value(&self, value: String) {
        if let ComponentProps::Radio { checked, styles, .. } = &*self.props.borrow() {
            *self.props.borrow_mut() =
                ComponentProps::Radio { value, checked: *checked, styles: styles.clone() };
            self.mark_dirty();
        }
    }

    /// Set text input value (for TextInput components)
    pub fn set_text_input_value(&self, value: String) {
        if matches!(self.component_type, ComponentType::TextInput) {
            let styles = {
                if let ComponentProps::TextInput { styles, .. } = &*self.props.borrow() {
                    styles.clone()
                } else {
                    return;
                }
            };
            *self.props.borrow_mut() = ComponentProps::TextInput { value, styles };
            self.mark_dirty();
        }
    }

    /// Set number input value (for NumberInput components)
    pub fn set_number_input_value(&self, value: f64) {
        if matches!(self.component_type, ComponentType::NumberInput) {
            let styles = {
                if let ComponentProps::NumberInput { styles, .. } = &*self.props.borrow() {
                    styles.clone()
                } else {
                    return;
                }
            };
            *self.props.borrow_mut() = ComponentProps::NumberInput { value, styles };
            self.mark_dirty();
        }
    }

    /// Set show condition (for Show components)
    pub fn set_show_when(&self, when: bool) {
        if matches!(self.component_type, ComponentType::Show) {
            *self.props.borrow_mut() = ComponentProps::Show { when };
            self.mark_dirty();
        }
    }

    /// Set for item count (for For components)
    pub fn set_for_item_count(&self, item_count: usize) {
        if matches!(self.component_type, ComponentType::For) {
            *self.props.borrow_mut() = ComponentProps::For { item_count };
            self.mark_dirty();
        }
    }

    pub fn on_click_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_0arg(handler);
        self.event_handlers.borrow_mut().on_click = Some(handler);
        self.flags.borrow_mut().insert(ComponentFlags::ACCEPTS_POINTER);
    }

    pub fn on_click_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent) + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_1arg(handler);
        self.event_handlers.borrow_mut().on_click = Some(handler);
        self.flags.borrow_mut().insert(ComponentFlags::ACCEPTS_POINTER);
    }

    pub fn on_click<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new(
                handler,
            );
        self.event_handlers.borrow_mut().on_click = Some(handler);
        self.flags.borrow_mut().insert(ComponentFlags::ACCEPTS_POINTER);
    }

    pub fn on_pointer_down_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_0arg(handler);
        self.event_handlers.borrow_mut().on_pointer_down = Some(handler);
    }

    pub fn on_pointer_down_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent) + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_1arg(handler);
        self.event_handlers.borrow_mut().on_pointer_down = Some(handler);
    }

    pub fn on_pointer_down<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_down = Some(handler);
    }

    pub fn on_pointer_up_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_0arg(handler);
        self.event_handlers.borrow_mut().on_pointer_up = Some(handler);
    }

    pub fn on_pointer_up_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent) + 'static,
    {
        let handler = crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new_1arg(handler);
        self.event_handlers.borrow_mut().on_pointer_up = Some(handler);
    }

    pub fn on_pointer_up<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerButtonEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerButtonEvent>::new(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_up = Some(handler);
    }

    pub fn on_pointer_enter_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_enter = Some(handler);
    }

    pub fn on_pointer_enter_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerInfo) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_enter = Some(handler);
    }

    pub fn on_pointer_enter<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerInfo, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new(handler);
        self.event_handlers.borrow_mut().on_pointer_enter = Some(handler);
    }

    pub fn on_pointer_leave_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_leave = Some(handler);
    }

    pub fn on_pointer_leave_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerInfo) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_leave = Some(handler);
    }

    pub fn on_pointer_leave<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerInfo, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerInfo>::new(handler);
        self.event_handlers.borrow_mut().on_pointer_leave = Some(handler);
    }

    pub fn on_pointer_move_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerMoveEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_move = Some(handler);
    }

    pub fn on_pointer_move_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerMoveEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerMoveEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_move = Some(handler);
    }

    pub fn on_pointer_move<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::PointerMoveEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::PointerMoveEvent>::new(
                handler,
            );
        self.event_handlers.borrow_mut().on_pointer_move = Some(handler);
    }

    pub fn on_key_down_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_key_down = Some(handler);
    }

    pub fn on_key_down_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::KeyboardEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_key_down = Some(handler);
    }

    pub fn on_key_down<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::KeyboardEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new(handler);
        self.event_handlers.borrow_mut().on_key_down = Some(handler);
    }

    pub fn on_key_up_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_key_up = Some(handler);
    }

    pub fn on_key_up_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::KeyboardEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_key_up = Some(handler);
    }

    pub fn on_key_up<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::types::KeyboardEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::types::KeyboardEvent>::new(handler);
        self.event_handlers.borrow_mut().on_key_up = Some(handler);
    }

    pub fn on_focus_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_focus = Some(handler);
    }

    pub fn on_focus_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::FocusEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_focus = Some(handler);
    }

    pub fn on_focus<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::FocusEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new(handler);
        self.event_handlers.borrow_mut().on_focus = Some(handler);
    }

    pub fn on_blur_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_blur = Some(handler);
    }

    pub fn on_blur_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::FocusEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_blur = Some(handler);
    }

    pub fn on_blur<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::FocusEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::FocusEvent>::new(handler);
        self.event_handlers.borrow_mut().on_blur = Some(handler);
    }

    pub fn on_input_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_input = Some(handler);
    }

    pub fn on_input_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::InputEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_input = Some(handler);
    }

    pub fn on_input<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::InputEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new(handler);
        self.event_handlers.borrow_mut().on_input = Some(handler);
    }

    pub fn on_change_0arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn() + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new_0arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_change = Some(handler);
    }

    pub fn on_change_1arg<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::InputEvent) + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new_1arg(
                handler,
            );
        self.event_handlers.borrow_mut().on_change = Some(handler);
    }

    pub fn on_change<F>(self: &Gc<Self>, handler: F)
    where
        F: Fn(&crate::event::status::InputEvent, &mut crate::event::context::EventContext)
            + 'static,
    {
        let handler =
            crate::event::handler::EventHandler::<crate::event::status::InputEvent>::new(handler);
        self.event_handlers.borrow_mut().on_change = Some(handler);
    }

    /// Provide context to this component and its descendants
    pub fn provide_context<T>(&self, value: T)
    where
        T: Clone + 'static,
        T: Trace,
    {
        let type_id = TypeId::of::<T>();
        let context_value = ContextValueEnum::from_value(value);
        self.contexts.borrow_mut().push(ContextEntry { type_id, value: context_value });
    }

    /// Find context of type T in this component or its ancestors
    pub fn find_context<T>(&self) -> Option<Gc<T>>
    where
        T: 'static,
        T: Trace + Clone,
    {
        let type_id = TypeId::of::<T>();
        let contexts = self.contexts.borrow();
        for entry in contexts.iter().rev() {
            if entry.type_id == type_id {
                if let Some(value) = entry.value.to_gc::<T>() {
                    return Some(value);
                }
            }
        }

        if let Some(parent) = self.parent.borrow().as_ref() {
            return parent.find_context::<T>();
        }

        None
    }
}

/// Collect Taffy node IDs from child layouts, including grandchildren for control-flow components
fn collect_child_node_ids(
    component: &Gc<Component>,
    child_layouts: &[LayoutNode],
) -> Vec<taffy::NodeId> {
    let mut node_ids = Vec::new();

    for (child, child_layout) in component.children.borrow().iter().zip(child_layouts.iter()) {
        if let Some(node_id) = child_layout.taffy_node() {
            // Direct child has a layout node - include it
            node_ids.push(node_id);
        } else if matches!(child.component_type, ComponentType::For | ComponentType::Show) {
            // Control-flow component - include its children's nodes
            for grandchild in child.children.borrow().iter() {
                if let Some(grandchild_layout) = grandchild.layout_node() {
                    if let Some(node_id) = grandchild_layout.taffy_node() {
                        node_ids.push(node_id);
                    }
                }
            }
        }
    }

    node_ids
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

    // Control-flow components (For, Show) are transparent - their children's
    // Taffy nodes should be passed through to the parent, not wrapped
    let is_control_flow =
        matches!(component.component_type, ComponentType::For | ComponentType::Show);

    // Get Taffy node IDs from child layouts, including grandchildren for control-flow
    let child_node_ids = collect_child_node_ids(component, &child_layouts);

    // For control-flow components, return a transparent node
    if is_control_flow {
        for (child, child_layout) in component.children.borrow().iter().zip(child_layouts.iter()) {
            child.set_layout_node(child_layout.clone());
            child.set_parent(Some(Gc::clone(component)));
        }

        return LayoutNode { taffy_node: None, is_dirty: true, layout_result: None };
    }

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

        // Run cleanups
        let cleanups = {
            let mut cleanups = self.cleanups.borrow_mut();
            std::mem::take(&mut *cleanups)
        };
        for cleanup in cleanups {
            cleanup();
        }
    }

    fn update(&self) {
        // Use atomic operation to detect cycles
        let is_updating = self.is_updating.swap(true, Ordering::SeqCst);
        if is_updating {
            // Cycle detected - skip this update to prevent infinite recursion
            return;
        }

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

        self.is_updating.store(false, Ordering::SeqCst);
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
