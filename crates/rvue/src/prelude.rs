//! Prelude module - re-exports commonly used types

pub use crate::component::{Component, ComponentId, ComponentLifecycle, ComponentType};
pub use crate::effect::{create_effect, on_cleanup, untracked, Effect};
pub use crate::ev::{
    Blur, Change, Click, Focus, Input, KeyDown, KeyUp, PointerDown, PointerMove, PointerUp,
};
pub use crate::event::{EventContext, EventDescriptor, EventHandler};
pub use crate::signal::{
    create_memo, create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal,
};
pub use crate::slot::{Children, ChildrenFn, MaybeChildren, ToChildren};
pub use crate::view::{View, ViewStruct};
pub use crate::widget::{IntoReactiveValue, IntoWidget, ReactiveValue};
pub use rvue_style::{
    AlignItems, BackgroundColor, BorderColor, BorderRadius, BorderStyle, Color, FlexDirection,
    FontWeight, Gap, JustifyContent, Margin, Padding, TextColor,
};

/// Event descriptors module (Leptos-style)
/// Use as: `component.on(ev::Click, |e| { ... })`
pub mod ev {
    pub use super::Blur;
    pub use super::Change;
    pub use super::Click;
    pub use super::Focus;
    pub use super::Input;
    pub use super::KeyDown;
    pub use super::KeyUp;
    pub use super::PointerDown;
    pub use super::PointerMove;
    pub use super::PointerUp;
}

#[cfg(feature = "async")]
pub use crate::async_runtime::{
    spawn_debounced, spawn_interval, spawn_task, watch_signal, ComponentScope, DebouncedTask,
    IntervalHandle, Resource, ResourceState, SignalWatcher, TaskHandle, TaskId, UiThreadDispatcher,
    WriteSignalUiExt,
};
