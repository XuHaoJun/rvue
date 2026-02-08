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
    create_resource, dispatch_to_ui, spawn_debounced, spawn_interval, spawn_task,
    spawn_task_with_result, spawn_watch_signal, DebouncedTask, Resource, ResourceState,
    SignalSender, SignalWatcher, TaskHandle, TaskId,
};

#[cfg(feature = "async")]
pub use crate::prelude::write_signal_ext::WriteSignalExt;

#[cfg(feature = "async")]
mod write_signal_ext {
    use rudo_gc::Trace;

    use super::{SignalSender, WriteSignal};

    pub trait WriteSignalExt<T: Trace + Clone + 'static> {
        fn sender(&self) -> SignalSender<T>
        where
            T: Send;
    }

    impl<T: Trace + Clone + 'static> WriteSignalExt<T> for WriteSignal<T>
    where
        T: Send,
    {
        fn sender(&self) -> SignalSender<T> {
            let setter = self.clone();
            SignalSender::new(move |value: T| {
                setter.set(value);
            })
        }
    }
}
