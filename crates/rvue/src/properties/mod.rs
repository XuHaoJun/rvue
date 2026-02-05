//! Widget properties system.
//!
//! This module provides a trait-based property system for widgets,
//! inspired by Xilem's approach. Properties are type-safe values
//! that can be attached to any widget.

pub mod map;
pub mod traits;
pub mod types;

pub use map::{GcPropertyMap, PropertyMap};
pub use traits::WidgetProperty;
pub use types::*;

/// Default properties for global theming support.
///
/// This module provides infrastructure for setting default property values
/// that apply globally to all widgets of a given type.
pub mod defaults {
    use once_cell::sync::Lazy;
    use std::sync::RwLock;

    macro_rules! define_default_storage {
        ($name:ident, $type:ty) => {
            static $name: Lazy<RwLock<Option<$type>>> = Lazy::new(|| RwLock::new(None));
        };
    }

    define_default_storage!(DEFAULT_TEXT_CONTENT, String);
    define_default_storage!(DEFAULT_FLEX_DIRECTION, String);
    define_default_storage!(DEFAULT_FLEX_GAP, f32);
    define_default_storage!(DEFAULT_FLEX_ALIGN_ITEMS, String);
    define_default_storage!(DEFAULT_FLEX_JUSTIFY_CONTENT, String);

    /// Set a default property value globally
    #[inline]
    pub fn set_default_text_content<S: Into<String>>(value: S) {
        *DEFAULT_TEXT_CONTENT.write().unwrap() = Some(value.into());
    }

    /// Get the default text content, if set
    #[inline]
    pub fn get_default_text_content() -> Option<String> {
        DEFAULT_TEXT_CONTENT.read().unwrap().clone()
    }

    /// Set a default flex direction globally
    #[inline]
    pub fn set_default_flex_direction<S: Into<String>>(value: S) {
        *DEFAULT_FLEX_DIRECTION.write().unwrap() = Some(value.into());
    }

    /// Get the default flex direction, if set
    #[inline]
    pub fn get_default_flex_direction() -> Option<String> {
        DEFAULT_FLEX_DIRECTION.read().unwrap().clone()
    }

    /// Set a default flex gap globally
    #[inline]
    pub fn set_default_flex_gap(value: f32) {
        *DEFAULT_FLEX_GAP.write().unwrap() = Some(value);
    }

    /// Get the default flex gap, if set
    #[inline]
    pub fn get_default_flex_gap() -> Option<f32> {
        *DEFAULT_FLEX_GAP.read().unwrap()
    }

    /// Set a default flex align-items globally
    #[inline]
    pub fn set_default_flex_align_items<S: Into<String>>(value: S) {
        *DEFAULT_FLEX_ALIGN_ITEMS.write().unwrap() = Some(value.into());
    }

    /// Get the default flex align-items, if set
    #[inline]
    pub fn get_default_flex_align_items() -> Option<String> {
        DEFAULT_FLEX_ALIGN_ITEMS.read().unwrap().clone()
    }

    /// Set a default flex justify-content globally
    #[inline]
    pub fn set_default_flex_justify_content<S: Into<String>>(value: S) {
        *DEFAULT_FLEX_JUSTIFY_CONTENT.write().unwrap() = Some(value.into());
    }

    /// Get the default flex justify-content, if set
    #[inline]
    pub fn get_default_flex_justify_content() -> Option<String> {
        DEFAULT_FLEX_JUSTIFY_CONTENT.read().unwrap().clone()
    }
}
