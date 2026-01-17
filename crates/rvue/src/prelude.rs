//! Prelude module - re-exports commonly used types

pub use crate::signal::{create_signal, ReadSignal, WriteSignal, SignalRead, SignalWrite};
pub use crate::effect::{create_effect, Effect};
pub use crate::component::{Component, ComponentType, ComponentProps, ComponentLifecycle, ComponentId};
pub use crate::view::{View, ViewStruct};
pub use crate::style::{Style, Color, Spacing, Size, Border, BorderStyle, FontWeight, FlexDirection, AlignItems, JustifyContent};
