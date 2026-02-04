//! Property map for storing widget properties.
//!
//! This module provides a type-safe property container that can hold
//! any type implementing `WidgetProperty`. It uses type erasure
//! internally to store heterogeneous property types.

use rudo_gc::{GcCell, Trace, Visitor};
use std::any::{Any, TypeId};

use super::WidgetProperty;

trait DynClone: Any {
    fn clone_box(&self) -> Box<dyn DynClone>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Clone + 'static + Send + Sync> DynClone for T {
    fn clone_box(&self) -> Box<dyn DynClone> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct DynProperty {
    type_id: TypeId,
    value: Box<dyn DynClone>,
}

impl DynProperty {
    fn new<P>(value: P) -> Self
    where
        P: WidgetProperty,
    {
        Self { type_id: TypeId::of::<P>(), value: Box::new(value) }
    }

    fn downcast<P: WidgetProperty>(&self) -> Option<&P> {
        self.value.as_any().downcast_ref::<P>()
    }

    fn downcast_mut<P: WidgetProperty>(&mut self) -> Option<&mut P> {
        self.value.as_any_mut().downcast_mut::<P>()
    }
}

impl Clone for DynProperty {
    fn clone(&self) -> Self {
        Self { type_id: self.type_id, value: self.value.clone_box() }
    }
}

unsafe impl Trace for DynProperty {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Default, Clone)]
pub struct PropertyMap {
    map: std::collections::HashMap<TypeId, DynProperty>,
}

impl PropertyMap {
    #[inline]
    pub fn new() -> Self {
        Self { map: std::collections::HashMap::new() }
    }

    #[inline]
    pub fn with<P>(value: P) -> Self
    where
        P: WidgetProperty,
    {
        let mut map = std::collections::HashMap::new();
        map.insert(TypeId::of::<P>(), DynProperty::new(value));
        Self { map }
    }

    #[inline]
    pub fn get<P: WidgetProperty>(&self) -> Option<&P> {
        self.map.get(&TypeId::of::<P>()).and_then(|p| p.downcast::<P>())
    }

    #[inline]
    pub fn get_or_default<P: WidgetProperty>(&self) -> &P {
        self.get::<P>().unwrap_or_else(|| P::static_default())
    }

    #[inline]
    pub fn get_mut<P: WidgetProperty>(&mut self) -> Option<&mut P> {
        self.map.get_mut(&TypeId::of::<P>()).and_then(|p| p.downcast_mut::<P>())
    }

    #[inline]
    pub fn insert<P>(&mut self, value: P) -> Option<P>
    where
        P: WidgetProperty,
    {
        let old = self.map.insert(TypeId::of::<P>(), DynProperty::new(value));
        old.and_then(|dyn_prop| dyn_prop.downcast::<P>().cloned())
    }

    #[inline]
    pub fn remove<P: WidgetProperty>(&mut self) -> Option<P> {
        self.map.remove(&TypeId::of::<P>()).and_then(|dyn_prop| dyn_prop.downcast::<P>().cloned())
    }

    #[inline]
    pub fn contains<P: WidgetProperty>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<P>())
    }

    #[inline]
    pub fn and<P>(mut self, value: P) -> Self
    where
        P: WidgetProperty,
    {
        self.insert(value);
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clone(&self) -> Self {
        let mut new_map = std::collections::HashMap::new();
        for (type_id, prop) in &self.map {
            new_map.insert(*type_id, prop.clone());
        }
        Self { map: new_map }
    }
}

unsafe impl Trace for PropertyMap {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

impl std::fmt::Debug for PropertyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyMap").field("len", &self.map.len()).finish_non_exhaustive()
    }
}

impl<P: WidgetProperty> From<P> for PropertyMap {
    fn from(prop: P) -> Self {
        Self::with(prop)
    }
}

#[derive(Default, Clone)]
pub struct GcPropertyMap(pub GcCell<PropertyMap>);

impl GcPropertyMap {
    #[inline]
    pub fn new() -> Self {
        Self(GcCell::new(PropertyMap::new()))
    }

    #[inline]
    pub fn with<P>(value: P) -> Self
    where
        P: WidgetProperty,
    {
        Self(GcCell::new(PropertyMap::with(value)))
    }

    #[inline]
    pub fn borrow(&self) -> std::cell::Ref<'_, PropertyMap> {
        self.0.borrow()
    }

    #[inline]
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, PropertyMap> {
        self.0.borrow_mut()
    }

    #[inline]
    pub fn get<P: WidgetProperty>(&self) -> Option<std::cell::Ref<'_, P>> {
        let map = self.0.borrow();
        if map.contains::<P>() {
            Some(std::cell::Ref::map(map, |m| m.get::<P>().unwrap()))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_or_default<P: WidgetProperty>(&self) -> std::cell::Ref<'_, P> {
        std::cell::Ref::map(self.0.borrow(), |m| m.get_or_default::<P>())
    }
}

impl<P: WidgetProperty> From<P> for GcPropertyMap {
    fn from(prop: P) -> Self {
        Self::with(prop)
    }
}
