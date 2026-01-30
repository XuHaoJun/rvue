//! Property trait and Properties container.

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize,
    FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Padding, Size, TextColor, Visibility,
    Width, ZIndex,
};
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

pub trait StyleStore {
    fn background_color(&self) -> Option<&BackgroundColor>;
    fn color(&self) -> Option<&TextColor>;
    fn padding(&self) -> Option<&Padding>;
    fn margin(&self) -> Option<&Margin>;
    fn font_size(&self) -> Option<&FontSize>;
    fn font_family(&self) -> Option<&FontFamily>;
    fn font_weight(&self) -> Option<&FontWeight>;
    fn width(&self) -> Option<&Width>;
    fn height(&self) -> Option<&Height>;
    fn display(&self) -> Option<&Display>;
    fn flex_direction(&self) -> Option<&FlexDirection>;
    fn justify_content(&self) -> Option<&JustifyContent>;
    fn align_items(&self) -> Option<&AlignItems>;
    fn align_self(&self) -> Option<&AlignSelf>;
    fn flex_grow(&self) -> Option<&FlexGrow>;
    fn flex_shrink(&self) -> Option<&FlexShrink>;
    fn flex_basis(&self) -> Option<&FlexBasis>;
    fn border_color(&self) -> Option<&BorderColor>;
    fn border_width(&self) -> Option<&BorderWidth>;
    fn border_radius(&self) -> Option<&BorderRadius>;
    fn border_style(&self) -> Option<&BorderStyle>;
    fn opacity(&self) -> Option<&Opacity>;
    fn visibility(&self) -> Option<&Visibility>;
    fn cursor(&self) -> Option<&Cursor>;
    fn z_index(&self) -> Option<&ZIndex>;
    fn gap(&self) -> Option<&Gap>;
    fn size(&self) -> Option<&Size>;
}

impl StyleStore for Properties {
    fn background_color(&self) -> Option<&BackgroundColor> {
        self.get()
    }

    fn color(&self) -> Option<&TextColor> {
        self.get()
    }

    fn padding(&self) -> Option<&Padding> {
        self.get()
    }

    fn margin(&self) -> Option<&Margin> {
        self.get()
    }

    fn font_size(&self) -> Option<&FontSize> {
        self.get()
    }

    fn font_family(&self) -> Option<&FontFamily> {
        self.get()
    }

    fn font_weight(&self) -> Option<&FontWeight> {
        self.get()
    }

    fn width(&self) -> Option<&Width> {
        self.get()
    }

    fn height(&self) -> Option<&Height> {
        self.get()
    }

    fn display(&self) -> Option<&Display> {
        self.get()
    }

    fn flex_direction(&self) -> Option<&FlexDirection> {
        self.get()
    }

    fn justify_content(&self) -> Option<&JustifyContent> {
        self.get()
    }

    fn align_items(&self) -> Option<&AlignItems> {
        self.get()
    }

    fn align_self(&self) -> Option<&AlignSelf> {
        self.get()
    }

    fn flex_grow(&self) -> Option<&FlexGrow> {
        self.get()
    }

    fn flex_shrink(&self) -> Option<&FlexShrink> {
        self.get()
    }

    fn flex_basis(&self) -> Option<&FlexBasis> {
        self.get()
    }

    fn border_color(&self) -> Option<&BorderColor> {
        self.get()
    }

    fn border_width(&self) -> Option<&BorderWidth> {
        self.get()
    }

    fn border_radius(&self) -> Option<&BorderRadius> {
        self.get()
    }

    fn border_style(&self) -> Option<&BorderStyle> {
        self.get()
    }

    fn opacity(&self) -> Option<&Opacity> {
        self.get()
    }

    fn visibility(&self) -> Option<&Visibility> {
        self.get()
    }

    fn cursor(&self) -> Option<&Cursor> {
        self.get()
    }

    fn z_index(&self) -> Option<&ZIndex> {
        self.get()
    }

    fn gap(&self) -> Option<&Gap> {
        self.get()
    }

    fn size(&self) -> Option<&Size> {
        self.get()
    }
}

impl From<Properties> for crate::widget::styled::StyleData {
    fn from(props: Properties) -> Self {
        crate::widget::styled::StyleData {
            background_color: props.get().cloned(),
            color: props.get().cloned(),
            padding: props.get().cloned(),
            margin: props.get().cloned(),
            font_size: props.get().cloned(),
            font_family: props.get().cloned(),
            font_weight: props.get().cloned(),
            width: props.get().cloned(),
            height: props.get().cloned(),
            display: props.get().cloned(),
            flex_direction: props.get().cloned(),
            justify_content: props.get().cloned(),
            align_items: props.get().cloned(),
            align_self: props.get().cloned(),
            flex_grow: props.get().cloned(),
            flex_shrink: props.get().cloned(),
            flex_basis: props.get().cloned(),
            border_color: props.get().cloned(),
            border_width: props.get().cloned(),
            border_radius: props.get().cloned(),
            border_style: props.get().cloned(),
            opacity: props.get().cloned(),
            visibility: props.get().cloned(),
            cursor: props.get().cloned(),
            z_index: props.get().cloned(),
            gap: props.get().cloned(),
            size: props.get().cloned(),
        }
    }
}

impl<P: Property> From<P> for Properties {
    #[inline]
    fn from(prop: P) -> Self {
        Self::with(prop)
    }
}
