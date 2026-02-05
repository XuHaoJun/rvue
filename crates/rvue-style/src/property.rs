//! Property trait and Properties container.

use crate::properties::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, BorderRadius, BorderStyle, BorderWidth,
    Cursor, Display, FlexBasis, FlexDirection, FlexGrow, FlexShrink, FontFamily, FontSize,
    FontWeight, Gap, Height, JustifyContent, Margin, Opacity, Padding, Size, TextColor, Visibility,
    Width, ZIndex,
};
use rudo_gc::{Trace, Visitor};
use std::alloc::{self, Layout};
use std::any::TypeId;
use std::collections::HashMap;

/// A property that can be styled on a widget.
///
/// Properties represent CSS styles that can be applied to widgets.
/// Each property has a CSS initial value that is used when the property
/// is not explicitly set, following CSS cascade semantics.
pub trait Property: Default + Clone + Send + Sync + 'static {
    /// Returns the CSS initial value for this property.
    ///
    /// This is used when a property is not explicitly set, following
    /// CSS semantics where unspecified properties use their initial values.
    fn initial_value() -> Self;
}

struct DynProperty {
    type_id: TypeId,
    ptr: *mut u8,        // Pointer to aligned memory containing the value
    layout: Layout,      // Memory layout (size and alignment)
    drop_fn: unsafe fn(*mut u8, Layout), // Function to drop the value stored in bytes
    clone_fn: unsafe fn(*const u8, Layout) -> *mut u8, // Function to clone the value stored in bytes
}

impl DynProperty {
    fn new<P>(value: P) -> Self
    where
        P: Property,
    {
        let layout = Layout::new::<P>();
        unsafe {
            // Allocate properly aligned memory
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            
            // Use ManuallyDrop to prevent the original value from being dropped
            // when it goes out of scope. The value is moved into the allocated memory
            // and will be dropped when DynProperty is dropped via the drop_fn.
            let value = std::mem::ManuallyDrop::new(value);
            std::ptr::write(ptr as *mut P, std::ptr::read(&*value));
            
            Self {
                type_id: TypeId::of::<P>(),
                ptr,
                layout,
                drop_fn: Self::drop_value::<P>,
                clone_fn: Self::clone_value::<P>,
            }
        }
    }

    /// Drop function for a specific type P
    unsafe fn drop_value<P>(ptr: *mut u8, layout: Layout) {
        // Drop the value
        std::ptr::drop_in_place(ptr as *mut P);
        // Deallocate the memory
        alloc::dealloc(ptr, layout);
    }

    /// Clone function for a specific type P
    unsafe fn clone_value<P: Clone>(ptr: *const u8, layout: Layout) -> *mut u8 {
        let value = &*(ptr as *const P);
        let cloned = value.clone();
        // Allocate aligned memory for the clone
        let new_ptr = alloc::alloc(layout);
        if new_ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        std::ptr::write(new_ptr as *mut P, cloned);
        new_ptr
    }

    fn downcast<P: Property>(&self) -> Option<&P> {
        if self.type_id == TypeId::of::<P>()
            && self.layout.size() == std::mem::size_of::<P>()
            && self.layout.align() == std::mem::align_of::<P>()
        {
            unsafe { Some(&*(self.ptr as *const P)) }
        } else {
            None
        }
    }

    fn downcast_mut<P: Property>(&mut self) -> Option<&mut P> {
        if self.type_id == TypeId::of::<P>()
            && self.layout.size() == std::mem::size_of::<P>()
            && self.layout.align() == std::mem::align_of::<P>()
        {
            unsafe { Some(&mut *(self.ptr as *mut P)) }
        } else {
            None
        }
    }
}

impl Clone for DynProperty {
    fn clone(&self) -> Self {
        // Use the clone function to properly clone the value stored in memory
        let new_ptr = unsafe { (self.clone_fn)(self.ptr, self.layout) };
        Self {
            type_id: self.type_id,
            ptr: new_ptr,
            layout: self.layout,
            drop_fn: self.drop_fn,
            clone_fn: self.clone_fn,
        }
    }
}

impl Drop for DynProperty {
    fn drop(&mut self) {
        unsafe {
            // Call the drop function to properly drop the value and deallocate memory
            (self.drop_fn)(self.ptr, self.layout);
        }
    }
}

unsafe impl Trace for DynProperty {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

#[derive(Default, Clone)]
pub struct Properties {
    map: HashMap<TypeId, DynProperty>,
}

impl Properties {
    #[inline]
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    #[inline]
    pub fn with<P>(value: P) -> Self
    where
        P: Property,
    {
        let mut map = HashMap::new();
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
    pub fn insert<P>(&mut self, value: P)
    where
        P: Property,
    {
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

    #[inline]
    pub fn clone(&self) -> Self {
        let mut new_map = HashMap::new();
        for (type_id, prop) in &self.map {
            new_map.insert(*type_id, prop.clone());
        }
        Self { map: new_map }
    }
}

unsafe impl Trace for Properties {
    fn trace(&self, _visitor: &mut impl Visitor) {}
}

impl std::fmt::Debug for Properties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Properties").field("len", &self.map.len()).finish_non_exhaustive()
    }
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

impl<P: Property> From<P> for Properties
where
    P: Clone,
{
    #[inline]
    fn from(prop: P) -> Self {
        Self::with(prop)
    }
}
