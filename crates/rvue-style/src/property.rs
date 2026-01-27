//! Property trait and Properties container.

use rudo_gc::{Trace, Visitor};
use std::any::{Any, TypeId};

/// A property that can be styled on a widget.
pub trait Property: Default + Clone + Send + Sync + 'static {}

#[derive(Debug)]
struct DynProperty {
    type_id: TypeId,
    value: Box<dyn Any>,
}

impl DynProperty {
    fn new<P: Property>(value: P) -> Self {
        Self { type_id: TypeId::of::<P>(), value: Box::new(value) }
    }

    fn downcast<P: Property>(&self) -> Option<&P> {
        if self.type_id == TypeId::of::<P>() {
            self.value.downcast_ref()
        } else {
            None
        }
    }

    fn downcast_mut<P: Property>(&mut self) -> Option<&mut P> {
        if self.type_id == TypeId::of::<P>() {
            self.value.downcast_mut()
        } else {
            None
        }
    }
}

unsafe impl Trace for DynProperty {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Default, Debug)]
pub struct Properties {
    map: std::collections::HashMap<TypeId, DynProperty>,
}

impl Properties {
    #[inline]
    pub fn new() -> Self {
        Self { map: std::collections::HashMap::new() }
    }

    #[inline]
    pub fn with<P: Property>(value: P) -> Self {
        let mut map = std::collections::HashMap::new();
        map.insert(TypeId::of::<P>(), DynProperty::new(value));
        Self { map }
    }

    #[inline]
    pub fn get<P: Property>(&self) -> Option<&P> {
        self.map.get(&TypeId::of::<P>()).and_then(|p| p.downcast::<P>())
    }

    #[inline]
    pub fn get_mut<P: Property>(&mut self) -> Option<&mut P> {
        self.map.get_mut(&TypeId::of::<P>()).and_then(|p| p.downcast_mut::<P>())
    }

    #[inline]
    pub fn insert<P: Property>(&mut self, value: P) {
        self.map.insert(TypeId::of::<P>(), DynProperty::new(value));
    }

    #[inline]
    pub fn remove<P: Property>(&mut self) {
        self.map.remove(&TypeId::of::<P>());
    }

    #[inline]
    pub fn contains<P: Property>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<P>())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

unsafe impl Trace for Properties {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

impl<P: Property> From<P> for Properties {
    #[inline]
    fn from(prop: P) -> Self {
        Self::with(prop)
    }
}
