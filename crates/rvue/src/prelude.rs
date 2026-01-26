//! Prelude module - re-exports commonly used types

pub use crate::component::{
    Component, ComponentId, ComponentLifecycle, ComponentProps, ComponentType,
};
pub use crate::effect::{create_effect, on_cleanup, untracked, Effect};
pub use crate::signal::{
    create_memo, create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal,
};
pub use crate::slot::{Children, ChildrenFn, MaybeChildren, ToChildren};
pub use crate::style::{
    AlignItems, Border, BorderStyle, Color, FlexDirection, FontWeight, JustifyContent, Size,
    Spacing, Style,
};
pub use crate::view::{View, ViewStruct};
pub use crate::widget::{IntoReactiveValue, IntoWidget, ReactiveValue};
