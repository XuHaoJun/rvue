//! Rvue - Rust GUI framework with Vue-like syntax and GC-managed memory
//!
//! Performance optimizations:
//! - Lazy renderer initialization
//! - Optimized component tree creation
//! - Efficient memory allocation patterns

#![feature(arbitrary_self_types)]

pub mod app;
pub mod component;
pub mod effect;
pub mod error;
pub mod event;
pub mod layout;
pub mod prelude;
pub mod render;
pub mod signal;
pub mod style;
pub mod text;
pub mod vello_util;
pub mod view;
pub mod widget;
pub mod widgets;

pub use app::{run_app, AppError};
pub use component::{Component, ComponentId, ComponentLifecycle, ComponentProps, ComponentType};
pub use effect::{create_effect, Effect};
pub use error::{
    validate_email, validate_number_input, validate_text_input, ValidationError, ValidationResult,
};
pub use render::{Scene, VelloFragment};
pub use signal::{create_memo, create_signal, ReadSignal, SignalRead, SignalWrite, WriteSignal};
pub use style::{
    AlignItems, Border, BorderStyle, Color, FlexDirection, FontWeight, JustifyContent, Size,
    Spacing, Style,
};
pub use taffy::TaffyTree;
pub use text::TextContext;
pub use view::{View, ViewStruct};
pub use widget::{
    BuildContext, IntoReactiveValue, IntoWidget, Mountable, ReactiveValue, Widget, WidgetWrapper,
};
pub use widgets::{Button, Checkbox, Flex, For, NumberInput, Radio, Show, Text, TextInput};
