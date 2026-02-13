//! Rvue - Rust GUI framework with Vue-like syntax and GC-managed memory
//!
//! Performance optimizations:
//! - Lazy renderer initialization
//! - Optimized component tree creation
//! - Efficient memory allocation patterns

#![feature(arbitrary_self_types)]

pub mod app;
pub mod component;
pub mod context;
pub mod effect;
pub mod error;
pub mod ev;
pub mod event;
pub mod gc;
pub mod layout;
pub mod prelude;
pub mod properties;
pub mod reactivity;
pub mod render;
pub mod runtime;
pub mod signal;
pub mod slot;
pub mod style;
pub mod text;
pub mod vello_util;
pub mod view;
pub mod widget;
pub mod widgets;

#[cfg(feature = "async")]
pub mod async_runtime;

#[cfg(feature = "async")]
pub mod headless {
    pub use crate::async_runtime::dispatch::{dispatch_to_ui, UiDispatchQueue};
    pub use crate::async_runtime::get_or_init_runtime;
    pub use crate::effect::flush_pending_effects;

    /// Initialize the async runtime for testing
    pub fn init_runtime() {
        let _ = get_or_init_runtime();
    }

    /// Drain all pending UI callbacks
    pub fn drain_ui_queue() {
        UiDispatchQueue::drain_all_and_execute();
    }

    /// Run all pending effects
    pub fn run_effects() {
        flush_pending_effects();
    }

    /// Advance the async system by draining all queues
    pub fn advance() {
        drain_ui_queue();
        run_effects();
    }

    /// Run the async system until no more work is pending
    pub fn settle() {
        loop {
            let before = UiDispatchQueue::len();
            advance();
            let after = UiDispatchQueue::len();
            if before == 0 && after == 0 {
                break;
            }
        }
    }

    /// Run async operations until the resource reaches a specific state
    pub fn wait_for_resource_state<T, S, F>(
        resource: &crate::async_runtime::Resource<T, S>,
        max_iterations: u32,
        check: F,
    ) -> bool
    where
        T: rudo_gc::Trace + Clone + 'static,
        S: rudo_gc::Trace + Clone + 'static,
        F: Fn(&crate::async_runtime::ResourceState<T>) -> bool,
    {
        for _ in 0..max_iterations {
            let state = resource.get();
            if check(&state) {
                return true;
            }
            advance();
        }
        false
    }
}

pub use app::{run_app, run_app_with_stylesheet, AppError};
pub use component::{Component, ComponentId, ComponentLifecycle, ComponentType};
pub use effect::{
    create_effect, flush_pending_effects, on_cleanup, set_defer_effect_run, untracked, Effect,
};
pub use error::{
    validate_email, validate_number_input, validate_text_input, ValidationError, ValidationResult,
};
pub use event::ScrollDragState;
pub use gc::impl_gc_capture;
pub use properties::{
    CheckboxChecked, FlexAlignItems, FlexDirection, FlexGap, FlexJustifyContent, ForItemCount,
    GcPropertyMap, NumberInputValue, PropertyMap, RadioChecked, RadioValue, ShowCondition,
    TextContent, TextInputValue, WidgetProperty, WidgetStyles,
};
pub use render::{render_component, FlexScrollState, Scene};
pub use rudo_gc::handles::HandleScope;
pub use rudo_gc::Gc;
pub use rvue_style::Overflow;
pub use signal::{
    create_memo, create_memo_with_equality, create_signal, ReadSignal, SignalRead, SignalWrite,
    WriteSignal,
};
pub use style::{Stylesheet, StylesheetProvider};
pub use taffy::TaffyTree;
pub use text::TextContext;
pub use view::{View, ViewStruct};
pub use widget::{
    get_current_ctx, with_current_ctx, BuildContext, IntoReactiveValue, IntoWidget, Mountable,
    ReactiveValue, Widget, WidgetWrapper,
};
pub use widgets::{Button, Checkbox, Flex, For, NumberInput, Radio, Show, Text, TextInput};
