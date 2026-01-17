//! Rvue - Rust GUI framework with Vue-like syntax and GC-managed memory

pub mod prelude;
pub mod signal;
pub mod effect;
pub mod component;
pub mod view;
pub mod style;
pub mod widgets;
pub mod layout;
pub mod render;
pub mod app;

pub use signal::{create_signal, ReadSignal, WriteSignal, SignalRead, SignalWrite};
pub use effect::{create_effect, Effect};
pub use component::{Component, ComponentType, ComponentProps, ComponentLifecycle, ComponentId};
pub use view::{View, ViewStruct};
pub use style::{Style, Color, Spacing, Size, Border, BorderStyle, FontWeight, FlexDirection, AlignItems, JustifyContent};
pub use widgets::{Text, Button};
pub use app::{run_app, AppError};
pub use render::{Scene, VelloFragment};
