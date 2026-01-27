//! Properties container using an enum-based approach.

use crate::properties::{Color, Height, Margin, Padding, Size, Width};
use std::collections::HashMap;

/// All possible style properties as an enum.
/// This allows for simple storage and retrieval without dynamic dispatch.
#[derive(Clone, Debug, PartialEq)]
pub enum Property {
    Color(Color),
    Padding(Padding),
    Margin(Margin),
    Width(Width),
    Height(Height),
}

impl Property {
    /// Returns the type name of this property.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Color(_) => "color",
            Self::Padding(_) => "padding",
            Self::Margin(_) => "margin",
            Self::Width(_) => "width",
            Self::Height(_) => "height",
        }
    }
}

/// A collection of style properties for a widget.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Properties {
    map: HashMap<&'static str, Property>,
}

impl Properties {
    /// Creates a new empty properties container.
    #[inline]
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Creates a properties container with a single property.
    #[inline]
    pub fn one(property: Property) -> Self {
        let mut map = HashMap::new();
        map.insert(property.type_name(), property.clone());
        Self { map }
    }

    /// Inserts a property.
    #[inline]
    pub fn insert(&mut self, property: Property) {
        self.map.insert(property.type_name(), property);
    }

    /// Returns a reference to the color property, if it exists.
    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.map.get("color").and_then(|p| match p {
            Property::Color(c) => Some(c),
            _ => None,
        })
    }

    /// Returns a reference to the padding property, if it exists.
    #[inline]
    pub fn padding(&self) -> Option<&Padding> {
        self.map.get("padding").and_then(|p| match p {
            Property::Padding(p) => Some(p),
            _ => None,
        })
    }

    /// Returns a reference to the margin property, if it exists.
    #[inline]
    pub fn margin(&self) -> Option<&Margin> {
        self.map.get("margin").and_then(|p| match p {
            Property::Margin(m) => Some(m),
            _ => None,
        })
    }

    /// Returns a reference to the width property, if it exists.
    #[inline]
    pub fn width(&self) -> Option<&Width> {
        self.map.get("width").and_then(|p| match p {
            Property::Width(w) => Some(w),
            _ => None,
        })
    }

    /// Returns a reference to the height property, if it exists.
    #[inline]
    pub fn height(&self) -> Option<&Height> {
        self.map.get("height").and_then(|p| match p {
            Property::Height(h) => Some(h),
            _ => None,
        })
    }

    /// Returns the number of properties in this container.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if this container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl From<Property> for Properties {
    #[inline]
    fn from(prop: Property) -> Self {
        Self::one(prop)
    }
}
