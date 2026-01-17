//! Prelude module - re-exports commonly used types

pub use crate::component::{
    Component, ComponentId, ComponentLifecycle, ComponentProps, ComponentType,
};
pub use crate::effect::{create_effect, Effect};
pub use crate::signal::{create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal};
pub use crate::style::{
    AlignItems, Border, BorderStyle, Color, FlexDirection, FontWeight, JustifyContent, Size,
    Spacing, Style,
};
pub use crate::view::{View, ViewStruct};
